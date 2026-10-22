# AniBeam for Android

AniBeam on the phone, as its own app: it runs the same core as the desktop over its own
library, so what you keep on the phone is the phone's, and only tracking is shared through
AniList and MyAnimeList. It can also show another AniBeam's library over your tailnet, read
only.

Everything is Rust: the UI is [Slint](https://slint.dev) on its Android backend, behind
the stock `android.app.NativeActivity`, and the core is `anibeam-core` from `core/`. There
is no Gradle in the build.

## What it does

- **Onboarding** runs once: AniList and MAL sign-in, Wi-Fi only (on by default; turning it
  off asks first), and a storage limit, suggested at 18 GB and off unless chosen. More can
  run it again.
- **Home** is this phone's: the next episode to air on a band in its poster's colour, this
  week with a dot on each day something airs, and the recently watched and recently
  downloaded shelves.
- **Library** is every show on the phone as a poster with the desktop's episode cubes, with
  search. When another AniBeam on the tailnet has answered, a switcher above shows its
  library instead. Tap a poster for its series page: the facts, the cubes, the synopsis and
  every episode, on the phone, missing or still to air.
- **More** holds the Wi-Fi and storage settings, tracker sign-in, the other device's
  address, and the version. Search and download come in the next version (#111).

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
   adb install -r dist/AniBeam-0.3.0.apk
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
src/                the app: android_main, the UI thread (app.rs), the phone's core
                    (local.rs, phone.rs), the other device (remote.rs, worker.rs), the
                    phone's settings (prefs.rs), JNI for the browser (android.rs)
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
