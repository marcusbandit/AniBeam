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

### Library

**Source**:
A folder the core scans. Every series lives under exactly one source. A source whose path is missing is unavailable, not gone: it stays in the library and its series attach again when the path returns.
_Avoid_: folder source, root, watch folder, library folder

**Series**:
One entry in the library: a show, which is a folder, or a film, which is a single file. The word covers films too, a wart inherited from the Electron line.
_Avoid_: show, title, entry, anime, media

**Match**:
The link from a series to one provider record: AniList (carrying the MAL id when known), MAL alone, or TMDB. A series has at most one match. A match the user applied or imported is confirmed, and the auto-match sweep never replaces it.
_Avoid_: mapping, source mapping, metadata link

**Export**:
The JSON file that carries a library out of one AniBeam and into another. A library export holds sources and series with their matches; a full export adds private data. The same file is the native app's backup.
_Avoid_: backup, dump, migration file

**Private data**:
What the full export adds: tracker accounts and their tokens, API keys, history, preferences.
_Avoid_: secrets, credentials, settings

**Import**:
The job that merges an export into the library. The file wins for matches, flags, accounts and preferences; the newer timestamp wins for history; nothing is deleted.
_Avoid_: restore, load, sync

### Route

**Parity checklist**:
The list of behaviours the Electron app has that the native shell must reproduce, in build order. It lives on its ticket and the spec lifts it.
_Avoid_: feature list, backlog, roadmap, scope

**Switch line**:
The first part of the parity checklist. When every item in it is green, the launcher entry points at the native binary and Electron stays installed beside it.
_Avoid_: MVP, daily-driver build, phase 2 exit

**Retire line**:
The rest of the parity checklist. When it is green too, Electron is deleted.
_Avoid_: full parity, phase 3 gate, the remainder
