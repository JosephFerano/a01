extern crate a01;

use std::env;
use std::process;
use a01::*;
use std::net::UdpSocket;
use std::thread;
use std::time::Duration;

fn main() -> std::io::Result<()> {
    let args : Vec<String> = env::args()
        .skip(1) // Skip the first argument, it's the app name
        .collect();
    let endpoint = get_endpoint(args.get(0), args.get(1))
        .unwrap_or_else(|e| {
            eprintln!("Endpoint Parsing Error: {}", e);
            process::exit(1);
        });

    println!("Starting Server at Endpoint {}:{}", endpoint.ip(), endpoint.port());

    let socket = UdpSocket::bind(endpoint)
        .unwrap_or_else(|e| {
            eprintln!("Connection Error: {}", e);
            process::exit(1);
        });

    println!("Server Connected, listening...");

    let mut buf = [0; 10];

    loop {
        match socket.recv_from(&mut buf) {
            Ok((byte_count, source_endpoint)) => {
                println!("Got {} bytes from {}", byte_count, source_endpoint);
                for i in 0..byte_count {
                    println!("{}", buf[i]);
                }
            },
            Err(e) => println!("Error : {}", e),
        }
        thread::sleep(Duration::from_millis(150));
    }

//    let t = thread::spawn(move || {
//        read_message(socket);
//    });

//    let received = t.join().unwrap();
    Ok(())
}

fn read_message(socket : UdpSocket) -> Vec<u8> {
    println!("Received message!");
    Vec::new()
}
