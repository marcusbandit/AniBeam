//! The host address as the user types it in Settings.

use crate::api::DEFAULT_PORT;

/// What a fresh install points at: the owner's desktop, by its tailnet MagicDNS name.
pub const DEFAULT_ADDRESS: &str = "banditbox:8790";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HostAddress {
    pub host: String,
    pub port: u16,
}

impl HostAddress {
    /// Reads `banditbox`, `banditbox:8790`, `http://100.64.0.1:8790/` or `[fd7a::1]:8790`.
    /// Without a port it is [`DEFAULT_PORT`].
    pub fn parse(text: &str) -> Result<Self, String> {
        let mut rest = text.trim();
        for scheme in ["http://", "HTTP://"] {
            if let Some(stripped) = rest.strip_prefix(scheme) {
                rest = stripped;
            }
        }
        if rest.starts_with("https://") {
            return Err("Your computer's AniBeam answers on plain http over Tailscale; drop the https://.".into());
        }
        let rest = rest.split('/').next().unwrap_or("");
        if rest.is_empty() {
            return Err("Type the host's name or address, for example banditbox:8790.".into());
        }

        let (host, port) = if let Some(inner) = rest.strip_prefix('[') {
            // [v6]:port
            let (addr, after) = inner
                .split_once(']')
                .ok_or_else(|| "An IPv6 address needs its closing ].".to_string())?;
            let port = match after.strip_prefix(':') {
                Some(p) => Some(p),
                None if after.is_empty() => None,
                None => return Err("Put the port after the ], as [fd7a::1]:8790.".into()),
            };
            (format!("[{addr}]"), port)
        } else {
            match rest.rsplit_once(':') {
                Some((h, p)) => (h.to_string(), Some(p)),
                None => (rest.to_string(), None),
            }
        };

        if host.is_empty() || host.contains(char::is_whitespace) {
            return Err(format!("\"{}\" is not a host name.", text.trim()));
        }
        let port = match port {
            None => DEFAULT_PORT,
            Some(p) => p
                .parse::<u16>()
                .ok()
                .filter(|p| *p > 0)
                .ok_or_else(|| format!("\"{p}\" is not a port."))?,
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
        self.host.split('.').next().unwrap_or(&self.host).to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_the_forms_people_type() {
        let a = HostAddress::parse("banditbox").unwrap();
        assert_eq!((a.host.as_str(), a.port), ("banditbox", 8790));
        let a = HostAddress::parse(" http://banditbox.tail24c6a9.ts.net:8791/ ").unwrap();
        assert_eq!(a.display(), "banditbox.tail24c6a9.ts.net:8791");
        assert_eq!(a.short_name(), "banditbox");
        let a = HostAddress::parse("10.0.2.2:8790").unwrap();
        assert_eq!(a.base_url(), "http://10.0.2.2:8790");
        assert_eq!(a.short_name(), "10.0.2.2");
        let a = HostAddress::parse("[fd7a:115c::1]:8790").unwrap();
        assert_eq!(a.base_url(), "http://[fd7a:115c::1]:8790");
        let a = HostAddress::parse("[fd7a:115c::1]").unwrap();
        assert_eq!(a.port, 8790);
    }

    #[test]
    fn says_what_is_wrong() {
        assert!(HostAddress::parse("").is_err());
        assert!(HostAddress::parse("banditbox:http").is_err());
        assert!(HostAddress::parse("banditbox:0").is_err());
        assert!(HostAddress::parse("https://banditbox").is_err());
        assert!(HostAddress::parse("bandit box").is_err());
    }
}
