use route_rs::{Route, RouteAction, RouteSock};

fn main() {
    let mut handle = RouteSock::new().unwrap();

    // let route = Route::new("0.0.0.0".parse().unwrap(), "0".parse().unwrap());
    let route = Route::new("10.69.56.4".parse().unwrap(), "22".parse().unwrap());
    let ret = handle.get(&route).unwrap();
    println!("{:?}", ret);
}
