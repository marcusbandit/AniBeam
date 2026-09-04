# libmpv inside a Qt Quick window: throwaway spike

Wayfinder tickets #9 (NVIDIA desktop) and #18 (AMD laptop). Findings live in `docs/spikes/libmpv-qml.md` and
`docs/spikes/libmpv-qml-laptop.md`. This code is a probe, not a
starting point for the shell; it exists so the numbers in that document can be reproduced.

Build (needs `mpvqt`, `qt6-declarative`, `cmake`, `ninja` from the Arch repos):

    cmake -S spikes/libmpv-qml -B build-spike -G Ninja -DCMAKE_BUILD_TYPE=RelWithDebInfo
    ninja -C build-spike

Run interactively (space pauses, `,` and `.` step frames, `f` toggles fullscreen, `m` unmutes,
`r` prints a property report, `q` quits):

    build-spike/mpvspike /path/to/episode.mkv --hwdec=auto --preview

`--script` runs the 68 second scripted sequence and quits; `run.sh` wraps it with the Hyprland
window moves and screenshots used for the ticket and prints the observed events. Every event is a
`SPIKE <tag> <json>` line on stdout; mpv's own verbose log goes to `mpv-player.log` and
`mpv-preview.log` in the `--out` directory.

`quality.sh NAME FILE [args]` is the wrapper for ticket #23's quality matrix: it plays a fixed
60 seconds with whatever `--set key=value` options are given, samples the GPU four times a second,
and grabs the panel on each `--stills` timestamp. `matrix.sh` runs all 23 configs, `table.py` prints
drops and GPU load, `compare2.py` diffs every config's stills against its block's baseline.
Findings live in `docs/spikes/mpv-quality-options-laptop.md`.

`run-laptop.sh` is the same wrapper for the laptop (one monitor, no window moves; `QPA=xcb` and
`RENDER_LOOP=basic` select the variants) and `occlude.sh` switches the workspace away and back to count
the frames dropped while the window is hidden. Both expect the tree exported to `~/spike-libmpv`.
