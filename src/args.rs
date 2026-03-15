use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(
    name = "webot-relay",
    disable_version_flag = true,
    disable_help_subcommand = true
)]
#[command(about = "Webot Relay Server | 企业微信通知机器人转发服务", long_about = None)]
pub struct Args {
    #[command(subcommand)]
    pub command: Option<Command>,

    #[arg(short, long, help = "Print version")]
    pub version: bool,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Manage the relay server service | 服务管理命令（安装、启动、停止、状态、卸载）
    #[command(disable_help_subcommand = true)]
    Service {
        #[command(subcommand)]
        command: ServiceCommand,
    },

    /// Run the relay server instance | 直接在前台运行 HTTP 中继服务器
    Run {
        #[arg(
            short,
            long,
            default_value = "localhost:8000",
            help = "Address to bind to (e.g., :8000 or localhost:8000) | 绑定地址（例如：:8000 或 localhost:8000）"
        )]
        bind: String,

        #[arg(long, help = "URL to relay requests to | 转发请求到的 URL")]
        url: Option<String>,

        #[arg(long, help = "Enable verbose logging | 启用详细日志")]
        verbose: bool,
    },
}

#[derive(Subcommand, Debug)]
#[command(disable_help_subcommand = true)]
pub enum ServiceCommand {
    /// 安装服务 | Install service
    Install {
        #[arg(
            short,
            long,
            default_value = "localhost:8000",
            help = "Address to bind to (e.g., :8000 or localhost:8000) | 绑定地址（例如：:8000 或 localhost:8000）"
        )]
        bind: String,

        #[arg(long, help = "URL to relay requests to | 转发请求到的 URL")]
        url: Option<String>,

        #[arg(short, long, help = "Enable verbose logging | 启用详细日志")]
        verbose: bool,
    },
    /// 启动服务 | Start service
    Start,
    /// 停止服务 | Stop service
    Stop,
    /// 查看服务 | Check service status
    Status,
    /// 卸载服务 | Uninstall service
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
