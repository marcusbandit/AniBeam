#include "spikevideo.h"

SpikeVideo::SpikeVideo(QQuickItem *parent)
    : MpvAbstractItem(parent)
{
    connect(this, &MpvAbstractItem::ready, this, [this] {
        m_mpvVersion = getProperty(QStringLiteral("mpv-version")).toString();
        Q_EMIT mpvVersionChanged();
    });
}
