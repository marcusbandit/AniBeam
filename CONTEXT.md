# Anibeam

A local anime library: it scans folders, matches what it finds against AniList and Jikan, plays the files, and keeps trackers up to date. The native line splits the app into one core and one shell per platform; this glossary is the vocabulary both halves share.

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

**Catalog**:
The Fluent files under `i18n/` that hold every word a person reads, one folder per language, English the source. Code asks for a message by its ID through `tr`; no shell writes a word of its own.
_Avoid_: translations, strings file, locale files

**Message ID**:
The stable name of one message in the catalog, saying where it shows and what it says (`player-skip-intro`), never the English text.
_Avoid_: key, string key, label

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

**On hold**:
A tracker list: AniList calls it Paused, MAL calls it On Hold. With the On hold switch on, which is the default, the core moves a Watching entry there once nothing has touched it for the set number of days, 60 unless changed. A touch is any change either tracker dates, or a play here. An episode past the entry's progress coming out starts the count again, and an entry waiting for its next episode never moves. The check runs after each tracker refresh: at launch, on Sync, on a new connection, and when the switch goes on. Each entry is read live from its tracker just before it moves. A view of the series here, or a mark, puts the entry back on Watching. The desktop runs all of this and the phone none of it. Home shows the series on hold after Continue, in the same row, whenever Continue leaves room there, and a card on hold says so in a tag.
_Avoid_: paused, idle, stale, backlog

**Local list**:
The list status and score kept on this device for a series no connected tracker carries: someone with no account, a show matched to a tracker whose account is not connected, a TVmaze show, an unmatched folder. The status menu and the score picker write it at once with no job; a card shows it wherever no tracker entry says otherwise, and a tracker's entry always wins. Export carries it as `series[].list`. Nothing in it leaves the device until the person confirms moving it to a tracker: once a tracker connects and its list is read, a sheet shows each show it would change, this computer's side beside the tracker's, and Move sends them; Not now leaves a pointer under Settings, Accounts. The rows stay after a move, and the tracker's entry wins from then on.
_Avoid_: offline list, guest list

**Rewatch**:
Watching a completed show again. AniList keeps a rewatch under way as the Repeating list; MAL keeps it as a flag on Watching. The rewatch count goes up one when a rewatch finishes: a mark completes a Repeating entry, or the last episode of a completed show is marked after its first episode was watched to the end here since the list last changed. A film counts when its list has stood still for twelve hours. Playing only the last episode again is not a rewatch, and the status menu never counts one.
_Avoid_: repeat, rerun

**List dates**:
The start and finish dates on a tracker's list entry. A write fills one in when the list has none, dated the UTC day: the start when the entry goes onto Watching or its first episode is marked, the finish when it goes onto Completed. A date already there is never moved.
_Avoid_: watch dates, history dates

**List notes**:
Free text on a tracker's list entry: AniList's notes, MAL's comments, at most 2000 characters. Written from the series page to every connected tracker, read back with each list refresh. Not the schedule notes, which are about a season's dates.
_Avoid_: comments, schedule notes

**Schedule agent**:
The coding agent on this computer that looks a show's episode dates up from the Episodes and dates sheet (#419): Claude Code today, run as a live session over its own stream protocol in an empty folder with only web search and web fetch, safe mode and nothing saved. It gets the show and the dates the library has, says what it does as it goes, and answers in lines the schedule parser reads; the answer lands in the sheet's paste box, so the tiles preview it and nothing is kept until Save. Found on the machine, never shipped; with none, the sheet only says what one would do. ACP, and Codex through it, comes next.
_Avoid_: AI, assistant, bot

**Resume point**:
The position playback restarts from the next time an episode opens. Not recorded for a session the player never reported on.
_Avoid_: position, progress, bookmark

**Completion**:
The outcome of the end rule: the position reached the last 30 seconds, a known outro window or the end of the file, so the resume point is cleared and the episode is recorded as completed.
_Avoid_: finished, done, watched to the end

**Profile**:
One person's own progress on a computer they share over the tailnet: their history, resume points and tracker accounts. A profile is the Tailscale user a device belongs to, so nobody signs in. The first profile is the person the computer belongs to. The library and the metadata are everyone's.
_Avoid_: account, user, login

**Watch log**:
Every finish of an episode, each with its own time and where it came from: played here, set by hand, or imported. A completion records the latest finish for the series page and the cards, and adds one more to the log. Stats reads the log and leaves out what was set by hand. History from before the log came in as one finish per episode.
_Avoid_: watch history, play log, finishes table

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
The link from a series to one provider record: AniList (carrying the MAL id when known), MAL alone, or TVmaze, which holds the series AniList has no entry for, western animation and live-action television. A TMDB match only arrives through an import and carries its ids with nothing behind them. A series has at most one match. The auto-match tries the providers in the user's provider order, AniList then TVmaze by default, and keeps the first that matches. A match the user applied or imported is confirmed, and the auto-match sweep never replaces it. A match the user cleared keeps the auto-match off the series until they pick one, whatever matcher version comes later. A TVmaze match carries no tracker ids, so it is never marked or synced.
_Avoid_: mapping, source mapping, metadata link

**Provider order**:
The order the auto-match tries the match providers in, and the one the match panel offers first. Every match provider appears in it exactly once; it reorders them and never switches one off.
_Avoid_: provider priority, fallback list

**Missing**:
A series whose path is gone while its source is available. It keeps its match, its flags and its history, leaves every list but the Metadata table, and attaches again when the path returns. An import creates a missing series for a path it cannot find.
_Avoid_: orphaned, stale, deleted, dead

**Forget**:
Removing a missing series with everything it holds. The only way a series leaves the library other than its source being removed.
_Avoid_: delete, purge, remove series

**Export**:
The JSON file that carries a library out of one Anibeam and into another. A library export holds sources and series with their matches; a full export adds private data. The same file is the native app's backup.
_Avoid_: backup, dump, migration file

**Private data**:
What the full export adds: tracker accounts and their tokens, API keys, history, preferences.
_Avoid_: secrets, credentials, settings

**Import**:
The job that merges an export into the library. The file wins for matches, flags, accounts and preferences; the newer timestamp wins for history; nothing is deleted.
_Avoid_: restore, load, sync

**Reading**:
What the core takes a file to be: an episode with its number and season, or an extra (an opening, ending, preview, special or other) with its index. The scan reads it from the file's name, then from the series' own record, where a run of specials or OVAs reads its numbered specials as its episodes, then from any quick fix.
_Avoid_: classification, label, parse

**Quick fix**:
The user saying what a file's reading should be when its name read wrong. It is stored by the file's path, so it outlives every scan, and the file's history follows it to its new key. The series page offers one by itself when it sees a likely misreading, and applying one can file it as a GitHub issue, triaged first by Claude Code or Codex when one is installed.
_Avoid_: override, correction, remap, manual match

**Airing now**:
Every show AniList has on air, not only the library's, read whole from AniList and kept apart from the library's own tables, so a show only browsed never reaches the franchise graph or the image fill. Its other pick is the next season by name. Along the top sits Out now, the episodes out in the last 24 hours, newest first.
_Avoid_: seasonal, schedule, calendar, discover

**Not interested**:
The user saying one AniList entry should stay off Airing now. It covers that entry and nothing related to it, so the next season of the same show is a new entry and appears again. Settings, Library lists every one with a way back.
_Avoid_: hidden, ignored, blocked, dismissed

**Recommended**:
The shows ranked for the user from their own list, best first: Home's Recommended for you row and its See all page. A show is in it while the user has yet to watch it, which is on no list, on Planning, or on Watching or On hold with a first episode or two seen. All holds every such show; In library only those the library has files for; On list only those on one of the user's lists, whether the library has them or not. Not interested keeps a show off it too. The ranking is learnt again after every tracker refresh, from a cache of AniList reads that stays apart from the library's tables, the way Airing now's does.
_Avoid_: suggestions, picks, discover, for you feed

**Forecast**:
The percentage on a recommended show: the share of the user's rated shows it is predicted to beat. 84% means the predicted score sits above 84% of what they rated.
_Avoid_: match, match score, compatibility, rating

**Hidden tag**:
A tag AniList marks as a spoiler. It counts toward the ranking like any other, and nothing names it: the app only says how many the user loves are behind a show, as "A hidden tag you love".
_Avoid_: spoiler tag (in the interface), secret tag

**Show page**:
The page for one AniList show the library does not have: what the core holds for it, the user's list entry, and Get episodes. Once files for the show land in the library it becomes that series' page.
_Avoid_: external series page, ghost page, AniList page

**Episode out**:
The moment an episode of a show on the Watching list airs, said once by the core from the schedule it keeps, with no polling. The desktop shows it as a notification unless Settings turned that off.
_Avoid_: new episode alert, release notification, reminder

**Stats**:
A year of watching added up from the watch log: hours, episodes and series finished by month, the top genres, studios and seasons. The recap is the one card for the year that saves as an image.
_Avoid_: analytics, insights, wrapped, report

**Planning list**:
The shows on the AniList Planning list, read off the same refresh as the watching list and shown as the second tab of Watching. Each says whether the library has it; Start watching moves one to Watching and plays its first episode.
_Avoid_: plan to watch list, backlog, queue

**Pick**:
A card chosen on a page that acts on several at once: Library and Airing now. Ctrl+click, a held press or Ctrl+Space picks one; Shift picks a run of them. A bar at the foot acts on every pick in one go, and the activity log says it once.
_Avoid_: select, selection, checked, bulk

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

### Downloads

**Release**:
One entry a search endpoint returns: a title, a link (a magnet or a `.torrent` URL) and whatever the feed carries beside them. A count the feed did not carry is unknown, never nought. The shape is anirss's `Item`, kept field for field so a release goes back to anirss unchanged.
_Avoid_: torrent (that is what it becomes once added), result, item, entry

**Release facts**:
What the core reads off a release's title beside anirss's fields: the episode (and season), whether it is a batch, the release group, the resolution, the size in bytes, and whether the library series the search names already has that episode (On disk). A file whose torrent the core has seen (its file origin, recorded from the torrent client after a scan brings files in or when the Downloads page reads the client) is on disk only for that torrent, matched by the magnet's hash or by the torrent's name; a file with no origin is matched by its name. A title the core cannot read gets no episode, never episode 1. The date each release was posted, and a torznab feed's counts, come from the core's own read of the feed the search used, after the search answers.
_Avoid_: metadata, parsed info

**On-demand endpoint**:
An endpoint anirss starts only when a search needs it (a site reached through a search proxy: a podman pod behind Jackett), and keeps up while the sidecar holds its lease. The first search waits for it to come up, up to the endpoint's timeout; the page says so while it waits ("Starting <name>"), then "Searching <name>".
_Avoid_: harness, container, lazy endpoint

**Endpoint**:
One site a search goes to, as anirss configures it: an RSS search built from a site's address with a category and a filter, or any RSS search URL. They sit in an order; a search that finds nothing on one falls through to the next.
_Avoid_: tracker site, indexer, source

**Search state**:
What the Downloads page holds while a search is up: the endpoint, the query and the releases, plus the groups and the best fit the last step returned, and a trail of earlier states for Back. anirss keeps none of it; every step sends the whole state back.
_Avoid_: session, result set, context

**Group**:
A token, or a few tokens with one membership, that split the releases: a release group, a resolution, a codec. Picking one is a step.
_Avoid_: facet, filter, tag

**Step**:
One narrowing of the search state: a group's tokens, a term to keep, a term to drop, or the best fit. A step that would change nothing says why and leaves the state as it is. Back is the page's own, popping the trail.
_Avoid_: refine, filter, apply

**Best fit**:
The step that pins the profile of the best release on offer (its group, resolution, source and audio) by anirss's own ranking.
_Avoid_: auto-pick, recommended

**Subscription**:
An RSS feed and an auto-download rule registered in the torrent client under one name, so every new release that matches lands under that name's folder. anirss registers and removes them; the core only asks.
_Avoid_: rule, feed, watch

**Episode watch**:
What the core adds to a subscription whose folder is a library show (#215): a one-shot timer for the episode that airs next, and for one that aired in the last three days and is not here. A little after it airs the core asks the torrent client to read the show's feed now; it looks again six hours and a day on, reading only the library and asking the feed again when the episode is still missing; three days on it says once in the activity log that the episode has not turned up, and the subscription goes on looking. The followed shows are kept from the last read of the client's rules, so the timers are set at launch without reaching the client, and a feed is only ever asked through a client already running.
_Avoid_: poll, auto-download, follow job

**Wire query**:
The query an endpoint itself was last asked. On a generic RSS endpoint (a site reached through a search proxy) anirss narrows a search on its own side, so the page's query can hold words the endpoint never saw, and a feed built from them finds nothing. Subscribing hands the core the wire query too: the subscription's feed reads that, and the narrowing words go into the rule's own filters (its "keeps" and "drops"). A feed found empty later can be repaired the same way: its search shortened from the end until the endpoint answers, the dropped words kept in the rule.
_Avoid_: base query, raw query

**Complete subscription**:
A subscription whose show has finished airing (or was cancelled) with every episode from 1 to the total on disk, counted by distinct episode number so a v2 beside its v1 counts once. Nothing more will come, so the subscription has nothing left to do: the rail counts these in red and the page offers to unsubscribe from all of them at once. A film is complete once its file is there. An estimated or unknown total is never complete.
_Avoid_: finished, done, stale

**Unsubscribe**:
Taking a subscription away: its feed and its download rule always leave the torrent client; ticked, its torrents go too, and ticked again its files go to the trash (which takes the torrents along). Three answers, never more: the rule; the rule and the torrents; the rule, the torrents and the files. Files are never deleted for good, only moved to the trash on their own drive.
_Avoid_: remove RSS, delete subscription

**Downloaded show**:
Every torrent the client holds for one folder, as one entry: the series that folder fills when the library has it, else the folder by name. Its state is what the client is doing with the torrents: downloading, seeding, done, stopped, or files missing (the torrents point at a folder that is not there). Removing it takes the torrents, and ticked, the files to the trash; a subscription it has stays.
_Avoid_: group, torrent list

**Show link**:
How a subscription or a torrent finds its library series: by the save folder's exact path, else its last component, else that component with case and punctuation dropped. A client can name the library's folder under another mount, and anirss can spell a rule's folder a little differently.
_Avoid_: match (that is the metadata match), mapping

**Transfer**:
A torrent as the torrent client reports it: its state, progress and speed. The one thing in the app that is polled, and only while the Downloads page is up: every two seconds while one is moving, every fifteen while none is.
_Avoid_: download, job, task

**Downloads choice**:
What searches, follows and downloads on the desktop, under Settings, Downloads: Anibeam itself (the core's own port of anirss, which searches the core's sources and writes the torrent client's rules directly) or the search helper (anirss, run as the sidecar or reached on a server). Anibeam itself is the default (#552): fresh settings pick it, and so do settings stored before the choice existed or naming one this build does not know. Settings that name the search helper keep it. Sources written only into anirss's own config.toml are not read by Anibeam itself; they come in through an endpoint repo or by hand.
_Avoid_: backend, mode. The code calls it the downloads engine (`DownloadsEngine`), native or anirss.

**Sidecar**:
anirss run by the core: `anirss --serve` on a port and token the core chose, started the first time something asks for it and stopped with the core. The other way in is a server somebody else runs, reached by address and token.
_Avoid_: daemon, service, backend, child

**Torrent client**:
The program that holds the torrents and the subscriptions: qBittorrent today. anirss drives it for anything that creates a torrent or a rule; the core reaches it directly for the transfers and for stopping, starting and removing one.
_Avoid_: downloader, qBittorrent (as the general word), client

### Frame

**Frame**:
The parts of the window every page shares: the rail, the page area and the status strip. The player is the one page that takes the whole window.
_Avoid_: chrome, layout, main window

**Rail**:
The column at the window's left edge that lists the pages (Home and Library large, then Feed, Watching, Metadata, Downloads and Settings as glyphs) and carries the title language switch and the version.
_Avoid_: sidebar, nav, tab bar

**Status strip**:
The one-line bar at the foot of every page but the player: the latest activity line, a running job's phase and the unseen errors. Clicking it opens the activity log.
_Avoid_: status bar, footer, toast

**Activity log**:
The event stream at Info and above, shown in a drawer that rises from the status strip. The core writes every line; the shell filters, groups and shows them.
_Avoid_: console, log panel, notification centre

**Unseen errors**:
Error events that arrived since the activity log was last open, counted on the status strip. Opening the log clears the count, and it starts at zero on every launch.
_Avoid_: badge, unread, error count

**Inline confirm**:
The guard on a destructive action: the row's controls give way to a line naming the consequence, a red confirm button and Keep. There is no modal dialog.
_Avoid_: confirm dialog, modal, alert

**Patch notes**:
An app's `CHANGELOG.md`: every version newest first, each a few one-line changes under New, Changed, Fixed or Removed. The desktop and the phone keep their own, since each has its own version, and each build carries its own copy. A pull request adds its lines as a **change file** in the app's `changes/`, which shows under the newest version until the release folds it into the file.
_Avoid_: release notes (the GitHub page), changelog entry, commit log

**First run**:
The walkthrough a fresh desktop install opens with instead of an empty library: what Anibeam is, the anime folder (or a new Anime folder in the videos folder), the tracker, how Downloads works, and what is left for later. Every step can be skipped. It shows once; an install that was already in use never sees it, and Settings, About runs it again.
_Avoid_: onboarding wizard, tour, setup assistant

**What's new**:
The sheet that shows the patch notes. It opens by itself once after an update, with a line at the version the install came from; the rail's version, Settings and More open it again. Closing it marks the notes read.
_Avoid_: changelog dialog, update popup, splash

### Devices

**Tailnet host**:
The core serving its library, downloads and episodes to the person's other devices on Tailscale while Share over Tailscale is on.
_Avoid_: server, sync server

**Trusted device**:
A device the tailnet host answers. Every device on the tailnet is trusted until the person switches it off in Settings, Phone app; nothing off the tailnet is.
_Avoid_: allowed device, whitelisted device

**Device key**:
The key pair each Anibeam makes once and keeps with its secrets. The tailnet host says its public key in its hello; a phone or another Anibeam signs each request to it with its own.
_Avoid_: token, password, certificate

**Proof**:
The signature a request carries of the device key that sent it, good for that one request to that one host, around the time it was made, once. A request whose proof does not check out is refused wherever it comes from.
_Avoid_: auth header, token

**Paired device**:
A device whose device key the host keeps. Its requests are answered on any network the host listens on, unless the person switched it off on the tailnet.
_Avoid_: linked device, registered device

### Watch together

**Room**:
Up to 10 people watching the same episode at once, each playing their own copy. A room holds the episode's identity, the play state and who is in it, never video or where to get it. It has a 6 character code, and its host moves it to the next episode.
_Avoid_: party, session, lobby, watch party

**Relay**:
The server rooms live on: it hands out codes, passes the play state between members and serves the join page a room link opens. `anibeam-relay` is the native one, for self-hosting and tests; the Cloudflare Worker is the one the app uses by default.
_Avoid_: room server, sync server, backend

**Offset**:
How far one person's file is from the host's, in seconds, so the same moment plays at the same time: local position is the room's position plus the offset. Kept per file, and suggested from the openings when both are known.
_Avoid_: delay, drift, sync offset

### Route

**Parity checklist**:
The list of behaviours the Electron app has that the native shell must reproduce, in build order. It lives on its ticket and the spec lifts it.
_Avoid_: feature list, backlog, roadmap, scope

**Switch line**:
The first part of the parity checklist. When every item in it is green, the launcher entry points at the native binary and Electron stays installed beside it.
_Avoid_: MVP, daily-driver build, phase 2 exit

**Retire line**:
The rest of the parity checklist. When it is green too, Electron is deleted. That happened on 2026-10-19 (#70).
_Avoid_: full parity, phase 3 gate, the remainder

**App id**:
The one name a shell's window, its launcher entry, its icon and its bus name share, so the desktop can pair them. Each shell has its own. The Linux shell's is not Electron's, so the two stood side by side until the retire line.
_Avoid_: window class, WM_CLASS, desktop file id, bundle id

**Install**:
A build of the checkout put on a machine. Every install carries a version unique to the commit it came from, so a build is a version, never just a file.
_Avoid_: deploy, release build, dev build

### On screen

The words above are for the code and the docs. The app says these instead, so a person never needs the glossary:

| Here | On screen |
| --- | --- |
| Metadata page | Show info |
| Unmatched series | Not recognised |
| Missing | Folder not found |
| Forget | Remove |
| Endpoint | source |
| Endpoint repo | source list |
| Sidecar, anirss | search helper |
| Best fit | Best match |
| Rolling download | Keep the next few ready |
| Sequential download | Download in episode order |
| Tracker | your anime lists, or AniList and MyAnimeList by name |
| Managed torrent client | Built-in, Built-in client |
