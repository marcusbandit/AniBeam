//! Whether a newer desktop build is out. One request to GitHub's releases for the repo, read
//! once as the shell starts; the shell shows the answer and the release page is where the
//! download is. Desktop releases are the `v*` tags, the phone's are `android-v*`, so the
//! phone's never count here. A draft or a pre-release never counts either.

use serde::Deserialize;

use crate::changelog::newer;
use crate::contract::CoreError;
use crate::net::{Http, HttpRequest, Method};

const RELEASES: &str = "https://api.github.com/repos/marcusbandit/AniBeam/releases?per_page=30";

/// A desktop release newer than the running build: its version and its page on GitHub.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Update {
    pub version: String,
    pub url: String,
}

#[derive(Deserialize)]
struct Release {
    tag_name: String,
    html_url: String,
    #[serde(default)]
    draft: bool,
    #[serde(default)]
    prerelease: bool,
}

/// The newest desktop release in `releases`, GitHub's list as it sends it, when it is newer
/// than `current`.
pub fn newest(releases: &[u8], current: &str) -> Result<Option<Update>, CoreError> {
    let releases: Vec<Release> = serde_json::from_slice(releases)
        .map_err(|e| CoreError::internal(format!("releases: {e}")))?;
    let mut best: Option<Update> = None;
    for r in releases {
        if r.draft || r.prerelease {
            continue;
        }
        let Some(version) = r.tag_name.strip_prefix('v') else {
            continue;
        };
        if newer(version, current) && best.as_ref().is_none_or(|b| newer(version, &b.version)) {
            best = Some(Update {
                version: version.to_string(),
                url: r.html_url,
            });
        }
    }
    Ok(best)
}

/// Asks GitHub. A failure is the caller's to drop: no answer means no notice.
pub async fn check(http: &dyn Http, current: &str) -> Result<Option<Update>, CoreError> {
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
    newest(&res.body, current)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn list() -> Vec<u8> {
        serde_json::json!([
            { "tag_name": "android-v2.4.0", "html_url": "a", "draft": false, "prerelease": false },
            { "tag_name": "v2.5.0", "html_url": "draft", "draft": true, "prerelease": false },
            { "tag_name": "v2.4.1", "html_url": "pre", "draft": false, "prerelease": true },
            { "tag_name": "v2.3.0", "html_url": "u230", "draft": false, "prerelease": false },
            { "tag_name": "v2.4.0", "html_url": "u240", "draft": false, "prerelease": false },
            { "tag_name": "v2.2.0", "html_url": "u220", "draft": false, "prerelease": false }
        ])
        .to_string()
        .into_bytes()
    }

    #[test]
    fn the_newest_desktop_release_wins_and_the_phone_drafts_and_previews_never_count() {
        let u = newest(&list(), "2.2.0").unwrap().unwrap();
        assert_eq!(u.version, "2.4.0");
        assert_eq!(u.url, "u240");
    }

    #[test]
    fn nothing_is_offered_when_the_build_is_current_or_ahead() {
        assert_eq!(newest(&list(), "2.4.0").unwrap(), None);
        assert_eq!(newest(&list(), "2.6.0").unwrap(), None);
    }

    #[test]
    fn a_body_that_is_not_a_release_list_is_an_error() {
        assert!(newest(b"{\"message\":\"rate limited\"}", "2.2.0").is_err());
    }
}
