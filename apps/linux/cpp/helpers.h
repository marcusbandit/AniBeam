#pragma once
// Free functions cxx-qt-lib 0.10 does not wrap. Each is declared to Rust in
// src/bridge/helpers.rs. Later tasks add to this file; the list at the end of Task 13 is
// the whole set.
#include <QtCore/QObject>
#include <QtCore/QString>
#include <QtGui/QColor>

void use_opengl_scene_graph();
void set_desktop_file_name(const QString &name);
// QSG_RENDER_LOOP=threaded on both GPUs and QT_XCB_GL_INTEGRATION=xcb_egl for the X11
// fallback, set into this process's environment before QGuiApplication reads it. A value
// the user set in the environment wins.
void set_render_loop_env();
// Raises a window with the xdg-activation token a launcher or a second launch passed. The
// token goes into the environment first, because that is where Qt's Wayland plugin reads it.
void raise_window(QObject *window, const QString &token);
// The tokens into the application palette, so a stock control (the file dialog, a scroll
// bar) matches the shell.
void set_app_palette(const QColor &window, const QColor &text, const QColor &base, const QColor &highlight,
                     const QColor &highlightedText, const QColor &button, const QColor &buttonText);
