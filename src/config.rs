pub fn parse_bind_address(bind_addr: &str) -> Result<(String, u16), String> {
    let default_host = "localhost";
    let default_port = 8000;

    if bind_addr.is_empty() {
        return Ok((default_host.to_string(), default_port));
    }

    if bind_addr.starts_with('[') && bind_addr.contains("]:") {
        let parts: Vec<&str> = bind_addr.splitn(2, "]:").collect();
        if parts.len() == 2 {
            let ipv6_addr = parts[0].trim_start_matches('[');
            let port: u16 = parts[1].parse().map_err(|_| "Invalid port number")?;
            return Ok((ipv6_addr.to_string(), port));
        } else {
            return Ok((default_host.to_string(), default_port));
        }
    } else if bind_addr.starts_with(':') {
        let port: u16 = bind_addr[1..].parse().map_err(|_| "Invalid port number")?;
        return Ok(("0.0.0.0".to_string(), port));
    } else if bind_addr.contains(':') {
        let parts: Vec<&str> = bind_addr.rsplitn(2, ':').collect();
        if parts.len() == 2 {
            let host_part = parts[1];
            let port: u16 = parts[0].parse().map_err(|_| "Invalid port number")?;
            if host_part == "*" {
                return Ok(("[::]".to_string(), port));
            }
            return Ok((host_part.to_string(), port));
        } else {
            return Ok((default_host.to_string(), default_port));
        }
    } else {
        let port = bind_addr.parse().map_err(|_| "Invalid port number")?;
        return Ok((default_host.to_string(), port));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_bind_empty() {
        let (host, port) = parse_bind_address("").unwrap();
        assert_eq!(host, "localhost");
        assert_eq!(port, 8000);
    }

    #[test]
    fn test_parse_bind_port_number() {
        let (host, port) = parse_bind_address("8080").unwrap();
        assert_eq!(host, "localhost");
        assert_eq!(port, 8080);
    }

    #[test]
    fn test_parse_bind_port_colon() {
        let (host, port) = parse_bind_address(":8080").unwrap();
        assert_eq!(host, "0.0.0.0");
        assert_eq!(port, 8080);
    }

    #[test]
    fn test_parse_bind_star() {
        let (host, port) = parse_bind_address("*:8080").unwrap();
        assert_eq!(host, "[::]");
        assert_eq!(port, 8080);
    }

    #[test]
    fn test_parse_bind_host_port() {
        let (host, port) = parse_bind_address("127.0.0.1:9000").unwrap();
        assert_eq!(host, "127.0.0.1");
        assert_eq!(port, 9000);
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
}
