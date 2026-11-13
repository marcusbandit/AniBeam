# What the phone app (apps/android) shows. English is the source.
# A word every shell says belongs in core.ftl instead.

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
