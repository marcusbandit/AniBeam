# AniBeam

A local anime library: it scans folders, matches what it finds against AniList, Jikan and TMDB, plays the files, and keeps trackers up to date. The native line splits the app into one core and one shell per platform; this glossary is the vocabulary both halves share.

## Language

### Architecture

**Core**:
The Rust crate that holds every rule: scanning, matching, trackers, franchise graph, playback rules, storage. There is one, and every shell uses it.
_Avoid_: backend, unified backend, engine

**Shell**:
A per-platform app that owns the window, the input and the video surface, and nothing with a rule in it.
_Avoid_: frontend, client, renderer

**Bridge**:
The generated binding layer between the core and one shell. cxx-qt is the Linux bridge, uniffi the macOS one.
_Avoid_: binding layer, FFI layer, IPC

**Call**:
A request a shell sends to the core, expressed as plain data (an enum variant with fields), never a function reference, so the same call can travel in-process or over a socket.
_Avoid_: command, request, message, IPC channel

**Event**:
A fact the core pushes to every subscribed shell. Events reach a shell through a subscription.
_Avoid_: push, notification, signal, callback

**Job**:
Long-running work started by a call that returns at once and reports through events until it finishes: a scan, a match sweep, a crawl.
_Avoid_: task, sweep, operation

### Playback

**Tick**:
A playback position report from whichever player is running. The single input to the view and mark rules.
_Avoid_: progress report, time-pos update, heartbeat

**View**:
The outcome of the 30 second rule: enough forward playback happened that the episode counts as seen, recorded locally in view history.
_Avoid_: watched, seen, play

**Mark**:
The outcome of the 85 percent rule: the episode is reported as completed to a connected tracker.
_Avoid_: complete, progress update, sync

**Resume point**:
The position playback restarts from the next time an episode opens. Not recorded for a session the player never reported on.
_Avoid_: position, progress, bookmark
