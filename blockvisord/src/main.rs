use clap::Parser;
use cli::{App, Command};
use core::time;
use daemonize::Daemonize;
use std::fs::{self, File, OpenOptions};
use std::path::Path;

mod cli;
mod containers;

const CONFIG_FILE: &str = "config.toml";

const PID_FILE: &str = "/tmp/blockvisor.pid";
const OUT_FILE: &str = "/tmp/blockvisor.out";
const ERR_FILE: &str = "/tmp/blockvisor.err";

fn main() {
    let args = App::parse();
    println!("{:?}", args);

    match args.command {
        Command::Configure(_) => {
            println!("Configuring blockvisor");
            File::create(CONFIG_FILE).unwrap();
        }
        Command::Start(cmd_args) => {
            if !Path::new(CONFIG_FILE).exists() {
                eprintln!("Error: not configured, please run `configure` first");
                return;
            }

            if cmd_args.daemonize {
                let stdout = OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(OUT_FILE)
                    .unwrap();
                let stderr = OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(ERR_FILE)
                    .unwrap();

                let daemonize = Daemonize::new()
                    .pid_file(PID_FILE)
                    .stdout(stdout)
                    .stderr(stderr);

                match daemonize.start() {
                    Ok(_) => println!("Starting blockvisor in background"),
                    Err(e) => {
                        eprintln!("Error: {}", e);
                        return;
                    }
                }
            }
            work(cmd_args.daemonize);
        }
        Command::Stop(_) => {
            if Path::new(PID_FILE).exists() {
                fs::remove_file(PID_FILE).unwrap()
            }
        }
        _ => {}
    }
}

fn work(daemonized: bool) {
    loop {
        if !daemonized || Path::new(PID_FILE).exists() {
            println!("Hello");
            std::thread::sleep(time::Duration::from_secs(2));
        } else {
            println!("Stopping blockvisor");
            break;
        }
    }
}
