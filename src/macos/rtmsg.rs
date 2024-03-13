use std::net::IpAddr;

use libc::{
    in6_addr, in_addr, rt_msghdr, sockaddr, sockaddr_dl, sockaddr_in, sockaddr_in6, AF_INET, AF_INET6, AF_LINK, RTM_VERSION
};


#[repr(C)]
#[derive(Clone, Copy)]
#[allow(non_camel_case_types)]
pub(super) struct m_rtmsg {
    pub hdr: rt_msghdr,
    pub attr: [i8; 2048],
    pub attr_len: usize,
}

impl Default for m_rtmsg {
    fn default() -> Self {
        let mut rtmsg = unsafe { std::mem::zeroed::<m_rtmsg>() };
        rtmsg.hdr.rtm_version = RTM_VERSION as u8;

        rtmsg
    }
}

macro_rules! roundup {
    ($a:expr) => {{
        let size = std::mem::size_of::<u32>();
        let val = if $a > 0 { 1 + (($a - 1) | (size - 1)) } else { size };
        val
    }};
}

impl m_rtmsg {
    pub fn new_buf() -> [u8; std::mem::size_of::<m_rtmsg>()] {
        [0u8; std::mem::size_of::<m_rtmsg>()]
    }

    pub fn len(&self) -> usize {
        std::mem::size_of::<rt_msghdr>() + self.attr_len
    }

    fn put_addr(&mut self, addr: &IpAddr) {
        match addr {
            IpAddr::V4(addr) => {
                let sa_len = std::mem::size_of::<sockaddr_in>();
                let sa_in =
                    unsafe { &mut *(self.attr[self.attr_len..].as_mut_ptr() as *mut sockaddr_in) };
                sa_in.sin_len = sa_len as u8;
                sa_in.sin_family = AF_INET as u8;
                sa_in.sin_port = 0;
                sa_in.sin_addr = in_addr {
                    s_addr: unsafe { std::mem::transmute(addr.octets()) },
                };

                self.attr_len += sa_len;
            }
            IpAddr::V6(addr) => {
                let sa_len = std::mem::size_of::<sockaddr_in6>();
                let sa_in6 =
                    unsafe { &mut *(self.attr[self.attr_len..].as_mut_ptr() as *mut sockaddr_in6) };
                sa_in6.sin6_len = sa_len as u8;
                sa_in6.sin6_family = AF_INET6 as u8;
                sa_in6.sin6_port = 0;
                sa_in6.sin6_flowinfo = 0;
                sa_in6.sin6_addr = in6_addr {
                    s6_addr: unsafe { std::mem::transmute(addr.octets()) },
                };
                sa_in6.sin6_scope_id = 0;

                self.attr_len += sa_len;
            }
        }
    }

    pub fn put_destination(&mut self, dest: &IpAddr) {
        self.put_addr(dest);
    }

    pub fn put_gateway(&mut self, gateway: &IpAddr) {
        self.put_addr(&gateway)
    }

    pub fn put_index(&mut self, ifindex: u32) {
        let sdl_len = std::mem::size_of::<sockaddr_dl>();
        let sa_dl = unsafe { &mut *(self.attr[self.attr_len..].as_mut_ptr() as *mut sockaddr_dl) };
        sa_dl.sdl_len = sdl_len as u8;
        sa_dl.sdl_family = AF_LINK as u8;
        sa_dl.sdl_index = ifindex as u16;

        self.attr_len += sdl_len;
    }

    pub fn put_netmask(&mut self, mask: &IpAddr) {
        self.put_addr(&mask)
    }

    pub fn get_addr(&mut self) -> IpAddr {
        let sa = unsafe {
            &*(self.attr[self.attr_len..].as_ptr() as *const sockaddr)
        };

        if sa.sa_family == AF_INET as u8 {
            let sa_in: &sockaddr_in = unsafe { std::mem::transmute(sa) };

            self.attr_len += roundup!(sa_in.sin_len as usize);

            return IpAddr::from(sa_in.sin_addr.s_addr.to_ne_bytes());
        } else {
            let sa_in6: &sockaddr_in6 = unsafe { std::mem::transmute(sa) };

            self.attr_len += roundup!(sa_in6.sin6_len as usize);

            return IpAddr::from(sa_in6.sin6_addr.s6_addr);
        }
    }

    pub fn get_destination(&mut self) -> IpAddr {
        self.get_addr()
    }

    pub fn get_gateway(&mut self) -> IpAddr {
        self.get_addr()
    }

    pub fn get_netmask(&mut self, family: u8) -> IpAddr {
        let sa= unsafe {
            &mut *(self.attr[self.attr_len..].as_ptr() as *mut sockaddr)
        };
        sa.sa_family = family;

        self.get_addr()
    }

    pub fn get_index(&mut self) -> u32 {
        let sa_dl = unsafe {
            &mut *(self.attr[self.attr_len..].as_ptr() as *mut sockaddr_dl)
        };
        self.attr_len += roundup!(sa_dl.sdl_len as usize);

        sa_dl.sdl_index as u32
    }
}