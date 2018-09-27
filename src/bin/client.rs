extern crate a01;
extern crate rand;

use std::env;
use a01::*;
use std::process;
use std::net;
use std::net::UdpSocket;
use rand::Rng;
use std::thread;
use std::time::Duration;

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

    let client_port = parse_client_port(args.get(3))
        .unwrap_or_else(|e| {
            eprintln!("Client Port Parsing Error: {}", e);
            process::exit(1);
        });

    println!("Connecting Mobile with Id {} to Endpoint {}:{}",
        mobile_id,
        endpoint.ip(),
        endpoint.port());

    let local = net::Ipv4Addr::new(127, 0, 0, 1);
    let socket = UdpSocket::bind(std::net::SocketAddrV4::new(local, client_port as u16))
        .unwrap_or_else(|e| {
            eprintln!("Connection Error: {}", e);
            process::exit(1);
        });

    println!("Connected!");

    let id_buf = unsafe {
        std::mem::transmute::<u32, [u8; 4]>(mobile_id)
    };

    loop {
        let random = rand::thread_rng().gen_range(500, 3500);
        let time_buf = unsafe {
            std::mem::transmute::<u32, [u8; 4]>(random)
        };

        let mut buf : [u8; 8] = [0; 8];
        for i in 0..buf.len() {
            if i < 4 {
                buf[i] = id_buf[i];
            } else {
                buf[i] = time_buf[i - 4];
            }
        }

        println!("Sending job that will take {} milliseconds", random);
        socket.send_to(&buf, endpoint)?;
        let sleep_rand = rand::thread_rng().gen_range(1000, 5000);
        println!("Sent! Sleeping for {}", sleep_rand);
        thread::sleep(Duration::from_millis(sleep_rand));
    }

}

fn parse_id(id_arg : Option<&String>) -> Result<u32, String> {
    match id_arg {
        None => Err(String::from("No MobileId argument provided")),
        Some(a) => a.parse::<u32>().map_err(|_| String::from("Invalid MobileId provided")),
    }
}

fn parse_client_port(port_arg : Option<&String>) -> Result<u32, String> {
    match port_arg {
        None => Err(String::from("No Client Port argument provided")),
        Some(a) => a.parse::<u32>().map_err(|_| String::from("Invalid Client Port provided")),
    }
}
