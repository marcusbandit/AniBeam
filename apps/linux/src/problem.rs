//! Every call error in a person's words: what happened, what to do about it, and the one
//! fix a button can offer. The core's own message stays as the detail, one click away,
//! never the only thing on screen. The words come from the catalog (#792), in the language
//! the app speaks when the error is shown.

use anibeam_core::i18n::{Arg, tr};
use anibeam_core::{CoreError, Provider, Refusal, provider_name, tracker_name};

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
    /// What to do about it, one plain sentence, or empty when there is nothing to do.
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

/// A message from the catalog that takes no arguments.
fn t(id: &str) -> String {
    tr(id, &[])
}

/// A message from the catalog that names one service, site or provider as `$name`.
fn named(id: &str, name: &str) -> String {
    tr(id, &[("name", Arg::from(name))])
}

fn report() -> String {
    t("problem-report")
}

/// A message the core wrote for people, as a sentence of its own: a capital first letter
/// and a full stop.
fn as_sentence(message: &str) -> String {
    let m = message.trim();
    let mut chars = m.chars();
    let Some(first) = chars.next() else {
        return String::new();
    };
    let mut out: String = first.to_uppercase().chain(chars).collect();
    if !out.ends_with(['.', '!', '?', '。', '！', '？']) {
        out.push('.');
    }
    out
}

/// The services Anibeam runs or talks to on this machine for downloads, by the name a
/// person knows them under. Anything else named in a `Service` error is a search source
/// or a site, which keeps its own name.
enum Service {
    SearchHelper,
    TorrentClient,
    Trash,
    SourceList,
    Remote(String),
}

fn service_of(name: &str) -> Service {
    match name.to_ascii_lowercase().as_str() {
        "anirss" => Service::SearchHelper,
        "qbittorrent" | "torrent" | "torrents" => Service::TorrentClient,
        "trash" => Service::Trash,
        "endpoint repo" => Service::SourceList,
        _ => Service::Remote(name.to_string()),
    }
}

fn service_problem(service: &str, code: &str) -> Problem {
    // One message per service and code, so a language can say "the search helper" in
    // whatever form its sentence needs; `who` is the start of the IDs, `problem-helper` or
    // `problem-torrent`.
    let local = |who: &str| {
        let said = |what: &str| t(&format!("{who}-{what}"));
        match code {
            "unreachable" => problem(
                said("unreachable"),
                t("problem-downloads-unreachable-next"),
                Fix::Settings("downloads"),
            ),
            "unauthorized" => problem(
                said("unauthorized"),
                t("problem-downloads-unauthorized-next"),
                Fix::Settings("downloads"),
            ),
            "not_running" => problem(
                said("not-running"),
                t("problem-downloads-not-running-next"),
                Fix::Settings("downloads"),
            ),
            "missing" => problem(
                said("missing"),
                t("problem-downloads-missing-next"),
                Fix::Settings("downloads"),
            ),
            _ => problem(said("failed"), report(), Fix::Retry),
        }
    };
    match service_of(service) {
        // anirss's own word for a qBittorrent that wants the password it was not given
        Service::SearchHelper if code == "qbt_auth_required" => problem(
            t("problem-helper-no-login"),
            t("problem-helper-no-login-next"),
            Fix::Settings("downloads"),
        ),
        Service::SearchHelper => local("problem-helper"),
        Service::TorrentClient => local("problem-torrent"),
        Service::Trash => problem(t("problem-trash"), t("problem-trash-next"), Fix::None),
        Service::SourceList => problem(
            t("problem-source-list"),
            t("problem-source-list-next"),
            Fix::Settings("downloads"),
        ),
        Service::Remote(name) => match code {
            "unreachable" => problem(
                named("problem-remote-unreachable", &name),
                t("problem-remote-unreachable-next"),
                Fix::Retry,
            ),
            "unauthorized" => problem(
                named("problem-remote-unauthorized", &name),
                t("problem-remote-unauthorized-next"),
                Fix::Settings("downloads"),
            ),
            _ => problem(
                named("problem-remote-failed", &name),
                t("problem-remote-failed-next"),
                Fix::Retry,
            ),
        },
    }
}

fn provider_problem(provider: Provider, status: Option<u32>, retry_after: Option<f64>) -> Problem {
    let name = provider_name(provider);
    let signs_in = matches!(provider, Provider::Anilist | Provider::Mal);
    match status {
        Some(401) | Some(403) if signs_in => problem(
            named("problem-provider-sign-in", name),
            t("problem-provider-sign-in-next"),
            Fix::Settings("accounts"),
        ),
        Some(401) | Some(403) => problem(
            named("problem-provider-turned-away", name),
            t("problem-try-later"),
            Fix::Retry,
        ),
        Some(429) => problem(
            named("problem-provider-busy", name),
            t("problem-provider-busy-next"),
            Fix::Retry,
        ),
        _ if retry_after.is_some() => problem(
            named("problem-provider-busy", name),
            t("problem-provider-busy-next"),
            Fix::Retry,
        ),
        None => problem(
            named("problem-provider-no-answer", name),
            t("problem-provider-no-answer-next"),
            Fix::Retry,
        ),
        Some(s) if s >= 500 => problem(
            named("problem-provider-trouble", name),
            t("problem-provider-trouble-next"),
            Fix::Retry,
        ),
        Some(_) => problem(
            named("problem-provider-rejected", name),
            report(),
            Fix::Retry,
        ),
    }
}

pub fn plain(e: &CoreError) -> Problem {
    match e {
        CoreError::NotFound { .. } => problem(
            t("problem-not-found"),
            t("problem-not-found-next"),
            Fix::Retry,
        ),
        // The core writes these reasons for people ("already a source", "The address must
        // be https"), so the reason is the sentence; the field name is the log's.
        CoreError::Invalid { message, .. } => {
            problem(as_sentence(message), t("problem-invalid-next"), Fix::None)
        }
        CoreError::Unavailable { .. } => problem(
            t("problem-unavailable"),
            t("problem-unavailable-next"),
            Fix::Settings("library"),
        ),
        CoreError::NotConnected { tracker } => problem(
            tr(
                "problem-not-connected",
                &[("tracker", Arg::from(tracker_name(*tracker)))],
            ),
            t("problem-not-connected-next"),
            Fix::Settings("accounts"),
        ),
        CoreError::Refused { reason } => refused(*reason),
        CoreError::Provider {
            provider,
            status,
            retry_after,
            ..
        } => provider_problem(*provider, *status, *retry_after),
        CoreError::Io { .. } => problem(t("problem-io"), t("problem-io-next"), Fix::Retry),
        CoreError::Storage { .. } => {
            problem(t("problem-storage"), t("problem-storage-next"), Fix::Retry)
        }
        CoreError::Keyring { .. } => {
            problem(t("problem-keyring"), t("problem-keyring-next"), Fix::Retry)
        }
        // `what` names the feature the way the code does ("anirss on this platform"), so
        // it stays in the details
        CoreError::Unsupported { .. } => problem(t("problem-unsupported"), "", Fix::None),
        CoreError::Version { .. } => problem(
            t("problem-version"),
            t("problem-version-next"),
            Fix::Settings("about"),
        ),
        CoreError::Internal { .. } => problem(t("problem-internal"), report(), Fix::Retry),
        CoreError::Service { service, code, .. } => service_problem(service, code),
    }
}

fn refused(reason: Refusal) -> Problem {
    match reason {
        Refusal::Hidden => problem(
            t("problem-refused-hidden"),
            t("problem-refused-hidden-next"),
            Fix::None,
        ),
        Refusal::NoMatch => problem(
            t("problem-refused-no-match"),
            t("problem-refused-no-match-next"),
            Fix::None,
        ),
        Refusal::NotNewer => problem(t("problem-refused-not-newer"), "", Fix::None),
        Refusal::Extra => problem(t("problem-refused-extra"), "", Fix::None),
        Refusal::Unmatched => problem(
            t("problem-refused-unmatched"),
            t("problem-refused-unmatched-next"),
            Fix::None,
        ),
        Refusal::OnDisk => problem(t("problem-refused-on-disk"), "", Fix::None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anibeam_core::{Entity, Tracker};

    fn every_error() -> Vec<CoreError> {
        let mut all = vec![
            CoreError::NotFound {
                what: Entity::Series,
                id: 3,
            },
            CoreError::invalid("path", "already a source"),
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
                what: "anirss on this platform".into(),
            },
            CoreError::Version {
                found: 3,
                supported: 1,
            },
            CoreError::internal("json: expected value at line 1"),
        ];
        for provider in [Provider::Anilist, Provider::Tvmaze] {
            for status in [None, Some(400), Some(401), Some(404), Some(429), Some(503)] {
                all.push(CoreError::Provider {
                    provider,
                    status,
                    message: "error sending request for url (https://graphql.anilist.co/)".into(),
                    retry_after: None,
                });
            }
        }
        for service in [
            "anirss",
            "qBittorrent",
            "torrents",
            "trash",
            "Endpoint repo",
            "Some Feed",
        ] {
            for code in [
                "unreachable",
                "unauthorized",
                "not_running",
                "missing",
                "failed",
                "qbt_auth_required",
            ] {
                all.push(CoreError::Service {
                    service: service.into(),
                    code: code.into(),
                    message: "connection refused".into(),
                });
            }
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
    fn no_error_shows_raw_text_or_a_developer_word() {
        let raw = [
            "os error",
            "json",
            "database is locked",
            "org.freedesktop",
            "http",
            "connection refused",
            "anirss",
            "_",
            "trash could",
            "torrents",
        ];
        for e in every_error() {
            let p = plain(&e);
            let shown = format!("{} {}", p.said, p.next);
            for r in raw {
                assert!(!shown.contains(r), "{e:?} shows {r:?}: {shown}");
            }
            assert!(
                p.said.chars().next().is_some_and(char::is_uppercase),
                "{e:?}: {}",
                p.said
            );
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
    fn an_invalid_input_keeps_the_cores_reason() {
        let p = plain(&CoreError::invalid("path", "already a source"));
        assert_eq!(p.said, "Already a source.");
        let p = plain(&CoreError::invalid("url", "The address must be https"));
        assert_eq!(p.said, "The address must be https.");
    }

    #[test]
    fn only_a_list_site_sign_in_problem_opens_accounts() {
        let mal = CoreError::Provider {
            provider: Provider::Mal,
            status: Some(401),
            message: "invalid_token".into(),
            retry_after: None,
        };
        assert_eq!(plain(&mal).fix, Fix::Settings("accounts"));
        assert_eq!(
            plain(&mal).said,
            "MyAnimeList turned down Anibeam's sign-in."
        );
        let tvmaze = CoreError::Provider {
            provider: Provider::Tvmaze,
            status: Some(403),
            message: "forbidden".into(),
            retry_after: None,
        };
        assert_eq!(plain(&tvmaze).fix, Fix::Retry);
        let e = CoreError::NotConnected {
            tracker: Tracker::Anilist,
        };
        assert_eq!(plain(&e).fix.key(), "settings:accounts");
    }

    #[test]
    fn only_no_answer_blames_the_connection() {
        let at = |status| {
            plain(&CoreError::Provider {
                provider: Provider::Tvmaze,
                status,
                message: "x".into(),
                retry_after: None,
            })
        };
        assert_eq!(at(None).said, "TVmaze did not answer.");
        assert!(at(None).next.contains("internet"));
        for status in [Some(400), Some(404), Some(500), Some(503)] {
            assert!(!at(status).next.contains("internet"), "{status:?}");
        }
        assert_eq!(at(Some(503)).said, "TVmaze is having trouble right now.");
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
    fn a_local_downloads_service_opens_downloads_settings() {
        let e = CoreError::Service {
            service: "qBittorrent".into(),
            code: "unauthorized".into(),
            message: "403".into(),
        };
        let p = plain(&e);
        assert_eq!(p.said, "The torrent client turned down the login.");
        assert_eq!(p.fix.key(), "settings:downloads");
    }

    #[test]
    fn a_search_helper_that_cannot_log_in_points_at_the_torrent_password() {
        let e = CoreError::Service {
            service: "anirss".into(),
            code: "qbt_auth_required".into(),
            message: "qBittorrent wants a login".into(),
        };
        let p = plain(&e);
        assert_eq!(
            p.said,
            "The search helper cannot log in to the torrent client."
        );
        assert!(p.next.contains("password"), "{}", p.next);
        assert_eq!(p.fix.key(), "settings:downloads");
    }

    #[test]
    fn a_remote_source_that_times_out_is_not_something_to_start() {
        let e = CoreError::Service {
            service: "Some Feed".into(),
            code: "unreachable".into(),
            message: "timed out".into(),
        };
        let p = plain(&e);
        assert_eq!(p.said, "Some Feed did not answer.");
        assert!(!p.next.contains("running"), "{}", p.next);
        assert_eq!(p.fix, Fix::Retry);
    }

    #[test]
    fn a_trash_failure_offers_no_retry_that_cannot_work() {
        let e = CoreError::Service {
            service: "trash".into(),
            code: "failed".into(),
            message: "no trash can".into(),
        };
        assert_eq!(plain(&e).fix, Fix::None);
    }

    #[test]
    fn a_hidden_refusal_points_at_unhide() {
        let p = plain(&CoreError::Refused {
            reason: Refusal::Hidden,
        });
        assert_eq!(p.fix, Fix::None);
        assert!(p.next.contains("Unhide"));
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
