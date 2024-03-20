// use std::{net::IpAddr, slice};

// use libc::{c_void, nlmsghdr, RTA_DST, RTA_GATEWAY, RTA_OIF};


// const NLMSG_ALIGNTO: usize = 4;

// #[macro_export]
// macro_rules! NLMSG_ALIGN {
//     ($len: expr) => {
//         (($len + NLMSG_ALIGNTO - 1) & !(NLMSG_ALIGNTO - 1))
//     };
// }

// const NLMSG_HDRLEN: usize = NLMSG_ALIGN!(std::mem::size_of::<nlmsghdr>());

// #[macro_export]
// macro_rules! NLMSG_LENGTH {
//     ($len: expr) => {
//         ($len + NLMSG_HDRLEN)
//     };
// }

// #[macro_export]
// macro_rules! NLMSG_TAIL {
//     ($nmsg: expr) => {
//         {
//             let rtattr = unsafe {
//                 ($nmsg as *const nlmsghdr as *const c_void)
//                     .add(NLMSG_ALIGN!($nmsg.nlmsg_len as usize))
//             };

//             unsafe {
//                 &mut *(rtattr as *mut rtattr)
//             }
//         }
//     }
// }

// const RTA_ALIGNTO: usize = 4;

// #[macro_export]
// macro_rules! RTA_ALIGN {
//     ($len: expr) => {
//         (($len + RTA_ALIGNTO - 1) & !(RTA_ALIGNTO - 1))
//     };
// }

// #[macro_export]
// macro_rules! RTA_LENGTH {
//     ($len: expr) => {
//         RTA_ALIGN!(std::mem::size_of::<rtattr>()) + $len
//     };
// }

// // #define RTA_DATA(rta)   ((void*)(((char*)(rta)) + RTA_LENGTH(0)))
// #[macro_export]
// macro_rules! RTA_DATA {
//     ($rta: expr) => {
//         {
//             let data = unsafe {
//                 ($rta as *const rtattr as *const c_void).add(RTA_LENGTH!(0))
//             };

//             unsafe {
//                 &mut *(data as *mut u8)
//             }
//         }
//     }
// }


// #[repr(C)]
// #[allow(non_camel_case_types)]
// pub(super) struct rtmsg {
//     pub rtm_family: u8,
//     pub rtm_dst_len: u8,
//     pub rtm_src_len: u8,
//     pub rtm_tos: u8,

//     pub rtm_table: u8,      /* Routing table id */
//     pub rtm_protocol: u8,   /* Routing protocol */
//     pub rtm_scope: u8,   
//     pub rtm_type: u8,   

//     pub rtm_flags: u32,
// }

// #[repr(C)]
// #[allow(non_camel_case_types)]
// pub(super) struct rtattr {
//     rta_len: u16,
//     rta_type: u16,
// }

// #[repr(C)]
// #[allow(non_camel_case_types)]
// pub(super) struct m_rtmsg {
//     pub n: nlmsghdr,
//     pub r: rtmsg,
//     pub attr: [u8; 4096],
// }

// impl Default for m_rtmsg {
//     fn default() -> Self {
//         let mut rtmsg = unsafe {
//             std::mem::zeroed::<m_rtmsg>()
//         };

//         rtmsg.n.nlmsg_len = NLMSG_LENGTH!(std::mem::size_of::<rtmsg>()) as u32;

//         rtmsg
//     }
// }

// impl m_rtmsg {
//     fn add_attr(&mut self, ty: u16, data: &[u8]) {
//         let rta = NLMSG_TAIL!(&self.n);
//         rta.rta_type = ty;

//         let len = RTA_LENGTH!(data.len());

//         if NLMSG_ALIGN!(self.n.nlmsg_len as usize) + RTA_ALIGN!(len) > std::mem::size_of::<m_rtmsg>() {
//             return;
//         }

//         rta.rta_len = len as u16;
//         let p = RTA_DATA!(rta);
//         let slice = unsafe {
//             slice::from_raw_parts_mut(p, data.len())
//         };
//         slice.copy_from_slice(data);

//         self.n.nlmsg_len = (NLMSG_ALIGN!(self.n.nlmsg_len as usize) + RTA_ALIGN!(len)) as u32;
//     }

//     pub fn put_destination(&mut self, dst: &IpAddr) {
//         match dst {
//             IpAddr::V4(addr) => {
//                 self.add_attr(RTA_DST, &addr.octets())
//             },
//             IpAddr::V6(addr) => {
//                 self.add_attr(RTA_DST, &addr.octets())
//             },
//         }
//     }

//     pub fn put_index(&mut self, ifindex: u32) {
//         self.add_attr(RTA_OIF, &ifindex.to_ne_bytes())
//     }

//     pub fn put_gateway(&mut self, gateway: &IpAddr) {
//         match gateway {
//             IpAddr::V4(addr) => {
//                 self.add_attr(RTA_GATEWAY, &addr.octets())
//             },
//             IpAddr::V6(addr) => {
//                 self.add_attr(RTA_GATEWAY, &addr.octets())
//             },
//         }
//     }
// }