use rust_i18n::{i18n, t};
i18n!("locales", fallback = "en");

mod args;
mod config;
mod response;
mod server;
mod service;

use args::{Args, Command, ServiceCommand};
use axum;
use clap::{CommandFactory, Parser};
use config::parse_bind_address;
use server::{AppState, create_app};
use service::get_service_manager;
use std::net::ToSocketAddrs;
use std::sync::Arc;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    let locale = sys_locale::get_locale().unwrap_or_else(|| String::from("en"));
    rust_i18n::set_locale(&locale);

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
        eprintln!("{}", t!("errors.parsing_address", error = err));
        std::process::exit(1);
    });

    let socket_addr = format!("{}:{}", host, port)
        .to_socket_addrs()
        .unwrap_or_else(|err| {
            eprintln!("{}", t!("errors.invalid_ip", host = bind, error = err));
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
        eprintln!("{}", t!("errors.binding_address", error = err));
        std::process::exit(1);
    });

    if is_relay {
        eprintln!(
            "{}",
            t!("messages.server_running_relay", address = socket_addr)
        );
    } else {
        eprintln!(
            "{}",
            t!("messages.server_running_dumb", address = socket_addr)
        );
    }

    axum::serve(listener, app).await.unwrap_or_else(|err| {
        eprintln!("{}", t!("errors.starting_server", error = err));
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
                Ok(_) => println!("{}", t!("messages.service_installed")),
                Err(e) => eprintln!("{}", t!("errors.installing_service", error = e)),
            }
        }
        ServiceCommand::Start => match service_manager.start() {
            Ok(_) => println!("{}", t!("messages.service_started")),
            Err(e) => eprintln!("{}", t!("errors.starting_service", error = e)),
        },
        ServiceCommand::Stop => match service_manager.stop() {
            Ok(_) => println!("{}", t!("messages.service_stopped")),
            Err(e) => eprintln!("{}", t!("errors.stopping_service", error = e)),
        },
        ServiceCommand::Status => match service_manager.status() {
            Ok(status) => println!("{}", status),
            Err(e) => eprintln!("{}", t!("errors.getting_status", error = e)),
        },
        ServiceCommand::Uninstall => match service_manager.uninstall() {
            Ok(_) => println!("{}", t!("messages.service_uninstalled")),
            Err(e) => eprintln!("{}", t!("errors.uninstalling_service", error = e)),
        },
    }
}
