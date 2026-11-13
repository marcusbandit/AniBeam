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
