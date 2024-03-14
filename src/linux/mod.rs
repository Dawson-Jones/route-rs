use std::{io, os::fd::RawFd};

use libc::{AF_ROUTE, AF_UNSPEC, SOCK_RAW};

use crate::syscall;

pub struct RouteSock(RawFd);


impl RouteSock {
    pub fn new() -> io::Result<Self> {
        let fd = syscall!(
            socket(AF_ROUTE, SOCK_RAW, AF_UNSPEC)
        )?;

        Ok(Self(fd))
    }
}

impl Drop for RouteSock {
    fn drop(&mut self) {
        syscall!(close(self.0)).unwrap();
    }
}