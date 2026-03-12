use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "webot-relay")]
#[command(about = "HTTP Relay Server - Forward POST requests to target URL", long_about = None)]
pub struct Args {
    #[arg(
        short,
        long,
        default_value = ":8000",
        help = "Address to bind to (e.g., :8000 or localhost:8000)"
    )]
    pub bind: String,

    #[arg(long, help = "URL to relay requests to")]
    pub url: Option<String>,

    #[arg(short, long, help = "Enable verbose logging")]
    pub verbose: bool,
}

impl Args {
    pub fn get_url(&self) -> Option<String> {
        if self.url.is_some() {
            self.url.clone()
        } else {
            std::env::var("WEBOT_URL").ok()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_bind() {
        let args = Args::parse_from(&["webot-relay"]);
        assert_eq!(args.bind, ":8000");
    }

    #[test]
    fn test_custom_bind() {
        let args = Args::parse_from(&["webot-relay", "--bind", ":8080"]);
        assert_eq!(args.bind, ":8080");
    }

    #[test]
    fn test_verbose_flag() {
        let args = Args::parse_from(&["webot-relay", "-v"]);
        assert!(args.verbose);
    }

    #[test]
    fn test_url_option() {
        let args = Args::parse_from(&["webot-relay", "--url", "http://example.com"]);
        assert_eq!(args.url, Some("http://example.com".to_string()));
    }
}
