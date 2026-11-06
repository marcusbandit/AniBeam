//! Every call error in a person's words: what happened, what to do about it, and the one
//! fix a button can offer. The core's own message stays as the detail, one click away,
//! never the only thing on screen.

use anibeam_core::{CoreError, Provider, Refusal, Tracker};

/// What a button next to the error does. The shell turns each into a labelled button.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fix {
    /// Nothing a button can do; the next step is in the words.
    None,
    /// Run the same thing again.
    Retry,
    /// Open a settings tab by its id.
    Settings(&'static str),
}

impl Fix {
    /// The word the shell reads: `""`, `"retry"` or `"settings:<tab>"`.
    pub fn key(self) -> String {
        match self {
            Fix::None => String::new(),
            Fix::Retry => "retry".into(),
            Fix::Settings(tab) => format!("settings:{tab}"),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Problem {
    /// What happened, one plain sentence.
    pub said: String,
    /// What to do about it, one plain sentence, or empty when the fix says it all.
    pub next: String,
    pub fix: Fix,
}

fn problem(said: impl Into<String>, next: impl Into<String>, fix: Fix) -> Problem {
    Problem {
        said: said.into(),
        next: next.into(),
        fix,
    }
}

fn tracker_name(t: Tracker) -> &'static str {
    match t {
        Tracker::Anilist => "AniList",
        Tracker::Mal => "MyAnimeList",
    }
}

fn provider_name(p: Provider) -> &'static str {
    match p {
        Provider::Anilist => "AniList",
        Provider::Mal => "MyAnimeList",
        Provider::Tmdb => "TMDB",
        Provider::Tvmaze => "TVmaze",
    }
}

/// The service by the name a person knows it under: the search helper is a program the
/// app runs for them, so it gets no name of its own on screen.
fn service_name(service: &str) -> String {
    if service.eq_ignore_ascii_case("anirss") {
        "The search helper".into()
    } else {
        service.to_string()
    }
}

pub fn plain(e: &CoreError) -> Problem {
    match e {
        CoreError::NotFound { .. } => problem(
            "AniBeam could not find that any more.",
            "It may have been removed. Try again to load the page afresh.",
            Fix::Retry,
        ),
        CoreError::Invalid { field, .. } => problem(
            format!(
                "The {} is not one AniBeam can use.",
                field.replace('_', " ")
            ),
            "Check it and try again.",
            Fix::None,
        ),
        CoreError::Unavailable { .. } => problem(
            "That folder cannot be reached right now.",
            "Plug in its drive or check the folder in Settings, under Library.",
            Fix::Settings("library"),
        ),
        CoreError::NotConnected { tracker } => problem(
            format!("{} is not connected.", tracker_name(*tracker)),
            "Connect it in Settings, under Accounts.",
            Fix::Settings("accounts"),
        ),
        CoreError::Refused { reason } => refused(*reason),
        CoreError::Provider {
            provider,
            status,
            retry_after,
            ..
        } => {
            let name = provider_name(*provider);
            match status {
                Some(401) | Some(403) => problem(
                    format!("{name} turned down AniBeam's sign-in."),
                    "Connect it again in Settings, under Accounts.",
                    Fix::Settings("accounts"),
                ),
                Some(429) => problem(
                    format!("{name} is busy right now."),
                    "Wait a minute, then try again.",
                    Fix::Retry,
                ),
                _ if retry_after.is_some() => problem(
                    format!("{name} is busy right now."),
                    "Wait a minute, then try again.",
                    Fix::Retry,
                ),
                _ => problem(
                    format!("{name} did not answer."),
                    "Check your internet connection, then try again.",
                    Fix::Retry,
                ),
            }
        }
        CoreError::Io { .. } => problem(
            "AniBeam could not read or write a file.",
            "Check the disk has room and the folder is yours, then try again.",
            Fix::Retry,
        ),
        CoreError::Storage { .. } => problem(
            "AniBeam could not save your library.",
            "Check the disk has room, then try again.",
            Fix::Retry,
        ),
        CoreError::Keyring { .. } => problem(
            "The system password store did not answer.",
            "Unlock your desktop's keyring, then try again.",
            Fix::Retry,
        ),
        CoreError::Unsupported { what } => {
            problem(format!("{what} is not supported yet."), "", Fix::None)
        }
        CoreError::Version { .. } => problem(
            "This backup comes from a newer AniBeam.",
            "Update AniBeam in Settings, under About, then import it again.",
            Fix::Settings("about"),
        ),
        CoreError::Internal { .. } => problem(
            "Something went wrong inside AniBeam.",
            "Try again. If it keeps happening, report it from the activity log.",
            Fix::Retry,
        ),
        CoreError::Service { service, code, .. } => {
            let name = service_name(service);
            match code.as_str() {
                "unreachable" => problem(
                    format!("{name} did not answer."),
                    "Check it is running and its address is right in Settings, under Downloads.",
                    Fix::Settings("downloads"),
                ),
                "unauthorized" => problem(
                    format!("{name} turned down the login."),
                    "Check the username and password in Settings, under Downloads.",
                    Fix::Settings("downloads"),
                ),
                "not_running" => problem(
                    format!("{name} is not running."),
                    "Start it from Settings, under Downloads.",
                    Fix::Settings("downloads"),
                ),
                "missing" => problem(
                    format!("{name} is not installed."),
                    "Settings, under Downloads, says how to set it up.",
                    Fix::Settings("downloads"),
                ),
                _ => problem(
                    format!("{name} could not do that."),
                    "Try again.",
                    Fix::Retry,
                ),
            }
        }
    }
}

fn refused(reason: Refusal) -> Problem {
    match reason {
        Refusal::Hidden => problem(
            "That show is hidden.",
            "Show hidden shows in Settings, under Library, to work with it.",
            Fix::Settings("library"),
        ),
        Refusal::NoMatch => problem(
            "That show is not on your lists yet.",
            "Match it from its page first, under Fixes.",
            Fix::None,
        ),
        Refusal::NotNewer => problem("Your list is already past this episode.", "", Fix::None),
        Refusal::Extra => problem("That file is an extra, not an episode.", "", Fix::None),
        Refusal::Unmatched => problem(
            "That file is not tied to an episode.",
            "Fix it from the show's page, under Fixes.",
            Fix::None,
        ),
        Refusal::OnDisk => problem("That file is only on this computer.", "", Fix::None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anibeam_core::Entity;

    fn every_error() -> Vec<CoreError> {
        let mut all = vec![
            CoreError::NotFound {
                what: Entity::Series,
                id: 3,
            },
            CoreError::invalid("season_pick", "nope"),
            CoreError::Unavailable { path: "/m".into() },
            CoreError::NotConnected {
                tracker: Tracker::Mal,
            },
            CoreError::Io {
                path: None,
                message: "Permission denied (os error 13)".into(),
            },
            CoreError::Storage {
                message: "database is locked".into(),
            },
            CoreError::Keyring {
                message: "org.freedesktop.Secret: no such object".into(),
            },
            CoreError::Unsupported {
                what: "Casting a film".into(),
            },
            CoreError::Version {
                found: 3,
                supported: 1,
            },
            CoreError::internal("json: expected value at line 1"),
        ];
        for status in [None, Some(401), Some(429), Some(503)] {
            all.push(CoreError::Provider {
                provider: Provider::Anilist,
                status,
                message: "error sending request for url (https://graphql.anilist.co/)".into(),
                retry_after: None,
            });
        }
        for code in [
            "unreachable",
            "unauthorized",
            "not_running",
            "missing",
            "conflict",
        ] {
            all.push(CoreError::Service {
                service: "anirss".into(),
                code: code.into(),
                message: "connection refused".into(),
            });
        }
        for r in [
            Refusal::Hidden,
            Refusal::NoMatch,
            Refusal::NotNewer,
            Refusal::Extra,
            Refusal::Unmatched,
            Refusal::OnDisk,
        ] {
            all.push(CoreError::Refused { reason: r });
        }
        all
    }

    #[test]
    fn no_error_shows_its_raw_text_or_a_developer_word() {
        for e in every_error() {
            let p = plain(&e);
            let shown = format!("{} {}", p.said, p.next);
            for raw in [
                "os error",
                "json",
                "database is locked",
                "org.freedesktop",
                "http",
                "connection refused",
                "anirss",
                "_",
            ] {
                assert!(!shown.contains(raw), "{e:?} shows {raw:?}: {shown}");
            }
            assert!(p.said.ends_with('.'), "{e:?}: {}", p.said);
            assert!(
                p.next.is_empty() || p.next.ends_with('.'),
                "{e:?}: {}",
                p.next
            );
            assert!(
                !shown.contains('\u{2014}') && !shown.contains('\u{2013}'),
                "{e:?} has a dash"
            );
        }
    }

    #[test]
    fn every_error_says_what_to_do_or_offers_a_button() {
        for e in every_error() {
            let p = plain(&e);
            let terminal = matches!(
                e,
                CoreError::Unsupported { .. }
                    | CoreError::Refused {
                        reason: Refusal::NotNewer | Refusal::Extra | Refusal::OnDisk
                    }
            );
            assert!(
                terminal || !p.next.is_empty() || p.fix != Fix::None,
                "{e:?} leaves the person stuck"
            );
        }
    }

    #[test]
    fn a_sign_in_problem_opens_accounts() {
        let e = CoreError::Provider {
            provider: Provider::Mal,
            status: Some(401),
            message: "invalid_token".into(),
            retry_after: None,
        };
        assert_eq!(plain(&e).fix, Fix::Settings("accounts"));
        assert_eq!(plain(&e).said, "MyAnimeList turned down AniBeam's sign-in.");
        let e = CoreError::NotConnected {
            tracker: Tracker::Anilist,
        };
        assert_eq!(plain(&e).fix.key(), "settings:accounts");
    }

    #[test]
    fn a_network_failure_offers_a_retry() {
        let e = CoreError::Provider {
            provider: Provider::Tvmaze,
            status: None,
            message: "timed out".into(),
            retry_after: None,
        };
        let p = plain(&e);
        assert_eq!(p.said, "TVmaze did not answer.");
        assert_eq!(p.fix.key(), "retry");
    }

    #[test]
    fn a_rate_limit_says_wait() {
        let e = CoreError::Provider {
            provider: Provider::Anilist,
            status: None,
            message: "slow down".into(),
            retry_after: Some(30.0),
        };
        assert_eq!(plain(&e).next, "Wait a minute, then try again.");
    }

    #[test]
    fn a_downloads_service_problem_opens_downloads_settings() {
        let e = CoreError::Service {
            service: "qBittorrent".into(),
            code: "unauthorized".into(),
            message: "403".into(),
        };
        let p = plain(&e);
        assert_eq!(p.said, "qBittorrent turned down the login.");
        assert_eq!(p.fix.key(), "settings:downloads");
    }

    #[test]
    fn a_newer_backup_points_at_the_update() {
        let e = CoreError::Version {
            found: 9,
            supported: 1,
        };
        assert_eq!(plain(&e).fix.key(), "settings:about");
    }
}
