use std::{io, net::{IpAddr, Ipv4Addr, Ipv6Addr}};

mod macos;

#[cfg(all(target_os = "macos"))]
pub use macos::{
    RouteSock,
    if_nametoindex,
};


pub struct Route {
    pub destination: IpAddr,
    pub prefix: u8,
    pub gateway: Option<IpAddr>,
    pub ifindex: Option<u32>,
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

    pub(crate) fn mask(&self) -> IpAddr {
        match self.destination {
            IpAddr::V4(_) => IpAddr::V4(Ipv4Addr::from(
                u32::MAX.checked_shl(32 - self.prefix as u32).unwrap_or(0),
            )),
            IpAddr::V6(_) => IpAddr::V6(Ipv6Addr::from(
                u128::MAX.checked_shl(128 - self.prefix as u32).unwrap_or(0),
            )),
        }
    }

    #[cfg(target_os = "linux")]
    pub fn via(mut self, gateway: IpAddr) -> Route {
        self.gateway = Some(gateway);
        self
    }

    pub fn gateway(mut self, gateway: IpAddr) -> Route {
        self.gateway = Some(gateway);
        self
    }

    pub fn ifindex(mut self, ifindex: u32) -> Route {
        self.ifindex = Some(ifindex);
        self
    }

    #[cfg(target_os = "macos")]
    pub fn interface(mut self, interface: &str) -> Route {
        self.ifindex = if_nametoindex(interface);
        self
    }

    #[cfg(target_os = "linux")]
    pub fn dev(mut self, interface: &str) -> Route {
        self.ifindex = if_nametoindex(interface);
        self
    }
}


pub trait RouteAction {
    fn add(&mut self, route: Route) -> io::Result<()>;
}