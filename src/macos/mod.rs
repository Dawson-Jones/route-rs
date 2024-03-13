mod rtmsg;

use std::{
    ffi::CString, io::{self, Read, Write}, os::fd::{AsRawFd, RawFd}
};

use crate::{macos::rtmsg::m_rtmsg, Route, RouteAction};
use libc::{
    rt_msghdr, AF_INET, AF_INET6, AF_ROUTE, AF_UNSPEC, RTA_BRD, RTA_DST, RTA_GATEWAY, RTA_IFP, RTA_NETMASK, RTF_GATEWAY, RTF_STATIC, RTF_UP, RTM_ADD, RTM_DELETE, RTM_GET, RTM_VERSION, SOCK_RAW
};

#[macro_export]
macro_rules! syscall {
    ($fn: ident ( $($arg: expr),* ) ) => {{
        #[allow(unused_unsafe)]
        let res = unsafe { libc::$fn($( $arg), *) };
        if res < 0 {
            Err(std::io::Error::last_os_error())
        } else {
            Ok(res)
        }
    }};
}

pub struct RouteSock(RawFd);

impl AsRawFd for RouteSock {
    fn as_raw_fd(&self) -> RawFd {
        self.0
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

impl RouteAction for RouteSock {
    fn add(&mut self, route: &Route) -> io::Result<()> {
        let mut rtm_flags = (RTF_STATIC | RTF_UP) as i32;

        if route.gateway.is_some() {
            rtm_flags |= RTF_GATEWAY as i32;
        };

        let rtm_addrs = (RTA_DST | RTA_NETMASK | RTA_GATEWAY) as i32;

        let mut rtmsg = m_rtmsg::default();
        rtmsg.hdr.rtm_type = RTM_ADD as u8;
        rtmsg.hdr.rtm_flags = rtm_flags;
        rtmsg.hdr.rtm_addrs = rtm_addrs;
        rtmsg.hdr.rtm_seq = 1;

        rtmsg.put_destination(&route.destination);
        if let Some(gateway) = route.gateway {
            rtmsg.put_gateway(&gateway);
        }
        if let Some(ifindex) = route.ifindex {
            rtmsg.put_index(ifindex);
        }
        rtmsg.put_netmask(&route.mask());

        rtmsg.hdr.rtm_msglen = rtmsg.len() as u16;

        let slice = {
            let ptr = &rtmsg as *const m_rtmsg as *const u8;
            let len = rtmsg.hdr.rtm_msglen as usize;

            unsafe { std::slice::from_raw_parts(ptr, len) }
        };

        self.write(slice)?;

        let mut buf = [0; std::mem::size_of::<m_rtmsg>()];
        let n = self.read(&mut buf)?;
        if n < std::mem::size_of::<rt_msghdr>() {
            return Err(io::Error::new(io::ErrorKind::Other, "invalid response"));
        }

        let rt_hdr: &rt_msghdr = unsafe { std::mem::transmute(buf.as_ptr()) };

        assert_eq!(rt_hdr.rtm_type, RTM_ADD as u8);
        assert_eq!(rt_hdr.rtm_version, RTM_VERSION as u8);
        if rt_hdr.rtm_errno != 0 {
            return Err(code2error(rt_hdr.rtm_errno));
        }

        Ok(())
    }

    fn delete(&mut self, route: &Route) -> io::Result<()> {
        let rtm_flags = (RTF_STATIC | RTF_UP | RTF_GATEWAY) as i32;
        let rtm_addrs = (RTA_DST | RTA_NETMASK) as i32;

        let mut rtmsg = m_rtmsg::default();
        rtmsg.hdr.rtm_type = RTM_DELETE as u8;
        rtmsg.hdr.rtm_flags = rtm_flags;
        rtmsg.hdr.rtm_addrs = rtm_addrs;
        rtmsg.hdr.rtm_seq = 1;

        rtmsg.put_destination(&route.destination);
        rtmsg.put_netmask(&route.mask());

        rtmsg.hdr.rtm_msglen = rtmsg.len() as u16;

        let slice = {
            let ptr = &rtmsg as *const m_rtmsg as *const u8;
            let len = rtmsg.hdr.rtm_msglen as usize;

            unsafe { std::slice::from_raw_parts(ptr, len) }
        };

        self.write(slice)?;

        let mut buf = [0; std::mem::size_of::<m_rtmsg>()];
        let n = self.read(&mut buf)?;
        if n < std::mem::size_of::<rt_msghdr>() {
            return Err(io::Error::new(io::ErrorKind::Other, "invalid response"));
        }

        let rt_hdr: &rt_msghdr = unsafe { std::mem::transmute(buf.as_ptr()) };

        assert_eq!(rt_hdr.rtm_version, RTM_VERSION as u8);
        if rt_hdr.rtm_errno != 0 {
            return Err(code2error(rt_hdr.rtm_errno));
        }

        Ok(())
    }

    fn get(&mut self, route: &Route) -> io::Result<Route> {
        let mut ret = Route::default();
        let rtm_flags = (RTF_STATIC | RTF_UP | RTF_GATEWAY) as i32;
        let rtm_addrs = (RTA_DST | RTA_NETMASK | RTA_IFP) as i32;
        // if !route.destination.is_unspecified() {
        //     rtm_flags |= RTF_HOST as i32;
        // }
        let mut rtmsg = m_rtmsg::default();
        rtmsg.hdr.rtm_type = RTM_GET as u8;
        rtmsg.hdr.rtm_flags = rtm_flags;
        rtmsg.hdr.rtm_addrs = rtm_addrs;
        rtmsg.hdr.rtm_seq = 1;

        rtmsg.put_destination(&route.destination);
        rtmsg.put_netmask(&route.mask());
        rtmsg.put_index(0);

        rtmsg.hdr.rtm_msglen = rtmsg.len() as u16;

        let slice = {
            let ptr = &rtmsg as *const m_rtmsg as *const u8;
            let len = rtmsg.hdr.rtm_msglen as usize;

            unsafe { std::slice::from_raw_parts(ptr, len) }
        };

        println!("write: {} bytes", slice.len());
        println!("{:x?}", slice);
        self.write(slice)?;
        println!("flags: {}", rtmsg.hdr.rtm_flags);

        let mut buf = [0; std::mem::size_of::<m_rtmsg>()];
        let n = self.read(&mut buf)?;
        if n < std::mem::size_of::<rt_msghdr>() {
            return Err(io::Error::new(io::ErrorKind::Other, "invalid response"));
        }

        let rtmsg: &mut m_rtmsg = unsafe { std::mem::transmute(buf.as_ptr()) };
        // rtmsg.attr_len = n - std::mem::size_of::<rt_msghdr>();
        println!("{n} size readed...");
        println!("hdr");
        println!("{:x?}", &buf[0..std::mem::size_of::<rt_msghdr>()]);
        println!("attr");
        println!("{:x?}", &buf[std::mem::size_of::<rt_msghdr>()..n]);

        assert_eq!(rtmsg.hdr.rtm_version, RTM_VERSION as u8);
        if rtmsg.hdr.rtm_errno != 0 {
            return Err(code2error(rtmsg.hdr.rtm_errno));
        }

        if rtmsg.hdr.rtm_msglen > n as u16 {
            return Err(io::Error::new(
                io::ErrorKind::Other, 
                format!("message length mismatch, in packet {}, returned {}", rtmsg.hdr.rtm_msglen, n)
            ));
        }

        let mut bit = 1;
        // RTA_BRD is max bitmask value of rtm_addrs
        while bit <= RTA_BRD && rtmsg.attr_len + std::mem::size_of::<rt_msghdr>() < n {
            if rtmsg.hdr.rtm_addrs & bit != 0 {
                match bit {
                    RTA_DST => {
                        ret.destination = rtmsg.get_destination()
                    },
                    RTA_GATEWAY => ret.gateway = {
                        let gateway = rtmsg.get_gateway();
                        if rtmsg.hdr.rtm_flags & RTF_GATEWAY != 0 {
                            Some(gateway)
                        } else {
                            None
                        }
                    },
                    RTA_NETMASK => ret.cidr(rtmsg.get_netmask(
                        if route.destination.is_ipv4() {AF_INET as u8} else {AF_INET6 as u8})),
                    RTA_IFP => ret.ifindex = Some(rtmsg.get_index()),
                    _ => (),
                }
            }

            bit <<= 1;
        }

        Ok(ret)
    }
}

impl RouteSock {
    pub fn new() -> io::Result<Self> {
        let fd = syscall!(
            socket(AF_ROUTE as i32, SOCK_RAW as i32, AF_UNSPEC as i32)
        )?;

        Ok(Self(fd))
    }
}

fn code2error(err: i32) -> io::Error {
    let kind = match err {
        17 => io::ErrorKind::AlreadyExists, // EEXIST
        3 => io::ErrorKind::NotFound,       // ESRCH
        3436 => io::ErrorKind::OutOfMemory, // ENOBUFS
        _ => io::ErrorKind::Other,
    };

    io::Error::new(kind, format!("rtm_errno {}", err))
}

pub fn if_nametoindex(name: &str) -> Option<u32> {
    let name = CString::new(name).ok()?;
    let ifindex = unsafe { libc::if_nametoindex(name.as_ptr()) };

    Some(ifindex)
}
