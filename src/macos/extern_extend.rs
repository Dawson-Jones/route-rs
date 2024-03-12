use std::net::IpAddr;

use super::extern_c::*;

#[repr(C)]
#[derive(Clone, Copy)]
#[allow(non_camel_case_types)]
pub(super) struct m_rtmsg {
    pub hdr: rt_msghdr,
    pub attr: [i8; 512],
    pub attr_len: usize,
}

impl Default for m_rtmsg {
    fn default() -> Self {
        let mut rtmsg = unsafe { std::mem::zeroed::<m_rtmsg>() };
        rtmsg.hdr.rtm_version = RTM_VERSION as u8;

        rtmsg
    }
}

impl m_rtmsg {
    pub fn len(&self) -> usize {
        std::mem::size_of::<m_rtmsg>() + self.attr_len
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
                    __u6_addr: unsafe { std::mem::transmute(addr.octets()) },
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
}

impl Default for rt_metrics {
    fn default() -> Self {
        unsafe { std::mem::zeroed() }
    }
}

impl Default for sockaddr_dl {
    fn default() -> Self {
        unsafe { std::mem::zeroed() }
    }
}
