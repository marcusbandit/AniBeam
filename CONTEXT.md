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

**Reply**:
What a call returns at once: the data asked for, or the id of the job it started.
_Avoid_: response, result, return value

**Event**:
A fact the core pushes to every subscribed shell. Events reach a shell through a subscription.
_Avoid_: push, notification, signal, callback

**Job**:
Long-running work started by a call that returns at once and reports through events until it finishes: a scan, a match sweep, a crawl.
_Avoid_: task, sweep, operation

### Playback

**Session**:
One run of a player over one file, from the open call to the close call. Ticks belong to a session, and the view and mark rules fire at most once per session.
_Avoid_: playback, play, run

**Tick**:
A playback position report from the player. The single input to the view, mark and completion rules.
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

**Completion**:
The outcome of the end rule: the position reached the last 30 seconds, a known outro window or the end of the file, so the resume point is cleared and the episode is recorded as completed.
_Avoid_: finished, done, watched to the end

**Skip window**:
A span of an episode the player offers to jump over, an intro or an outro, taken from the file's chapters first and from AniSkip otherwise.
_Avoid_: skip times, OP/ED range, chapter

**Track choice**:
The audio and subtitle track a series remembers from the user's last pick in the player, applied to every later file of that series. Off is a choice too.
_Avoid_: preferred track, track preference, aid/sid

**Subtitle defaults**:
The one global set of subtitle options every session starts from, each one an mpv option. There is no per-file or per-series style.
_Avoid_: subtitle style, style record, sub settings

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

### Look

**Token**:
One named value the shell styles from. Every colour, size, radius and spacing in the shell is a token; nothing is set inline.
_Avoid_: variable, style constant, CSS var

**Terminal palette**:
The sixteen ANSI colours plus foreground and background the user's terminal draws with, read from the terminal's own config. It is what the system colour source means by "the user's colours".
_Avoid_: neofetch colours, system colours, Xresources, scheme.json

**Colour source**:
Where the colour tokens come from: the system, meaning the terminal palette and, failing that, the portal's scheme and accent, or a theme.
_Avoid_: override, provider, colour mode

**Theme**:
A named palette file that fills the colour tokens, shipped with the app or dropped into the user's config directory. Two themes are chosen at a time, one per mode.
_Avoid_: skin, colour scheme, style, look

**Mode**:
Dark or light. The setting adds system, which resolves to one of the two.
_Avoid_: scheme, appearance, variant, colour-scheme

**Accent**:
The one colour that marks attention: selection, focus, the active item, progress.
_Avoid_: highlight, primary, brand colour

**Density**:
The spacing setting: compact, normal or comfortable. It scales spacing, control heights and radii, never type or poster size.
_Avoid_: scale, zoom, compact mode

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

**App id**:
The one name a shell's window, its launcher entry, its icon and its bus name share, so the desktop can pair them. Each shell has its own. The Linux shell's is not Electron's, so the two stand side by side until the retire line.
_Avoid_: window class, WM_CLASS, desktop file id, bundle id

**Install**:
A build of the checkout put on a machine. Every install carries a version unique to the commit it came from, so a build is a version, never just a file.
_Avoid_: deploy, release build, dev build
