# Anibeam for desktop

<!--
The app shows this file in What's new, and anibeam.app/whats-new/ builds from it.
Write for people who use Anibeam, not as a commit log:
- Pull requests do not add lines here. Curate each release from git history before
  shipping (`.github/changelog-draft.sh desktop`), or update the newest heading as a
  living draft for major milestones.
- Start a release with a warm, natural intro saying what we're happy to bring to the
  app, followed by a short list of the actual highlights (`- **Feature.** ...`), an
  optional `### Heads up` when users need to know something, and a 1-2 line
  `### Fixes & polish` roundup for smaller fixes. Tiny patch releases can be a single
  paragraph.
- The newest heading matches the workspace version in the root Cargo.toml, with an
  optional title of at most 40 characters: `## 2.5.0 · Watch together · 2026-11-21`.
- No em or en dashes. Keep lists tight (15 items at most; aim for 3-8).
The rules in full: CONTRIBUTING.md, under Patch notes.
-->

## 2.5.0 · Watch together · 2026-11-21

We're happy to bring Watch together to Anibeam, along with 13 languages, personalized recommendations on Home, and a much cleaner layout across the app.

- **Watch together.** Start a room before or during an episode and invite friends with a six-letter code, a `rooms.anibeam.app` link, or a QR code. Friends who don't have the episode can grab the host's copy directly in the room, and playback can wait until everyone is ready.
- **Simpler navigation.** The sidebar is down to four tabs (Home, Library, Airing, Downloads), with Watching, Stats, and Show info folded into Library and Settings reorganized into seven cleaner sections.
- **Recommendations on Home.** Get show suggestions ranked from your own ratings, and open shows outside your library on their own page to grab their episodes.
- **13 languages.** Pick your language during setup or in Settings > Appearance.
- **Easier Linux installs.** Official apt (Ubuntu and Debian) and dnf (Fedora) repositories, a one-line installer on anibeam.app, and a much smaller AppImage.
- **Smoother downloads & setup.** Paste a source list once to turn on its sources, right-click active downloads to open their folder, and open magnet links on Linux or paste them straight into Downloads, matched to a show. Built-in qBittorrent now saves straight into your library folder.
- **Terminal control.** Drive an open Anibeam window from scripts or agents with `anibeam ctl`.

### Heads up

Anibeam is now spelled with a lowercase `b` and uses a new app ID, so you may need to re-pin it to your dock or taskbar. On macOS, allow it once more in Privacy & Security if prompted; on apt, run `apt update --allow-releaseinfo-change` once for the renamed repository.

### Fixes & polish

Fixes for multi-season episode tracking, season-pack folders, season matching and batch episode counts when getting episodes, long-running shows and outages on Jikan, AniList syncing, casting, converted streams, Mac dock icons, keyboard navigation, and narrow-window layouts.

## 2.4.0 · Quality of life · 2026-10-30

This is a huge quality-of-life update. Anibeam now streams to your phone over Tailscale, casts to your TV, searches and follows shows on its own built-in engine, and feels faster and clearer from first launch on.

- **Stream to your phone & cast to TV.** Share your library over Tailscale (even as a background service with no window open) with automatic transcoding, per-person progress, and downloads sent from your phone. You can also cast directly from the player to Chromecast and DLNA TVs.
- **Built-in downloads & richer sources.** Search and follow shows natively in Anibeam without needing an external helper, with Torznab (Prowlarr and Jackett) and JSON source support in endpoint repos.
- **First-run setup & clearer Settings.** A new setup walkthrough helps create or pick your library folder, connect a tracker, and match unmatched shows. Settings now has a searchable sidebar, and plain wording replaces jargon across the app.
- **Local lists, Stats, and Discord.** Track status and scores locally without an account, log start/finish dates, rewatches, and notes on AniList and MyAnimeList, explore your watching stats and year recap, export airing times to a calendar app, and show what you're watching on Discord.
- **Upgraded player & library.** Hold the video to play at 2x, jump between chapter markers, save screenshots with `S`, adjust audio and subtitle delay, pick between multiple files for the same episode, multi-select shows in Library and Airing, and customize episode schedules under Fixes.
- **Faster & self-updating.** Library, Airing, and Feed open 2 to 4x faster, in-app updates install and restart automatically (with a Stable or Nightly toggle), and screen readers and adjustable text sizes are supported throughout.

### Fixes & polish

Dozens of fixes across episode and special parsing, skip-intro timing, multi-release counts, Nvidia transcode seeking, credential masking in logs and URLs, tracker and qBittorrent error handling, keyboard shortcuts, and responsive layouts.

## 2.3.0 · 2026-10-27

Anibeam arrives on Windows, adds Airing now to browse the current season, and introduces rolling downloads for long-running shows.

- **Windows support.** Install for your user with the new Windows installer, or run the portable zip from anywhere.
- **Rolling downloads.** Keep long shows a set number of episodes ahead (12 by default), grabbing the next episode as you watch and clearing older watched files automatically.
- **Airing now.** Browse everything airing this season, mark shows Not interested, and plan or download episodes straight from the page.
- **Update notices & macOS media keys.** Get notified when a new release is out, control playback with Mac media keys and Control Centre, and right-click shows to move episodes to the trash.

### Fixes & polish

More reliable episode folder routing, Rip quality tags in release tables, a nine-day This week calendar, and fixes for airing statuses and "On disk" badges.

## 2.2.0 · 2026-10-26

Manage sources through endpoint repos, act on episodes straight from a series page, and see what changed after every update.

- **Endpoint repos.** Add source repositories in Settings > Downloads, add custom endpoints by hand, and update source tokens in place.
- **Series page tools.** Download episodes, move files to the trash, and fix misidentified episode files right from a series page.
- **What's new & On hold.** Release notes open automatically after an update, and shows untouched for 60 days move to On hold beside Continue.

### Fixes & polish

Double-click to toggle fullscreen, episode runtimes on series pages, and fixes for numbered specials, OVAs, and remote watch counts.

## 2.1.0 · 2026-10-24

Built-in Downloads arrives in the native app, the legacy Electron shell is retired, and browsing and playback get a thorough round of polish.

- **Downloads page.** Search releases by show, subscribe to airing series, and watch active downloads right inside Anibeam.
- **Richer series pages.** Added IMDb scores, episode cubes (including missing and upcoming episodes with predicted air dates), a Dropped shelf, and season sorting.
- **Keyboard & Homebrew.** Move between cards with arrow keys (`Return` to open, `Space` to play), and install on macOS with Homebrew.

### Fixes & polish

Natural browser-style scrolling, sharper Retina posters, display sleep prevention during playback, better episode filename parsing, and fixes for seeking, watch thresholds, and seasonal tracker sync. The Electron app and scroll speed slider have been removed.

## 2.0.0 · 2026-10-13

Anibeam has been rebuilt from the ground up as a fast native app, with a slim sidebar, Home's Continue row and week ahead, a single-grid Library with genre filtering, and TVmaze fallback matching. Also fixed shows without a known episode count being marked complete on their latest episode.

## 1.0.0 · 2026-09-03

The first release of Anibeam, with MyAnimeList search by default, subfolder library scanning, and a 1080p default window.
