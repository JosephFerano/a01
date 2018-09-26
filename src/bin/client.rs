extern crate a01;

use std::env;
use a01::*;
use std::process;
use std::net::UdpSocket;

fn main() -> std::io::Result<()> {
    let args : Vec<String> = env::args()
        .skip(1) // Skip the first argument, it's the app name
        .collect();
    let mobile_id = parse_id(args.get(0))
        .unwrap_or_else(|e| {
            eprintln!("MobileId Parsing Error: {}", e);
            process::exit(1);
        });

    let endpoint = get_endpoint(args.get(1), args.get(2))
        .unwrap_or_else(|e| {
            eprintln!("Endpoint Parsing Error: {}", e);
            process::exit(1);
        });

    println!("Connecting Mobile with Id {} to Endpoint {}:{}",
        mobile_id,
        endpoint.ip(),
        endpoint.port());

    let socket = UdpSocket::bind("127.0.0.1:7789")
        .unwrap_or_else(|e| {
            eprintln!("Connection Error: {}", e);
            process::exit(1);
        });

//    socket.connect("127.0.0.1:8080").expect("Something went wrong");

    let mut buf : Vec<u8> = Vec::with_capacity(128);
    buf.push(0x00);
    buf.push(0x0f);
    buf.push(0xf0);
//    socket.send_to(&[0; 10], endpoint)?;
    socket.send_to(buf.as_mut_slice(), endpoint)?;

    Ok(())
}

fn parse_id(id_arg : Option<&String>) -> Result<usize, String> {
    match id_arg {
        None => Err(String::from("No MobileId argument provided")),
        Some(a) => a.parse::<usize>().map_err(|_| String::from("Invalid MobileId provided")),
    }
}
