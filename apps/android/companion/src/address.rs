//! The host address as the user types it in Settings.

use crate::api::DEFAULT_PORT;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HostAddress {
    pub host: String,
    pub port: u16,
}

impl HostAddress {
    /// Reads `desktop`, `desktop:8790`, `http://100.64.0.1:8790/`, `[fd7a::1]:8790` or
    /// `fd7a::1`. Without a port it is [`DEFAULT_PORT`].
    pub fn parse(text: &str) -> Result<Self, String> {
        let mut rest = text.trim();
        // A scheme is read in any case, the way a browser reads it.
        let scheme = |rest: &str, name: &str| {
            rest.get(..name.len())
                .is_some_and(|s| s.eq_ignore_ascii_case(name))
        };
        if scheme(rest, "http://") {
            rest = &rest["http://".len()..];
        }
        if scheme(rest, "https://") {
            return Err(tr!("address-https"));
        }
        let rest = rest.split('/').next().unwrap_or("");
        if rest.is_empty() {
            return Err(tr!("address-empty"));
        }

        let (host, port) = if let Some(inner) = rest.strip_prefix('[') {
            // [v6]:port
            let (addr, after) = inner
                .split_once(']')
                .ok_or_else(|| tr!("address-ipv6-close"))?;
            let port = match after.strip_prefix(':') {
                Some(p) => Some(p),
                None if after.is_empty() => None,
                None => return Err(tr!("address-ipv6-port")),
            };
            (format!("[{addr}]"), port)
        } else if rest.parse::<std::net::Ipv6Addr>().is_ok() {
            // An IPv6 address without brackets has no room for a port.
            (format!("[{rest}]"), None)
        } else {
            match rest.rsplit_once(':') {
                Some((h, p)) => (h.to_string(), Some(p)),
                None => (rest.to_string(), None),
            }
        };

        if host.is_empty() || host.contains(char::is_whitespace) {
            return Err(tr!("address-bad-host", text = text.trim()));
        }
        let port = match port {
            None => DEFAULT_PORT,
            Some(p) => p
                .parse::<u16>()
                .ok()
                .filter(|p| *p > 0)
                .ok_or_else(|| tr!("address-bad-port", port = p))?,
        };
        Ok(HostAddress { host, port })
    }

    pub fn base_url(&self) -> String {
        format!("http://{}:{}", self.host, self.port)
    }

    /// As it reads back in Settings.
    pub fn display(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }

    /// The machine part for the status line before the host has said its own name.
    pub fn short_name(&self) -> String {
        if self.host.starts_with('[') || self.host.parse::<std::net::Ipv4Addr>().is_ok() {
            return self.host.clone();
        }
        self.host
            .split('.')
            .next()
            .unwrap_or(&self.host)
            .to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_the_forms_people_type() {
        let a = HostAddress::parse("desktop").unwrap();
        assert_eq!((a.host.as_str(), a.port), ("desktop", 8790));
        let a = HostAddress::parse(" http://desktop.example.ts.net:8791/ ").unwrap();
        assert_eq!(a.display(), "desktop.example.ts.net:8791");
        assert_eq!(a.short_name(), "desktop");
        let a = HostAddress::parse("10.0.2.2:8790").unwrap();
        assert_eq!(a.base_url(), "http://10.0.2.2:8790");
        assert_eq!(a.short_name(), "10.0.2.2");
        let a = HostAddress::parse("[fd7a:115c::1]:8790").unwrap();
        assert_eq!(a.base_url(), "http://[fd7a:115c::1]:8790");
        let a = HostAddress::parse("[fd7a:115c::1]").unwrap();
        assert_eq!(a.port, 8790);
        let a = HostAddress::parse("fd7a:115c::1").unwrap();
        assert_eq!(a.base_url(), "http://[fd7a:115c::1]:8790");
        let a = HostAddress::parse("Http://desktop:8791").unwrap();
        assert_eq!(a.display(), "desktop:8791");
        assert_eq!(HostAddress::parse("HTTP://desktop").unwrap().host, "desktop");
    }

    #[test]
    fn says_what_is_wrong() {
        assert!(HostAddress::parse("").is_err());
        assert!(HostAddress::parse("desktop:http").is_err());
        assert!(HostAddress::parse("desktop:0").is_err());
        let https = HostAddress::parse("https://desktop").unwrap_err();
        assert_eq!(HostAddress::parse("HTTPS://desktop"), Err(https));
        assert!(HostAddress::parse("my desk").is_err());
    }
}
