use core::time;
use fork::{daemon, Fork};

mod containers;

fn main() {
    if let Ok(Fork::Child) = daemon(false, false) {
        work();
    }
}

fn work() {
    loop {
        println!("Hello");
        std::thread::sleep(time::Duration::from_secs(2));
    }
}
