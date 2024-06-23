use std::env;
use std::process;
use std::thread;
use std::time::Duration;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: test <success|failure>");
        process::exit(1);
    }

    match args[1].as_str() {
        "success" => {
            println!("Program started successfully");
            thread::sleep(Duration::from_secs(5));
            process::exit(0);
        }
        "failure" => {
            println!("Program started but failed");
            thread::sleep(Duration::from_secs(1));
            process::exit(1);
        }
        _ => {
            eprintln!("Invalid argument");
            process::exit(1);
        }
    }
}
