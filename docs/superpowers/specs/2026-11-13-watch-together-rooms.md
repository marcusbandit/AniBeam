# Watch together rooms, phase 1: friends rooms (#846)

Everyone in a room plays their own copy of the episode. A room carries the episode's identity,
the play state, who is in it and how far along each person's file is. It never carries video,
download links or infohashes (the app ships no sources).

Phase 1 covers rooms of 2 to 10 people where everyone controls playback: create, join by code,
link or QR, play, pause, seek, clock sync, per-viewer offset with a suggestion from the opening,
and the host moving the room to the next episode. Roles beyond "host", 100-person rooms, chat and
"Find this episode" are later phases.

## Pieces and who owns which files

| Piece | Path | Crate |
|---|---|---|
| Protocol and room state machine, no I/O | `room/` | `anibeam-room` (workspace member) |
| Native relay: self-hosting, local dev, core tests | `apps/relay/` | `anibeam-relay` (lib + bin, workspace member) |
| Cloudflare Worker relay, free plan | `relay/worker/` | own Cargo workspace, workers-rs, wasm only |
| Room client in the core | `core/src/room/`, contract additions | `anibeam-core` |
| Desktop UI | `apps/linux/` | `anibeam` |

The phone gets rooms in a follow-up on the same issue.

## Protocol (`anibeam-room::proto`), version 1

JSON text frames, internally tagged with `"t"`, snake_case. Times are milliseconds since the
Unix epoch on the relay's clock (`u64`). Positions are seconds (`f64`) on the room's canonical
timeline, which is the host's file. Each client converts with its own offset:
`local = canonical + offset`.

HTTP routes on the relay (same on both relays):

- `POST /rooms` body `{ "episode": EpisodeRef }` → `201 { "code": "K7QM2X", "host_key": "<32 hex>" }`.
  The relay generates the code and retries on a collision.
- `GET /rooms/{code}/ws` WebSocket upgrade. `404` when the room does not exist.
- `GET /{code}` the join page (HTML from `anibeam_room::page::join_html`), with Open Graph tags
  naming the series and episode so a pasted link previews in Discord. It tries
  `anibeam://room/{code}` and otherwise points at `https://anibeam.app/download/`.
- `GET /health` → `200 ok`.

Codes: 6 characters from `ABCDEFGHJKLMNPQRSTUVWXYZ23456789` (no I, O, 0, 1). `code::normalize`
uppercases and drops spaces and dashes, and returns `None` for anything else.

Client → relay:

```
hello   { name, proto: 1, client: "anibeam/2.5.0", host_key?: str, rejoin_key?: str }
ping    { c: client_ms }
cmd     { action: "play" | "pause" | "seek", position: f64 }        canonical seconds
episode { episode: EpisodeRef }                                     host only
status  { ready: "ready" | "missing" | "loading" }
bye     {}
```

Relay → client:

```
welcome { you: MemberId, rejoin_key, room: RoomView, server_ms }
state   { state: PlayState, by: MemberId }
members { members: [Member] }
episode { episode: EpisodeRef, state: PlayState, by }
pong    { c, s: server_ms }
error   { code: ErrorCode, message }       then the relay closes when the code is fatal
closed  { reason }
```

Types:

```
EpisodeRef { anilist_id?, mal_id?, tvmaze_id?: i64, episode_key: str, number?: f64,
             series_title: str, episode_title?: str, duration?: f64, opening_start?: f64 }
PlayState  { playing: bool, position: f64, rate: f64, at_ms: u64, rev: u64 }
Member     { id: u32, name: str, host: bool, ready: Ready, joined_ms: u64 }
RoomView   { code, host?: MemberId, members, episode: EpisodeRef, state: PlayState, max: u32 }
ErrorCode  full | not_found | bad_name | bad_message | not_allowed | proto | rate_limited
```

Rules the state machine enforces (`anibeam_room::Room`, pure, serializable with serde so the
Worker can persist it):

- `hello` must be the first message. Names are trimmed, 1 to 32 characters, no control
  characters. A `host_key` matching the room's makes that member the host. A `rejoin_key` from
  an earlier `welcome` within 120 s of leaving gets the same id and host flag back.
- At most 10 members (`max`). The 11th gets `full`.
- `cmd` from any member: `play` and `pause` set `playing` and `position`; `seek` sets `position`
  and keeps `playing`. `at_ms = now`, `rev += 1`, broadcast `state` to everyone, sender included.
  Positions must be finite, from 0 to 86400.
- `episode` from the host only (else `not_allowed`): replaces the episode, resets the state to
  paused at 0, broadcasts `episode`.
- Leaving or a dropped socket broadcasts `members`. When the host leaves for good (120 s), the
  longest-present member becomes host. An empty room is dropped after 10 minutes.
- Bad JSON or an unknown `t` gets `bad_message`, not a close. 20 messages a second per member
  at most; past that `rate_limited` and the message is dropped.

- `bye` leaves for good at once: the rejoin key stops working, the place frees up, and a host
  who says `bye` hands over right away. The relay answers `closed { reason: "bye" }`.
- A `rejoin_key` arriving on a new socket while the old one still looks connected moves the
  member over and sends the old socket `closed { reason: "replaced" }`.
- Fatal codes, after which the relay closes the socket: `full`, `not_found`, `proto` (a message
  before `hello`, or another protocol version). `bad_name` is not fatal, so the client can send
  another `hello`. A member in their 120 s grace still holds a place toward `max`.

API shape: `Room::new(code, host_key, episode, now_ms, secret: [u8; 16])`,
`Room::handle(&mut self, conn: ConnId, msg: ClientMsg, now_ms) -> Vec<Out>`,
`Room::handle_text(&mut self, conn, text: &str, now_ms) -> Vec<Out>` (parses, and answers
`bad_message` itself), `Room::disconnect(&mut self, conn, now_ms) -> Vec<Out>`,
`Room::tick(now_ms) -> Vec<Out>` for the 120 s and 10 min timers, `Room::is_closed()` once the
relay should drop it, `Out { to: To::{One(ConnId), All, AllBut(ConnId)}, msg }`, and
`Room::targets(&To) -> Vec<ConnId>`, which the relay calls right after each call to turn `All`
into the connected members' sockets. A message whose `ServerMsg::ends_connection()` is true
closes its socket after sending. `ConnId` is the transport's handle (a socket); `MemberId` is
what the protocol shows. `secret` is 16 random bytes the room keys its rejoin keys with
(SipHash-2-4): the crate has no randomness of its own, so it builds for wasm32 unchanged, and
deriving the keys from `host_key` instead would let the host forge everyone's.

## Relays

Native (`apps/relay`): axum with the `ws` feature, rooms in memory, `anibeam_relay::serve(listener)`
for tests, `anibeam-relay --listen 0.0.0.0:8795` as a binary. Per-IP limit of 10 room creations
and 30 WebSocket opens a minute.

Worker (`relay/worker`): one SQLite-backed Durable Object per room, named by code, using the
WebSocket Hibernation API (`accept_websocket`, `websocket_message`, `websocket_close`). The room
is persisted to Durable Object storage after every change, because a hibernated object loses
its memory; each socket's `ConnId` rides in its attachment or tag. Same routes, the join page
included. Free plan only: no paid features. `wrangler.toml` uses `new_sqlite_classes`.

## Core client (`core/src/room/`)

Contract additions:

- Calls: `RoomCreate { session }`, `RoomJoin { code }`, `RoomAttach { session }`,
  `RoomCommand { action: RoomAction, position }` (local seconds), `RoomSetOffset { offset }`,
  `RoomLeave`, `RoomStatus`. Replies: `Reply::Room { room: Option<RoomInfo> }`.
- `RoomInfo { code, link, you, host: bool, members, episode, offset, suggested_offset?,
  connected: bool }`. `link` is `<relay>/<code>`.
- Events: `RoomChanged { room: Option<RoomInfo> }`, `RoomEpisode { episode, series?, file? }`
  (what a joiner should open; `file` is `None` when this library lacks it),
  `RoomApply { session, playing, seek_to: Option<f64>, speed: f64 }` (debug level, never in the
  activity log).
- Preferences: `room_name` (default empty: the shell asks once) and `room_relay` (default
  `https://rooms.anibeam.app`, changeable). Per-file offsets under a settings key.

Behaviour:

- One room at a time. A background task on `core.handle` owns the socket (tokio-tungstenite,
  rustls, matching reqwest's TLS setup). Reconnects with backoff 1, 2, 4 ... 30 s for 2 minutes
  using `rejoin_key`, then gives up with `RoomChanged { room: None }`.
- Clock: five pings on connect, keep the offset from the lowest round trip; one ping a minute
  after that (also keeps NATs open).
- Joiner's episode: match a local series by AniList, then MAL, then TVmaze id, then the file by
  `episode_key`. Emit `RoomEpisode`.
- Offset: stored per file. With none stored, when both the host's `opening_start` and this
  file's opening start are known and differ by under 30 s, suggest and apply `mine - host`.
- Sync: target local position = `position + (server_now - at_ms)/1000 * rate` (only while
  playing) `+ offset`. `session::tick` hands each tick to the room. On a new `state`, emit
  `RoomApply` with `playing` and a `seek_to` when off by more than 0.5 s. On ticks, off by
  under 0.25 s: nothing (and speed back to 1.0 once under 0.1 s); 0.25 to 2 s: speed 1.05 or
  0.95; over 2 s: seek. Never emit more than one correction a second.
- Activity log, with words: room created, joined, left, someone joined or left, room closed,
  connection lost and back. Never sync ticks or corrections.
- Integration tests in `core/tests/room.rs` run `anibeam_relay::serve` in-process with two or
  three cores.

## Desktop

- Player chrome: a "Watch together" button opens the room panel. Not in a room: "Start a room"
  and a code field to join. In a room: the code large, the link with Copy, a QR code (from the
  `qrcode` crate as SVG), people with ready state, the offset with -0.5 / +0.5 and the
  suggestion, Leave.
- Outside the player: "Join a room" reachable from the frame (and `Ctrl+J`), the same code field.
- Joining with nothing open: on `RoomEpisode` with a file, open the player on it and call
  `RoomAttach`; without one, say the episode is missing from this library.
- In a room, the user's play, pause and seek also send `RoomCommand`; `RoomApply` sets mpv's
  `pause`, seeks and `speed` directly without sending anything back. Hold-for-2x is off in a room.
- The host's player moving to the next episode sends the new `EpisodeRef` (host only); the
  others follow through `RoomEpisode`.
- Links: `anibeam://room/<code>` registered in the .desktop file (`x-scheme-handler/anibeam`),
  accepted on the command line and through the single-instance `Open` handoff.
- Settings: name and relay URL rows. Onboarding: one skippable "Watch together" step that asks
  for the name and shows what a room looks like.
- Words in `i18n/en/desktop.ftl`, a change file `apps/linux/changes/846-watch-together-rooms.md`.

## Deploy

The Worker deploys with `npx wrangler deploy` from `relay/worker` to the owner's Cloudflare
account on the free plan. The address (`rooms.anibeam.app` or a workers.dev name) is settled with
the owner before the first deploy; until then the default relay URL is a placeholder and local
development runs `anibeam-relay`.
