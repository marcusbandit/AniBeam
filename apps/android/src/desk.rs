//! Where a download goes: this phone, or the AniBeam computer the phone is connected to
//! (#733). The computer is asked once a show opens, in the background, with its cheap
//! hello; one that does not answer, or runs a version without downloads, is not offered.

use std::path::Path;
use std::sync::Arc;

use anibeam_companion::address::HostAddress;
use anibeam_companion::api;
use anibeam_companion::client::{self, HostClient};
use anibeam_companion::store::Store;
use anibeam_core::{Call, CoreError, Reply};

/// The connected computer, when it takes downloads.
#[derive(Clone)]
pub struct Desk {
    /// Its own name, from its hello ("banditbox").
    pub name: String,
    client: Arc<HostClient>,
    /// Its library from the last snapshot: AniList id, the names it goes by, its id there.
    series: Vec<(Option<u64>, Vec<String>, u64)>,
}

/// Asks the computer set in More whether it takes downloads. Blocking: a computer that
/// is off costs the connect timeout, so call it off the UI thread.
pub fn probe(data_dir: &Path) -> Option<Desk> {
    let store = Store::open(data_dir).ok()?;
    let address = HostAddress::parse(&store.settings().address).ok()?;
    let client = HostClient::new(client::agent(), address);
    let hello = client.hello().ok()?;
    if hello.app != api::APP || !hello.features.iter().any(|f| f == api::FEATURE_DOWNLOADS) {
        return None;
    }
    let series = store
        .cache()
        .map(|c| {
            c.snapshot
                .series
                .into_iter()
                .map(|s| {
                    let names = [Some(s.title), s.titles.romaji, s.titles.english]
                        .into_iter()
                        .flatten()
                        .map(|n| n.to_lowercase())
                        .collect();
                    (s.anilist_id, names, s.id)
                })
                .collect()
        })
        .unwrap_or_default();
    Some(Desk {
        name: hello.host,
        client: Arc::new(client),
        series,
    })
}

impl Desk {
    /// The computer's own series for a show, so its episodes land in the folder it
    /// already has: by AniList id, else by any of its names.
    pub fn series_for(&self, anilist: Option<u64>, names: &[String]) -> Option<u64> {
        if let Some(id) = anilist
            && let Some(s) = self.series.iter().find(|s| s.0 == Some(id))
        {
            return Some(s.2);
        }
        let names: Vec<String> = names.iter().map(|n| n.to_lowercase()).collect();
        self.series
            .iter()
            .find(|s| s.1.iter().any(|n| names.contains(n)))
            .map(|s| s.2)
    }

    /// One download call, run on the computer's core. Blocking.
    pub fn call(&self, call: &Call) -> Result<Reply, CoreError> {
        crate::stream::call_on(&self.client, &self.name, call)
    }
}

/// What the phone tells a person once the computer answered: done, or still adding.
pub fn sent_line(reply: &Reply, what: &str, name: &str) -> String {
    match reply {
        Reply::Started { .. } => format!("Sent {what} to {name}. It is still adding them."),
        _ => format!("Getting {what} on {name}."),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn desk() -> Desk {
        Desk {
            name: "computer".into(),
            client: Arc::new(HostClient::new(
                client::agent(),
                HostAddress::parse("computer.example:8790").unwrap(),
            )),
            series: vec![
                (Some(1), vec!["frieren".into()], 10),
                (
                    None,
                    vec!["dungeon meshi".into(), "delicious in dungeon".into()],
                    20,
                ),
            ],
        }
    }

    #[test]
    fn a_show_finds_its_series_on_the_computer() {
        let d = desk();
        assert_eq!(d.series_for(Some(1), &[]), Some(10));
        assert_eq!(
            d.series_for(Some(99), &["Delicious in Dungeon".into()]),
            Some(20)
        );
        assert_eq!(d.series_for(None, &["Something Else".into()]), None);
    }

    #[test]
    fn the_line_says_whether_the_computer_finished() {
        assert_eq!(
            sent_line(&Reply::Ok, "3 episodes", "computer"),
            "Getting 3 episodes on computer."
        );
        assert!(sent_line(&Reply::Started { job: 1 }, "1 episode", "computer").contains("still"));
    }
}
