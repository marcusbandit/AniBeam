# AniBeam for desktop

<!--
The app shows this file in What's new, newest version first. Keep it short.
- One line per change a person can notice, under New, Changed, Fixed or Removed.
- 80 characters at most, no full stop. Say what they see, never why or how.
- The newest heading is the workspace version in the root Cargo.toml. A version
  bump adds a heading, and a new heading bumps the version.
- A version may have a title between its number and its day, 40 characters at
  most: "## 2.5.0 · Quality of life · 2026-11-02". The release goes out under it.
- Tests, CI, refactors and docs get no line.
The rules in full: CONTRIBUTING.md, under Patch notes.
-->

## 2.4.0 · Quality of life · 2026-10-30

### New
- Developer options, in Settings under About, time every page and turn images off
- Pick Stable or Nightly updates in Settings, under About
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
- A version can have a name, shown in What's new and when an update is out
- Hold a show on Airing now, or Ctrl+click it, to pick several at once
- Put every picked show on Planning, or mark them all Not interested
- Get lights the episodes a click will pick, before you click
- Hold the mouse on the video to play at 2x until you let go
- Page Up and Page Down jump between chapters, now marked on the seek bar
- S saves a screenshot of the video to Pictures/AniBeam
- Home goes to the start of an episode and End to its outro
- F11 toggles fullscreen
- A resumed episode says where it picked up, with Start over
- The Next button names the next episode, or only its number if you like
- Arrow keys and Enter pick audio and subtitle tracks
- N, P and C say why when there is nothing to do
- Right-click a show for Open, Open folder, Copy title and Open on AniList
- Right-click an episode to play it, open its folder or copy its path
- Forward, from the menu, Alt+Right or the mouse's forward button
- Library tabs show how many each holds, and so do Airing now's filters
- Ctrl+F searches the page you are on
- Select and copy a show's synopsis
- Pick all when deleting episodes
- Characters open their page on AniList
- Marking several episodes watched at once has Undo, and so has Hide
- Toasts have a close button and stay while the pointer is on them
- A missing episode in Metadata opens Get
- This week on Home links to Airing now
- Airing now's menu leads with Play next for shows you have
- Mark episodes watched on a show with no tracker; AniBeam keeps them
- An unmatched show shows what folder and file names match
- Right-click a release to copy its magnet or open its page
- Open the folder a download is saving to
- A failed source says why, and a click asks it again
- A slow search offers the other sources
- Downloads opens with your last search, unless you switch that off
- Reset subtitles to how they came, in Settings, Playback
- Check for updates now, in Settings, About
- Open the folder an export went to
- Click a line in Activity to copy it
- Shift+F10 or the Menu key opens the right-click menu
- Tooltips show on keyboard focus too
- Press ? anywhere for the list of keys
- Exports carry an episode count you set by hand
- The player's controls set subtitle and audio delay; a and A shift the audio
- Discord shows what you are watching, a switch in Settings, Playback
- An endpoint repo can search your own Prowlarr or Jackett over torznab
- An endpoint repo can add a source that answers in JSON

### Changed
- Searches wait a second between requests to one site, except your own server
- Settings lists its sections down the left, each saying how it stands now
- Search settings: type a word and land on the setting it names
- Accounts, Subtitles, Keyboard, Phone app and About are sections of their own
- Unsaved download settings wait in a bar with Save, wherever you scroll
- A show's Fixes menu holds Fix files, the episode count, the match and Rescan
- Skip outro stays up with the controls hidden, like Skip intro
- Skipping an intro lands right where the episode starts
- The speed you watch at carries over to the next episode
- Tracks read in plain words, like English (file)
- Switching a track says which one on the video
- The rating prompt goes away by itself
- AniBeam opens at the size and place you left it
- Settings opens on the tab you left it on
- A page from the rail comes back with its search, filter and scroll
- The rail icon of the page you are on takes it back to the top
- See all on Home opens the Library at that list
- Clear match and Forget ask before they drop anything
- Closing Fix files with fixes not applied asks first
- A click anywhere on a match result applies it, and Return works too
- The match panel says where a show is matched, without ids
- Searching for an unmatched show starts from a clean folder name
- Tabs and switches across the app work with the arrow keys
- Films are called movies everywhere in the Library
- Library's controls read Sort by and Group by
- A downloading show says Cancel, and clears what it wrote unless you untick it
- Download all takes the releases the table shows, and says how many
- Rows hold still until seeders and dates are in, then sort once
- Your column sort stays for the next search
- Speeds say down or up, sizes go up to TB, and seeders are called seeders
- Later on an update hides it until AniBeam opens again, not for good
- Unsaved Downloads settings survive a switch to another tab
- Test saves your changes first, then tests them
- A subtitle value Settings cannot use turns red
- Sliders say what they count, like 60 days
- Enter flips a switch, as Space does
- Activity's Clear asks first, Copy says Copied, older lines show the day
- Downloading now shows the rest past twelve on a click
- The status strip says Nothing is running when nothing is
- Reduce motion stops every slide and grow, scrolling aside
- Settings and errors use plain words instead of anirss terms
- The activity log speaks plain words and leaves out background chores
- Library, Airing and Feed open two to four times faster and use less memory

### Fixed
- A download's speed and time left no longer run under its Stop button
- The Replay button no longer stays on the video after you go back
- Watching an episode again after its end keeps your place
- The rating prompt no longer covers the subtitle delay line
- The Movies tab counts movies, not series
- Airing now's count follows the filter
- Space on a Watching show you do not have no longer opens an empty page
- A tracker that refuses a change now says so
- Watching no longer says Connected before you connect AniList
- An empty Home shows how to connect AniList or MyAnimeList
- Tooltips no longer blink and vanish when moving between cards
- A show you watched to the end with no tracker reads caught up
- An unfolded run of repeated lines in Activity folds back up
- Episode cubes in Get now grow under the pointer

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
