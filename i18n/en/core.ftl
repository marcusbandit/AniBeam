# What the core says to a person: problems, errors, notices, and the words every shell
# shares. English is the source; every other language's core.ftl translates it.
# All of a language's files load into one bundle, so the terms here are seen from
# desktop.ftl and android.ftl as well.

## Names that stay the same in every language. Messages use them; none is shown alone.

-app-name = Anibeam
-anilist = AniList
-mal = MyAnimeList

## The tailnet host

# A person on the tailnet whom Tailscale gives neither a name nor a login.
tailnet-person-no-name = No name in Tailscale

## Counts

count-episodes =
    { $count ->
        [one] { $count } episode
       *[other] { $count } episodes
    }
count-shows =
    { $count ->
        [one] { $count } show
       *[other] { $count } shows
    }

## Actions every screen uses

action-cancel = Cancel
action-close = Close
action-try-again = Try again
action-remove = Remove
action-delete = Delete
action-open = Open
action-save = Save
action-apply = Apply
action-clear = Clear
action-refresh = Refresh
action-play = Play
action-connect = Connect
action-start = Start
action-back = Back
action-see-all = See all
action-open-series = Open series
action-open-settings = Open settings

## The lists a tracker keeps

status-watching = Watching
status-completed = Completed
status-paused = Paused
status-on-hold = On hold
status-dropped = Dropped
status-planning = Planning
status-rewatching = Rewatching

## Episodes

episode-number = Episode { $number }

## AniList's words for a show: its format, its airing status, how two shows relate

anilist-format-tv = TV
anilist-format-tv-short = TV short
anilist-format-film = Movie
anilist-format-special = Special
anilist-format-ova = OVA
anilist-format-ona = ONA
anilist-format-music = Music
anilist-status-finished = Finished
anilist-status-airing = Airing
anilist-status-not-yet-aired = Not yet aired
anilist-status-cancelled = Canceled
anilist-status-hiatus = On hiatus
anilist-relation-adaptation = Adaptation
anilist-relation-prequel = Prequel
anilist-relation-sequel = Sequel
anilist-relation-parent = Parent story
anilist-relation-side-story = Side story
anilist-relation-character = Character
anilist-relation-summary = Summary
anilist-relation-alternative = Alternative
anilist-relation-spin-off = Spin-off
anilist-relation-other = Other
anilist-relation-source = Source
anilist-relation-compilation = Compilation
anilist-relation-contains = Contains

## The activity log: a problem whose place had no words of its own

event-problem-library = Something went wrong with your library. Developer options have the details.
event-problem-metadata = Something went wrong with show info. Developer options have the details.
event-problem-trackers = Something went wrong with your lists. Developer options have the details.
event-problem-franchise = Something went wrong with related shows. Developer options have the details.
event-problem-playback = Something went wrong with playback. Developer options have the details.
event-problem-store = Something went wrong with saving your data. Developer options have the details.
event-problem-system = Something went wrong with Anibeam. Developer options have the details.
event-problem-downloads = Something went wrong with downloads. Developer options have the details.

## Errors (contract::CoreError)

core-error-not-found-source = Could not find folder { $id }.
core-error-not-found-series = Could not find series { $id }.
core-error-not-found-file = Could not find file { $id }.
core-error-not-found-session = Could not find playback session { $id }.
core-error-not-found-job = Could not find job { $id }.
# $field is the setting's own name; $message is already in the reader's language.
core-error-invalid = Invalid { $field }: { $message }.
# The same, for a $message that already ends in its own full stop, question mark or ellipsis.
core-error-invalid-ended = Invalid { $field }: { $message }
core-error-unavailable = This folder is not available: { $path }
core-error-not-connected = { $tracker } is not connected.
core-error-refused-hidden = Cannot do that: the show is hidden.
core-error-refused-no-match = Cannot do that: the show is not matched yet; match it on Show info.
core-error-refused-not-newer = Cannot do that: your list is already past this episode.
core-error-refused-extra = Cannot do that: this is an extra.
core-error-refused-unmatched = Cannot do that: the file is not matched to an episode.
core-error-refused-on-disk = Cannot do that: the file is only on disk.
# Why a list was not updated, read after "Not tracked:" or a tracker's name, on every shell's player
refusal-hidden = the show is hidden
refusal-no-match = the show is not matched yet; match it on Show info
refusal-not-newer = your list is already past this episode
refusal-extra = this is an extra
refusal-unmatched = the file is not matched to an episode
refusal-on-disk = the file is only on disk
core-error-provider = { $provider } did not respond: { $message }
core-error-io = Could not read or write a file: { $message }
core-error-storage = Could not save your library: { $message }
core-error-keyring = Could not reach the system password store: { $message }
core-error-unsupported = { $what } is not supported yet.
core-error-version = This export is from a newer Anibeam (format { $found }). Update Anibeam to import it.
core-error-internal = Anibeam hit an internal error: { $message }
core-error-service = { $service }: { $message }

# What the core says to a person outside downloads, releases, trackers, metadata, the
# library, franchise, images and transfer: jobs, playback, casting, the host, the phone,
# endpoint repos, updates and the rest.

## releases/src/repos/mod.rs

core-repos-paste-address = paste the address of a source list
core-repos-must-be-https = the address must use https
core-repos-unreachable = could not reach the source list: { $message }
core-repos-not-found = nothing found at that address (a private GitHub repo also returns 404)
core-repos-http-status = the source list returned HTTP { $status }
core-repos-unreadable = could not read { $path }: { $error }
core-repos-no-such-endpoint = no installed source with that ID
core-repos-key-changed = this source list is now signed with a different key than the one you trusted. Remove and re-add it to accept the new key.
core-repos-stale = the source list returned an older index (serial { $serial } after { $kept }); keeping the newer one
core-repos-key-not-shown = the signing key does not match the one shown to you. Add the source list again and compare the fingerprint.
core-repos-repo-added = Added a source list.
core-repos-repo-removed = Removed a source list.
# $key is the field's key in quotes.
core-repos-no-field = { $name } has no field { $key }
core-repos-needs-field = { $name } requires its { $label }
core-repos-host-only = { $name }: { $label } must be a host or host:port only, like 192.0.2.10:9696
core-repos-not-added = that source list has not been added
core-repos-not-offered = { $repo } does not offer { $id }
core-repos-already-installed = { $name } is already installed
core-repos-taken-by-you = a source called { $id } is already installed (added manually); remove it first
core-repos-taken-by-repo = a source called { $id } is already installed (from another source list); remove it first
core-repos-source-added = Added a download source.
core-repos-source-updated = Updated a download source.
core-repos-source-removed = Removed a download source.
core-repos-source-changed = Edited a download source.
core-repos-give-name = enter a name
core-repos-put-query = include {"{"}query{"}"} in the address where the search term goes
core-repos-name-taken = a source called { $id } is already installed
core-repos-only-by-hand = only manually added sources can be edited
# $key is a placeholder's name; it stays in braces in the address.
core-repos-keep-key = keep {"{"}{ $key }{"}"} in the address and update its value with Change { $label }, or remove and re-add the site without it
core-repos-note-kind = anirss cannot search { $kind } sources, so desktop skips it; the phone app can search it
core-repos-note-by-hand = anirss has its own manually configured { $id }; desktop searches that one
core-repos-note-theirs = You manage this anirss instance yourself, so { -app-name } does not edit its config: add { $id } to anirss's config to search it on desktop
core-repos-note-sandbox = Sandbox mode does not write to anirss's config, so desktop will not search { $id } here
core-repos-note-old = anirss { $version } does not support sources from { -app-name }; update to { $min } or newer to search this on desktop

## releases/src/repos/index.rs

core-repos-key-not-ed25519 = the index key is not an ed25519 key
core-repos-key-not-base64 = the index key is not 32 bytes of base64
core-repos-key-invalid = the index key is not a valid ed25519 key
core-repos-too-large = the file is too large to be a source list
core-repos-not-a-repo = this is not a valid { -app-name } source list
core-repos-format-version = this source list uses format version { $found }, but { -app-name } reads version { $reads }. Update { -app-name }.
core-repos-payload-not-base64 = the source list payload is not base64
core-repos-signature-malformed = the source list signature is malformed
core-repos-index-unreadable = could not read the source list index: { $error }
core-repos-signature-mismatch = the source list signature does not match its contents
# $id is the source's id; the rest of these checks name a source by its id first.
core-repos-check-scheme = { $id }: the address must start with https://
core-repos-check-scheme-http = { $id }: the address must start with http:// or https://
# $key is the field's key in quotes.
core-repos-check-auth-key = { $id }: invalid auth key { $key }
core-repos-check-host-torznab = { $id }: only a torznab address may use {"{"}{ $key }{"}"} as its host
core-repos-check-host-placeholder = { $id }: placeholder not allowed in the host

## playback/src/tracks.rs

core-playback-track-number = Track { $number }
# A sidecar subtitle file beside the video; $label is the rest of its name.
core-playback-track-file = { $label } (file)
# A language and its region or script: "Portuguese (Brazil)", "Chinese (Simplified)".
core-playback-lang-region = { $name } ({ $region })
core-playback-lang-english = English
core-playback-lang-japanese = Japanese
core-playback-lang-spanish = Spanish
core-playback-lang-portuguese = Portuguese
core-playback-lang-french = French
core-playback-lang-german = German
core-playback-lang-italian = Italian
core-playback-lang-russian = Russian
core-playback-lang-chinese = Chinese
core-playback-lang-korean = Korean
core-playback-lang-arabic = Arabic
core-playback-lang-hindi = Hindi
core-playback-lang-indonesian = Indonesian
core-playback-lang-malay = Malay
core-playback-lang-thai = Thai
core-playback-lang-vietnamese = Vietnamese
core-playback-lang-polish = Polish
core-playback-lang-turkish = Turkish
core-playback-lang-dutch = Dutch
core-playback-lang-swedish = Swedish
core-playback-lang-norwegian = Norwegian
core-playback-lang-danish = Danish
core-playback-lang-finnish = Finnish
core-playback-lang-hebrew = Hebrew
core-playback-lang-ukrainian = Ukrainian
core-playback-lang-czech = Czech
core-playback-lang-hungarian = Hungarian
core-playback-lang-greek = Greek
core-playback-lang-romanian = Romanian
core-playback-lang-filipino = Filipino
core-playback-region-brazil = Brazil
core-playback-region-portugal = Portugal
core-playback-region-latin-america = Latin America
core-playback-region-spain = Spain
core-playback-region-simplified = Simplified
core-playback-region-traditional = Traditional

## contract/src/enums.rs

# What a job is doing, for a status line while it runs. .stopped is the line when it
# ended in a problem.
core-jobs-scan = Checking your folders
    .stopped = Checking your folders stopped with a problem.
core-jobs-auto-match = Matching shows
    .stopped = Matching shows stopped with a problem.
core-jobs-search = Searching for a show
    .stopped = Searching for a show stopped with a problem.
core-jobs-resolve-link = Reading a link
    .stopped = Reading a link stopped with a problem.
core-jobs-apply-match = Matching a show
    .stopped = Matching a show stopped with a problem.
core-jobs-refresh = Updating a show
    .stopped = Updating a show stopped with a problem.
core-jobs-refresh-all = Updating show info
    .stopped = Updating show info stopped with a problem.
core-jobs-refresh-airing = Checking what is airing
    .stopped = Checking what is airing stopped with a problem.
core-jobs-refresh-imdb = Updating ratings
    .stopped = Updating ratings stopped with a problem.
core-jobs-clear-images = Clearing pictures
    .stopped = Clearing pictures stopped with a problem.
core-jobs-fill-images = Fetching pictures
    .stopped = Fetching pictures stopped with a problem.
core-jobs-connect-tracker = Connecting your list
    .stopped = Connecting your list stopped with a problem.
core-jobs-mark = Marking an episode watched
    .stopped = Marking an episode watched stopped with a problem.
core-jobs-set-progress = Setting your progress
    .stopped = Setting your progress stopped with a problem.
core-jobs-score = Saving your score
    .stopped = Saving your score stopped with a problem.
core-jobs-refresh-progress = Reading your list progress
    .stopped = Reading your list progress stopped with a problem.
core-jobs-refresh-watching = Reading your Watching list
    .stopped = Reading your Watching list stopped with a problem.
core-jobs-refresh-airing-now = Checking this season
    .stopped = Checking this season stopped with a problem.
core-jobs-refresh-recommended = Finding shows for you
    .stopped = Finding shows for you stopped with a problem.
core-jobs-read-show-page = Reading a show
    .stopped = Reading a show stopped with a problem.
core-jobs-on-hold = Moving idle shows to On hold
    .stopped = Moving idle shows to On hold stopped with a problem.
core-jobs-set-status = Changing a list status
    .stopped = Changing a list status stopped with a problem.
core-jobs-move-lists = Moving your list to your tracker
    .stopped = Moving your list to your tracker stopped with a problem.
core-jobs-notes = Saving your notes
    .stopped = Saving your notes stopped with a problem.
core-jobs-crawl = Finding related shows
    .stopped = Finding related shows stopped with a problem.
core-jobs-skip-windows = Finding the opening and ending
    .stopped = Finding the opening and ending stopped with a problem.
core-jobs-export = Saving your library to a file
    .stopped = Saving your library to a file stopped with a problem.
core-jobs-import = Loading your library from a file
    .stopped = Loading your library from a file stopped with a problem.
core-jobs-subscriptions = Checking your subscriptions
    .stopped = Checking your subscriptions stopped with a problem.
core-jobs-connect-downloads = Connecting the download app
    .stopped = Connecting the download app stopped with a problem.
core-jobs-search-releases = Looking for episodes
    .stopped = Looking for episodes stopped with a problem.
core-jobs-refine-releases = Narrowing down episodes
    .stopped = Narrowing down episodes stopped with a problem.
core-jobs-subscribe = Subscribing to a show
    .stopped = Subscribing to a show stopped with a problem.
core-jobs-download = Adding downloads
    .stopped = Adding downloads stopped with a problem.
core-jobs-remove-subscription = Removing a subscription
    .stopped = Removing a subscription stopped with a problem.
core-jobs-control-torrents = Changing your downloads
    .stopped = Changing your downloads stopped with a problem.
core-jobs-test-torrent-client = Testing the download app
    .stopped = Testing the download app stopped with a problem.
core-jobs-remove-show = Removing a show
    .stopped = Removing a show stopped with a problem.
core-jobs-repair-subscription = Repairing a subscription
    .stopped = Repairing a subscription stopped with a problem.
core-jobs-check-endpoints = Checking your download sources
    .stopped = Checking your download sources stopped with a problem.
core-jobs-start-endpoint = Starting a download source
    .stopped = Starting a download source stopped with a problem.
core-jobs-schedule-agent = Asking your coding agent for a schedule
    .stopped = Asking your coding agent for a schedule stopped with a problem.

## metadata/src/airing_now.rs

core-airing-now-now = Airing now
core-airing-now-winter = Winter { $year }
core-airing-now-spring = Spring { $year }
core-airing-now-summer = Summer { $year }
core-airing-now-fall = Fall { $year }
core-airing-now-planned = Added { $title } to your Planning list on { $site }.
core-airing-now-planned-both = Added { $title } to your Planning list on { $site } and { $other }.
# $status is the list the show is on: watching, repeating, planning, completed, paused or dropped.
core-airing-now-already-on =
    { $title } is already on your { $site } list as { $status ->
        [watching] { status-watching }
        [repeating] { status-rewatching }
        [completed] { status-completed }
        [paused] { status-on-hold }
        [dropped] { status-dropped }
       *[planning] { status-planning }
    }.
core-airing-now-plan-failed = Could not add { $title } to your Planning list. Try again later.

## metadata/src/aired.rs

core-aired-episode-out = Episode { $number } of { $title } is out.

## playback/src/cast/mod.rs

core-cast-connecting = Casting to { $device }.
core-cast-ended = Finished casting to { $device }.
core-cast-stopped = Stopped casting to { $device }.
core-cast-failed = Casting to { $device } failed. Check that it is on the same network.
core-cast-not-built = Casting is not part of this build
core-cast-serve-failed = could not serve the episode: { $error }
core-cast-device-gone = That device is no longer on the network
core-cast-not-a-playlist = not a playlist address
core-cast-nothing = Nothing is casting
core-cast-still-starting = The device is still starting
core-cast-no-route = this device has no route to the TV
core-cast-no-control = the TV gave no control address
core-cast-not-handed = the other computer could not send the episode: { $error }
core-cast-empty-playlist = the other computer sent an empty playlist
# $error is why the TV refused the converted stream, $first why it refused the file.
core-cast-refused-both = { $error }; it also rejected the original file ({ $first })
core-cast-chromecast-stopped = the Chromecast stopped
core-cast-hevc = { $error }. This episode uses HEVC, which only newer Chromecasts (with Google TV or Ultra) support

## playback/src/cast/castv2.rs

core-cast-chromecast-unreachable = could not reach the Chromecast: { $error }
core-cast-chromecast-broke = the Chromecast connection dropped: { $error }
core-cast-chromecast-no-player = the Chromecast did not start its media player
# $reason is the Chromecast's own word for it, or core-cast-no-reason.
core-cast-chromecast-refused-player = the Chromecast failed to start its media player ({ $reason })
core-cast-no-reason = no reason
# $kind is the Chromecast's own word for the failure.
core-cast-chromecast-cannot-play = the Chromecast could not play this file ({ $kind })
core-cast-chromecast-error = the Chromecast stopped with an error playing this file

## playback/src/cast/dlna.rs

core-cast-tv-no-answer = the TV did not respond: { $error }
# $action is the DLNA command; $code the TV's own error.
core-cast-tv-refused = the TV rejected { $action }
core-cast-tv-refused-code = the TV rejected { $action } ({ $code })

## playback/src/cast/media.rs

core-cast-no-video = This file has no video.
core-cast-hi10p = This episode uses 10-bit H.264 (Hi10P), which TVs and Chromecasts cannot decode. Casting it requires transcoding, which is not supported yet.
core-cast-video-codec = This episode's video ({ $codec }) cannot be cast.
core-cast-audio-codec = This episode's audio ({ $codec }) cannot be cast.

## downloads/src/phone/mod.rs

core-phone-download-started =
    { $count ->
        [one] Started downloading an episode of { $folder }.
       *[other] Started downloading { $count } episodes of { $folder }.
    }
core-phone-download-partial = Not every release of { $folder } could be added: { $error }
core-phone-subscribed = Subscribed to { $name }.
core-phone-rule-stopped = { $name } is not checking for new episodes: its source was removed or needs its key again.
core-phone-new-episode = A new episode of { $name } is downloading.
# $name is the source's name in quotes.
core-phone-name-taken = { $name }: another source has that name
core-phone-nothing-to-download = nothing to download
# $used and $cap are in GB, with one decimal.
core-phone-over-cap = this would exceed { -app-name }'s storage cap ({ $used } of { $cap } GB used or queued); delete episodes or raise the cap
core-phone-empty-search = the search is empty
core-phone-no-results = no results for { $query }
# Ends "... is not supported yet." (core-error-unsupported).
core-phone-own-engine = a torrent engine of its own

## downloads/src/phone/check.rs

# The connection check: each line is a step for one host ($label), and what came of it.
core-phone-check-connected = connected
core-phone-check-refused = refused: { $error }
core-phone-check-dropped = no response (dropped)
core-phone-check-finding = { $label }: resolving address
core-phone-check-failed = failed: { $error }
core-phone-check-no-answer = no response
core-phone-check-private = { $label }: resolving privately
core-phone-check-skipped-address = skipped, IP address
core-phone-check-skipped-home = skipped, local network name
core-phone-check-no-record = no public record, falling back to network DNS
core-phone-check-no-resolver = no DNS resolver responded
core-phone-check-reaching = { $label }: connecting to { $ip }
core-phone-check-http = HTTP { $status }, { $bytes } bytes
core-phone-check-setup = setup

## playback/src/session.rs

core-playback-not-saved = Your progress on { $folder } was not saved. Match the show first.
# $done is a whole percentage.
core-playback-still-downloading = episode { $number } is still downloading, { $done }% done

## playback/src/skip.rs

core-playback-bad-duration = duration must be a positive number of seconds

## core/src/agent.rs

# A coding agent looking up a show's schedule; these lines say what it is doing.
core-agent-no-answer = the agent returned no answer
core-agent-searching = Searching the web for { $query }
core-agent-searching-schedule = Searching the web for the schedule
core-agent-reading = Reading { $url }
core-agent-reading-page = Reading a page
# $tool is the agent's own name for a tool.
core-agent-using = Using { $tool }
# Ends "... is not supported yet." (core-error-unsupported).
core-agent-none = a coding agent on this computer
# $name is the coding agent's name.
core-agent-ended = { $name } finished without an answer
core-agent-too-long = { $name } took too long

## core/src/core.rs

# A guest device on another computer's library asks for something that is the owner's.
core-core-guest-trackers = the tracker accounts on that computer belong to its owner, so this device cannot update them
core-core-guest-not-interested = Not interested on that computer belongs to its owner, so this device cannot change it
core-core-guest-recommended = recommendations on that computer are learned from its owner's list
core-core-nothing-picked = nothing selected

## foundation/src/prefs.rs

core-prefs-not-a-provider = { $provider } is not a match provider
core-prefs-provider-count = { $provider } appears { $count } times; each provider can appear only once
# A setting out of its bounds: $value is outside $low to $high.
core-prefs-outside = { $value } must be between { $low } and { $high }
core-prefs-empty = empty
core-prefs-room-name = name must be 1 to { $max } characters with no control characters
core-prefs-room-relay = relay address must start with https:// or http://

## metadata/src/recommended/mod.rs

core-recommended-ranking = Ranking shows for you

## metadata/src/recommended/sync.rs

core-recommended-finding-people = Finding people with your taste
core-recommended-reading-lists = Reading their lists
core-recommended-reading-yours = Reading your shows
core-recommended-reading-suggestions = Reading shows to suggest

## metadata/src/recommended/page.rs

core-recommended-no-anime = { -anilist } has no anime { $id }

## host/src/lib.rs

# The tailnet host's status line in Settings.
core-host-off = Off
core-host-starting = Starting
# $addresses is a list of address:port, joined with commas.
core-host-serving = Serving on { $addresses }
core-host-no-tailscale = Tailscale is not connected
core-host-cannot-listen = Could not listen
core-host-port-taken = Port { $port } is taken
core-host-cannot-listen-on = Could not listen on { $address }: { $error }

## host/src/addr.rs

core-host-not-an-address = { $text } is not a valid address and port

## host/src/keys.rs

# $key is the text given, in quotes.
core-host-not-a-key = { $key } is not a device key

## net/src/limiter.rs

core-net-timed-out = timed out after { $ms }ms
core-net-rate-limited = { $provider } rate limited

## net/src/lib.rs

core-net-waiting = { $host }: timed out recently; { -app-name } will wait { $seconds } s before trying again

## net/src/anilist.rs

core-net-no-data = no data

## core/src/update/install.rs

core-update-no-download = { -app-name } { $version } has no download for this computer
core-update-cannot-make = Could not create { $path }: { $error }
core-update-cannot-mark = Could not make the new AppImage executable: { $error }
core-update-cannot-replace = Could not replace { $path }: { $error }
core-update-not-here = This install cannot update itself here
core-update-cannot-start = Could not start the download: { $error }
core-update-did-not-start = The download did not start: { $error }
core-update-cannot-write = Could not write { $path }: { $error }
core-update-stopped = The download stopped: { $error }
core-update-cancelled = Canceled
# $got and $size are in bytes.
core-update-ended-early = The download ended early ({ $got } of { $size } bytes)
core-update-checksum = The download does not match GitHub's checksum
core-update-cannot-move = Could not move the download into place: { $error }
core-update-cannot-open-dmg = Could not open the DMG: { $error }
core-update-cannot-read-dmg = Could not read the DMG: { $error }
core-update-no-app = The DMG does not contain an app
core-update-cannot-copy = Could not copy the new app next to this one: { $error }

## metadata/src/calendar.rs

# An episode's event in the calendar file.
core-calendar-season-episode = { $title } - Season { $season } Episode { $number }
core-calendar-episode = { $title } - Episode { $number }

## foundation/src/paths.rs

core-paths-no-home = no home directory

## core/src/first_run.rs

core-first-run-not-absolute = the videos folder must be an absolute path

## playback/src/room/mod.rs

# The service named in a watch together error: "Watch together: That room is full."
core-room-service = Watch together
core-room-not-found = No room has that code. It may have closed.
core-room-full = That room is full.
core-room-proto = That server uses a different Watch together version. Update { -app-name } to join.
core-room-needs-episode = That server requires an episode before starting a room. Start one from the player, or update the server.
# $message is the network's own words.
core-room-unreachable = Could not reach the room server: { $message }
core-room-refused = The room server rejected the request: { $message }
core-room-not-in = You are not in a room.
core-room-no-name = enter the display name others in the room will see
core-room-bad-code = room codes must be 6 letters or digits
core-room-created = Started a Watch together room. Code: { $code }.
core-room-joined = Joined Watch together room { $code }.
core-room-left = Left the Watch together room.
core-room-member-joined = { $name } joined the room.
core-room-member-left = { $name } left the room.
core-room-closed = The Watch together room closed.
core-room-lost = Lost connection to the Watch together room. Reconnecting.
core-room-back = Reconnected to the Watch together room.
core-room-media-asked = Asked the host for the episode.
core-room-media-cannot = The host can't share this episode right now.
core-room-media-sent = Sent { $name } the episode's torrent.
core-room-media-sent-name = Sent { $name } the episode's release name.
core-room-media-nothing = The host has no copy of the episode to share.
core-room-media-no-answer = The host's app did not respond. Try again in a moment.
core-room-media-torrent = Downloading the episode from the host's torrent.
core-room-media-searching = Searching your sources for the host's release.
core-room-media-not-found = None of your sources have the host's release.
core-room-media-no-sources = All sources are turned off. Enable one to search.
core-room-media-no-start = Could not start the download: { $message }
core-room-media-client = the torrent client rejected it
core-room-media-lost = The torrent client lost the episode's download.
core-room-media-stalled = The download has been stalled for ten minutes. Try again later.
core-room-media-not-in-library = The episode finished downloading, but it did not appear in your library. Check that downloads save into a library folder.
core-room-media-here = The episode is in your library now.
core-room-media-rest-missing = Could not get the rest of the show. Only this episode is downloading.

## playback/src/presence/mod.rs

# Discord shows this in place of an episode line while an extra plays.
core-presence-extra = Extra
core-presence-room = Watching together
core-presence-in-room = In a room
core-presence-no-room = That Discord invite has no room yet. Ask them to start one.

# The core's words for the library group: trackers, metadata, library, franchise,
# images and transfer (#798).

## trackers/src/accounts.rs

# $tracker is the tracker's short name: AniList or MAL.
core-trackers-no-client-id = No client ID set for { $tracker }.
core-trackers-mal-reconnect = Reconnect MyAnimeList in Settings.
core-trackers-mal-wants-secret = MyAnimeList requires this app's client secret. Add it under Accounts in Anibeam's settings, or register the app with App Type "other".
# $error is the parser's own message.
core-trackers-token-reply-unreadable = could not read token response: { $error }
core-trackers-token-reply-no-token = token response did not include an access token

## trackers/src/lib.rs

core-trackers-no-token = no access token stored, reconnect in Settings

## trackers/src/cache.rs

core-trackers-no-user-id = account response did not include a user ID

## trackers/src/move_up.rs

core-trackers-move-none-picked = no series selected to move
# $site is the tracker's full name: AniList or MyAnimeList.
core-trackers-moved-up =
    { $count ->
        [one] Moved { $count } show from this computer to { $site }.
       *[other] Moved { $count } shows from this computer to { $site }.
    }
core-trackers-moved-up-some-refused =
    Moved { $moved ->
        [one] { $moved } show
       *[other] { $moved } shows
    } to { $site }; could not move { $failed ->
        [one] { $failed } show
       *[other] { $failed } shows
    }.
# $show is the show's folder name; $error is the tracker's answer, cleaned up.
core-trackers-move-refused = { $site } could not update { $show }: { $error }

## trackers/src/oauth.rs

core-trackers-oauth-timed-out = Authorization timed out, try again.
core-trackers-oauth-state-mismatch = state mismatch, possible CSRF, aborted.
core-trackers-oauth-no-code = no code in callback
core-trackers-oauth-no-token = no access_token in callback
core-trackers-oauth-waiting = Waiting for you to sign in to { $site } in your browser.
# $username is the account's name on the tracker.
core-trackers-oauth-connected = Connected { $site } as { $username }.
core-trackers-oauth-no-name = profile response did not include a username
# Shown before a colon and the reason.
core-trackers-oauth-could-not-start = Could not start sign-in
# $error is the system's own message.
core-trackers-oauth-port-failed = port { $port } on this computer could not be opened: { $error }
core-trackers-oauth-port-busy = port { $port } on this computer is in use by another program. Wait a moment and try again, or close the program using it if it stays busy.
# $error is the code the tracker sent back.
core-trackers-oauth-provider-error = { $site } returned an error: { $error }
# The browser tab the sign-in leaves. $app is Anibeam's name, in the brand colour in the
# page's body; keep it as a variable.
core-trackers-page-connected = { $site } is connected to { $app }
core-trackers-page-close-tab = You can close this tab.
core-trackers-page-failed-title = { $app }, sign-in failed
core-trackers-page-failed = Sign-in did not finish
core-trackers-page-go-back = Go back to { $app } and try again.
core-trackers-page-finishing-title = { $app }, still finishing
core-trackers-page-finishing = Still finishing
core-trackers-page-finishing-body = { $site } responded and Anibeam is finishing sign-in. Return to { $app } to continue.
core-trackers-page-cancelled-title = { $app }, sign-in canceled
core-trackers-page-cancelled = Sign-in was canceled
core-trackers-page-cancelled-body = You can close this tab. Start again from { $app } if you still want to sign in.

## trackers/src/on_hold.rs

# A progress label while the pass runs; $tracker is AniList or MAL.
core-trackers-on-hold-moving = Moving to On hold on { $tracker }
core-trackers-on-hold-moved =
    { $count ->
        [one] Moved { $count } show to On hold on { $site }.
       *[other] Moved { $count } shows to On hold on { $site }.
    }
# $second is how many moved on the second tracker, $secondSite.
core-trackers-on-hold-moved-two =
    { $count ->
        [one] Moved { $count } show to On hold on { $site } and { $second } on { $secondSite }.
       *[other] Moved { $count } shows to On hold on { $site } and { $second } on { $secondSite }.
    }
core-trackers-on-hold-failed =
    { $failed ->
        [one] { $failed } show could not move to On hold.
       *[other] { $failed } shows could not move to On hold.
    }
core-trackers-on-hold-moved-failed =
    { $count ->
        [one] Moved { $count } show to On hold on { $site }.
       *[other] Moved { $count } shows to On hold on { $site }.
    } { $failed ->
        [one] { $failed } show could not move to On hold.
       *[other] { $failed } shows could not move to On hold.
    }
core-trackers-on-hold-moved-two-failed =
    { $count ->
        [one] Moved { $count } show to On hold on { $site } and { $second } on { $secondSite }.
       *[other] Moved { $count } shows to On hold on { $site } and { $second } on { $secondSite }.
    } { $failed ->
        [one] { $failed } show could not move to On hold.
       *[other] { $failed } shows could not move to On hold.
    }
# A show's name when its row is gone.
core-trackers-series-fallback = series { $id }
# $name is the show's folder name.
core-trackers-back-on-watching = { $name } is back on Watching on { $site }.
core-trackers-back-on-watching-two = { $name } is back on Watching on { $site } and { $secondSite }.
core-trackers-back-on-watching-failed = { $name } could not go back on Watching. Try again later.

## trackers/src/writes.rs

# A tracker's failure in words a person can act on. $tracker is AniList or MAL.
core-trackers-rate-limited = { $tracker } rate limited, try again in a minute.
core-trackers-auth-expired = { $tracker } auth expired, reconnect in Settings.
core-trackers-entry-not-found = { $tracker } entry not found.
# $status is the HTTP status code.
core-trackers-server-error = { $tracker } server error ({ $status }), try again later.
core-trackers-error-see-log = { $tracker } error, see activity log.
# $episode is the number asked for, as it was typed.
core-trackers-episode-not-whole = { $episode } must be a whole number greater than 0
core-trackers-score-out-of-range = { $score } must be between 0 and 10
core-trackers-rewatch-not-settable = rewatching cannot be set manually
core-trackers-notes-too-long = notes cannot exceed { $max } characters
core-trackers-nothing-picked = nothing selected
core-trackers-notes-not-on-list = Not on your { $tracker } list yet. Add it to a list first.
# Where a write landed, picked by $sites: anilist, mal or both. Used inside the lines below.
core-trackers-sites =
    { $sites ->
        [both] { -anilist } and { -mal }
        [anilist] { -anilist }
       *[mal] { -mal }
    }
# A list status as the status pill names it, picked by $status.
core-trackers-status =
    { $status ->
        [watching] { status-watching }
        [planning] { status-planning }
        [completed] { status-completed }
        [paused] { status-on-hold }
        [dropped] { status-dropped }
       *[repeating] Repeating
    }
# $show is the show's folder name.
core-trackers-marked = Marked episode { $episode } of { $show } on { core-trackers-sites }.
core-trackers-progress-set = Set your progress on { $show } to { $progress } on { core-trackers-sites }.
core-trackers-scored = Scored { $show } { $score } on { core-trackers-sites }.
core-trackers-score-cleared = Cleared your score for { $show } on { core-trackers-sites }.
core-trackers-status-set = Set { $show } to { core-trackers-status } on { core-trackers-sites }.
core-trackers-notes-saved = Saved your notes on { $show } to { core-trackers-sites }.
core-trackers-notes-cleared = Cleared your notes on { $show } on { core-trackers-sites }.
# $site is AniList or MyAnimeList.
core-trackers-update-failed = Could not update { $site }. Try again later.
# One write to many shows: what landed, what failed and what was left out, in one line.
# $done shows were written and $failed were not; $named is yes when the one show
# written goes by its $name. $action is score, clear or status; $hasReason is yes when
# $reason holds the first tracker's answer.
core-trackers-written =
    { $done ->
        [0] { core-trackers-written-failed }
       *[other] { core-trackers-written-done }{ $failed ->
            [0] {""}
           *[other] {" "}{ core-trackers-written-failed-more }
        }
    }{ core-trackers-left-out }
core-trackers-written-who =
    { $named ->
        [yes] { $name }
       *[no] { $done ->
            [one] { $done } show
           *[other] { $done } shows
        }
    }
core-trackers-written-done =
    { $action ->
        [clear] Cleared your score for { core-trackers-written-who } on { core-trackers-sites }.
        [status] Set { core-trackers-written-who } to { core-trackers-status } on { core-trackers-sites }.
       *[score] Scored { core-trackers-written-who } { $score } on { core-trackers-sites }.
    }
core-trackers-written-failed =
    { $hasReason ->
        [yes] { $failed ->
            [one] { $failed } show could not be updated: { $reason }.
           *[other] { $failed } shows could not be updated: { $reason }.
        }
       *[no] { $failed ->
            [one] { $failed } show could not be updated.
           *[other] { $failed } shows could not be updated.
        }
    }
core-trackers-written-failed-more =
    { $hasReason ->
        [yes] { $failed } could not be updated: { $reason }.
       *[no] { $failed } could not be updated.
    }
# The shows a write to many left out, one sentence per reason, each after a space.
# Keep each [0] first: it leaves the sentence out.
core-trackers-left-out =
    { $noMatch ->
        [0] {""}
        [one] {" "}{ $noMatch } show has no tracker match.
       *[other] {" "}{ $noMatch } shows have no tracker match.
    }{ $hidden ->
        [0] {""}
        [one] {" "}{ $hidden } show is hidden, so its sync is off.
       *[other] {" "}{ $hidden } shows are hidden, so their sync is off.
    }{ $notConnected ->
        [0] {""}
        [one] {" "}{ $notConnected } show is only on a tracker that is not connected.
       *[other] {" "}{ $notConnected } shows are only on a tracker that is not connected.
    }{ $kept ->
        [0] {""}
        [one] {" "}{ $kept } show is kept on this computer, with no tracker to send to.
       *[other] {" "}{ $kept } shows are kept on this computer, with no tracker to send to.
    }

## metadata/src/automatch.rs

# $show is the show's folder name; $title is the title AniList or TVmaze gives it.
core-metadata-matched-to = Matched { $show } to { $title }.
core-metadata-auto-matched =
    { $count ->
        [one] Matched { $count } show.
       *[other] Matched { $count } shows.
    }
core-metadata-auto-unmatched =
    { $count ->
        [one] { $count } show still needs a match.
       *[other] { $count } shows still need a match.
    }
core-metadata-auto-matched-some =
    Matched { $matched ->
        [one] { $matched } show
       *[other] { $matched } shows
    }; { $unmatched } still { $unmatched ->
        [one] needs
       *[other] need
    } a match.
# Find a match's one line: the sentences below that have something to say, in order,
# each after the first set off by a space. $first names the sentence that leads.
core-metadata-find =
    { $matched ->
        [0] {""}
       *[other] { core-metadata-find-matched }
    }{ $unmatched ->
        [0] {""}
       *[other] { $first ->
            [unmatched] {""}
           *[other] {" "}
        }{ core-metadata-find-unmatched }
    }{ $failed ->
        [0] {""}
       *[other] { $first ->
            [failed] {""}
           *[other] {" "}
        }{ core-metadata-find-failed }
    }{ $had ->
        [0] {""}
       *[other] { $first ->
            [had] {""}
           *[other] {" "}
        }{ core-metadata-find-had }
    }{ $busy ->
        [0] {""}
       *[other] { $first ->
            [busy] {""}
           *[other] {" "}
        }{ core-metadata-find-busy }
    }
# $hitNamed is yes when the one show matched goes by its folder name, $hitFolder, and
# the title it matched, $hitTitle.
core-metadata-find-matched =
    { $hitNamed ->
        [yes] Matched { $hitFolder } to { $hitTitle }.
       *[no] { $matched ->
            [one] Matched { $matched } show.
           *[other] Matched { $matched } shows.
        }
    }
core-metadata-find-unmatched =
    { $missNamed ->
        [yes] No match found for { $missName }.
       *[no] { $unmatched ->
            [one] No match found for { $unmatched } show.
           *[other] No match found for { $unmatched } shows.
        }
    }
core-metadata-find-failed =
    { $failed ->
        [one] { $failed } show could not be searched; try again later.
       *[other] { $failed } shows could not be searched; try again later.
    }
core-metadata-find-had =
    { $had ->
        [one] { $had } show had a match already.
       *[other] { $had } shows had a match already.
    }
core-metadata-find-busy =
    { $busy ->
        [one] { $busy } show is being matched already.
       *[other] { $busy } shows are being matched already.
    }

## metadata/src/apply.rs

core-metadata-link-unreadable = Couldn't read that link. Paste an AniList, MyAnimeList, TVmaze or IMDb page URL.
core-metadata-no-mal-entry = AniList has no entry for that MyAnimeList id.
core-metadata-no-imdb-show = TVmaze has no show for that IMDb title.
# Shown before "is not supported yet". $provider is the provider's code: anilist, mal, tmdb.
core-metadata-search-on = search on { $provider }
core-metadata-query-too-short = Type at least two characters to search.
core-metadata-not-a-link = not a link
core-metadata-refresh-unsupported = refresh of a MAL-only or TMDB match
# $show is the show's folder name.
core-metadata-matched = Matched { $show }.
core-metadata-updated = Updated the info for { $show }.
core-metadata-update-failed = Could not update the info for { $show }. Try again later.
# Progress labels while the whole library is updated, or shows only seen in passing
# are filled in.
core-metadata-progress-refresh = refresh
core-metadata-progress-backfill = backfill
core-metadata-updated-many =
    { $count ->
        [one] Updated the info for { $count } show.
       *[other] Updated the info for { $count } shows.
    }
core-metadata-update-failed-many =
    { $count ->
        [one] Could not update the info for { $count } show. Try again later.
       *[other] Could not update the info for { $count } shows. Try again later.
    }
core-metadata-updated-some =
    { $count ->
        [one] Updated the info for { $count } show; { $failed } could not be updated.
       *[other] Updated the info for { $count } shows; { $failed } could not be updated.
    }

## metadata/src/fetch.rs

# $id is the provider's own number for the show.
core-metadata-no-such-show = TVmaze has no show { $id }
core-metadata-no-such-media = AniList has no media { $id }

## metadata/src/imdb.rs

# Progress labels while the ratings update.
core-metadata-progress-imdb-links = Wikidata's IMDb links
core-metadata-progress-imdb-ratings = IMDb ratings

## metadata/src/franchise/layout.rs

# The words under a related show's title when the two share characters.
core-franchise-relation-character = Shared characters

## net/src/images.rs

# A progress label while covers and banners download.
core-images-progress-fetching = fetching images

## host/src/transfer/export.rs

# $folder is the name of the folder the file went to.
core-transfer-exported =
    { $count ->
        [one] Saved { $count } show to { $folder }.
       *[other] Saved { $count } shows to { $folder }.
    }

## host/src/transfer/format.rs

# $error is the JSON parser's own message.
core-transfer-not-json = not JSON: { $error }
# $format is the file format's name, anibeam-export.
core-transfer-wrong-format = not an { $format } document

## host/src/transfer/import.rs

core-transfer-imported = Loaded your library from the file.

## library/src/delete.rs

# $id is the file's number in the library.
core-library-no-such-file = no file { $id } in the library
core-library-file-elsewhere-not-removed = file { $id } belongs to another series, so nothing was removed
core-library-file-outside = { $path } is not a file inside a library folder, so nothing was removed
# $place is deleted where the system has no trash, trash otherwise. $title is the show's
# folder name.
core-library-episodes-removed =
    { $count ->
        [one] { $count } episode
       *[other] { $count } episodes
    } of { $title } { $place ->
        [deleted] deleted
       *[trash] moved to the trash
    }.
core-library-finished-removed =
    { $count ->
        [one] { $count } finished show
       *[other] { $count } finished shows
    } { $place ->
        [deleted] deleted
       *[trash] moved to the trash
    }.
# $series is how many shows the episodes came from.
core-library-watched-removed =
    { $count ->
        [one] { $count } watched episode
       *[other] { $count } watched episodes
    } of { $series } series { $place ->
        [deleted] deleted
       *[trash] moved to the trash
    }.

## library/src/detail.rs

# A fix the series page offers. $read is episode, with its $number, or label, with the
# name the file was read by in $label. $what is film or episode.
core-library-fix-one-file =
    The one file here reads as { $read ->
        [episode] episode { $number }
       *[label] { $label }
    }. It looks like { $what ->
        [film] the movie
       *[episode] its one episode
    }.
# $numbers is a run of episode numbers, as core-library-runs-span or -list writes it;
# $episodes is how many there are.
core-library-fix-specials =
    { $count ->
        [one] { $count } special
       *[other] { $count } specials
    } look like the missing { $episodes ->
        [one] episode
       *[other] episodes
    } { $numbers }.
core-library-fix-numbered-on = Episodes { $from } look like episodes { $to }, continuing numbering from an earlier season.
# Episode numbers in a row: "1 to 6".
core-library-runs-span = { $first } to { $last }
# Episode numbers apart: "2, 4 and 7". $init is every number but the last, comma-separated.
core-library-runs-list = { $init } and { $last }

## library/src/fixes.rs

core-library-fix-episode-range = episode number must be between 0 and 10000
core-library-fix-season-range = season number must be 200 or less
core-library-fix-extra-range = extra index must be 10000 or less
core-library-file-elsewhere-not-fixed = file { $id } belongs to another series, so nothing was fixed
# $title is the show's title.
core-library-fix-undone =
    { $count ->
        [one] Went back to reading { $count } file of { $title } by name.
       *[other] Went back to reading { $count } files of { $title } by name.
    }
core-library-fixed =
    { $count ->
        [one] Fixed how { $count } file of { $title } is read.
       *[other] Fixed how { $count } files of { $title } are read.
    }

## library/src/reads.rs

# $title is the show's title. The -many lines are for any count but one.
core-library-hid-one = Hid { $title }.
core-library-back-one = { $title } is back.
core-library-hid-many = Hid { $count } shows.
core-library-back-many = { $count } shows are back.
core-library-episode-count-set = { $title } now has { $count } episodes.
core-library-episode-count-cleared = Cleared the episode count of { $title }.

## library/src/schedule.rs

core-library-schedule-cleared = Cleared the schedule notes of { $title }.
core-library-schedule-saved = Saved { $count } schedule notes for { $title }.

## library/src/scan.rs

core-library-source-not-absolute = a source path must be absolute
core-library-source-exists = already a source
core-library-source-nested = nested inside an existing source
# $folder is the last part of the folder's path.
core-library-source-added = Added the folder { $folder } to your library.
core-library-source-removed = Removed the folder { $folder } from your library.
core-library-source-removed-with-shows =
    { $count ->
        [one] Removed the folder { $folder } and its { $count } show from your library.
       *[other] Removed the folder { $folder } and its { $count } shows from your library.
    }
# $name is the show's folder name.
core-library-series-forgotten = Removed { $name } from your library.
core-library-source-back = The folder { $folder } is back.
core-library-source-unavailable = The folder { $folder } is not available. Check that its drive is connected.
core-library-source-unreadable = Could not read the folder { $folder }. Check that its drive is connected.
core-library-source-looks-empty = The folder { $folder } looks empty, so your shows were left as they were.
core-library-source-reconcile-failed = Could not update the shows in { $folder }.
# What a scan found: the parts below that are not nought, in order, the ones after the
# first set off by a comma. $first names the part that leads.
core-library-scan-done =
    Checked your folders: { $added ->
        [0] {""}
       *[other] { $added ->
            [one] { $added } new show
           *[other] { $added } new shows
        }
    }{ $changed ->
        [0] {""}
       *[other] { $first ->
            [changed] {""}
           *[other] {", "}
        }{ $changed ->
            [one] { $changed } show changed
           *[other] { $changed } shows changed
        }
    }{ $removed ->
        [0] {""}
       *[other] { $first ->
            [removed] {""}
           *[other] {", "}
        }{ $removed } missing
    }.

## library/src/report.rs

# Why a coding agent wrote no triage for a quick fix report. $span is
# core-library-span-minutes or -seconds.
core-library-agent-no-answer = no answer within { $span }
core-library-agent-stopped-code = it stopped with code { $code }
core-library-agent-stopped = it stopped
core-library-agent-answered-nothing = it returned no output
core-library-span-minutes =
    { $count ->
        [one] { $count } minute
       *[other] { $count } minutes
    }
core-library-span-seconds =
    { $count ->
        [one] { $count } second
       *[other] { $count } seconds
    }
core-library-report-no-fix = none of these files has a fix to report
# $agent is the agent's name, such as Claude Code; $why is one of the reasons above, or
# the agent's own last words.
core-library-agent-no-triage = { $agent } wrote no triage: { $why }
# Progress labels while a file is loaded, one per part of the library.
core-transfer-progress-sources = sources
core-transfer-progress-series = series
core-transfer-progress-history = history
core-transfer-progress-accounts = accounts
core-transfer-progress-preferences = preferences
core-transfer-progress-done = done

## downloads/src/anirss.rs

# $message is what the connection said
core-downloads-anirss-no-answer = anirss did not respond: { $message }
# $status is the HTTP status code, $text the body anirss sent
core-downloads-anirss-answered = anirss returned HTTP { $status }: { $text }
core-downloads-no-free-port = no free port: { $error }
# $status is how the process ended, as the system words it
core-downloads-anirss-exited = anirss exited ({ $status })
# $said is the last lines anirss printed
core-downloads-anirss-exited-said = anirss exited ({ $status }): { $said }
core-downloads-anirss-slow-start = anirss did not start within { $seconds } seconds
core-downloads-anirss-slow-start-said = anirss did not start within { $seconds } seconds: { $said }
# $command is a command to type, kept as it is
core-downloads-anirss-install-mac = Install anirss { $version } or newer with: { $command }
core-downloads-anirss-install-linux = Install anirss { $version } or newer: the anirss package on the AUR, or { $command }
# $update is core-downloads-anirss-update-mac or -linux
core-downloads-anirss-too-old = anirss { $version } is too old for Anibeam, which needs { $minimum } or newer. { $update }.
core-downloads-anirss-lacks-feature = anirss { $version } cannot keep each torrent client's login separate, which Anibeam needs to switch clients; anirss { $minimum } and newer can. { $update }.
core-downloads-anirss-update-mac = Update it with: { $command }
core-downloads-anirss-update-linux = Update the anirss package, or run: { $command }

## downloads/src/contents.rs

core-downloads-torrent-file-no-answer = could not download the torrent file: { $message }
core-downloads-torrent-file-answered = torrent file returned HTTP { $status }

## downloads/src/feed.rs

core-downloads-feed-answered = feed returned HTTP { $status }
core-downloads-not-a-feed = not a feed: { $error }

## downloads/src/jackett.rs

# $program is the command that was not found
core-downloads-jackett-missing = Jackett is not installed: { $program } was not found. Install it, or change the command in Settings.
# $name is the endpoint, $why what went wrong
core-downloads-endpoint-start-failed = { $name } could not start: { $why }
core-downloads-endpoint-start-timeout = { $name } started but did not respond within { $seconds } seconds.
core-downloads-command-exited = the command exited with { $status }

## downloads/src/lib.rs

core-downloads-anirss-stopped = anirss stopped
# $install is core-downloads-anirss-install-mac or -linux
core-downloads-anirss-missing = { $install }. You can also point Settings to a running server.
# $command is the anirss command set in Settings
core-downloads-anirss-command-missing = { $command } was not found. { $install }. You can also point Settings to a running server.
core-downloads-anirss-start-failed = { $command } could not start: { $error }
# $tooOld is core-downloads-anirss-too-old; $message is what anirss said
core-downloads-anirss-too-old-said = { $tooOld } It said: { $message }
core-downloads-no-torrent-client = No qBittorrent responded at { $address }, and Anibeam has no built-in client available. Install qBittorrent and enable its Web UI, or install qbittorrent-nox for Anibeam to run automatically
core-downloads-own-client-gone = Anibeam's built-in qBittorrent is missing (qbittorrent-nox is not installed). Reinstall Anibeam, or point Settings to your own qBittorrent
core-downloads-no-port = no port in { $address }
core-downloads-own-client-settings = cannot write qBittorrent's settings: { $error }
core-downloads-own-client-start-failed = { $command } could not start: { $error }
core-downloads-own-client-slow-start = Anibeam's qBittorrent did not start
# $said is the last lines of its log
core-downloads-own-client-slow-start-said = Anibeam's qBittorrent did not start: { $said }
core-downloads-not-ready = Downloads are not ready. Check Downloads in Settings.
core-downloads-ready = Downloads are ready.
# $count is how many sources anirss's own config names
core-downloads-helper-sources =
    { $count ->
        [one] Downloads now search directly in Anibeam, which does not read the { $count } source in the search helper's config. Add it in Settings > Downloads, or switch back to the search helper there.
       *[other] Downloads now search directly in Anibeam, which does not read the { $count } sources in the search helper's config. Add them in Settings > Downloads, or switch back to the search helper there.
    }
core-downloads-client-connected = The download app is connected.
core-downloads-client-not-connected = The download app is not connected. Check its address in Settings.
core-downloads-not-on-demand = { $name } is not an on-demand source
core-downloads-source-running = The download source is running.
core-downloads-source-not-running = { $name } is not running. Enable it in your sources and confirm to start it.
core-downloads-empty-query = type something to search for
# What a search's progress line names when it was given no source
core-downloads-first-endpoint = the first source
# A search's progress line. The desktop's search card compares the line with these, so
# keep the name in core-downloads-search-starting-endpoint a whole placeable.
core-downloads-search-starting = Starting the search
core-downloads-search-starting-endpoint = Starting { $endpoint }
core-downloads-search-searching-endpoint = Searching { $endpoint }
core-downloads-search-reading = Reading the results
# $name is the subscription's name
core-downloads-subscribed = Subscribed to { $name }.
core-downloads-client-no-rule = qBittorrent has no rule named { $name }
# A progress line while a subscription's feed is narrowed; $search is the search tried
core-downloads-trying-search = Trying "{ $search }"
core-downloads-nothing-selected = nothing selected to download
core-downloads-added =
    { $count ->
        [one] Added { $count } download to your downloads.
       *[other] Added { $count } downloads to your downloads.
    }
core-downloads-movie-link = a movie link must be a magnet or an http(s) URL
core-downloads-added-movie = Added a movie to your downloads.
# $folder is a path, $name the library show that lives there
core-downloads-library-folder = { $folder } is the library folder for { $name } and contains episodes this subscription did not download. Remove the subscription without deleting its files, or remove { $name } from its page
core-downloads-no-torrents-named = no torrents named
core-downloads-paused =
    { $count ->
        [one] Paused { $count } download.
       *[other] Paused { $count } downloads.
    }
core-downloads-resumed =
    { $count ->
        [one] Resumed { $count } download.
       *[other] Resumed { $count } downloads.
    }
core-downloads-deleted-with-files =
    { $count ->
        [one] Deleted { $count } download and the files.
       *[other] Deleted { $count } downloads and the files.
    }
core-downloads-removed-from-list =
    { $count ->
        [one] Removed { $count } download from your list.
       *[other] Removed { $count } downloads from your list.
    }
core-downloads-nothing-to-remove = nothing to remove
# $title is the show's title
core-downloads-removed-show = Removed { $title } from your downloads.
core-downloads-removed-the-show = Removed the show from your downloads.
core-downloads-deleted-still-listed = Deleted files are still listed in the download app.
core-downloads-feed-url = feed URL must start with http:// or https://
core-downloads-rule-needs-name = rule name cannot be empty
core-downloads-no-rule = no rule named { $name }
core-downloads-feed-no-search = this feed has no search query to shorten
core-downloads-no-repair = { $name } needs no repair.
core-downloads-repaired = Repaired the subscription to { $name }.
# $search is the shortest search tried
core-downloads-repair-failed = no shorter search returned results; even "{ $search }" came back empty
# $id is the client's id, in quotes
core-downloads-no-saved-client = no saved torrent client { $id }
core-downloads-client-switched = Downloads now use a different download app.
# After the name of anirss or qBittorrent and a colon
core-downloads-host-not-running = is not running on the host
core-downloads-host-late = did not respond in time

## downloads/src/native.rs

core-downloads-native-empty-query = empty query
# $query is the search, in quotes
core-downloads-native-no-results = no results for { $query }
# $name is the feed's name, in quotes
core-downloads-native-feed-taken = a feed named { $name } already points to another address
core-downloads-native-no-items = no items given
# $link is the link given, in quotes
core-downloads-native-bad-link = links must be magnet: or http(s) URLs, got { $link }
core-downloads-native-no-links = no links given
core-downloads-native-no-names = no subscription names given
core-downloads-native-files-elsewhere = the torrent client stores its files on another machine, so they can only be deleted along with its torrents
# $names is a list of feed names
core-downloads-native-unknown-feeds = unknown feed(s): { $names }
# $paths is a list of folders
core-downloads-native-outside-folders = refusing to remove files outside the download folders: { $paths }
# Removals refused; $path is the folder or file
core-downloads-refuse-home-folder = { $path } is or contains your home folder, so nothing was removed
core-downloads-refuse-library-folder = { $path } is or contains a library folder, so nothing was removed

## downloads/src/paths.rs

core-downloads-no-default-folder = qBittorrent has no default save folder set; set one in qBittorrent
# $folder is the folder given
core-downloads-save-folder-not-absolute = { $folder } is not a full path to a folder
# $folder is the folder given
core-downloads-save-folder-unusable = { $folder } cannot be used for downloads: pick an existing folder that is not your home folder
core-downloads-name-empty = the name cannot be empty
core-downloads-name-slash = the name must not contain / or \
# $name is the name given, in quotes
core-downloads-name-not-folder = { $name } is not a folder name

## downloads/src/prune.rs

# $title is the show's title
core-downloads-pruned = Unsubscribed from { $title } (finished airing).
core-downloads-prune-failed = Could not unsubscribe from { $title }. Check Downloads in Settings.

## library/src/trash.rs

core-downloads-refuse-not-absolute = { $path } is not a plain absolute path, so nothing was removed
core-downloads-refuse-root = { $path } is the filesystem root, so nothing was removed
core-downloads-refuse-outside-library = { $path } is not inside a library folder, so nothing was removed
core-downloads-refuse-save-folder = { $path } is the torrent save folder, not the downloaded files, so nothing was removed

## downloads/src/watch.rs

# $episode is the episode's number, $title the show's
core-downloads-episode-missed = Episode { $episode } of { $title } still hasn't appeared three days after airing. Your feed will keep checking for it.

## downloads/src/settings.rs

core-downloads-settings-command-empty = the anirss command cannot be empty
core-downloads-settings-anirss-address = the anirss address must start with http:// or https://
# $client is the torrent client's name, such as qBittorrent
core-downloads-settings-client-address = the { $client } address must start with http:// or https://
# $id is the id given, in quotes
core-downloads-settings-bad-id = { $id } is not a client ID: use letters, digits, - and _
core-downloads-settings-same-id = two clients have the ID { $id }
# $name is a saved client's name
core-downloads-settings-saved-address = the address of { $name } must start with http:// or https://
core-downloads-settings-same-name = two clients are named { $name }; give each a unique name

## downloads/src/qbt.rs

# $address is where qBittorrent was asked, $message what the connection said
core-downloads-qbt-no-answer = qBittorrent did not respond at { $address }: { $message }
core-downloads-qbt-password-rejected = qBittorrent rejected the password; fix it in Settings
core-downloads-qbt-banned = qBittorrent has banned this address after failed logins; wait, or restart qBittorrent
core-downloads-qbt-login-rejected = qBittorrent rejected the username or password
# $status is the HTTP status code
core-downloads-qbt-login-failed = qBittorrent login at { $address } returned HTTP { $status }; will try again on the next request
core-downloads-qbt-not-web-ui = { $address } returned HTTP { $status }, which does not look like qBittorrent's Web UI. Check the address in Settings and make sure the Web UI is enabled in qBittorrent
# $said is what qBittorrent answered, in quotes
core-downloads-qbt-not-added = qBittorrent did not add the torrents: { $said }
core-downloads-qbt-some-not-added = qBittorrent could not add { $count } of the torrents: { $said }
# $line is the first line of what qBittorrent answered
core-downloads-qbt-answered = qBittorrent returned HTTP { $status }: { $line }

## downloads/src/replace.rs

# $item is the number of the release asked for, $count how many there are
core-downloads-replace-bad-item = item { $item } is not one of the { $count } releases
# $episode is the episode's number
core-downloads-not-replaced = Episode { $episode } was not replaced.
core-downloads-replaced = Replaced episode { $episode } with the new download.
# $name is a torrent's name, $folder the folder it moved to
core-downloads-no-video-moved = { $name } has no video in it, so Anibeam can't play it. It's in { $folder } now.
core-downloads-no-video = { $name } has no video in it, so Anibeam can't play it.

## downloads/src/rolling.rs

# $id is the rolling download's number
core-downloads-no-roll = no rolling download { $id }
# $title is the show, $from and $to the episodes kept on disk out of $total
core-downloads-rolled = { $title }: now on episodes { $from } to { $to } of { $total }.
core-downloads-roll-ahead = keep at least one episode ahead
core-downloads-roll-nothing = nothing to fetch

## releases/src/endpoints.rs

core-releases-json-needs-repo = a JSON search source must come from a source list whose definition maps its fields
core-releases-endpoint-no-name = missing `name`
# $name is the endpoint's name, in quotes
core-releases-endpoint-bad-name = { $name }: use only letters, digits, '.', '_', '-'
# $why is core-releases-json-needs-repo
core-releases-endpoint-json = { $name }: { $why }
# $kind is the kind given, in quotes; $valid lists the kinds there are
core-releases-endpoint-unknown-kind = { $name }: unknown kind { $kind } (valid: { $valid })
core-releases-endpoint-no-url = { $name }: missing `url`
core-releases-endpoint-not-http = { $name }: the address must start with http:// or https://
# $kind is the endpoint's kind; {query} is written as it is in the address
core-releases-endpoint-needs-query = { $name }: { $kind } URL must contain a {"{"}query{"}"} placeholder
core-releases-endpoint-no-query = { $name }: a { $kind } URL adds the search query automatically, so it should not include {"{"}query{"}"}

## releases/src/fetch.rs

# $code is the error code the indexer sent
core-releases-indexer-error = indexer returned error { $code }
core-releases-bad-feed = bad feed: { $error }
# $status is the HTTP status code
core-releases-feed-answered = feed returned HTTP { $status }

## releases/src/json_feed.rs

core-releases-not-feed-json = not the feed's JSON: { $error }

## releases/src/json.rs

core-releases-not-json = not JSON: { $error }
# $message is what the search said about itself
core-releases-search-error-said = search returned an error: { $message }
core-releases-search-error = search returned an error
# $path is where the results were looked for, in quotes
core-releases-no-results-field = the response has no results at { $path }
core-releases-results-not-list = the response results are not a list
core-releases-search-answered = search returned HTTP { $status }

## releases/src/native.rs

core-releases-no-endpoints = no download sources yet: add a source list or a custom source first
core-releases-no-source-named = no source named { $name }
core-releases-empty-query = type something to search for
core-releases-cannot-follow = { $name } cannot be followed: it supports search, but has no feed for new episodes
core-releases-endpoint-gone = { $name } is no longer available: removed, disabled, or missing its key

## releases/src/quirks.rs

# What is wrong with an endpoint definition's quirks. one_variant, show_feed, slug and
# {slug} are the definition's own words and stay as they are.
# $word is the word given, in quotes
core-releases-quirks-variant = one_variant { $word } must be one short word
core-releases-quirks-pattern-count = show_feed takes 1 to { $most } page patterns
core-releases-quirks-pattern-long = a show_feed page pattern is too long
core-releases-quirks-pattern-bad = show_feed page pattern: { $error }
core-releases-quirks-pattern-slug = a show_feed page pattern needs a group named slug
core-releases-quirks-url-https = show_feed url must be https
core-releases-quirks-url-host = show_feed url must be on the endpoint's own host
core-releases-quirks-url-slug = show_feed url takes {"{"}slug{"}"} once and no other placeholder

## releases/src/refinement.rs

# 'tokens' and `tokens` name the call's own field and stay as they are
core-releases-refine-no-tokens = op 'tokens' needs a non-empty `tokens` list

## releases/src/grid.rs

# A release group's whole-season torrent; $group is the group's name
core-releases-group-batch = { $group }'s batch
# $count is how many episodes, $list their numbers and runs ("1 to 4, 7")
core-releases-episodes-phrase =
    { $count ->
        [one] Episode { $list }
       *[other] Episodes { $list }
    }
# A run of episode numbers
core-releases-episode-range = { $from } to { $to }
