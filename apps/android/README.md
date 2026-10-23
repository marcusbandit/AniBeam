# AniBeam for Android

AniBeam on the phone, as its own app: it runs the same core as the desktop over its own
library, so what you keep on the phone is the phone's, and only tracking is shared through
AniList and MyAnimeList. It can also show another AniBeam's library over your tailnet, read
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
  search. When another AniBeam on the tailnet has answered, a switcher above shows its
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
  storage), Wi-Fi only, the sources (nyaa, and any RSS feed such as erai with your own
  address), tracker sign-in, the other device's address, and the version.

**Onboarding** runs once: AniList and MAL sign-in, Wi-Fi only (on by default; turning it
off asks first), and the storage cap, suggested at 18 GB and off unless chosen. More can
run it again.

**The player** is the desktop's, on the same libmpv: the core opens the session, keeps the
resume point and the history, and marks the trackers; the track pick and AniBeam's mpv
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
with a reason until the transcode (#141) lands. The core's cast code is in `core/src/cast/`;
`core/examples/cast_drive.rs` drives a whole cast from the desktop. To try it from the
emulator, which cannot see the LAN's multicast, write `/data/local/tmp/anibeam-cast-dev` with
`renderer=<a DLNA description URL>`, `port=47000` and `advertise=127.0.0.1:47000`, and
`adb forward tcp:47000 tcp:47000`.

Downloads run in the core's own torrent engine (`librqbit`, the core's `phone` feature),
held while the phone is off Wi-Fi unless mobile data was allowed. While anything
downloads, a small Java service (`android/java/`) keeps a notification up so Android does
not cut the network with the screen off. The player's screen calls, landscape and the bars
and keeping the display awake, are Java too, since only Android's UI thread may make them. A finished download stops: the phone does not
seed.

The core keeps its database and cache under the app's `files/core`; episodes live in
`files/Anime`, the one library folder. The other device's snapshot and posters are cached
too, so its library still shows when it is off.

## Using it on a phone

1. Install the Tailscale app on the phone, sign in to the same tailnet as the desktop, and
   keep it connected. The host only listens on its Tailscale addresses.
2. On the desktop, switch on **Share over Tailscale** in AniBeam.
3. Sideload the APK. Either copy `dist/AniBeam-<version>.apk` to the phone and open it
   (Android asks once to allow installs from that app), or with USB debugging on:

   ```sh
   adb install -r dist/AniBeam-0.8.0.apk
   ```

4. Open AniBeam. It starts out pointed at `banditbox:8790`, the desktop's MagicDNS name.
   If the phone cannot resolve the short name, use the full one in More,
   `banditbox.tail24c6a9.ts.net:8790`, or the machine's `100.x.y.z` address. **Test**
   says whether AniBeam answered and which version it is.

The APK is signed with the debug key in `~/.android/debug.keystore`. Android only installs
an update over an existing install when both are signed with the same key, so build
updates on a machine with that keystore, or uninstall first.

## Building

You need:

- rustup with the Android targets: `rustup target add aarch64-linux-android x86_64-linux-android`
- the Android SDK (`~/Android/Sdk` or `$ANDROID_HOME`) with `platforms;android-36`, a
  recent `build-tools`, and the NDK (the newest under `ndk/` is used)
- curl and unzip: `build.sh` runs `scripts/fetch-mpv.sh` first, which takes libmpv and its
  ffmpeg out of a pinned mpv-android release APK (checked by hash) into `vendor/mpv/`,
  about 30 MB per ABI and not in git
- JDK 17 at `/usr/lib/jvm/java-17-openjdk` or `$JAVA_HOME`. Slint compiles its small
  keyboard helper with javac and d8 during the build; that is Slint's code, not the app's.

```sh
scripts/build.sh                 # x86_64 APK for the emulator: build/AniBeam-debug.apk
scripts/build.sh --abi all       # arm64-v8a and x86_64 in one APK
scripts/build.sh --dist          # both, copied to dist/AniBeam-<versionName>.apk
scripts/build.sh cargo test -p anibeam-companion   # any cargo command, same limits
```

`build.sh` runs inside a systemd scope capped at 6 GB with swap forbidden
(`ANIBEAM_BUILD_MEM` changes the cap), and refuses to start with less than 7 GB free.

The version lives in `android/AndroidManifest.xml` (`versionCode` and `versionName`).
Bump both for every build that leaves the machine; `build.sh` stops if `versionName` and
the `version` in `Cargo.toml` disagree.

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

Without a desktop to talk to, run the mock host and point the app at `10.0.2.2:8790`,
which is how the emulator reaches the machine it runs on:

```sh
python3 scripts/mock-host.py --demo    # the fixture plus six shows airing now on AniList
python3 scripts/mock-host.py --aired   # an episode that aired just before the snapshot
```

## Layout

```text
android/            AndroidManifest.xml (package, version, NativeActivity), icons, theme,
                    java/ with the one download service
companion/          the logic, no UI: client, store, AniList, Next up rows, On disk rows,
                    sync. Tests run on the desktop; tests/mock_host.rs syncs with the mock.
src/                the app: android_main, the UI thread (app.rs), the phone's core
                    (local.rs, phone.rs), the other device (remote.rs, worker.rs), the
                    phone's settings (prefs.rs), JNI for the browser, the service and the network
                    (android.rs)
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
