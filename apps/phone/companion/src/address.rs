//! The host address as the user types it in Settings.

use crate::api::DEFAULT_PORT;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HostAddress {
    pub host: String,
    pub port: u16,
}

impl HostAddress {
    /// Reads `desktop`, `desktop:8790`, `http://100.64.0.1:8790/` or `[fd7a::1]:8790`.
    /// Without a port it is [`DEFAULT_PORT`].
    pub fn parse(text: &str) -> Result<Self, String> {
        let mut rest = text.trim();
        if let Some(after) = strip_scheme(rest, "http://") {
            rest = after;
        }
        if strip_scheme(rest, "https://").is_some() {
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
            if addr.parse::<std::net::Ipv6Addr>().is_err() {
                return Err(tr!("address-bad-host", text = text.trim()));
            }
            (format!("[{addr}]"), port)
        } else if rest.matches(':').count() > 1 {
            // An IPv6 address without brackets cannot say where a port starts:
            // fd7a::1:8790 is itself an address.
            return Err(tr!("address-ipv6-brackets"));
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
}

/// What follows `scheme` at the start of `text`, the scheme matched in any case, as a
/// browser matches it.
fn strip_scheme<'a>(text: &'a str, scheme: &str) -> Option<&'a str> {
    let head = text.get(..scheme.len())?;
    head.eq_ignore_ascii_case(scheme)
        .then(|| &text[scheme.len()..])
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
        let a = HostAddress::parse("10.0.2.2:8790").unwrap();
        assert_eq!(a.base_url(), "http://10.0.2.2:8790");
        let a = HostAddress::parse("[fd7a:115c::1]:8790").unwrap();
        assert_eq!(a.base_url(), "http://[fd7a:115c::1]:8790");
        let a = HostAddress::parse("[fd7a:115c::1]").unwrap();
        assert_eq!(a.port, 8790);
        let a = HostAddress::parse("Http://desktop:8791").unwrap();
        assert_eq!(a.display(), "desktop:8791");
        assert_eq!(
            HostAddress::parse("HTTP://desktop").unwrap().host,
            "desktop"
        );
    }

    #[test]
    fn says_what_is_wrong() {
        assert!(HostAddress::parse("").is_err());
        assert!(HostAddress::parse("desktop:http").is_err());
        assert!(HostAddress::parse("desktop:0").is_err());
        let https = HostAddress::parse("https://desktop").unwrap_err();
        assert_eq!(HostAddress::parse("HTTPS://desktop"), Err(https));
        assert!(HostAddress::parse("my desk").is_err());
        // Without brackets an IPv6 address and its port cannot be told apart.
        let brackets = HostAddress::parse("fd7a:115c::1").unwrap_err();
        assert_eq!(
            HostAddress::parse("fd7a:115c::1:8790"),
            Err(brackets.clone())
        );
        assert_eq!(HostAddress::parse("fe80::1%wlan0"), Err(brackets));
        assert!(HostAddress::parse("[]:8790").is_err());
        assert!(HostAddress::parse("[desktop]:8790").is_err());
    }
}
