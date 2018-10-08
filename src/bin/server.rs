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
    // Get the command line args
    let args : Vec<String> = env::args()
        .skip(1) // Skip the first argument, it's the app name
        .collect();
    let endpoint = parse_endpoint(args.get(0), args.get(1))
        // This function either gets the parsed ip and port or invokes this lambda/anonymous function
        // which invokes an error that can be redirected to sterr
        .unwrap_or_else(|e| {
            eprintln!("Endpoint Parsing Error: {}", e);
            process::exit(1);
        });

    println!("Starting Server at Endpoint {}:{}", endpoint.ip(), endpoint.port());

    // Here we bind to the UDP socket. The bind() function returns a Result<T, E>, where
    // T is a Socket and E is an error. First we create
    let socket = UdpSocket::bind(endpoint)
        .unwrap_or_else(|e| {
            eprintln!("Connection Error: {}", e);
            process::exit(1);
        });

    println!("Server Connected, listening...");

    // This data structure, while somewhat ugly, is several nested parametric types that enhance
    // our inner collection of MobileMessage to give us a concurrent dictionary. The first type
    // is a smart pointer called Arc<T>, which stands for Async Ref Count. Rust has a "borrow checker", which helps us
    // prevent memory leaks in the absence of a garbage collector. Without going intto too much
    // detail, Arc<T> lets us have multiple references from different threads to a single piece
    // of memory. Without it, Rust wouldn't compile.
    //
    // Next is the Mutex. This provides the mutal exclusion neeed to keep our data thread safe
    // then we have a VecDeque which is just a vector with nice functions that allow us to mimic
    // a queue.
    let empty : Arc<Semaphore> = Arc::new(Semaphore::new(0));
    let full : Arc<Semaphore> = Arc::new(Semaphore::new(0));
    let queue : Arc<Mutex<VecDeque<MobileMessage>>> = Arc::new(Mutex::new(VecDeque::new()));

    // This clone's the reference and ups the count of the Arc<T> async smart pointer.
    // Note that this is NOT a duplicate queue in memory.
    let q = queue.clone();
    let s = semaphore.clone();

    // When we spawn the worker thread, we do so inside a lambda with the "move" keyword.
    // The reason for this keywoard is because it captures and takes ownership of the "q" variable
    // which is the reference to the queue.
    let worker = thread::spawn(move || {
        // Keep the worker thread looping, so it can try to recheck if we have messages
        loop {
            // Rust has a kind of RAII here where we call lock() on the Mutex then match over it
            // using an expression to get the length of the queue. In doing so, the lock()
            // goes out of the scope as soon as it exits this "let length" assignment. Rust
            // automatically unlocks our data. unwrap() simply extracts the data and throws
            // a panic exception that will cause the program to exit
            // Note that the lock() will protect the data from other threads, but will panic
            // if we try to lock from the same thread. We're safe though
            let length = match q.lock().unwrap() {
                q => q.len()
            };
            if length < 1 {
                println!("Worker thread yielding");
                // Since we have no messages, we "park()", which will block the thread until
                // someone calls unpark()
                thread::park();
            } else {
                // We have messages, so lets lock the queue and pop the front to get the MobileMessage
                // Same RAII style locking and unlocking with a match
                let mm = match q.lock().unwrap() {
                    mut q => q.pop_front().unwrap()
                };
                println!("Processing job for MobileId {} for {} milliseconds ",
                    mm.id,
                    mm.job_time_in_ms);
                // Finally, we execute the job
                thread::sleep(Duration::from_millis(mm.job_time_in_ms as u64));
            }
        }
    });

    // Prepare buffer to read data we received
    let mut buf : [u8; 8] = [0; 8];

    // Continue looping to keep listening to incoming messages
    loop {
        // We pass buf as a mutable reference so recv_from can populate it with the data we
        // received
        match socket.recv_from(&mut buf) {
            Ok((_byte_count, _source_endpoint)) => {
                // Prepare the actual message before sticking it in the back of the queue
                // Use get_int() helper function to parse the data inside the buf
                let mm = MobileMessage { id : get_int(0, buf) , job_time_in_ms : get_int(4, buf) };
                println!("Received job from Mobile {}, it'll take about {} milliseconds",
                    mm.id,
                    mm.job_time_in_ms);
                // Same lock() and free semantics, we push the MobileMessage
                match queue.lock().unwrap() {
                    mut q => q.push_back(mm)
                };
                // Awaken the worker thread since we have a message for it to handle
                worker.thread().unpark()
            },
            Err(e) => println!("Error : {}", e),
        }
    }

}

// Simple helper to convert half a portion of a byte array of size 8 into an 32-bit integer
fn get_int(start_index: usize, buf : [u8; 8]) -> u32 {
    let mut h = [0; 4];
    h.copy_from_slice(&buf[start_index..(start_index + 4)]);
    unsafe { std::mem::transmute::<[u8; 4], u32>(h) }
}

// Debug allows us to print the message
#[derive(Debug)]
struct MobileMessage {
    id : u32,
    job_time_in_ms : u32,
}

