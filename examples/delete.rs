


use route_rs::{if_nametoindex, Route, RouteAction, RouteSock};


fn main() {
    let mut handle = RouteSock::new().unwrap();
    let ifindex = if_nametoindex("en0").unwrap();

    let route = Route::new(
        "1.1.1.1".parse().unwrap(), 
        "32".parse().unwrap()
    ).ifindex(ifindex);
    handle.add(&route).unwrap();
    println!("1.1.1.1/32 add to en0");

    std::thread::sleep(std::time::Duration::from_secs(10));

    handle.delete(&route).unwrap();
    println!("1.1.1.1/32 delete from en0");
}
