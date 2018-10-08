use std::net;
use std::str::FromStr;
use std::ops::Drop;
use std::sync::{Condvar, Mutex};

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

pub struct Semaphore {
    lock : Mutex<isize>,
    cvar : Condvar,
}

pub struct SemaphoreGuard<'a> {
    sem : &'a Semaphore,
}

impl Semaphore {

    pub fn new(count : isize) -> Semaphore {
        Semaphore {
            lock : Mutex::new(count),
            cvar : Condvar::new(),
        }
    }

    pub fn acquire(&self) {
        let mut count = self.lock.lock().unwrap();
        while *count <= 0 {
            count = self.cvar.wait(count).unwrap();
        }
        *count -= 1;
    }

    pub fn release(&self) {
        *self.lock.lock().unwrap() += 1;
        self.cvar.notify_one();
    }

    pub fn access(&self) -> SemaphoreGuard {
        self.acquire();
        SemaphoreGuard { sem : self }
    }

}

impl<'a> Drop for SemaphoreGuard<'a> {
    fn drop(&mut self) {
        self.sem.release();
    }
}