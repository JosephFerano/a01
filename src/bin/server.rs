extern crate a01;

use std::env;
use std::process;
use a01::*;
use std::net::UdpSocket;
use std::thread;
use std::time::Duration;
use std::sync::Arc;
use std::collections::VecDeque;
use std::sync::Mutex;

fn main() -> std::io::Result<()> {
    let args : Vec<String> = env::args()
        .skip(1) // Skip the first argument, it's the app name
        .collect();
    let endpoint = parse_endpoint(args.get(0), args.get(1))
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

    let mut buf : [u8; 8] = [0; 8];
    let queue : Arc<Mutex<VecDeque<MobileMessage>>> = Arc::new(Mutex::new(VecDeque::new()));
    let q = queue.clone();

    let worker = thread::spawn(move || {
        loop {
            let length = match q.lock().unwrap() {
                q => q.len()
            };
            if length < 1 {
                println!("Worker thread yielding");
                thread::park();
            } else {
                let mut lock = q.lock().unwrap();
                let mm = lock.pop_front().unwrap();
                println!("Processing job for MobileId {} for {} milliseconds ",
                    mm.id,
                    mm.job_time_in_ms);
                thread::sleep(Duration::from_millis(mm.job_time_in_ms));
            }
        }
    });

    loop {
        match socket.recv_from(&mut buf) {
            Ok((byte_count, source_endpoint)) => {
                println!("Got {} bytes from {}", byte_count, source_endpoint);
                for i in 0..byte_count {
                    println!("{}", buf[i]);
                }
                let test = vec![0..4];

                let mut id_buf = get_little_endian_int();
                let mut time_buf = unsafe {
                    std::mem::transmute::<[u8; 4], u32>(rand::thread_rng().gen_range(0, 3000))
                };
                queue.lock().unwrap().push_back(MobileMessage { id : 1 , job_time_in_ms : 1000 });
                worker.thread().unpark()
            },
            Err(e) => println!("Error : {}", e),
        }
        thread::sleep(Duration::from_millis(150));
    }

}
//fn first_half(a: [u8; 8]) -> [u8; 4] {
//  let mut h = [0; 4];
//  h.copy_from_slice(&a[0..4]);
//  h
//}
//
//fn main() {
//  println!("{:?}", first_half([3;8]));
//}
fn get_little_endian_int(buf : [u8; 8]) -> u32 {
    unsafe {
        std::mem::transmute::<[u8; 4], u32>([buf[3], buf[2], buf[1], buf[0]])
    }
}

#[derive(Debug)]
struct MobileMessage {
    id : usize,
    job_time_in_ms : u64,
}

