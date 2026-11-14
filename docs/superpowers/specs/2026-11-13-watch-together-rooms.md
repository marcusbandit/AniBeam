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
- At most 10 members (`max`), counting the people connected, as the view shows them. The 11th
  gets `full`. A member in their 120 s grace who comes back with their `rejoin_key` gets in
  even when that takes the room past `max`, since their place was kept. Places in grace and
  connected together are capped at twice `max`, so churn cannot grow a room without bound.
- `cmd` from any member: `play` and `pause` set `playing` and `position`; `seek` sets `position`
  and keeps `playing`. `at_ms = now`, `rev += 1`, broadcast `state` to everyone, sender included.
  Positions must be finite, from 0 to 86400.
- `episode` from the host only (else `not_allowed`): replaces the episode, resets the state to
  paused at 0, broadcasts `episode`.
- Leaving or a dropped socket broadcasts `members`. When the host leaves for good (120 s), the
  longest-present member becomes host. An empty room is dropped after 10 minutes.
- Bad JSON or an unknown `t` gets `bad_message`, not a close. 20 messages a second per member
  at most; past that `rate_limited` and the message is dropped. The relays hold every socket to
  the same rate before the room sees a message, member or not (`anibeam_room::limit::Window`),
  and close a socket that has not said hello within 10 s (`limit::HELLO_MS`) with `proto`.

- `bye` leaves for good at once: the rejoin key stops working, the place frees up, and a host
  who says `bye` hands over right away. The relay answers `closed { reason: "bye" }`.
- A `rejoin_key` arriving on a new socket while the old one still looks connected moves the
  member over and sends the old socket `closed { reason: "replaced" }`.
- Fatal codes, after which the relay closes the socket: `full`, `not_found`, `proto` (a message
  before `hello`, or another protocol version). `bad_name` is not fatal, so the client can send
  another `hello`.

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

### Get media (#889), protocol 1 still

Added without a version bump: every new field is optional and a relay or client from before
ignores it, and a relay from before answers the new `seal` with `bad_message`.

- `hello` may carry `key`: the member's X25519 public key for this room session, base64. The
  member list carries it as `Member.key` (at most `MAX_KEY` characters of base64, else dropped).
- `status` may carry `progress { percent, eta? }` while the member downloads the episode;
  `Member.progress` shows it to everyone, cleared when ready or when the episode changes.
- `seal { to, data }` passes `data` (at most `MAX_SEALED`, 8 KB) to one other connected
  member as `sealed { from, data }`, under the same rate. Too large is `bad_message`; nobody
  by that id, or oneself, is `not_allowed`. Neither closes the socket.

`data` is sealed by the core (`core/src/room/seal.rs`): X25519 between the two members' keys,
HKDF-SHA256 salted with the room code with both public keys in its info, XChaCha20-Poly1305
with a random nonce and the sender's then the recipient's key as additional data. The relay
sees neither the torrent link, the info hash, the release's name nor the match ids inside.
What it carries (`core/src/room/media.rs`): `ask`, the host's `offer` (magnet with trackers
while its torrent client still holds the torrent its file came from, the info hash, the
release's and the file's names, group, resolution and the match ids) or `nothing`.

## Relays

Native (`apps/relay`): axum with the `ws` feature, rooms in memory, `anibeam_relay::serve(listener)`
for tests (`serve_with` for a shorter hello wait), `anibeam-relay --listen 0.0.0.0:8795` as a
binary. Per-IP limit of 10 room creations, 30 WebSocket opens and 60 join pages a minute, an
IPv6 address counted by its /64 (`anibeam_room::limit::limit_key`, shared by both relays).

Worker (`relay/worker`): one SQLite-backed Durable Object per room, named by code, using the
WebSocket Hibernation API (`accept_websocket`, `websocket_message`, `websocket_close`). The room
is persisted to Durable Object storage after every change, because a hibernated object loses
its memory; each socket's `ConnId` rides in its attachment or tag. Same routes, the join page
included, and the same limits through three rate limiting bindings (`CREATE_LIMIT`,
`OPEN_LIMIT`, `PAGE_LIMIT`). The join page only wakes a room object for a path that
`code::normalize` takes and that is within the limit. Each socket carries the time it opened
as a second tag: every message checks the hello deadline against it, and an alarm set 10 s
after each open sweeps sockets that never sent anything. Per socket message counts live in
memory only. Free plan only: no paid features. `wrangler.toml` uses `new_sqlite_classes`.

## Core client (`core/src/room/`)

Contract additions:

- Calls: `RoomCreate { session }`, `RoomJoin { code }`, `RoomAttach { session }`,
  `RoomCommand { action: RoomAction, position }` (local seconds), `RoomSetOffset { offset }`,
  `RoomLeave`, `RoomStatus`. Replies: `Reply::Room { room: Option<RoomInfo> }`.
- `RoomInfo { code, link, you, host: bool, members, episode, offset, suggested_offset?,
  session?, connected: bool }`. `link` is `<relay>/<code>`. `session` is the playback session
  `RoomAttach` named, so the shell can tell whether the room drives the player it shows.
- Room errors are `CoreError::Service` with a fixed `code` the shell branches on: `not_found`,
  `full`, `unreachable`, `refused`, `proto`, `not_in_room`. A bad code or a missing name is
  `Invalid` on `code` or `room_name`.
- Events: `RoomChanged { room: Option<RoomInfo> }`, `RoomEpisode { episode, series?, file? }`
  (what a joiner should open; `file` is `None` when this library lacks it),
  `RoomApply { session, playing, seek_to: Option<f64>, speed: f64 }` (debug level, never in the
  activity log).
- Preferences: `room_name` (default empty: the shell asks once) and `room_relay` (default
  `https://rooms.anibeam.app`, changeable). Per-file offsets under a settings key.

Behaviour:

- One room at a time. A background task on `core.handle` owns the socket. The WebSocket is an
  HTTP/1.1 upgrade made by a reqwest client (`net::room_client`), so it is the same TLS stack
  and roots as every other request; tokio-tungstenite only frames the upgraded stream.
  `RoomCreate` and `RoomJoin` wait for the first welcome, so their errors come back as the
  call's error. Reconnects with backoff 1, 2, 4 ... 30 s for 2 minutes
  using `rejoin_key`, then gives up with `RoomChanged { room: None }`.
- Clock: five pings on connect, keep the offset from the lowest round trip; one ping a minute
  after that (also keeps NATs open).
- Joiner's episode: match a local series by AniList, then MAL, then TVmaze id, then the file by
  `episode_key`. Emit `RoomEpisode`. When the host moves the room, a member's attached player
  gets a pause (`RoomApply` with no seek) and is let go before the new `RoomEpisode`.
- Offset: stored per file. With none stored, when both the host's `opening_start` and this
  file's opening start are known and differ by under 30 s, suggest and apply `mine - host`.
  The file that set the room's timeline (the host's, when they made the room or moved it to an
  episode) plays at nought unless its member sets one by hand, and that one is kept in memory
  for the room only, since stored offsets are measured against someone else's file. A host
  change never resets anything: a new host's file keeps its stored offset, because the timeline
  is still the old host's file until the new host moves the room.
- A host whose own move to another episode was lost with the connection makes it again after
  the welcome, rather than following the room back to the episode they left.
- Sync: target local position = `position + (server_now - at_ms)/1000 * rate` (only while
  playing) `+ offset`. `session::tick` hands each tick to the room. On a new `state`, emit
  `RoomApply` with `playing` and a `seek_to` when off by more than 0.5 s. On ticks, off by
  under 0.25 s: nothing (and speed back to 1.0 once under 0.1 s); 0.25 to 2 s: speed 1.05 or
  0.95; over 2 s: seek. Never emit more than one correction a second. A tick only corrects
  while the player and the room agree on playing; a paused room only seeks, past 0.5 s. A tick
  that disagrees with the room on playing gets a `RoomApply` with the room's playing (and a
  seek past 0.5 s), unless this member's own command is still unanswered.
- The relay's order is the room's. A member's own command is taken as done when sent, so the
  player moves at once. While own commands are unanswered, others' states are not applied: the
  relay put them first, and the echo of this member's last command carries the room as it ended
  up. That echo applies nothing when nothing came between and the player agrees on playing;
  otherwise it is applied. An own command unanswered for 5 s is taken as lost, and the relay's
  last state wins.
- Seeks go to the relay at most every 150 ms (`SEEK_GAP`). One inside the gap is held and the
  last held one is sent when the gap is up; a play or pause sends a held seek first. A held
  arrow key repeats 25 to 33 times a second, past the relay's 20 messages.
- Activity log, with words: room created, joined, left, someone joined or left, room closed,
  connection lost and back. Never sync ticks or corrections.
- Integration tests in `core/tests/room.rs` run `anibeam_relay::serve` in-process with two or
  three cores.

## Desktop

- Player chrome: a "Watch together" button opens the room panel. Not in a room: "Start a room"
  and a code field to join. In a room: the code large, the link with Copy, a QR code (from the
  `qrcode` crate as SVG), people with ready state, the offset with -0.5 / +0.5 and the
  suggestion (for everyone, the host included), Leave.
- Outside the player: "Join a room" reachable from the frame (and `Ctrl+J`), the same code field.
- Joining with nothing open: on `RoomEpisode` with a file, open the player on it and call
  `RoomAttach`; without one, say the episode is missing from this library.
- In a room, the user's play, pause and seek also send `RoomCommand`; `RoomApply` sets mpv's
  `pause`, seeks and `speed` directly without sending anything back. Hold-for-2x is off in a room.
- Auto-skip in a room: only the host's player auto-skips, and its skip is a seek for the whole
  room. Everyone else's setting is ignored while in a room, so one setting moves everyone once
  and two members with different openings never send competing seeks. A skip that stayed local
  is not an option: the next tick would see the player over 2 s off and seek it back. Pressing
  Skip intro (or Ctrl+Right) is anyone's own seek and goes to the room like any other.
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
