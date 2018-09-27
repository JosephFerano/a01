use std::net;
use std::str::FromStr;

// Helper function shared between client and server to parse the command line arguments
// and convert into an endpoint
pub fn parse_endpoint(ip_arg : Option<&String>, port_arg : Option<&String>) -> Result<net::SocketAddrV4, String> {
    // ok_or allows us to convert an Option to our return type which is a Result<>
    let ip_str = ip_arg.ok_or(String::from("No IP argument provided"))?;
    let port_str = port_arg.ok_or(String::from("No Port argument provided"))?;

    let ip = net::Ipv4Addr::from_str(&ip_str).map_err(|e| e.to_string())?;
    let port : u16 = port_str.parse().map_err(|_| String::from("Invalid Port number provided"))?;
    Ok(net::SocketAddrV4::new(ip, port))
}

