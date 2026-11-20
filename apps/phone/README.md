# Anibeam for Android

Anibeam on the phone, as its own app: it runs the same core as the desktop over its own
library, so what you keep on the phone is the phone's, and only tracking is shared through
AniList and MyAnimeList. It can also show another Anibeam's library over your tailnet, read
only.

Everything is Rust: the UI is [Slint](https://slint.dev) on its Android backend, behind
the stock `android.app.NativeActivity`, and the core is `anibeam-core` from `core/`. There
is no Gradle in the build.

## What it does

The look is the desktop's Downloads page at phone size: flat glass panes with hairline
edges, pills for every control, page and section titles in Archivo's tall condensed cut with
a dim line of fact beside them, and B612 for figures. Slint cannot set a variable font's
axes, so `scripts/cut-fonts.py` cuts each of the desktop's Archivo roles into a static font
under `ui/fonts/`.

Four tabs:

- **Home** is this phone's: the next episode to air, this week with a dot on each day
  something airs, and the recently watched and recently downloaded shelves.
- **Library** is every show on the phone as a poster with the desktop's episode cubes, with
  search. When another Anibeam on the tailnet has answered, a switcher above shows its
  library instead. Tap a poster for its series page: the facts, the cubes, the synopsis,
  every episode, Play for the next one up, and Get episodes, which opens the show in
  Downloads. A tap on any whole episode plays it.
- **Downloads** works the way the desktop's does, shows first. Typing lists shows, the
  library's and then AniList's; a show opens its view: where to get it (every source and
  what it answered), its releases as rows of episode cubes to tap, and a plan that says
  what Get and Follow will do. Return searches release names instead, with the desktop's
  refine steps and best fit. With nothing typed it shows what is downloading, what finished,
  and four lenses: Gaps, Airing, Following, Downloaded.
- **More** holds the settings: storage and its cap (a slider up to the phone's whole
  storage), Wi-Fi only, the sources (a filtered RSS search, and any RSS feed, such as one that needs a token, with your own
  address), tracker sign-in, the other device's address, and the version.

**Onboarding** runs once: AniList and MAL sign-in, Wi-Fi only (on by default; turning it
off asks first), the storage cap, suggested at 18 GB and off unless chosen, and the
download notification, shown as it will look before Android asks for it. More can run it
again. Turned down, the notification has a row in More and a line in Downloads while
something downloads, both opening the app's notification settings.

**The player** is the desktop's, on the same libmpv: the core opens the session, keeps the
resume point and the history, and marks the trackers; the track pick and Anibeam's mpv
options are the core's too, so a file plays the same on both. The screen turns to landscape
with the system bars hidden. The controls island has the time, the groove with the skip
windows on it, previous, play and next, Skip intro or outro while inside one, the audio and
subtitle pickers, speed and Mark watched; the auto-next pill rises at the outro and counts
five seconds, and Replay sits on the last frame. A tap shows or hides the controls, and a
double tap on either side jumps ten seconds. mpv draws each frame through its render API
into a texture on a GL context shared with Slint's (`src/video.rs`), so it never touches
the context Slint draws with.

**Casting**: the cast button at the player's top right looks for Chromecasts (mDNS) and
DLNA TVs (SSDP) on the Wi-Fi while its sheet is up, and hands the episode over from where it
is, with the audio and subtitles playing here. The phone stays the remote: play, pause,
seek, skip intro, the next episode, and the position comes back from the TV for progress and
tracking. Nothing is re-encoded: a Chromecast gets HLS cut on keyframes from the Matroska
file (fragmented MP4, so HEVC works where the device decodes it), a DLNA TV gets the file as
it is, or the same fragments as one stream if it refuses. Subtitles go as text (WebVTT for
Chromecast, SRT for TVs that read it), so ASS styling is lost. 10-bit H.264 (Hi10P) is refused
with a reason until the transcode (#141) lands. The cast code is in `playback/src/cast/`;
`core/examples/cast_drive.rs` drives a whole cast from the desktop. To try it from the
emulator, which cannot see the LAN's multicast, write `/data/local/tmp/anibeam-cast-dev` with
`renderer=<a DLNA description URL>`, `port=47000` and `advertise=127.0.0.1:47000`, and
`adb forward tcp:47000 tcp:47000`.

Downloads run in the core's own torrent engine (`librqbit`, the core's `phone` feature),
held while the phone is off Wi-Fi unless mobile data was allowed. While anything
downloads, a small Java service (`android/java/`) keeps a notification up so Android does
not cut the network with the screen off; it shows what is downloading with a bar and the
numbers Downloads shows, and a finished episode posts one of its own, on a channel that can
be turned off apart from the first. The player's screen calls, landscape and the bars
and keeping the display awake, are Java too, since only Android's UI thread may make them. A finished download stops: the phone does not
seed.

The core keeps its database and cache under the app's `files/core`; episodes live in
`files/Anime`, the one library folder. The other device's snapshot and posters are cached
too, so its library still shows when it is off.

## Using it on a phone

1. Install the Tailscale app on the phone, sign in to the same tailnet as the desktop, and
   keep it connected. The host only listens on its Tailscale addresses.
2. On the desktop, switch on **Share over Tailscale** in Anibeam.
3. Sideload the APK. Either copy `dist/Anibeam-<version>.apk` to the phone and open it
   (Android asks once to allow installs from that app), or with USB debugging on:

   ```sh
   adb install -r dist/Anibeam-0.8.0.apk
   ```

4. Open Anibeam. Onboarding's last step asks for your computer: its MagicDNS name and port,
   such as `my-computer:8790`. If the phone cannot resolve the short name, use the full one,
   `my-computer.example.ts.net:8790`, or the machine's `100.x.y.z` address. **Test**
   says whether Anibeam answered and which version it is.

Release APKs are signed with the project's key, so each one installs over the last. An APK
built in a checkout is signed with that machine's `~/.android/debug.keystore`, and its
versionCode sits below every nightly and release of its version, so it goes onto a phone
that has one of those only after an uninstall. On the emulator that is no trouble.

## Building

You need:

- rustup. The repo's `rust-toolchain.toml` picks the Rust version, and `build.sh` adds the
  Android targets to it the first time it builds for one
- the Android SDK (`~/Android/Sdk` or `$ANDROID_HOME`) with `platforms;android-36`, a
  recent `build-tools`, and the NDK (the newest under `ndk/` is used)
- curl and unzip: `build.sh` runs `scripts/fetch-mpv.sh` first, which takes libmpv and its
  ffmpeg out of a pinned mpv-android release APK (checked by hash) into `vendor/mpv/`,
  about 30 MB per ABI and not in git
- JDK 17 at `/usr/lib/jvm/java-17-openjdk` or `$JAVA_HOME`. Slint compiles its small
  keyboard helper with javac and d8 during the build; that is Slint's code, not the app's.

```sh
scripts/build.sh                 # x86_64 APK for the emulator: build/Anibeam-debug.apk
scripts/build.sh --abi all       # arm64-v8a and x86_64 in one APK
scripts/build.sh --dist          # both, copied to dist/Anibeam-<versionName>.apk
scripts/build.sh --bundle        # App Bundle for Google Play: build/Anibeam-release.aab
scripts/build.sh cargo test -p anibeam-companion   # any cargo command, same limits
```

`--bundle` builds the Play flavor (`ANIBEAM_FLAVOR=play`) with a pinned bundletool
(`scripts/fetch-bundletool.sh`) and signs it with jarsigner and the APK's key. To try it on the
emulator: `java -jar "$(scripts/fetch-bundletool.sh)" build-apks --local-testing --bundle
build/Anibeam-release.aab --output build/Anibeam.apks`, then the same `java -jar` with
`install-apks --apks build/Anibeam.apks`.

`build.sh` runs inside a systemd scope capped at 6 GB with swap forbidden
(`ANIBEAM_BUILD_MEM` changes the cap), and refuses to start with less than 7 GB free.

This directory is its own Cargo workspace, so the root's checks skip it. Before a PR, run
these here. `.github/workflows/android.yml` runs them too, and clippy once more for the
emulator's target, which catches code only the phone compiles:

```sh
cargo fmt --check -p anibeam-android -p anibeam-companion
scripts/build.sh cargo test --workspace
scripts/build.sh cargo clippy --workspace --all-targets -- -D warnings
```

The version lives in `Cargo.toml` and nowhere else. `build.sh` works the APK's versionName
and versionCode out of it, or out of `ANIBEAM_RELEASE_VERSION` when CI ships the build as a
nightly or a release, and stops when the newest heading in `CHANGELOG.md` is not that
version. Builds for people come from CI, never from a checkout: CONTRIBUTING.md, under
Releases.

### iOS

The same app builds for the iPhone, without video for now. Slint draws through winit
there, and `src/main.rs` is the executable that `ios/cargo.sh`, the Xcode project's build
step, puts in the bundle. You need a Mac with Xcode, xcodegen (`brew install xcodegen`)
and the simulator target (`rustup target add aarch64-apple-ios-sim`).

```sh
scripts/build-ios.sh                       # build/ios/Build/Products/Debug-iphonesimulator/Anibeam.app
xcrun simctl install booted build/ios/Build/Products/Debug-iphonesimulator/Anibeam.app
xcrun simctl launch --console-pty booted com.marcusrosado.anibeam
```

`scripts/build-ios.sh ipa` builds `build/ios/Anibeam.ipa` for a device, unsigned: AltStore and
LiveContainer sign it themselves when they install it. It needs the `aarch64-apple-ios` target.
A PR's test build comment carries Add to AltStore, SideStore and LiveContainer links: they open
`site/install/`, which hands the IPA's release URL to the app's `<app>://install?url=` scheme.

Info.plist asks for the local network (`NSLocalNetworkUsageDescription`), which the host
and casting need, and declares `_googlecast._tcp` under `NSBonjourServices`. Finding devices
to cast to still needs more on a sideloaded build: the core's Chromecast search (mDNS) and
DLNA search (SSDP) send their own multicast, which iOS only allows an app holding the
`com.apple.developer.networking.multicast` entitlement. Apple grants it on request to a
paid developer account's provisioning profile, so an AltStore or LiveContainer install
never carries it, and the plist cannot add it. Until then the iPhone finds no Chromecast or
DLNA renderer on its own.

`librqbit-dualstack-sockets` 0.7.0 does not build for iOS, so Cargo.toml pins the fix
from its pull request #7 under `[patch.crates-io]`. Drop the pin once a release has it.

## The emulator loop

```sh
scripts/create-avd.sh      # once: the anibeam_pixel AVD, Pixel 10 Pro, Android 16
scripts/emulator.sh        # start it headless, capped at 7 GB, console port 5580
scripts/run.sh             # build, install, launch (--no-build to skip the build)
scripts/shot.sh name       # screenshot into captures/name.png over adb
scripts/stop-emulator.sh   # sync the guest, then stop it
```

The scripts only talk to `emulator-5580`, so other emulators on the same adb server are
left alone.

The phone lays out for three postures, worked out in `ui/fold.slint`: folded (the bar along
the bottom, one column), open (a rail and two columns) and half folded, where the panes sit
either side of the crease, left and right held like a book or above and below stood up like
a laptop. To try them, use the fold AVD, which has a hinge:

```sh
scripts/create-avd.sh --fold                     # once: anibeam_fold, Pixel 10 Pro Fold
ANIBEAM_AVD=anibeam_fold scripts/emulator.sh
ANIBEAM_AVD=anibeam_fold scripts/fold.sh half 120  # fold, unfold, half [degrees], state
scripts/seed-demo.sh                             # shows airing this week, and downloads
```

A clamshell folds the other way, its crease across the screen held upright. The flip AVD
has one, and the app reads where the crease runs from the phone's own report (Android's
window extensions) rather than guessing a book fold:

```sh
scripts/create-avd.sh --flip                     # once: anibeam_flip, 6.7 inch clamshell
ANIBEAM_AVD=anibeam_flip scripts/emulator.sh
ANIBEAM_AVD=anibeam_flip scripts/fold.sh half 100
```

A foldable guest takes 4 GB, so the 7 GB cap is the floor. Folding locks the emulator unless
`adb shell settings put secure fold_lock_behavior_setting stay_awake_on_fold_key` is set
once.

Without a desktop to talk to, run the mock host and point the app at `10.0.2.2:8790`,
which is how the emulator reaches the machine it runs on:

```sh
python3 scripts/mock-host.py --demo    # the fixture plus six shows airing now on AniList
python3 scripts/mock-host.py --aired   # an episode that aired just before the snapshot
```

## Driving the app over adb

Slint hands Android no accessibility tree, so `uiautomator` sees one blank surface.
`scripts/ctl.sh` drives the running app by name instead, on the emulator or a phone
(`ANDROID_SERIAL`, or `-s <serial>`), and prints what the app answers. The words are the
desktop's `anibeam ctl` ones where the two apps do the same thing:

```sh
scripts/ctl.sh tree                   # the page, what is open over it, every control shown
scripts/ctl.sh go downloads           # home, calendar, library, downloads, more
scripts/ctl.sh click "Get episodes"   # words on screen or x,y; --nth 2 picks the second match
scripts/ctl.sh shot captures/a.png    # the device's own pixels
scripts/ctl.sh 'go series Frieren; wait Play 5s; state'   # one line, ; between commands
scripts/ctl.sh help                   # every command
```

| Command | Example |
| --- | --- |
| `go <tab>`, `where`, `back`, `key <name> ...` | `go library`, `key down enter` |
| `go series <title\|id>` | `go series 12`: opens the show on the phone |
| `go downloads <words>` | `go downloads frieren`: as if typed and Return pressed |
| `click <words\|x,y> [--nth n]` | `click Continue`: the control's own action, or a tap on it |
| `tap`, `hold <words\|x,y>` | `tap 206,640`, `hold Frieren` |
| `type <text>`, `fill <n\|placeholder> <text>` | `fill 1 one piece` |
| `scroll <dir> [px] [at <words>]`, `swipe x1 y1 x2 y2 [ms]` | `scroll down 900` |
| `onboarding [next\|back\|step n\|finish\|start]` | `onboarding step 3` |
| `tree [filter]`, `find <text>`, `wait <words> [ns]` | `wait "Get episodes" 10s` |
| `state`, `perf [ns]` | `perf 5s`: UI loop waits, frame times, the busiest threads |
| `shot <file.png>` | from the powered display, as `shot.sh` takes it |

A label held by several controls, or words that only sit inside a label, get the matches
back instead of a press. A line has 45 s, 8 s on Android 13 and older, where every app's
broadcasts wait behind it. A `wait` or `perf` that does not fit is cut to what the line has
left; a cut-short wait names the wait to send next, and ctl.sh sends it until the label shows
or the time is used. ctl.sh exits 1 when a command failed, 2 when the app did not answer and
64 when ctl.sh was called wrong.

`press`, `open`, `search` and `dump` still work as the older names. Points are the window's
logical pixels, the ones `tree` prints. The app answers in the broadcast's result data
(`am broadcast -a com.marcusrosado.anibeam.CONTROL -p com.marcusrosado.anibeam --es cmd
'<line>'`), and only the shell user and the system may send it: the receiver asks for
`android.permission.DUMP`. The Play build has none of it (`src/control/`). Completion for zsh
and bash:

```sh
ln -s "$PWD/scripts/completions/_ctl.sh" ~/.local/share/zsh/site-functions/
ln -s "$PWD/scripts/completions/ctl.sh.bash" ~/.local/share/bash-completion/completions/ctl.sh
```

## Layout

```text
android/            AndroidManifest.xml (package, version, NativeActivity), icons, theme,
                    java/ with the one download service
companion/          the logic, no UI: client, store, AniList, Next up rows, On disk rows,
                    sync. Tests run on the desktop; tests/mock_host.rs syncs with the mock.
src/                the app: the UI thread (app.rs), the phone's core (local.rs, phone.rs),
                    the other device (remote.rs, worker.rs), the phone's settings (prefs.rs)
src/platform/       everything that differs by platform: android.rs holds android_main and
                    the JNI for the browser, the service, the network and the screen;
                    ios.rs holds ios_main, the hinge, and the language and light or
                    dark setting; other.rs stands in on every other target
src/main.rs         the iPhone's executable
ios/                the iPhone's Xcode project (project.yml for xcodegen), Info.plist
                    and cargo.sh, the build step that runs cargo
ui/                 the Slint screens; theme.slint is the one token file, fonts/ holds
                    Archivo and B612 as on the desktop
scripts/            the build and emulator loop, and the mock host
```

The wire format is the `anibeam-host-api` crate in `host-api/` at the repository root,
shared with the desktop's server. This directory is its own Cargo workspace, so desktop
builds and CI never compile it.

## Not in this version

No notifications when an episode airs. Android only wakes a closed app at a set time
through an `AlarmManager` alarm delivered to a `BroadcastReceiver`, and a receiver is a
Java class, which this all-Rust app does not have. A notification posted from Rust while
the app happens to be running would miss most episodes, so there is none.
