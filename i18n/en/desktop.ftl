# What the desktop app (apps/linux) shows. English is the source.
# IDs start with the page or part of the window they belong to: player-, settings-,
# library-. A word every shell says belongs in core.ftl instead.

## Nav

nav-home = Home
nav-library = Library
nav-airing = Airing now
nav-feed = Feed
nav-watching = Watching
nav-stats = Stats
nav-metadata = Show info
nav-downloads = Downloads
nav-settings = Settings
nav-series = Series
nav-player = Player
nav-show = Show
nav-recommended = Recommended

## StatusStrip

status-strip-errors = { $count ->
    [one] { $count } error
   *[other] { $count } errors
}

## ScanSummary

scan-summary-nothing = Nothing to show yet
scan-summary-found = Found { $count ->
    [one] { $count } show
   *[other] { $count } shows
}, { $recognised } recognised
scan-summary-found-need = Found { $count ->
    [one] { $count } show
   *[other] { $count } shows
}, { $recognised } recognised, { $need } need you
scan-summary-episodes = { $count ->
    [one] { $count } episode
   *[other] { $count } episodes
} on disk.
scan-summary-episodes-watched = { $count ->
    [one] { $count } episode
   *[other] { $count } episodes
} on disk, { $watched } already watched on { $tracker }.
scan-summary-title = Your folder is in
scan-summary-count-found = found
scan-summary-count-recognised = recognised
scan-summary-count-need = need you
scan-summary-later = Later
scan-summary-match = Match the { $count }
scan-summary-open-library = Open the library

## UpdateNotice

update-notice-check-failed = AniBeam could not check for updates. Check your internet connection, then try again.
update-notice-up-to-date = AniBeam is up to date
update-notice-downloading = Downloading AniBeam { $version }
update-notice-ready = AniBeam { $version } is ready
update-notice-failed = The update failed
update-notice-on-aur = AniBeam { $version } is on the AUR
update-notice-out = AniBeam { $version } is out
update-notice-out-titled = AniBeam { $version } “{ $title }” is out
update-notice-progress = { $percent }%  ·  { $got } of { $total } MB
update-notice-restart-hint = Restart now, or it opens next time
update-notice-from-source = This copy is built from source: pull and rebuild
update-notice-no-download = No download here for this computer
update-notice-install-hint = Update installs it and restarts AniBeam
update-notice-retry = Retry
update-notice-update = Update
update-notice-restart = Restart
update-notice-copied = Copied
update-notice-copy = Copy
update-notice-release-page = Release page
update-notice-later = Later

## WhatsNew

whats-new-title = What's new
whats-new-got-it = Got it

## ActivityDrawer

activity-drawer-time-format = hh:mm:ss
activity-drawer-stamp-format = d MMM hh:mm
activity-drawer-stage-library = Library
activity-drawer-stage-metadata = Metadata
activity-drawer-stage-trackers = Trackers
activity-drawer-stage-franchise = Franchise
activity-drawer-stage-playback = Playback
activity-drawer-stage-store = Store
activity-drawer-stage-system = System
activity-drawer-stage-downloads = Downloads
activity-drawer-level-info = Info
activity-drawer-level-warn = Warning
activity-drawer-level-error = Error
activity-drawer-copied-line = Copied the line
activity-drawer-title = Activity
activity-drawer-everything = Everything
activity-drawer-problems = Problems
activity-drawer-copied = Copied
activity-drawer-copy = Copy
activity-drawer-clear-every = Clear every line
activity-drawer-copy-tip = Click to copy the line
activity-drawer-fold = Fold
activity-drawer-empty-technical = Nothing logged yet.
activity-drawer-empty-problems = No problems. Everything worked.
activity-drawer-empty = Nothing to report yet. What AniBeam does for you shows up here.

## ActivityColumn

activity-column-episode-run = Episodes { $first } to { $last }
activity-column-episode-list = Episodes { $list }
activity-column-downloads = { $count ->
    [one] { $count } download
   *[other] { $count } downloads
}
activity-column-batch = Batch download
activity-column-download = Download
activity-column-name-batch = { $title }, a batch
activity-column-name-episode = { $title }, episode { $number }
activity-column-title = Activity
activity-column-down = down
activity-column-up = up
activity-column-not-connected = { $client } is not connected, so nothing moves here.
activity-column-connect-again = Connect again
activity-column-downloading = Downloading now
activity-column-fewer = Fewer
activity-column-more = And { $count } more
activity-column-nothing-downloading = Nothing is downloading.
activity-column-finished-today = Finished today
activity-column-today = Today
activity-column-today-empty = Searches, downloads and new subscriptions show up here as they happen.
activity-column-stopping = Stopping
activity-column-starting = Starting
activity-column-start-tip = Start downloading
activity-column-stop-tip = Stop downloading; what it has stays
activity-column-cancelling = Cancelling
activity-column-cancel-all-tip = Cancel these downloads
activity-column-cancel-tip = Cancel this download
activity-column-size-of = { $got } of { $total }
activity-column-files-missing = Files missing
activity-column-time-format = hh:mm
activity-column-left = { $time } left
activity-column-waiting-peers = Waiting for peers
activity-column-episodes-done = { $count ->
    [one] { $done } of { $count } episode done
   *[other] { $done } of { $count } episodes done
}
activity-column-cube-skipped = Episode { $number }: skipped
activity-column-cube-done = Episode { $number }: done
activity-column-cube-progress = Episode { $number }: { $percent }%

## Frame

frame-window-title = { $page } - AniBeam
frame-strip-ready = Ready
frame-strip-progress = { $doing }, { $done } of { $total }
frame-strip-idle = Nothing is running
frame-strip-time-format = hh:mm
frame-finish-sign-in = Finish the { $tracker } sign-in first
frame-did-not-work = That did not work.
frame-could-not-open = AniBeam could not open that. Copy it and open it yourself.
frame-copy = Copy
frame-copied = Copied
frame-status-refused = { $tracker } did not take it
frame-status-refused-why = { $tracker } did not take it: { $why }
frame-lists-refused = { $tracker } did not take { $count ->
    [one] 1 show
   *[other] { $count } shows
}; see the activity log
frame-lists-moved = Moved { $count ->
    [one] 1 show
   *[other] { $count } shows
} to { $tracker }
frame-notes-refused = { $tracker } did not take your notes
frame-notes-refused-why = { $tracker } did not take your notes: { $why }
frame-notes-saved = Notes saved
frame-connected = Connected to { $tracker } as { $user }
frame-sign-in-stopped = The { $tracker } sign-in stopped before it finished.
frame-toast-hide-details = Hide details
frame-toast-details = Details
frame-toast-details-tip = Show what went wrong, word for word, and copy it
frame-forward = Forward
frame-copy-magnet = Copy magnet
frame-copy-link = Copy torrent link
frame-magnet-copied = Magnet copied
frame-link-copied = Link copied
frame-open-release-page = Open release page
frame-open-folder = Open folder
frame-copy-path = Copy path
frame-path-copied = Path copied
frame-mark-unwatched = Mark unwatched from here
frame-mark-watched = Mark watched up to here
frame-copy-title = Copy title
frame-open-on-anilist = Open on AniList
frame-list-home-online = Change it on each list that is connected
frame-list-home-unmatched = Kept on this device: no tracker carries this show
frame-list-home-not-carried = Kept on this device: your tracker does not carry this show
frame-list-home-offline = Kept on this device until you connect AniList or MyAnimeList
frame-moving-to = Moving to { $list }
frame-moved-to = Moved to { $list }
frame-plan-added = { $title } goes on your Planning list
frame-plan-listed = { $title } is on your list already
frame-delete-watched = { $kind ->
    [film] Delete watched films
   *[episode] Delete watched episodes
}
frame-delete-all = { $count ->
    [one] { $kind ->
        [film] Delete the film
       *[episode] Delete the episode
    }
   *[other] { $kind ->
        [film] Delete all { $count } films
       *[episode] Delete all { $count } episodes
    }
}
frame-series-gone = That series is gone
frame-delete-none-watched = No watched episodes of { $title } on disk
frame-delete-none = Nothing of { $title } on disk
frame-delete-files = { $watched ->
    [yes] { $kind ->
        [film] { $count ->
            [one] { $count } watched film
           *[other] { $count } watched films
        }
       *[episode] { $count ->
            [one] { $count } watched episode
           *[other] { $count } watched episodes
        }
    }
   *[no] { $kind ->
        [film] { $count ->
            [one] { $count } film
           *[other] { $count } films
        }
       *[episode] { $count ->
            [one] { $count } episode
           *[other] { $count } episodes
        }
    }
}
frame-delete-extras = { $count ->
    [one] { $count } extra
   *[other] { $count } extras
}
frame-delete-and = { $first } and { $second }
frame-delete-heading = Delete { $what } of { $title }?
frame-delete-base = The files and their subtitles go to the trash, so you can get them back from there. Their torrents leave the torrent client, so they can be downloaded again.
frame-delete-confirm = Move to trash
frame-delete-done = { $what } of { $title } moved to the trash
frame-delete-watched-done = { $count ->
    [one] { $count } episode
   *[other] { $count } episodes
}, { $size }, moved to the trash
frame-delete-nothing = Nothing to delete
frame-fix-not-drafted = Fixed. The issue could not be drafted
frame-fix-agent = Fixed. { $agent } is looking into it, and the issue opens when it is done
frame-fix-opening = Fixed. Opening the issue
frame-fix-not-saved = The fix did not save: { $error }
frame-fix-undone = { $count ->
    [one] { $count } file
   *[other] { $count } files
} read by name again
frame-fix-done = Fixed how { $count ->
    [one] { $count } file
   *[other] { $count } files
} read
frame-report-failed = The issue could not be drafted: { $error }
frame-report-untriaged = The issue is open in your browser, without a triage. { $error }
frame-report-open = The issue is open in your browser, ready to send
frame-downloads-not-read = Not read.

## HomePage

home-title = Home
home-scan-started = Scan started
home-ago-minutes = { $count }m ago
home-ago-hours = { $count }h ago
home-ago-days = { $count }d ago
home-today = Today
home-tomorrow = Tomorrow
home-yesterday = Yesterday
home-weekday-format = dddd
home-weekday-short-format = ddd
home-date-format = d MMM
home-day-date-format = ddd d MMM
home-time-format = HH:mm
home-day-time-format = ddd HH:mm
home-next-time-format = dddd HH:mm
home-no-date-yet = No date yet
home-unit-days = { $count ->
    [one] day
   *[other] days
}
home-unit-hours = h
home-unit-minutes = m
home-week-late = { $count ->
    [one] { $count } episode is late
   *[other] { $count } episodes are late
}
home-week-waiting = { $count ->
    [one] { $count } episode out to watch
   *[other] { $count } episodes out to watch
}
home-week-coming = { $count ->
    [one] { $count } episode in the next seven days
   *[other] { $count } episodes in the next seven days
}
home-week-next = Next up: { $title } in { $time }
home-countdown-days = { $days }d { $hours }h
home-countdown-hours = { $hours }h { $minutes }m
home-countdown-minutes = { $minutes }m
home-out-not-downloaded = Not downloaded yet
home-out-downloading = Downloading
home-out-when = { $day } { $time }
home-next-up = Next up
home-next-when = { $span }, { $time }
home-next-when-estimated = { $span }, { $time }, time estimated
home-next-when-season = Season { $season }, { $span }, { $time }
home-next-when-season-estimated = Season { $season }, { $span }, { $time }, time estimated
home-no-date = No date
home-tile-watched = watched
home-tile-downloading = downloading
home-tile-out = out { $ago }
home-tile-not-out = not out yet
home-tile-estimated = estimated
home-tile-season-span = S{ $season } { $span }
home-tile-to-watch = { $count } to watch
home-day-nothing = Nothing
home-now = Now { $time }
home-card-aired = { $episode } out { $ago }
home-continue = Continue
home-continue-fact = { $count } series to go on with
home-on-hold = On hold
home-on-hold-fact = { $count } series to come back to
home-hero-open = Open { $title }
home-hero-pick-up = Pick up where you stopped
home-hero-up-next = Up next
home-hero-downloading = Downloading { $count ->
    [one] { $count } episode
   *[other] { $count } episodes
}
home-hero-min-left = { $count } min left
home-hero-watched = { $watched } of { $total } watched
home-hero-resume = Resume episode { $number }
home-hero-play = Play episode { $number }
home-hero-resume-unnumbered = Resume episode
home-hero-play-unnumbered = Play episode
home-downloaded = Recently downloaded
home-downloaded-fact = { $count } series with new files this fortnight
home-week = This week
home-week-show-empty = Show empty days
home-week-hide-empty = Hide empty days
home-week-airing = Airing now
home-week-out = Out now
home-week-coming-up = Coming up
home-recommended = Recommended for you
home-recommended-fact = From your own list
home-recommended-scope = Recommended from
home-recommended-all = All
home-recommended-in-library = In library
home-recommended-on-list = On list
home-recommended-none-library = Nothing in your library is left to recommend. All finds shows you do not have yet.
home-recommended-none-list = Nothing on your lists is left to recommend. All finds shows you have not listed.
home-catch-up = Catch up
home-catch-up-fact = { $count } series with episodes waiting
home-fresh = Start something new
home-fresh-fact = { $count } series not started
home-genres = Genres
home-open-library = Open library
home-dropped = Dropped
home-dropped-fact = { $count ->
    [one] { $count } file still on disk
   *[other] { $count } files still on disk
}
home-out-now = Out now
home-out-now-fact = New episodes this week, from AniList
home-popular = Popular this season
home-add-folder-title = Your own shows go here too
home-add-folder-body = Add the folder your anime lives in and Home fills with what you are watching.
home-add-folder-button = Add a folder
home-empty-scanned-title = Nothing found in your folders yet
home-empty-title = No series yet
home-empty-scanned-body = AniBeam found no series or films in the folders it watches. Scan them again, or check the folders in Settings.
home-empty-body = Add the folder your anime lives in and the library fills itself.
home-scan-all = Scan all
home-library-settings = Library settings
home-failed-title = Home did not load
home-failed-body = AniBeam could not read the library just now. Try again in a moment.

## Onboarding

onboarding-step-welcome = Welcome
onboarding-step-library = Your library
onboarding-step-lists = Your list
onboarding-step-downloads = Downloads
onboarding-step-together = Watch together
onboarding-step-ready = Ready
onboarding-scan-reading = Reading the folder
onboarding-scan-found = Found { $count } series. Go on whenever you like; the rest fills in by itself.
onboarding-scan-starter-empty = Your new folder is empty for now. Shows you put in it turn up here by themselves.
onboarding-scan-no-video = No video files turned up. Check that it is the right folder.
onboarding-scan-your-shows = your shows
onboarding-scan-looking-up = Looking up { $what }
onboarding-scan-looking-up-tally = Looking up { $what }, { $posters } of { $count } found
onboarding-scan-posters = Fetching posters
onboarding-scan-posters-tally = Fetching posters, { $posters } of { $count } found
onboarding-scan-seasons = Finding related seasons
onboarding-scan-seasons-tally = Finding related seasons, { $posters } of { $count } found
onboarding-choose-folder = Choose your anime folder
onboarding-about = A minute of setup. Your library finishes on its own.
onboarding-step-later = Later
onboarding-skip-setup = Skip setup
onboarding-title-welcome = Welcome to AniBeam
onboarding-title-library = Where is your anime?
onboarding-title-lists = Connect your anime list
onboarding-title-downloads = How downloads work
onboarding-title-together = Watch with friends
onboarding-title-ready = You're all set
onboarding-title-later = { $count ->
    [1] Ready, one thing for later
    [2] Ready, two things for later
   *[other] Ready, three things for later
}
onboarding-body-welcome = AniBeam keeps your anime in one place. It finds the shows on your disk, plays them right here, and keeps AniList or MyAnimeList up to date as you watch.
onboarding-body-library = Pick the folder you keep your shows in. Every folder inside it becomes a series, with its poster, its episodes and where you left off. AniBeam only reads the folder. It never moves or renames a file.
onboarding-body-lists = Sign in once, and every episode you finish here is marked on your list, with nothing to type. The sign-in happens in your browser, so AniBeam never sees your password.
onboarding-body-downloads = Follow a show on the Downloads page and each new episode lands in your library as it airs. Two things make that work: somewhere to search, and a torrent client to fetch with.
onboarding-body-together = Start a room from the player and send its code. Everyone plays their own copy of the episode, and play, pause and seek stay in step for all of you.
onboarding-body-ready = Everything here can be changed later in Settings, and Settings, About runs this setup again.
onboarding-skip-step = Skip for now
onboarding-get-started = Get started
onboarding-open-app = Open AniBeam
onboarding-continue = Continue
onboarding-payoff-library = Your library
onboarding-payoff-library-line = Posters, episodes and what comes next, for every show in your folders.
onboarding-payoff-plays = Plays right here
onboarding-payoff-plays-line = Subtitles, intro skips, and it remembers where you stopped.
onboarding-payoff-list = Your list, synced
onboarding-payoff-list-line = Finish an episode and it counts on AniList or MyAnimeList.
onboarding-payoff-episodes = Episodes arrive
onboarding-payoff-episodes-line = Follow a show and each new episode lands in your library.
onboarding-films-too = A folder of films works too: call it Movies.
onboarding-choose-folder-button = Choose folder
onboarding-no-folder = I don't have a folder yet
onboarding-starter-where = That makes an Anime folder in { $dir } and uses it.
onboarding-add-another = Add another folder
onboarding-shelf-more = more
onboarding-preview-show = Your show
onboarding-preview-finished = Episode 3 finished here
onboarding-preview-on-list = On your list: 3 watched
onboarding-preview-set-by = Set by AniBeam the moment the credits roll
onboarding-lists-connected = Connected. What you watch here now counts on your list.
onboarding-lists-no-account = No account yet? Both are free, and one is plenty. You can disconnect any time in Settings, Accounts.
onboarding-search-title = Somewhere to search
onboarding-search-ready = { $count ->
    [one] { $count } source ready to search.
   *[other] { $count } sources ready to search.
}
onboarding-search-none = AniBeam comes with no sources of its own. You add them from a source repo: a link someone shares with you, which you paste in Settings, Downloads.
onboarding-sources-after = Opens after setup
onboarding-sources-add = Add a repo after setup
onboarding-fetch-title = Who fetches
onboarding-fetch-auto = Let AniBeam handle it
onboarding-fetch-recommended = Recommended
onboarding-fetch-auto-line = AniBeam brings its own qBittorrent and runs it quietly in the background. Nothing to install, nothing to fill in.
onboarding-fetch-existing = I already use qBittorrent
onboarding-fetch-existing-line = AniBeam connects to the qBittorrent you already run. You enter its address and login in Settings next.
onboarding-ready-library = Library
onboarding-ready-library-none = No folder yet. It lives in Settings, Library when you want it.
onboarding-ready-library-series = { $count ->
    [one] { $count } folder
   *[other] { $count } folders
}, { $series } series so far
onboarding-ready-library-empty = { $count ->
    [one] { $count } folder
   *[other] { $count } folders
}, empty for now
onboarding-ready-library-scanning = { $count ->
    [one] { $count } folder
   *[other] { $count } folders
}, scanning
onboarding-ready-lists = Your list
onboarding-ready-lists-as = { $tracker }, as { $user }
onboarding-ready-lists-none = Not connected yet. It lives in Settings, Accounts when you want it.
onboarding-ready-downloads = Downloads
onboarding-ready-downloads-line = { $sources ->
    [0] No sources yet
    [one] { $sources } source
   *[other] { $sources } sources
}, { $client ->
    [own] your own qBittorrent.
   *[bundled] AniBeam's own qBittorrent.
} { $next ->
    [ready] All set.
    [opens] Settings, Downloads opens next.
   *[both] Both live in Settings, Downloads.
}
onboarding-ready-together = Watch together
onboarding-ready-together-as = Rooms show you as { $name }.
onboarding-ready-together-none = No name yet. The room panel asks the first time.
onboarding-together-you = You
onboarding-together-code = Room code
onboarding-together-episode = Frieren · Episode 3
onboarding-together-friend-1 = Sam
onboarding-together-friend-2 = Robin
onboarding-together-name = Your name in rooms
onboarding-ready-open-downloads = Open Settings, Downloads
onboarding-ready-set-up = Set up now

## Rail.qml

rail-home = Home
rail-library = Library
rail-airing = Airing
rail-feed = Feed
rail-stats = Stats
rail-show-info = Show info
rail-downloads = Downloads
rail-settings = Settings
rail-downloads-complete =
    { $count ->
        [one] Downloads: { $count } subscription is complete
       *[other] Downloads: { $count } subscriptions are complete
    }
rail-title-language-name = Title language: { $language }. Press for the next one
rail-title-language-english = English
rail-title-language-tip = Title language: Romaji, English or Japanese
rail-whats-new-in = What's new in { $release }
rail-whats-new = What's new
rail-settings-note = Settings: { $note }
rail-room = Watch together (Ctrl+J)
rail-room-in = In a watch together room (Ctrl+J)

## FixSheet.qml

fix-kind-episode = Episode
fix-kind-opening = Opening
fix-kind-ending = Ending
fix-kind-preview = Preview
fix-kind-special = Special
fix-kind-other = Other
fix-reads-season-episode = Season { $season } episode { $number }
# An extra with its number: "Opening 2"
fix-reads-extra = { $kind } { $index }
fix-apply-failed = Could not apply the fixes.
fix-title = Fix how files are read
fix-lede = { $title }. Say what each file really is; every scan after this keeps it.
fix-rescan = Rescan folder
fix-rescan-started = Reading the folder again
fix-filled-in = Filled in
fix-fill-in = Fill in
fix-column-file = File
fix-column-reads-as = Reads as
fix-column-should-be = Should be
fix-fixed = Fixed
# The number field's placeholder for an episode, and for an extra that may have none
fix-number-placeholder = No.
fix-index-placeholder = None
fix-keep = Keep the fix
fix-leave-as-read = Leave this file as it reads
fix-read-by-name = Read this file by its name again
fix-report = File an issue on GitHub
fix-report-body = Opens a filled in issue for you to send. It is public and names these files.
fix-report-body-agent = Opens a filled in issue for you to send. It is public and names these files. { $agent } looks into it first.
fix-close-unapplied =
    { $count ->
        [one] Close without applying { $count } fix?
       *[other] Close without applying { $count } fixes?
    }
fix-number-invalid = A number in red needs fixing first
fix-close-anyway = Close anyway
fix-keep-editing = Keep editing
fix-apply-count =
    { $count ->
        [one] Apply { $count } fix
       *[other] Apply { $count } fixes
    }

## MatchPanel.qml

match-matched-to = Matched to { $title }
match-applied = Match applied
match-found-tvmaze = Found the show on TVmaze
match-found-anilist = Found it on AniList
match-found-anilist-season = Found it on AniList, season { $season }
match-found-mal = Found it on MyAnimeList
# The sites a series is matched on, two at a time: "AniList and MyAnimeList"
match-sites-and = { $first } and { $next }
match-matched-on = Matched on { $sites }
match-provider-silent = { $provider } did not answer
# The heading in a walk through several series, with "2 of 5"
match-title-walk = Match, { $walk }
match-title = Match
match-skip = Skip
match-refresh-info = Refresh info
match-refreshing = Refreshing { $title }
match-clear-confirm = Clear it: drops the link and the set episode count
match-clear = Clear match
match-cleared = Match cleared
match-search-placeholder = Search { $provider }
match-link-tvmaze = Paste a TVmaze or IMDb link
match-link-anilist = Paste an AniList or MyAnimeList link
match-look-up = Look up
match-searching = Searching { $provider }
match-type-more = Type at least two letters to search { $provider }.
match-no-results = No titles found on { $provider }.

## WatchingPage.qml

# A card's progress code, the number padded to two digits: "Ep 03"
watching-episode-code = Ep { $number }
watching-countdown-days = { $days }d { $hours }h
watching-countdown-hours = { $hours }h { $minutes }m
watching-countdown-minutes = { $minutes }m
watching-on-disk = On disk
watching-on-anilist = On AniList
watching-ready-to-start = Ready to start
watching-not-in-library = Not in the library
watching-get-to-start = Get episodes to start
# $when is a relative time such as "3d ago"
watching-anilist-updated = AniList updated { $when }
watching-airing = Airing
# $wait is a short countdown such as "2d 4h"
watching-next-in = Next: { $title } in { $wait }
watching-episodes-waiting = Episodes waiting
watching-series-behind =
    { $count ->
        [one] { $count } series behind
       *[other] { $count } series behind
    }
watching-up-to-date = Up to date
watching-anilist-only = On AniList only
watching-planning-summary-off = What you plan to watch on AniList.
watching-planning-summary =
    { $count ->
        [one] { $count } show planned, { $here } on disk.
       *[other] { $count } shows planned, { $here } on disk.
    }
watching-summary-off = What you're watching or rewatching on AniList.
watching-summary-waiting =
    { $count ->
        [one] { $count } series has episodes waiting.
       *[other] { $count } series have episodes waiting.
    }
watching-caught-up = All caught up.
watching-start-watching = Start watching
watching-play-next = Play next
watching-open-on-anilist = Open on AniList
watching-refreshing = Refreshing
watching-connect-title = AniList is not connected
watching-connect-body = Connect it and this page lists the shows you are watching or rewatching on AniList, with Play on the ones you have and a countdown on the ones airing.
watching-connect-waiting = Waiting for the browser
watching-connect-anilist = Connect AniList
watching-load-failed-title = The watching list did not load
watching-load-failed-body = AniList could not be reached. The list comes back on the next refresh.
watching-planning-failed-title = The planning list did not load
watching-planning-failed-body = Its cache could not be read. Try again, or it comes back on the next refresh.
watching-planning-empty-title = Nothing on your planning list
watching-planning-empty-body = Plan to watch on Airing now, or Planning in a show's list menu, puts a show here.
watching-airing-now = Airing now
watching-empty-title = Nothing on your watching list
watching-empty-body = Mark a show as Watching on AniList and it lands here.
watching-open-anilist = Open AniList

## LibraryPage.qml

library-hidden =
    { $count ->
        [one] { $count } show hidden
       *[other] { $count } shows hidden
    }
library-hidden-sync-stops =
    { $count ->
        [one] { $count } show hidden, and their tracker sync stops
       *[other] { $count } shows hidden, and their tracker sync stops
    }
library-unhidden =
    { $count ->
        [one] { $count } show back, and syncing again
       *[other] { $count } shows back, and syncing again
    }
library-undo = Undo
library-clearing-score =
    { $count ->
        [one] Clearing the score of { $count } show
       *[other] Clearing the score of { $count } shows
    }
library-rating =
    { $count ->
        [one] Rating { $count } show { $score }
       *[other] Rating { $count } shows { $score }
    }
# $list is the list's name, such as Watching
library-moving =
    { $count ->
        [one] Moving { $count } show to { $list }
       *[other] Moving { $count } shows to { $list }
    }
library-rescanning =
    { $count ->
        [one] Rescanning { $count } show
       *[other] Rescanning { $count } shows
    }
library-finding-matches =
    { $count ->
        [one] Looking for a match for { $count } show
       *[other] Looking for a match for { $count } shows
    }
library-sort-last-viewed = Recently watched
library-sort-downloaded = Recently downloaded
library-sort-title = Title
library-sort-progress = Progress
library-sort-average-score = Average score
library-sort-my-score = My score
library-dir-a-to-z = A to Z
library-dir-z-to-a = Z to A
library-dir-oldest = Oldest first
library-dir-newest = Newest first
library-dir-least-watched = Least watched first
library-dir-most-watched = Most watched first
library-dir-lowest = Lowest first
library-dir-highest = Highest first
library-group-none = Nothing
library-group-season = Season
library-group-type = Type
library-group-genre = Genre
library-group-watch-state = Watch state
library-scan-started = Scan started
library-tab-all = All
library-tab-series = Series
library-tab-movies = Movies
library-tab-watched = Watched
library-tab-hidden = Hidden
library-count-movies =
    { $count ->
        [one] { $count } movie
       *[other] { $count } movies
    }
library-count-watched = { $count } watched
library-count-hidden = { $count } hidden
library-count-series =
    { $count ->
        [one] { $count } series
       *[other] { $count } series
    }
library-title = Library
library-all-genres = All genres
library-search-placeholder = Search titles or folders
library-sort = Sort
library-group = Group
library-franchises-together = Franchises together
library-watched-shows = Watched shows
library-load-failed-title = The library did not load
library-no-matches = No matches for "{ $query }"
# $what is the tab: series, films, watched, hidden or shows
library-none-in-genre =
    { $what ->
        [series] No { $genre } series here
        [films] No { $genre } films here
        [watched] No { $genre } watched shows here
        [hidden] No { $genre } hidden shows here
       *[shows] No { $genre } shows here
    }
library-none-here =
    { $what ->
        [series] No series here
        [films] No films here
        [watched] No watched shows here
        [hidden] No hidden shows here
       *[shows] No shows here
    }
library-load-failed-body = AniBeam could not read the library just now. Try again in a moment.
library-no-matches-body = Check the spelling, or search the whole library from All.
library-none-in-genre-body = Nothing in this view has that genre. All genres shows the rest.
library-none-in-tab-body = Your library has none of these yet. All shows everything in it.
library-all-filtered-body = Everything in your library is filtered out of this view.
library-clear-search = Clear search
library-show-all = Show all
library-picked = { $count } picked
library-pick-all = All
library-pick-all-tip = Pick every card shown (Ctrl+A)
library-unhide = Unhide
library-hide = Hide
library-rate = Rate
library-rate-tip = One score for every pick, on every connected tracker
library-needs-tracker = Needs a connected tracker and a matched show
library-move-to-list = Move to list
library-rescan = Rescan
library-find-match = Find a match
library-find-match-tip = Search for each pick without a match
library-all-matched = Every pick has a match
library-empty-scanned-title = Nothing found in your folders yet
library-empty-title = Your library is empty
library-empty-scanned-body = AniBeam found no series or films in the folders it watches. Scan them again, or check the folders in Settings.
library-empty-body = Add the folder your anime lives in and the library fills itself, or import an AniBeam export.
library-scan-all = Scan all
library-add-folder = Add a folder
library-settings = Library settings
library-import = Import
library-import-title = Import an AniBeam export
library-import-filter-export = AniBeam export (*.json)
library-import-filter-all = All files (*)
library-import-started = Import started

## MetadataPage.qml

metadata-title = Show info
metadata-filter-all = All { $count }
metadata-filter-series = Series { $count }
metadata-filter-films = Films { $count }
metadata-filter-missing = Missing { $count }
metadata-scan-started = Scan started
# Complete's own sort
metadata-sort-added = Added
metadata-sort-season = Season
metadata-sort-a-to-z = A to Z
metadata-section-unmatched = Not recognised
metadata-section-missing = Folder not found
metadata-section-incomplete = Missing episodes
metadata-section-empty = Not downloaded
metadata-section-airing = Airing
metadata-section-complete = Complete
metadata-no-info = No show info yet
# A tile's state line, short: "5 of 7 aired", "9 of 12", "12 of 12 +2"; $expected may be "?"
metadata-state-aired = { $have } of { $aired } aired
metadata-state-of = { $have } of { $expected }
metadata-state-of-extra = { $have } of { $expected } +{ $extra }
metadata-detail-none-aired = None of the { $aired } aired episodes are on disk
metadata-detail-aired = { $have } of { $aired } aired on disk
metadata-detail-aired-missing = { $have } of { $aired } aired on disk, { $missing } missing
metadata-detail-none = None of the { $expected } episodes are on disk
metadata-detail-missing = { $have } of { $expected } on disk, { $missing } missing
# The match panel's place in a walk through the series that need a match: "2 of 5"
metadata-walk-position = { $at } of { $total }
metadata-walk-done = That was the last one
metadata-walk-left =
    { $count ->
        [one] One left under Needs a match
       *[other] { $count } left under Needs a match
    }
metadata-countdown-days = { $days }d { $hours }h
metadata-countdown-hours = { $hours }h { $minutes }m
metadata-countdown-minutes = { $minutes }m
# $wait is a short countdown such as "2d 4h"
metadata-next-in = Next in { $wait }
metadata-files-of = { $have } of { $expected }
metadata-files-on-disk = Files on disk
metadata-forgot = Forgot { $title }
metadata-refreshed-failed = Refreshed { $refreshed }, { $failed } could not be refreshed
metadata-refreshed =
    { $count ->
        [one] Refreshed { $count } series
       *[other] Refreshed { $count } series
    }
metadata-found =
    { $count ->
        [one] Found { $count } show
       *[other] Found { $count } shows
    }
metadata-found-some-left =
    { $count ->
        [one] Found { $count } show, { $unmatched } still not recognised
       *[other] Found { $count } shows, { $unmatched } still not recognised
    }
metadata-found-nothing = Found nothing new, { $unmatched } still not recognised
metadata-all-recognised = Every show is recognised
metadata-count =
    { $count ->
        [one] { $count } series
       *[other] { $count } series
    }
metadata-search-placeholder = Filter titles
metadata-find-rest = Find the rest
metadata-find-rest-started = Looking up the folders AniBeam does not know yet
metadata-find-rest-tip = Looks up every folder AniBeam does not know yet, on each show info site in the order Settings gives them
metadata-refresh-all = Refresh all
metadata-refresh-all-started = Updating the info of every show
metadata-refresh-all-tip = Fetches the info of every recognised show again
metadata-crawling = Finding franchise links
metadata-crawl-progress = { $done } of { $total }
metadata-load-failed-title = Show info did not load
metadata-empty-scanned-title = Nothing found in your folders yet
metadata-no-matches = No matches for "{ $query }"
# $filter is the filter's code from the core: Series, Movies or MissingFiles
metadata-nothing-under =
    { $filter ->
        [Series] Nothing under Series
        [Movies] Nothing under Movies
        [MissingFiles] Nothing under MissingFiles
       *[other] Nothing under { $filter }
    }
metadata-load-failed-body = AniBeam could not read the library just now. Try again in a moment.
metadata-empty-scanned-body = AniBeam found no series in the folders it watches. Scan them again, or check the folders in Settings.
metadata-empty-body = Your library is empty. Add the folder your anime lives in and AniBeam scans it.
metadata-no-matches-body = No title in this view has those words.
metadata-nothing-under-body = None of your series fit this filter right now. All shows the rest.
metadata-scan-all = Scan all
metadata-add-folder = Add a folder
metadata-clear-search = Clear search
metadata-show-all = Show all
metadata-refreshing = Refreshing { $title }
metadata-refresh-tip = Fetch this show's info again
metadata-forget-armed = Remove it
metadata-forget-confirm-tip = Click again to remove this show and its watch history from AniBeam
metadata-forget-tip = The folder is not on the disk any more: remove this show and its history from AniBeam

## AiringPage.qml

airing-title = Airing now
airing-sort-popular = Popularity
airing-sort-score = Score
airing-sort-trending = Trending
airing-sort-favourites = Favourites
airing-sort-soonest = Next episode
airing-sort-aired = Episodes aired
airing-sort-episodes = Episode count
airing-sort-start = Start date
airing-sort-duration = Episode length
airing-sort-title = Title
airing-dir-most-popular = Most popular first
airing-dir-least-popular = Least popular first
airing-dir-lowest = Lowest first
airing-dir-highest = Highest first
airing-dir-least-trending = Least trending first
airing-dir-most-trending = Most trending first
airing-dir-fewest = Fewest first
airing-dir-most = Most first
airing-dir-soonest = Soonest first
airing-dir-latest = Latest first
airing-dir-oldest = Oldest first
airing-dir-newest = Newest first
airing-dir-shortest = Shortest first
airing-dir-longest = Longest first
airing-dir-a-to-z = A to Z
airing-dir-z-to-a = Z to A
# A season and its year; $season is the code Winter, Spring, Summer or Fall
airing-season =
    { $season ->
        [Winter] Winter { $year }
        [Spring] Spring { $year }
        [Summer] Summer { $year }
        [Fall] Fall { $year }
       *[other] { $season } { $year }
    }
airing-ago-minutes = { $minutes }m ago
airing-ago-hours = { $hours }h ago
airing-ago-days = { $days }d ago
# $episode is the episode's name, such as "Episode 7"; $ago a time such as "2h ago"
airing-episode-out = { $episode } out { $ago }
airing-no-date = No date yet
airing-in-library = In library
airing-out-today =
    { $count ->
        [one] { $count } episode out today
       *[other] { $count } episodes out today
    }
airing-dismissed-one = { $title } won't show up here again
airing-dismissed =
    { $count ->
        [one] { $count } show won't show up here again
       *[other] { $count } shows won't show up here again
    }
airing-planned-already-one = { $title } is on your list already
airing-planned-already-all = They are all on your list already
airing-planned =
    { $count ->
        [one] { $count } show goes on your Planning list
       *[other] { $count } shows go on your Planning list
    }
airing-planned-some-already =
    { $count ->
        [one] { $count } show goes on your Planning list, { $already } were on it already
       *[other] { $count } shows go on your Planning list, { $already } were on it already
    }
airing-play-next = Play next
airing-plan-to-watch = Plan to watch
airing-get-more-episodes = Get more episodes
airing-get-episodes = Get episodes
airing-open-on-anilist = Open on AniList
airing-not-interested = Not interested
airing-not-interested-count = { $count } not interested
airing-not-interested-tip = Shows you marked not interested, in Settings
airing-refreshing = Refreshing
airing-pick-season = Pick another season
airing-filter-all = Everything
airing-filter-lists = On my lists
airing-filter-library = In my library
airing-hide-lists = Hide my lists
airing-hide-lists-tip = Leave out the shows on your AniList or MAL lists
airing-hide-library = Hide my library
airing-hide-library-tip = Leave out the shows you have episodes of
airing-out-now = Out now
airing-out-last-day =
    { $count ->
        [one] { $count } episode in the last 24 hours
       *[other] { $count } episodes in the last 24 hours
    }
airing-out-fewer = Fewer
airing-out-all = All { $count }
airing-sort = Sort
airing-load-failed-title = AniList could not be reached
airing-load-failed-body = The shows on air come from AniList. They come back on the next refresh.
airing-reading-title = Reading what is on air
airing-nothing-now = Nothing on air right now
airing-nothing-for = Nothing on air for { $season }
airing-nothing-read = Nothing read yet
airing-reading-body = Every show on air, from AniList. This takes a few seconds the first time.
airing-nothing-body = AniList lists no shows for this season yet. Refresh reads it again, or pick another season.
airing-nothing-read-body = Refresh reads every show on air from AniList.
airing-hides-empty-title = Your hides leave nothing here
airing-lists-empty-title = Nothing here is on your lists
airing-library-empty-title = Nothing here is in your library
airing-hides-empty-body = Every show here is on your lists or in your library, and you hid those.
airing-lists-empty-body = Plan to watch on a card puts it on your list.
airing-library-empty-body = Get episodes on a card downloads it into your library.
airing-show-everything = Show everything
airing-picked = { $count } picked
airing-undo = Undo

## EpisodeRow.qml

episode-row-date-time-format = ddd d MMM, HH:mm
episode-row-date-upcoming-format = ddd d MMM
episode-row-date-format = d MMM yyyy
episode-row-mark-unwatched-tip = { $state }. Click to mark unwatched from here
episode-row-mark-watched-tip = { $state }. Click to mark watched up to here
episode-row-next-up = Next up
episode-row-versions =
    { $count ->
        [one] { $count } version
       *[other] { $count } versions
    }
episode-row-versions-tip =
    { $count ->
        [one] The same episode from { $count } release. Click to pick which plays
       *[other] The same episode from { $count } releases. Click to pick which plays
    }
episode-row-downloading = Downloading { $percent }%
episode-row-get = Get episode
episode-row-predicted = Predicted
episode-row-extra = Extra
episode-row-resume = Resume at { $time }

## EpisodeCubes.qml

episode-cubes-one = One episode
episode-cubes-range =
    { $count ->
        [one] { $count } episode, { $first } to { $last }
       *[other] { $count } episodes, { $first } to { $last }
    }
# $state is a cube state as it reads mid-sentence, the .inline form of a tokens-cube-* message.
episode-cubes-tip = Episode { $number }: { $state }

## EpisodesPane.qml

episodes-pane-extras =
    { $count ->
        [one] { $count } extra
       *[other] { $count } extras
    }
episodes-pane-unmatched =
    { $count ->
        [one] { $count } unmatched file
       *[other] { $count } unmatched files
    }
episodes-pane-on-disk = On disk
episodes-pane-not-on-disk = Not on disk
episodes-pane-season = S{ $season }
episodes-pane-get-tip = Get episode { $number }
episodes-pane-none-on-disk = No episodes on disk
episodes-pane-change-match = Change match
episodes-pane-refreshing = Refreshing { $title }

## ShowDetail.qml

show-detail-not-in-library = This folder is not in the library yet. Once a scan finds it, its poster and episodes show here.
show-detail-film-on-disk = The film is on disk.
show-detail-film-not-on-disk = The film is not on disk yet.
show-detail-complete-line = Finished airing, and all { $total } episodes are here. Nothing more will come, so the subscription has nothing left to do.
show-detail-finished = Finished airing.
show-detail-airing = Airing.
show-detail-have-of-total = { $have } of { $total } episodes here.
show-detail-have =
    { $count ->
        [one] { $count } episode here.
       *[other] { $count } episodes here.
    }
# $span is "Episode 7" or "Episodes 7 to 9"; $count is 1 for one episode, more for several
show-detail-next-predicted =
    { $count ->
        [one] { $span } is predicted in { $wait }.
       *[other] { $span } are predicted in { $wait }.
    }
show-detail-next-airs =
    { $count ->
        [one] { $span } airs in { $wait }.
       *[other] { $span } air in { $wait }.
    }
# $list is the episode numbers, joined with commas
show-detail-missing =
    { $count ->
        [one] Episode { $list } is not here.
       *[other] Episodes { $list } are not here.
    }
show-detail-missing-more = Episodes { $list } and { $more } more are not here.
show-detail-downloaded = Downloaded
show-detail-downloading = Downloading { $percent }%
show-detail-taken-adding = Taken by the rule, adding
show-detail-taken = Taken by the rule
show-detail-left-out = Left out by the rule
show-detail-subscribed-reading = Subscribed. qBittorrent is reading the feed.
show-detail-taken-moving = The rule takes { $taken } of the feed's { $releases ->
        [one] { $releases } release
       *[other] { $releases } releases
    }; { $moving ->
        [one] { $moving } is
       *[other] { $moving } are
    } downloading below.
show-detail-taken-held = The rule takes { $taken } of the feed's { $releases ->
        [one] { $releases } release
       *[other] { $releases } releases
    }; { $torrents ->
        [one] { $torrents } torrent is
       *[other] { $torrents } torrents are
    } in the client.
show-detail-taken-adding-line = The rule takes { $taken } of the feed's { $releases ->
        [one] { $releases } release
       *[other] { $releases } releases
    }; qBittorrent is adding { $taken ->
        [one] it.
       *[other] them.
    }
show-detail-feed-holds-nothing = The feed holds nothing yet. Releases that fit download as soon as they appear in it.
show-detail-nothing-fits = Nothing in the feed fits the rule yet. Releases that fit download as soon as they appear.
show-detail-complete = Complete
show-detail-open-in-library = Open in library
show-detail-feed-finds-nothing = This feed finds nothing
show-detail-feed-empty-query = It asks { $source } for “{ $query }”, which comes back empty, so nothing new can reach the rule. A shorter search usually answers; the words it drops can stay in the rule instead.
show-detail-feed-empty = It asks { $source }, which comes back empty, so nothing new can reach the rule. A shorter search usually answers; the words it drops can stay in the rule instead.
show-detail-feed-empty-earlier = qBittorrent still shows what an earlier version of this feed found.
show-detail-repairing = Repairing
show-detail-repair = Repair
show-detail-unsubscribe = Unsubscribe
show-detail-remove-torrents =
    { $count ->
        [one] Remove the torrent
       *[other] Remove { $count } torrents
    }
show-detail-searches = Searches
show-detail-quoted = “{ $words }”
show-detail-whole-feed = the whole feed
show-detail-from = From
show-detail-rule = Rule
show-detail-rule-unfollowable = Cannot fetch new episodes: its source answers searches but has no feed
show-detail-rule-active = Active
show-detail-keeps = Keeps
show-detail-keeps-value = releases with “{ $words }”
show-detail-drops = Drops
# $words is the quoted words, joined with show-detail-or
show-detail-drops-value = releases with { $words }
show-detail-or = { $first } or { $second }
show-detail-last-took = Last took
show-detail-saves-to = Saves to
show-detail-folder = Folder
show-detail-in-the-feed = In the feed
show-detail-feed-reading = Reading the feed.
show-detail-feed-offers = What the feed offers now, newest first.
show-detail-feed-is-empty = The feed is empty.
show-detail-read-again = Read again
show-detail-seeders =
    { $count ->
        [one] { $count } seeder
       *[other] { $count } seeders
    }
show-detail-download = Download
show-detail-torrents = Torrents
show-detail-torrents-moving =
    { $count ->
        [one] { $count } downloading now; the bars fill as they go.
       *[other] { $count } downloading now; the bars fill as they go.
    }
show-detail-torrents-held = What the client holds for this folder.
show-detail-torrents-waiting = Waiting for qBittorrent to add what the rule takes.
show-detail-torrents-none = The client holds none for this folder.

## SeriesPage.qml

series-trackers-both = { $first } and { $second }
series-count-cleared = Episode count cleared
series-count-set = Set to { $count } episodes
series-held-one-undated = Episode { $first } not dated yet
series-held-range-undated = Episodes { $first } to { $last } not dated yet
# $when is a month and year (series-held-month-format) or a year alone
series-held-one-due = Episode { $first } due in { $when }
series-held-range-due = Episodes { $first } to { $last } due in { $when }
series-held-month-format = MMMM yyyy
series-fix-schedule = Episodes and dates
series-fix-files = Files
series-fix-rescan-folder = Rescan folder
series-fix-match = Match
series-find-match = Find a match
series-rescan-show = Rescan show
series-open-in-show-info = Open in Show info
# $kind is what the picks are: film, episode or file
series-delete-done =
    { $kind ->
        [film] { $count ->
            [one] { $count } film moved to the trash
           *[other] { $count } films moved to the trash
        }
        [episode] { $count ->
            [one] { $count } episode moved to the trash
           *[other] { $count } episodes moved to the trash
        }
       *[file] { $count ->
            [one] { $count } file moved to the trash
           *[other] { $count } files moved to the trash
        }
    }
series-delete-picked =
    { $kind ->
        [film] { $count ->
            [one] { $count } film picked.
           *[other] { $count } films picked.
        }
        [episode] { $count ->
            [one] { $count } episode picked.
           *[other] { $count } episodes picked.
        }
       *[file] { $count ->
            [one] { $count } file picked.
           *[other] { $count } files picked.
        }
    }
series-delete-move-picked =
    { $kind ->
        [film] { $count ->
            [one] Move { $count } film to the trash
           *[other] Move { $count } films to the trash
        }
        [episode] { $count ->
            [one] Move { $count } episode to the trash
           *[other] Move { $count } episodes to the trash
        }
       *[file] { $count ->
            [one] Move { $count } file to the trash
           *[other] Move { $count } files to the trash
        }
    }
series-delete-click = Click the files to delete.
series-delete-where = They go to the trash, where you can get them back. What you watched stays tracked.
series-delete-move = Move to the trash
series-pick-none = Pick none
series-pick-all = Pick all
series-rescan-started = Rescan started
series-rated = Rated { $score }
series-rated-cleared = Rated cleared
series-score-failed = Score failed
series-still-downloading = { $episode } is still downloading
series-format-series = Series
series-tracker-refused = { $tracker } did not take it
series-tracker-refused-why = { $tracker } did not take it: { $why }
series-watched-up-to = Watched up to { $episode }
series-unwatched-from = Unwatched from { $episode }
series-undo = Undo
series-play-downloading = Downloading episode { $number }
series-play-downloading-unnumbered = Downloading episode
series-play-continue = Continue
series-play-again = Watch again
series-play-continue-episode = Continue episode { $number }
series-play-again-from = Watch again from episode { $number }
series-play-episode = Play episode { $number }
series-page-not-tracked = Not tracked
series-not-tracked-on-disk =
    { $count ->
        [one] Not tracked, { $count } episode on disk
       *[other] Not tracked, { $count } episodes on disk
    }
series-watched = { $count } watched
series-watched-of = { $watched } of { $total } watched
series-watched-of-estimate = { $watched } of { $total }+ watched
series-facts-episodes-estimate =
    { $count ->
        [one] { $count } episode+
       *[other] { $count } episodes+
    }
series-facts-each = { $runtime } each
series-facts-on-disk = { $count } on disk
series-next = Next: { $episode }
series-episodes-downloading =
    { $count ->
        [one] { $count } episode downloading
       *[other] { $count } episodes downloading
    }
series-all-watched = All watched
series-no-poster = No poster
# $span is "Episode 7" or "Episodes 7 to 9"
series-next-season-span = S{ $season } { $span }
series-next-predicted = { $span } predicted for { $date }, in { $wait }
series-next-date-format = ddd d MMM
series-next-airs =
    { $count ->
        [one] { $span } airs in { $wait }
       *[other] { $span } air in { $wait }
    }
series-downloading-episodes =
    { $count ->
        [one] Downloading { $count } episode
       *[other] Downloading { $count } episodes
    }
series-count-season-ends = Season { $season } ends at episode { $number }
series-count-ends = Ends at episode { $number }
series-count-gave-way = You set { $count } episodes; that no longer applies
# $line is what the user set, the parts joined with " · "
series-count-set-by-you = { $line }, set by you
series-change = Change
series-get-episodes = Get episodes
series-your-score = Your score { $score }
series-rate = Rate
series-page-average-score = Average score { $score }
series-page-imdb = IMDb { $score }
series-page-imdb-votes = { $votes } votes
series-add-to-list = Add to a list
series-page-rewatched =
    { $count ->
        [one] Rewatched { $count } time
       *[other] Rewatched { $count } times
    }
series-your-notes = Your notes
series-add-notes = Add notes
# $sites is AniList, MyAnimeList, or both
series-notes-tip = Notes on your list entry, kept on { $sites }
series-unhide = Unhide
series-hide = Hide
series-hidden-toast = { $title } is hidden here, and AniBeam stops syncing it to AniList and MyAnimeList
series-unhidden-toast = { $title } is back, and syncs again
series-hide-tip-tracked = Hidden from every page here, and AniBeam stops syncing it to AniList and MyAnimeList
series-hide-tip = Hidden from every list
series-fixes = Fixes
series-fixes-tip = Fix files, the episode count, the match, or rescan
series-quick-fix = Quick fix
series-film = Film
series-episodes = Episodes
series-extras = Openings, endings and more
series-extras-openings = Openings
series-extras-endings = Endings
series-extras-previews = Previews and trailers
series-extras-specials = Specials
series-extras-other = Other
# $kind is one of the series-extras- names above
series-extras-group = { $kind }, { $count }
series-extra-files = Extra files
series-extra-files-warning =
    { $count ->
        [one] One file is more than the { $total } expected episodes: misnamed, duplicates or specials. Rename or remove it.
       *[other] { $count } files are more than the { $total } expected episodes: misnamed, duplicates or specials. Rename or remove them.
    }
series-synopsis = Synopsis
series-less = Less
series-more = More
series-match = Match
series-not-matched = Not matched yet, so there is no poster, synopsis or tracking. Find its show info on AniList or TVmaze.
series-naming-title = Names that match
series-naming-body = The folder is the show's title, the way AniList writes it, with the season after it when it is not the first. Each file carries the episode number.
series-fix-how-files-read = Fix how files read
series-characters = Characters
series-related = Related
series-related-filling = Still filling in from AniList.
series-tags = Tags
series-hide-spoilers = Hide spoilers
series-spoilers = Spoilers
series-all-tags = All tags
series-tag = { $name }, { $rank }%
series-recommendations = Recommendations
series-relation-owned = In your library
series-relation-unreleased = Not yet released
series-relation-on-anilist = On AniList
series-score-saved = Score saved
series-score-cleared = Score cleared

## GetShow.qml

get-show-the-client = the torrent client
get-show-group-batch = { $group }'s batch
get-show-run = { $first } to { $last }
# $list is the episode numbers and runs (get-show-run), joined with commas
get-show-episodes =
    { $count ->
        [one] Episode { $list }
       *[other] Episodes { $list }
    }
# The same, in the middle of a sentence
get-show-episodes-inline =
    { $count ->
        [one] episode { $list }
       *[other] episodes { $list }
    }
get-show-sending-to = Sending to { $client }
get-show-go =
    { $count ->
        [one] Get { $count } episode
       *[other] Get { $count } episodes
    }
get-show-go-replace =
    { $count ->
        [one] Get { $count } episode and replace
       *[other] Get { $count } episodes and replace
    }
get-show-go-subscribe =
    { $count ->
        [one] Get { $count } episode and subscribe
       *[other] Get { $count } episodes and subscribe
    }
get-show-go-replace-subscribe =
    { $count ->
        [one] Get { $count } episode and replace and subscribe
       *[other] Get { $count } episodes and replace and subscribe
    }
get-show-subscribe = Subscribe for new episodes
get-show-nothing-new = Nothing new to get
get-show-pick-episode = Pick an episode
# $episodes is get-show-episodes-inline
get-show-replace-off = Off. The picks for { $episodes } are left out, since the library has them. Turn it on to swap them.
get-show-replace-on = The files you have for { $episodes } go to the trash once the new ones finish.
# $episodes is get-show-episodes
get-show-replace-on-the-way =
    { $count ->
        [one] { $episodes } is downloading now: that download is cancelled and what it wrote goes to the trash.
       *[other] { $episodes } are downloading now: that download is cancelled and what it wrote goes to the trash.
    }
get-show-kind-on-demand = Started when a search needs it
get-show-kind-rss-search = RSS search
get-show-kind-rss-feed = RSS feed
get-show-kind-json-search = JSON search
get-show-kind-indexer-search = Indexer search
get-show-ready = Ready
get-show-running = Running
get-show-not-running = Not running
get-show-not-running-start = Not running. Click its source above to start it.
get-show-switched-off = Switched off
get-show-click-to-see = Click to see what it has for this show
get-show-searching = Searching
get-show-searching-source = Searching { $name }
get-show-searching-elapsed = { $label }, { $seconds } s
get-show-releases =
    { $count ->
        [one] { $count } release
       *[other] { $count } releases
    }
get-show-releases-covered =
    { $count ->
        [one] { $count } release, { $cover } of { $total } episodes covered
       *[other] { $count } releases, { $cover } of { $total } episodes covered
    }
get-show-on-your-list = On your list
get-show-plan-to-watch = Plan to watch
get-show-on-your-list-tip = It is on your list
get-show-plan-to-watch-tip = Put it on your Planning list
get-show-open-series-tip = The series page: its details, episodes and playing
get-show-next-in = Episode { $number } in { $wait }
get-show-airing = Airing
get-show-finished = Finished
get-show-total-episodes = { $count } episodes
get-show-downloading-episodes =
    { $count ->
        [one] Downloading { $count } episode
       *[other] Downloading { $count } episodes
    }
get-show-downloading-torrents =
    { $count ->
        [one] Downloading { $count } torrent
       *[other] Downloading { $count } torrents
    }
get-show-in-library = In your library
get-show-not-in-library = Not in your library
get-show-downloading = Downloading
get-show-stop-all = Stop all
get-show-start-all = Start all
get-show-stop-all-tip = Stop every download of this show; what they have stays
get-show-start-all-tip = Start every download of this show again
get-show-cancel-all = Cancel all
get-show-cancel-all-tip = Cancel every download of this show and move what they wrote to the trash
get-show-sources = Available sources
get-show-check-again = Check again
get-show-check-again-tip = Asks each on-demand provider whether it is running. Starts nothing.
get-show-search-source = Search { $name }
get-show-release-list = Every release as a list
get-show-your-library = Your library
get-show-library-on-disk = { $have } of { $total } on disk
get-show-library-downloading = { $count } downloading
get-show-library-nothing = Nothing yet
get-show-row-batch = { $group }, batch
get-show-batch-counting = counting its episodes, one torrent
get-show-batch-unlisted = episodes not listed, the whole show by its title, one torrent
get-show-batch-episodes =
    { $count ->
        [one] { $count } episode, one torrent
       *[other] { $count } episodes, one torrent
    }
get-show-nothing-new-picked = Nothing new picked
get-show-nothing-picked = Nothing picked
get-show-pick-provider = Pick a provider
# $groups is the release groups, joined with get-show-and
get-show-plan-line = { $episodes } from { $groups } on { $source }.
get-show-and = { $first } and { $second }
get-show-library-has =
    { $count ->
        [one] The library has { $episodes }. Turn on Replace current episodes below to swap it.
       *[other] The library has { $episodes }. Turn on Replace current episodes below to swap them.
    }
get-show-together = together
get-show-torrents-unit =
    { $count ->
        [one] torrent
       *[other] torrents
    }
get-show-replace = Replace current episodes
get-show-sequential = Download in order
get-show-sequential-on = On: each new torrent fetches its pieces from the start, so the first episodes finish first and you can watch while the rest comes. A little slower overall.
get-show-sequential-off = Off: the client fetches pieces in whatever order the swarm offers them.
get-show-rolling = Keep the next few ready
get-show-rolling-on = On: { $count } episodes download now, and each one you finish starts the next.
get-show-rolling-on-delete = On: { $count } episodes download now, and each one you finish starts the next. Watched episodes leave the disk.
get-show-rolling-off = Off: every picked episode downloads at once.
get-show-follow-off = Off: new episodes show up here as gaps, and nothing downloads by itself.
get-show-follow-no-feed = { $source } has no feed to follow, so new episodes cannot download by themselves from it.
get-show-follow-pick-group = Pick a group first: a subscription follows one group and quality.
get-show-follow-batch-only = { $group } has only the batch here, so there is no feed of single episodes to subscribe to.
# $feed is the release group and its quality
get-show-follow-on = On: a feed for { $feed } on { $source }. Each new episode downloads as it appears.
get-show-confirm =
    { $count ->
        [one] { $episodes } is downloading now. Replacing it cancels that download and moves what it wrote to the trash.
       *[other] { $episodes } are downloading now. Replacing them cancels that download and moves what it wrote to the trash.
    }
get-show-confirm-go = Cancel it and replace
get-show-state-error = Error
get-show-state-files-missing = Files missing
get-show-state-stopped = Stopped
get-show-state-waiting-peers = Waiting for peers
get-show-state-metadata = Fetching metadata
get-show-state-queued = Queued
get-show-state-checking = Checking files
get-show-state-allocating = Allocating
get-show-state-moving = Moving files
get-show-state-waiting = Waiting
get-show-a-batch = A batch
get-show-no-group = Group not in the name
get-show-likely = likely { $source }
get-show-tracker = tracker { $tracker }
get-show-not-known = Not known yet
get-show-bytes-of = { $done } of { $size }
get-show-speed-down = { $speed }/s down
get-show-time-left = { $wait } left
get-show-stopping = Stopping
get-show-starting = Starting
get-show-stop = Stop
get-show-start-tip = Start downloading again
get-show-stop-tip = Stop downloading; what it has stays
get-show-cancelling = Cancelling
get-show-cancel-tip = Cancel this download and move what it wrote to the trash
get-show-episode-order = Download in episode order
get-show-episode-order-on = On: pieces come from the start of each file, so the first episodes finish first.
get-show-cube-skipped = Episode { $number }: skipped
get-show-cube-done = Episode { $number }: done
get-show-cube-percent = Episode { $number }: { $percent }%
get-show-fact-torrent = Torrent
get-show-fact-release = Release
get-show-fact-from = From
get-show-fact-saving-to = Saving to
get-show-span-counting = Counting
get-show-span-unknown = { $runs }?
get-show-span-episodes = Episodes { $runs }
get-show-cube-no-release = Episode { $number }: no release from { $group }
get-show-seeders = { $count } seeders
get-show-cube-on-disk = already on disk, click to replace it
get-show-cube-downloading = downloading now, click to replace it

## TorrentRow

torrent-row-state-downloading = Downloading
torrent-row-state-stalled = Stalled
torrent-row-state-metadata = Fetching metadata
torrent-row-state-queued = Queued
torrent-row-state-stopped = Stopped
torrent-row-state-done = Done
torrent-row-state-seeding = Seeding
torrent-row-state-checking = Checking
torrent-row-state-moving = Moving
torrent-row-state-error = Error
torrent-row-state-files-missing = Files missing
torrent-row-state-making-room = Making room
torrent-row-state-finding-peers = Finding peers
torrent-row-state-waiting = Waiting
torrent-row-done-of-size = { $done } of { $size }
torrent-row-speed-down = { $speed }/s down
torrent-row-speed-up = { $speed }/s up
torrent-row-time-left = { $time } left
torrent-row-seeders =
    { $count ->
        [one] { $count } seeder
       *[other] { $count } seeders
    }
torrent-row-rolling = rolling, episodes { $next } to { $through } of { $total }
torrent-row-stop = Stop

## SourceRow

source-row-unavailable-fix = Plug in its drive, then Rescan. If it moved, remove it and add the new place.
source-row-counts = { $count } series
source-row-counts-movies =
    { $movies ->
        [one] { $count } series, { $movies } movie folder
       *[other] { $count } series, { $movies } movie folders
    }
source-row-unavailable = Unavailable
    .tip = AniBeam cannot reach this folder. If it is on a drive that is not plugged in or not mounted, connect it and AniBeam picks it up again. If it moved, remove it here and add the new place.
source-row-rescan = Rescan
source-row-remove-question = Remove { $name }, { $count } series and their history?

## StartSourceSheet

start-source-title = Start { $service } for { $name }?
start-source-busy = Starting { $service }. { $name } switches on when it answers; this can take up to a minute.
start-source-body = { $service } is the service that searches { $name } for AniBeam. Starting it runs it in the background on this computer, and it keeps running after AniBeam closes.
start-source-starting = Starting

## SearchProgress

search-progress-why-helper = AniBeam is starting its search helper; this takes a second or two.
search-progress-why-starting = { $endpoint } runs through a helper that starts when a search needs it. The first search waits for it, up to { $seconds } seconds; the ones after skip this.
search-progress-why-reading = Splitting the releases into groups and checking them against the library.
search-progress-why-ondemand = A helper reads { $endpoint } afresh for every new search, which usually takes ten to twenty seconds.
search-progress-why-slow = { $endpoint } is slow to answer right now.
search-progress-why-usual = This usually takes a second or two.
search-progress-why-and-slow = { $why } This is taking longer than usual; cancel to try another source.
search-progress-searching = Searching
search-progress-seconds = { $seconds } s
search-progress-try = Try { $endpoint }

## ReleaseTable

release-table-col-release = Release
release-table-col-group = Group
release-table-col-res = Res
release-table-col-rip = Rip
release-table-col-ep = Ep
release-table-col-size = Size
release-table-col-seeders = Seeders
release-table-col-leechers = Leechers
release-table-col-downloads = Downloads
release-table-col-released = Released
release-table-tick-all = Tick every release
release-table-untick-all = Untick every release
release-table-on-disk = On disk
release-table-other-on-disk = Other release on disk
release-table-season-batch = Season { $season } batch
release-table-batch = Batch
release-table-via = via { $source }
release-table-season-episode = S{ $season } { $episode }
release-table-date-format = d MMMM yyyy, HH:mm
release-table-download = Download
release-table-more = and { $count } more. Narrow the list or sort it to see others first.
release-table-pick = Pick this release

## DownloadsPage

downloads-title = Downloads
downloads-builtin-client = Built-in client
downloads-start-done = { $name } is running and switched on
downloads-disk-film = Film
downloads-disk-watched = { $size } watched
downloads-disk-all-watched = all watched
downloads-delete-watched-menu =
    { $count ->
        [one] Every watched episode: { $count } episode, { $size }
       *[other] Every watched episode: { $count } episodes, { $size }
    }
downloads-delete-finished-menu =
    { $count ->
        [one] Every finished show you watched: { $count } show, { $size }
       *[other] Every finished show you watched: { $count } shows, { $size }
    }
downloads-titles-and = { $list } and { $last }
downloads-titles-more = { $list } and { $count } more
downloads-delete-finished-none = No finished show is watched all the way through
downloads-delete-watched-none = Nothing watched is on disk
downloads-delete-finished-heading =
    { $count ->
        [one] Delete { $count } finished show, { $size }?
       *[other] Delete { $count } finished shows, { $size }?
    }
downloads-delete-finished-base = { $titles }. Only shows the provider says have finished, every episode watched; a show still airing, or whose end is a guess, stays. Every file of them goes to the trash, extras too. Their torrents leave { $client } too, so they can be downloaded again.
downloads-delete-watched-heading =
    { $count ->
        [one] Delete { $count } watched episode, { $size }?
       *[other] Delete { $count } watched episodes, { $size }?
    }
downloads-delete-watched-base = Every episode you have watched, in every show. Episodes not watched yet and extras stay. The files and their subtitles go to the trash, so you can get them back from there. Their torrents leave { $client } too, so they can be downloaded again.
downloads-move-to-trash = Move to trash
downloads-remove-group-torrents =
    { $count ->
        [one] Remove the torrent
       *[other] Remove its { $count } torrents
    }
downloads-run-span = { $from } to { $to }
downloads-run-pair = { $first } and { $second }
downloads-gap-missing =
    { $count ->
        [one] Episode { $runs } missing
       *[other] Episodes { $runs } missing
    }
downloads-gap-not-downloaded =
    { $count ->
        [one] Episode { $runs } not downloaded
       *[other] Episodes { $runs } not downloaded
    }
downloads-gap-then-not-downloaded = { $runs } not downloaded
downloads-trying-query = Trying { $query }
downloads-client-settings = Torrent client settings
downloads-error-unknown = Something went wrong. Try again.
downloads-search-all-off = Every source is switched off. Switch one on to search.
downloads-turn-on = Turn on { $name }
downloads-search-no-sources = There are no sources to search yet. Add a source list in Settings, under Downloads.
downloads-search-none-started = No source could start the search. Try again in a moment.
downloads-auto-resolution = Narrowed to { $resolution }, the sharpest on offer
downloads-search-one-failed = The search on { $name } did not work. Try again, or search another source.
downloads-search-all-failed = None of the sources could search. Try again in a moment.
downloads-narrowing-on = Narrowing on { $name }
downloads-refine-zero = That would leave nothing, so the list stays as it is.
downloads-refine-no-match = No release matches that term.
downloads-refine-empty = Type a term to narrow by.
downloads-refine-no-change = That would change nothing.
downloads-movie-tick-one = Tick one release, then press Download as a movie.
downloads-group-downloading = Downloading
downloads-group-downloading-line = Moving now; Activity at the right has the detail.
downloads-group-seeding = Seeding
downloads-group-seeding-line = Downloaded and still sharing.
downloads-group-done = Done
downloads-group-done-line = Downloaded; the client is not sharing them.
downloads-group-stopped = Stopped
downloads-group-stopped-line = Stopped before they finished.
downloads-group-broken = Files missing
downloads-group-broken-line = The client cannot find their files: they point at a folder that is not there. Removing the torrents tidies the client.
downloads-sub-nofeed = Cannot follow
downloads-sub-nofeed-line = Their source answers searches but has no feed, so no new episodes arrive. Unsubscribe, then follow the show on a source with one.
downloads-sub-complete = Complete
downloads-sub-complete-line = Every episode is in the library and nothing more will air: these subscriptions have nothing left to do.
downloads-sub-airing = Airing
downloads-sub-airing-line = New episodes still to come, the soonest first.
downloads-sub-incomplete = Incomplete
downloads-sub-incomplete-line = Finished airing with episodes not here, or nothing says where the show stands.
downloads-sub-missing = Not in the library
downloads-sub-missing-line = Their folder holds nothing the library has found: nothing has arrived yet, or it saves where the library does not look.
downloads-torrents =
    { $count ->
        [one] { $count } torrent
       *[other] { $count } torrents
    }
downloads-rule-film-on-disk = Film, on disk
downloads-rule-all-here = All { $total } here
downloads-rule-next =
    { $predicted ->
        [yes] Ep { $episode } predicted in { $time }
       *[no] Ep { $episode } in { $time }
    }
downloads-rule-next-span =
    { $predicted ->
        [yes] Ep { $episode } to { $through } predicted in { $time }
       *[no] Ep { $episode } to { $through } in { $time }
    }
downloads-rule-have-of = { $have } of { $total } here
downloads-rule-have =
    { $count ->
        [one] { $count } episode here
       *[other] { $count } episodes here
    }
downloads-tag-not-downloaded = Not downloaded
downloads-tag-missing = { $count } missing
downloads-tag-n-not-downloaded = { $count } not downloaded
downloads-tag-unknown = Unknown
downloads-speed-down = { $speed }/s down
downloads-time-left = { $time } left
downloads-repair-trying = Trying shorter searches
downloads-sent-to-client = Sent { $title } to { $client }
downloads-files-torrents = what the torrents downloaded
downloads-files-torrents-remote = what the torrents downloaded, deleted by { $client }
downloads-files-folder-and = { $folder } and { $torrents }
downloads-unsubscribe-heading = Unsubscribe from { $title }?
downloads-unsubscribe-base = Its feed and download rule leave { $client }, so nothing new arrives.
downloads-unsubscribe-complete-heading = Unsubscribe from all { $count } complete shows?
downloads-unsubscribe-complete-base = Their feeds and download rules leave { $client }. Every episode of them is in the library.
downloads-files-folders-and-torrents = their folders and what their torrents downloaded
downloads-one-at-a-time = They go one at a time; the page shows each as it goes.
downloads-subscription-stays = { $base } The subscription stays.
downloads-cancel-heading = Cancel { $title }?
downloads-cancel-group-base =
    { $count ->
        [one] Its download stops and leaves { $client }.
       *[other] Its { $count } downloads stop and leave { $client }.
    }
downloads-files-so-far-it = what it has downloaded so far
downloads-files-so-far-they = what they have downloaded so far
downloads-cancel-download = Cancel the download
downloads-keep-downloading = Keep downloading
downloads-remove-heading = Remove { $title }?
downloads-remove-group-base =
    { $count ->
        [one] Its torrent leaves { $client }; the files stay unless you tick below.
       *[other] Its { $count } torrents leave { $client }; the files stay unless you tick below.
    }
downloads-remove-detail-base =
    { $count ->
        [one] Its { $count } torrent leaves { $client }; the files stay unless you tick below.
       *[other] Its { $count } torrents leave { $client }; the files stay unless you tick below.
    }
downloads-remove-broken-heading = Remove the torrents of all { $count } shows with files missing?
downloads-remove-broken-base = Their { $count } torrents leave { $client }. They point at folders that are not there, so no file is touched; subscriptions stay.
downloads-remove-torrent-heading = Remove this torrent?
downloads-files-it-downloaded = what it downloaded
downloads-cancel-torrent-heading = Cancel this download?
downloads-cancel-many-heading =
    { $count ->
        [one] Cancel { $count } download?
       *[other] Cancel { $count } downloads?
    }
downloads-cancel-many-base = Every download of this show stops and leaves { $client }.
downloads-cancel-all = Cancel them all
downloads-summary-connecting = Connecting.
downloads-summary-client-on = { $client } { $version } is connected.
downloads-summary-client-off = { $client } is not connected. Check its address and password in Settings, under Downloads.
downloads-summary-native = AniBeam searches your sources itself.
downloads-summary-native-with = AniBeam searches your sources itself. { $client }
downloads-summary-starting = Starting downloads.
downloads-summary-not-running = Downloads is not running.
downloads-summary-not-started = Downloads has not started yet.
downloads-summary-helper-ready = The search helper { $version } is ready. { $client }
downloads-subscribed-as = Subscribed as { $name }.
downloads-repaired =
    { $count ->
        [one] Repaired { $name }: the feed finds { $count } release
       *[other] Repaired { $name }: the feed finds { $count } releases
    }
downloads-repaired-keeps =
    { $count ->
        [one] Repaired { $name }: the feed finds { $count } release and the rule keeps “{ $term }”
       *[other] Repaired { $name }: the feed finds { $count } releases and the rule keeps “{ $term }”
    }
downloads-sent-torrents =
    { $count ->
        [one] Sent { $count } torrent to { $client }.
       *[other] Sent { $count } torrents to { $client }.
    }
downloads-added-torrents =
    { $count ->
        [one] Added { $count } torrent.
       *[other] Added { $count } torrents.
    }
downloads-trashed-paths =
    { $count ->
        [one] { $count } path of { $title } moved to the trash
       *[other] { $count } paths of { $title } moved to the trash
    }
downloads-show-gone = That show is no longer in the library
downloads-activity-hide = Hide activity
downloads-activity = Activity
downloads-moving =
    { $count ->
        [one] { $count } download moving
       *[other] { $count } downloads moving
    }
downloads-back = Back to Downloads
downloads-chip-search-ready = Search ready
downloads-chip-search-starting = Search starting
downloads-chip-search-off = Search off
downloads-chip-answers-at = { $summary } It answers at { $url }.
downloads-chip-helper-missing = The search helper is not installed. Click for the Downloads settings, which say how to add it
downloads-chip-helper-failed = The search helper stopped. Reconnect starts it again; the Downloads settings say what went wrong
downloads-chip-open-settings = Open the Downloads settings
downloads-chip-client-on = { $client } { $version }
downloads-chip-client-off = { $client } off
downloads-chip-client-not-connected = { $client } is not connected. Reconnect, or check its address in Settings
downloads-chip-client-remote = A search helper you run yourself uses the client its own settings name
downloads-chip-client-switch = Click to switch clients
downloads-connecting = Connecting
downloads-reconnect = Reconnect
downloads-search-placeholder = Search for a show, or a release name
downloads-search-go-tip = Search release names on { $names }
downloads-search-go-off = Every source is switched off.
downloads-suggest-airing =
    { $predicted ->
        [yes] Airing, episode { $episode } predicted in { $time }
       *[no] Airing, episode { $episode } in { $time }
    }
downloads-suggest-airing-span =
    { $predicted ->
        [yes] Airing, episodes { $episode } to { $through } predicted in { $time }
       *[no] Airing, episodes { $episode } to { $through } in { $time }
    }
downloads-suggest-finished = Finished
downloads-suggest-episodes = { $count } episodes
downloads-suggest-in-library = In your library
downloads-suggest-on-anilist = On AniList, not in your library
downloads-plan-to-watch = Plan to watch
downloads-suggest-release-search = Search release names for “{ $query }” on { $names } instead
downloads-suggest-release-search-none = Search release names for “{ $query }” on no source instead
downloads-key-return = Return
downloads-sources = Sources
downloads-source-hint-starting = starting
downloads-source-hint-down = not running
downloads-source-tip-down = { $name } needs a helper service that is not running. Click to start it.
downloads-source-tip-on = Searches ask { $name }. Click to switch it off.
downloads-source-tip-off = Switched off. Click to ask { $name } again.
downloads-source-tip-url = { $tip } { $url }
downloads-check = Check
    .tip = Asks whether each on-demand source is running. Starts nothing.
downloads-check-busy = Checking
downloads-back-to-grid = Back to the grid
    .tip = Back to { $title } and what each provider found. Escape does the same.
downloads-releases-on =
    { $count ->
        [one] { $count } release on { $name }
       *[other] { $count } releases on { $name }
    }
downloads-found-on = Found on
downloads-found-searching = searching
downloads-found-failed = failed
downloads-found-cancelled = cancelled
downloads-releases =
    { $count ->
        [one] { $count } release
       *[other] { $count } releases
    }
downloads-found-retry-tip =
    { $why }
    Click to try again
downloads-facet-groups = Release group
downloads-facet-quality = Quality
downloads-facet-words = Words
downloads-filters-fewer = Fewer filters
downloads-filters-more = More filters ({ $count })
downloads-narrow-placeholder = Narrow by a term, or -term to drop
downloads-best-match = Best match
    .tip = Keeps the releases that look like the best one: { $query }
downloads-best-match-none = No release stands out here
downloads-strip-have-of = { $have } of { $total } episodes on disk
downloads-strip-have =
    { $count ->
        [one] { $count } episode on disk
       *[other] { $count } episodes on disk
    }
downloads-strip-next =
    { $predicted ->
        [yes] episode { $episode } predicted in { $time }
       *[no] episode { $episode } in { $time }
    }
downloads-strip-next-span =
    { $predicted ->
        [yes] episodes { $episode } to { $through } predicted in { $time }
       *[no] episodes { $episode } to { $through } in { $time }
    }
downloads-strip-complete = complete
downloads-strip-line-more = { $disk }, { $more }.
downloads-sentence = { $text }.
downloads-strip-none-on-disk = None of these releases is for an episode you already have.
downloads-strip-on-disk =
    { $count ->
        [one] { $count } release here is the file you have, marked On disk
       *[other] { $count } releases here are the file you have, marked On disk
    }
downloads-strip-other-on-disk =
    { $count ->
        [one] { $count } release is for an episode you have from another release
       *[other] { $count } releases are for an episode you have from another release
    }
downloads-strip-both = { $first }; { $second }.
downloads-tick-missing =
    { $count ->
        [one] Tick the { $count } missing episode
       *[other] Tick the { $count } missing episodes
    }
    .tip = For each episode here the library does not have, the release with the most seeders
downloads-open-in-library = Open in library
downloads-not-this-show = Not this show
downloads-no-releases = No releases
downloads-no-releases-body = Nothing on { $name } matches “{ $query }”. Another source, or fewer words, may find it.
downloads-try-source = Try { $name }
downloads-ticked =
    { $count ->
        [one] { $count } release ticked
       *[other] { $count } releases ticked
    }
downloads-nothing-ticked = Nothing ticked
downloads-ticked-size = { $size } together.
downloads-tick-hint = Tick releases in the table, or download them all.
downloads-tick-all = Tick all
downloads-working = Working
downloads-download-ticked =
    { $count ->
        [one] Download { $count } release
       *[other] Download { $count } releases
    }
downloads-download-shown = Download the { $count } shown
downloads-download-all = Download all { $count }
downloads-download-movie = Download as a movie
    .tip = One ticked release, straight into the movie folder
downloads-subscribe = Subscribe
downloads-subscribe-body = New releases that match this search download as they appear, into the folder named here.
downloads-name-placeholder = Picked from the first release
downloads-subscribe-search = Subscribe to this search
downloads-subscribe-no-feed = No feed to subscribe to here
downloads-preview-again = Show the rule again
downloads-preview-show = Show the rule it makes
downloads-preview-from = From { $host }
downloads-preview-from-named = From { $host } ({ $name })
downloads-preview-must = Must contain { $terms }
downloads-preview-must-not = Must not contain { $terms }
downloads-helper-changed = Downloads is set to a helper that does not answer
downloads-helper-missing = This copy of AniBeam came without downloads
downloads-helper-failed = Downloads could not start
downloads-helper-changed-body = Searches and downloads run through a small helper program. Settings point at another one than AniBeam's own, and it is not answering. Switch back and it starts by itself.
downloads-helper-missing-body = It was built without the helper program searches and downloads run through. The installers on the releases page have it. Settings, Downloads says how to add it to this copy.
downloads-helper-failed-body = The helper program searches and downloads run through stopped before it was ready. Starting it again fixes this most of the time.
downloads-helper-use-own = Use AniBeam's own
downloads-helper-start-again = Start again
downloads-helper-show-how = Show me how
downloads-helper-what-went-wrong = What went wrong
downloads-lens-gaps-count = Gaps in your library  { $count }
downloads-lens-airing-count = Airing  { $count }
downloads-lens-following-count = Following  { $count }
downloads-lens-downloaded-count = Downloaded  { $count }
downloads-lens-gaps = Gaps in your library
downloads-lens-gaps-line = Aired episodes that are not on disk. Open a show to see who has them.
downloads-lens-gaps-none = Nothing is missing: every aired episode of every show is on disk.
downloads-lens-airing-line = Shows in your library with episodes still to come. Open one to follow it.
downloads-lens-airing-none = Nothing in your library is airing
downloads-lens-airing-none-body = A show lands here once it is in your library and has episodes still to come. Airing now lists everything on air.
downloads-airing-now = Airing now
downloads-lens-following = Following
downloads-card-next =
    { $predicted ->
        [yes] Episode { $episode } predicted in { $time }
       *[no] Episode { $episode } in { $time }
    }
downloads-card-next-span =
    { $predicted ->
        [yes] Episodes { $episode } to { $through } predicted in { $time }
       *[no] Episodes { $episode } to { $through } in { $time }
    }
downloads-rules-unread = Your subscriptions could not be read.
downloads-lens-following-line = Feeds AniBeam watches, as the shows they fill.
downloads-none-yet = None yet.
downloads-reading = Reading.
downloads-rules-error = AniBeam could not read your subscriptions. Try again in a moment.
downloads-removing-to-go = Removing, { $count } to go
downloads-unsubscribe-all = Unsubscribe all { $count }
downloads-unsubscribe = Unsubscribe
downloads-no-rules = No subscriptions
downloads-no-rules-body = Search above and subscribe to keep a show coming.
downloads-search = Search
downloads-lens-downloaded = Downloaded
downloads-dl-line = { $size } on disk. { $rest }
downloads-dl-client-off = { $client } is not connected.
downloads-dl-unread = The download list could not be read.
downloads-dl-in-client =
    { $count ->
        [one] { $count } torrent, { $size } in { $client }.
       *[other] { $count } torrents, { $size } in { $client }.
    }
downloads-usage-watched = Watched
downloads-usage-not-watched = Not watched
downloads-deleting = Deleting
downloads-delete-watched = Delete watched
downloads-remove-all = Remove all { $count }
downloads-on-disk = On disk
downloads-on-disk-line = In your library with no torrent in { $client }, the biggest first. Right-click one to delete it.
downloads-nothing-downloaded = Nothing downloaded
downloads-nothing-downloaded-body = What you download or subscribe to shows up here, a show at a time.
downloads-client-off-title = { $client } is not connected
downloads-client-off-body = Check it is running, then connect again. Its address and password are in Settings, under Downloads.
downloads-connect-again = Connect again
downloads-view-grid = Grid
downloads-view-list = List
downloads-chip-subscribed = Subscribed
downloads-speed-up = { $speed }/s up
downloads-broken-some = { $count } with files missing
downloads-tag-watched = Watched

## PlayerDelays.qml

player-delays-title = Delay
player-delays-subtitles = Subtitles
player-delays-audio = Audio
player-delays-subtitles-earlier = Subtitles 0.1 s earlier
player-delays-subtitles-later = Subtitles 0.1 s later
player-delays-audio-earlier = Audio 0.1 s earlier
player-delays-audio-later = Audio 0.1 s later
player-delays-reset = Reset to 0 until the next file
player-delays-none = No delay

## TrackPicker.qml

track-picker-off = Off

## RollControls.qml

roll-controls-window = Keeping { $ahead } ready: episodes { $next } to { $through } of { $total }
roll-controls-window-watched = Keeping { $ahead } ready: episodes { $next } to { $through } of { $total }, { $watched } watched
roll-controls-more = Get { $count } more
roll-controls-more-tip = Download the next episodes now. AniBeam keeps the next few ready again once you catch up.
roll-controls-fewer-tip = Keep one fewer ahead
roll-controls-ahead = { $count } ahead
roll-controls-further-tip = Keep one more ahead
roll-controls-all = Get all
roll-controls-all-tip = Stop keeping a few ready and download every episode left
roll-controls-all-question = Download all { $count } episodes left?

## PlayerChrome.qml

player-chrome-delays = Subtitle and audio delay
player-chrome-audio = Audio track
player-chrome-subtitles = Subtitles
player-chrome-subtitles-off = Subtitles off
player-chrome-casting-to = Casting to { $device }
player-chrome-cast = Cast to a TV
player-chrome-mark-watched = Mark watched
player-chrome-shortcuts = Keyboard shortcuts
player-chrome-fullscreen = Fullscreen
player-chrome-exit-fullscreen = Exit fullscreen
player-chrome-stay = Stay
player-chrome-skip-intro = Skip intro
player-chrome-skip-outro = Skip outro
player-chrome-playing-on = Playing on { $device }
player-chrome-replay = Replay
player-chrome-watch-together = Watch together
player-chrome-room = Room { $code } · { $count ->
    [one] 1 person
   *[other] { $count } people
}

## Room.qml

room-name-empty = Type the name others in the room will see.
room-name-bad = That name does not work. Use 1 to 32 characters.
room-error-not-found = No room has that code. Check it with whoever sent it; the room may have closed.
room-error-full = That room is full. Rooms take 10 people.
room-error-unreachable = Could not reach the room server. Check your connection and try again.
room-error-refused = The room server turned this down. Try again in a minute.
room-error-proto = That room runs on a newer version of rooms. Update { -app-name } to join it.
room-error-not-in-room = You are no longer in that room. Start or join one again.
room-error-bad-code = A room code is 6 letters and digits, like K7QM2X.
room-error-bad-relay = The room server in Settings, Playback is not a web address. Fix it there and try again.

## RoomPanel.qml

room-panel-title = Watch together
room-panel-close = Close
room-panel-connected = Connected
room-panel-reconnecting = Reconnecting
room-panel-name-label = Your name in rooms
room-panel-name-placeholder = Your name
room-panel-name-start = Start the room
room-panel-name-join = Join the room
room-panel-start = Start a room
room-panel-or-join = Or join one with its code
room-panel-join-label = Room code
room-panel-join = Join
room-panel-you-are = You show as { $name }
room-panel-change-name = Change
room-panel-code-label = Room code
room-panel-copy = Copy link
room-panel-link-copied = Room link copied
room-panel-watching = { $series } · { $episode }
room-panel-missing-here = This episode is not in your library, so you can't play along yet.
room-panel-open-episode = Open the episode
room-panel-people = { $count ->
    [one] 1 person
   *[other] { $count } people
}
room-panel-host = Host
room-panel-you = You
room-panel-ready = Ready
room-panel-opening = Opening
room-panel-missing = Doesn't have it
room-panel-offset = Your offset
room-panel-offset-earlier = Half a second earlier
room-panel-offset-later = Half a second later
room-panel-seconds = { $value } s
room-panel-suggested = The openings suggest { $offset }
room-panel-use-suggested = Use it
room-panel-leave = Leave the room

## PlayerPage.qml

player-title = Player
player-next = Next
player-extra-no-next = An extra has no next episode
player-last-episode = This is the last episode
player-extra-no-previous = An extra has no previous episode
player-first-episode = This is the first episode
player-resumed-at = Resumed at { $time }
player-start-over = Start over
player-audio-track = Audio: { $track }
player-subtitles-track = Subtitles: { $track }
player-subtitles-on = Subtitles: on
player-subtitles-off = Subtitles off
player-no-subtitles = This file has no subtitles
# A delay in seconds, signed: "+0.1 s", "0.0 s"
player-delay-seconds = { $value } s
player-audio-delay = Audio delay { $value }
player-subtitle-delay = Subtitle delay { $value }
player-volume = Volume { $volume }
player-speed = Speed { $speed }×
player-not-while-casting = Not while casting
player-room-speed = Speed stays with the room
player-hold-fast = 2× while held
player-no-chapters = This file has no chapters
player-last-chapter = This is the last chapter
player-chapter = Chapter { $number }
player-chapter-titled = Chapter { $number }: { $title }
player-screenshot-saved = Saved to Pictures/AniBeam
player-skipped-intro = Skipped intro
player-skipped-outro = Skipped outro
player-undo = Undo
# Why a tracker was not updated, read after "Not tracked:" or a tracker's name
player-refusal-hidden = the show is hidden
player-refusal-no-match = the show is not matched yet; match it on Show info
player-refusal-not-newer = your list is already past this episode
player-refusal-extra = this is an extra
player-refusal-unmatched = the file is not matched to an episode
player-refusal-on-disk = the file is only on disk
player-not-tracked-because = Not tracked: { $reason }.
# A tracker and what it said: "AniList: the show is hidden"
player-outcome-said = { $tracker }: { $words }
player-outcome-failed = { $tracker } could not be updated
player-outcome-ok = { $tracker } ok
player-outcome-at = { $tracker } at { $progress }
player-outcome-tracked = Tracked
player-outcome-not-tracked = Not tracked
player-outcome-could-not-update = Could not update your list
# $list is each tracker's outcome joined by "; "
player-outcome-tracked-list = Tracked: { $list }
player-outcome-not-tracked-list = Not tracked: { $list }
player-outcome-failed-list = Could not update your list: { $list }
player-rated = Rated { $score }
player-score-failed = Could not save the score
player-frame = Frame { $number } at { $time }
player-audio-title = Audio
player-subtitles-title = Subtitles
player-cast-to = Cast to
player-cast-looking = Looking for Chromecasts and TVs
player-cast-unreachable = Could not reach the TV
player-cast-tv = TV
player-cast-stop = Stop casting to { $device }
player-cast-connecting = Connecting
player-cast-loading = Loading
player-cast-playing = Playing
player-cast-paused = Paused
player-cast-finished = Finished
player-cast-stopped = Stopped
player-rate-prompt = Tracked the final episode. Rate this show?
player-rate = Rate
player-not-now = Not now

## EndpointReposPanel.qml

endpoint-repos-panel-title = Source lists
endpoint-repos-panel-helper = A source list is a signed set of places to search, added by its address. AniBeam checks the signature and shows you the key before anything is saved, then you install the sources you want. Installed ones join Search sources above.
endpoint-repos-panel-could-not-start = AniBeam could not start that. Try again in a moment.
endpoint-repos-panel-needs-value = { $name } needs a value
endpoint-repos-panel-values-of = { $name }'s values
endpoint-repos-panel-secret-note = Kept in this machine's secret store, never in the library or an export.
endpoint-repos-panel-secret-note-keep = Kept in this machine's secret store, never in the library or an export. A field left empty keeps what is stored.
endpoint-repos-panel-asks-again = { $name } version { $version } asks again
endpoint-repos-panel-update-sends-to = The new version sends this to { $host }.
endpoint-repos-panel-fields-changed = Its fields changed.
# $fields is the field names joined by ", "
endpoint-repos-panel-needs-fields = Needs its { $fields } before it searches.
endpoint-repos-panel-no-longer-offered = No longer offered by its source list.
endpoint-repos-panel-update-requests-to = The update sends requests to { $host }.
endpoint-repos-panel-unreadable = This list could not be read. Refresh lists tries it again.
endpoint-repos-panel-add-list = Add a source list
endpoint-repos-panel-add-list-helper = The address of its index file, or a path to one on this machine.
endpoint-repos-panel-checking = Checking
endpoint-repos-panel-add = Add
endpoint-repos-panel-trust-question = Trust { $name }?
endpoint-repos-panel-fingerprint = Key fingerprint  { $fingerprint }
endpoint-repos-panel-preview =
    { $url }
    { $count } sources, signed. Its sources decide where your searches go; trust a list only from someone you trust.
endpoint-repos-panel-preview-trusted =
    { $url }
    { $count } sources, signed, already trusted. Its sources decide where your searches go; trust a list only from someone you trust.
endpoint-repos-panel-trust = Trust
endpoint-repos-panel-saving = Saving
endpoint-repos-panel-install = Install
endpoint-repos-panel-update = Update
endpoint-repos-panel-added-by-you = added by you
endpoint-repos-panel-version = v{ $version }
endpoint-repos-panel-update-to = Update to v{ $version }
endpoint-repos-panel-add-value = Add its value
endpoint-repos-panel-change = Change { $fields }
endpoint-repos-panel-asks-for = Asks for your { $fields }.
endpoint-repos-panel-by-hand = Add one by hand
endpoint-repos-panel-by-hand-helper = For a site no list carries: any feed with {"{"}query{"}"} where the search goes, one from a search proxy among them. A key in the address is kept apart from it, in the system password store where there is one.
endpoint-repos-panel-name = Name
endpoint-repos-panel-refreshing = Refreshing
endpoint-repos-panel-refresh = Refresh lists

## ScheduleSheet.qml

schedule-episodes-range = Episodes { $first } to { $last }
# Qt date formats: a full day, a month, and the short ones on an episode tile
schedule-day-format = ddd d MMM yyyy
schedule-month-format = MMMM yyyy
schedule-tile-day-format = d MMM
schedule-tile-month-format = MMM yyyy
schedule-some-time-in = Some time in { $year }
schedule-no-date-yet = No date yet
# A date guessed from a weekly run
schedule-tile-guess = ≈ { $date }
schedule-tile-no-date = No date
schedule-tile-last = Last
schedule-tile-out = Out
schedule-ends-at = Ends at episode { $number }
schedule-dated-by-site = { $count } dated by { $site }
schedule-dated-by-record = { $count } dated by the record
schedule-set-by-you = { $count } set by you
schedule-guessed-weekly = { $count } guessed weekly
schedule-nothing-set = Nothing set yet
schedule-saved = Saved
schedule-cleared = Schedule notes cleared
schedule-title = Episodes and dates
schedule-season = Season { $number }
schedule-ask-agent = Ask { $agent }
schedule-done-pasting = Done pasting
schedule-paste-list = Paste a list
schedule-agent-looking = { $agent } is looking up the dates. Nothing is saved until you press Save.
schedule-agent-stopped = { $agent } stopped: { $error }
schedule-stop = Stop
schedule-paste-placeholder = Paste a release list, a table or a post
schedule-paste-helper = What AniBeam reads shows on the episodes below.
schedule-agent-hint = With Claude Code on this computer, AniBeam can ask it to look the dates up for you.
schedule-show-more = Show more episodes
schedule-pick-hint = Click an episode to date it. Shift-click picks a run.
schedule-picked-one = Episode { $number }:
schedule-picked = { $count ->
    [one] { $count } episode:
   *[other] { $count } episodes:
}
# What a typed date may look like; the parser reads these shapes
schedule-date-placeholder = 7 Jan, March 2027, 2027
schedule-set-date = Set date
schedule-weekly = Weekly
schedule-not-last = Not the last
schedule-last-episode = Last episode
schedule-clear-date = Clear date

## ShowPage.qml

show-title = Show
# AniList's formats
# AniList's airing states
# $runtime is a length such as "24 min"
show-runtime-each = { $runtime } each
show-add-to-list = Add to list
show-moving-to = Moving to { $list }
show-moved-to = Moved to { $list }
show-relation-related = Related
show-no-poster = No poster
show-reading-anilist = Reading AniList
show-not-in-library = Not in your library
show-get-episodes = Get episodes
show-trailer = Trailer
show-your-score = Your score { $score }
show-rate-it = Rate it
show-anilist-score = AniList { $score }
show-forecast = { $percent }% for you
show-because-you-watched = Because you watched
show-hidden-loved = { $count ->
    [one] A hidden tag you love is behind it
   *[other] { $count } hidden tags you love are behind it
}
show-you-gave-it = You gave it { $score }
show-story = Story
show-genres-tags = Genres and tags
show-studios = Studios  { $studios }
show-related = Related
show-recommendations = Recommendations
show-from-anilist = From AniList
show-reading-rest = Reading the rest from AniList

## ConnectSheet.qml

# $name is the tracker: AniList or MyAnimeList
connect-connected-to = Connected to { $name }
connect-connect-to = Connect { $name }
connect-did-not-finish = The sign-in did not finish
connect-finish-in-browser = Finish in your browser
# $account is the user's name on the tracker
connect-signed-in-anilist = Signed in as { $account }. Your list is on the Watching page.
connect-signed-in-mal = Signed in as { $account }. Episodes are marked there as you watch.
connect-setup-lead = { $name } signs AniBeam in through an app of your own. It takes about a minute, once.
connect-page-open = { $name }'s sign-in page is open in your browser. Approve AniBeam there and this closes itself.
connect-page-opening = Opening { $name }'s sign-in page in your browser.
connect-open-watching = Open Watching
connect-done = Done
connect-open-again = Open the sign-in page again
connect-secret-where = It is the long string under "Client Secret" on the app's page at myanimelist.net/apiconfig.
connect-client-secret = Client secret
connect-save-try-again = Save and try again
connect-change-client-id = Change the Client ID
connect-step-create-anilist = Create the client on AniList
connect-step-create-mal = Create the app on MyAnimeList
connect-step-create-anilist-body = Open AniList's developer settings, create a new client, and paste this address into its Redirect URL box exactly as it is here.
connect-step-create-mal-body = Open MyAnimeList's API page, create an ID with App Type "other", and paste this address into its App Redirect URL box exactly as it is here.
connect-open-developer-page = Open the developer page
connect-copy-address = Copy the address
connect-step-paste-id = Paste the app's ID
connect-step-paste-id-anilist = AniList calls it Client ID: the number on your new client's page.
connect-step-paste-id-mal = MyAnimeList calls it Client ID: the long string on your new app's page.
connect-client-id = Client ID
connect-client-secret-web = Client secret (only for Web apps)
connect-step-connect-body = The browser opens, { $name } asks you to sign in and approve AniBeam, and AniBeam catches the reply.

## SettingsPage

settings-title = Settings
settings-search-placeholder = Search settings
settings-section-library = Library
settings-section-accounts = Accounts
settings-section-appearance = Appearance
settings-section-playback = Playback
settings-section-subtitles = Subtitles
settings-section-keys = Keyboard
settings-section-downloads = Downloads
settings-section-phone = Phone app
settings-section-storage = Storage and backup
settings-section-help = Help
settings-section-about = About
settings-section-developer = Developer
settings-status-library-empty = No folders yet
settings-status-library =
    { $count ->
        [one] { $count } series
       *[other] { $count } series
    } in { $folders ->
        [one] { $folders } folder
       *[other] { $folders } folders
    }
settings-status-library-missing =
    { $gone ->
        [one] { $gone } folder
       *[other] { $gone } folders
    } not found, { $count ->
        [one] { $count } series
       *[other] { $count } series
    } in { $folders ->
        [one] { $folders } folder
       *[other] { $folders } folders
    }
settings-status-accounts-anilist = Syncing to AniList
settings-status-accounts-mal = Syncing to MyAnimeList
settings-status-accounts-both = Syncing to AniList and MyAnimeList
settings-status-accounts-none = No list connected
settings-status-appearance =
    { $mode ->
        [system] Follows the system
        [light] Light
       *[dark] Dark
    }, { $source ->
        [theme] theme colours
       *[desktop] desktop colours
    }
settings-status-playback =
    { $skip ->
        [both] Skips intros and outros
        [intro] Skips intros
        [outro] Skips outros
       *[none] Plays everything through
    }
settings-status-playback-audio =
    { $skip ->
        [both] Skips intros and outros
        [intro] Skips intros
        [outro] Skips outros
       *[none] Plays everything through
    }, audio { $languages }
settings-status-subtitles =
    Size { $percent } percent, { $override ->
        [AsScripted] styled files as made
        [Force] your style on every file
       *[ScaleOnly] styled files keep their look
    }
settings-status-keys = Every key AniBeam answers to
settings-status-downloads-unread = Not read yet
settings-status-downloads-answered = qBittorrent answered
settings-status-downloads-missing = Search helper not installed
settings-status-downloads-unconnected = Not connected yet
settings-status-phone-shared = Shared on your tailnet
settings-status-phone-off = Not shared
settings-status-storage = { $size } of images cached
settings-status-storage-unread = Images, export and import
settings-status-help = Answers to what people ask most
settings-status-about-update = Version { $version } is out
settings-status-developer = Timing every page
settings-status-developer-last = { $page } built in { $ms } ms
settings-results =
    { $count ->
        [one] { $count } setting matches
       *[other] { $count } settings match
    }
settings-results-none = Nothing matches
settings-results-hint = Return opens the first one.
settings-results-none-hint = Try a plainer word: theme, subtitles, tracker, password, folder.

# The search's index: each row or group by its name, as its own tab shows it, and the
# words a person might type for it
settings-find-folders =
    .words = add folder scan rescan remove source path collection
settings-find-matching =
    .words = metadata show info provider anilist tvmaze order match
settings-find-hidden-shows =
    .words = hidden reveal
settings-find-hidden-airing =
    .words = not interested airing show again
settings-find-calendar =
    .words = calendar ics ical airing schedule subscribe
settings-find-anime-lists =
    .words = anilist myanimelist mal sign in connect login account disconnect tracker
settings-find-main-list =
    .words = progress cards primary tracker
settings-find-sync-now =
    .words = refresh progress lists tracker
settings-find-on-hold =
    .words = on hold paused inactive untouched days
settings-find-mode =
    .words = dark light system theme night
settings-find-colours-from =
    .words = terminal desktop system theme source kitty
settings-find-dark-theme =
    .words = colour scheme base16
settings-find-light-theme =
    .words = colour scheme base16
settings-find-accent =
    .words = colour highlight
settings-find-text-size =
    .words = font type words bigger smaller larger zoom scale read accessibility
settings-find-density =
    .words = compact comfortable spacing size
settings-find-poster-size =
    .words = cards small medium large
settings-find-out-now =
    .words = airing out now cards list rows style banner posters
settings-find-out-now-cards =
    .words = airing out now grid columns rows layout
settings-find-reduce-motion =
    .words = animation accessibility
settings-find-app-icon =
    .words = logo launcher dock taskbar window dark
settings-find-skip-intros =
    .words = opening op auto skip aniskip chapters
settings-find-skip-outros =
    .words = ending ed credits auto skip aniskip
settings-find-mpv-settings =
    .words = mpv.conf config advanced
settings-find-discord =
    .words = discord rich presence status profile
settings-find-next-episode =
    .words = next button title autoplay end
settings-find-subtitle-languages =
    .words = language track subs
settings-find-audio-languages =
    .words = language track dub japanese
settings-find-room-name =
    .words = watch together room name friends
settings-find-room-relay =
    .words = watch together room relay server
settings-find-start-over =
    .words = reset default subtitle style
settings-find-size =
    .words = scale subtitle
settings-find-position =
    .words = height margin subtitle
settings-find-styled =
    .words = ass override force
settings-find-font =
    .words = typeface
settings-find-text-colour =
    .words = color
settings-find-outline =
    .words = border stroke
settings-find-background =
    .words = box opacity
settings-find-bold =
    .words = weight
settings-find-keys-player =
    .words = keyboard shortcuts keys hotkeys
settings-find-keys-player-around =
    .words = keyboard shortcuts fullscreen next previous episode back
settings-find-keys-around =
    .words = keyboard shortcuts back forward right-click menu fullscreen
settings-find-keys-anywhere =
    .words = keyboard shortcuts keys hotkeys settings quit
settings-find-torrent-client =
    .words = qbittorrent password username address webui seedbox
settings-find-search-helper =
    .words = anirss rss feeds token command server advanced
settings-find-next-few =
    .words = rolling roll episodes ahead delete watched keep disk space
settings-find-unsubscribe =
    .words = prune feeds finished
settings-find-last-search =
    .words = downloads query remember
settings-find-search-sources =
    .words = endpoints feeds config anirss
settings-find-source-lists =
    .words = repo repos endpoint signed endpoints jackett prowlarr
settings-find-share-tailscale =
    .words = phone android tailnet remote host
settings-find-stream =
    .words = stream play cast episodes phone files
settings-find-devices =
    .words = trust trusted switch off block device tailscale
settings-find-back-up =
    .words = export backup file json
settings-find-restore =
    .words = import restore file json
settings-find-image-cache =
    .words = posters clear images disk cache
settings-find-files =
    .words = database data config cache folder path
settings-find-help-folders =
    .words = help folder layout naming episodes seasons movies files structure
settings-find-help-recognised =
    .words = help match wrong show not found unmatched recognised
settings-find-help-anilist =
    .words = help anilist myanimelist mal sign in connect login tracker
settings-find-help-downloads =
    .words = help downloads empty sources nothing torrent qbittorrent
settings-find-help-data =
    .words = help data files backup database privacy
settings-find-help-else =
    .words = help setup onboarding walkthrough report bug problem issue
settings-find-version =
    .words = build what's new changelog
settings-find-setup =
    .words = first run walkthrough onboarding run setup again
settings-find-updates =
    .words = stable nightly channel check
settings-find-developer-options =
    .words = debug timing
settings-find-ratings =
    .words = imdb credit

## SettingsDownloadsTab

settings-downloads-title = Downloads
settings-downloads-lead = Where AniBeam sends what you download, and how it searches the release feeds.
settings-downloads-keep-none = Finishing an episode deletes the one before it.
settings-downloads-keep-line = Watched episodes kept behind the one you just finished. With { $count }, finishing episode { $finishing } deletes episode 1.
settings-downloads-anirss-ready = Running, version { $version } at { $url }.
settings-downloads-anirss-starting = Starting the search helper.
settings-downloads-anirss-missing = The search helper is not installed. Reinstall AniBeam, or name your own anirss under Command.
settings-downloads-anirss-failed = The search helper stopped. Try again starts it; Details says what it said.
settings-downloads-anirss-idle = Not started yet. It starts the first time you search on the Downloads page.
settings-downloads-torrent-trying = Trying { $client }.
settings-downloads-torrent-answered = { $client } { $version } answered.
settings-downloads-torrent-failed = { $client } did not connect. Check the address and password above, then test again.
settings-downloads-torrent-untried = Not tried yet.
settings-downloads-mode-existing = A qBittorrent you run yourself, with its Web UI turned on.
settings-downloads-mode-managed = AniBeam runs a qBittorrent of its own into ~/Downloads/Anime. It keeps going after AniBeam quits, so subscriptions keep downloading.
settings-downloads-mode-auto = Uses a qBittorrent that answers at the address below, else runs one of its own.
settings-downloads-password-saved = A password is saved here for this client.
settings-downloads-password-anirss = Using the password the search helper saved; save one here to stop depending on it.
settings-downloads-password-none = No password saved. A client that bypasses its login for this machine needs none.
settings-downloads-mismatch = The search helper talks to { $helperUrl } while AniBeam talks to { $url }. A search helper you run yourself reads the address from its own config.toml, so switching clients here does not move it.
settings-downloads-torrent-title = Torrent client
settings-downloads-torrent-helper = The qBittorrent that does the downloading. Its password is kept in the system password store.
settings-downloads-which-client = Which client
settings-downloads-pick-for-me = Pick for me
settings-downloads-my-own = My own
settings-downloads-built-in = Built-in
settings-downloads-saved-clients = Saved clients
settings-downloads-saved-clients-helper = A seedbox and the one on this machine, say. The client chip on the Downloads page switches between them too.
settings-downloads-add-client = Add
settings-downloads-name = Name
settings-downloads-name-placeholder = Named after its address
settings-downloads-address = Address
settings-downloads-address-managed-helper = Only this machine can reach it, and it needs no password.
settings-downloads-address-helper = The Web UI's address.
settings-downloads-username = Username
settings-downloads-password = Password
settings-downloads-secret-saved = Saved
settings-downloads-secret-unset = Not set
settings-downloads-connection = Connection
settings-downloads-open-config = Open its config
settings-downloads-details = Details
settings-downloads-details-torrent = { $client } said this when AniBeam tried it.
settings-downloads-test-trying = Trying
settings-downloads-test-save = Save and test
settings-downloads-test = Test
settings-downloads-searching = Searching
settings-downloads-search-helper = Search helper
settings-downloads-searching-helper = AniBeam searches your sources itself and sends what you pick to the torrent client. The search helper is not needed.
settings-downloads-search-helper-helper = The part of AniBeam that searches and subscribes, a program called anirss. AniBeam runs it for you; change this only if you run your own.
settings-downloads-runs-in = Searches run in
settings-downloads-runs-in-native-helper = Subscriptions still go to the torrent client, so new episodes arrive while AniBeam is closed.
settings-downloads-runs-in-helper = The search helper, a separate program AniBeam runs or you run yourself.
settings-downloads-engine-native = AniBeam itself
settings-downloads-how-runs = How it runs
settings-downloads-how-runs-sidecar = AniBeam starts it when it opens and stops it when it quits.
settings-downloads-how-runs-remote = AniBeam uses one you already run, at the address and with the key below.
settings-downloads-runs-sidecar = AniBeam runs it
settings-downloads-runs-remote = I run it
settings-downloads-command = Command
settings-downloads-command-helper = A name on PATH or a full path. ~/.local/bin is searched too.
settings-downloads-server-address-helper = The server's base, without /api/v1.
settings-downloads-key = Key
settings-downloads-key-saved-helper = A key is saved. Type a new one to replace it, or clear the field to remove it.
settings-downloads-key-helper = The key the search helper showed when it started. It lets AniBeam in.
settings-downloads-details-anirss = The search helper said this when it stopped.
settings-downloads-starting = Starting
settings-downloads-reconnect = Reconnect
settings-downloads-save-first = Save your changes first
settings-downloads-roll-title = Keep the next few ready
settings-downloads-roll-helper = A long show downloads a few episodes at a time, and each one you finish starts the next. The whole run never has to fit on the disk.
settings-downloads-roll-big = For long shows
settings-downloads-roll-big-helper = Anything over { $count } episodes downloads a few at a time by itself. Its row on the Downloads page can still get everything at once.
settings-downloads-roll-ahead = Episodes ahead
settings-downloads-roll-ahead-helper = How many stay downloaded past the one you are on. 12 is about a season.
# The unit after a slider's number: "12 episodes"
settings-downloads-unit-episodes = episodes
settings-downloads-roll-over = Long means over
settings-downloads-roll-over-helper = Downloads with more episodes than this get a few at a time.
settings-downloads-roll-watched = Watched episodes
settings-downloads-roll-watched-delete-helper = Deleted once you are further along, so the disk only holds what is left to watch.
settings-downloads-roll-watched-keep-helper = Every file stays on the disk after you watch it.
settings-downloads-keep-all = Keep all
settings-downloads-keep-behind = Keep behind
settings-downloads-subscriptions-title = Subscriptions and searches
settings-downloads-last-search = Keep the last search
settings-downloads-last-search-helper = The Downloads page opens with the search you last ran. Off, it opens empty.
settings-downloads-prune = Unsubscribe from finished shows
settings-downloads-prune-helper = 16 days after a show's last episode airs, its feed is dropped. The show and its files stay.
settings-downloads-sources-title = Search sources
settings-downloads-sources-helper = Where a search goes, in the order they are tried. One that finds nothing falls through to the next.
settings-downloads-sources-empty = No sources set up yet
settings-downloads-sources-connect = Connect the search helper to list them
settings-downloads-sources-empty-helper = Add a source list below and its sources show up here.
settings-downloads-config-file = Search helper's settings file
settings-downloads-config-native-helper = Holds the save folders, the tags and the best match rules, which AniBeam itself follows when it runs the helper. Its own sources are the helper's alone.
settings-downloads-config-helper = Holds the sources, the save folders, the tags and the best match rules.
settings-downloads-unsaved = Unsaved changes
settings-downloads-unsaved-helper = Saving restarts the search helper on the new settings.
settings-downloads-discard = Discard

## SettingsDeveloperTab

settings-developer-title = Developer
settings-developer-lead = Why a page is slow, measured. Each switch applies to the next page you open.
settings-developer-ms = { $ms } ms
settings-developer-mb = { $mb } MB
settings-developer-seconds = { $seconds } s
settings-developer-switches = Switches
settings-developer-load-images = Load images
settings-developer-load-images-helper = Off, no poster, banner or portrait is read or decoded.
settings-developer-overlay = Page load overlay
settings-developer-overlay-helper = The last page load and the memory, in the window's bottom right corner.
settings-developer-read-cache = Keep library reads
settings-developer-read-cache-helper = The answer to a whole library read is kept until the library changes. Off, every page asks the core again.
settings-developer-poster-pool = Warm posters in the background
settings-developer-poster-pool-helper = Decodes the 60 posters you are most likely to see and keeps them in memory.
settings-developer-options = Developer options
settings-developer-options-helper = Turns this tab, the overlay and the measuring off.
settings-developer-memory = Memory
settings-developer-memory-helper = What the kernel counts for this process. Images hold decoded pixels in memory and on the graphics card.
settings-developer-memory-unknown = This system does not say how much memory the app uses.
settings-developer-in-use = In use
settings-developer-peak = Peak
settings-developer-heap = Heap
settings-developer-files-mapped = Files mapped
settings-developer-threads = Threads
settings-developer-give-back = Give memory back
settings-developer-give-back-helper = Drops what the scene graph can rebuild, collects JavaScript garbage and hands free heap to the system. What it frees is what the app was holding without needing it.
settings-developer-released = { $before } to { $after }
settings-developer-free-now = Free now
settings-developer-images-total = Images since the start
settings-developer-images-total-line = { $decodes } decoded, { $pixels } of pixels from { $disk } on disk, { $ms } ms
settings-developer-loads = Page loads
settings-developer-loads-helper = Built is the wait after a click: the Qt thread does nothing else until the page exists. Settled is when its last image arrived.
settings-developer-touring = Timing, { $at } of { $steps }
settings-developer-tour = Time every page
settings-developer-copy-report = Copy report
settings-developer-col-page = Page
settings-developer-col-built = Built
settings-developer-col-painted = Painted
settings-developer-col-settled = Settled
settings-developer-col-calls = Calls, data
settings-developer-col-decodes = Decodes, time, pixels
settings-developer-col-items = Items
settings-developer-col-offscreen = Images out of view
settings-developer-col-memory = Memory
settings-developer-load-no-images = { $page }, no images
settings-developer-offscreen-of = { $offscreen } of { $total }
settings-developer-loads-empty = Open any page and it shows up here.
settings-developer-load-images-off = { $page }, images off
settings-developer-load-frames = { $frames } frames painted while it loaded, the longest gap between two { $gap }.
settings-developer-load-frames-timed-out = { $frames } frames painted while it loaded, the longest gap between two { $gap }. Gave up waiting for its images after 15 s.
settings-developer-built = Built
settings-developer-settled = Settled
settings-developer-core-data = Data from the core
settings-developer-pixels = Pixels decoded
settings-developer-page-items = Items on the page
settings-developer-cached-images = Images already in memory
settings-developer-calls = Core calls, slowest first
settings-developer-call-cached = { $name }  { $ms } ms  { $size }  cached
settings-developer-call = { $name }  { $ms } ms  { $size }  core { $core }, to QML { $qml }
settings-developer-decodes = Slowest decodes
settings-developer-decodes-line = { $count } decodes one after another on the image thread, { $span } from the first to the last; { $requests } asked for, { $cached } already decoded, { $blocked } blocked
settings-developer-decode = { $ms } ms  { $srcW }x{ $srcH } to { $w }x{ $h }  { $size }
settings-developer-decodes-none = Nothing decoded: every image was in memory already.
settings-developer-images-were-off = Images were off.
settings-developer-scene = What the page is made of
settings-developer-scene-line = { $items } items, { $offscreen } outside the window; { $texts } texts, { $shapes } shapes, { $icons } icons; { $images } images, { $imagesOffscreen } of them outside the window holding { $size }
settings-developer-copy-load = Copy this load
settings-developer-tour-title = Every page, images on and off
settings-developer-tour-helper = The middle of three visits to each page. Where the two columns match, images are not what makes the page slow.
settings-developer-col-built-on = Built, images on
settings-developer-col-built-off = Built, images off
settings-developer-col-settled-on = Settled, on
settings-developer-col-settled-off = Settled, off
settings-developer-col-decodes-time = Decodes, time
settings-developer-col-data = Data
settings-developer-tour-peak = Peak memory during the tour { $size }
settings-developer-events = Events
settings-developer-events-helper = Everything the core says, every level, newest first. Each event is copied to the Qt thread as JSON; the size is what it cost to send.
settings-developer-event-rate = { $kind } { $count }/min, { $size }
settings-developer-events-filter = Filter by words, kind or stage
settings-developer-event-plain = { $message }   (plain: { $plain })
settings-developer-copy-events = Copy events

## SettingsKeysTab

settings-keys-title = Keyboard
settings-keys-lead = Every key AniBeam answers to. Press ? anywhere for the keys that work where you are.
settings-keys-player = In the player
settings-keys-player-around = Getting around the player
settings-keys-around = Getting around
settings-keys-anywhere = Anywhere

## KeyHelp

key-help-title = Keyboard shortcuts
key-help-group-playback = Playback
key-help-group-navigation = Navigation
key-help-group-library = Library
# Key names on the chips. " / " splits one chip into two, so keep it between keys.
key-help-key-space = Space / K
key-help-key-left-right = Left / Right
key-help-key-ctrl-right = Ctrl+Right
key-help-key-backspace = Backspace
key-help-key-up-down = Up / Down
key-help-key-hold = Hold the video
key-help-key-page = Page Up / Down
key-help-key-home-end = Home / End
key-help-key-escape = Escape
key-help-key-alt-left = Alt+Left
key-help-key-alt-right = Alt+Right
key-help-key-shift-f10 = Shift+F10
key-help-key-ctrl-k = Ctrl+K
key-help-key-search = / or Ctrl+F
key-help-key-ctrl-space = Ctrl+Space
key-help-key-shift-space = Shift+Space
key-help-key-ctrl-a = Ctrl+A
key-help-key-ctrl-l = Ctrl+L
key-help-key-ctrl-j = Ctrl+J
key-help-key-ctrl-comma = Ctrl+,
key-help-key-ctrl-q = Ctrl+Q
key-help-play-pause = Play or pause
key-help-seek-5 = Seek 5 s
key-help-seek-10 = Seek 10 s
key-help-skip = Skip the intro or outro, else 90 s
key-help-frame-step = One frame back or forward
key-help-speed = Slower or faster, in steps of 0.25
key-help-normal-speed = Normal speed
key-help-volume = Volume up or down by 5
key-help-mute = Mute
key-help-subtitles-toggle = Subtitles off and back
key-help-subtitle-delay = Subtitle delay 100 ms earlier or later
key-help-audio-delay = Audio delay 100 ms earlier or later
key-help-hold-speed = 2× speed while held
key-help-screenshot = Save a screenshot to Pictures
key-help-episode = Next or previous episode
key-help-chapter = Previous or next chapter
key-help-start-end = The start, or the outro
key-help-fullscreen = Fullscreen
key-help-escape-player = Close what is open, else leave fullscreen, else leave the player
key-help-toggle-list = Show or hide this list
key-help-library-search = Library and search
key-help-search-page = Search this page
key-help-activity = Activity
key-help-room = Watch together
key-help-settings = Settings
key-help-quit = Quit
key-help-forward = Forward
key-help-escape-app = Close what is open, else back
key-help-context-menu = The right-click menu
key-help-pick-card = Pick a card, to act on several
key-help-pick-range = Pick every card up to this one
key-help-pick-all = Pick every card shown

## SettingsAboutTab

settings-about-title = About
settings-about-lead = Which AniBeam this is, and how it hears about the next one.
settings-about-update-failed = The update did not finish. Try again, or get it from the release page.
settings-about-update-ready = { $version } is ready to install.
settings-about-update-command = { $version } is out. Update it with { $command }
settings-about-update-progress = { $headline }  ·  { $detail }
settings-about-update-source = { $version } is out. This copy is built from source: pull and rebuild.
settings-about-update-elsewhere = { $version } is out, with no download here for this computer.
settings-about-version = Version
settings-about-build = AniBeam { $build }
settings-about-whats-new = What's new
settings-about-install = Install { $version }
settings-about-setup = Setup
settings-about-setup-helper = The walkthrough a new install opens with: your folder, your list and downloads.
settings-about-run-setup = Run setup again
settings-about-updates = Updates
settings-about-nightly-helper = A new build most days. Newer, and less tested.
settings-about-stable-helper = A new version once it has been tested and released.
settings-about-stable = Stable
settings-about-nightly = Nightly
settings-about-check-now = Check now
settings-about-developer = Developer options
settings-about-developer-helper = Adds a Developer section that times every page, shows memory and the full technical log, and can turn images off.
settings-about-credits = Credits
settings-about-ratings = Ratings
settings-about-ratings-helper = Information courtesy of IMDb (https://www.imdb.com). Used with permission.
settings-about-series = Series and episodes
settings-about-series-helper = AniList and TVmaze. Skip times from AniSkip.

## SettingsPlaybackTab

settings-playback-title = Playback
settings-playback-lead = What the player does on its own, so you only reach for it when you want to.
settings-playback-skipping = Skipping
settings-playback-skipping-helper = Uses the file's chapters, else skip times other viewers shared online. Undo in the player turns skipping off until AniBeam closes.
settings-playback-skip-intros = Skip intros
settings-playback-skip-outros = Skip outros
settings-playback-skip-outros-helper = Reaching the outro also marks the episode watched.
settings-playback-next-episode = Next episode
settings-playback-name-next = Name the next episode
settings-playback-name-next-helper = The Next button at the end of an episode shows the next one's title. Off, it shows only its number.
settings-playback-languages = Languages
settings-playback-languages-helper = Language codes, most wanted first. The first one a file has wins. A track you pick in the player is remembered for that series.
settings-playback-subtitle-languages = Subtitle languages
settings-playback-audio-languages = Audio languages
settings-playback-discord = Show what you watch on Discord
settings-playback-discord-check-failed = AniBeam could not check Discord just now. Try again in a moment.
settings-playback-discord-helper = The series, episode and poster appear on your Discord profile while you play. Nothing is sent while this is off or Discord is closed.
settings-playback-discord-unavailable = This build has no Discord application.
settings-playback-discord-connected = Connected to Discord
settings-playback-discord-not-connected = Not connected. AniBeam connects while an episode plays and Discord is open
settings-playback-advanced = Advanced
settings-playback-together = Watch together
settings-playback-room-name = Your name in watch together rooms
settings-playback-room-name-placeholder = Your name
settings-playback-room-relay = Room server
settings-playback-room-relay-helper = Rooms are made here. Change it only for a server you run yourself.
settings-playback-my-mpv = Use my mpv settings
settings-playback-my-mpv-helper = Reads your mpv.conf. Options that only work at startup are skipped, and scripts never load.
settings-playback-no-mpv-conf = No mpv settings file found. Put your options in mpv.conf, then turn this on.

## SliderRow

# The spoken name of a slider whose number counts something: "On hold after, in days"
slider-row-spoken-unit = { $name }, in { $unit }
# The readout beside a slider: "60 days", "1.00 ×"
slider-row-readout = { $value } { $unit }

## Swatches

swatches-accent-red = Accent Red
swatches-accent-orange = Accent Orange
swatches-accent-yellow = Accent Yellow
swatches-accent-green = Accent Green
swatches-accent-cyan = Accent Cyan
swatches-accent-blue = Accent Blue

## LookPreview

look-preview-dark = Dark
look-preview-light = Light

## SettingsAccountsTab

settings-accounts-title = Accounts
settings-accounts-lead = Connect AniList or MyAnimeList and every episode you finish is marked there for you. Counts only ever go up.
settings-accounts-syncing = Syncing your lists
settings-accounts-synced = Lists synced
settings-accounts-lists = Your anime lists
settings-accounts-lists-helper = Without one, AniBeam still remembers what you watched on this computer. A list on AniList or MyAnimeList carries it to your phone and your profile.
settings-accounts-progress = Progress
settings-accounts-main-list = Main list
settings-accounts-main-list-helper = The cards show progress from this one. The other still gets every update.
settings-accounts-local-lists = Lists on this computer
settings-accounts-local-lists-helper =
    { $count ->
        [one] One show has
       *[other] { $count } shows have
    } a status or a score only here. Move them to { $tracker } to see them on your phone and your profile.
settings-accounts-review-move = Review and move
settings-accounts-nothing-to-move = Nothing left to move
settings-accounts-sync-now = Sync now
settings-accounts-sync-now-helper = Reads every connected list again. AniBeam also does this when it starts.
settings-accounts-sync = Sync
settings-accounts-move-on-hold = Move to On hold
settings-accounts-move-on-hold-helper = A Watching series you have not touched for a while moves to On hold on your lists. Playing it moves it back. A show waiting for its next episode stays put.
settings-accounts-after = After
settings-accounts-after-line = Days without playing an episode
settings-accounts-days-unit = days

## SettingsPhoneTab

settings-phone-title = Phone app
settings-phone-lead = AniBeam on your phone can play this library, see what is airing and follow your downloads, wherever you are.
settings-phone-online = Online
settings-phone-offline = Offline
settings-phone-untrusted = Gets nothing from this computer
settings-phone-sharing = Sharing
settings-phone-sharing-helper = Only your own devices on Tailscale can reach it. Episodes watched on the phone keep their place and progress here.
settings-phone-share = Share over Tailscale
settings-phone-stream = Stream episodes
settings-phone-stream-on = Your devices can play and cast the episodes on this computer.
settings-phone-stream-off = Your devices see the library and what is airing, but cannot play from here.
settings-phone-host-service = Host service
settings-phone-not-running = Not running
settings-phone-running = Running
settings-phone-service-off-helper = Run the command once and your devices reach AniBeam from login on, with no window open.
settings-phone-service-on-helper = Your devices reach AniBeam with no window open. While this window is open, it shares in the service's place.
settings-phone-copy-command = Copy command
settings-phone-command-copied = Command copied
# Tailscale ties the device to no person, so this computer cannot tell whose it is.
settings-phone-nobody = Gets nothing: Tailscale names no person for it
# The people whose devices this computer serves, its own person first.
settings-phone-people = People
# The chip on the person this computer belongs to.
settings-phone-own-profile = This computer's profile
# The owner's line: this computer plus how many of their other devices it serves.
settings-phone-person-own-devices =
    { $count ->
        [0] This computer
        [one] This computer and { $count } device
       *[other] This computer and { $count } devices
    }
# Anyone else's line: how many of their devices this computer serves.
settings-phone-person-devices =
    { $count ->
        [one] { $count } device
       *[other] { $count } devices
    }
settings-phone-devices = Devices on your tailnet
settings-phone-devices-helper = Every device on your tailnet can reach this computer while sharing is on. Switch one off and it gets nothing from here.
settings-phone-addresses = Addresses
settings-phone-addresses-helper = Type one of these into the phone app the first time it asks.
settings-phone-copy = Copy
settings-phone-address-copied = Address copied

## SettingsHelpTab

settings-help-title = Help
settings-help-lead = Short answers to what people ask most, each with a way straight to the fix.
settings-help-folders = How should my folders look?
settings-help-folders-helper = One folder per show, with its episodes inside. Name each file with its episode number, the way releases do: "Frieren - 01.mkv" or "Frieren S01E01.mkv". A second season can be a folder of its own or a "Season 2" folder inside the show. Films go in a folder named Movies, one file each.
settings-help-for-example = For example
settings-help-folders-example =
    Anime / Frieren / Frieren - 01.mkv
    Anime / Frieren / Season 2 / Frieren S02E01.mkv
    Anime / Movies / Suzume.mkv
settings-help-no-folder = No folder added yet
settings-help-reads-folders =
    { $count ->
        [one] AniBeam reads one folder
       *[other] AniBeam reads { $count } folders
    }
settings-help-add-folder-helper = Add the folder that holds your shows, not each show on its own.
settings-help-add-folder = Add a folder
settings-help-open-folders = Open your folders
settings-help-match = Why didn't my show get recognised?
settings-help-match-helper = AniBeam looks each folder up by its name, on AniList first, then TVmaze. Extra words, a typo or a name in another language can miss. Pick the right show by hand on Show info; a choice you make is never replaced.
settings-help-all-recognised = Every show is recognised
settings-help-unrecognised =
    { $count ->
        [one] One show is not recognised
       *[other] { $count } shows are not recognised
    }
settings-help-open-show-info = Open Show info
settings-help-connect = How do I connect AniList?
settings-help-connect-helper = Connect it and every episode you finish is marked on your list, so your phone and your profile keep up. AniList asks you to make a small app of your own once; AniBeam walks you through it, and it takes about a minute. MyAnimeList works the same way.
settings-help-both-connected = Both are connected
settings-help-anilist-connected = AniList is connected
settings-help-mal-connected = MyAnimeList is connected
settings-help-none-connected = Neither is connected yet
settings-help-connect-anilist = Connect AniList
settings-help-connect-mal = Connect MyAnimeList
settings-help-downloads = Why is Downloads empty?
settings-help-downloads-helper = AniBeam comes with no places to search. Add a source list in Settings, under Downloads, and install the sources you want from it. Downloading also needs a torrent client: AniBeam can run its own, or use a qBittorrent you already have.
settings-help-sources = Sources and the torrent client
settings-help-open-downloads = Open Downloads settings
settings-help-data = Where is my data?
settings-help-data-helper = Your library, what you watched and your settings stay on this computer. Sign-ins for AniList and MyAnimeList go in the system password store, or in a file only you can read where there is none. Back it all up to one file from Storage and backup.
settings-help-library-history = Library and history
settings-help-show-folder = Show the folder
settings-help-backup = Back up or restore
settings-help-open-storage = Open Storage and backup
settings-help-more = Something else?
settings-help-more-helper = Run the first setup again, or tell us what went wrong. A screenshot helps a lot.
settings-help-first-setup = The first setup
settings-help-first-setup-helper = Your folder, your list and downloads, one step at a time.
settings-help-run-setup = Run setup again
settings-help-report = Report a problem
settings-help-report-helper = Opens a bug report on GitHub in your browser, with this version filled in.

## SettingsSubtitlesTab

settings-subtitles-title = Subtitles
settings-subtitles-lead = What every episode starts with. Changes show on the frame below as you make them.
settings-subtitles-reset-done = Subtitles look as they did at first
settings-subtitles-size-place = Size and place
settings-subtitles-size = Size
settings-subtitles-position = Position
settings-subtitles-position-helper = 100 sits on the bottom edge. Lower numbers lift the lines up the frame.
settings-subtitles-style = Style
settings-subtitles-styled = Styled subtitles
settings-subtitles-as-made-helper = Styled (ASS) files look exactly as their makers set them.
settings-subtitles-mine-helper = Your style below goes on every file. Signs and karaoke can break.
settings-subtitles-size-only-helper = Styled subtitle files keep their look and take your size. Plain files take your style.
settings-subtitles-as-made = As made
settings-subtitles-size-only = Size only
settings-subtitles-mine = Mine
settings-subtitles-font = Font
settings-subtitles-font-invalid = Type a font name, such as Noto Sans.
settings-subtitles-bold = Bold
settings-subtitles-text-colour = Text colour
settings-subtitles-colour-invalid = Type a colour like #ffcc00.
settings-subtitles-outline = Outline
settings-subtitles-outline-invalid = The width is a number, 0 or more.
settings-subtitles-outline-colour-invalid = Type the colour like #000000.
settings-subtitles-outline-helper = Its width, then its colour.
settings-subtitles-shadow = Shadow
settings-subtitles-shadow-invalid = A number, 0 or more. 0 means no shadow.
settings-subtitles-shadow-helper = How far the shadow falls, 0 for none.
settings-subtitles-box = Background box
settings-subtitles-box-helper = A dark box behind the lines. 0 leaves it out.
settings-subtitles-start-over = Start over
settings-subtitles-start-over-helper = Puts the size, font, colours, outline, box and position back to how they came.
settings-subtitles-reset = Reset

## SettingsDataTab

settings-data-title = Storage and backup
settings-data-lead = Move your library to another computer, keep a copy safe, or free the space images take.
settings-data-backed-up = Backed up to { $path }
settings-data-restore-finished = Restore finished
settings-data-images-removed =
    { $count ->
        [one] Removed { $count } image
       *[other] Removed { $count } images
    }
# What the last restore did: the parts below, joined with commas, make one sentence
settings-data-summary = { $parts }.
settings-data-summary-folders =
    { $count ->
        [one] { $count } folder added
       *[other] { $count } folders added
    }
settings-data-summary-series = { $count } series created
settings-data-summary-matches =
    { $count ->
        [one] { $count } match applied
       *[other] { $count } matches applied
    }
settings-data-summary-views =
    { $views ->
        [one] { $views } view
       *[other] { $views } views
    } and { $episodes ->
        [one] { $episodes } finished episode
       *[other] { $episodes } finished episodes
    } merged
settings-data-summary-resume-points =
    { $count ->
        [one] { $count } resume point merged
       *[other] { $count } resume points merged
    }
settings-data-summary-accounts =
    { $count ->
        [one] { $count } list account brought in
       *[other] { $count } list accounts brought in
    }
settings-data-summary-fixes =
    { $count ->
        [one] { $count } file takes a fix you made.
       *[other] { $count } files take a fix you made.
    }
settings-data-summary-unavailable =
    { $count ->
        [one] { $count } folder was not found on this computer.
       *[other] { $count } folders were not found on this computer.
    }
settings-data-summary-left-out = Left out: { $fields }.
settings-data-backup = Backup
settings-data-back-up-library = Back up the library
settings-data-back-up-helper = One file with your folders, matches and what you have watched.
settings-data-last-written = Last written to { $path }
settings-data-open-folder = Open folder
settings-data-back-up = Back up
settings-data-private = Include private data
settings-data-private-helper = Adds AniList and MyAnimeList sign-ins, watch history and preferences. Sign-ins are written as plain text, so keep the file to yourself.
settings-data-restore-from = Restore from a backup
settings-data-restore-helper = Reads a backup into this library. What is on disk wins; the file fills in the rest.
settings-data-restore = Restore
settings-data-image-cache = Image cache
settings-data-image-cache-helper = Posters, banners and portraits are kept on disk so the library works offline. Cleared images download again as pages need them.
settings-data-cache-unmeasured = The image cache could not be measured
settings-data-cache-size =
    { $count ->
        [one] { $size } in { $count } image
       *[other] { $size } in { $count } images
    }
settings-data-cache-empty = No images cached
settings-data-cache-unmeasured-helper = Try again in a moment. The images themselves are fine.
settings-data-remove-images =
    { $count ->
        [one] Remove { $count } image?
       *[other] Remove { $count } images?
    }
settings-data-files = Where AniBeam keeps its files
settings-data-files-database = Library database
settings-data-files-data = Data
settings-data-files-settings = Settings
settings-data-files-cache = Cache
settings-data-backup-filter = AniBeam backup (*.json)
settings-data-all-files-filter = All files (*)
settings-data-backing-up = Backing up
settings-data-restoring = Restoring

## SettingsLibraryTab

settings-library-title = Library
settings-library-lead = The folders AniBeam watches, and how it works out what is in them.
settings-library-not-scanned = Not scanned yet
settings-library-scan-nothing-changed = Nothing changed
settings-library-scan-added = { $count } added
settings-library-scan-changed = { $count } changed
settings-library-scan-removed = { $count } removed
# The last scan of one folder: "Anime: 3 added, 1 removed"
settings-library-scan-in-folder = { $folder }: { $what }
settings-library-anilist-about = Anime: series, films and how they relate. What you watch is marked on your trackers.
settings-library-tvmaze-about = Other television: live action, cartoons, documentaries. Nothing is marked on a tracker.
settings-library-tried-first = Tried first
settings-library-tried-second = Tried second
settings-library-tried-third = Tried third
settings-library-tried-fourth = Tried fourth
settings-library-tried-nth = Tried { $number }th
settings-library-folders = Folders
settings-library-folders-helper = Each folder inside one of these is a series. A file at the top of a Movies folder is a film.
settings-library-rescan-started = Rescan started
settings-library-load-failed = Your folders could not be read
settings-library-load-failed-helper = AniBeam could not read its list of folders. Try again in a moment.
settings-library-no-folders = No folders yet
settings-library-no-folders-helper = Point AniBeam at the folder your anime lives in. It finds the series and films by itself.
settings-library-scanned = Scanned { $when }
settings-library-add-a-folder = Add a folder
settings-library-scan-all = Scan all
settings-library-scan-started = Scan started
settings-library-add-folder = Add folder
settings-library-matching = Matching
settings-library-matching-helper = A new folder is looked up in this order and keeps the first good match. A match you set by hand is never replaced.
settings-library-try-later = Try later
settings-library-try-first = Try first
settings-library-check-matches = Check the matches
settings-library-check-matches-helper = The Show info page lists every show with the info AniBeam found for it, and fixes a wrong one.
settings-library-open-show-info = Open Show info
settings-library-hidden-shows = Hidden shows
settings-library-show-hidden = Show hidden shows
settings-library-show-hidden-helper = Brings the series you hid back onto every page until AniBeam closes.
settings-library-hidden-from-airing = Hidden from Airing now
settings-library-hidden-from-airing-helper = Shows you marked not interested. Each stays off Airing now until you bring it back.
settings-library-hidden-from-airing-empty = Nothing yet. Shows you mark not interested on Airing now collect here, so any of them can come back.
settings-library-show-again = Show again
settings-library-calendar = Calendar
settings-library-calendar-feed = Airing times in a calendar
settings-library-calendar-feed-helper = Keeps a calendar file of the coming episodes of the shows on your lists, updated when the schedule changes.
settings-library-calendar-file = Calendar file
settings-library-calendar-file-helper = Subscribe to this file in your calendar app.
settings-library-copy = Copy
settings-library-path-copied = Path copied
settings-library-open-folder = Open folder
settings-library-notifications = Notifications
settings-library-new-episodes = New episodes
settings-library-new-episodes-helper = A desktop notification when an episode of a show on your Watching list airs. Click it to open the show.
settings-library-folder-added = Folder added, scanning

## SettingsAppearanceTab

settings-appearance-text-smaller = Smaller
settings-appearance-text-normal = Normal
settings-appearance-text-large = Large
settings-appearance-text-larger = Larger
settings-appearance-text-largest = Largest
settings-appearance-title = Appearance
settings-appearance-lead = Changes show at once, here and on every page. They live in ~/.config/anibeam/theme.toml, which you can edit by hand too.
settings-appearance-colours = Colours
settings-appearance-mode = Mode
settings-appearance-mode-dark = Dark
settings-appearance-mode-light = Light
settings-appearance-mode-system = System
settings-appearance-colours-from = Colours from
settings-appearance-from-theme-helper = The theme pair below, one for dark and one for light.
settings-appearance-from-desktop-helper = Your terminal's colours, else the desktop's colour scheme and accent.
settings-appearance-from-desktop = Desktop
settings-appearance-from-theme = Theme
settings-appearance-dark-theme = Dark theme
settings-appearance-light-theme = Light theme
settings-appearance-themes-helper = Theme files you drop in ~/.config/anibeam/themes show up in both lists.
settings-appearance-accent = Accent
settings-appearance-accent-helper = Marks what is selected, playing or new.
settings-appearance-icon = Icon
settings-appearance-app-icon = App icon
settings-appearance-app-icon-helper = The window's, the taskbar's and the launcher's. Dark swaps the yellow and the black.
settings-appearance-icon-default = Default
settings-appearance-icon-default-name = Default app icon
settings-appearance-icon-dark = Dark
settings-appearance-icon-dark-name = Dark app icon
settings-appearance-size-motion = Size and motion
settings-appearance-text-size = Text size
settings-appearance-text-size-helper = How big the words are everywhere in AniBeam, on top of your system's own size.
settings-appearance-density = Density
settings-appearance-density-helper = The room around everything, from tight to airy.
settings-appearance-density-compact = Compact
settings-appearance-density-normal = Normal
settings-appearance-density-comfortable = Comfortable
settings-appearance-poster-size = Poster size
settings-appearance-poster-size-helper = How big a series is in the library and on Home.
settings-appearance-poster-small = Small
settings-appearance-poster-medium = Medium
settings-appearance-poster-large = Large
settings-appearance-out-now = Out now on Airing
settings-appearance-out-now-helper = Cards with each show's banner, or a list of small posters and titles, three rows deep.
settings-appearance-out-now-cards = Cards
settings-appearance-out-now-list = List
settings-appearance-out-now-grid = Out now cards
settings-appearance-out-now-grid-helper = How many cards show, columns by rows, before All opens the rest.
# A grid of cards, columns by rows: "4 × 2"
settings-appearance-grid = { $columns } × { $rows }
settings-appearance-reduce-motion = Reduce motion
settings-appearance-reduce-motion-helper = Skips the library's opening sweep and other movement that is only for show.

## LookPane

look-pane-sample-series = Sample series
look-pane-episode-code = Ep { $number }
look-pane-library = Library
look-pane-series-count =
    { $count ->
        [one] { $count } series
       *[other] { $count } series
    }
look-pane-search = Search titles or folders
look-pane-search-hint = / or Ctrl+K
look-pane-all = All
look-pane-series = Series
look-pane-movies = Movies
look-pane-plain = Plain
look-pane-selected = Selected
look-pane-errors =
    { $count ->
        [one] { $count } error
       *[other] { $count } errors
    }
look-pane-type-large = Large
look-pane-type-normal = Normal
look-pane-type-small = Small, dim
look-pane-sample-switch = Sample switch
look-pane-button = Button
look-pane-scan = Scan
look-pane-sample-status =
    { $count ->
        [one] { $count } new episode in { $title }
       *[other] { $count } new episodes in { $title }
    }

## FeedPage

feed-title = Feed
feed-aired = { $episode } aired { $when }
feed-downloaded = File downloaded { $when }
feed-countdown-days = { $days }d { $hours }h
feed-countdown-hours = { $hours }h { $minutes }m
feed-countdown-minutes = { $minutes }m
feed-section-this-week = This week
feed-new-episodes =
    { $count ->
        [one] { $count } new episode
       *[other] { $count } new episodes
    }
feed-section-earlier = Earlier
feed-latest-out = Latest out { $when }
feed-section-coming-soon = Coming soon
feed-this-week-count = { $count } this week
feed-section-everything-else = Everything else
feed-out-this-week = { $count } out this week
feed-nothing-scheduled = Nothing scheduled yet.
feed-next = Next: { $title } in { $countdown }
feed-nothing-new = Nothing new this week.
feed-new-this-week =
    { $count ->
        [one] { $count } new episode this week.
       *[other] { $count } new episodes this week.
    }
feed-sort-recent = Recently released
feed-sort-upcoming = Coming soon
feed-failed-title = The feed did not load
feed-empty-title = Nothing here yet
feed-failed-body = AniBeam could not read the feed just now. Try again in a moment.
feed-empty-body-folders = The feed fills as episodes air and arrive. Airing now shows what is on this season.
feed-empty-body = Add the folder your anime lives in and the feed fills with its episodes.
feed-add-folder = Add a folder
feed-airing-now = Airing now

## RecommendedPage

recommended-title = Recommended for you
recommended-ranked-minutes = Ranked { $minutes }m ago
recommended-ranked-hours = Ranked { $hours }h ago
recommended-ranked-days = Ranked { $days }d ago
recommended-scope-name = Recommended from
recommended-scope-all = All
recommended-scope-library = In library
recommended-scope-list = On list
recommended-ranking = Ranking
recommended-rank-again = Rank again
recommended-no-scores-title = Rate a few shows first
recommended-no-list-title = Nothing to learn from yet
recommended-never-fitted-title = Ranking your shows
recommended-nothing-left-title = Nothing left here
recommended-no-scores-body = Recommendations learn from the scores on your list. Score a few shows on AniList and they appear here.
recommended-no-list-body = Connect AniList in Settings, or keep a list on this device, and the shows you rate teach this page.
recommended-never-fitted-body = The first ranking reads AniList for a minute or two. It runs in the background.
recommended-nothing-left-library-body = Everything in your library that fits is watched. All finds shows you do not have yet.
recommended-nothing-left-body = Everything on your lists that fits is watched. All finds shows you have not listed.

## MoveListsSheet

move-lists-moving =
    { $count ->
        [one] Moving 1 show to { $site }
       *[other] Moving { $count } shows to { $site }
    }
move-lists-score = score { $score }
move-lists-title = Move your list to { $site }
move-lists-body =
    { $count ->
        [one] One show has
       *[other] { $count } shows have
    } a status or a score only on this computer. Moving them puts them on your { $site } list, where your phone and your profile see them. For these shows, what { $site } has now is replaced. Nothing is sent until you press Move.
move-lists-head-show = Show
move-lists-head-here = On this computer
move-lists-head-tracker = On { $site } now
move-lists-nothing = Nothing
move-lists-not-on-list = Not on your list
move-lists-later-hint = Later: Settings, Accounts
move-lists-not-now = Not now
move-lists-move =
    { $count ->
        [one] Move 1 show
       *[other] Move { $count } shows
    }

## StatsPage

stats-title = Stats
stats-month-jan = Jan
stats-month-feb = Feb
stats-month-mar = Mar
stats-month-apr = Apr
stats-month-may = May
stats-month-jun = Jun
stats-month-jul = Jul
stats-month-aug = Aug
stats-month-sep = Sep
stats-month-oct = Oct
stats-month-nov = Nov
stats-month-dec = Dec
stats-nothing-yet = Nothing yet
stats-ranked-episodes =
    { $count ->
        [one] { $count } ep
       *[other] { $count } eps
    }
stats-year-summary = What you watched in { $year }
stats-empty-title = Nothing watched yet
stats-empty-body = Finish an episode and it counts here: hours, genres and the seasons you watched.
stats-open-library = Library
stats-hours-watched = hours watched
stats-episodes-label =
    { $count ->
        [one] episode
       *[other] episodes
    }
stats-series-finished = series finished
stats-none = None
stats-busiest-month = busiest month
stats-by-month = By month
stats-month-hours = { $hours }h
stats-top-genres = Top genres
stats-top-studios = Top studios
stats-seasons-watched = Seasons watched
stats-recap-title = My year in anime
stats-recap-hours = hours
stats-recap-episodes = episodes
stats-recap-top-genre = top genre
stats-save-image = Save as image
stats-save-dialog-title = Save the recap
stats-save-filter-png = PNG image (*.png)
stats-recap-saved = Recap saved
stats-recap-save-failed = Could not save the recap

## TrackerRow

tracker-row-connected = Connected as { $user }, synced { $when }
tracker-row-unknown-user = ?
tracker-row-never = never
tracker-row-waiting = Waiting for you to approve in the browser
tracker-row-not-connected = Not connected
tracker-row-open-again = Open again
tracker-row-open-again-tip = Open the sign-in page again
tracker-row-disconnect = Disconnect
tracker-row-disconnect-question = Disconnect { $name }? Your sign-in will be removed.
tracker-row-secret-saved = A client secret is saved for this app.
tracker-row-remove-secret = Remove it
tracker-row-secret-removed = Client secret removed
tracker-row-add-secret = Add client secret
tracker-row-secret-where = It is the long string under "Client Secret" on the app's page at myanimelist.net/apiconfig.
tracker-row-secret-placeholder = Client secret
tracker-row-save-connect = Save and connect

## RemoveSheet

remove-unsubscribe-delete = Unsubscribe and delete
remove-unsubscribe-torrents = Unsubscribe and remove torrents
remove-unsubscribe = Unsubscribe
remove-delete-files = Remove and delete files
remove-torrents =
    { $count ->
        [one] Remove the torrent
       *[other] Remove torrents
    }
remove-also-torrents =
    { $count ->
        [one] Also remove its torrent
       *[other] Also remove its { $count } torrents
    }
remove-torrents-go-with-files = Deleting the files takes the torrents with them.
remove-torrents-files-stay = They leave the client; the files stay.
remove-also-files = Also delete the files
remove-files-default-where = what the torrents downloaded
remove-files-detail-rule = Moves { $where } to the trash, and removes the torrents.
remove-files-detail = Moves { $where } to the trash.

## NotesSheet

notes-your-tracker = your tracker
notes-title = Notes
notes-placeholder = Where you left off, who recommended it, which release to get
notes-over-limit = { $count } characters over the limit
notes-saved-to = Saved to { $sites }

## RecommendationCard

recommendation-card-available = Available
recommendation-card-open-library = Open { $title } in your library
recommendation-card-open-anilist = Open { $title } on AniList

## SearchField

search-field-placeholder = Search titles or folders
search-field-hint = / or Ctrl+K
search-field-clear = Clear the search

## RecommendedHover

recommended-hover-anilist-score = AniList { $score }
recommended-hover-forecast = { $percent }%
recommended-hover-hidden-tags =
    { $count ->
        [one] A hidden tag you love
       *[other] { $count } hidden tags you love
    }
recommended-hover-because = Because you watched

## PersonCard

person-card-unknown = Unknown
person-card-role-main = Main
person-card-role-supporting = Supporting
person-card-role-background = Background
person-card-the-character = the character
person-card-open = Open { $name } on AniList

## StatusDot

status-dot-on-list = On your list: { $status }

## TrackerNudge

tracker-nudge-title = Connect your anime list
tracker-nudge-body = AniBeam marks episodes on AniList or MyAnimeList as you watch.
tracker-nudge-connect-anilist = Connect AniList
tracker-nudge-connect-mal = Connect MyAnimeList
tracker-nudge-not-now = Not now

## RecommendedCard

recommended-card-spoken = { $title }, { $percent } percent for you
recommended-card-forecast = { $percent }%
recommended-card-in-library = In library

## InlineConfirm

inline-confirm-keep = Keep

## Tokens

tokens-cube-watched-tracked = Watched, tracked online
    .inline = watched, tracked online
tokens-cube-watched-here = Watched here, not tracked yet
    .inline = watched here, not tracked yet
tokens-cube-on-disk = On disk
    .inline = on disk
tokens-cube-downloading = Downloading
    .inline = downloading
tokens-cube-missing = Missing
    .inline = missing
tokens-cube-error = Error
    .inline = error
tokens-cube-not-downloaded = Not downloaded
    .inline = not downloaded
tokens-cube-not-aired = Not aired yet
    .inline = not aired yet
tokens-cube-available = Available
    .inline = available
tokens-cube-picked = Picked to get
    .inline = picked to get
tokens-cube-already-on-disk = Already on disk
    .inline = already on disk
tokens-cube-no-release = No release
    .inline = no release

## Card

card-not-interested = Not interested

## PagePlaceholder

page-placeholder-coming-soon = Coming soon

## ErrorLine

error-line-hide-details = Hide details
error-line-details = Details
error-line-details-tip = Show what went wrong, word for word, and copy it
error-line-copied = Copied.

## Readouts (apps/linux/src/format.rs): times, countdowns and lengths

fmt-just-now = just now
fmt-now = now
fmt-ago = { $span } ago
fmt-in = in { $span }
fmt-seconds-short = { $n }s
fmt-minutes-short = { $n }m
fmt-hours-short = { $n }h
fmt-days-short = { $n }d
fmt-months-short = { $n }mo
fmt-years-short = { $n }y
fmt-years-months-short = { $years }y { $months }mo
# $h, $m and $s arrive as two digits ("04") after the first unit.
fmt-countdown-days = { $d }d { $h }h { $m }m
fmt-countdown-hours = { $h }h { $m }m
fmt-countdown-days-seconds = { $d }d { $h }h { $m }m { $s }s
fmt-countdown-hours-seconds = { $h }h { $m }m { $s }s
fmt-countdown-minutes-seconds = { $m }m { $s }s
fmt-runtime-minutes = { $m } min
fmt-runtime-hours = { $h } h
fmt-runtime-hours-minutes = { $h } h { $m } min
fmt-episode = Episode
fmt-episode-span = Episodes { $first } to { $through }

## apps/linux/src/problem.rs: every call error in a person's words, what happened and what to do

problem-report = Try again. If it keeps happening, report it from the activity log.
problem-try-later = Try again later.
problem-helper-unreachable = The search helper did not answer.
problem-helper-unauthorized = The search helper turned down the login.
problem-helper-not-running = The search helper is not running.
problem-helper-missing = The search helper is not installed.
problem-helper-failed = The search helper could not do that.
problem-helper-no-login = The search helper cannot log in to the torrent client.
problem-helper-no-login-next = Save the torrent client's password in Settings, under Downloads.
problem-torrent-unreachable = The torrent client did not answer.
problem-torrent-unauthorized = The torrent client turned down the login.
problem-torrent-not-running = The torrent client is not running.
problem-torrent-missing = The torrent client is not installed.
problem-torrent-failed = The torrent client could not do that.
problem-downloads-unreachable-next = Check it is running and its address is right in Settings, under Downloads.
problem-downloads-unauthorized-next = Check the username and password in Settings, under Downloads.
problem-downloads-not-running-next = Start it from Settings, under Downloads.
problem-downloads-missing-next = Settings, under Downloads, says how to set it up.
problem-trash = { -app-name } could not move the files to the trash.
problem-trash-next = Check the files are yours and the disk is not read only.
problem-source-list = That source list did not work.
problem-source-list-next = Check its address in Settings, under Downloads, Source lists.
# $name is a search source or a site, by its own name.
problem-remote-unreachable = { $name } did not answer.
problem-remote-unreachable-next = The site may be down. Try again later, or search another source.
problem-remote-unauthorized = { $name } turned down the key it was given.
problem-remote-unauthorized-next = Change its key in Settings, under Downloads, Source lists.
problem-remote-failed = { $name } could not do that.
problem-remote-failed-next = Try again later, or search another source.
# $name is AniList, MyAnimeList, TVmaze and the like.
problem-provider-sign-in = { $name } turned down { -app-name }'s sign-in.
problem-provider-sign-in-next = Connect it again in Settings, under Accounts.
problem-provider-turned-away = { $name } turned { -app-name } away.
problem-provider-busy = { $name } is busy right now.
problem-provider-busy-next = Wait a minute, then try again.
problem-provider-no-answer = { $name } did not answer.
problem-provider-no-answer-next = Check your internet connection, then try again.
problem-provider-trouble = { $name } is having trouble right now.
problem-provider-trouble-next = Try again in a few minutes.
problem-provider-rejected = { $name } did not accept that.
problem-not-found = { -app-name } could not find that any more.
problem-not-found-next = It may have been removed. Try again to load the page afresh.
problem-invalid-next = Check it and try again.
problem-unavailable = That folder cannot be reached right now.
problem-unavailable-next = Plug in its drive or check the folder in Settings, under Library.
problem-not-connected = { $tracker } is not connected.
problem-not-connected-next = Connect it in Settings, under Accounts.
problem-io = { -app-name } could not read or write a file.
problem-io-next = Check the disk has room and the folder is yours, then try again.
problem-storage = { -app-name } could not save your library.
problem-storage-next = Check the disk has room, then try again.
problem-keyring = The system password store did not answer.
problem-keyring-next = Unlock your desktop's keyring, then try again.
problem-unsupported = { -app-name } cannot do that here yet.
problem-version = This backup comes from a newer { -app-name }.
problem-version-next = Update { -app-name } in Settings, under About, then import it again.
problem-internal = Something went wrong inside { -app-name }.
problem-refused-hidden = That show is hidden.
problem-refused-hidden-next = Unhide it from its page first.
problem-refused-no-match = That show is not matched yet.
problem-refused-no-match-next = Match it on Show info first.
problem-refused-not-newer = Your list is already past this episode.
problem-refused-extra = That file is an extra, not an episode.
problem-refused-unmatched = That file is not tied to an episode.
problem-refused-unmatched-next = Fix it from the show's page, under Fixes.
problem-refused-on-disk = That file is only on this computer.

## apps/linux/src/library_view.rs: the library page's section titles and the one fact under each
## (Dropped, On hold and Watching reuse status-* from core.ftl)

library-view-airing = Airing
library-view-no-date = No date
library-view-type-tv = TV
library-view-type-movies = Movies
library-view-type-extras = OVA, ONA and specials
library-view-type-music = Music
library-view-type-unknown = Unknown
# The Genre section for shows with no genre yet, most often because they are not matched.
library-view-no-genre = Unmatched
library-view-state-caught-up = Caught up
library-view-state-not-started = Not started
# $span is a short countdown: library-view-countdown-*.
library-view-next-episode = Next episode in { $span }
library-view-countdown-days = { $d }d { $h }h
library-view-countdown-hours = { $h }h { $m }m
library-view-countdown-minutes = { $m }m
library-view-in-progress = { $count } in progress
library-view-not-started = { $count } not started
library-view-all-watched = All watched
library-view-on-disk =
    { $count ->
        [one] { $count } episode
       *[other] { $count } episodes
    } on disk
library-view-dropped-on-disk =
    { $count ->
        [one] { $count } file
       *[other] { $count } files
    } still on disk

## apps/linux/src/dbus/notify.rs: the desktop notification for a new episode
## (its Open button reuses action-open from core.ftl)

# $episode is the episode's number, $title the show's.
notify-episode-out = Episode { $episode } of { $title } is out
notify-click-open = Click to open it in { -app-name }
notify-click-get = Click to get the episodes in { -app-name }
notify-get-episodes = Get episodes

## apps/linux/src/nowplaying.rs: the second line a media widget shows while an episode plays

# $episode is episode-number from core.ftl ("Episode 12").
now-playing-show-episode = { $show }, { $episode }

## apps/linux/src/dbus/inhibit.rs: why the screen stays on, as the desktop's power panel shows it

inhibit-reason = Playing an episode

## apps/linux/src/bridge/door.rs: why a value a page sent cannot be used (an error's detail line)

door-not-a-choice = { $value } is not a valid choice
door-tokens-op-empty = a tokens op needs tokens
door-not-a-refine-op = { $value } is not a refine op
door-limit-too-low = must be at least 1, got { $limit }
door-not-a-torrent-action = { $value } is not a torrent action

## apps/linux/src/bridge/shell.rs: the first-run starter folder and the update notice

# $folder is a path, $error the system's own reason.
shell-starter-folder-failed = Could not make { $folder }: { $error }
shell-update-gone = { -app-name } { $version } is no longer on offer
