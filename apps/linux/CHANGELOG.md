# AniBeam for desktop

<!--
The app shows this file in What's new, newest version first. Keep it short.
- One line per change a person can notice, under New, Changed, Fixed or Removed.
- 80 characters at most, no full stop. Say what they see, never why or how.
- The newest heading is the workspace version in the root Cargo.toml. A version
  bump adds a heading, and a new heading bumps the version.
- Tests, CI, refactors and docs get no line.
The rules in full: CONTRIBUTING.md, under Patch notes.
-->

## 2.4.0 · 2026-10-30

### New
- Developer options, in Settings under Data, time every page and turn images off
- Magnet-only search results show their seeders and leechers
- Pick Stable or Nightly updates in Settings, under Data
- Update installs the new version and restarts, no trip to GitHub
- An AUR install shows the command that updates it, once the AUR has it
- Downloaded lists every show on disk, not only what the torrent client holds
- Downloaded shows how much of the disk is watched and how much is not
- Delete watched clears every watched episode, or every finished show you saw
- Right-click a show on Gaps or Downloaded to delete it
- Deleting episodes removes their torrent too, so they can be downloaded again
- Ctrl-click a release group or one of its episodes to pick all it has missing
- Get says it is sending to your torrent client the moment you click it
- Set how many episodes a TVmaze show's season has, so no more are predicted

### Changed
- A show's Fixes menu holds Fix files, the episode count, the match and Rescan
- The activity log speaks plain words and leaves out background chores

### Fixed
- A download's speed and time left no longer run under its Stop button

## 2.3.0 · 2026-10-27

### New
- AniBeam runs on Windows: an installer, or a zip to run from anywhere
- Long shows download a few episodes at a time, 12 ahead by default
- Each episode you finish starts the next; older watched ones leave the disk
- Get more episodes now, or all of them, from a rolling download
- Rolling download settings in Settings, Downloads
- A notice says when a new version is out, with a button to download it
- Airing now: every show on air, and each episode as it comes out
- Mark a show not interested and it stays off Airing now
- Plan to watch or get episodes straight from Airing now
- Media keys and Control Centre play, pause and skip on the Mac
- Set how many episodes a show has, and nothing past it is predicted
- Release lists show a Rip column: WEB-DL, WEBRip, BluRay and the rest
- This week shows nine days, two back and six ahead, every day even when empty
- Right-click a card to delete its watched episodes or all of them, to the trash

### Fixed
- This week cards keep the airing time inside the card
- On disk marks only the release you have, not every release of that episode
- An episode that aired earlier today stays on Today and says if it landed
- An episode that aired with no file yet says it is late
- Episodes you get for a show land in its folder and join it, not new shows

## 2.2.0 · 2026-10-26

### New
- What's new opens after an update, marked at the version you came from
- Click the version in the sidebar to open What's new again
- Endpoint repos in Settings, Downloads: add one, check its key, install
- Add an endpoint by hand, such as 1337x through Jackett
- Change an installed endpoint's token
- Move episodes to the trash from a series page
- Get episodes straight from a series page
- Fix files that read as the wrong episode from the series page
- The series page says how long an episode runs
- Watching series untouched for 60 days move to On hold, beside Continue
- Double-click the picture to toggle fullscreen

### Fixed
- Episodes watched on another device no longer count as on disk
- Numbered specials and OVAs keep their own episode numbers

## 2.1.0 · 2026-10-24

### New
- The Downloads page: search by show, subscribe, see what is downloading
- IMDb scores on the series page
- Dropped series get their own shelf, out of every suggestion
- The series band shows other titles, scores and list status
- Seasons in the library, with Order and Sort
- Episode cubes on the series page, missing and upcoming episodes included
- Predicted air dates for episodes not announced yet
- Episodes take their title from the file name when no provider has one
- Arrow keys move between cards; Return opens one, Space plays it
- Install on a Mac with brew

### Changed
- The wheel scrolls the way your browser does
- Pages scroll from anywhere in the window
- Home's Continue row follows what your tracker says you started
- The week on Home says which episode lands, and when
- Skip intro sits where Next does
- Group franchises is a switch, kept over any order in the library
- Metadata is one grid of tiles, with an episodes pane

### Fixed
- The display stays awake while an episode plays
- Posters fill their cards on a Retina display
- An episode counts as watched at the mark line, not only at its very end
- The series page's play button counts what you watched here
- The seek bar scrubs, and a release off the groove still seeks
- The removal sheet removes what it says
- Tracker progress counts the season the files are in
- An airing show's newest episode is not taken for its last
- File names like 01x36, OVAs and S05E extras read the right episode

### Removed
- The Electron app
- The scroll speed slider

## 2.0.0 · 2026-10-13

### New
- AniBeam is rebuilt as a native app
- Home: the series to go on with, the week ahead, episodes waiting
- The library as sections of one grid, filtered by a genre picked on Home
- A slim sidebar with Home and Library first
- TVmaze matches series that AniList does not carry

### Fixed
- A series with no episode count is not marked watched at its newest episode

## 1.0.0 · 2026-09-03

### New
- The first release
- MyAnimeList is the default search

### Changed
- The window opens at 1080p

### Fixed
- Search finds series in sub-folders
