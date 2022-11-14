mod disk;

use disk::Disk;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    println!();

    match Disk::load(&args[1]) {
        Ok(disk) => println!("{}", disk),
        Err(err) => println!("Error loading disk image: {}", err),
    }
}
