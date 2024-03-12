use route_rs::{if_nametoindex, Route, RouteAction, RouteSock};

use std::fs::read_to_string;

const NON_CN_LIST: &str =
    "/Users/bytedance/project/my_project/Rust/quicocks/examples/non_cn_ip_cidr.txt";

fn main() {
    let mut handle = RouteSock::new().unwrap();
    let ifindex = if_nametoindex("utun4").unwrap();

    for line in read_to_string(NON_CN_LIST).unwrap().lines() {
        if let Some((ip, cidr)) = line.split_once('/') {
            let route = Route::new(ip.parse().unwrap(), cidr.parse().unwrap()).ifindex(ifindex);

            handle.add(route).unwrap();
        } else {
            println!("{} can not be splited by `/`", line);
        }
    }
}
