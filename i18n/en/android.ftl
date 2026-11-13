# What the phone app (apps/android) shows. English is the source.
# A word every shell says belongs in core.ftl instead.

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

## Your computer

# Whose watch history the computer shows this phone, by the person's Tailscale name.
host-progress = { $name }'s progress

## The player

# The speed picker while the episode plays on a TV, which keeps its own speed.
player-speed-not-while-casting = Not while casting

## Downloads

# A release in the find list for an episode the phone has from another release.
downloads-other-release-on-disk = Other release on disk
