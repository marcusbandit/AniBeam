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
    /// Its library from the last snapshot, when that snapshot came from this computer.
    series: Vec<Series>,
}

/// One show in the computer's library, enough to find it from the phone.
#[derive(Clone, Debug, Default)]
struct Series {
    id: u64,
    anilist: Option<u64>,
    /// The names it goes by, lower case.
    names: Vec<String>,
    /// The episodes with a file there.
    had: Vec<u32>,
}

/// Asks the computer set in More whether it takes downloads. Blocking: a computer that
/// is off costs the connect timeout, so call it off the UI thread.
pub fn probe(data_dir: &Path) -> Option<Desk> {
    let store = Store::open(data_dir).ok()?;
    let typed = store.settings().address;
    let address = HostAddress::parse(&typed).ok()?;
    let client = HostClient::new(client::agent(), address);
    let hello = client.hello().ok()?;
    if hello.app != api::APP || !hello.features.iter().any(|f| f == api::FEATURE_DOWNLOADS) {
        return None;
    }
    // A snapshot from another computer names that one's series, whose ids mean nothing
    // here: a download sent with one would land in whichever show has that id.
    let series = store
        .cache()
        .filter(|c| c.address == typed)
        .map(|c| c.snapshot.series.into_iter().map(series_of).collect())
        .unwrap_or_default();
    Some(Desk {
        name: hello.host,
        client: Arc::new(client),
        series,
    })
}

fn series_of(s: api::Series) -> Series {
    let names = [Some(s.title), s.titles.romaji, s.titles.english]
        .into_iter()
        .flatten()
        .map(|n| n.to_lowercase())
        .collect();
    let mut had: Vec<u32> = s
        .episodes
        .iter()
        .filter(|e| e.file.is_some() && e.number >= 1.0 && e.number.fract() == 0.0)
        .map(|e| e.number as u32)
        .collect();
    had.sort_unstable();
    had.dedup();
    Series {
        id: s.id,
        anilist: s.anilist_id,
        names,
        had,
    }
}

impl Desk {
    /// The computer's own series for a show, so its episodes land in the folder it
    /// already has: by AniList id, else by any of its names.
    pub fn series_for(&self, anilist: Option<u64>, names: &[String]) -> Option<u64> {
        self.find(anilist, names).map(|s| s.id)
    }

    /// The episodes the computer has a file for, as of the last snapshot.
    pub fn had(&self, anilist: Option<u64>, names: &[String]) -> Vec<u32> {
        self.find(anilist, names)
            .map(|s| s.had.clone())
            .unwrap_or_default()
    }

    fn find(&self, anilist: Option<u64>, names: &[String]) -> Option<&Series> {
        if let Some(id) = anilist
            && let Some(s) = self.series.iter().find(|s| s.anilist == Some(id))
        {
            return Some(s);
        }
        let names: Vec<String> = names.iter().map(|n| n.to_lowercase()).collect();
        self.series
            .iter()
            .find(|s| s.names.iter().any(|n| names.contains(n)))
    }

    /// One download call, run on the computer's core. Blocking.
    pub fn call(&self, call: &Call) -> Result<Reply, CoreError> {
        crate::stream::call_on(&self.client, &self.name, call)
    }
}

/// What the phone tells a person once the computer answered: done, or still adding.
pub fn sent_line(reply: &Reply, what: &str, name: &str) -> String {
    match reply {
        Reply::Started { .. } => tr!("desk-sent-adding", what = what, name = name),
        _ => tr!("desk-sent", what = what, name = name),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn desk() -> Desk {
        let series = |id, anilist, names: &[&str], had: &[u32]| Series {
            id,
            anilist,
            names: names.iter().map(|n| n.to_string()).collect(),
            had: had.to_vec(),
        };
        Desk {
            name: "computer".into(),
            client: Arc::new(HostClient::new(
                client::agent(),
                HostAddress::parse("computer.example:8790").unwrap(),
            )),
            series: vec![
                series(10, Some(1), &["frieren"], &[1, 2, 3]),
                series(20, None, &["dungeon meshi", "delicious in dungeon"], &[]),
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
        assert_eq!(d.had(Some(1), &[]), vec![1, 2, 3]);
        assert!(d.had(None, &["Something Else".into()]).is_empty());
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
