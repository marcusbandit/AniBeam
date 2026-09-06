#include "helpers.h"
#include <QtCore/QByteArray>
#include <QtCore/qglobal.h>
#include <QtGui/QGuiApplication>
#include <QtGui/QPalette>
#include <QtGui/QWindow>
#include <QtQuick/QQuickWindow>

void use_opengl_scene_graph()
{
    QQuickWindow::setGraphicsApi(QSGRendererInterface::OpenGL);
}

void set_desktop_file_name(const QString &name)
{
    QGuiApplication::setDesktopFileName(name);
}

void set_render_loop_env()
{
    if (qgetenv("QSG_RENDER_LOOP").isEmpty())
        qputenv("QSG_RENDER_LOOP", QByteArrayLiteral("threaded"));
    if (qgetenv("QT_XCB_GL_INTEGRATION").isEmpty())
        qputenv("QT_XCB_GL_INTEGRATION", QByteArrayLiteral("xcb_egl"));
}

void raise_window(QObject *window, const QString &token)
{
    auto *w = qobject_cast<QWindow *>(window);
    if (!w)
        return;
    // The only setenv in the shell that runs with other threads alive, and it runs there
    // because Qt's Wayland plugin reads the xdg-activation token from the environment and
    // offers no API to hand it one. The plugin takes the value and unsets it as part of
    // requestActivate, so the window is the only reader and the variable is gone by the
    // time this returns. On xcb nothing reads it and the value lingers, harmlessly: an
    // xdg-activation token means nothing to an X11 server. Everything else that sets the
    // environment does so in main, before the core's runtime exists.
    if (!token.isEmpty())
        qputenv("XDG_ACTIVATION_TOKEN", token.toUtf8());
    w->show();
    w->raise();
    w->requestActivate();
}

void set_app_palette(const QColor &window, const QColor &text, const QColor &base, const QColor &highlight,
                     const QColor &highlightedText, const QColor &button, const QColor &buttonText)
{
    QPalette p = QGuiApplication::palette();
    p.setColor(QPalette::Window, window);
    p.setColor(QPalette::WindowText, text);
    p.setColor(QPalette::Base, base);
    p.setColor(QPalette::Text, text);
    p.setColor(QPalette::Highlight, highlight);
    p.setColor(QPalette::HighlightedText, highlightedText);
    p.setColor(QPalette::Button, button);
    p.setColor(QPalette::ButtonText, buttonText);
    p.setColor(QPalette::Accent, highlight);
    QGuiApplication::setPalette(p);
}
