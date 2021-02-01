/// TODO
use std::os::unix::io::{AsRawFd, RawFd};
use std::ptr;

use io_uring::{squeue, IoUring};
use io_uring::opcode::{self, types};

#[derive(Clone, Debug)]
enum Token {
    Accept,
    Poll {
        fd: RawFd,
    },
    Read {
        fd: RawFd,
        buf_index: usize,
    },
    Write {
        fd: RawFd,
        buf_index: usize,
        offset: usize,
        len: usize,
    },
}

pub struct AcceptCount {
    entry: squeue::Entry,
    count: usize,
}

impl AcceptCount {
    fn new(fd: RawFd, token: usize, count: usize) -> AcceptCount {
        AcceptCount {
            entry: opcode::Accept::new(types::Fd(fd), ptr::null_mut(), ptr::null_mut())
                .build()
                .user_data(token as _),
            count,
        }
    }

    pub fn push_to(&mut self, sq: &mut squeue::AvailableQueue) {
        while self.count > 0 {
            unsafe {
                match sq.push(self.entry.clone()) {
                    Ok(_) => self.count -= 1,
                    Err(_) => break,
                }
            }
        }
    }
}

/// io_uring builder?
pub fn new_io_uring() -> {

}
