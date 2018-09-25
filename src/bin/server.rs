extern crate a01;

use std::env;
use a01::*;

fn main() -> std::io::Result<()> {
    let args : Vec<String> = env::args()
        .skip(1) // Skip the first argument, it's the app name
        .collect();
    let address = get_address(&args);

    println!("Running Server");
    Ok(())
}

