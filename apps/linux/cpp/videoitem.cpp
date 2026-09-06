#include "videoitem.h"
#include <MpvQt/MpvController>
#include <QtCore/QDebug>

VideoItem::VideoItem(QQuickItem *parent)
    : MpvAbstractItem(parent)
{
    // Queued, every one: the controller emits from its own thread, and the page's handlers
    // touch QML properties, which belong to the GUI thread.
    connect(mpvController(), &MpvController::fileLoaded, this, &VideoItem::loaded, Qt::QueuedConnection);
    connect(mpvController(), &MpvController::endFile, this, &VideoItem::ended, Qt::QueuedConnection);
    connect(mpvController(), &MpvController::propertyChanged, this, &VideoItem::changed, Qt::QueuedConnection);
    connect(mpvController(), &MpvController::videoReconfig, this, &VideoItem::reconfigured, Qt::QueuedConnection);
}

void VideoItem::observe(const QString &name, int format)
{
    observeProperty(name, static_cast<mpv_format>(format));
}

void VideoItem::include(const QString &path)
{
    // A layer mpv rejects is the user's own mpv.conf far more often than ours, and the
    // only sign of it otherwise is options that quietly did not apply, so it is reported.
    const int code = setPropertyBlocking(QStringLiteral("include"), path);
    if (code < 0)
        qWarning("anibeam: mpv refused the config layer %s: %s", qUtf8Printable(path), qUtf8Printable(MpvController::getError(code)));
}
