# What the phone app (apps/android) shows. English is the source.
# A word every shell says belongs in core.ftl instead.

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
