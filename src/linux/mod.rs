use std::{io::{self, Read, Write}, os::fd::{AsRawFd, RawFd}};

use libc::{AF_ROUTE, AF_UNSPEC, SOCK_RAW};

use crate::{syscall, RouteAction};

pub struct RouteSock(RawFd);

impl AsRawFd for RouteSock {
    fn as_raw_fd(&self) -> RawFd {
        self.0
    }
}

impl RouteSock {
    pub fn new() -> io::Result<Self> {
        let fd = syscall!(
            socket(AF_ROUTE, SOCK_RAW, AF_UNSPEC)
        )?;

        Ok(Self(fd))
    }
}

impl Write for RouteSock {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let n = syscall!(write(self.as_raw_fd(), buf.as_ptr() as *const _, buf.len()))?;

        Ok(n as usize)
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl Read for RouteSock {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let n = syscall!(read(
            self.as_raw_fd(),
            buf.as_mut_ptr() as *mut _,
            buf.len()
        ))?;

        Ok(n as usize)
    }
}

impl Drop for RouteSock {
    fn drop(&mut self) {
        syscall!(close(self.0)).unwrap();
    }
}

impl RouteAction for RouteSock {
    fn add(&mut self, route: &crate::Route) -> io::Result<()> {
        todo!()
    }

    fn delete(&mut self, route: &crate::Route) -> io::Result<()> {
        todo!()
    }

    fn get(&mut self, route: &crate::Route) -> io::Result<crate::Route> {
        todo!()
    }

    fn monitor(&mut self, buf: &mut [u8]) -> io::Result<(crate::RouteChange, crate::Route)> {
        todo!()
    }
}