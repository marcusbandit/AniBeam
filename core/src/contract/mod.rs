pub mod calls;
pub mod downloads;
pub mod enums;
pub mod error;
pub mod events;
pub mod records;
pub mod words;

pub use calls::*;
pub use downloads::*;
pub use enums::*;
pub use error::*;
pub use events::*;
pub use records::*;
pub use words::*;

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_send_sync<T: Send + Sync + Clone + std::fmt::Debug>() {}

    #[test]
    fn contract_types_are_send_sync_clone_debug() {
        assert_send_sync::<Call>();
        assert_send_sync::<Reply>();
        assert_send_sync::<Event>();
        assert_send_sync::<EventBody>();
        assert_send_sync::<CoreError>();
        assert_send_sync::<SeriesCard>();
        assert_send_sync::<PlaybackSession>();
        assert_send_sync::<RoomInfo>();
        assert_send_sync::<EpisodeRef>();
    }

    /// A blob saved before rooms existed reads with no name and the default relay.
    #[test]
    fn room_preferences_default_for_old_blobs() {
        let old = r#"{"title_language":"English","library_tab":"All","library_sort":"Alpha","library_direction":"Asc","feed_sort":"Recent"}"#;
        let p: Preferences = serde_json::from_str(old).unwrap();
        assert_eq!(p.room_name, "");
        assert_eq!(p.room_relay, ROOM_RELAY);
        assert_eq!(Preferences::default().room_relay, ROOM_RELAY);
    }

    #[test]
    fn calls_round_trip_through_json_externally_tagged() {
        let call = Call::MarkEpisode {
            series: 3,
            episode: 12.0,
        };
        let json = serde_json::to_value(&call).unwrap();
        assert_eq!(
            json,
            serde_json::json!({ "MarkEpisode": { "series": 3, "episode": 12.0 } })
        );
        let back: Call = serde_json::from_value(json).unwrap();
        assert_eq!(back, call);
        let unit: Call = serde_json::from_value(serde_json::json!("ListSources")).unwrap();
        assert_eq!(unit, Call::ListSources);
    }

    #[test]
    fn replies_have_named_fields() {
        let reply = Reply::Started { job: 9 };
        assert_eq!(
            serde_json::to_value(&reply).unwrap(),
            serde_json::json!({ "Started": { "job": 9 } })
        );
    }

    /// Every kind. The match below stops compiling when a kind is added, which
    /// is the reminder to add it to this list too; the list itself is not
    /// checked by the compiler.
    fn every_job_kind() -> Vec<JobKind> {
        use JobKind::*;
        let all = vec![
            Scan,
            AutoMatch,
            Search,
            ResolveLink,
            ApplyMatch,
            Refresh,
            RefreshAll,
            RefreshAiring,
            RefreshImdb,
            ClearImages,
            FillImages,
            ConnectTracker,
            Mark,
            SetProgress,
            Score,
            RefreshProgress,
            RefreshWatching,
            RefreshAiringNow,
            RefreshRecommended,
            ReadShowPage,
            OnHold,
            SetStatus,
            MoveLists,
            Notes,
            Crawl,
            SkipWindows,
            Export,
            Import,
            Subscriptions,
            ConnectDownloads,
            SearchReleases,
            RefineReleases,
            Subscribe,
            Download,
            RemoveSubscription,
            ControlTorrents,
            TestTorrentClient,
            RemoveShow,
            RepairSubscription,
            CheckEndpoints,
            StartEndpoint,
            ScheduleAgent,
        ];
        for kind in &all {
            match kind {
                Scan | AutoMatch | Search | ResolveLink | ApplyMatch | Refresh | RefreshAll
                | RefreshAiring | RefreshImdb | ClearImages | FillImages | ConnectTracker
                | Mark | SetProgress | Score | RefreshProgress | RefreshWatching
                | RefreshAiringNow | RefreshRecommended | ReadShowPage | OnHold | SetStatus
                | Notes | Crawl | SkipWindows | Export | Import | Subscriptions
                | ConnectDownloads | SearchReleases | RefineReleases | Subscribe | Download
                | RemoveSubscription | ControlTorrents | TestTorrentClient | RemoveShow
                | RepairSubscription | CheckEndpoints | StartEndpoint | ScheduleAgent
                | MoveLists => {}
            }
        }
        all
    }

    #[test]
    fn every_job_kind_has_plain_words_for_a_status_line() {
        for kind in every_job_kind() {
            let words = kind.doing();
            assert!(!words.is_empty(), "{kind:?} has no words");
            assert!(
                crate::i18n::exists(kind.words_id())
                    && crate::i18n::exists(&format!("{}.stopped", kind.words_id())),
                "{kind:?} is missing from the catalog"
            );
            assert!(
                !words.contains(['-', '\u{2013}', '\u{2014}', '.']),
                "{kind:?}: {words:?} has a dash or a full stop"
            );
            assert!(
                words
                    .split(' ')
                    .all(|w| !w.chars().skip(1).any(char::is_uppercase)),
                "{kind:?}: {words:?} has a CamelCase word"
            );
            assert!(
                words.chars().next().is_some_and(char::is_uppercase),
                "{kind:?}: {words:?} does not start with a capital"
            );
        }
        assert_eq!(JobKind::Scan.doing(), "Checking your folders");
        assert_eq!(JobKind::RefreshAll.doing(), "Updating show info");
    }

    /// The whole migration for `library_arrange`: a blob written before it
    /// existed reads as Related, and the five values cross the wire as the
    /// shell spells them, "None" included.
    #[test]
    fn a_choice_this_build_does_not_know_falls_back() {
        let blob = r#"{"title_language":"English","library_tab":"Series","library_sort":"SomeNewSort","library_direction":"Sideways","feed_sort":"Recent"}"#;
        let p: Preferences = serde_json::from_str(blob).unwrap();
        assert_eq!(p.library_sort, Sort::Alpha);
        assert_eq!(p.library_direction, Direction::Asc);
        assert_eq!(p.library_tab, Tab::Series, "a known value is kept");
        assert_eq!(p.title_language, TitleLanguage::English);
    }

    #[test]
    fn library_arrange_defaults_for_old_blobs_and_spells_its_values() {
        let old = r#"{"title_language":"English","library_tab":"Movies","library_sort":"MyScore","library_direction":"Desc","feed_sort":"Upcoming","library_show_watched":true}"#;
        let p: Preferences = serde_json::from_str(old).unwrap();
        assert_eq!(p.library_arrange, Arrange::Related);
        assert!(
            p.library_group_franchises,
            "a blob before the toggle groups"
        );
        assert_eq!(p.library_sort, Sort::MyScore);
        for (a, wire) in [
            (Arrange::Related, "Related"),
            (Arrange::Type, "Type"),
            (Arrange::Genre, "Genre"),
            (Arrange::Season, "Season"),
            (Arrange::WatchState, "WatchState"),
            (Arrange::Flat, "None"),
        ] {
            assert_eq!(serde_json::to_value(a).unwrap(), wire);
            assert_eq!(serde_json::from_value::<Arrange>(wire.into()).unwrap(), a);
        }
        let back: Preferences = serde_json::from_value(
            serde_json::to_value(Preferences {
                library_arrange: Arrange::Flat,
                ..Preferences::default()
            })
            .unwrap(),
        )
        .unwrap();
        assert_eq!(back.library_arrange, Arrange::Flat);
    }
}
