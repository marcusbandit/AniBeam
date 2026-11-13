# The Android app's words (#793), English, the source every other language follows.
# Pages ask by ID through the core's tr: Tr.t("look-mode") in Slint, tr!("look-mode") in Rust.
# IDs say where a line lives and what it is, never the English.

## Names that stay names

tracker-anilist = AniList
tracker-mal = MyAnimeList

## Words many pages share. Use these before adding a page's own copy.

common-cancel = Cancel
common-back = Back
common-save = Save
common-done = Done
common-close = Close
common-delete = Delete
common-retry = Try again
common-copy = Copy
common-clear = Clear
common-on = On
common-off = Off
common-play = Play
common-hide = Hide
common-remove = Remove
common-add = Add
common-test = Test
common-reading = Reading...
common-working = Working...
common-checking = Checking...
common-all = All
common-none = None
common-episodes = { $count ->
    [one] { $count } episode
   *[other] { $count } episodes
}
common-episode-n = Episode { $n }
# Two parts of a line of facts, such as "1.2 GB of 1.4 GB" and "3 MB/s".
common-join-comma = { $a }, { $b }

## Language (#793)

lang-follow-phone = Same as the phone
more-language = Language
more-language-phone = Same as the phone: { $lang }
lang-name-en = English
lang-name-da = Danish
lang-name-de = German
lang-name-es = Spanish
lang-name-fr = French
lang-name-pt-BR = Portuguese (Brazil)
lang-name-it = Italian
lang-name-nl = Dutch
lang-name-pl = Polish
lang-name-ru = Russian
lang-name-ja = Japanese
lang-name-ko = Korean
lang-name-zh-CN = Chinese (Simplified)

## Onboarding

onb-welcome-title = AniBeam
onb-welcome-note = Your anime on this phone. Find a show, see who has it, download it, and keep your lists in step.
onb-language-title = Language
onb-language-note = AniBeam speaks your phone's language unless you pick another. You can change it later in More.
onb-lists-title = Your lists
onb-lists-note = Sign in so what you watch here counts on AniList or MyAnimeList. You can do this later in More.
onb-wifi-title = Wi-Fi only
onb-wifi-note = Episodes are big. AniBeam downloads on Wi-Fi unless you say otherwise.
onb-storage-title = Storage
onb-storage-note = A cap on how much AniBeam may download. It reserves nothing: the space stays free until episodes fill it.
onb-notify-title = Notifications
onb-notify-note = While episodes download, a notification shows how far along they are, and another says when one is ready to watch. Downloads run either way.
onb-sites-title = Search sites
onb-sites-note = AniBeam comes with nothing to search. A repo is a signed list of sites to search. Paste a repo's address, check its key, then install the sites you want. You can do this later in More.
onb-wifi-only = Only on Wi-Fi
onb-wifi-on = On. Mobile data asks first.
onb-wifi-off = Off. Downloads use mobile data too.
onb-storage-caption = 18 GB is about 12 episodes. Slide it anywhere up to the phone's whole storage.
onb-use-cap = Use this cap
onb-cap-on = On. At { $cap }, AniBeam downloads nothing more until you delete episodes.
onb-cap-off = Off. AniBeam downloads until the phone is full.
onb-start = Start
onb-continue = Continue
onb-back = Back

## Appearance

look-colours = Colours
look-mode = Mode
look-dark = Dark
look-light = Light
look-system = System
look-follows-phone-dark = Follows the phone's own setting: dark.
look-follows-phone-light = Follows the phone's own setting: light.
look-changes-at-once = Changes show at once, here and on every page.
look-dark-theme = Dark theme
look-light-theme = Light theme
look-accent = Accent
look-accent-note = Marks what is selected, playing or new.
look-size = Size
look-text-size = Text size
look-size-smaller = Smaller
look-size-normal = Normal
look-size-large = Large
look-size-larger = Larger
look-size-largest = Largest
look-text-size-note = How big the words are everywhere in AniBeam, on top of your phone's own size.

## The window: tabs and the sheets over every page

app-title = AniBeam
nav-home = Home
nav-calendar = Calendar
nav-library = Library
nav-downloads = Downloads
nav-more = More
app-mobile-title = Download over mobile data?
app-mobile-note = One episode is often 1 to 2 GB. On a capped plan that can use up a month of data in an evening.
app-mobile-keep = Keep Wi-Fi only
app-mobile-use = Use mobile data too
# Above a list of releases of one episode.
app-versions-note = The same episode from { $count } releases. Pick the one to play.
# A release's name, on the button for the one playing now.
app-versions-plays-now = { $release }, plays now
app-menu-get = Get episodes
app-menu-delete = Delete episodes

## More: the index and each section's line of how it stands

more-settings = Settings
more-search = Search settings
more-search-none = Nothing matches
more-search-hits = { $count ->
    [one] 1 setting
   *[other] { $count } settings
}
more-search-tap = Tap one to go there
more-search-hint = Try a plainer word: storage, Wi-Fi, tracker, site, computer.
more-accounts = Accounts
more-streaming = Streaming
more-subtitles = Subtitles
more-connection = Connection
more-computer = Your computer
more-icon = App icon
more-stats = Stats
more-data = Data
more-appearance = Appearance
more-activity = Activity
more-about = About
more-accounts-none = No tracker connected
# A tracker's name: AniList or MAL.
more-accounts-syncing = Syncing to { $tracker }
more-accounts-syncing-both = Syncing to AniList and MAL
more-dl-status-wifi = Wi-Fi only, no cap
more-dl-status-wifi-cap = Wi-Fi only, capped at { $cap }
more-dl-status-mobile = Wi-Fi and mobile data, no cap
more-dl-status-mobile-cap = Wi-Fi and mobile data, capped at { $cap }
more-stream-status-auto = Auto, 720p on mobile data or a relay
more-stream-status-original = The original file, every time
# A quality such as 1080p.
more-stream-status-converted = Converted to { $quality }
more-sites-status-none = No repo yet
more-sites-status-on = { $sites ->
    [one] 1 site on
   *[other] { $sites } sites on
}
more-sites-status-on-from = { $sites ->
    [one] 1 site on
   *[other] { $sites } sites on
}, { $repos ->
    [one] from 1 repo
   *[other] from { $repos } repos
}
more-conn-status-unchecked = Not checked yet
more-conn-status-ok = Every step went through
more-conn-status-failed = { $total ->
    *[other] { $failed } of { $total } steps failed
}
more-computer-status-none = Not connected yet
more-icon-default = Default
more-icon-dark = Dark
more-icon-line = Line art
more-subs-status = Size, place, font, colours and box
more-data-status = Back up, restore, image cache
more-activity-status = What AniBeam did, newest first
more-look-status-phone = Follows the phone
# Hours is a number already written out, such as 312.
more-stats-status = { $count ->
    [one] { $hours } hour watched in { $year }
   *[other] { $hours } hours watched in { $year }
}
more-stats-status-empty = A year of watching

## More: the sections' own settings

more-accounts-note = What you watch here marks your lists, and progress from them shows on your shows.
more-storage = Storage
more-storage-capped = Capped at { $cap }
more-storage-no-cap = No cap
# Under a figure such as 4.2 GB.
more-storage-used = used by episodes
# Under the cap's figure, such as 18 GB.
more-storage-cap-caption = cap
more-storage-free = free on the phone
more-set-storage-cap = Storage cap
more-storage-cap-on = At { $cap }, nothing more downloads until you delete episodes.
more-storage-cap-off = Off: AniBeam downloads until the phone is full.
more-network = Network
more-network-wifi = Wi-Fi only
more-network-mobile = Wi-Fi and mobile data
more-wifi-on = On. Downloads wait for Wi-Fi; mobile data asks first.
more-set-stream-quality = Stream quality
more-stream-question = How an episode from your computer plays here.
more-stream-auto-note = Auto plays the file as it is on Wi-Fi, and converted to 720p on mobile data or when Tailscale reaches your computer through a relay.
more-stream-original-note = The file as it is, every time. Big files need a fast link.
more-stream-converted-note = Converted on your computer to { $quality }. Uses less data; seeking takes a moment.
more-set-feeds = Your own feeds
more-feeds-note = Searched before the sites
more-set-check = Check the connection
more-check-note = Tests each step between this phone and the search sites: finding them, reaching them, and a secure connection.
more-check-copy = Copy the results
more-set-computer = Your computer's AniBeam
# my-computer stands for a computer's name; translate it like a placeholder. The numbers are an address.
more-computer-note = Its library shows in Library, and its episodes play here. A name like my-computer or an address like 100.64.0.1.
more-computer-placeholder = my-computer:8790
more-subs-note = What every episode on this phone starts with. One streamed from your computer starts with that computer's settings.
more-icon-note = The icon on your home screen and in the app drawer.
more-icon-line-note = Line art takes your wallpaper's colours, like your other Material You icons.
more-icon-moved-note = Some launchers take the icon off the home screen when it changes. It stays in the app drawer; drag it back from there.
more-set-whats-new = What's new
more-whats-new-note = What changed in each version, newest first.
more-set-onboarding = Set up again
more-onboarding-note = Walk through the first-run questions once more.
more-set-version = Version

## Status lines Rust writes on the pages

# When is a time such as "17:56" or "Sat 25 Oct 14:00".
host-synced = Synced { $when }.
host-last-try = Last try { $when }: { $error }
host-asking = Asking...
host-reached = Reached { $host }, AniBeam { $version }.
host-saved = Saved. Syncing...
acct-signed-in-as = Signed in as { $name }
acct-signed-in = Signed in
acct-not-in-build = Not available in this build
acct-tap-to-sign-in = Not signed in. Tap to sign in.
acct-sign-in-failed = Sign-in failed: { $why }. Tap to try again.
acct-sign-in-not-started = Sign-in did not start: { $error }. Tap to try again.
acct-still-starting = AniBeam is still starting. Try again in a moment.
home-recently-watched = Recently watched
home-recently-downloaded = Recently downloaded
home-note-empty = On this phone
home-note = { $count ->
    [one] On this phone, 1 show
   *[other] On this phone, { $count } shows
}
lib-this-phone = This phone
lib-note-phone = { $count ->
    [one] This phone, 1 show
   *[other] This phone, { $count } shows
}
# Device is the computer's name; when is how long ago, such as "3h ago".
lib-note-remote = { $device }, synced { $when }
lib-empty-search = Nothing matches "{ $query }".
lib-empty-phone = Nothing on this phone yet.
lib-empty-remote = That device has nothing in its library.
cal-view-airing = Airing now
cal-note-none = Nothing airs this week
cal-note = { $count ->
    [one] 1 episode in the next 7 days
   *[other] { $count } episodes in the next 7 days
}
airing-asking = Asking AniList
airing-asking-newest = Asking AniList for the newest
# Filter chips: the word, then how many shows it holds.
airing-filter-all = Everything { $count }
airing-filter-listed = On my lists { $count }
airing-filter-owned = On this phone { $count }
airing-this-season = This season
airing-next-season = Next season
airing-other-season = Other
airing-hidden = { $count ->
    *[other] { $count } hidden as not interested
}
airing-plan-needs-tracker = Plan to watch needs AniList or MAL. Sign in under More, Accounts.
airing-plan-failed = Plan to watch did not go through: { $error }
# A poster's flag: where the show is on the person's list.
list-watching = Watching
list-planning = Planning
list-completed = Completed
list-paused = On hold
list-dropped = Dropped
list-repeating = Rewatching
more-check-all-ok = Every step went through.
more-check-failed = { $failed } of { $total } steps failed. A screenshot of this list says where.
# A step of the connection check and how long it took.
more-check-row = { $step }  { $ms } ms
more-check-copied = Copied. Paste it in a message.
more-version = AniBeam for Android { $version } (build { $build })
dl-note-waiting = Waiting for Wi-Fi. Downloads go on by themselves when the phone is on Wi-Fi.
dl-note-nothing = Nothing downloading.
dl-state-done = Done
dl-state-waiting = Waiting for Wi-Fi
dl-state-paused = Paused
dl-state-peers = Looking for peers
dl-state-checking = Checking
dl-state-error = Stopped by an error
dl-state-downloading = Downloading
dl-amount = { $got } of { $size }
dl-speed = { $rate }/s
# Span is a short time such as "2h 05m".
dl-left = { $span } left
dl-episodes-range = Episodes { $from } to { $to }
dl-batch = A batch
disk-storage-line = { $used } used
disk-storage-line-free = { $used } used, { $free } free
disk-free = This phone has { $free } free.
disk-free-below-cap = This phone has { $free } free, less than the cap: the phone fills up before AniBeam stops.

## More's search: the words a person might type for each setting

more-set-subs-size = Subtitle size and place
more-set-subs-style = Subtitle style
more-words-anilist = anilist tracker sign in login account connect list progress
more-words-mal = mal myanimelist tracker sign in login account connect list progress
more-words-storage-cap = space disk limit gb full storage delete size
more-words-wifi = wifi wi-fi mobile data cellular network
more-words-notifications = notification notify alert progress permission shade allow
more-words-stream-quality = auto original 1080p 720p 480p convert data computer play
more-words-check-repos = refresh update repo endpoint
more-words-add-repo = repo endpoint trust fingerprint address url
more-words-add-site = custom endpoint feed rss torznab jackett prowlarr address
more-words-feeds = feed rss source remove
more-words-check = network dns test https secure reach internet
more-words-computer = tailscale tailnet host address desktop library stream test save device
more-words-whats-new = changelog patch notes changes update
more-words-onboarding = onboarding first run start over
more-words-version = build about
more-words-ratings = imdb credit scores ratings
more-words-stats = hours episodes genres studios seasons year watched recap busiest month history
more-words-backup = backup export file save copy move computer json private sign-ins
more-words-restore = restore import file load library merge
more-words-cache = images posters banners clear storage space cache
more-words-activity = log events errors problems copy clear history what happened
more-words-colours = appearance theme dark light mode system accent colour color
more-words-text-size = appearance font bigger smaller larger size words read
more-words-icon = icon launcher home screen logo dark line art themed material you
more-words-subs-size = subtitles captions size scale position lift bottom
more-words-subs-style = subtitles captions font colour color outline shadow box bold ass styled reset
more-words-language = language translation english locale

## Home

home-title = Home
home-next-to-air = Next to air
home-after-that = After that
home-coming-up = Coming up
home-nothing-week = Nothing in the next week.
home-empty-title = Nothing on this phone yet
home-empty-body = Shows you download here, and what airs for them, show up on Home. Find one in Downloads.

## Library

lib-title = Library
lib-refresh = Refresh
lib-search-phone = Search this phone
# { $device } is the name of another AniBeam the library can show, a computer.
lib-search-device = Search { $device }
# { $query } is what the person typed; the button runs the same search in Downloads.
lib-find-in-downloads = Find "{ $query }" in Downloads
lib-empty-body = Episodes you download on this phone land here. Find a show in Downloads.
lib-pick-show = Pick a show to open it here.
# The caption under a number: "12" then "shows".
lib-shows-caption = { $count ->
    [one] show
   *[other] shows
}
# The caption under the space the downloads take, "3.2 GB" then "on the phone".
lib-on-phone = on the phone
# The caption under the free space, "40 GB" then "free".
lib-free = free
lib-hold-poster = Hold a poster to get more episodes or delete them.
# Per-device counts under a show tile, "3 on phone".
phone-nothing-on-phone = nothing on phone
phone-on-phone = { $count } on phone

## Calendar

cal-title = Calendar
cal-week = Week
cal-days = Days
# Date patterns in chrono's strftime form: %a short weekday, %A weekday, %-d day of the
# month, %B month. Reorder them and add the language's own words or marks around them.
cal-weekday-short = %a
cal-weekday = %A
cal-day-full = %A %-d %B
cal-today = Today
cal-tomorrow = Tomorrow
cal-yesterday = Yesterday
cal-nothing = Nothing
# A badge on the next episode to air.
cal-next = Next
# A button on an episode that aired and is not on the phone: go get it.
cal-get = Get
# The heading note of a day: { $date } is "Thursday 13 November", { $count } is "2 episodes".
cal-day-note = { $date }, { $count }
# { $title } is a show's title; the button jumps to the day it airs next.
cal-next-airing = Next airing: { $title }
cal-nothing-aired-yesterday = Nothing aired yesterday.
cal-nothing-aired-that-day = Nothing aired that day.
cal-nothing-today = Nothing airs today.
cal-nothing-tomorrow = Nothing airs tomorrow.
cal-nothing-that-day = Nothing airs that day.
cal-empty-title = No shows on this phone yet
cal-empty-body = Episodes of the shows you keep here show up on their day, two days back and a week ahead.
# Lines under an episode on Home and in the calendar.
phone-episodes-range = Episodes { $from } to { $to }
# { $episode } is "Episode 5", { $day } is "Today" or a weekday.
phone-episode-day = { $episode }, { $day }
phone-on-the-phone = On the phone
# { $ago } is "3h ago".
phone-out-ago = Out { $ago }

## Airing now

airing-out-now = Out now
airing-last-two-days = The last two days
airing-season-note = The season's shows come from AniList.
airing-none-lists-title = Nothing on air is on your lists
airing-none-lists-body = Plan to watch on a show puts it on your list.
airing-none-phone-title = Nothing on air is on this phone
airing-none-phone-body = Get episodes on a show downloads it here.
# { $title } is a show's title.
airing-marked = { $title } is marked not interested.
airing-undo = Undo
airing-plan = Plan to watch
airing-get-more = Get more episodes
airing-get = Get episodes
airing-not-interested = Not interested

## Seasons

season-winter = Winter
season-spring = Spring
season-summer = Summer
season-fall = Fall
# { $season } is a season's name above, { $year } the year: "Spring 2027".
season-named = { $season } { $year }
season-pick = Pick a season

## A show's page

series-back-home = Home
series-back-library = Library
series-back-downloads = Downloads
series-back-calendar = Calendar
series-get-episodes = Get episodes
series-episodes = Episodes
series-delete-episodes = Delete episodes
series-delete-tap = Tap the episodes to delete.
series-delete-picked = { $count ->
    [one] { $count } episode picked.
   *[other] { $count } episodes picked.
}
series-delete-note = Their files are deleted from this phone. What you watched stays tracked.
series-delete-button = { $count ->
    [one] Delete { $count } episode
   *[other] Delete { $count } episodes
}
series-no-episodes = No episodes on this phone and none scheduled.
# The chip on an episode kept in several releases.
series-versions = { $count ->
    [one] { $count } version
   *[other] { $count } versions
}
# Notes under an episode's title. The next-up and missing words are also compared by the page.
series-note-next-up = Next up
series-note-missing = Missing
series-note-not-here = Not on this phone
series-note-not-downloaded = Not downloaded
# { $percent } is a number from 0 to 100.
series-note-downloading = Downloading { $percent }%
# { $time } is a position in the episode, "12:30".
series-note-resume-at = Resume at { $time }
# The facts line and the lines under the title of a show.
phone-episodes-estimate = { $count ->
    [one] { $count }? episode
   *[other] { $count }? episodes
}
# { $n } is an episode number, { $time } a countdown such as "3d 02h 15m 07s".
phone-next-airs = Episode { $n } airs in { $time }
phone-watched = Watched
phone-not-watched = Not watched yet
# { $watched } and { $total } are episode counts, { $count } is how many are on this phone.
phone-progress = { $watched } of { $total } watched, { $count } on this phone
phone-on-this-phone = { $count } on this phone
# The main button of a show: { $n } is an episode number.
phone-play-film = Play the film
phone-resume-film = Resume the film
phone-play-episode = Play episode { $n }
phone-resume-episode = Resume episode { $n }
# Used for an upcoming episode whose date is a guess, { $date } is "3 Nov".
phone-about-date = about { $date }

## A show's list status, score, notes and fixes

show-status-watching = Watching
show-status-planning = Planning
show-status-completed = Completed
show-status-paused = On hold
show-status-dropped = Dropped
show-status-repeating = Rewatching
# A button for a list the show is on already. { $label } is a status above.
show-status-now = { $label }, now
# The chip that opens the list sheet when the show is on no list.
show-add-to-list = Add to a list
# The chip that opens the score sheet. { $score } is "8.5".
show-your-score = Your score { $score }
show-rate = Rate
show-fixes = Fixes
show-your-notes = Your notes
show-add-notes = Add notes
# Under the list sheet's title: where a status or score goes.
show-home-each = Change it on each list that is connected.
show-home-no-tracker = Kept on this phone: no tracker carries this show.
show-home-tracker-lacks = Kept on this phone: your tracker does not carry this show.
show-home-until-connected = Kept on this phone until you connect AniList or MyAnimeList.
# The trackers a note is saved to, when both are connected.
show-sites-both = AniList and MyAnimeList
show-notes-title = Notes
show-notes-placeholder = Where you left off, who recommended it, which release to get
# { $count } is how many characters past the limit.
show-notes-over = { $count ->
    [one] { $count } character over the limit
   *[other] { $count } characters over the limit
}
# { $sites } is "AniList", "MyAnimeList" or both.
show-notes-saved-to = Saved to { $sites }.
show-notes-saved-to-tracker = Saved to your tracker.
show-notes-saving = Saving your notes.
show-notes-clearing = Clearing your notes.
show-score-saving = Saving your score.
show-score-saved = Score saved.
show-score-clearing = Clearing your score.
show-score-cleared = Score cleared.
# { $label } is a status such as "On hold".
show-moving-to = Moving to { $label }.
show-moved-to = Moved to { $label }.
show-rescan-started = Rescan started.
show-still-starting = AniBeam is still starting. Try again in a moment.
show-fix-match = Match
show-fix-find-match = Find a match
show-fix-rescan = Rescan folder

## A show's characters, related shows and recommendations

show-characters = Characters
show-related = Related
# The button under the first dozen related shows. { $count } is how many there are.
show-related-all = Show all { $count }
show-related-filling = Still filling in from AniList.
show-recommendations = Recommendations
# The line under a related or recommended show.
show-card-owned = In your library
show-card-unreleased = Not yet released
show-card-on-anilist = On AniList
# The badge on a recommended show this phone has.
show-rec-available = Available
# What a related show is to this one.
show-relation-prequel = Prequel
show-relation-sequel = Sequel
show-relation-side-story = Side story
show-relation-parent = Parent
show-relation-spin-off = Spin off
show-relation-adaptation = Adaptation
show-relation-alternative = Alternative
show-relation-summary = Summary
show-relation-character = Character
show-relation-source = Source
show-relation-compilation = Compilation
show-relation-contains = Contains
show-relation-other = Other
# A character's role in the show.
show-role-main = Main
show-role-supporting = Supporting
show-role-background = Background
# A show's format, as the line under a related show or a match result names it.
show-format-series = Series
show-format-tv = TV
show-format-tv-short = TV short
show-format-ova = OVA
show-format-ona = ONA
show-format-special = Special
show-format-film = Film
show-entry-get = Get episodes
show-entry-open-anilist = Open on AniList
# The sheet's second line: { $line } is "Sequel, TV, 2024", { $status } a list status.
show-entry-line = { $line }, { $status }

## Fix a wrong match

match-title = Match
match-refresh = Refresh info
match-clear = Clear match
match-clear-it = Clear it
match-clear-warning = Clearing drops the tracker link and a hand-set episode count. Tap again to clear.
match-tvmaze = TVmaze
match-search-anilist = Search AniList
match-search-tvmaze = Search TVmaze
match-paste-anilist = Paste an AniList or MyAnimeList link
match-paste-tvmaze = Paste a TVmaze or IMDb link
match-look-up = Look up
match-apply = Apply
match-look-up-all = Look up every folder AniBeam does not know yet
match-format-music = Music
# { $first } and { $second } are names of sites: "AniList and MyAnimeList".
match-join-and = { $first } and { $second }
# { $sites } is one or more site names.
match-matched-on = Matched on { $sites }
match-found-tvmaze = Found the show on TVmaze
match-found-anilist = Found it on AniList
# { $season } is a season number.
match-found-anilist-season = Found it on AniList, season { $season }
match-found-mal = Found it on MyAnimeList
# { $name } is a site: "AniList" or "TVmaze".
match-searching = Searching { $name }
match-type-more = Type at least two letters to search { $name }.
match-none = No titles found on { $name }.
match-looking-up = Looking it up.
# { $title } is a show's title.
match-matched-to = Matched to { $title }.
match-applied = Match applied.
match-applying = Applying the match.
match-refreshing = Refreshing { $title }.
match-clearing = Clearing the match.
match-cleared = Match cleared.
match-auto-looking = Looking up the folders AniBeam does not know yet.

## Batches (the line under a release in a search)

inspect-reading = Reading its file list
# { $count } is how many episodes the release holds, { $runs } is "1-12, 14".
inspect-holds = { $count ->
    [one] { $count } episode in it: { $runs }
   *[other] { $count } episodes in it: { $runs }
}

## What's new

notes-title = What's new
notes-got-it = Got it

## Move the list to a tracker

move-lists-label = Lists on this phone
# { $site } is the tracker: "AniList" or "MyAnimeList".
move-title = Move your list to { $site }
# { $list } is "Watching, score 8.0".
move-here = On this phone: { $list }
move-there = On { $site } now: { $list }
move-later-hint = Later: More, Accounts
move-not-now = Not now
move-nothing = Nothing
move-not-on-list = Not on your list
move-nothing-left = Nothing left to move
# { $status } is "On hold", { $score } is "8.0".
move-side-status-score = { $status }, score { $score }
move-side-score = score { $score }
move-lead = { $count ->
    [one] One show has a status or a score only on this phone. Moving them puts them on your { $site } list, where your computer and your profile see them. For these shows, what { $site } has now is replaced. Nothing is sent until you press Move.
   *[other] { $count } shows have a status or a score only on this phone. Moving them puts them on your { $site } list, where your computer and your profile see them. For these shows, what { $site } has now is replaced. Nothing is sent until you press Move.
}
move-label = { $count ->
    [one] Move { $count } show
   *[other] Move { $count } shows
}
move-waiting = { $count ->
    [one] One show has a status or a score only here. Move them to { $site } to see them on your computer and your profile.
   *[other] { $count } shows have a status or a score only here. Move them to { $site } to see them on your computer and your profile.
}
move-moving = { $count ->
    [one] Moving { $count } show to { $site }
   *[other] Moving { $count } shows to { $site }
}
move-moved = { $count ->
    [one] Moved { $count } show to { $site }
   *[other] Moved { $count } shows to { $site }
}
move-failed = { $count ->
    [one] { $site } did not take { $count } show; see the activity log
   *[other] { $site } did not take { $count } shows; see the activity log
}

## Downloads: find (search, shows, sources, releases)

# Joins. Each language picks its own comma, conjunction and sentence gap.
find-join-comma = { $a }, { $b }
find-join-and = { $a } and { $b }
find-join-sentences = { $a } { $b }

# Page title of the back link in the show view and the release list.
find-downloads = Downloads

# The five lens chips under the field, and the line under each when it has shows.
find-lens-list = Your list
find-lens-gaps = Gaps
find-lens-airing = Airing
find-lens-following = Following
find-lens-downloaded = Downloaded
# A lens chip with how many shows it holds: { $lens } is the chip's name, { $count } a number.
find-lens-with-count = { $lens } { $count }
find-what-to-get = What to get
find-lens-list-line = Watching and Planning on your list. Open one to get its episodes.
find-lens-gaps-line = Aired episodes that are not on this phone. Open a show to see who has them.
find-lens-airing-line = New episodes still to come, the soonest first.
find-lens-following-line = The subscriptions: new episodes download by themselves. Open one to stop it.
find-lens-downloaded-line = What this phone downloaded, a show at a time.

# The line under a lens when it has nothing to show.
find-lens-gaps-empty = No gaps: every show you have started has its aired episodes here.
find-lens-airing-empty = Nothing on this phone is airing.
find-lens-following-empty = No subscriptions yet. Open a show and turn on Follow.
find-lens-downloaded-empty = Nothing downloaded yet.
find-list-empty = Nothing on your list yet. Search a show above and add it.
find-list-reading = Reading your list...
find-list-signed-out = Sign in to AniList under More, Accounts, and your list shows up here.
# { $error } is the reason the core gave.
find-list-failed = Your list could not be read: { $error }

# Small line on a show tile.
find-aired = aired
find-ledger-no-feed = no new episodes: its source has no feed
find-ledger-downloads = { $count ->
    [one] { $count } download
   *[other] { $count } downloads
}
find-ledger-downloaded = downloaded
find-ledger-planning = Planning
# { $progress } episodes seen out of { $total }.
find-ledger-watching-of = Watching, { $progress } of { $total }
find-ledger-watching-seen = Watching, { $progress } seen

# Gaps lens: what a show is missing, each part a count.
find-gap-missing = { $count } missing
find-gap-not-downloaded = { $count } not downloaded

# Where the open show is on the person's list.
find-list-status-planning = On your list: Planning
find-list-status-watching = On your list: Watching
find-list-status-already = Already on your list
find-going-planning = { $title } is going on Planning.
find-already-listed = { $title } is already on your list.
find-plan-needs-tracker = Adding to your list needs AniList or MAL. Sign in under More, Accounts.
find-plan-failed = Not added to your list: { $error }
find-show-gone = That show is no longer in the library on this phone.

## Shows as you type

find-shows = Shows
find-suggesting = Asking AniList...
find-suggest-tap = Tap one to see who has it. Return searches release names.
find-suggest-none = No show by that name. Return searches release names instead.
find-suggest-hint = Shows come up as you type. Return searches release names.
# { $error } is why the lookup failed.
find-suggest-failed = AniList could not be asked: { $error }
find-library-line = In your library
find-library-line-none = In your library, none of { $total } on this phone
find-library-line-some = In your library, { $here } of { $total } on this phone

## The show view

find-plan-to-watch = Plan to watch
find-on-phone-count = { $count } on this phone
find-airing = Airing
find-dropped = { $count ->
    [one] { $count } release that does not name this show is left out.
   *[other] { $count } releases that do not name this show are left out.
}

# The Following card and its confirm.
find-following = Following
# { $name } is the subscription's name.
find-stop-following-ask = Stop following { $name }?
find-unfollow-explain = New episodes stop downloading. What is in its folder can stay, or go with it: every download there, episodes you got by hand included.
find-stop-following = Stop following
find-stop-keep = Stop, keep the episodes
find-stop-delete = Stop and delete its folder's downloads
find-rule-taken = { $downloads ->
    [one] { $downloads } download
   *[other] { $downloads } downloads
}, { $files ->
    [one] { $files } file
   *[other] { $files } files
} in its folder
# { $taken } is the folder's contents, such as "3 downloads, 6 files in its folder".
find-rule-paused = It is paused, so no new episodes come. { $taken }.
find-rule-no-feed = Its source has no feed, so no new episodes come. { $taken }.
find-rule-active = New episodes download by themselves. { $taken }.
find-unfollowed-deleted = Stopped following { $name } and deleted the downloads in its folder.
find-unfollowed-kept = Stopped following { $name }. Its episodes stay on this phone.
find-stop-failed = Stopping did not finish: { $error }

# Where to get it: the sources.
find-where = Where to get it
find-where-pick = Pick a source to see what it has.
find-where-nothing = Nothing downloads until you say so.
find-source-idle = Tap to ask
# { $name } is another name for the show being tried.
find-source-trying = Trying { $name }
find-source-searching = Searching
find-source-silent = Not answering
find-no-sites-title = No search sites yet
find-no-sites-body = Add a repo of search sites, or a feed of your own, in More.
find-open-more = Open More
find-pick-source-title = Pick a source
find-pick-source-body = Its releases show up here as a grid, an episode to a cube.
# { $count } is a number of releases.
find-releases = { $count ->
    [one] { $count } release
   *[other] { $count } releases
}
# { $host } is the source's name.
find-unreachable = { $host } could not be reached. Some networks and countries block release sites; AniBeam already gets past blocks on names, so this one blocks the site itself, or the site is down. A VPN gets past it, or try another source.
find-the-source = The source

# What a source has: the grid of releases.
find-has = What it has
find-has-note = Tap a row to take it all, or single episodes.
# The tag on a row that is one episode, beside the Batch tag of a row of many.
find-episodes-tag = Episodes
# { $label } is the episodes a batch covers, such as 1-12.
find-batch = Batch { $label }
# Row facts: { $count } episodes, { $seeds } seeders, { $size } file size. Keep the two-space gaps.
find-row-meta = { common-episodes }  { $seeds } seeds  { $size }
find-on-this-phone = On this phone
# { $name } is the connected computer's name.
find-on-computer = On { $name }

# The plan under the grid.
find-plan-no-releases = This source has no releases of this show.
find-plan-nothing-picked = Nothing picked: tap a row or its cubes.
# { $episodes } such as "episodes 1-3", { $groups } the release groups, { $size } the total size.
find-plan-line = { $episodes } from { $groups }, { $size }
find-plan-skipped = { $count ->
    [one] { $count } episode you already have is left out. Turn on Replace to get it again.
   *[other] Episodes you already have are left out. Turn on Replace to get them again.
}
# { $query } is the search that will be followed.
find-plan-follow-note = New episodes from { $query } will download by themselves.
find-plan-on-desk = They download on { $name }, into its library.
find-follow-new = Follow new episodes
find-follow-new-note = They download by themselves as they air.
find-replace = Replace what you have
find-replace-note = Downloads the picked episodes again over the ones on this phone.
# The main button. { $mode } is replace, subscribe, both or plain; { $count } the picked episodes.
find-go = { $mode ->
    [replace] Get { common-episodes } and replace
    [subscribe] Get { common-episodes } and subscribe
    [both] Get { common-episodes } and replace and subscribe
   *[plain] Get { common-episodes }
}
find-go-pick = Pick episodes to get
find-every-release = Every release from this source

## Download to

find-download-to = Download to
find-this-phone = This phone
find-your-computer = Your computer
# { $name } is the connected computer. "This phone" is the other choice in Download to.
find-desk-asking = Asking { $name }...
find-desk-silent = { $name } does not answer. Pick This phone, or try again when it is on.
find-sending-to = Sending to { $name }...

## The release list

find-best-fit = Best fit
find-on-disk = On disk
find-get = Get
find-get-all = Get all { $count }
find-subscribe = Subscribe
find-back-step = Back a step
find-narrow-placeholder = Narrow, or -word to leave out
find-best-fit-searches = Best fit searches: { $query }
find-fact-batch = batch
find-fact-seeds = { $count } seeds
# { $source } is the site a release came from.
find-fact-via = via { $source }
# { $text } is what the person typed.
find-searching-for = Searching for { $text }...
# { $count } releases found on { $source }; { $fell } and { $narrowed } are 1 or 0, { $token } the quality it narrowed to.
find-search-line = { $count ->
    [0] Nothing on { $source } for that
    [one] { $count } release on { $source }
   *[other] { $count } releases on { $source }
}{ $fell ->
    [1] , after the first source had nothing
   *[other] { "" }
}{ $narrowed ->
    [1] , narrowed to { $token }
   *[other] { "" }
}.
# { $dir } is down or up, { $delta } how many releases the step changed the list by.
find-refined = { find-releases }, { $dir ->
    [down] down
   *[up] up
} { $delta }.
find-refine-nothing-left = That leaves nothing.
find-refine-no-match = No release has that.
find-refine-empty = Type something first.
find-refine-no-change = That changes nothing.

## Getting and subscribing

# { $what } is what was sent, such as "3 episodes".
find-getting = Getting { $what }.
find-subscribed = Subscribed: new episodes download by themselves.
find-subscribed-desk = Subscribed on { $name }: new episodes download there by themselves.
find-subscribed-as = Subscribed as { $name }: new episodes download by themselves.
find-downloading = { $count ->
    [one] Downloading { $count } release.
   *[other] Downloading { $count } releases.
}
find-not-added = Not added: { $error }
find-not-subscribed = Not subscribed: { $error }
find-desk-refused = { $name } did not take it: { $error }
find-desk-not-subscribed = Not subscribed on { $name }: { $error }

## Player

player-subtitles = Subtitles
player-audio = Audio
player-speed = Speed
player-connecting = Connecting
# The 2x speed pill shown while the screen is held down.
player-hold-speed = 2× speed
player-replay = Replay
player-unlock = Unlock
# Button on the auto-next tile: stay on this episode.
player-next-stay = Stay
# Button on the auto-next tile: go to the next episode.
player-next-go = Next
# Shown on the picture after a double tap on the left side: ten seconds back.
player-jump-back = −10 s
# Shown on the picture after a double tap on the right side: ten seconds forward.
player-jump-forward = +10 s
# Beside the clock while the speed is not 1×. { $speed } is like 1.5×.
player-speed-now = Speed { $speed }
player-skip-intro = Skip intro
player-skip-outro = Skip outro
player-skipped-intro = Skipped intro
player-skipped-outro = Skipped outro
# Notice button that undoes a skip.
player-undo = Undo
# Notice button that goes back to the start of the episode.
player-start-over = Start over
# { $time } is a clock like 12:34.
player-resumed-at = Resumed at { $time }
# Shown when the picture will not open but the sound plays.
player-picture-failed = The picture could not start. The sound plays on.
# { $why } is mpv's own reason, in English.
player-file-failed = This file did not play: { $why }.
# { $host } is the other device's name, { $height } the video height in pixels, like 720.
player-streaming-from = Streaming from { $host } at { $height }p
player-mpv-no-start = mpv did not start
# { $why } is mpv's own reason, in English.
player-mpv-no-start-why = mpv did not start: { $why }
player-android-only = the player runs on Android only

## Player: delays

player-delay-title = Delay
player-delay-note = Each tap moves it 0.1 s. Until the next file.
# { $value } is a signed number of seconds, like +0.1 or -0.25.
player-delay-seconds = { $value } s
# The notice after a tap on the delay sheet. { $value } is like +0.1 s.
player-sub-delay-now = Subtitle delay { $value }
player-audio-delay-now = Audio delay { $value }

## Player: cast

# { $device } is the TV's or speaker's name.
player-cast-playing-on = Playing on { $device }
# Row in the Cast to sheet, ends the cast.
player-cast-stop = Stop casting to { $device }
# Sheet title while no device has answered yet.
player-cast-searching = Looking for Chromecasts and TVs
player-cast-to = Cast to
# Kind of device in a Cast to row, for a TV that is not a Chromecast.
player-cast-tv = TV
# What the TV is doing, on the cast screen.
player-cast-loading = Loading
player-cast-playing = Playing
player-cast-paused = Paused
player-cast-finished = Finished
player-cast-stopped = Stopped

## Player: the tracker's word

player-rated = Rated
player-score-failed = Could not save the score
player-marked-watched = Marked watched
player-tracked = Tracked
player-not-tracked = Not tracked
player-track-update-failed = Could not update your list
# { $reason } is one of the player-refusal lines.
player-not-tracked-why = Not tracked: { $reason }.
# { $tracker } is AniList or MyAnimeList, { $progress } the episode number.
player-track-ok-at = { $tracker } at { $progress }
player-track-ok = { $tracker } ok
# { $note } is the reason the tracker gave.
player-track-note = { $tracker }: { $note }
player-track-failed = { $tracker } could not be updated
# The whole line. { $head } is Tracked, Not tracked or Could not update your list; { $parts } is the list above joined by semicolons.
player-track-line = { $head }: { $parts }
player-refusal-hidden = the show is hidden
player-refusal-no-match = the show has no tracker match
player-refusal-not-newer = the tracker is already past this episode
player-refusal-extra = this is an extra
player-refusal-unmatched = the file is not matched to an episode
player-refusal-on-disk = the file is only on disk

## Subtitles

subs-line-default = Size 1×, plain files take your style
# The sample line in the preview frame.
subs-sample = Where did you hide it?
subs-size-place-title = Size and place
subs-size = Size
subs-position = Position
subs-position-helper = 100 sits on the bottom edge. Lower numbers lift the lines up the frame.
subs-style-title = Style
subs-styled = Styled subtitles
subs-styled-as-made = Styled (ASS) files look exactly as their makers set them.
subs-styled-mine = Your style below goes on every file. Signs and karaoke can break.
subs-styled-size-only = Styled subtitle files keep their look and take your size. Plain files take your style.
# The three choices of the Styled subtitles switch.
subs-mode-as-made = As made
subs-mode-size-only = Size only
subs-mode-mine = Mine
subs-font = Font
subs-font-bad = Type a font name, such as Noto Sans.
subs-bold = Bold
subs-colour = Text colour
subs-colour-bad = Type a colour like #ffcc00.
subs-outline = Outline
subs-outline-bad-width = The width is a number, 0 or more.
subs-outline-bad-colour = Type the colour like #000000.
subs-outline-helper = Its width, then its colour.
subs-shadow = Shadow
subs-shadow-bad = A number, 0 or more. 0 means no shadow.
subs-shadow-helper = How far the shadow falls, 0 for none.
subs-box = Background box
subs-box-helper = A dark box behind the lines. 0 leaves it out.
# Heading of the row whose button puts every subtitle setting back.
subs-reset-title = Start over
subs-reset-helper = Puts the size, font, colours, outline, box and position back to how they came.
subs-reset = Reset

## Streaming

stream-quality-auto = Auto
stream-quality-original = Original
stream-app-starting = The app is still starting
# { $host } is the other device's name. "Stream episodes", "Settings" and "Phone app" are names on its screen.
stream-off-on-host = Streaming is off on { $host }. Turn on Stream episodes there, under Settings, Phone app.
# { $host } is the other device's name, { $version } its AniBeam version.
stream-cannot-stream = { $host } runs AniBeam { $version }, which cannot stream yet. Update it there.
stream-no-session = The other device gave no session
# { $why } is the technical reason, in English.
stream-answer-unreadable = The answer did not read: { $why }
# { $name } is the other device's name, { $status } an HTTP status like 502.
stream-answered-status = { $name } answered { $status }

## Stats

stats-nothing-yet = Nothing yet
stats-empty-title = Nothing watched yet
stats-empty-body = Finish an episode and it counts here: hours, genres and the seasons you watched.
# Captions under the four big numbers.
stats-hours-caption = hours watched
stats-finished-caption = series finished
stats-busiest-caption = busiest month
# { $count } is the number of episodes watched; the caption reads under it.
stats-episodes-caption = { $count ->
    [one] episode
   *[other] episodes
}
stats-by-month = By month
stats-top-genres = Top genres
stats-top-studios = Top studios
stats-seasons = Seasons watched
stats-watched-most = Watched most
# The heading note of the page. { $year } is a year such as 2026.
stats-year-line = What you watched in { $year }
# The count at the right of a ranked row.
stats-eps = { $count ->
    [one] { $count } ep
   *[other] { $count } eps
}
# The month's short name, for the busiest-month figure.
stats-month-1 = Jan
stats-month-2 = Feb
stats-month-3 = Mar
stats-month-4 = Apr
stats-month-5 = May
stats-month-6 = Jun
stats-month-7 = Jul
stats-month-8 = Aug
stats-month-9 = Sep
stats-month-10 = Oct
stats-month-11 = Nov
stats-month-12 = Dec
# One character that stands for the month under its bar; two months may share one.
stats-month-letter-1 = J
stats-month-letter-2 = F
stats-month-letter-3 = M
stats-month-letter-4 = A
stats-month-letter-5 = M
stats-month-letter-6 = J
stats-month-letter-7 = J
stats-month-letter-8 = A
stats-month-letter-9 = S
stats-month-letter-10 = O
stats-month-letter-11 = N
stats-month-letter-12 = D

## Backup and image cache

backup-title = Backup
backup-export = Back up the library
backup-export-helper = One file with your folders, matches and what you have watched.
backup-private = Include private data
backup-private-helper = Adds AniList and MyAnimeList sign-ins, watch history and preferences. Sign-ins are written as plain text, so keep the file to yourself.
backup-restore = Restore from a backup
backup-restore-helper = Reads a backup into this library. What is on disk wins; the file fills in the rest.
backup-done = Backed up. The file is where you saved it.
backup-not-saved = Not saved. Nothing was written.
backup-starting = AniBeam is still starting. Try again in a moment.
backup-images-title = Image cache
backup-images-body = Posters, banners and portraits are kept on the phone so the library works offline. Cleared images download again as pages need them.
# Button that answers "Remove 340 images?" by leaving the cache alone.
backup-keep = Keep
backup-removed = { $count ->
    [one] Removed { $count } image
   *[other] Removed { $count } images
}
backup-images-none = No images cached
backup-images-unmeasured = The image cache could not be measured
backup-images-retry-note = Try again in a moment. The images themselves are fine.
# { $size } is a size such as 12.3 MB.
backup-images-line = { $size } in { $count ->
    [one] { $count } image
   *[other] { $count } images
}
backup-images-question = { $count ->
    [one] Remove { $count } image?
   *[other] Remove { $count } images?
}

# What a restore did. The clauses below are strung together, then end with a full stop.
# { $items } is the clauses joined by backup-sum-sep.
backup-sum-list = { $items }.
# What goes between two clauses, and between the names of fields left out.
backup-sum-sep = { ", " }
backup-sum-folders = { $count ->
    [one] { $count } folder added
   *[other] { $count } folders added
}
backup-sum-series = { $count } series created
backup-sum-matches = { $count ->
    [one] { $count } match applied
   *[other] { $count } matches applied
}
# { $views } watch records and { $episodes } finished episodes were merged.
backup-sum-views = { $views ->
    [one] { $views } view
   *[other] { $views } views
} and { $episodes ->
    [one] { $episodes } finished episode
   *[other] { $episodes } finished episodes
} merged
backup-sum-resume = { $count ->
    [one] { $count } resume point merged
   *[other] { $count } resume points merged
}
backup-sum-accounts = { $count ->
    [one] { $count } list account brought in
   *[other] { $count } list accounts brought in
}
# Sentences added after the list when they apply.
backup-sum-fixes = { $count ->
    [one] { $count } file takes a fix you made.
   *[other] { $count } files take a fix you made.
}
backup-sum-missing = { $count ->
    [one] { $count } folder was not found on this phone.
   *[other] { $count } folders were not found on this phone.
}
# { $fields } are names of settings the backup held that this phone does not read.
backup-sum-left-out = Left out: { $fields }.

## Activity

# The two filters of the log, side by side.
activity-everything = Everything
activity-problems = Problems
# The Copy button reads this for a moment after it worked.
activity-copied = Copied
# The Clear button after a first tap, asking for a second.
activity-clear-every-line = Clear every line
# Button on the first line of an opened run of identical lines: closes it again.
activity-fold = Fold
activity-no-problems = No problems. Everything worked.
activity-nothing-yet = Nothing to report yet. What AniBeam does for you shows up here.
# The time on a log line, as strftime patterns (see the date patterns above). Today's lines
# say the time; older ones say the day as well.
activity-time-today = %H:%M:%S
activity-time-other = %-d %b %H:%M

## Dates and numbers (companion format.rs)

# Date and time patterns. These are strftime patterns, not sentences: %b is the short month,
# %a the short weekday, %-d the day, %Y the year, %H:%M the 24-hour clock. The month and
# weekday names come out in the language by themselves; a language only reorders the pieces
# or adds its own marks (Japanese writes %b%-d日, for example).
fmt-clock = %H:%M
# "17 Oct"
fmt-day-month = %-d %b
# "Sat 25 Oct 14:00"
fmt-weekday-date-time = %a %-d %b %H:%M
# "Fri 15 Jan 2027 15:40"
fmt-weekday-date-year-time = %a %-d %b %Y %H:%M
# "Oct 17 14:02", a moment on the status line
fmt-stamp = %b %-d %H:%M
# The mark between whole and tenth: "1.4 GB" in English, "1,4 GB" where a comma is used.
fmt-decimal-point = .

# Air times. { $time } is a clock time such as 14:00.
fmt-today-at = Today { $time }
fmt-tomorrow-at = Tomorrow { $time }
fmt-yesterday-at = Yesterday { $time }

# Time left, short. d, h, m and s are days, hours, minutes and seconds.
fmt-span-days = { $d }d { $h }h
fmt-span-hours = { $h }h { $m }m
# { $s } is two digits: 05
fmt-span-minutes = { $m }m { $s }s
fmt-span-seconds = { $s }s
# A countdown to an air time. { $span } is one of the four above.
fmt-in = in { $span }
# The ticking form on a series page. Minutes and seconds are two digits.
fmt-full-days = { $d }d { $h }h { $m }m { $s }s
fmt-full-hours = { $h }h { $m }m { $s }s
fmt-full-minutes = { $m }m { $s }s

# How long ago the last sync was.
fmt-ago-now = just now
fmt-ago-minutes = { $count } min ago
fmt-ago-hours = { $count }h ago
fmt-ago-days = { $count }d ago

# A size. { $value } is the number: 1.4, 820.
fmt-bytes-b = { $value } B
fmt-bytes-kb = { $value } KB
fmt-bytes-mb = { $value } MB
fmt-bytes-gb = { $value } GB
fmt-bytes-tb = { $value } TB

## Talking to the computer (companion client.rs, sync.rs, address.rs)

# { $host } is the computer's name or address.
client-not-found = { $host } is not a name the phone can find. Is Tailscale connected?
client-timeout = { $host } did not answer. Is it on, and is Tailscale connected on both?
client-connect-failed = Could not reach { $host }. Is it on, and is Tailscale connected on both?
client-refused = { $host } is on, but AniBeam there is closed or not sharing. Open AniBeam on it and switch on Share over Tailscale in Settings, Phone app.
client-no-route = No route to { $host }. Is Tailscale connected?
client-invalid-input = Could not connect to { $host }
# { $why } is the system's own error text.
client-unreachable = Could not reach { $host }: { $why }
client-cannot-find = The phone cannot find { $host }. Is Tailscale connected?
client-broke-off = The answer from { $host } broke off: { $why }
# { $status } is an HTTP status such as 404, { $path } the address asked.
client-answered = { $host } answered { $status } for { $path }
client-not-anibeam-status = Something answered, but not AniBeam ({ $host } answered { $status } for { $path })
client-not-anibeam-hello = Something answered, but not AniBeam (its answer is not AniBeam's hello)
# { $name } is what the other program calls itself.
client-not-anibeam-name = Something answered, but not AniBeam (it calls itself { $name })
# API numbers: the computer's and this app's.
client-too-new = The host speaks API { $host_api }; this app reads { $app_api }. Update the app.
client-snapshot-unreadable = The snapshot did not read: { $why }
client-probe-unreadable = The file's details did not read: { $why }
client-peers-unreadable = The list of AniBeams did not read: { $why }
sync-save-failed = Could not save the snapshot: { $why }
address-https = Your computer's AniBeam answers on plain http over Tailscale; drop the https://.
address-empty = Type the host's name or address, for example my-computer:8790.
address-ipv6-close = An IPv6 address needs its closing ].
address-ipv6-port = Put the port after the ], as [fd7a::1]:8790.
address-bad-host = "{ $text }" is not a host name.
address-bad-port = "{ $port }" is not a port.

## Series facts (companion library.rs)

# The status and format chips on a series page.
clib-status-airing = Airing
clib-status-not-yet = Not yet aired
clib-format-tv-short = TV short
clib-format-movie = Movie
clib-format-special = Special
clib-format-music = Music

## Search sites and repos

repos-by-hand-title = Add one by hand
# { "{query}" } is the literal text {query}, where the search term goes in the address.
repos-by-hand-note = For a site no repo carries. Any feed with { "{query}" } where the search goes, a Jackett or Prowlarr feed among them. A key or token in the address is moved to this phone's secret store.
# Placeholder of a text field.
repos-name-placeholder = Name
# Placeholder of a text field. Keep the address as it is.
repos-url-placeholder = https://...{ "{query}" }...
# Placeholder of a text field. Keep the address as it is.
repos-index-placeholder = https://.../index.json
# Button
repos-add-site = Add site
# Heading, and the name the settings search finds this part by.
repos-add-title = Add a repo
repos-add-note = Paste the address of a repo's index. AniBeam checks its signature and shows you its key before anything is saved.
# Button
repos-add-repo = Add repo
repos-trust-title = Trust this repo?
repos-fingerprint = Key fingerprint
repos-trust-warning = Its sites decide where your searches go. Trust a repo only from someone you trust.
# Button
repos-trust = Trust
# Button that opens a page. { $url } is its address.
repos-open-link = Open { $url }
# Button label while it works
repos-saving = Saving...
# Buttons of the form that asks for an endpoint's values
repos-install = Install
repos-update = Update
repos-edit-note = A key or token typed into the address is moved to this phone's secret store.
repos-edit-note-keys = Leave each { "{key}" } in the address as it is; its value is kept on this phone.
# Button. { $keys } is what the source asks for, such as "API key".
repos-change-its = Change its { $keys }
# Button, and the name the settings search finds it by.
repos-check-repos = Check repos for new sites
# Button, and the name the settings search finds it by.
repos-check-sites = Check the sites
# Only the settings search uses this name, to light up the part for adding a site by hand.
repos-add-site-label = Add a site by hand
# { $labels } is what the endpoint asks for, such as "API key, token".
repos-needs-before-search = Needs its { $labels } before it searches
repos-no-longer-offered = No longer offered by its repo
# { $host } is the address of a site.
repos-update-host = The update sends requests to { $host }
repos-tap-change-custom = Tap to change its name or address
# { $labels } is what the endpoint asks for, such as "API key, token".
repos-tap-change-keys = Tap to change its { $labels }
# A line of small print under an endpoint's name. { $host } is its address and { $kind } its type, such as rss.
repos-line-custom = { $host }  { $kind }  added by you
# Button. { $version } is a number.
repos-update-to = Update to v{ $version }
# Button
repos-add-its-value = Add its value
# { $labels } is what the endpoint asks for, such as "API key, token".
repos-asks-for = Asks for your { $labels }
# Under a repo about to be trusted.
repos-preview-line = { $count ->
    [one] { $count } site, signed
   *[other] { $count } sites, signed
}
repos-preview-trusted = { $count ->
    [one] { $count } site, signed, already trusted
   *[other] { $count } sites, signed, already trusted
}
repos-ask-kept-empty = Kept on this phone only. A field left empty keeps what is stored.
repos-ask-kept-never = Kept on this phone only, never in the library or an export.
# { $host } is the address of a site.
repos-ask-new-host = The new version sends this to { $host }, so it asks again.
repos-ask-new = The new version asks for these again.
repos-type-value = Type a value first.
# { $ms } is a number of milliseconds.
repos-health-answered = Answered in { $ms } ms
# { $step } is what was tried, { $detail } why it failed.
repos-health-failed = Did not answer. { $step }: { $detail }

## Downloads

dl-title = Downloads
dl-wifi-only = Wi-Fi only
dl-wifi-and-mobile = Wi-Fi and mobile data
# Two things joined in one line: a state and the episode it is about, or an amount and a release.
dl-joined = { $first }, { $second }
# Chip on a download made of several episodes. { $count } is the number of files.
dl-batch-files = { $count ->
    [one] Batch, { $count } file
   *[other] Batch, { $count } files
}
# Chip on a download made of several episodes, with no count.
dl-batch-tag = Batch
# Figure caption on Downloads. { $count } is how many are moving.
dl-downloading-count = { $count ->
    [one] { $count } downloading
   *[other] { $count } downloading
}
# Caption under a figure of free space.
dl-free-on-phone = free on the phone
dl-search-placeholder = A show, or a release name
# Label before the chips of the search sites.
dl-sources = Sources
# Section heading
dl-downloading = Downloading
dl-queue-note = { $count ->
    [one] { $count } episode
   *[other] { $count } downloads
}
# Section heading
dl-finished = Finished
dl-finished-note = { $count ->
    [one] { $count } download, in your library
   *[other] { $count } downloads, in your library
}
# Button that starts the download of what was picked
dl-get = Get
# Button label when nothing is picked yet
dl-pick = Pick

## Storage and hidden shows

disk-on-phone = On this phone
disk-watched = Watched
disk-not-watched = Not watched
disk-deleting = Deleting
disk-delete-watched = Delete watched
disk-hidden-shows = Hidden shows
disk-hidden-helper = Out of Home and the library until you bring them back.
disk-unhide = Unhide
disk-delete-note = Files and subtitles are deleted from this phone for good.
disk-every-watched = Every watched episode
disk-every-finished = Every finished show you watched
# A count and a size, such as "12 episodes, 3.4 GB".
disk-episodes-size = { $count ->
    [one] { $count } episode, { $size }
   *[other] { $count } episodes, { $size }
}
disk-shows-size = { $count ->
    [one] { $count } show, { $size }
   *[other] { $count } shows, { $size }
}
# Names of shows in a list: two, three, and more than three.
disk-list-two = { $a } and { $b }
disk-list-three = { $a }, { $b } and { $c }
disk-list-more = { $a }, { $b }, { $c } and { $count } more
# Question in the sheet that confirms a delete. { $size } is a size such as 3.4 GB.
disk-delete-finished-title = Delete { $count ->
    [one] { $count } finished show
   *[other] { $count } finished shows
}, { $size }?
disk-delete-watched-title = Delete { $count ->
    [one] { $count } watched episode
   *[other] { $count } watched episodes
}, { $size }?
# { $names } is a list of show names.
disk-delete-finished-body = { $names }. Only shows the provider says have finished, every episode watched; a show still airing, or whose end is a guess, stays. Every file of them is deleted from this phone, extras too. Their downloads leave too, so they can be downloaded again.
disk-delete-watched-body = Every episode you have watched, in every show. Episodes not watched yet and extras stay. The files and their subtitles are deleted from this phone for good.
disk-no-finished = No finished show is watched all the way through
disk-nothing-watched = Nothing watched is on this phone
disk-deleted = Deleted { $count ->
    [one] { $count } episode
   *[other] { $count } episodes
}, { $size }
# { $error } is why it failed.
disk-delete-failed = Nothing was deleted: { $error }

## Download notifications

# Small head line of a notification as the shade draws it.
notify-card-head = AniBeam · now
# Sample used in the picture of the notification, "Frieren" is a show.
notify-sample-called = Frieren episode 7
notify-sample-called-ready = Frieren episode 6
# Sample figures in the picture of the notification: size, speed and time left.
notify-sample-facts = 420 MB of 1.4 GB, 3.2 MB/s, 5m left
# { $called } is what is downloading, such as "Frieren episode 7".
notify-downloading = Downloading { $called }
# { $percent } is a number without the percent sign, { $facts } the line Downloads shows.
notify-progress = { $percent }%, { $facts }
# { $done } and { $size } are sizes such as 420 MB.
notify-progress-of = { $percent }%, { $done } of { $size }
notify-starting = Starting
# Title of the notification while several downloads run.
notify-many = { $count ->
    [one] { $count } download
   *[other] { $count } downloads
}
# { $show } is a show, { $n } an episode number. Part of a notification, so it reads as a name.
notify-called-episode = { $show } episode { $n }
notify-called-episodes = { $show } episodes { $from } to { $to }
# A finished download. { $called } is such as "Frieren episode 7".
notify-ready = { $called } is ready
# For a run of episodes: "Frieren episodes 1 to 12 are ready".
notify-ready-many = { $called } are ready
notify-ready-batch = A batch of { $show } is ready
notify-ready-text = Downloaded to this phone. Tap to open AniBeam.
# Names of the two notification channels in the phone's own settings.
notify-channel-downloads = Downloads
notify-channel-ready = Episodes ready
# Title of the notification when no other is given.
notify-fallback-title = Downloading
# The row for the download notification.
notify-row-on = Notifications
notify-row-allow = Allow notifications
notify-row-open-settings = Open notification settings
notify-helper-on = On. Downloads show in your notifications while they run.
notify-helper-asks-next = Android asks next.
notify-helper-asks-next-later = Android asks next. You can change your mind later in More.
notify-helper-off = Off. Turn on notifications for AniBeam there.
notify-question-title = See downloads in your notifications?
notify-question-body = While episodes download, a notification shows how far along they are, and another says when one is ready. They download either way; this only decides whether you see it.
# Button that asks Android for permission
notify-allow = Allow
notify-not-now = Not now
notify-pointer = Downloads don't show in your notifications.
# Button that turns the notification on
notify-turn-on = Turn on

## Feed

feed-title = Feed
feed-recently-released = Recently released
feed-coming-soon = Coming soon
# Section titles on Home's feed. The first two belong to Recently released, the last two to Coming soon.
feed-this-week = This week
feed-earlier = Earlier
feed-everything-else = Everything else
# Short times left: days and hours, hours and minutes, minutes. The letters are units.
feed-countdown-days = { $d }d { $h }h
feed-countdown-hours = { $h }h { $m }m
feed-countdown-minutes = { $m }m
# { $title } is a show, { $countdown } a time such as 2d 3h.
feed-next = Next: { $title } in { $countdown }
feed-nothing-scheduled = Nothing scheduled yet.
feed-nothing-new = Nothing new this week.
feed-new-episodes = { $count ->
    [one] { $count } new episode this week.
   *[other] { $count } new episodes this week.
}
# Small print under a show. { $ago } is a time such as "3h ago".
feed-aired = aired { $ago }
feed-downloaded = downloaded { $ago }

## Files and the file picker

files-already-picking = A file is already being picked.
files-took-too-long = The file picker took too long.
# { $error } is why it failed.
files-did-not-open = The file picker did not open: { $error }
files-none-here = There is no file picker here.

## The computer's downloads

# { $what } is such as "3 episodes", { $name } the computer's name.
desk-sent-adding = Sent { $what } to { $name }. It is still adding them.
desk-sent = Getting { $what } on { $name }.

## Your computer's library

remote-watched = Watched
remote-movie = Movie
remote-not-watched-yet = Not watched yet
# { $done } of { $total } episodes, both numbers.
remote-watched-of = { $done } of { $total } watched
remote-watched-count = { $count } watched
remote-on-disk = { $count } on disk
# { $season } and { $n } are numbers. { $countdown } is a time such as 2d 3h 5m.
remote-airs-in-season = S{ $season } Episode { $n } airs in { $countdown }
remote-airs-in = Episode { $n } airs in { $countdown }
# Button. { $n } is an episode number.
remote-play-episode = Play episode { $n }
remote-watch-again = Watch episode { $n } again
# Under an episode of the other computer's library.
remote-update-host = Update AniBeam on that computer to play this here
# Date column of an episode that aired and is missing.
remote-aired = aired
remote-missing = Missing
remote-on-computer = On your computer
# { $ago } is a time such as "3h ago".
remote-out = Out { $ago }
remote-season-episode = Season { $season }, episode { $n }

## Episodes and dates: a season's episode count and air dates, set by hand

dates-title = Episodes and dates
dates-season-line = { $title } · Season { $season }
dates-paste-open = Paste a list
dates-paste-done = Done pasting
dates-paste-hint = Paste a release list, a table or a post
dates-paste-note = What AniBeam reads shows on the episodes below.
dates-tap-hint = Tap an episode to date it.
dates-run = Pick a run
dates-field-hint = 7 Jan, March 2027, 2027
dates-set = Set date
dates-no-date = No date yet
dates-weekly = Weekly
dates-last = Last episode
dates-not-last = Not the last
dates-clear = Clear date
dates-save = Save
dates-saving = Saving
dates-cancel = Cancel
dates-change = Change
dates-picked-one = Episode { $episode }:
dates-picked-many =
    { $count ->
        [one] { $count } episode:
       *[other] { $count } episodes:
    }
dates-saved = Saved.
dates-cleared = Schedule notes cleared.
dates-starting = AniBeam is still starting. Try again in a moment.
# A tile's line under its number.
dates-tile-out = Out
dates-tile-last = Last
dates-tile-none = No date
dates-tile-guess = ≈ { $date }
# The sheet's summary, its parts joined by " · ".
dates-summary-none = Nothing set yet
dates-summary-ends = Ends at episode { $episode }
dates-summary-record =
    { $count ->
        [one] { $count } dated by { $site }
       *[other] { $count } dated by { $site }
    }
dates-summary-record-unnamed =
    { $count ->
        [one] { $count } dated by the record
       *[other] { $count } dated by the record
    }
dates-summary-set =
    { $count ->
        [one] { $count } set by you
       *[other] { $count } set by you
    }
dates-summary-guessed =
    { $count ->
        [one] { $count } guessed weekly
       *[other] { $count } guessed weekly
    }
# The show page's line on what the user told the season.
dates-told-one = Episode { $episode }
dates-told-many = Episodes { $first } to { $last }
dates-told-undated = { $episodes } not dated yet
dates-told-due = { $episodes } due in { $when }
dates-told-ends = Ends at episode { $episode }
dates-told-season-ends = Season { $season } ends at episode { $episode }
dates-told-by-you = { $told }, set by you
dates-told-gave-way =
    { $count ->
        [one] You set { $count } episode; that no longer applies
       *[other] You set { $count } episodes; that no longer applies
    }

## The series page

# The facts line under a show's title: how long an episode or a film runs.
series-runtime =
    { $hours ->
        [0] { $minutes } min
       *[other] { $minutes ->
            [0] { $hours } h
           *[other] { $hours } h { $minutes } min
        }
    }
# How long each episode of a show runs, after its episode count.
series-runtime-each = { $runtime } each
# Whether a show airs, at the end of the facts line.
series-status-airing = Airing
series-status-finished = Finished
series-status-upcoming = Upcoming
series-status-cancelled = Cancelled
series-status-hiatus = On hiatus
# The scores line under the facts.
series-average-score = Average score { $score }
series-imdb = IMDb { $score }
series-imdb-votes = IMDb { $score } from { $votes } votes
series-rewatched =
    { $count ->
        [one] Rewatched { $count } time
       *[other] Rewatched { $count } times
    }

## More, About

about-ratings = Ratings
about-ratings-credit = Information courtesy of IMDb (https://www.imdb.com). Used with permission.

## Marking episodes on the series page

# The sheet a held episode opens.
mark-menu-watched = Mark watched up to here
mark-menu-unwatched = Mark unwatched from here
mark-menu-cancel = Cancel
# The line a mark that moves more than one episode leaves, with Undo.
mark-watched-up-to = Watched up to Episode { $episode }
mark-unwatched-from = Unwatched from Episode { $episode }
# A mark a tracker's show could not take, with no account signed in; $reason is the core's.
mark-not-connected = { $reason } Sign in under More, Accounts.
# The progress line of a show no tracker counts.
series-not-tracked = Not tracked
series-not-tracked-on-phone =
    { $count ->
        [one] Not tracked, { $count } episode on this phone
       *[other] Not tracked, { $count } episodes on this phone
    }

## The player

# The speed picker while the episode plays on a TV, which keeps its own speed.
player-speed-not-while-casting = Not while casting

## Downloads

# A release in the find list for an episode the phone has from another release.
downloads-other-release-on-disk = Other release on disk
