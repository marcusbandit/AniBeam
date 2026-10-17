# AniBeam for Android

A phone companion to the desktop AniBeam. It shows what airs next for everything you
subscribe to or are watching, and what the desktop has on disk. It syncs with the desktop
over your tailnet while that machine is on, and keeps working from the last sync when it
is off.

Everything is Rust: the UI is [Slint](https://slint.dev) on its Android backend, behind
the stock `android.app.NativeActivity`. There is no Java or Kotlin in the app and no
Gradle in the build.

## What it does

- **Next up** lists every anirss subscription and every releasing series in the library,
  soonest first, with a live countdown, the episode and the local air time. An episode
  that has aired but was not on the host's disk at the last sync is marked in orange and
  sorts to the top. Shows with no known air time go last. Pull down to sync.
- **On disk** shows the host's series with their episodes (watched ones dimmed) and its
  torrents with progress. A banner says whether it is live or how old it is.
- **Settings** holds the host address, a connection test, and what the last sync and the
  last AniList refresh did.

It syncs on launch, when it comes back to the foreground, on pull to refresh, and every
five minutes while open: hello, then the snapshot, then any poster whose tag changed.
The snapshot and the posters live in the app's internal storage, so everything shows
offline. While the host does not answer, the countdowns keep running from the cached
schedule, and the app asks AniList's public API for fresh airing times, all shows in one
request and at most once an hour.

## Using it on a phone

1. Install the Tailscale app on the phone, sign in to the same tailnet as the desktop, and
   keep it connected. The host only listens on its Tailscale addresses.
2. On the desktop, switch on **Share over Tailscale** in AniBeam.
3. Sideload the APK. Either copy `dist/AniBeam-<version>.apk` to the phone and open it
   (Android asks once to allow installs from that app), or with USB debugging on:

   ```sh
   adb install -r dist/AniBeam-0.1.1.apk
   ```

4. Open AniBeam. It starts out pointed at `banditbox:8790`, the desktop's MagicDNS name.
   If the phone cannot resolve the short name, use the full one in Settings,
   `banditbox.tail24c6a9.ts.net:8790`, or the machine's `100.x.y.z` address. **Test
   connection** says whether AniBeam answered and which version it is.

The APK is signed with the debug key in `~/.android/debug.keystore`. Android only installs
an update over an existing install when both are signed with the same key, so build
updates on a machine with that keystore, or uninstall first.

## Building

You need:

- rustup with the Android targets: `rustup target add aarch64-linux-android x86_64-linux-android`
- the Android SDK (`~/Android/Sdk` or `$ANDROID_HOME`) with `platforms;android-36`, a
  recent `build-tools`, and the NDK (the newest under `ndk/` is used)
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
android/            AndroidManifest.xml (package, version, NativeActivity), icons, theme
companion/          the logic, no UI: client, store, AniList, Next up rows, On disk rows,
                    sync. Tests run on the desktop; tests/mock_host.rs syncs with the mock.
src/                the app: android_main, the UI thread, the sync thread, poster decoding
ui/                 the Slint screens; theme.slint is the one token file
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
