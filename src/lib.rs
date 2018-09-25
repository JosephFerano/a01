use std::net;
use std::str::FromStr;

pub fn get_address(args : &Vec<String>) -> Result<net::SocketAddrV4, &'static str> {

    let ip_str = match args.get(1) {
        Some(a) => a,
        None => return Err("No IP argument provided"),
    };
    let port_str = match args.get(2) {
        Some(a) => a,
        None => return Err("No Port argument provided"),
    };
    let ip = net::Ipv4Addr::from_str(&ip_str);
    let port : u16 = port_str.parse().unwrap();
    Ok(net::SocketAddrV4::new(ip.unwrap(), port))
}
