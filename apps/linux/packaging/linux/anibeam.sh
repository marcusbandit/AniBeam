#!/bin/sh
# /usr/bin/anibeam in the .deb and the .rpm: runs the AppDir in /opt/anibeam the way the
# AppImage's AppRun does. The hooks it sources are the ones appimage.sh and linuxdeploy
# wrote (libva's driver paths, the Qt platform theme); the Qt plugin and QML paths come
# from usr/bin/qt.conf and the libraries from the binary's RUNPATH, so nothing else is
# needed. APPIMAGE stays unset on purpose: the updater reads it, together with
# APPDIR, to tell an AppImage from a package, and this is a package.
APPDIR=/opt/anibeam
export APPDIR
for hook in "$APPDIR"/apprun-hooks/*.sh; do
  # shellcheck disable=SC1090
  [ -f "$hook" ] && . "$hook"
done
exec "$APPDIR/usr/bin/anibeam" "$@"
