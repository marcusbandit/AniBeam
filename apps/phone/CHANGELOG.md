# Anibeam for Phone

<!--
The app shows this file in What's new, and anibeam.app/whats-new/ builds from it.
Write for people who use Anibeam, not as a commit log:
- Pull requests do not add lines here. Curate each release from git history before
  shipping (`.github/changelog-draft.sh phone`), or update the newest heading as a
  living draft for major milestones.
- Start a release with a warm, natural intro saying what we're happy to bring to the
  app, followed by a short list of the actual highlights (`- **Feature.** ...`), an
  optional `### Heads up` when users need to know something, and a 1-2 line
  `### Fixes & polish` roundup for smaller fixes. Tiny patch releases can be a single
  paragraph.
- The newest heading matches the version in apps/phone/Cargo.toml, with an optional
  title of at most 40 characters: `## 2.5.0 · Watch together · 2026-11-21`.
- No em or en dashes. Keep lists tight (15 items at most; aim for 3-8).
The rules in full: CONTRIBUTING.md, under Patch notes.
-->

## 2.4.3 · iPhone nightly & full show pages · 2026-11-21

We're happy to bring the first sideloadable iPhone nightly builds, along with full show and list management, rolling downloads, Stats, and 13 languages across the phone app.

- **iPhone nightly & sideload source.** Install Anibeam and Anibeam Nightly side by side on iPhone through AltStore, SideStore, or LiveContainer using one shared source, with local and streamed playback.
- **Full show & list management.** Set list status, score, notes, start/finish dates, and rewatches from any show page, browse characters, related shows, recommendations, and IMDb scores, fix wrong matches, edit episode counts and air dates, and long-press episodes to mark watched up to there.
- **Rolling downloads & send to computer.** Keep long shows a few episodes ahead automatically, delete watched episodes to reclaim storage, hide shows, send downloads straight to your computer's library, and follow progress in notifications.
- **Home feed, Stats, and 13 languages.** Browse recently released and upcoming episodes on Home, check your watching Stats and year recap in More, explore any Airing season back to 1940, and use Anibeam in 13 languages.
- **Appearance, subtitles, and player controls.** Customize theme, accent color, text size, and subtitle styling in More, adjust audio/subtitle delay and jump chapter notches in the player, carry playback speed across episodes, pick between multiple files for an episode, and cast transcoded desktop streams to DLNA TVs.
- **Updates, backups, and Tailscale profiles.** GitHub APKs now notify you about Stable or Nightly updates in More, libraries can be backed up and restored, and shared Tailscale hosts show each person's own progress with automatic quality over relayed links.

### Fixes & polish

Fixed leaving Downloads freezing the app, made downloads keep their progress and pause state across restarts, improved pause/resume during phone calls, and fixed foldable crease layouts, iPhone video colors and status bar hiding, multi-season tracking, Jikan outages, MAL/AniList sync, secret masking, and lowercase Anibeam branding.

## 2.4.2 · 2026-11-02

Fixed a crash that caused the app to close immediately when opened from the home screen icon.

## 2.4.1 · 2026-11-02

Fixed a startup crash on phones.

## 2.4.0 · Streaming, Calendar & foldables · 2026-11-01

A major update for the phone app: stream your desktop library over Tailscale, browse what's airing in the new Calendar tab, and enjoy layouts designed for foldables and tablets.

- **Stream & cast from your computer.** Play episodes from your desktop library over Tailscale with selectable quality (Auto, Original, 1080p, 720p, 480p), sync watch progress back, and cast transcoded episodes to Chromecast.
- **Calendar & Airing now.** See what airs from two days back to a week ahead, browse this season's shows, and plan, download, or mark shows Not interested (synced with your computer).
- **Foldable & tablet layouts.** Unfolded screens switch to a side rail and split two-column pages; half-folded phones adapt cleanly around the crease, keeping video above and controls below.
- **Better player controls.** Swipe for brightness and volume, hold the video to play at 2x, lock the screen against stray touches, and keep Skip intro/outro accessible while controls hide.
- **Redesigned More & custom icons.** Searchable, per-page Settings in plain language, custom app icons (Default, Dark, or Line art), long-press poster actions, and retry buttons on failed downloads.

### Fixes & polish

Smoother animations and touch targets, cleaner batch cards and episode pickers, and fixes for foldable transitions, skip-intro timing, tracker and search errors, and secret storage.

## 2.2.0 · 2026-10-31

Added Torznab (Prowlarr and Jackett) and JSON source support in endpoint repos, polite per-site search pacing, and a fix keeping home-network names off public DNS.

## 2.1.1 · 2026-10-26

Fixed episode numbering for numbered specials and OVAs.

## 2.1.0 · 2026-10-25

Added in-app What's new after updates, also available anytime in More > About.

## 2.0.8 · 2026-10-24

Dragging the seek bar now moves only the preview until you let go, and letting go near where you started no longer seeks.

## 2.0.7 · 2026-10-24

Added frame thumbnail previews while dragging the seek bar, with live scrubbing as your finger moves.

## 2.0.4 · 2026-10-24

You can now select and delete episodes right from a series page. Also fixed remote watch counts, downloading unwatched episodes without following, and duplicate "in" in countdowns.

## 2.0.2 · 2026-10-24

Fixed endpoint card layouts on narrow phones and inline token prompts on endpoint rows.

## 2.0.0 · 2026-10-24

Version numbers now move in step with the desktop app (2.0.0). Tap any installed endpoint to add or update its token. Preloaded sources have been removed; add a repo or endpoint in More instead.

## 0.9.0 · 2026-10-24

Added endpoint repos (with key verification), manual custom endpoints, and an Endpoints step in first-run setup.

## 0.8.0 · 2026-10-23

Cast episodes to Chromecast or DLNA TVs directly from the player, using your phone as the remote.

## 0.7.0 · 2026-10-23

Built-in playback arrives on the same player engine as desktop, with Skip intro/outro, double-tap 10-second seeks, and AniList and MyAnimeList progress syncing.

## 0.6.1 · 2026-10-23

Fixed back navigation returning to search results, folder names containing slashes, following airing shows from AniList, and offline search errors.

## 0.6.0 · 2026-10-23

Added a dedicated Downloads tab with show search and an episode picker grid, and unified every screen with the desktop look.

## 0.5.4 · 2026-10-23

Added a button to copy connection check results, and improved loading on networks with DNS blocking.

## 0.5.3 · 2026-10-23

Added a connection check tool that shows where a source fails, from DNS to HTTPS.

## 0.5.1 · 2026-10-23

Sources blocked by a network's DNS now load automatically, with clearer status when a source is blocked.

## 0.4.0 · 2026-10-23

Search sources and download episodes directly on your phone without a desktop, with a storage cap slider and Wi-Fi-only download protection.

## 0.3.0 · 2026-10-23

The phone app now works standalone with Home, Library, More, and first-run setup.

## 0.2.0 · 2026-10-17

Added Calendar, Library, and series pages matching the desktop design.

## 0.1.2 · 2026-10-17

Fixed long titles wrapping cleanly onto two lines.

## 0.1.0 · 2026-10-17

First build of the phone app, showing upcoming air times and your desktop library.
