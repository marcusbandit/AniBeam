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
