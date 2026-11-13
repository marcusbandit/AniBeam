# Jikan is an accepted risk for episode data

Status: accepted, 2026-11-13 (#840)

## Context

AniList, the provider every match goes through, has no full episode list. Its
airing schedule covers the episodes it tracked as they aired, and its
streaming entries give titles for some shows. For the rest, the core asks
[Jikan](https://jikan.moe), a community-run API that reads MyAnimeList, for
each episode's number, title and air date:

- `core/src/metadata/fetch.rs` at match time and on a full refresh;
- `core/src/metadata/airing.rs` when the airing schedule refreshes.

Jikan is not run or backed by MyAnimeList or AniList. It can go stale, change
its response shape, rate limit harder or shut down without notice.

## Decision

Keep Jikan as the source of episode lists. Losing it costs episode detail,
never a match or playback, and the alternatives cost more than that:
MyAnimeList's official API gives an episode count but no per-episode titles
or dates, and AniList's own data is what Jikan already fills in.

The core keeps Jikan a soft dependency:

- A Jikan error reads as an empty episode list. The match or refresh it is
  part of still succeeds.
- `merge_episodes` (`core/src/metadata/record.rs`) builds rows from AniList's
  schedule first, then Jikan, then AniList's streaming titles, so AniList
  alone still yields the scheduled episodes.
- A refresh never replaces a stored title (`keep_titles`), so an outage after
  a good fetch loses nothing already in the table.
- `Core::report_jikan_outage` warns in the activity log at most once per
  `OUTAGE_WINDOW` (10 minutes).
- Every call goes through the shared limiter in `core/src/net/limiter.rs`.

## Consequences

While Jikan is down, newly matched series lack the episode titles, air dates
and rows that AniList has no schedule for. Older finished shows feel it most.

New code that reads Jikan keeps it soft: no call may fail, block a match or
hide a series because Jikan did not answer.

Revisit this when Jikan stays down or changes shape for more than a few days,
or when AniList or MyAnimeList offers full episode lists without a per-user
credential.
