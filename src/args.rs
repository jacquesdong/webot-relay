use clap::{Parser, Subcommand};

use crate::i18n;

#[derive(Parser, Debug)]
#[command(
    name = "webot-relay",
    disable_version_flag = true,
    disable_help_subcommand = true
)]
#[command(about = i18n::welcome::description().to_string(), long_about = None)]
pub struct Args {
    #[command(subcommand)]
    pub command: Option<Command>,

    #[arg(short, long, help = i18n::options::version::description().to_string())]
    pub version: bool,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    #[command(disable_help_subcommand = true, about = i18n::commands::service::description().to_string())]
    Service {
        #[command(subcommand)]
        command: ServiceCommand,
    },

    #[command(about = i18n::commands::run::description().to_string())]
    Run {
        #[arg(
            short,
            long,
            default_value = ":8000",
            help = i18n::commands::run::bind::description().to_string()
        )]
        bind: String,

        #[arg(long, help = i18n::commands::run::url::description().to_string())]
        url: Option<String>,

        #[arg(long, help = i18n::commands::run::verbose::description().to_string())]
        verbose: bool,
    },
}

#[derive(Subcommand, Debug)]
#[command(disable_help_subcommand = true)]
pub enum ServiceCommand {
    #[command(about = i18n::commands::service::install::description().to_string())]
    Install {
        #[arg(
            short,
            long,
            default_value = ":8000",
            help = i18n::commands::run::bind::description().to_string()
        )]
        bind: String,

        #[arg(long, help = i18n::commands::run::url::description().to_string())]
        url: Option<String>,

        #[arg(short, long, help = i18n::commands::run::verbose::description().to_string())]
        verbose: bool,
    },
    #[command(about = i18n::commands::service::start::description().to_string())]
    Start,
    #[command(about = i18n::commands::service::stop::description().to_string())]
    Stop,
    #[command(about = i18n::commands::service::status::description().to_string())]
    Status,
    #[command(about = i18n::commands::service::uninstall::description().to_string())]
    Uninstall,
}

impl Args {
    pub fn get_version(&self) -> &'static str {
        env!("BUILD_VERSION")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_bind() {
        let args = Args::parse_from(&["webot-relay", "run"]);
        match &args.command {
            Some(Command::Run { bind, .. }) => assert_eq!(bind, ":8000"),
            _ => panic!("Expected Run command"),
        }
    }

    #[test]
    fn test_custom_bind() {
        let args = Args::parse_from(&["webot-relay", "run", "--bind", ":8080"]);
        match &args.command {
            Some(Command::Run { bind, .. }) => assert_eq!(bind, ":8080"),
            _ => panic!("Expected Run command"),
        }
    }

    #[test]
    fn test_version_flag() {
        let args = Args::parse_from(&["webot-relay", "--version"]);
        assert!(args.version);
        assert_eq!(args.get_version(), env!("BUILD_VERSION"));

        let args = Args::parse_from(&["webot-relay", "-v"]);
        assert!(args.version);
        assert_eq!(args.get_version(), env!("BUILD_VERSION"));
    }

    #[test]
    fn test_verbose_flag() {
        let args = Args::parse_from(&["webot-relay", "run", "--verbose"]);
        match &args.command {
            Some(Command::Run { verbose, .. }) => assert!(verbose),
            _ => panic!("Expected Run command"),
        }
    }

    #[test]
    fn test_url_option() {
        let args = Args::parse_from(&["webot-relay", "run", "--url", "http://example.com"]);
        match &args.command {
            Some(Command::Run { url, .. }) => {
                assert_eq!(*url, Some("http://example.com".to_string()))
            }
            _ => panic!("Expected Run command"),
        }
    }

    #[test]
    fn test_service_install() {
        let args = Args::parse_from(&[
            "webot-relay",
            "service",
            "install",
            "--bind",
            ":8000",
            "--url",
            "http://example.com",
        ]);
        match &args.command {
            Some(Command::Service {
                command: ServiceCommand::Install { bind, url, .. },
            }) => {
                assert_eq!(bind, ":8000");
                assert_eq!(*url, Some("http://example.com".to_string()));
            }
            _ => panic!("Expected Service Install command"),
        }
    }

    #[test]
    fn test_service_start() {
        let args = Args::parse_from(&["webot-relay", "service", "start"]);
        match &args.command {
            Some(Command::Service {
                command: ServiceCommand::Start,
            }) => {}
            _ => panic!("Expected Service Start command"),
        }
    }

    #[test]
    fn test_service_stop() {
        let args = Args::parse_from(&["webot-relay", "service", "stop"]);
        match &args.command {
            Some(Command::Service {
                command: ServiceCommand::Stop,
            }) => {}
            _ => panic!("Expected Service Stop command"),
        }
    }

    #[test]
    fn test_service_status() {
        let args = Args::parse_from(&["webot-relay", "service", "status"]);
        match &args.command {
            Some(Command::Service {
                command: ServiceCommand::Status,
            }) => {}
            _ => panic!("Expected Service Status command"),
        }
    }

    #[test]
    fn test_service_uninstall() {
        let args = Args::parse_from(&["webot-relay", "service", "uninstall"]);
        match &args.command {
            Some(Command::Service {
                command: ServiceCommand::Uninstall,
            }) => {}
            _ => panic!("Expected Service Uninstall command"),
        }
    }
}
