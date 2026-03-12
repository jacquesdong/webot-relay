mod args;
mod config;
mod response;
mod server;

use args::Args;
use axum;
use clap::Parser;
use config::parse_bind_address;
use server::{AppState, create_app};
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let (host, port) = parse_bind_address(&args.bind).unwrap_or_else(|err| {
        eprintln!("Error parsing address: {}", err);
        std::process::exit(1);
    });

    let ip = std::net::IpAddr::from_str(&host).unwrap_or_else(|err| {
        eprintln!("Invalid IP address: {}, {}", host, err);
        std::process::exit(1);
    });

    let socket_addr = SocketAddr::new(ip, port);

    let url = args.get_url();

    let is_relay = url.is_some();

    let state = Arc::new(AppState {
        url,
        verbose: args.verbose,
    });

    let app = create_app(state);

    let listener = TcpListener::bind(socket_addr).await.unwrap_or_else(|err| {
        eprintln!("Error binding address: {}", err);
        std::process::exit(1);
    });

    if is_relay {
        eprintln!("Server running on {:?} [relay]", socket_addr);
    } else {
        eprintln!("Server running on {:?} [dumb]", socket_addr);
    }

    axum::serve(listener, app).await.unwrap_or_else(|err| {
        eprintln!("Error starting server: {}", err);
        std::process::exit(1);
    });
}
