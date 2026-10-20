# Downloads redesign: from the mockup to the shell

The mockup on `mockup/downloads-visual` (`mockups/downloads/index.html`) was accepted as the
direction on 2026-10-19. This plan turns it into the real Qt shell. The owner's notes on the
mockup decide where the plan and the mockup disagree:

- Episode cubes replace the single progress bar everywhere in the app, coloured by state.
- The app's own font (Archivo, through the Tokens roles) stays. Numbers get a face made for
  numerals.
- The app's own rail stays; the mockup's left strip was scaffolding. The Activity column stays.
- Titles larger and more prominent across the existing pages; readability carries over.
- Rounder than the mockup: the app's tile and pill radii, not square cells.
- Clicking a show never downloads. It opens the show view, which lists the providers; clicking
  a provider fetches what it has. That view is where a new show or feed is added.
- A provider switch, with 1337x on it. 1337x is a service that must be started; AniBeam only
  ever checks whether it runs, and a search never starts it.

## 1. Episode cubes

One cube per episode, a short rounded square, in one row that wraps. A cube's colour is its
state. The states are a table, so a new one ("not caught up") is one row in the core and one
entry in `Tokens.cubeStates`.

| Code | State | Colour (theme slot) | Notes |
|---|---|---|---|
| `w` | Watched, tracked online | green (base0B) | tracker progress covers it |
| `l` | Watched here, not tracked | cyan (base0C) | in `completed`, past tracker progress |
| `d` | On disk, untouched | ink (white on dark) | |
| `t` | Downloading | purple (base0E) | from the torrent client, Downloads only |
| `m` | Missing | yellow (base0A) | aired, not on disk, a later episode is |
| `e` | Error | red (base08) | stalled torrent, files gone |
| `g` | Not downloaded | grey (glass) | aired, not on disk, nothing after it |
| `f` | Not aired | outline only | |

The colours come from the theme's own hues, so every bundled theme keeps a palette that holds
together. Green and cyan sit next to each other on purpose: both mean watched.

Precedence when an episode qualifies twice: `t` > `e` > `w` > `l` > `d` > `m` > `g` > `f`.

Data:

- Core: `SeriesCard.cubes: String`, one code per episode 1..total (capped at 400; an estimate
  total still draws, its tail as `f`). Built in `library/cards.rs` from the episode files, the
  tracker progress, the `completed` keys and the airing rows. Season one only when the files
  name several seasons, as `episodes_on_disk` does today.
- Shell: `EpisodeCubes.qml` takes the string (or a JS array of codes) and draws it. Size
  `small` on cards, `normal` in panels. A tooltip names the episode and its state.

Replaced: the foot bar on `Card.qml`, the bar on `ShowCard.qml` and `ShowRow.qml`, the rule bar
on the Downloads page, the cell grid in `ShowDetail.qml`, the squares in `EpisodesPane.qml`, and
the watched bar on `SeriesPage.qml`. Playback seek bars and the disk usage bar stay bars: those
are positions and sizes, not episodes.

## 2. Type

- A `figure` role in `Tokens.qml` using B612 (Airbus' cockpit face, OFL, bundled in
  `assets/fonts`), tabular. Speeds, sizes, counts, seeders, times, percentages.
- Page titles: `typeDisplay` from 2.8x to 4x the system size; the page heads grow with them.
  Section headings from 1.5x to 2.2x in the display cut.
- `textDim` reads a step lighter on dark themes, for body copy on glass.

## 3. The Downloads page

Two columns: the page and the Activity column (about 340 px, hidden under 1100 px of width,
where Activity moves under the page).

Activity: the transfer rate, the active torrents with cubes for their episodes and a bar for
the byte progress, then today's Downloads events from the log, newest first.

The page, in order:

1. Head: "Downloads" in the display cut, one status line (anirss, the client, following N).
2. Search: one wide field. Typing two letters lists shows: the library's own first, then
   AniList's (`SearchProvider`), each with its cubes and one fact line. Enter searches
   releases by name, the old flow, untouched (`ReleaseTable`, filters, Subscribe).
3. Lenses: Gaps in your library, Airing, Following, Downloaded.
   - Gaps: library series with `m` cubes, or with an aired tail they are following. One row
     each: poster, title, what is missing, cubes.
   - Airing: the library's airing series as posters with cubes.
   - Following and Downloaded: the existing subscription and torrent bands.
4. The show view, opened from any show anywhere on the page. Never a download by itself.

### The show view: adding a show or a feed

```
 [band: poster, title in display, facts: airing / N of M out / on disk / following?]

 Where to get it                                   [ Get N episodes and follow ]
 +---------------------------------------------+   +---------------------------+
 | nyaa           anime torrents      Ready   >|   | plan panel                |
 | erai           RSS feed            Ready   >|   |  what, from whom, size    |
 | 1337x          Jackett, on demand  Off  [ ]|   |  saves to <folder>        |
 +---------------------------------------------+   |  follow new episodes  [x] |
                                                    +---------------------------+
 nyaa: 31 releases, 4 groups          (after a click)
   Your library   [cubes 1..12]
   SubsPlease     [cubes: available / picked / have]
   Erai-raws      ...
   batches        one long pill per batch
```

- The provider list is anirss's endpoints. Each row says what the endpoint is and whether it
  can answer now, with no search: plain endpoints are Ready; an on-demand one is Running or
  Not running, from a cheap probe of its ready URL.
- Clicking a provider runs `SearchReleases` on that one endpoint with the show's title. Its
  rows open under the list: one row per release group and quality, a cube per episode
  (available, with the seeders as a short underline; owned; picked), and batches as one pill.
- Clicking a group's name picks its releases for every episode the library lacks; clicking a
  cube picks or drops one. The plan panel says what will happen: the episodes, the group, the
  size, the folder, and whether new episodes will follow.
- "Get and follow" downloads the picks and subscribes with the group's query on that endpoint
  (`Subscribe` with `wire_query`), which is how a new feed is added. "Get" alone downloads.
  "Follow" alone subscribes. A show not in the library gets its folder under the endpoint's
  save base, named by its title; the scan picks it up when the first file lands.
- "Show every release as a list" opens the existing release table for that search.

## 4. Providers and 1337x

- Core: `SearchEndpoint.running: Option<bool>`; `None` for a plain endpoint, `Some` for an
  on-demand one, from a GET on its `ready` URL with a 1.5 s timeout. Read with the endpoint list
  and again on `CheckEndpoints`, a new call that only probes.
- A search never starts a service: the core sends `fallback: false` whenever an on-demand
  endpoint is not answering, and refuses a search aimed at an on-demand endpoint whose ready URL
  does not answer at that moment ("1337x is not running; AniBeam does not start it").
- Shell: a Sources switch row in the Downloads head (and the show view's provider list). Plain
  endpoints default on. An on-demand endpoint defaults to on when running, off when not, and
  says why. Off endpoints are never searched. "Check again" re-probes.

## Order of work

1. Core: cubes on `SeriesCard`, `running` on endpoints, the search guard, `CheckEndpoints`,
   torrent episode facts. Tests.
2. Shell: tokens (cube states, figure role, title sizes), `EpisodeCubes.qml`, the swaps.
3. Shell: the Downloads page, Activity column, show view, provider switch.
4. Release build, captures on a copy of real data, drive scripts updated.
