//! Putting the snapshot together from what the core already has: the library
//! off one `Snapshot` load, the subscriptions and torrents off the
//! downloads module. Nothing here goes through the public `Call` round trip,
//! whose event-based reads carry no request ids.

use std::collections::HashMap;
use std::fmt::Write;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anibeam_host_api as wire;
use sha2::{Digest, Sha256};

use crate::Host;
use anibeam_contract::*;
use anibeam_downloads as downloads;
use anibeam_foundation::services::Services;
use anibeam_foundation::time;
use anibeam_library::snapshot::Snapshot;

/// How long one Downloads read may take before the snapshot goes without it.
pub const DOWNLOADS_LIMIT: Duration = Duration::from_secs(8);

/// A poster file the host will serve, and the tag the snapshot gave it.
#[derive(Clone, Debug, PartialEq)]
pub struct Poster {
    pub path: PathBuf,
    pub tag: String,
}

/// The library half of a snapshot, and the posters its tags name. Kept
/// together so a poster is served by the file the tag was made from.
#[derive(Clone, Debug, Default)]
pub struct LibraryPart {
    pub series: Vec<wire::Series>,
    pub posters: HashMap<u64, Poster>,
}

/// A short hash of a poster file's path, size and modification time: it
/// changes whenever the image does, which is what a client caches by.
pub fn poster_tag(path: &str, size: u64, modified: SystemTime) -> String {
    let since = modified
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::ZERO);
    let mut hash = Sha256::new();
    hash.update(path.as_bytes());
    hash.update([0]);
    hash.update(size.to_le_bytes());
    hash.update(since.as_nanos().to_le_bytes());
    let digest = hash.finalize();
    let mut out = String::with_capacity(10);
    for byte in &digest.as_slice()[..5] {
        let _ = write!(out, "{byte:02x}");
    }
    out
}

/// The tag of the file at `path`, or None when there is no such file.
fn tag_of(path: &str) -> Option<String> {
    let meta = std::fs::metadata(path).ok().filter(|m| m.is_file())?;
    Some(poster_tag(path, meta.len(), meta.modified().ok()?))
}

/// The whole library read once, on the reader connection, with one profile's
/// progress (#631). Blocking: the caller runs it off the async threads.
pub fn library_part(services: &Services, profile: u64) -> Result<LibraryPart, CoreError> {
    let images_dir = services.paths.images_dir();
    let now = time::now();
    let hosted = services
        .store
        .read(|c| Ok(Snapshot::load_for(c, profile, &images_dir, now, None)?.host_series()))?;
    let mut part = LibraryPart::default();
    for h in hosted {
        let mut series = h.series;
        if let Some(path) = h.poster_path
            && let Some(tag) = tag_of(&path)
        {
            series.poster = Some(tag.clone());
            part.posters.insert(
                series.id,
                Poster {
                    path: PathBuf::from(path),
                    tag,
                },
            );
        }
        part.series.push(series);
    }
    Ok(part)
}

/// The sentence for a Downloads half that could not be read.
pub fn problem(service: &str, error: &CoreError) -> String {
    match error {
        CoreError::Service { code, .. } if code == "timeout" => {
            format!("{service} did not answer")
        }
        CoreError::Service { code, .. } if code == "unauthorized" => {
            format!("{service} did not accept the login")
        }
        _ => format!("{service} is not running"),
    }
}

fn airing(a: &anibeam_contract::Airing) -> wire::Airing {
    wire::Airing {
        episode: a.episode,
        season: a.season,
        at: time::to_secs(a.at),
    }
}

pub fn subscriptions(rules: Vec<SubscriptionRule>) -> Vec<wire::Subscription> {
    rules
        .into_iter()
        .map(|r| {
            let show = r.show.as_ref();
            wire::Subscription {
                title: show.map_or_else(|| r.name.clone(), |s| s.title.clone()),
                series_id: show.map(|s| s.series_id),
                next_airing: show.and_then(|s| s.next_airing.as_ref()).map(airing),
                complete: show.is_some_and(|s| s.complete),
                missing: show.map(|s| s.missing.clone()).unwrap_or_default(),
                missing_season: show.and_then(|s| s.missing_season),
                name: r.name,
                feed_url: without_secrets(&r.feed_url),
                query: r.query,
                enabled: r.rule_enabled,
                torrent_count: r.torrent_count,
                file_count: r.file_count,
            }
        })
        .collect()
}

/// `url` without its credential parameters. A feed that needed one will not
/// open from a phone, which is the point; a public feed such as a tracker's has
/// none and passes through unchanged.
pub fn without_secrets(url: &str) -> String {
    let url = &without_userinfo(url);
    let (base, params) = anibeam_foundation::redact::split_params(url);
    let mut out = base.to_string();
    // Whether `out` has opened its query and its fragment yet.
    let (mut query, mut fragment) = (false, false);
    // The `?` or `#` that opened the query or the fragment with a dropped pair, which the
    // next kept pair stands behind instead of its own `&` or `;`.
    let mut lead: Option<&str> = None;
    for (sep, pair) in params {
        let secret = pair
            .split_once('=')
            .is_some_and(|(name, _)| anibeam_foundation::redact::secret_param(name).is_some());
        if secret {
            if sep == "#" && !fragment {
                lead = Some("#");
            } else if sep == "?" && !query && !fragment {
                lead = Some("?");
            }
            continue;
        }
        let sep = match lead.take() {
            Some(lead) if !matches!(sep, "?" | "#") => lead,
            _ => sep,
        };
        match sep {
            "#" => fragment = true,
            "?" if !fragment => query = true,
            _ => {}
        }
        out.push_str(sep);
        out.push_str(pair);
    }
    out
}

/// `url` without a `user:password@` in front of its host.
fn without_userinfo(url: &str) -> String {
    match anibeam_releases::repos::index::split_login(url) {
        Some((scheme, _, host)) => format!("{scheme}{host}"),
        None => url.to_string(),
    }
}

/// qBittorrent writes 8640000 for an ETA it has no estimate for, and a
/// completion time of 0 or below for a torrent not yet done.
const NO_ETA: i64 = 8_640_000;

pub fn torrents(list: Vec<anibeam_contract::Torrent>) -> Vec<wire::Torrent> {
    list.into_iter()
        .map(|t| wire::Torrent {
            hash: t.hash,
            name: t.name,
            state: t.state,
            progress: t.progress,
            size: t.size,
            download_speed: t.dlspeed,
            eta: Some(t.eta).filter(|e| (0..NO_ETA).contains(e)),
            added_on: t.added_on,
            completed_on: Some(t.completion_on).filter(|c| *c > 0),
            series_id: t.series_id,
        })
        .collect()
}

/// The snapshot, built fresh. The library read and the two Downloads reads
/// run side by side; a Downloads half that fails becomes a sentence in
/// `problems` and the snapshot goes out without it.
pub async fn snapshot(
    host: &Arc<Host>,
    hostname: &str,
    profile: u64,
) -> Result<(wire::Snapshot, LibraryPart), CoreError> {
    let reader = host.clone();
    let library = async move {
        tokio::task::spawn_blocking(move || library_part(&reader.services, profile))
            .await
            .map_err(|e| CoreError::internal(format!("library read: {e}")))?
    };
    let (library, (rules, list)) = tokio::join!(
        library,
        downloads::host_read(&host.downloads, DOWNLOADS_LIMIT)
    );
    let library = library?;
    let mut problems = Vec::new();
    let subscriptions = match rules {
        Ok(rules) => subscriptions(rules),
        Err(e) => {
            tracing::debug!("host snapshot without subscriptions: {e}");
            problems.push(problem("anirss", &e));
            Vec::new()
        }
    };
    let torrents = match list {
        Ok(list) => Some(torrents(list)),
        Err(e) => {
            tracing::debug!("host snapshot without torrents: {e}");
            problems.push(problem("qBittorrent", &e));
            None
        }
    };
    let snapshot = wire::Snapshot {
        api: wire::API_VERSION,
        host: hostname.to_string(),
        version: anibeam_net::VERSION.to_string(),
        generated_at: time::now_secs(),
        series: library.series.clone(),
        subscriptions,
        torrents,
        problems,
    };
    Ok((snapshot, library))
}

#[cfg(test)]
mod tests {

    #[test]
    fn a_feed_url_loses_userinfo_and_tracker_keys() {
        assert_eq!(
            without_secrets("https://user:pw@t.example/rss?torrent_pass=X&cat=5"),
            "https://t.example/rss?cat=5"
        );
        assert_eq!(
            without_secrets("https://t.example/rss?authkey=Y"),
            "https://t.example/rss"
        );
        assert_eq!(
            without_secrets("https://tracker.example/?page=rss&q=a@b"),
            "https://tracker.example/?page=rss&q=a@b"
        );
    }

    #[test]
    fn a_feed_url_loses_its_credentials_and_keeps_the_rest() {
        assert_eq!(
            without_secrets(
                "http://jackett:9117/api/v2.0/indexers/x/results/torznab/?apikey=abc123&t=search&q=Frieren"
            ),
            "http://jackett:9117/api/v2.0/indexers/x/results/torznab/?t=search&q=Frieren"
        );
        assert_eq!(
            without_secrets("https://t.example/rss?passkey=s3cret"),
            "https://t.example/rss"
        );
        let tracker = "https://tracker.example/?page=rss&q=Frieren+1080p&c=1_2&f=0";
        assert_eq!(without_secrets(tracker), tracker);
        assert_eq!(
            without_secrets("https://tracker.example/"),
            "https://tracker.example/"
        );
    }

    #[test]
    fn a_key_after_a_hash_or_a_semicolon_is_dropped_too() {
        assert_eq!(
            without_secrets("https://h.example/feed?q=x#token=K"),
            "https://h.example/feed?q=x"
        );
        assert_eq!(
            without_secrets("https://h.example/feed?q=x;passkey=K&c=1"),
            "https://h.example/feed?q=x&c=1"
        );
        assert_eq!(
            without_secrets("https://h.example/feed?apikey=K&q=x#token=T&f=1"),
            "https://h.example/feed?q=x#f=1"
        );
        assert_eq!(
            without_secrets("https://h.example/cb#token=K"),
            "https://h.example/cb"
        );
        let anchored = "https://h.example/feed?q=x#latest";
        assert_eq!(without_secrets(anchored), anchored);
        let anchored = "https://h.example/feed#key";
        assert_eq!(without_secrets(anchored), anchored);
    }

    #[test]
    fn a_key_with_a_semicolon_or_a_question_mark_in_it_goes_whole() {
        // Its value runs to the next & or #, so no tail of it is kept as a pair of its own.
        assert_eq!(
            without_secrets("https://h.example/feed?password=ab;cd&q=1"),
            "https://h.example/feed?q=1"
        );
        assert_eq!(
            without_secrets("https://h.example/feed?apikey=K;q=x"),
            "https://h.example/feed"
        );
        assert_eq!(
            without_secrets("https://h.example/cb#token=K?x=1"),
            "https://h.example/cb"
        );
    }

    #[test]
    fn a_dropped_key_leaves_the_rest_where_it_was() {
        // A query already open keeps its &, and a fragment keeps its #.
        assert_eq!(
            without_secrets("https://h.example/feed?q=x?apikey=K&c=1"),
            "https://h.example/feed?q=x&c=1"
        );
        assert_eq!(
            without_secrets("https://h.example/feed?apikey=K#a=1&token=T&b=2"),
            "https://h.example/feed#a=1&b=2"
        );
        assert_eq!(
            without_secrets("https://h.example/feed?apikey=K&amp;q=x"),
            "https://h.example/feed?q=x"
        );
    }

    use super::*;

    fn at(secs: u64) -> SystemTime {
        UNIX_EPOCH + Duration::from_secs(secs)
    }

    #[test]
    fn a_poster_tag_is_ten_hex_digits_that_move_with_the_file() {
        let tag = poster_tag("/img/ab/abc.jpg", 1000, at(5));
        assert_eq!(tag.len(), 10);
        assert!(
            tag.chars()
                .all(|c| c.is_ascii_hexdigit() && !c.is_uppercase())
        );
        assert_eq!(tag, poster_tag("/img/ab/abc.jpg", 1000, at(5)));
        assert_ne!(tag, poster_tag("/img/ab/abd.jpg", 1000, at(5)));
        assert_ne!(tag, poster_tag("/img/ab/abc.jpg", 1001, at(5)));
        assert_ne!(tag, poster_tag("/img/ab/abc.jpg", 1000, at(6)));
    }

    #[test]
    fn a_missing_poster_file_has_no_tag() {
        assert_eq!(tag_of("/no/such/poster.jpg"), None);
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(tag_of(dir.path().to_str().unwrap()), None);
        let file = dir.path().join("p.png");
        std::fs::write(&file, b"x").unwrap();
        assert_eq!(tag_of(file.to_str().unwrap()).unwrap().len(), 10);
    }

    #[test]
    fn problems_are_plain_sentences() {
        let late = CoreError::service("qBittorrent", "timeout", "slow");
        assert_eq!(problem("qBittorrent", &late), "qBittorrent did not answer");
        let down = CoreError::service("anirss", "unreachable", "refused");
        assert_eq!(problem("anirss", &down), "anirss is not running");
    }

    #[test]
    fn torrents_drop_the_clients_placeholders() {
        let t = |eta, done| anibeam_contract::Torrent {
            hash: "h".into(),
            name: "n".into(),
            state: "uploading".into(),
            progress: 1.0,
            size: 9,
            dlspeed: 3,
            upspeed: 0,
            eta,
            num_seeds: 0,
            num_leechs: 0,
            ratio: 0.0,
            save_path: String::new(),
            content_path: String::new(),
            added_on: 7,
            completion_on: done,
            tags: String::new(),
            category: String::new(),
            series_id: Some(12),
            episode: None,
            batch: false,
            files: Vec::new(),
            seq_dl: false,
            f_l_piece_prio: false,
            tracker: String::new(),
            group: String::new(),
            resolution: String::new(),
            provider: String::new(),
            roll: None,
            problem: None,
        };
        let out = torrents(vec![t(8_640_000, -1), t(90, 1_700_000_000)]);
        assert_eq!((out[0].eta, out[0].completed_on), (None, None));
        assert_eq!(
            (out[1].eta, out[1].completed_on),
            (Some(90), Some(1_700_000_000))
        );
        assert_eq!(out[1].download_speed, 3);
        assert_eq!(out[1].series_id, Some(12));
    }
}
