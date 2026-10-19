#include "helpers.h"
#include <QtCore/QByteArray>
#include <QtCore/qglobal.h>
#include <QtGui/QGuiApplication>
#include <QtGui/QPalette>
#include <QtGui/QWindow>
#include <QtQuick/QQuickWindow>
#include <QtCore/QElapsedTimer>
#include <QtCore/QPointF>
#include <QtCore/QPoint>
#include <QtCore/QHash>
#include <QtCore/QMutex>
#include <QtCore/QMutexLocker>
#include <QtGui/QImage>
#include <QtGui/QImageReader>
#include <QtGui/QCursor>
#include <QtCore/QProcess>
#include <QtCore/QUrl>
#include <QtGui/QWheelEvent>
#include <QtGui/QPointingDevice>

// Exported by QtGui for QtTest's benefit and declared nowhere else public; the declarations
// are QtTest's own (qtestmouse.h, qtestkeyboard.h, qtestwheel.h). Using them rather than
// QGuiApplication::sendEvent is what makes a synthetic key reach the shortcut map and a
// second press become a double click, and links nothing beyond QtGui.
QT_BEGIN_NAMESPACE
Q_GUI_EXPORT void qt_handleMouseEvent(QWindow *window, const QPointF &local, const QPointF &global,
                                      Qt::MouseButtons state, Qt::MouseButton button, QEvent::Type type,
                                      Qt::KeyboardModifiers mods, int timestamp);
Q_GUI_EXPORT void qt_handleKeyEvent(QWindow *w, QEvent::Type t, int k, Qt::KeyboardModifiers mods,
                                    const QString &text, bool autorep, ushort count);
Q_GUI_EXPORT void qt_handleWheelEvent(QWindow *window, const QPointF &local, const QPointF &global,
                                      QPoint pixelDelta, QPoint angleDelta, Qt::KeyboardModifiers mods,
                                      Qt::ScrollPhase phase);
QT_END_NAMESPACE

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

QColor poster_tone(const QString &path)
{
    static QMutex lock;
    static QHash<QString, QColor> cache;
    {
        QMutexLocker held(&lock);
        auto hit = cache.constFind(path);
        if (hit != cache.constEnd())
            return *hit;
    }
    QColor tone;
    if (!path.isEmpty()) {
        QImageReader reader(path);
        reader.setAutoTransform(false);
        reader.setScaledSize(QSize(8, 12));
        const QImage image = reader.read().convertToFormat(QImage::Format_RGB32);
        double r = 0, g = 0, b = 0, total = 0;
        for (int y = 0; y < image.height(); ++y) {
            const QRgb *line = reinterpret_cast<const QRgb *>(image.constScanLine(y));
            for (int x = 0; x < image.width(); ++x) {
                const QColor c = QColor::fromRgb(line[x]);
                // A grey pixel counts once; a fully saturated, bright one five times.
                const double weight = 1.0 + 4.0 * c.hsvSaturationF() * c.valueF();
                r += c.redF() * weight;
                g += c.greenF() * weight;
                b += c.blueF() * weight;
                total += weight;
            }
        }
        if (total > 0)
            tone = QColor::fromRgbF(float(r / total), float(g / total), float(b / total));
    }
    QMutexLocker held(&lock);
    cache.insert(path, tone);
    return tone;
}

namespace {
// The driver's mouse clock: real elapsed time, kept strictly increasing, plus a skew that
// jumps past the double-click interval after each finished click. QtTest does the same.
int driveTimestamp(int skewAfter = 0)
{
    static QElapsedTimer clock;
    static qint64 skew = 0;
    static int last = 0;
    if (!clock.isValid())
        clock.start();
    int now = int(clock.elapsed() + skew);
    if (now <= last)
        now = last + 1;
    last = now;
    skew += skewAfter;
    return now;
}
Qt::MouseButtons driveButtons;
}

void drive_mouse(QObject *window, int kind, double x, double y, int button, int modifiers)
{
    auto *w = qobject_cast<QWindow *>(window);
    if (!w)
        return;
    const QPointF local(x, y);
    const QPointF global = w->mapToGlobal(local);
    const auto mods = Qt::KeyboardModifiers(modifiers) & Qt::KeyboardModifierMask;
    const auto b = Qt::MouseButton(button);
    switch (kind) {
    case 1:
        driveButtons.setFlag(b, true);
        qt_handleMouseEvent(w, local, global, driveButtons, b, QEvent::MouseButtonPress, mods, driveTimestamp());
        break;
    case 2:
    case 3:
        driveButtons.setFlag(b, false);
        qt_handleMouseEvent(w, local, global, driveButtons, b, QEvent::MouseButtonRelease, mods,
                            driveTimestamp(kind == 3 ? 1000 : 0));
        break;
    default:
        qt_handleMouseEvent(w, local, global, driveButtons, Qt::NoButton, QEvent::MouseMove, mods, driveTimestamp());
        break;
    }
}

void drive_wheel(QObject *window, double x, double y, int dx, int dy, int modifiers)
{
    auto *w = qobject_cast<QWindow *>(window);
    if (!w)
        return;
    const QPointF local(x, y);
    // Shaped like a notch from Qt's Wayland plugin, which is what the shell runs on: a device
    // typed as a touchpad, libinput's 15 px beside the 120 angle, and no phase
    // The id is one no platform hands out, so it never stands in for the real core pointer
    static QPointingDevice wayland(QStringLiteral("drive wheel"), 0x7d17e, QInputDevice::DeviceType::TouchPad,
                                   QPointingDevice::PointerType::Generic,
                                   QInputDevice::Capability::Position | QInputDevice::Capability::Scroll |
                                       QInputDevice::Capability::PixelScroll,
                                   1, 0);
    QWheelEvent e(local, w->mapToGlobal(local), QPoint(dx, dy) / 8, QPoint(dx, dy), Qt::NoButton,
                  Qt::KeyboardModifiers(modifiers) & Qt::KeyboardModifierMask, Qt::NoScrollPhase, false,
                  Qt::MouseEventNotSynthesized, &wayland);
    e.setTimestamp(driveTimestamp());
    QGuiApplication::sendEvent(w, &e);
}

void drive_key(QObject *window, bool press, int key, int modifiers, const QString &text)
{
    auto *w = qobject_cast<QWindow *>(window);
    if (!w)
        return;
    qt_handleKeyEvent(w, press ? QEvent::KeyPress : QEvent::KeyRelease, key,
                      Qt::KeyboardModifiers(modifiers) & Qt::KeyboardModifierMask, text, false, 1);
}

void set_cursor_hidden(bool hidden)
{
    static bool overridden = false;
    if (hidden && !overridden) {
        QGuiApplication::setOverrideCursor(QCursor(Qt::BlankCursor));
        overridden = true;
    } else if (!hidden && overridden) {
        QGuiApplication::restoreOverrideCursor();
        overridden = false;
    }
}

bool open_url(const QString &url)
{
    QString target = url;
    // A bare local path becomes a file URL, so xdg-open hands it to the file manager
    if (target.startsWith(QLatin1Char('/')))
        target = QUrl::fromLocalFile(target).toString();
    return QProcess::startDetached(QStringLiteral("xdg-open"), {target});
}
