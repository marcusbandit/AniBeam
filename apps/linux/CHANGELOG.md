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

## 2.2.0 · 2026-10-25

### New
- What's new opens after an update, marked at the version you came from
- Click the version in the sidebar to open What's new again
- Endpoint repos in Settings, Downloads: add one, check its key, install
- Add an endpoint by hand, such as 1337x through Jackett
- Change an installed endpoint's token
- Move episodes to the trash from a series page
- Watching shows untouched for 60 days move to On hold on your tracker
- Home shows On hold beside Continue; playing one puts it back on Watching
- Settings, Library, Trackers sets the days, or turns the move off

### Fixed
- Episodes watched on another device no longer count as on disk

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
