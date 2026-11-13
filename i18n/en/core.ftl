# What the core says to a person: problems, errors, notices, and the words every shell
# shares. English is the source; every other language's core.ftl translates it.
# All of a language's files load into one bundle, so the terms here are seen from
# desktop.ftl and android.ftl as well.

## Names that stay the same in every language. Messages use them; none is shown alone.

-app-name = AniBeam
-anilist = AniList
-mal = MyAnimeList

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
