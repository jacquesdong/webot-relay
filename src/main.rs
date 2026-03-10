mod args;
mod config;
mod response;
mod server;

use clap::Parser;
use args::Args;
use config::{parse_bind_address, validate_port, parse_bind_address_to_socket_addr};
use server::{create_app, AppState};
use std::sync::Arc;
use tokio::net::TcpListener;
use axum;

#[tokio::main]
async fn main() {
    let args = Args::parse();
    
    let (host, port) = parse_bind_address(&args.bind)
        .unwrap_or_else(|err| {
            eprintln!("Error parsing address: {}", err);
            std::process::exit(1);
        });
    
    validate_port(port)
        .unwrap_or_else(|err| {
            eprintln!("Error: {}", err);
            std::process::exit(1);
        });
    
    let socket_addr = parse_bind_address_to_socket_addr(&args.bind)
        .unwrap_or_else(|err| {
            eprintln!("Error: {}", err);
            std::process::exit(1);
        });
    
    let url = args.get_url();
    
    if args.verbose {
        eprintln!("Server running on {}:{}", if host.is_empty() { "0.0.0.0" } else { &host }, port);
    }
    
    let state = Arc::new(AppState {
        url,
        verbose: args.verbose,
    });
    
    let app = create_app(state);
    
    let listener = TcpListener::bind(socket_addr).await
        .unwrap_or_else(|err| {
            eprintln!("Error binding address: {}", err);
            std::process::exit(1);
        });
    
    eprintln!("Server listening on {:?}", socket_addr);
    
    axum::serve(listener, app).await
        .unwrap_or_else(|err| {
            eprintln!("Error starting server: {}", err);
            std::process::exit(1);
        });
}
