//! Whether a newer desktop build is out. One request to GitHub's releases for the repo, read
//! once as the shell starts and again when the person switches track; the shell shows the
//! answer and the release page is where the download is.
//!
//! A desktop build is tagged `v2.4.0` when stable and `v2.4.0-nightly.20261030.12` when
//! nightly; the phone's `android-v*` tags never count here, and neither does a draft or a
//! pull request's test build. The Stable track is offered stable versions only, the Nightly
//! track both, so a nightly user moves on to the stable release its nightlies led up to.

use serde::Deserialize;

use crate::contract::CoreError;
use crate::net::{Http, HttpRequest, Method};

const RELEASES: &str = "https://api.github.com/repos/marcusbandit/AniBeam/releases?per_page=100";

/// Which builds a person wants to hear about.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Track {
    Stable,
    Nightly,
}

impl Track {
    /// The track a build belongs to when the person has not picked one: a nightly follows
    /// nightlies, everything else follows stable releases.
    pub fn of(build: &str) -> Track {
        match Version::parse(build) {
            Some(v) if v.nightly.is_some() => Track::Nightly,
            _ => Track::Stable,
        }
    }

    /// `stable` or `nightly` as the shell stores it; anything else is no pick.
    pub fn parse(name: &str) -> Option<Track> {
        match name {
            "stable" => Some(Track::Stable),
            "nightly" => Some(Track::Nightly),
            _ => None,
        }
    }
}

/// A desktop release newer than the running build: its version and its page on GitHub.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Update {
    pub version: String,
    pub url: String,
}

/// A version as builds name themselves: `2.4.0`, `2.4.0-nightly.20261030.12` or
/// `2.4.0-pr.280.3`. A nightly or test build comes before the stable version it leads up
/// to, nightlies order by day and then by run, and a test build sits before every nightly.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Version {
    numbers: [u64; 3],
    stable: bool,
    nightly: Option<[u64; 2]>,
}

impl Version {
    fn parse(v: &str) -> Option<Version> {
        let (base, suffix) = match v.split_once('-') {
            Some((base, suffix)) => (base, Some(suffix)),
            None => (v, None),
        };
        let numbers = digits::<3>(base)?;
        match suffix {
            None => Some(Version {
                numbers,
                stable: true,
                nightly: None,
            }),
            Some(s) => {
                if let Some(n) = s.strip_prefix("nightly.") {
                    Some(Version {
                        numbers,
                        stable: false,
                        nightly: Some(digits::<2>(n)?),
                    })
                } else if let Some(n) = s.strip_prefix("pr.") {
                    digits::<2>(n)?;
                    Some(Version {
                        numbers,
                        stable: false,
                        nightly: None,
                    })
                } else {
                    None
                }
            }
        }
    }
}

fn digits<const N: usize>(s: &str) -> Option<[u64; N]> {
    let mut parts = s.split('.');
    let mut out = [0; N];
    for slot in &mut out {
        let n = parts.next()?;
        if n.is_empty() || !n.bytes().all(|b| b.is_ascii_digit()) {
            return None;
        }
        *slot = n.parse().ok()?;
    }
    parts.next().is_none().then_some(out)
}

#[derive(Deserialize)]
struct Release {
    tag_name: String,
    html_url: String,
    #[serde(default)]
    draft: bool,
}

/// The newest release on `track` in `releases`, GitHub's list as it sends it, when it is
/// newer than `current`. A `current` that names no version is offered the newest.
pub fn newest(releases: &[u8], current: &str, track: Track) -> Result<Option<Update>, CoreError> {
    let releases: Vec<Release> = serde_json::from_slice(releases)
        .map_err(|e| CoreError::internal(format!("releases: {e}")))?;
    let running = Version::parse(current);
    let mut best: Option<(Version, Update)> = None;
    for r in releases {
        if r.draft {
            continue;
        }
        let Some(name) = r.tag_name.strip_prefix('v') else {
            continue;
        };
        let Some(v) = Version::parse(name) else {
            continue;
        };
        let on_track = v.stable || (track == Track::Nightly && v.nightly.is_some());
        if !on_track || running.is_some_and(|run| v <= run) {
            continue;
        }
        if best.as_ref().is_none_or(|(b, _)| v > *b) {
            best = Some((
                v,
                Update {
                    version: name.to_string(),
                    url: r.html_url,
                },
            ));
        }
    }
    Ok(best.map(|(_, u)| u))
}

/// Asks GitHub. A failure is the caller's to drop: no answer means no notice.
pub async fn check(
    http: &dyn Http,
    current: &str,
    track: Track,
) -> Result<Option<Update>, CoreError> {
    let res = http
        .send(HttpRequest {
            method: Method::Get,
            url: RELEASES.to_string(),
            headers: vec![("Accept".into(), "application/vnd.github+json".into())],
            body: None,
            timeout: None,
        })
        .await
        .map_err(|e| CoreError::internal(format!("releases: {}", e.message)))?;
    if !res.is_success() {
        return Err(CoreError::internal(format!(
            "releases: HTTP {}",
            res.status
        )));
    }
    newest(&res.body, current, track)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn list() -> Vec<u8> {
        serde_json::json!([
            { "tag_name": "android-v2.6.0", "html_url": "a", "draft": false },
            { "tag_name": "v2.6.0", "html_url": "draft", "draft": true },
            { "tag_name": "v2.5.0-nightly.20261102.40", "html_url": "n40" },
            { "tag_name": "v2.5.0-nightly.20261101.39", "html_url": "n39" },
            { "tag_name": "v2.5.0-pr.280.3", "html_url": "pr" },
            { "tag_name": "pr-builds", "html_url": "rolling" },
            { "tag_name": "v2.3.0", "html_url": "u230", "draft": false },
            { "tag_name": "v2.4.0", "html_url": "u240", "draft": false },
            { "tag_name": "v2.2.0", "html_url": "u220", "draft": false }
        ])
        .to_string()
        .into_bytes()
    }

    fn offered(current: &str, track: Track) -> Option<String> {
        newest(&list(), current, track).unwrap().map(|u| u.url)
    }

    #[test]
    fn stable_is_offered_the_newest_stable_release_only() {
        assert_eq!(offered("2.2.0", Track::Stable).as_deref(), Some("u240"));
        assert_eq!(offered("2.4.0", Track::Stable), None);
        assert_eq!(offered("2.6.0", Track::Stable), None);
    }

    #[test]
    fn nightly_is_offered_the_newest_nightly_by_day_then_run() {
        assert_eq!(offered("2.4.0", Track::Nightly).as_deref(), Some("n40"));
        assert_eq!(
            offered("2.5.0-nightly.20261101.39", Track::Nightly).as_deref(),
            Some("n40")
        );
        assert_eq!(offered("2.5.0-nightly.20261102.40", Track::Nightly), None);
    }

    #[test]
    fn a_nightly_moves_on_to_the_stable_release_it_led_up_to() {
        let list = serde_json::json!([
            { "tag_name": "v2.5.0-nightly.20261102.40", "html_url": "n40" },
            { "tag_name": "v2.5.0", "html_url": "u250" }
        ])
        .to_string();
        let u = newest(list.as_bytes(), "2.5.0-nightly.20261102.40", Track::Nightly).unwrap();
        assert_eq!(u.unwrap().url, "u250");
        let u = newest(list.as_bytes(), "2.5.0-nightly.20261102.40", Track::Stable).unwrap();
        assert_eq!(u.unwrap().url, "u250");
    }

    #[test]
    fn a_test_build_hears_of_the_release_it_was_tested_for() {
        assert_eq!(
            offered("2.4.0-pr.280.3", Track::Stable).as_deref(),
            Some("u240")
        );
        assert_eq!(offered("2.5.0-pr.280.3", Track::Stable), None);
        assert_eq!(
            offered("2.5.0-pr.280.3", Track::Nightly).as_deref(),
            Some("n40")
        );
    }

    #[test]
    fn the_build_picks_its_own_track() {
        assert_eq!(Track::of("2.5.0-nightly.20261102.40"), Track::Nightly);
        assert_eq!(Track::of("2.5.0"), Track::Stable);
        assert_eq!(Track::of("2.5.0-pr.280.3"), Track::Stable);
        assert_eq!(Track::of("2.3.1.r5.gabc1234"), Track::Stable);
        assert_eq!(Track::parse("nightly"), Some(Track::Nightly));
        assert_eq!(Track::parse(""), None);
    }

    #[test]
    fn a_body_that_is_not_a_release_list_is_an_error() {
        assert!(newest(b"{\"message\":\"rate limited\"}", "2.2.0", Track::Stable).is_err());
    }
}
