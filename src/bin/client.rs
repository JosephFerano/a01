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

static MIN_JOB_TIME_MS : u32 = 6000;
static MAX_JOB_TIME_MS : u32 = 6500;
static MIN_SLEEP_TIME_MS : u64 = 100;
static MAX_SLEEP_TIME_MS : u64 = 500;

fn main() -> std::io::Result<()> {
    // Get the command line args
    let args : Vec<String> = env::args()
        .skip(1) // Skip the first argument, it's the app name
        .collect();
    let mobile_id = parse_id(args.get(0))
        // This function either gets the parsed id or invokes this lambda/anonymous function
        // which invokes an error that can be redirected to sterr
        .unwrap_or_else(|e| {
            eprintln!("MobileId Parsing Error: {}", e);
            process::exit(1);
        });

    // We get the args and pass them to parse the endpoint
    let endpoint = parse_endpoint(args.get(1), args.get(2))
        .unwrap_or_else(|e| {
            eprintln!("Endpoint Parsing Error: {}", e);
            process::exit(1);
        });

    // Get the client port so we can bind to a UDP socket, we need to get this from the
    // command line, unless we're prepared to just try random ports
    let client_port = parse_client_port(args.get(3))
        .unwrap_or_else(|e| {
            eprintln!("Client Port Parsing Error: {}", e);
            process::exit(1);
        });

    println!("Connecting Mobile with Id {} to Endpoint {}:{}",
        mobile_id,
        endpoint.ip(),
        endpoint.port());

    // Create an IP
    let local = net::Ipv4Addr::new(127, 0, 0, 1);
    // Create the Socket Address that we will bind to, not the server endpoint, that's later
    let socket_addr = std::net::SocketAddrV4::new(local, client_port as u16);
    // Here we bind to the UDP socket. The bind() function returns a Result<T, E>, where
    // T is a Socket and E is an error. First we create
    let socket = UdpSocket::bind(socket_addr)
        // Same thing as before, either return the socket, or throw the error in this lambda
        .unwrap_or_else(|e| {
            eprintln!("Connection Error: {}", e);
            process::exit(1);
        });

    println!("Connected!");

    // Rust requires us to mark certain operations as unsafe, taking raw bits or data types and
    // conveting them into others is one of them. Here, transmute converts an unsigned 32-bit
    // integer into an array of bytes (unsigned 8 bit) with a size of 4. Rust also requires
    // that the size is known at compile time. This prepares the mobile_id number to be sent
    // over the wire via UDP
    let mobile_id_buf = unsafe {
        std::mem::transmute::<u32, [u8; 4]>(mobile_id)
    };

    // Loops are simple in Rust
    let mut job_id_count = 0;
    loop {
        let random = rand::thread_rng().gen_range(MIN_JOB_TIME_MS, MAX_JOB_TIME_MS);
        // Same as previous unsafe call, this one is nested in the loop cause it changes
        // every cycle, no need to do that for the mobile_id, it doesn't change
        let time_buf = unsafe {
            std::mem::transmute::<u32, [u8; 4]>(random)
        };

        let job_id_buf = unsafe {
            job_id_count += 1;
            std::mem::transmute::<u32, [u8; 4]>(job_id_count)
        };

        // We create a buffer of size 12 and populate it with the mobile_id, job_id and the random time.
        // This Buf will then be sent over the wire
        let mut buf : [u8; 12] = [0; 12];
        for i in 0..buf.len() {
            if i < 4 {
                buf[i] = mobile_id_buf[i];
            } else if i < 8 {
                buf[i] = job_id_buf[i - 4];
            } else {
                buf[i] = time_buf[i - 8];
            }
        }

        println!("Sending job {} that will take {} milliseconds", job_id_count, random);
        // Off we go!
        socket.send_to(&buf, endpoint)?;
        let sleep_rand = rand::thread_rng().gen_range(MIN_SLEEP_TIME_MS, MAX_SLEEP_TIME_MS);
        println!("Sent! Sleeping for {}", sleep_rand);
        // Now we sleep till the next iteration
        thread::sleep(Duration::from_millis(sleep_rand));
    }

}

// The function array.get() returns and Option, which can be either None or Some<T>
// We pattern match to extract the values, returning errors as strings when something goes wrong
// If no parameter was passed in through the terminal, we get the None case. If Some, we attempt
// to parse to unsigned 32-bit integer. If that causes and error, the user didn't provide a valid
// integer. map_err converts one error type to anothre
fn parse_id(id_arg : Option<&String>) -> Result<u32, String> {
    match id_arg {
        None => Err(String::from("No MobileId argument provided")),
        Some(a) => a.parse::<u32>().map_err(|_| String::from("Invalid MobileId provided")),
    }
}

// Same principle as the previous function
fn parse_client_port(port_arg : Option<&String>) -> Result<u32, String> {
    match port_arg {
        None => Err(String::from("No Client Port argument provided")),
        Some(a) => a.parse::<u32>().map_err(|_| String::from("Invalid Client Port provided")),
    }
}
