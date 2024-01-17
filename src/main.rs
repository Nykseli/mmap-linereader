use std::arch::asm;
use std::ffi::CString;
use std::slice;

// Yoinked from https://docs.rs/libc/latest/src/libc/unix/linux_like/linux/gnu/b64/x86_64/mod.rs.html
type DevT = u64;
type InoT = u64;
type NlinkT = u64;
type ModeT = u32;
type UidT = u32;
type GidT = u32;
type Cint = i32;
type OffT = i64;
type BlkSizeT = i64;
type BlkcntT = i64;
type TimeT = i64;

#[repr(C)]
#[derive(Debug, Default)]
struct Stat {
    st_dev: DevT,
    st_ino: InoT,
    st_nlink: NlinkT,
    st_mode: ModeT,
    st_uid: UidT,
    st_gid: GidT,
    __pad0: Cint,
    st_rdev: DevT,
    st_size: OffT,
    st_blksize: BlkSizeT,
    st_blocks: BlkcntT,
    st_atime: TimeT,
    st_atime_nsec: i64,
    st_mtime: TimeT,
    st_mtime_nsec: i64,
    st_ctime: TimeT,
    st_ctime_nsec: i64,
    __unused: [i64; 3],
}

fn fstat(fd: i64) -> Stat {
    let stat = Stat::default();

    unsafe {
        // TODO: error handling
        let sref = &stat;
        asm!(
            "mov rax, 5",
            "syscall",
            in("rdi") fd,
            in("rsi") sref,
        );
    }

    stat
}

fn open(path: &str) -> i64 {
    let path = CString::new(path).unwrap();
    let mut fd: i64;

    unsafe {
        let pptr = path.as_ptr();
        asm!(
            "mov rax, 2",
            "syscall",
            in("rdi") pptr,
            in("rsi") 0, // readonly
            in("rdx") 0, // it's not created so this can be 0
            lateout("rax") fd
        );
    }

    // TODO: if fd < 0 return error
    fd
}

fn close(fd: i64) -> i64 {
    let mut ret: i64;

    unsafe {
        asm!(
            "mov rax, 3",
            "syscall",
            in("rdi") fd,
            lateout("rax") ret
        );
    }

    ret
}

fn mmap(fd: i64, stat: &Stat) -> u64 {
    let mut ret: u64;

    unsafe {
        asm!(
            "mov rax, 9",
            "syscall",
            in("rdi") 0, // NULL
            in("rsi") stat.st_size,
            in("rdx") 1, // PROT_READ
            in("r10") 2, // MAP_PRIVATE
            in ("r8") fd,
            in ("r9") 0, // off
            lateout("rax") ret
        );
    }

    ret
}

fn munmap(addr: u64, stat: &Stat) -> i64 {
    let mut ret: i64;

    unsafe {
        asm!(
            "mov rax, 11",
            "syscall",
            in("rdi") addr, // NULL
            in("rsi") stat.st_size,
            lateout("rax") ret
        );
    }

    ret
}

struct MReader {
    fd: i64,
    stat: Stat,
    maddr: u64,
}

impl MReader {
    fn new(path: &str) -> Self {
        let fd = open(path);
        let stat = fstat(fd);
        let maddr = mmap(fd, &stat);

        Self { fd, stat, maddr }
    }
}

impl Drop for MReader {
    fn drop(&mut self) {
        close(self.fd);
        munmap(self.maddr, &self.stat);
    }
}

struct MReaderIter<'a> {
    data: &'a str,
    idx: usize,
}

impl<'a> IntoIterator for &'a MReader {
    type Item = &'a str;
    type IntoIter = MReaderIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        MReaderIter {
            data: unsafe {
                std::str::from_utf8_unchecked(slice::from_raw_parts(
                    self.maddr as *const u8,
                    self.stat.st_size as usize,
                ))
            },
            idx: 0,
        }
    }
}

impl<'a> Iterator for MReaderIter<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        let data = &self.data;
        let end_idx = data[self.idx..].find('\n');
        match end_idx {
            Some(idx) => {
                let ret = &data[self.idx..self.idx + idx];
                self.idx += idx + 1;
                return Some(ret);
            }
            None => {
                self.idx = 0;
                return None;
            }
        }
    }
}

fn main() {
    let lines = MReader::new("Cargo.toml");
    for line in lines.into_iter() {
        println!("{}", line);
    }
}
