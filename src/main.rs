mod args;
mod config;
mod response;
mod server;
mod service_manager;

use args::{Args, Command, ServiceCommand};
use axum;
use clap::{CommandFactory, Parser};
use config::parse_bind_address;
use server::{AppState, create_app};
use service_manager::get_service_manager;
use std::net::ToSocketAddrs;
use std::sync::Arc;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    let args = Args::parse();

    if args.version {
        println!("webot-relay {}", args.get_version());
        std::process::exit(0);
    }

    match args.command {
        Some(Command::Service {
            command: service_cmd,
        }) => {
            handle_service_command(service_cmd);
        }
        Some(Command::Run { bind, url, verbose }) => {
            handle_run_command(bind, url, verbose).await;
        }
        None => {
            Args::command().print_help().unwrap();
            std::process::exit(0);
        }
    }
}

async fn handle_run_command(bind: String, url: Option<String>, verbose: bool) {
    let (host, port) = parse_bind_address(&bind).unwrap_or_else(|err| {
        eprintln!("Error parsing address: {}", err);
        std::process::exit(1);
    });

    let socket_addr = format!("{}:{}", host, port)
        .to_socket_addrs()
        .unwrap_or_else(|err| {
            eprintln!("Invalid address: {}, {}", bind, err);
            std::process::exit(1);
        })
        .next()
        .unwrap();

    let url = if url.is_some() {
        url
    } else {
        std::env::var("WEBOT_URL").ok()
    };

    let is_relay = url.is_some();

    let state = Arc::new(AppState { url, verbose });

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

fn handle_service_command(cmd: ServiceCommand) {
    let service_manager = get_service_manager();

    match cmd {
        ServiceCommand::Install { bind, url, verbose } => {
            // 构建启动参数
            let mut args = vec![];
            args.push(format!("--bind={}", bind));
            if let Some(url) = url {
                args.push(format!("--url={}", url));
            }
            if verbose {
                args.push("--verbose".to_string());
            }
            let args_str = args.join(" ");

            match service_manager.install(&args_str) {
                Ok(_) => println!("Service installed successfully"),
                Err(e) => eprintln!("Failed to install service: {}", e),
            }
        }
        ServiceCommand::Start => match service_manager.start() {
            Ok(_) => println!("Service started successfully"),
            Err(e) => eprintln!("Failed to start service: {}", e),
        },
        ServiceCommand::Stop => match service_manager.stop() {
            Ok(_) => println!("Service stopped successfully"),
            Err(e) => eprintln!("Failed to stop service: {}", e),
        },
        ServiceCommand::Status => match service_manager.status() {
            Ok(status) => println!("{}", status),
            Err(e) => eprintln!("Failed to get service status: {}", e),
        },
        ServiceCommand::Uninstall => match service_manager.uninstall() {
            Ok(_) => println!("Service uninstalled successfully"),
            Err(e) => eprintln!("Failed to uninstall service: {}", e),
        },
    }
}
