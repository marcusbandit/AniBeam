//! Domain fronting for a host some networks block by name. The name a phone asks for travels
//! in the clear at the start of every HTTPS connection (SNI), and a filter that reads it can
//! drop the connection while the address and DNS stay fine. nyaa.si sits behind DDoS-Guard,
//! whose edge answers for every site it carries: the connection goes to nyaa's own address
//! under DDoS-Guard's name, the certificate checked is DDoS-Guard's, and the site asked for
//! travels inside the encrypted request, in its Host header.

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};

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

/// The gap between the end of one fronted request and the start of the next. DDoS-Guard, in
/// front of nyaa, quietly stops answering an address that asks for dozens of pages within
/// seconds, which a new subscription or a batch pick on a long show used to do (#135).
#[cfg_attr(not(target_os = "android"), allow(dead_code))]
pub const GAP: Duration = Duration::from_secs(1);
/// How long a front is left alone after a request it did not answer, so an address it has
/// stopped answering is not asked again and again, each time for a whole timeout.
#[cfg_attr(not(target_os = "android"), allow(dead_code))]
pub const QUIET: Duration = Duration::from_secs(120);

/// Paces the requests to fronted hosts. Held for the whole of one request, so they go one at
/// a time, `GAP` apart, and none during `QUIET` after one that went unanswered.
#[derive(Default)]
#[cfg_attr(not(target_os = "android"), allow(dead_code))]
pub struct Gate {
    done: Option<Instant>,
    quiet_until: Option<Instant>,
}

#[cfg_attr(not(target_os = "android"), allow(dead_code))]
impl Gate {
    /// How long a request at `now` waits before it goes; Err with what is left of the
    /// quiet when it must not go at all.
    pub fn admit(&self, now: Instant) -> Result<Duration, Duration> {
        if let Some(until) = self.quiet_until
            && now < until
        {
            return Err(until - now);
        }
        Ok(self.done.map_or(Duration::ZERO, |done| {
            (done + GAP).saturating_duration_since(now)
        }))
    }

    /// A request ended at `now`; `answered` when the server sent a response, whatever its
    /// status.
    pub fn finished(&mut self, now: Instant, answered: bool) {
        self.done = Some(now);
        self.quiet_until = (!answered).then(|| now + QUIET);
    }
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

    #[test]
    fn the_first_request_goes_at_once() {
        assert_eq!(Gate::default().admit(Instant::now()), Ok(Duration::ZERO));
    }

    #[test]
    fn a_request_right_after_another_waits_out_the_gap() {
        let mut g = Gate::default();
        let now = Instant::now();
        g.finished(now - Duration::from_millis(300), true);
        assert_eq!(g.admit(now), Ok(Duration::from_millis(700)));
        g.finished(now - Duration::from_secs(5), true);
        assert_eq!(g.admit(now), Ok(Duration::ZERO));
    }

    #[test]
    fn the_gap_counts_from_the_end_of_a_slow_request() {
        let mut g = Gate::default();
        let start = Instant::now();
        // Started long ago, answered just now: the next still waits the whole gap.
        let end = start + Duration::from_secs(30);
        g.finished(end, true);
        assert_eq!(g.admit(end), Ok(GAP));
    }

    #[test]
    fn an_unanswered_request_keeps_the_front_quiet_for_a_while() {
        let mut g = Gate::default();
        let now = Instant::now();
        g.finished(now, false);
        assert_eq!(
            g.admit(now + Duration::from_secs(20)),
            Err(QUIET - Duration::from_secs(20))
        );
        assert_eq!(g.admit(now + QUIET), Ok(Duration::ZERO));
        g.finished(now + QUIET, true);
        assert_eq!(g.admit(now + QUIET + GAP), Ok(Duration::ZERO));
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
