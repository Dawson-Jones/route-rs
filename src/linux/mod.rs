mod rtmsg;

use std::{
    io,
    net::IpAddr,
    ops::{Deref, DerefMut},
    os::fd::{AsRawFd, RawFd},
};

use ipnetwork::IpNetwork;
use netlink_packet_core::{
    NetlinkHeader, NetlinkMessage, NetlinkPayload, NLM_F_ACK, NLM_F_CREATE, NLM_F_DUMP, NLM_F_EXCL,
    NLM_F_REQUEST,
};
use netlink_packet_route::{
    route::{
        RouteAddress, RouteAttribute, RouteHeader, RouteMessage, RouteProtocol, RouteScope,
        RouteType,
    },
    AddressFamily, RouteNetlinkMessage,
};
use netlink_sys::{protocols::NETLINK_ROUTE, Socket, SocketAddr};

use crate::{Route, RouteAction, RouteChange};

pub struct RouteSock(Socket);

impl AsRawFd for RouteSock {
    fn as_raw_fd(&self) -> RawFd {
        self.0.as_raw_fd()
    }
}

impl RouteSock {
    pub fn new() -> io::Result<Self> {
        let mut socket = Socket::new(NETLINK_ROUTE)?;
        let _port_number = socket.bind_auto()?.port_number();
        socket.connect(&SocketAddr::new(0, 0))?;

        Ok(RouteSock(socket))
    }
}

impl Deref for RouteSock {
    type Target = Socket;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for RouteSock {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl RouteAction for RouteSock {
    fn add(&mut self, route: &Route) -> io::Result<()> {
        let mut nl_hdr = NetlinkHeader::default();
        nl_hdr.flags = NLM_F_REQUEST | NLM_F_EXCL | NLM_F_CREATE | NLM_F_ACK;
        nl_hdr.sequence_number = 1;

        let mut rt_msg = RouteMessage::default();
        rt_msg.header.table = RouteHeader::RT_TABLE_MAIN;
        rt_msg.header.protocol = RouteProtocol::Boot;
        rt_msg.header.scope = RouteScope::Universe;
        rt_msg.header.kind = RouteType::Unicast;

        match route.destination {
            std::net::IpAddr::V4(addr) => {
                rt_msg.header.address_family = AddressFamily::Inet;
                rt_msg
                    .attributes
                    .push(RouteAttribute::Destination(RouteAddress::Inet(addr)));
            }
            std::net::IpAddr::V6(addr) => {
                rt_msg.header.address_family = AddressFamily::Inet6;
                rt_msg
                    .attributes
                    .push(RouteAttribute::Destination(RouteAddress::Inet6(addr)));
            }
        }
        rt_msg.header.destination_prefix_length = route.prefix;

        if let Some(gateway) = route.gateway {
            match gateway {
                std::net::IpAddr::V4(addr) => {
                    rt_msg.header.address_family = AddressFamily::Inet;
                    rt_msg
                        .attributes
                        .push(RouteAttribute::Gateway(RouteAddress::Inet(addr)));
                }
                std::net::IpAddr::V6(addr) => {
                    rt_msg.header.address_family = AddressFamily::Inet6;
                    rt_msg
                        .attributes
                        .push(RouteAttribute::Gateway(RouteAddress::Inet6(addr)));
                }
            }
        }

        if let Some(index) = route.ifindex {
            rt_msg.header.scope = RouteScope::Link;
            rt_msg.attributes.push(RouteAttribute::Oif(index));
        }

        let mut req = NetlinkMessage::new(
            nl_hdr,
            NetlinkPayload::from(RouteNetlinkMessage::NewRoute(rt_msg)),
        );
        req.finalize();

        let mut buf = [0u8; 4096];
        req.serialize(&mut buf[..req.buffer_len()]);

        self.send(&buf[..req.buffer_len()], 0)?;

        let mut rbuf = Vec::with_capacity(4096);
        let n = self.recv(&mut rbuf, 0)?;
        let bytes = &rbuf[..n];
        let rx_packet = <NetlinkMessage<RouteNetlinkMessage>>::deserialize(bytes);
        // println!("<<< {:?}", rx_packet);
        match rx_packet {
            Ok(rx_packet) => {
                if let NetlinkPayload::Error(e) = rx_packet.payload {
                    match e.code {
                        Some(e) => {
                            return Err(io::Error::new(io::ErrorKind::Other, format!("{e:?}")))
                        }
                        None => return Ok(()),
                    }
                }
            }
            Err(e) => return Err(io::Error::new(io::ErrorKind::Other, format!("{e:?}"))),
        }

        Ok(())
    }

    fn delete(&mut self, route: &Route) -> io::Result<()> {
        let mut nl_hdr = NetlinkHeader::default();
        nl_hdr.flags = NLM_F_REQUEST | NLM_F_ACK;
        nl_hdr.sequence_number = 1;

        let mut rt_msg = RouteMessage::default();
        rt_msg.header.table = RouteHeader::RT_TABLE_MAIN;
        rt_msg.header.scope = RouteScope::NoWhere;

        match route.destination {
            std::net::IpAddr::V4(addr) => {
                rt_msg.header.address_family = AddressFamily::Inet;
                rt_msg
                    .attributes
                    .push(RouteAttribute::Destination(RouteAddress::Inet(addr)));
            }
            std::net::IpAddr::V6(addr) => {
                rt_msg.header.address_family = AddressFamily::Inet6;
                rt_msg
                    .attributes
                    .push(RouteAttribute::Destination(RouteAddress::Inet6(addr)));
            }
        }
        rt_msg.header.destination_prefix_length = route.prefix;

        let mut req = NetlinkMessage::new(
            nl_hdr,
            NetlinkPayload::from(RouteNetlinkMessage::DelRoute(rt_msg)),
        );
        req.finalize();

        let mut buf = [0u8; 4096];
        req.serialize(&mut buf[..req.buffer_len()]);
        self.send(&buf[..req.buffer_len()], 0)?;

        let mut rbuf = Vec::with_capacity(4096);
        let n = self.recv(&mut rbuf, 0)?;
        let bytes = &rbuf[..n];
        let rx_packet = <NetlinkMessage<RouteNetlinkMessage>>::deserialize(bytes);
        // println!("<<< {:?}", rx_packet);
        match rx_packet {
            Ok(rx_packet) => {
                if let NetlinkPayload::Error(e) = rx_packet.payload {
                    match e.code {
                        Some(e) => {
                            return Err(io::Error::new(io::ErrorKind::Other, format!("{e:?}")))
                        }
                        None => return Ok(()),
                    }
                }
            }
            Err(e) => return Err(io::Error::new(io::ErrorKind::Other, format!("{e:?}"))),
        }

        Ok(())
    }

    fn get(&mut self, route: &Route) -> io::Result<Route> {
        let mut nl_hdr = NetlinkHeader::default();
        nl_hdr.flags = NLM_F_DUMP | NLM_F_REQUEST;
        nl_hdr.sequence_number = 1;

        let mut rt_msg = RouteMessage::default();
        rt_msg.header.address_family = AddressFamily::Inet;
        rt_msg
            .attributes
            .push(RouteAttribute::Table(254 /*table main*/));
        if let Some(index) = route.ifindex {
            rt_msg.attributes.push(RouteAttribute::Oif(index));
        }

        let mut req = NetlinkMessage::new(
            nl_hdr,
            NetlinkPayload::from(RouteNetlinkMessage::GetRoute(rt_msg)),
        );
        req.finalize();
        let mut buf = [0u8; 4096];
        req.serialize(&mut buf[..req.buffer_len()]);
        println!(">>> {:?}", &buf[..req.buffer_len()]);
        self.send(&buf[..req.buffer_len()], 0)?;

        let mut ret = Route::default();
        let mut offset = 0;
        let mut rbuf = Vec::with_capacity(4096);
        if let Ok(n) = self.recv(&mut rbuf, 0) {
            loop {
                let bytes = &rbuf[offset..];

                match <NetlinkMessage<RouteNetlinkMessage>>::deserialize(bytes) {
                    Ok(rx_packet) => {
                        if matches!(rx_packet.payload, NetlinkPayload::Done(_)) {
                            println!("Done!");
                            break;
                        }

                        if let NetlinkPayload::InnerMessage(rtnl_msg) = rx_packet.payload {
                            if let RouteNetlinkMessage::NewRoute(rt_msg) = rtnl_msg {
                                if rt_msg.header.destination_prefix_length <= route.prefix
                                    && rt_msg.header.destination_prefix_length >= ret.prefix
                                {
                                    let mut t_route = Route::default();
                                    t_route.prefix = rt_msg.header.destination_prefix_length;

                                    let mut travel_over_nomal = true;
                                    for attr in &rt_msg.attributes {
                                        match attr {
                                            RouteAttribute::Destination(dst) => {
                                                if let RouteAddress::Inet(dst) = dst {
                                                    if let IpAddr::V4(target) = route.destination {
                                                        let contained = IpNetwork::new(
                                                            IpAddr::V4(*dst),
                                                            rt_msg.header.destination_prefix_length,
                                                        )
                                                        .unwrap()
                                                        .contains(IpAddr::V4(target));

                                                        if !contained {
                                                            travel_over_nomal = false;
                                                            break;
                                                        }

                                                        t_route.destination = IpAddr::V4(*dst)
                                                    }
                                                } else if let RouteAddress::Inet6(dst) = dst {
                                                    if let IpAddr::V6(target) = route.destination {
                                                        let contained = IpNetwork::new(
                                                            IpAddr::V6(*dst),
                                                            rt_msg.header.destination_prefix_length,
                                                        )
                                                        .unwrap()
                                                        .contains(IpAddr::V6(target));

                                                        if !contained {
                                                            travel_over_nomal = false;
                                                            break;
                                                        }

                                                        t_route.destination = IpAddr::V6(*dst)
                                                    }
                                                }
                                            }

                                            RouteAttribute::Gateway(gw) => {
                                                if let RouteAddress::Inet(gw) = gw {
                                                    t_route.gateway = Some(IpAddr::V4(*gw));
                                                } else if let RouteAddress::Inet6(gw) = gw {
                                                    t_route.gateway = Some(IpAddr::V6(*gw));
                                                } else {
                                                    return Err(io::Error::new(
                                                        io::ErrorKind::Other,
                                                        "Invalid gateway",
                                                    ));
                                                }
                                            }

                                            RouteAttribute::Oif(index) => {
                                                t_route.ifindex = Some(*index);
                                            }

                                            _ => (),
                                        }
                                    }

                                    if travel_over_nomal {
                                        ret = t_route
                                    }
                                }
                            }
                        }

                        offset += rx_packet.header.length as usize;
                        if offset == n || rx_packet.header.length == 0 {
                            break;
                        }
                    }
                    Err(e) => {
                        return Err(io::Error::new(io::ErrorKind::Other, format!("{e:?}")));
                    }
                }
            }
        }

        Ok(ret)
    }

    fn monitor(&mut self, buf: &mut [u8]) -> io::Result<(RouteChange, Route)> {
        todo!()
    }
}
