//! The room protocol, version 1: JSON text frames, internally tagged with `"t"`,
//! snake_case.
//!
//! Times are milliseconds since the Unix epoch on the relay's clock. Positions
//! are seconds on the room's canonical timeline, which is the host's file; each
//! client converts with its own offset, `local = canonical + offset`.

use serde::{Deserialize, Serialize};

/// The protocol version a client says in [`ClientMsg::Hello`].
pub const PROTO: u32 = 1;

/// The most people a room holds.
pub const MAX_MEMBERS: u32 = 10;

/// The largest position a command may carry, in seconds (a day).
pub const MAX_POSITION: f64 = 86_400.0;

/// The id the protocol shows for one person in a room. Stable across a rejoin.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct MemberId(pub u32);

/// Which episode a room watches. It names the episode and never carries video,
/// links or infohashes.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EpisodeRef {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub anilist_id: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mal_id: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tvmaze_id: Option<i64>,
    pub episode_key: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub number: Option<f64>,
    pub series_title: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub episode_title: Option<String>,
    /// The host's file length, in seconds.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration: Option<f64>,
    /// Where the opening starts in the host's file, in seconds. Joiners compare it
    /// with their own file's to suggest an offset.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub opening_start: Option<f64>,
}

/// The room's play state. While `playing`, the canonical position now is
/// `position + (now - at_ms) / 1000 * rate`.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct PlayState {
    pub playing: bool,
    pub position: f64,
    pub rate: f64,
    pub at_ms: u64,
    /// Goes up by one on every change, so a client can drop a stale state.
    pub rev: u64,
}

/// Whether a member has the episode open.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Ready {
    Ready,
    Missing,
    #[default]
    Loading,
}

/// One person in a room, as everyone in it sees them.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Member {
    pub id: MemberId,
    pub name: String,
    pub host: bool,
    pub ready: Ready,
    pub joined_ms: u64,
}

/// The whole room, as a joiner gets it in [`ServerMsg::Welcome`].
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RoomView {
    pub code: String,
    /// The host, when they are connected.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub host: Option<MemberId>,
    pub members: Vec<Member>,
    pub episode: EpisodeRef,
    pub state: PlayState,
    pub max: u32,
}

/// What a relay answers a bad or refused message with. The fatal ones
/// ([`ErrorCode::is_fatal`]) close the connection after the error is sent.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCode {
    Full,
    NotFound,
    BadName,
    BadMessage,
    NotAllowed,
    Proto,
    RateLimited,
}

impl ErrorCode {
    /// True when the relay closes the connection after sending this error.
    pub fn is_fatal(self) -> bool {
        matches!(self, Self::Full | Self::NotFound | Self::Proto)
    }
}

/// A playback command. `play` and `pause` set the position too; `seek` keeps
/// whether the room plays.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Action {
    Play,
    Pause,
    Seek,
}

/// A message from a client to the relay.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "t", rename_all = "snake_case")]
pub enum ClientMsg {
    /// The first message on every connection.
    Hello {
        name: String,
        proto: u32,
        /// The client and its version, `anibeam/2.5.0`.
        client: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        host_key: Option<String>,
        /// From an earlier [`ServerMsg::Welcome`]: the same member comes back.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        rejoin_key: Option<String>,
    },
    /// A clock probe; `c` is the client's clock in milliseconds.
    Ping { c: u64 },
    /// Play, pause or seek, at a canonical position in seconds.
    Cmd { action: Action, position: f64 },
    /// Move the room to another episode. The host only.
    Episode { episode: EpisodeRef },
    /// Whether this member has the episode open.
    Status { ready: Ready },
    /// Leaving for good: the member's rejoin key stops working at once.
    Bye,
}

impl ClientMsg {
    pub fn parse(text: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(text)
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(self).expect("a client message always serializes")
    }
}

/// A message from the relay to a client.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "t", rename_all = "snake_case")]
pub enum ServerMsg {
    /// The answer to a hello: who you are and the whole room.
    Welcome {
        you: MemberId,
        rejoin_key: String,
        room: RoomView,
        server_ms: u64,
    },
    /// The play state changed.
    State {
        state: PlayState,
        by: MemberId,
    },
    /// Someone joined, left, or changed their ready state.
    Members {
        members: Vec<Member>,
    },
    /// The host moved the room to another episode; the state starts paused at 0.
    Episode {
        episode: EpisodeRef,
        state: PlayState,
        by: MemberId,
    },
    /// The answer to a ping: the client's `c` back and the relay's clock.
    Pong {
        c: u64,
        s: u64,
    },
    Error {
        code: ErrorCode,
        message: String,
    },
    /// The relay is closing this connection on purpose.
    Closed {
        reason: String,
    },
}

impl ServerMsg {
    pub fn parse(text: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(text)
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(self).expect("a server message always serializes")
    }

    /// True when the relay closes the connection right after sending this.
    pub fn ends_connection(&self) -> bool {
        match self {
            Self::Closed { .. } => true,
            Self::Error { code, .. } => code.is_fatal(),
            _ => false,
        }
    }
}

/// The body of `POST /rooms`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CreateRoom {
    pub episode: EpisodeRef,
}

/// The `201` answer to `POST /rooms`. Whoever hellos with `host_key` is the host.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CreatedRoom {
    pub code: String,
    pub host_key: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn episode() -> EpisodeRef {
        EpisodeRef {
            anilist_id: Some(21),
            mal_id: None,
            tvmaze_id: None,
            episode_key: "e5".into(),
            number: Some(5.0),
            series_title: "One Piece".into(),
            episode_title: None,
            duration: Some(1440.0),
            opening_start: Some(30.5),
        }
    }

    #[test]
    fn hello_reads_the_wire_form() {
        let msg = ClientMsg::parse(
            r#"{"t":"hello","name":"Ann","proto":1,"client":"anibeam/2.5.0","host_key":"ab"}"#,
        )
        .unwrap();
        assert_eq!(
            msg,
            ClientMsg::Hello {
                name: "Ann".into(),
                proto: 1,
                client: "anibeam/2.5.0".into(),
                host_key: Some("ab".into()),
                rejoin_key: None,
            }
        );
    }

    #[test]
    fn client_messages_round_trip() {
        let all = [
            ClientMsg::Hello {
                name: "Ann".into(),
                proto: PROTO,
                client: "anibeam/2.5.0".into(),
                host_key: None,
                rejoin_key: Some("k".into()),
            },
            ClientMsg::Ping { c: 12 },
            ClientMsg::Cmd {
                action: Action::Seek,
                position: 61.25,
            },
            ClientMsg::Episode { episode: episode() },
            ClientMsg::Status {
                ready: Ready::Missing,
            },
            ClientMsg::Bye,
        ];
        for msg in all {
            assert_eq!(ClientMsg::parse(&msg.to_json()).unwrap(), msg);
        }
    }

    #[test]
    fn tags_and_enums_are_snake_case() {
        let cmd = ClientMsg::Cmd {
            action: Action::Play,
            position: 3.0,
        };
        let v: serde_json::Value = serde_json::from_str(&cmd.to_json()).unwrap();
        assert_eq!(v, json!({"t": "cmd", "action": "play", "position": 3.0}));
        assert_eq!(ClientMsg::Bye.to_json(), r#"{"t":"bye"}"#);
        let err = ServerMsg::Error {
            code: ErrorCode::RateLimited,
            message: "slow down".into(),
        };
        let v: serde_json::Value = serde_json::from_str(&err.to_json()).unwrap();
        assert_eq!(
            v,
            json!({"t": "error", "code": "rate_limited", "message": "slow down"})
        );
        let status = ClientMsg::parse(r#"{"t":"status","ready":"loading"}"#).unwrap();
        assert_eq!(
            status,
            ClientMsg::Status {
                ready: Ready::Loading
            }
        );
    }

    #[test]
    fn integer_positions_read_as_seconds() {
        let msg = ClientMsg::parse(r#"{"t":"cmd","action":"pause","position":90}"#).unwrap();
        assert_eq!(
            msg,
            ClientMsg::Cmd {
                action: Action::Pause,
                position: 90.0
            }
        );
    }

    #[test]
    fn unknown_tags_and_missing_fields_fail_to_parse() {
        assert!(ClientMsg::parse(r#"{"t":"dance"}"#).is_err());
        assert!(ClientMsg::parse(r#"{"t":"cmd","action":"play"}"#).is_err());
        assert!(ClientMsg::parse(r#"{"t":"cmd","action":"rewind","position":1}"#).is_err());
        assert!(ClientMsg::parse("not json").is_err());
    }

    #[test]
    fn episode_ref_leaves_out_what_it_does_not_know() {
        let e = EpisodeRef {
            anilist_id: None,
            mal_id: None,
            tvmaze_id: None,
            episode_key: "k".into(),
            number: None,
            series_title: "S".into(),
            episode_title: None,
            duration: None,
            opening_start: None,
        };
        let v: serde_json::Value = serde_json::to_value(&e).unwrap();
        assert_eq!(v, json!({"episode_key": "k", "series_title": "S"}));
        let back: EpisodeRef = serde_json::from_value(v).unwrap();
        assert_eq!(back, e);
    }

    #[test]
    fn server_messages_round_trip() {
        let state = PlayState {
            playing: true,
            position: 12.5,
            rate: 1.0,
            at_ms: 1_000,
            rev: 3,
        };
        let member = Member {
            id: MemberId(1),
            name: "Ann".into(),
            host: true,
            ready: Ready::Ready,
            joined_ms: 5,
        };
        let all = [
            ServerMsg::Welcome {
                you: MemberId(1),
                rejoin_key: "r".into(),
                room: RoomView {
                    code: "K7QM2X".into(),
                    host: Some(MemberId(1)),
                    members: vec![member.clone()],
                    episode: episode(),
                    state,
                    max: MAX_MEMBERS,
                },
                server_ms: 9,
            },
            ServerMsg::State {
                state,
                by: MemberId(2),
            },
            ServerMsg::Members {
                members: vec![member],
            },
            ServerMsg::Episode {
                episode: episode(),
                state,
                by: MemberId(1),
            },
            ServerMsg::Pong { c: 1, s: 2 },
            ServerMsg::Error {
                code: ErrorCode::Full,
                message: "full".into(),
            },
            ServerMsg::Closed {
                reason: "bye".into(),
            },
        ];
        for msg in all {
            assert_eq!(ServerMsg::parse(&msg.to_json()).unwrap(), msg);
        }
    }

    #[test]
    fn fatal_errors_and_closed_end_the_connection() {
        let fatal = [ErrorCode::Full, ErrorCode::NotFound, ErrorCode::Proto];
        let kept = [
            ErrorCode::BadName,
            ErrorCode::BadMessage,
            ErrorCode::NotAllowed,
            ErrorCode::RateLimited,
        ];
        for code in fatal {
            let msg = ServerMsg::Error {
                code,
                message: String::new(),
            };
            assert!(msg.ends_connection(), "{code:?}");
        }
        for code in kept {
            let msg = ServerMsg::Error {
                code,
                message: String::new(),
            };
            assert!(!msg.ends_connection(), "{code:?}");
        }
        assert!(
            ServerMsg::Closed {
                reason: "bye".into()
            }
            .ends_connection()
        );
        assert!(!ServerMsg::Pong { c: 0, s: 0 }.ends_connection());
    }

    #[test]
    fn create_room_bodies_read_and_write() {
        let body: CreateRoom = serde_json::from_value(json!({
            "episode": {"episode_key": "k", "series_title": "S", "anilist_id": 5}
        }))
        .unwrap();
        assert_eq!(body.episode.anilist_id, Some(5));
        let created = CreatedRoom {
            code: "K7QM2X".into(),
            host_key: "00".into(),
        };
        let v = serde_json::to_value(&created).unwrap();
        assert_eq!(v, json!({"code": "K7QM2X", "host_key": "00"}));
    }
}
