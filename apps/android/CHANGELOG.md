# AniBeam for Android

<!--
The app shows this file in What's new, newest version first. Keep it short.
- One line per change a person can notice, under New, Changed, Fixed or Removed.
- 80 characters at most, no full stop. Say what they see, never why or how.
- The newest heading is the app's versionName. A version bump adds a heading,
  and a new heading bumps the version.
- Tests, CI, refactors and docs get no line.
The rules in full: CONTRIBUTING.md, under Patch notes.
-->

## 2.4.0 · 2026-11-01

### New
- Airing now, in the Calendar: this season's shows, Plan to watch and Get
- Not interested hides a show here and on your computer
- See your computer's calendar beside this phone's
- Pick the app icon in More: Default, Dark, or Line art in your wallpaper colours
- A Calendar tab: what airs from two days back to a week ahead
- Unfolded, the tabs move to a rail and pages split into two columns
- Half folded, everything moves off the crease to either side of it
- Stood up half folded, the video plays above the crease, controls below
- Downloads show the show's poster; tap one to open the show
- Tap a site you added yourself to change its name or address
- Lock the player so a stray touch does nothing
- Swipe up or down on the left for brightness, on the right for volume
- Hold the picture to play at 2x until you let go
- Hold a poster to get or delete its episodes
- Tap a missing episode to find it in Downloads
- Search fields have a clear button
- Refresh your computer's library, and check repos for new sites
- A download stopped by an error offers Retry
- Play episodes from your computer's library over Tailscale
- Pick how streams play in More: Auto, Original, 1080p, 720p or 480p
- Cast an episode from your computer to a Chromecast, converted to play
- Episodes watched from your computer keep their place there

### Changed
- Bending the phone slides everything into place instead of jumping
- Half folded, the two sides sit either side of the crease without boxes
- Turned sideways and half folded, pages keep the unfolded layout
- Home lists the next episodes to air, with a way into the Calendar
- Downloads shows free space instead of upload speed
- When your computer cannot be reached, the app says what to do about it
- An episode your computer cannot stream yet says to update AniBeam there
- Editing a site keeps its keys out of the address and offers to change them
- More opens on a list of settings, each saying how it stands right now
- Each part of the settings opens as its own page
- Search the settings to land right on the one you want
- Paused, the player keeps its controls up
- Skip intro and Skip outro stay up while the controls hide
- Tap anywhere on a switch row to flip it
- Back leaves delete mode before it leaves the show
- Settings say search sites instead of endpoints, and plain words for the rest

### Fixed
- The buttons above What to get and the source buttons are no longer cut off
- Library's posters glide into place when the phone bends instead of jittering
- Sideways and half folded, the video keeps its shape above the crease
- Rows of buttons that fit no longer scroll
- Skip intro skips the opening, not the scene before it
- A long search wraps instead of running off the screen
- Storage says how much the episodes use
- A failed sign-in says why
- No more "Nothing on this phone yet" while the app starts

## 2.2.0 · 2026-10-31

### New
- An endpoint repo can search your own Prowlarr or Jackett over torznab
- An endpoint repo can add a source that answers in JSON

### Changed
- Searches wait a second between requests to one site, except your own server

### Fixed
- The connection check keeps home-network names off public DNS

## 2.1.1 · 2026-10-26

### Fixed
- Numbered specials and OVAs keep their own episode numbers

## 2.1.0 · 2026-10-25

### New
- What's new opens after an update, marked at the version you came from
- What's new stays in More, under About

## 2.0.8 · 2026-10-24

### Changed
- Dragging the seek bar moves only the preview until you let go

### Fixed
- Letting go near where the drag started no longer seeks

## 2.0.7 · 2026-10-24

### New
- A thumbnail of the frame under your finger while you drag the seek bar

### Changed
- The video follows your finger while you drag the seek bar

## 2.0.4 · 2026-10-24

### New
- Delete episodes from a series page: pick them, then Delete

### Fixed
- Episodes watched on another device no longer count as on this phone
- Those episodes can be downloaded without following the show
- Countdowns read "in 3d 1h", not "in in 3d 1h"

## 2.0.2 · 2026-10-24

### Fixed
- Endpoint cards keep their buttons inside on narrow phones
- An endpoint that needs a token asks for it inside its own row

## 2.0.0 · 2026-10-24

### Changed
- The version jumps to 2.0.0, in step with the desktop
- Tap an installed endpoint to add or change its token

### Removed
- The preloaded sources: add a repo or an endpoint under More

## 0.9.0 · 2026-10-24

### New
- Endpoint repos: add one by its address, check its key, install endpoints
- Add an endpoint by hand, such as 1337x through Jackett
- First run has an Endpoints step

## 0.8.0 · 2026-10-23

### New
- Cast to a Chromecast or a DLNA TV from the player
- The phone is the remote while an episode casts

## 0.7.0 · 2026-10-23

### New
- Episodes play in the app, on the same player as the desktop
- Skip intro, Skip outro and the next episode, as on the desktop
- Double tap a side of the picture to jump ten seconds
- Watching marks your AniList and MyAnimeList lists

## 0.6.1 · 2026-10-23

### Fixed
- Back from a show returns to your search instead of closing the app
- Shows with a slash in the name download into a working folder
- Airing shows found through AniList can be followed
- With no connection, search says AniList could not be reached

## 0.6.0 · 2026-10-23

### New
- Downloads has its own tab
- Search by show: type a series, see who has it, pick episodes off a grid

### Changed
- Every screen takes the look of the desktop's Downloads page

## 0.5.4 · 2026-10-23

### New
- Copy the connection check's results

### Fixed
- Nyaa loads on networks that block it by name

## 0.5.3 · 2026-10-23

### New
- Check the connection: see where a source stops, from DNS to HTTPS

## 0.5.1 · 2026-10-23

### Changed
- Sources blocked by a network's DNS load anyway
- A blocked source says so

## 0.4.0 · 2026-10-23

### New
- Search your sources and download on the phone, no desktop needed
- A storage cap, set with a slider

### Changed
- Downloads wait for Wi-Fi unless you allow mobile data

## 0.3.0 · 2026-10-23

### New
- The phone is its own AniBeam: Home, Library, More and a first-run setup

## 0.2.0 · 2026-10-17

### New
- Calendar, library and series pages, in the desktop's look

## 0.1.2 · 2026-10-17

### Fixed
- Long titles wrap onto two lines

## 0.1.0 · 2026-10-17

### New
- First build: what airs next and what the desktop has on disk
