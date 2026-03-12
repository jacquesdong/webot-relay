use std::net::SocketAddr;
use std::str::FromStr;

pub fn parse_bind_address(bind_addr: &str) -> Result<(String, u16), String> {
    let default_port = 8000;

    if bind_addr.is_empty() {
        return Ok(("".to_string(), default_port));
    }

    if bind_addr.starts_with('[') && bind_addr.contains("]:") {
        // IPv6 address format: [::1]:8000
        let parts: Vec<&str> = bind_addr.splitn(2, "]:").collect();
        if parts.len() == 2 {
            let ipv6_addr = parts[0].trim_start_matches('[');
            let port: u16 = parts[1].parse().map_err(|_| "Invalid port number")?;
            return Ok((ipv6_addr.to_string(), port));
        } else {
            return Ok(("".to_string(), default_port));
        }
    } else if bind_addr.contains(':') {
        let parts: Vec<&str> = bind_addr.rsplitn(2, ':').collect();
        if parts.len() == 2 {
            let port: u16 = parts[0].parse().map_err(|_| "Invalid port number")?;
            return Ok((parts[1].to_string(), port));
        } else {
            return Ok(("".to_string(), default_port));
        }
    } else {
        if let Ok(port) = bind_addr.parse::<u16>() {
            return Ok(("".to_string(), port));
        } else {
            return Ok((bind_addr.to_string(), default_port));
        }
    }
}

pub fn validate_port(port: u16) -> Result<(), String> {
    if port < 1 {
        return Err("Port must be between 1 and 65535".to_string());
    }
    Ok(())
}

pub fn parse_bind_address_to_socket_addr(bind_addr: &str) -> Result<SocketAddr, String> {
    let (host, port) = parse_bind_address(bind_addr)?;
    validate_port(port)?;

    let ip = if host.is_empty() {
        "0.0.0.0".parse().unwrap()
    } else {
        std::net::IpAddr::from_str(&host).map_err(|_| "Invalid IP address")?
    };

    Ok(SocketAddr::new(ip, port))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_bind_empty() {
        let (host, port) = parse_bind_address("").unwrap();
        assert_eq!(host, "");
        assert_eq!(port, 8000);
    }

    #[test]
    fn test_parse_bind_port_number() {
        let (host, port) = parse_bind_address("8080").unwrap();
        assert_eq!(host, "");
        assert_eq!(port, 8080);
    }

    #[test]
    fn test_parse_bind_port_colon() {
        let (host, port) = parse_bind_address(":8080").unwrap();
        assert_eq!(host, "");
        assert_eq!(port, 8080);
    }

    #[test]
    fn test_parse_bind_host_port() {
        let (host, port) = parse_bind_address("127.0.0.1:9000").unwrap();
        assert_eq!(host, "127.0.0.1");
        assert_eq!(port, 9000);
    }

    #[test]
    fn test_validate_port_valid() {
        assert!(validate_port(8080).is_ok());
        assert!(validate_port(1).is_ok());
        assert!(validate_port(65535).is_ok());
    }

    #[test]
    fn test_validate_port_invalid() {
        assert!(validate_port(0).is_err());
    }

    #[test]
    fn test_parse_bind_ipv6_address() {
        let (host, port) = parse_bind_address("[::1]:8000").unwrap();
        assert_eq!(host, "::1");
        assert_eq!(port, 8000);

        let (host, port) = parse_bind_address("[2001:db8::1]:9000").unwrap();
        assert_eq!(host, "2001:db8::1");
        assert_eq!(port, 9000);

        let (host, port) = parse_bind_address("[fe80::1%eth0]:8080").unwrap();
        assert_eq!(host, "fe80::1%eth0");
        assert_eq!(port, 8080);
    }

    #[test]
    fn test_parse_bind_address_to_socket_addr_ipv6() {
        let socket_addr = parse_bind_address_to_socket_addr("[::1]:8000").unwrap();
        assert_eq!(socket_addr.to_string(), "[::1]:8000");
    }
}
