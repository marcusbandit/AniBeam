pub mod calls;
pub mod enums;
pub mod error;
pub mod events;
pub mod records;

pub use calls::*;
pub use enums::*;
pub use error::*;
pub use events::*;
pub use records::*;

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::UNIX_EPOCH;

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

    #[test]
    fn events_round_trip() {
        let event = Event {
            seq: 1,
            at: UNIX_EPOCH,
            level: Level::Info,
            stage: Stage::Library,
            message: "scan finished".into(),
            job: Some(JobRef {
                id: 4,
                kind: JobKind::Scan,
                phase: JobPhase::Finished,
            }),
            body: EventBody::ScanFinished {
                source: None,
                added: 2,
                changed: 0,
                removed: 1,
            },
        };
        let json = serde_json::to_string(&event).unwrap();
        let back: Event = serde_json::from_str(&json).unwrap();
        assert_eq!(back, event);
    }

    #[test]
    fn job_kinds_know_their_stage_and_serialisation() {
        assert_eq!(JobKind::Scan.stage(), Stage::Library);
        assert_eq!(JobKind::Crawl.stage(), Stage::Franchise);
        assert!(JobKind::Scan.one_at_a_time());
        assert!(!JobKind::Search.one_at_a_time());
        assert!(JobKind::FillImages.one_at_a_time());
    }

    #[test]
    fn subtitle_defaults_are_mpvs_stock_values() {
        let d = SubtitleDefaults::default();
        assert_eq!(d.subtitle_languages, vec!["en".to_string()]);
        assert_eq!(d.audio_languages, vec!["ja".to_string()]);
        assert_eq!(d.scale, 1.0);
        assert_eq!(d.ass_override, AssOverride::ScaleOnly);
        assert_eq!(d.text_style.font, "sans-serif");
        assert_eq!(
            d.text_style.colour,
            Colour {
                r: 255,
                g: 255,
                b: 255,
                a: 255
            }
        );
        assert_eq!(d.text_style.outline_size, 1.65);
        assert_eq!(
            d.text_style.outline_colour,
            Colour {
                r: 0,
                g: 0,
                b: 0,
                a: 255
            }
        );
        assert_eq!(d.text_style.position, 100.0);
    }

    #[test]
    fn preferences_default_to_electrons_defaults() {
        let p = Preferences::default();
        assert_eq!(p.title_language, TitleLanguage::Romaji);
        assert_eq!(p.library_tab, Tab::All);
        assert_eq!(p.library_sort, Sort::Alpha);
        assert_eq!(p.library_direction, Direction::Asc);
        assert_eq!(p.feed_sort, FeedSort::Recent);
    }
}
