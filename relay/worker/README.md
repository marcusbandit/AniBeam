# Room relay on Cloudflare Workers

The watch together relay (#846) as a Cloudflare Worker in Rust, built with
[workers-rs](https://github.com/cloudflare/workers-rs). It serves the same routes
as the native relay in `apps/relay` and drives the same `anibeam-room` state
machine. The design is `docs/superpowers/specs/2026-11-13-watch-together-rooms.md`.

| Route | Answer |
|---|---|
| `POST /rooms` `{"episode": EpisodeRef}` | `201 {"code", "host_key"}`, `400` bad body, `413` over 16 KiB, `429` over the limit |
| `GET /rooms/{code}/ws` | WebSocket upgrade, `404` no such room, `426` not an upgrade, `429` over the limit |
| `GET /{code}` | the join page, `200`, or `404` with a "Room not found" page |
| `GET /health` | `200 ok` |

This crate is its own Cargo workspace, so `cargo test --workspace` at the repo
root never builds it. It only builds for `wasm32-unknown-unknown`.

## How it works

- One SQLite-backed Durable Object per room (`RoomObject`), addressed with
  `id_from_name(code)`. `POST /rooms` draws a code, asks that object to set
  itself up, and draws again when the object answers that it already holds a
  room.
- Sockets use the WebSocket Hibernation API (`accept_websocket_with_tags`,
  `websocket_message`, `websocket_close`, `websocket_error`). An object whose
  sockets are quiet is evicted from memory, and duration is not billed while
  it is.
- Eviction loses memory, so the `Room` is written to storage after every
  message that others see (hello, play, pause, seek, episode, status, bye, a
  drop) and read back on the first event after a wake. Pings and refused
  messages only move the per member rate counters, so they write nothing.
  Each socket's `ConnId` is a random 64 bit id in its tag (`c<id>`).
- Timers run on one Durable Object alarm, set only while a timer is pending:
  120 s after a member drops (grace, host handover), 120 s after a room is made
  (the host never came), and 10 minutes after the room empties. An idle room
  with nobody in it closes on that alarm and its storage is deleted.
- Pings are not served by `setWebSocketAutoResponse`. workers-rs exposes it,
  but an auto-response answers one fixed string with another, and the
  protocol's `ping` carries the client's clock while `pong` carries the
  relay's. Each ping therefore wakes the object and costs one twentieth of a
  request.
- Per address limits use the Workers rate limiting binding: 10 room creations
  and 30 WebSocket opens a minute, as on the native relay, an IPv6 address
  counted by its /64. These limits are approximate and count per Cloudflare
  location. They are checked in the Worker before any Durable Object is
  touched, so a refused request costs no Durable Object request.
- TODO: the native relay also caps open rooms at 10,000. The Worker has no
  global count; the 32^6 code space and the free plan's daily caps stand in
  for it. The join page (`GET /{code}`) has no per address limit either, the
  same as the native relay, and each one costs a Durable Object request.

## Free plan numbers this relies on

- 100,000 Durable Object requests a day. Opening a socket, a join page, a room
  creation and each alarm are one request each; incoming WebSocket messages
  count 20 to 1. Outgoing messages are free.
- 13,000 GB-s of Durable Object duration a day. Hibernation keeps quiet rooms
  out of this.
- 100,000 SQLite rows written a day: each stored change above is one row, each
  alarm set or deleted is one more.
- 5 million rows read a day and 5 GB stored in total.
- Over any daily cap, further requests of that kind fail until midnight UTC.
  Nothing is billed on the free plan.

## Build

    rustup target add wasm32-unknown-unknown
    cargo install worker-build
    worker-build --release

The output lands in `build/` (`build/index.js` plus the wasm). `wrangler` runs
the same build itself through `[build]` in `wrangler.toml`.

## Run locally

    npx wrangler@4.161.0 dev --local --port 8797

No Cloudflare account or login is needed. Point a client at
`http://127.0.0.1:8797`.

## Test

    ./smoke.sh
    SMOKE_QUICK=1 ./smoke.sh
    SMOKE_CLOSE=1 ./smoke.sh

`smoke.sh` starts `wrangler dev` in local mode and runs `smoke.mjs` against it
(node 22 or later, no packages): it makes a room, joins two members, checks
play, seek, ping, the error paths and the join page, restarts the relay to
check the room reloads from storage, waits out the 120 s grace to check the
alarm hands the host role over, and checks the creation limit. `SMOKE_QUICK=1`
skips the two minute timer phase. `SMOKE_CLOSE=1` also waits out the 10 minute
close of an empty room and restarts once more to check its storage is gone.

## Deploy

On the free plan, from this directory:

1. `npx wrangler@4.161.0 login` with the Cloudflare account that will own the
   relay.
2. Pick the address. Either keep the default
   `anibeam-rooms.<account>.workers.dev`, or add a custom domain on a zone in
   that account:

       [[routes]]
       pattern = "rooms.example.com"
       custom_domain = true

3. `npx wrangler@4.161.0 deploy`.

The first deploy creates the `RoomObject` class from the `v1` migration and the
two rate limiters. Then set the app's `room_relay` default to that address.
