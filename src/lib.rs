use std::{
    ffi::CString,
    io,
    net::IpAddr,
};

#[cfg(all(target_os = "macos"))]
mod macos;
#[cfg(all(target_os = "linux"))]
mod linux;

#[cfg(all(target_os = "linux"))]
pub use linux::RouteSock;
#[cfg(all(target_os = "macos"))]
pub use macos::RouteSock;

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

#[derive(Debug)]
pub struct Route {
    pub destination: IpAddr,
    pub prefix: u8,
    pub gateway: Option<IpAddr>,
    pub ifindex: Option<u32>,
}

impl Default for Route {
    fn default() -> Self {
        use std::net::Ipv4Addr;

        Route {
            destination: IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)),
            prefix: 0,
            gateway: None,
            ifindex: None,
        }
    }
}

impl Route {
    pub fn new(destination: IpAddr, prefix: u8) -> Route {
        Route {
            destination,
            prefix,
            gateway: None,
            ifindex: None,
        }
    }

    #[cfg(target_os = "macos")]
    pub(crate) fn mask(&self) -> IpAddr {
        use std::net::{Ipv4Addr, Ipv6Addr};

        match self.destination {
            IpAddr::V4(_) => IpAddr::V4(Ipv4Addr::from(
                u32::MAX.checked_shl(32 - self.prefix as u32).unwrap_or(0),
            )),
            IpAddr::V6(_) => IpAddr::V6(Ipv6Addr::from(
                u128::MAX.checked_shl(128 - self.prefix as u32).unwrap_or(0),
            )),
        }
    }

    #[cfg(target_os = "macos")]
    pub(crate) fn cidr(&mut self, netmask: IpAddr) {
        use std::net::{Ipv4Addr, Ipv6Addr};

        self.prefix = match netmask {
            IpAddr::V4(netmask) => <Ipv4Addr as Into<u32>>::into(netmask).leading_ones() as u8,
            IpAddr::V6(netmask) => <Ipv6Addr as Into<u128>>::into(netmask).leading_ones() as u8,
        }
    }

    pub fn gateway(mut self, gateway: IpAddr) -> Route {
        self.gateway = Some(gateway);
        self
    }

    pub fn ifindex(mut self, ifindex: u32) -> Route {
        self.ifindex = Some(ifindex);
        self
    }

    pub fn interface(mut self, interface: &str) -> Route {
        self.ifindex = if_nametoindex(interface);
        self
    }
}

#[derive(Debug)]
pub enum RouteChange {
    ADD,
    DELETE,
    CHANGE,
    OTHER(u8),
}

impl From<u8> for RouteChange {
    fn from(value: u8) -> Self {
        match value {
            1 => RouteChange::ADD,
            2 => RouteChange::DELETE,
            3 => RouteChange::CHANGE,
            _ => RouteChange::OTHER(value),
        }
    }
}

pub trait RouteAction {
    fn add(&mut self, route: &Route) -> io::Result<()>;
    fn delete(&mut self, route: &Route) -> io::Result<()>;
    fn get(&mut self, route: &Route) -> io::Result<Route>;
    fn monitor(&mut self, buf: &mut [u8]) -> io::Result<(RouteChange, Route)>;
}

pub fn if_nametoindex(name: &str) -> Option<u32> {
    let name = CString::new(name).ok()?;
    let ifindex = unsafe { libc::if_nametoindex(name.as_ptr()) };

    Some(ifindex)
}
