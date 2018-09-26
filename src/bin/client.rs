extern crate a01;
extern crate rand;

use std::env;
use a01::*;
use std::process;
use std::net::UdpSocket;
use rand::Rng;

fn main() -> std::io::Result<()> {
    let args : Vec<String> = env::args()
        .skip(1) // Skip the first argument, it's the app name
        .collect();
    let mobile_id = parse_id(args.get(0))
        .unwrap_or_else(|e| {
            eprintln!("MobileId Parsing Error: {}", e);
            process::exit(1);
        });

    let endpoint = parse_endpoint(args.get(1), args.get(2))
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

    let mut id_buf = unsafe {
        std::mem::transmute::<u32, [u8; 4]>(mobile_id)
    };
    let mut time_buf = unsafe {
        std::mem::transmute::<u32, [u8; 4]>(rand::thread_rng().gen_range(0, 3000))
    };

    let mut buf : [u8; 8] = [0; 8];
    for i in 0..buf.len() {
        if i < 4 {
            buf[i] = id_buf[i];
        } else {
            buf[i] = time_buf[i - 4];
        }
    }

    socket.send_to(&buf, endpoint)?;

    Ok(())
}

fn parse_id(id_arg : Option<&String>) -> Result<u32, String> {
    match id_arg {
        None => Err(String::from("No MobileId argument provided")),
        Some(a) => a.parse::<u32>().map_err(|_| String::from("Invalid MobileId provided")),
    }
}
