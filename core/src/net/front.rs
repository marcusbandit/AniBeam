//! Domain fronting for a host some networks block by name. The name a phone asks for travels
//! in the clear at the start of every HTTPS connection (SNI), and a filter that reads it can
//! drop the connection while the address and DNS stay fine. nyaa.si sits behind DDoS-Guard,
//! whose edge answers for every site it carries: the connection goes to nyaa's own address
//! under DDoS-Guard's name, the certificate checked is DDoS-Guard's, and the site asked for
//! travels inside the encrypted request, in its Host header.

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use reqwest::dns::{Addrs, Name, Resolve, Resolving};

use super::doh::Doh;
use crate::contract::CoreError;

/// Host to the name its connections carry instead.
const FRONTS: [(&str, &str); 1] = [("nyaa.si", "ddos-guard.net")];

/// The URL to ask, with the front's name for the host, and the real host for the Host
/// header; None for a host that is not fronted.
pub fn fronted(url: &str) -> Option<(String, String)> {
    let (scheme, rest) = url.split_once("://")?;
    if scheme != "https" {
        return None;
    }
    let end = rest.find(['/', '?', '#']).unwrap_or(rest.len());
    let host = &rest[..end];
    let (_, front) = FRONTS.iter().find(|(h, _)| *h == host)?;
    Some((format!("https://{front}{}", &rest[end..]), host.to_string()))
}

/// Looks a front's name up as the host it stands for: the connection must reach that site's
/// own server, which also answers under the front's name.
#[derive(Clone)]
struct FrontResolve {
    doh: Doh,
}

impl Resolve for FrontResolve {
    fn resolve(&self, name: Name) -> Resolving {
        let doh = self.doh.clone();
        let asked = name.as_str().to_string();
        let real = FRONTS
            .iter()
            .find(|(_, f)| *f == asked)
            .map_or(asked.clone(), |(h, _)| h.to_string());
        Box::pin(async move {
            let ips = match doh.lookup(&real).await {
                Some(ips) => ips,
                None => tokio::net::lookup_host((real.as_str(), 0))
                    .await?
                    .map(|a| a.ip())
                    .collect(),
            };
            let addrs: Addrs = Box::new(ips.into_iter().map(|ip| SocketAddr::new(ip, 0)));
            Ok(addrs)
        })
    }
}

/// The client fronted requests go through. HTTP/1.1 only: there the Host header is the site
/// asked for, where HTTP/2 would take it from the URL. It follows no redirects, which would
/// leave the front.
/// `roots` are the phone's bundled roots; None keeps the platform's.
pub fn client(
    doh: &Doh,
    roots: Option<Vec<reqwest::Certificate>>,
    timeout: Duration,
) -> Result<reqwest::Client, CoreError> {
    let builder = reqwest::Client::builder()
        .user_agent(format!("AniBeam/{}", crate::VERSION))
        .timeout(timeout)
        .http1_only()
        .redirect(reqwest::redirect::Policy::none());
    let builder = match roots {
        Some(roots) => builder.tls_certs_only(roots),
        None => builder,
    };
    builder
        .dns_resolver(Arc::new(FrontResolve { doh: doh.clone() }))
        .build()
        .map_err(|e| CoreError::internal(format!("fronted client: {e}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_a_fronted_https_host_is_rewritten() {
        assert_eq!(
            fronted("https://nyaa.si/?page=rss&q=frieren"),
            Some((
                "https://ddos-guard.net/?page=rss&q=frieren".to_string(),
                "nyaa.si".to_string()
            ))
        );
        assert_eq!(
            fronted("https://nyaa.si"),
            Some(("https://ddos-guard.net".to_string(), "nyaa.si".to_string()))
        );
        assert_eq!(fronted("http://nyaa.si/"), None);
        assert_eq!(fronted("https://animetosho.org/feed/json?q=x"), None);
        assert_eq!(fronted("https://sub.nyaa.si/"), None);
    }

    /// Fetches nyaa's feed through the front, from wherever the test runs.
    #[tokio::test]
    #[ignore = "reaches the network"]
    async fn nyaa_answers_through_its_front() {
        let doh = Doh::new(reqwest::Client::new());
        let c = client(&doh, None, Duration::from_secs(20)).unwrap();
        let (url, host) = fronted("https://nyaa.si/?page=rss&q=frieren").unwrap();
        let r = c.get(url).header("Host", host).send().await.unwrap();
        assert_eq!(r.status().as_u16(), 200);
        assert!(r.text().await.unwrap().contains("<rss"));
    }
}
