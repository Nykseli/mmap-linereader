use core::slice;
use core::str::from_utf8_unchecked;

use crate::asm::{close, fstat, mmap, munmap, open, Stat};

pub struct MReader {
    fd: i64,
    stat: Stat,
    maddr: u64,
}

impl MReader {
    pub fn new(path: &str) -> Self {
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

pub struct MReaderIter<'a> {
    data: &'a str,
    idx: usize,
}

impl<'a> IntoIterator for &'a MReader {
    type Item = &'a str;
    type IntoIter = MReaderIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        MReaderIter {
            data: unsafe {
                from_utf8_unchecked(slice::from_raw_parts(
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
                Some(ret)
            }
            None => {
                self.idx = 0;
                None
            }
        }
    }
}
