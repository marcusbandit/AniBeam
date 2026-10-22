## Install

**Mac** (Apple silicon, macOS 15 or later): download `AniBeam-@VERSION@-macos-arm64.dmg`, open it and drag AniBeam into Applications. The first launch is blocked because the build is not notarized. Go to System Settings > Privacy & Security and click Open Anyway.

**Linux, any distribution:** download `AniBeam-@VERSION@-x86_64.AppImage`, then `chmod +x` it and run it. It needs glibc 2.39 or newer, which covers Ubuntu 24.04, Debian 13 and Fedora 40 and later.

**Arch:** `yay -S anibeam` (or `paru -S anibeam`) builds this tag from the AUR.

**Downloads page:** nothing to install. AniBeam brings its own anirss, and the first time the page opens it uses your qBittorrent if its Web UI answers, or sets up its own if not. The README walks through the rest: https://github.com/marcusbandit/AniBeam#getting-started
