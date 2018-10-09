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

static MAX_JOBS : isize = 12;
static QUANTUM : u64 = 2000;

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

    // C style semaphores that allow us to block the parent/worker thread when certain conditions
    // aren't met. This follows the same pattern in the book, note we need the Arc<T> type to
    // surround the semaphore, that's explained int eh next comment.
    let full_sem : Arc<Semaphore> = Arc::new(Semaphore::new(0));
    let empty_sem : Arc<Semaphore> = Arc::new(Semaphore::new(MAX_JOBS - 1));

    // This data structure, while somewhat ugly, is several nested parametric types that enhance
    // our  MobileMessage to give us a concurrent queue. The first type // is a smart pointer called
    // Arc<T>, which stands for Async Ref Count. Rust has a "borrow checker", // which helps us
    // prevent memory leaks in the absence of a garbage collector. Without going into
    // too much detail, Arc<T> lets multiple threads have references to a single allocation
    // of memory. Without it, Rust wouldn't compile.
    //
    // Next is the Mutex. This provides the mutual exclusion needed to keep our data thread safe
    // then we have a VecDeque which is just a vector with nice functions that allow us to mimic
    // a queue.
    let queue : Arc<Mutex<VecDeque<MobileMessage>>> = Arc::new(Mutex::new(VecDeque::new()));

    // This clone's the reference and ups the count of the Arc<T> async smart pointer.
    // Note that this is NOT a duplicate of the data structure in memory.
    let q = queue.clone();
    let full_sem_ref = full_sem.clone();
    let empty_sem_ref = empty_sem.clone();

    // When we spawn the worker thread, we do so inside a lambda with the "move" keyword.
    // The reason for this keyword is because it captures and takes ownership of the cloned
    // references above, so we have access to the semaphores and the queue from the worker thread.
    let _worker = thread::spawn(move || {
        // Keep the worker thread looping, so it can try to recheck if we have messages
        loop {
            println!("    ---Consumer--- Is Empty?");
            // acquire is equivalent to "down()"
            full_sem_ref.acquire();
            println!("    ---Consumer--- Not Empty, entering critical region");
            // Rust has a kind of RAII here where we call lock() on the Mutex inside the { } scope
            // In doing so, the lock() goes out of the scope as soon as it exits this
            // "let (mm, was_popped) = {}" assignment. Rust
            // automatically unlocks our data. unwrap() simply extracts the data and throws
            // a panic exception that will cause the program to exit
            // Note that the lock() will protect the data from other threads, but will panic
            // if we try to lock from the same thread. We're safe though
            // We have messages, so lets lock the queue and pop the front to get the MobileMessage
            let (mm, was_popped) = {
                let mut q = q.lock().unwrap();
                let mut mm = q.pop_front().unwrap().clone();
                if mm.job_time_in_ms < QUANTUM as u32 {
                    // This is the last round, return the message and announce that it's been popped
                    (mm , true)
                } else {
                    mm.job_time_in_ms -= QUANTUM as u32;
                    // Return to the back of the queue, since it hasn't finished processing
                    q.push_back(mm);
                    // Release the semaphore because we didn't pop
                    full_sem_ref.release();
                    (mm.clone() , false)
                }
            };
            let time_to_run = if was_popped { mm.job_time_in_ms as u64 } else { QUANTUM };
            if was_popped {
                println!("    ---Consumer--- Releasing full, leaving critical region");
                empty_sem_ref.release();
                println!("    ---Consumer--- Full Released");
            }
            println!("---Consumer--- Processing job {} for MobileId {} for {} ms",
                mm.job_id,
                mm.mobile_id,
                time_to_run);
            thread::sleep(Duration::from_millis(time_to_run));
            println!("                             ");
            if was_popped {
                println!("---Consumer--- Job {} completed for MobileId {}",
                    mm.job_id,
                    mm.mobile_id);
            } else {
                println!("---Consumer--- Processed job {} for MobileId {}, {} left",
                    mm.job_id,
                    mm.mobile_id,
                    mm.job_time_in_ms);
            }
        }
    });

    // Prepare buffer to read data we received
    let mut buf : [u8; 12] = [0; 12];

    // Continue looping to keep listening to incoming messages
    loop {
        // We pass buf as a mutable reference so recv_from can populate it with the data we
        // received
        match socket.recv_from(&mut buf) {
            Ok((_byte_count, _source_endpoint)) => {
                // Prepare the actual message before sticking it in the back of the queue
                // Use get_int() helper function to parse the data inside the buf
                let mm = MobileMessage {
                    mobile_id: get_int(0, buf),
                    job_id : get_int(4, buf),
                    job_time_in_ms : get_int(8, buf)
                };
                println!("---Producer--- Received job {} from Mobile {} for {} ms",
                    mm.job_id,
                    mm.mobile_id,
                    mm.job_time_in_ms);
                println!("    ---Producer--- Is Full?");
                empty_sem.acquire();
                println!("    ---Producer--- Not Full, entering critical region");
                // Same lock() and free semantics, we push the MobileMessage to the back of the queue
                match queue.lock().unwrap() {
                    mut q => q.push_back(mm)
                };
                println!("    ---Producer--- Releasing Empty, leaving critical region");
                full_sem.release();
                println!("    ---Producer--- Empty Released ");
            },
            Err(e) => println!("Error : {}", e),
        }
    }

}

// Simple helper to convert half a portion of a byte array of size 8 into an 32-bit integer
fn get_int(start_index: usize, buf : [u8; 12]) -> u32 {
    let mut h = [0; 4];
    h.copy_from_slice(&buf[start_index..(start_index + 4)]);
    unsafe { std::mem::transmute::<[u8; 4], u32>(h) }
}

// Debug allows us to print the message
#[derive(Debug, Copy, Clone)]
struct MobileMessage {
    mobile_id: u32,
    job_id: u32,
    job_time_in_ms : u32,
}

