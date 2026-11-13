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
