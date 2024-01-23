use core::arch::asm;
use core::ffi::CStr;

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
pub struct Stat {
    pub st_dev: DevT,
    pub st_ino: InoT,
    pub st_nlink: NlinkT,
    pub st_mode: ModeT,
    pub st_uid: UidT,
    pub st_gid: GidT,
    pub __pad0: Cint,
    pub st_rdev: DevT,
    pub st_size: OffT,
    pub st_blksize: BlkSizeT,
    pub st_blocks: BlkcntT,
    pub st_atime: TimeT,
    pub st_atime_nsec: i64,
    pub st_mtime: TimeT,
    pub st_mtime_nsec: i64,
    pub st_ctime: TimeT,
    pub st_ctime_nsec: i64,
    pub __unused: [i64; 3],
}

pub fn fstat(fd: i64) -> Stat {
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

pub fn open(path: &str) -> i64 {
    let path = CStr::from_bytes_until_nul(path.as_bytes()).unwrap();
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

pub fn close(fd: i64) -> i64 {
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

pub fn mmap(fd: i64, stat: &Stat) -> u64 {
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

pub fn munmap(addr: u64, stat: &Stat) -> i64 {
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

pub fn writeln(line: &str) {
    unsafe {
        let lineptr = line.as_bytes().as_ptr();
        let linelen = line.len();
        asm!(
            "syscall",
            in("rax") 1,
            in("rdi") 1, // stdout
            in("rsi") lineptr,
            in("rdx") linelen,
        );

        let nl = "\n".as_bytes().as_ptr();
        asm!(
            "syscall",
            in("rax") 1,
            in("rdi") 1, // stdout
            in("rsi") nl,
            in("rdx") 1,
        );
    }
}
