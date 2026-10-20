#pragma once
// Free functions cxx-qt-lib 0.10 does not wrap. Each is declared to Rust in
// src/bridge/helpers.rs. Later tasks add to this file; the list at the end of Task 13 is
// the whole set.
#include <QtCore/QObject>
#include <QtCore/QString>
#include <QtGui/QColor>

class QQmlApplicationEngine;

void use_opengl_scene_graph();
void set_desktop_file_name(const QString &name);
// QSG_RENDER_LOOP=threaded on both GPUs (never on a Mac, where AppKit forbids it for
// OpenGL) and QT_XCB_GL_INTEGRATION=xcb_egl for the X11 fallback, set into this process's
// environment before QGuiApplication reads it. A value the user set in the environment wins.
void set_render_loop_env();
// Raises a window with the xdg-activation token a launcher or a second launch passed. The
// token goes into the environment first, because that is where Qt's Wayland plugin reads it.
void raise_window(QObject *window, const QString &token);
// The tokens into the application palette, so a stock control (the file dialog, a scroll
// bar) matches the shell.
void set_app_palette(const QColor &window, const QColor &text, const QColor &base, const QColor &highlight,
                     const QColor &highlightedText, const QColor &button, const QColor &buttonText);

// The offscreen driver's input (--drive). Each call goes in through the same window-system
// entry points QtTest uses, so an event takes the path a real one does: QGuiApplication's
// mouse and key processing, the shortcut map, double-click synthesis, then the window's
// delivery agent, which is what feeds MouseArea, the pointer handlers and Popup's
// close-on-press-outside. Coordinates are the window's own.
//
// kind: 0 move, 1 press, 2 release, 3 release that ends a gesture (the clock jumps past the
// double-click interval after it, so two separate clicks never fuse into a double click).
// button: 1 left, 2 right, 4 middle (Qt::MouseButton values).
void drive_mouse(QObject *window, int kind, double x, double y, int button, int modifiers);
// One wheel notch is angle delta 120, sent as Qt's Wayland plugin sends it.
void drive_wheel(QObject *window, double x, double y, int dx, int dy, int modifiers);
// A key press or release; key is a Qt::Key, text what the key types (empty for none).
void drive_key(QObject *window, bool press, int key, int modifiers, const QString &text);

// A poster's tone: its average colour, decoded at 8x12 (a JPEG decodes at that size through
// libjpeg's DCT scaling, so the cost is a fraction of a full decode), each pixel weighted by
// how saturated and bright it is, so a mostly white or black poster with one strong colour
// still comes back with that colour's hue. Deterministic for a given file, and cached per
// path for the process (the cache is guarded; the GUI thread is the expected caller). An
// unreadable or missing file is an invalid QColor, also cached.
QColor poster_tone(const QString &path);

// The pointer hidden over every window of the process (the player, once its chrome has
// gone), or shown again. An application override, not an item's cursor shape: the item
// under a resting pointer is not re-asked for its cursor on Wayland until the pointer moves,
// so an item-level blank cursor left the arrow standing on the picture.
void set_cursor_hidden(bool hidden);
// Opens a URL (or a file:// path) with the desktop's handler by running xdg-open
// detached, rather than through QDesktopServices: on Wayland Qt first asks the
// compositor for an xdg-activation token and launches nothing until it arrives, and on
// Hyprland the token never came, so every link in the shell did nothing. A Mac runs
// open(1) instead. Returns false when the opener could not be started.
bool open_url(const QString &url);

// Dev only: serves the shell's own QML files from a directory on disk instead of the
// compiled resources, so a QML-only edit needs no cargo build. Every file the engine would
// load from the module's resource path is redirected to the same name under `dir` when
// that file exists there, and anything else (the C++ types, a file not in `dir`) loads as
// before. The interceptor lives as long as the process.
void install_qml_dir(QQmlApplicationEngine &engine, const QString &dir);
