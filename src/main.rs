mod args;
mod config;

use clap::Parser;
use args::Args;
use config::{parse_bind_address, validate_port, parse_bind_address_to_socket_addr};

fn main() {
    let args = Args::parse();
    
    println!("Command line arguments: {:?}", args);
    
    match parse_bind_address(&args.bind) {
        Ok((host, port)) => {
            println!("Parsed address: host='{}', port={}", host, port);
            
            match validate_port(port) {
                Ok(_) => println!("Port validation passed"),
                Err(err) => {
                    eprintln!("Error: {}", err);
                    std::process::exit(1);
                }
            }
        },
        Err(err) => {
            eprintln!("Error parsing address: {}", err);
            std::process::exit(1);
        }
    }
    
    match parse_bind_address_to_socket_addr(&args.bind) {
        Ok(addr) => println!("Socket address: {:?}", addr),
        Err(err) => {
            eprintln!("Error: {}", err);
            std::process::exit(1);
        }
    }
    
    println!("URL: {:?}", args.get_url());
    println!("Verbose: {}", args.verbose);
}
