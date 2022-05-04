use clap::Parser;
use cli::{App, Command};
use core::time;
use fork::{daemon, Fork};

mod cli;
mod containers;

fn main() {
    let args = App::parse();
    println!("{:?}", args);

    match args.command {
        Command::Start(cmd_args) => {
            if cmd_args.daemonize == true {
                if let Ok(Fork::Child) = daemon(false, false) {
                    work();
                }
            } else {
                work()
            }
        }
        _ => {}
    }
}

fn work() {
    loop {
        println!("Hello");
        std::thread::sleep(time::Duration::from_secs(2));
    }
}
