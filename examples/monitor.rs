use route_rs::{RouteAction, RouteSock};

fn main() {
    let mut handle = RouteSock::new().unwrap();

    let mut buf = RouteSock::new_buf();
    loop {
        let ret = handle.monitor(&mut buf).unwrap();
        match ret.0 {
            route_rs::RouteChange::OTHER(n) if n == 0xc => {
                let route = ret.1;
                if route.destination.is_unspecified() {
                    println!("default route changed: {:?}", route);
                }
            },
            _ => ()
        }
    }
}
