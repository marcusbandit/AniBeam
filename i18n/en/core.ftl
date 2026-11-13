# What the core says to a person: problems, errors, notices, and the words every shell
# shares. English is the source; every other language's core.ftl translates it.
# All of a language's files load into one bundle, so the terms here are seen from
# desktop.ftl and android.ftl as well.

## Names that stay the same in every language. Messages use them; none is shown alone.

-app-name = AniBeam
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
anilist-format-film = Film
anilist-format-special = Special
anilist-format-ova = OVA
anilist-format-ona = ONA
anilist-format-music = Music
anilist-status-finished = Finished
anilist-status-airing = Airing
anilist-status-not-yet-aired = Not yet aired
anilist-status-cancelled = Cancelled
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
event-problem-system = Something went wrong with AniBeam. Developer options have the details.
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
core-error-provider = { $provider } did not respond: { $message }
core-error-io = Could not read or write a file: { $message }
core-error-storage = Could not save your library: { $message }
core-error-keyring = Could not reach the system password store: { $message }
core-error-unsupported = { $what } is not supported yet.
core-error-version = This export is from a newer AniBeam (format { $found }). Update AniBeam to import it.
core-error-internal = AniBeam hit an internal error: { $message }
core-error-service = { $service }: { $message }

# What the core says to a person outside downloads, releases, trackers, metadata, the
# library, franchise, images and transfer: jobs, playback, casting, the host, the phone,
# endpoint repos, updates and the rest.

## core/src/repos/mod.rs

core-repos-paste-address = paste the address of a repo
core-repos-must-be-https = the address must be https
core-repos-unreachable = could not reach the repo: { $message }
core-repos-not-found = nothing at that address (a private GitHub repo answers 404 too)
core-repos-http-status = the repo answered HTTP { $status }
core-repos-unreadable = could not read { $path }: { $error }
core-repos-no-such-endpoint = no installed endpoint with that id
core-repos-key-changed = the repo is now signed by a different key than the one you trusted. Remove it and add it again to accept the new key.
core-repos-stale = the repo served an older index (serial { $serial } after { $kept }); keeping the newer one
core-repos-key-not-shown = the repo's key is not the one you were shown. Add it again and compare the fingerprint.
core-repos-repo-added = Added a list of download sources.
core-repos-repo-removed = Removed a list of download sources.
# $key is the field's key in quotes.
core-repos-no-field = { $name } has no field { $key }
core-repos-needs-field = { $name } needs its { $label }
core-repos-host-only = { $name }: the { $label } is a host or host:port only, like 192.168.1.10:9696
core-repos-not-added = that repo is not added
core-repos-not-offered = { $repo } does not offer { $id }
core-repos-already-installed = { $name } is already installed
core-repos-taken-by-you = an endpoint called { $id } is already installed (added by you); remove it first
core-repos-taken-by-repo = an endpoint called { $id } is already installed (from another repo); remove it first
core-repos-source-added = Added a download source.
core-repos-source-updated = Updated a download source.
core-repos-source-removed = Removed a download source.
core-repos-source-changed = Changed a download source.
core-repos-give-name = give it a name
core-repos-put-query = put {"{"}query{"}"} in the address where the search goes
core-repos-name-taken = an endpoint called { $id } is already installed
core-repos-only-by-hand = only a source added by hand can be edited
# $key is a placeholder's name; it stays in braces in the address.
core-repos-keep-key = keep {"{"}{ $key }{"}"} in the address and change its value with Change its { $label }, or remove the site and add it again without it
core-repos-note-kind = anirss cannot search the { $kind } kind, so the desktop leaves it out; the phone searches it
core-repos-note-by-hand = anirss has its own { $id }, written by hand; the desktop searches that one
core-repos-note-theirs = This anirss is one you run, and the desktop never changes its config: add { $id } to that anirss's own config for the desktop to search it
core-repos-note-sandbox = This run never writes into anirss's config, so the desktop does not search { $id } here
core-repos-note-old = anirss { $version } cannot take endpoints from { -app-name }; update it to { $min } or newer for the desktop to search this

## core/src/repos/index.rs

core-repos-key-not-ed25519 = the index's key is not an ed25519 key
core-repos-key-not-base64 = the index's key is not 32 bytes of base64
core-repos-key-invalid = the index's key is not a valid ed25519 key
core-repos-too-large = the file is too large to be a repo
core-repos-not-a-repo = this is not an { -app-name } endpoint repo
core-repos-format-version = the repo is format version { $found }; this { -app-name } reads version { $reads }. Update { -app-name }.
core-repos-payload-not-base64 = the repo's payload is not base64
core-repos-signature-malformed = the repo's signature is malformed
core-repos-index-unreadable = the repo's index does not read: { $error }
core-repos-signature-mismatch = the repo's signature does not match its contents
# $id is the source's id; the rest of these checks name a source by its id first.
core-repos-check-scheme = { $id }: the address must start with https://
core-repos-check-scheme-http = { $id }: the address must start with http:// or https://
# $key is the field's key in quotes.
core-repos-check-auth-key = { $id }: bad auth key { $key }
core-repos-check-host-torznab = { $id }: only a torznab address may name its host as {"{"}{ $key }{"}"}
core-repos-check-host-placeholder = { $id }: a placeholder in the host

## core/src/playback/tracks.rs

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

## core/src/contract/enums.rs

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

## core/src/airing_now.rs

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

## core/src/aired.rs

core-aired-episode-out = Episode { $number } of { $title } is out.

## core/src/cast/mod.rs

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
core-cast-not-handed = the other computer did not hand over the episode: { $error }
core-cast-empty-playlist = the other computer sent an empty playlist
# $error is why the TV refused the converted stream, $first why it refused the file.
core-cast-refused-both = { $error }; it refused the file as it is too ({ $first })
core-cast-chromecast-stopped = the Chromecast stopped
core-cast-hevc = { $error }. This episode is HEVC, which only newer Chromecasts (with Google TV, or Ultra) decode

## core/src/cast/castv2.rs

core-cast-chromecast-unreachable = could not reach the Chromecast: { $error }
core-cast-chromecast-broke = the Chromecast connection broke: { $error }
core-cast-chromecast-no-player = the Chromecast did not start its media player
# $reason is the Chromecast's own word for it, or core-cast-no-reason.
core-cast-chromecast-refused-player = the Chromecast would not start its media player ({ $reason })
core-cast-no-reason = no reason
# $kind is the Chromecast's own word for the failure.
core-cast-chromecast-cannot-play = the Chromecast could not play this ({ $kind })
core-cast-chromecast-error = the Chromecast stopped with an error playing this

## core/src/cast/dlna.rs

core-cast-tv-no-answer = the TV did not answer: { $error }
# $action is the DLNA command; $code the TV's own error.
core-cast-tv-refused = the TV refused { $action }
core-cast-tv-refused-code = the TV refused { $action } ({ $code })

## core/src/cast/media.rs

core-cast-no-video = This file has no video.
core-cast-hi10p = This episode is 10-bit H.264 (Hi10P), which no TV or Chromecast decodes. Casting it needs a transcode, which is not built yet.
core-cast-video-codec = This episode's video ({ $codec }) cannot be cast.
core-cast-audio-codec = This episode's audio ({ $codec }) cannot be cast.

## core/src/phone/mod.rs

core-phone-download-started =
    { $count ->
        [one] Started downloading an episode of { $folder }.
       *[other] Started downloading { $count } episodes of { $folder }.
    }
core-phone-download-partial = Not every release of { $folder } could be added: { $error }
core-phone-subscribed = Subscribed to { $name }.
core-phone-rule-stopped = { $name } is not checked for new episodes: its source is gone or needs its key again.
core-phone-new-episode = A new episode of { $name } is downloading.
# $name is the source's name in quotes.
core-phone-name-taken = { $name }: another source has that name
core-phone-nothing-to-download = nothing to download
# $used and $cap are in GB, with one decimal.
core-phone-over-cap = this would take { -app-name } past its storage cap ({ $used } of { $cap } GB used or on the way); delete episodes or raise the cap
core-phone-empty-search = the search is empty
core-phone-no-results = no results for { $query }
# Ends "... is not supported yet." (core-error-unsupported).
core-phone-own-engine = a torrent engine of its own

## core/src/phone/check.rs

# The connection check: each line is a step for one host ($label), and what came of it.
core-phone-check-connected = connected
core-phone-check-refused = refused: { $error }
core-phone-check-dropped = no answer (dropped)
core-phone-check-finding = { $label }: finding its address
core-phone-check-failed = failed: { $error }
core-phone-check-no-answer = no answer
core-phone-check-private = { $label }: finding it privately
core-phone-check-skipped-address = skipped, an address
core-phone-check-skipped-home = skipped, home-network name
core-phone-check-no-record = no public address, left to this network's DNS
core-phone-check-no-resolver = no resolver answered
core-phone-check-reaching = { $label }: reaching { $ip }
core-phone-check-http = HTTP { $status }, { $bytes } bytes
core-phone-check-setup = setup

## core/src/playback/session.rs

core-playback-not-saved = Your progress on { $folder } was not saved. Match the show first.
# $done is a whole percentage.
core-playback-still-downloading = episode { $number } is still downloading, { $done }% done

## core/src/playback/skip.rs

core-playback-bad-duration = a duration is a positive number of seconds

## core/src/agent.rs

# A coding agent looking up a show's schedule; these lines say what it is doing.
core-agent-no-answer = the agent gave no answer
core-agent-searching = Searching the web for { $query }
core-agent-searching-schedule = Searching the web for the schedule
core-agent-reading = Reading { $url }
core-agent-reading-page = Reading a page
# $tool is the agent's own name for a tool.
core-agent-using = Using { $tool }
# Ends "... is not supported yet." (core-error-unsupported).
core-agent-none = a coding agent on this computer
# $name is the coding agent's name.
core-agent-ended = { $name } ended without an answer
core-agent-too-long = { $name } took too long

## core/src/core.rs

# A guest device on another computer's library asks for something that is the owner's.
core-core-guest-trackers = the tracker accounts on that computer are its owner's, so this device marks nothing there
core-core-guest-not-interested = Not interested on that computer is its owner's list, so this device changes nothing there
core-core-guest-recommended = the recommendations on that computer are learnt from its owner's list
core-core-nothing-picked = nothing picked

## core/src/prefs.rs

core-prefs-not-a-provider = { $provider } is not a match provider
core-prefs-provider-count = { $provider } appears { $count } times; every provider appears once
# A setting out of its bounds: $value is outside $low to $high.
core-prefs-outside = { $value } is outside { $low } to { $high }
core-prefs-empty = empty

## core/src/recommended/mod.rs

core-recommended-ranking = Ranking shows for you

## core/src/recommended/sync.rs

core-recommended-finding-people = Finding people with your taste
core-recommended-reading-lists = Reading their lists
core-recommended-reading-yours = Reading your shows
core-recommended-reading-suggestions = Reading shows to suggest

## core/src/recommended/page.rs

core-recommended-no-anime = { -anilist } has no anime { $id }

## core/src/host/mod.rs

# The tailnet host's status line in Settings.
core-host-off = Off
core-host-starting = Starting
# $addresses is a list of address:port, joined with commas.
core-host-serving = Serving on { $addresses }
core-host-no-tailscale = Tailscale is not connected
core-host-cannot-listen = Could not listen
core-host-port-taken = Port { $port } is taken
core-host-cannot-listen-on = Could not listen on { $address }: { $error }

## core/src/host/addr.rs

core-host-not-an-address = { $text } is not an address and port

## core/src/host/keys.rs

# $key is the text given, in quotes.
core-host-not-a-key = { $key } is not a device key

## core/src/net/limiter.rs

core-net-timed-out = timed out after { $ms }ms
core-net-rate-limited = { $provider } rate limited

## core/src/net/mod.rs

core-net-waiting = { $host }: timed out a moment ago; { -app-name } waits { $seconds } s before asking it again

## core/src/net/anilist.rs

core-net-no-data = no data

## core/src/update/install.rs

core-update-no-download = { -app-name } { $version } has no download for this computer
core-update-cannot-make = Could not make { $path }: { $error }
core-update-cannot-mark = Could not mark the new AppImage runnable: { $error }
core-update-cannot-replace = Could not replace { $path }: { $error }
core-update-not-here = This copy is not updated here
core-update-cannot-start = Could not start the download: { $error }
core-update-did-not-start = The download did not start: { $error }
core-update-cannot-write = Could not write { $path }: { $error }
core-update-stopped = The download stopped: { $error }
core-update-cancelled = Cancelled
# $got and $size are in bytes.
core-update-ended-early = The download ended early ({ $got } of { $size } bytes)
core-update-checksum = The download does not match GitHub's checksum
core-update-cannot-move = Could not move the download into place: { $error }
core-update-cannot-open-dmg = Could not open the dmg: { $error }
core-update-cannot-read-dmg = Could not read the dmg: { $error }
core-update-no-app = The dmg holds no app
core-update-cannot-copy = Could not copy the new app beside this one: { $error }

## core/src/calendar.rs

# An episode's event in the calendar file.
core-calendar-season-episode = { $title } - Season { $season } Episode { $number }
core-calendar-episode = { $title } - Episode { $number }

## core/src/paths.rs

core-paths-no-home = no home directory

## core/src/first_run.rs

core-first-run-not-absolute = the videos folder must be an absolute path

## core/src/presence/mod.rs

# Discord shows this in place of an episode line while an extra plays.
core-presence-extra = Extra

# The core's words for the library group: trackers, metadata, library, franchise,
# images and transfer (#798).

## core/src/trackers/accounts.rs

# $tracker is the tracker's short name: AniList or MAL.
core-trackers-no-client-id = No client ID set for { $tracker }.
core-trackers-mal-reconnect = Reconnect MyAnimeList in Settings.
core-trackers-mal-wants-secret = MyAnimeList wants this app's client secret. Add it under Trackers in Settings, or register the app with App Type "other".
# $error is the parser's own message.
core-trackers-token-reply-unreadable = unreadable token reply: { $error }
core-trackers-token-reply-no-token = the token reply carried no access token

## core/src/trackers/mod.rs

core-trackers-no-token = no access token stored, reconnect in Settings

## core/src/trackers/cache.rs

core-trackers-no-user-id = the account carries no user id

## core/src/trackers/move_up.rs

core-trackers-move-none-picked = no series picked to move
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
    } to { $site }; { $site } did not take { $failed ->
        [one] { $failed } show
       *[other] { $failed } shows
    }.
# $show is the show's folder name; $error is the tracker's answer, cleaned up.
core-trackers-move-refused = { $site } did not take { $show }: { $error }

## core/src/trackers/oauth.rs

core-trackers-oauth-timed-out = Authorization timed out, try again.
core-trackers-oauth-state-mismatch = state mismatch, possible CSRF, aborted.
core-trackers-oauth-no-code = no code in callback
core-trackers-oauth-no-token = no access_token in callback
core-trackers-oauth-waiting = Waiting for you to sign in to { $site } in your browser.
# $username is the account's name on the tracker.
core-trackers-oauth-connected = Connected { $site } as { $username }.
core-trackers-oauth-no-name = the profile came back without a name
# Shown before a colon and the reason.
core-trackers-oauth-could-not-start = Signing in could not start
# $error is the system's own message.
core-trackers-oauth-port-failed = port { $port } on this computer could not be opened: { $error }
core-trackers-oauth-port-busy = port { $port } on this computer is in use by another program. Wait a moment and try again, or close the program using it if it stays busy.
# $error is the code the tracker sent back.
core-trackers-oauth-provider-error = { $site } returned an error: { $error }
# The browser tab the sign-in leaves. $app is AniBeam's name, in the brand colour in the
# page's body; keep it as a variable.
core-trackers-page-connected = { $site } is connected to { $app }
core-trackers-page-close-tab = You can close this tab.
core-trackers-page-failed-title = { $app }, sign-in failed
core-trackers-page-failed = The sign-in did not finish
core-trackers-page-go-back = Go back to { $app } and try again.
core-trackers-page-finishing-title = { $app }, still finishing
core-trackers-page-finishing = Still finishing
core-trackers-page-finishing-body = { $site } answered and AniBeam is finishing the sign-in. Return to { $app }, which says when it is done.
core-trackers-page-cancelled-title = { $app }, sign-in cancelled
core-trackers-page-cancelled = This sign-in was cancelled
core-trackers-page-cancelled-body = You can close this tab. Start again from { $app } if you meant to.

## core/src/trackers/on_hold.rs

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

## core/src/trackers/writes.rs

# A tracker's failure in words a person can act on. $tracker is AniList or MAL.
core-trackers-rate-limited = { $tracker } rate limited, try again in a minute.
core-trackers-auth-expired = { $tracker } auth expired, reconnect in Settings.
core-trackers-entry-not-found = { $tracker } entry not found.
# $status is the HTTP status code.
core-trackers-server-error = { $tracker } server error ({ $status }), try again later.
core-trackers-error-see-log = { $tracker } error, see activity log.
# $episode is the number asked for, as it was typed.
core-trackers-episode-not-whole = { $episode } is not a whole episode above nought
core-trackers-score-out-of-range = { $score } is outside 0 to 10
core-trackers-rewatch-not-settable = a rewatch is not a status to set by hand
core-trackers-notes-too-long = notes are kept to { $max } characters
core-trackers-nothing-picked = nothing picked
core-trackers-notes-not-on-list = Not on your { $tracker } list yet, add it to a list first.
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

## core/src/metadata/automatch.rs

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

## core/src/metadata/apply.rs

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

## core/src/metadata/fetch.rs

# $id is the provider's own number for the show.
core-metadata-no-such-show = TVmaze has no show { $id }
core-metadata-no-such-media = AniList has no media { $id }

## core/src/metadata/imdb.rs

# Progress labels while the ratings update.
core-metadata-progress-imdb-links = Wikidata's IMDb links
core-metadata-progress-imdb-ratings = IMDb ratings

## core/src/franchise/layout.rs

# The words under a related show's title when the two share characters.
core-franchise-relation-character = Shared characters

## core/src/images/mod.rs

# A progress label while covers and banners download.
core-images-progress-fetching = fetching images

## core/src/transfer/export.rs

# $folder is the name of the folder the file went to.
core-transfer-exported =
    { $count ->
        [one] Saved { $count } show to { $folder }.
       *[other] Saved { $count } shows to { $folder }.
    }

## core/src/transfer/format.rs

# $error is the JSON parser's own message.
core-transfer-not-json = not JSON: { $error }
# $format is the file format's name, anibeam-export.
core-transfer-wrong-format = not an { $format } document

## core/src/transfer/import.rs

core-transfer-imported = Loaded your library from the file.

## core/src/library/delete.rs

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

## core/src/library/detail.rs

# A fix the series page offers. $read is episode, with its $number, or label, with the
# name the file was read by in $label. $what is film or episode.
core-library-fix-one-file =
    The one file here reads as { $read ->
        [episode] episode { $number }
       *[label] { $label }
    }. It looks like { $what ->
        [film] the film
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
core-library-fix-numbered-on = Episodes { $from } look like this record's { $to }, numbered on from an earlier season.
# Episode numbers in a row: "1 to 6".
core-library-runs-span = { $first } to { $last }
# Episode numbers apart: "2, 4 and 7". $init is every number but the last, comma-separated.
core-library-runs-list = { $init } and { $last }

## core/src/library/fixes.rs

core-library-fix-episode-range = an episode number runs from 0 to 10000
core-library-fix-season-range = a season runs up to 200
core-library-fix-extra-range = an extra's index runs up to 10000
core-library-file-elsewhere-not-fixed = file { $id } belongs to another series, so nothing was fixed
# $title is the show's title.
core-library-fix-undone =
    { $count ->
        [one] Went back to reading { $count } file of { $title } by name.
       *[other] Went back to reading { $count } files of { $title } by name.
    }
core-library-fixed =
    { $count ->
        [one] Fixed how { $count } file of { $title } are read.
       *[other] Fixed how { $count } files of { $title } are read.
    }

## core/src/library/reads.rs

# $title is the show's title. The -many lines are for any count but one.
core-library-hid-one = Hid { $title }.
core-library-back-one = { $title } is back.
core-library-hid-many = Hid { $count } shows.
core-library-back-many = { $count } shows are back.
core-library-episode-count-set = { $title } now has { $count } episodes.
core-library-episode-count-cleared = Cleared the episode count of { $title }.

## core/src/library/schedule.rs

core-library-schedule-cleared = Cleared the schedule notes of { $title }.
core-library-schedule-saved = Saved { $count } schedule notes for { $title }.

## core/src/library/scan.rs

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

## core/src/library/report.rs

# Why a coding agent wrote no triage for a quick fix report. $span is
# core-library-span-minutes or -seconds.
core-library-agent-no-answer = no answer within { $span }
core-library-agent-stopped-code = it stopped with code { $code }
core-library-agent-stopped = it stopped
core-library-agent-answered-nothing = it answered nothing
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

## core/src/downloads/anirss.rs

# $message is what the connection said
core-downloads-anirss-no-answer = anirss did not answer: { $message }
# $status is the HTTP status code, $text the body anirss sent
core-downloads-anirss-answered = anirss answered { $status }: { $text }
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
core-downloads-anirss-too-old = anirss { $version } is too old for AniBeam, which needs { $minimum } or newer. { $update }.
core-downloads-anirss-lacks-feature = anirss { $version } cannot keep each torrent client's login to itself, which AniBeam needs to switch clients; anirss { $minimum } and newer can. { $update }.
core-downloads-anirss-update-mac = Update it with: { $command }
core-downloads-anirss-update-linux = Update the anirss package, or run: { $command }

## core/src/downloads/contents.rs

core-downloads-torrent-file-no-answer = the torrent file did not come: { $message }
core-downloads-torrent-file-answered = the torrent file answered { $status }

## core/src/downloads/feed.rs

core-downloads-feed-answered = the feed answered { $status }
core-downloads-not-a-feed = not a feed: { $error }

## core/src/downloads/jackett.rs

# $program is the command that was not found
core-downloads-jackett-missing = Jackett is not installed: { $program } was not found. Install it, or change the command in Settings.
# $name is the endpoint, $why what went wrong
core-downloads-endpoint-start-failed = { $name } could not start: { $why }
core-downloads-endpoint-start-timeout = { $name } started but did not answer within { $seconds } seconds.
core-downloads-command-exited = the command exited with { $status }

## core/src/downloads/mod.rs

core-downloads-anirss-stopped = anirss stopped
# $install is core-downloads-anirss-install-mac or -linux
core-downloads-anirss-missing = { $install }. Or point Settings at a running server.
# $command is the anirss command set in Settings
core-downloads-anirss-command-missing = { $command } was not found. { $install }. Or point Settings at a running server.
core-downloads-anirss-start-failed = { $command } could not start: { $error }
# $tooOld is core-downloads-anirss-too-old; $message is what anirss said
core-downloads-anirss-too-old-said = { $tooOld } It said: { $message }
core-downloads-no-torrent-client = No qBittorrent answers at { $address }, and AniBeam has none of its own to run. Install qBittorrent and turn on its Web UI, or install qbittorrent-nox, which AniBeam then runs for you
core-downloads-own-client-gone = AniBeam's qBittorrent is gone: qbittorrent-nox is not installed. Reinstall AniBeam, or point Settings at a qBittorrent of your own
core-downloads-no-port = no port in { $address }
core-downloads-own-client-settings = cannot write qBittorrent's settings: { $error }
core-downloads-own-client-start-failed = { $command } could not start: { $error }
core-downloads-own-client-slow-start = AniBeam's qBittorrent did not start
# $said is the last lines of its log
core-downloads-own-client-slow-start-said = AniBeam's qBittorrent did not start: { $said }
core-downloads-not-ready = Downloads are not ready. Check Downloads in Settings.
core-downloads-ready = Downloads are ready.
# $count is how many sources anirss's own config names
core-downloads-helper-sources =
    { $count ->
        [one] Downloads now search with AniBeam itself, which does not read the { $count } source in the search helper's own config. Add them in Settings, Downloads, or pick the search helper there.
       *[other] Downloads now search with AniBeam itself, which does not read the { $count } sources in the search helper's own config. Add them in Settings, Downloads, or pick the search helper there.
    }
core-downloads-client-connected = The download app is connected.
core-downloads-client-not-connected = The download app is not connected. Check its address in Settings.
core-downloads-not-on-demand = { $name } is not an on-demand endpoint
core-downloads-source-running = The download source is running.
core-downloads-source-not-running = { $name } is not running. Switch it on in the sources and confirm to start it.
core-downloads-empty-query = type something to search for
# What a search's progress line names when it was given no source
core-downloads-first-endpoint = the first endpoint
# A search's progress line. The desktop's search card compares the line with these, so
# keep the name in core-downloads-search-starting-endpoint a whole placeable.
core-downloads-search-starting = Starting the search
core-downloads-search-starting-endpoint = Starting { $endpoint }
core-downloads-search-searching-endpoint = Searching { $endpoint }
core-downloads-search-reading = Reading the results
# $name is the subscription's name
core-downloads-subscribed = Subscribed to { $name }.
core-downloads-client-no-rule = qBittorrent holds no rule named { $name }
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
core-downloads-unsubscribed = Unsubscribed from { $name }.
# $folder is a path, $name the library show that lives there
core-downloads-library-folder = { $folder } is the library folder of { $name }, with episodes this subscription did not download: remove the subscription without its files, or remove { $name } from its page
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
core-downloads-feed-url = a feed is an http(s) URL
core-downloads-rule-needs-name = a rule has a name
core-downloads-no-rule = no rule named { $name }
core-downloads-feed-no-search = the feed carries no search to shorten
core-downloads-no-repair = { $name } needs no repair.
core-downloads-repaired = Repaired the subscription to { $name }.
# $search is the shortest search tried
core-downloads-repair-failed = no shorter search finds anything; even "{ $search }" comes back empty
# $id is the client's id, in quotes
core-downloads-no-saved-client = no saved torrent client { $id }
core-downloads-client-switched = Downloads now use a different download app.
# After the name of anirss or qBittorrent and a colon
core-downloads-host-not-running = is not running on the host
core-downloads-host-late = did not answer in time

## core/src/downloads/native.rs

core-downloads-native-empty-query = empty query
# $query is the search, in quotes
core-downloads-native-no-results = no results for { $query }
# $name is the feed's name, in quotes
core-downloads-native-feed-taken = a feed named { $name } already reads another address
core-downloads-native-no-items = no items given
# $link is the link given, in quotes
core-downloads-native-bad-link = links must be magnet: or http(s) URLs, got { $link }
core-downloads-native-no-links = no links given
core-downloads-native-no-names = no subscription names given
core-downloads-native-files-elsewhere = the torrent client keeps its files on another machine, so they can only go with its torrents
# $names is a list of feed names
core-downloads-native-unknown-feeds = unknown feed(s): { $names }
# $paths is a list of folders
core-downloads-native-outside-folders = refusing to remove files outside the download folders: { $paths }
# Removals refused; $path is the folder or file
core-downloads-refuse-home-folder = { $path } is the home folder or holds it, so nothing was removed
core-downloads-refuse-library-folder = { $path } is a library folder or holds one, so nothing was removed

## core/src/downloads/paths.rs

core-downloads-no-default-folder = qBittorrent names no default save folder; set one in qBittorrent
core-downloads-name-empty = the name cannot be empty
core-downloads-name-slash = the name must not contain / or \
# $name is the name given, in quotes
core-downloads-name-not-folder = { $name } is not a folder name

## core/src/downloads/prune.rs

# $title is the show's title
core-downloads-pruned = Unsubscribed from { $title }, it finished airing.
core-downloads-prune-failed = Could not unsubscribe from { $title }. Check Downloads in Settings.

## core/src/downloads/trash.rs

core-downloads-refuse-not-absolute = { $path } is not a plain absolute path, so nothing was removed
core-downloads-refuse-root = { $path } is the filesystem root, so nothing was removed
core-downloads-refuse-outside-library = { $path } is not inside a library folder, so nothing was removed
core-downloads-refuse-save-folder = { $path } is the folder the torrent was saved in, not what it downloaded, so nothing was removed

## core/src/downloads/watch.rs

# $episode is the episode's number, $title the show's
core-downloads-episode-missed = Episode { $episode } of { $title } has not turned up three days after it aired. Your feed keeps looking for it.

## core/src/downloads/settings.rs

core-downloads-settings-command-empty = the anirss command cannot be empty
core-downloads-settings-anirss-address = the anirss address must start with http:// or https://
# $client is the torrent client's name, such as qBittorrent
core-downloads-settings-client-address = the { $client } address must start with http:// or https://
# $id is the id given, in quotes
core-downloads-settings-bad-id = { $id } is not a client id: use letters, digits, - and _
core-downloads-settings-same-id = two clients have the id { $id }
# $name is a saved client's name
core-downloads-settings-saved-address = the address of { $name } must start with http:// or https://
core-downloads-settings-same-name = two clients are called { $name }; give each a name of its own

## core/src/downloads/qbt.rs

# $address is where qBittorrent was asked, $message what the connection said
core-downloads-qbt-no-answer = qBittorrent did not answer at { $address }: { $message }
core-downloads-qbt-password-rejected = qBittorrent rejected the password; fix it in Settings
core-downloads-qbt-banned = qBittorrent has banned this address after failed logins; wait, or restart qBittorrent
core-downloads-qbt-login-rejected = qBittorrent rejected the username or password
# $status is the HTTP status code
core-downloads-qbt-login-failed = qBittorrent's login at { $address } answered { $status }; trying again on the next call
core-downloads-qbt-not-web-ui = { $address } answered { $status }, but not as qBittorrent's Web UI. Check the address in Settings, and that the Web UI is turned on in qBittorrent
# $said is what qBittorrent answered, in quotes
core-downloads-qbt-not-added = qBittorrent did not add the torrents: { $said }
core-downloads-qbt-some-not-added = qBittorrent could not add { $count } of the torrents: { $said }
# $line is the first line of what qBittorrent answered
core-downloads-qbt-answered = qBittorrent answered { $status }: { $line }

## core/src/downloads/replace.rs

# $item is the number of the release asked for, $count how many there are
core-downloads-replace-bad-item = item { $item } is not one of the { $count } releases
# $episode is the episode's number
core-downloads-not-replaced = Episode { $episode } was not replaced.
core-downloads-replaced = Replaced episode { $episode } with the new download.

## core/src/downloads/rolling.rs

# $id is the rolling download's number
core-downloads-no-roll = no rolling download { $id }
# $title is the show, $from and $to the episodes kept on disk out of $total
core-downloads-rolled = { $title }: now on episodes { $from } to { $to } of { $total }.
core-downloads-roll-ahead = keep at least one episode ahead
core-downloads-roll-nothing = nothing to fetch

## core/src/releases/endpoints.rs

core-releases-json-needs-repo = a JSON search source comes from an endpoint repo whose definition maps its fields
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
core-releases-endpoint-needs-query = { $name }: { $kind } url must contain a {"{"}query{"}"} placeholder
core-releases-endpoint-no-query = { $name }: a { $kind } url adds the search itself, so it takes no {"{"}query{"}"}

## core/src/releases/fetch.rs

# $code is the error code the indexer sent
core-releases-indexer-error = the indexer answered error { $code }
core-releases-bad-feed = bad feed: { $error }
# $status is the HTTP status code
core-releases-feed-answered = the feed answered { $status }

## core/src/releases/json_feed.rs

core-releases-not-feed-json = not the feed's JSON: { $error }

## core/src/releases/json.rs

core-releases-not-json = not JSON: { $error }
# $message is what the search said about itself
core-releases-search-error-said = the search answered with an error: { $message }
core-releases-search-error = the search answered with an error
# $path is where the results were looked for, in quotes
core-releases-no-results-field = the reply has no results at { $path }
core-releases-results-not-list = the reply's results are not a list
core-releases-search-answered = the search answered { $status }

## core/src/releases/native.rs

core-releases-no-endpoints = no endpoints yet: add a repo, or one by hand, under Endpoints in More
core-releases-no-source-named = no source named { $name }
core-releases-empty-query = type something to search for
core-releases-cannot-follow = { $name } cannot be followed: it answers searches, but has no feed to check for new episodes
core-releases-endpoint-gone = { $name } is not available any more: removed, switched off or missing its key

## core/src/releases/quirks.rs

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

## core/src/releases/refinement.rs

# 'tokens' and `tokens` name the call's own field and stay as they are
core-releases-refine-no-tokens = op 'tokens' needs a non-empty `tokens` list

## core/src/releases/grid.rs

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

## core/src/subscriptions.rs

# Fills core-error-unsupported: "anirss on this platform is not supported yet."
core-subscriptions-platform = anirss on this platform
# After "anirss: "; $message is why its answer did not parse
core-subscriptions-unreadable = its answer could not be read ({ $message })
core-subscriptions-stopped = it stopped without saying why
# $said is what anirss printed
core-subscriptions-stopped-said = it stopped: { $said }
