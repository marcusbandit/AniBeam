#pragma once
// A C++ MpvAbstractItem subclass registered into the same QML module as the Rust
// singleton. Registration and construction only: the libmpv spike covers rendering.
#include <MpvQt/MpvAbstractItem>
#include <QtQml/qqmlregistration.h>

class SpikeVideo : public MpvAbstractItem
{
    Q_OBJECT
    QML_ELEMENT
    Q_PROPERTY(QString mpvVersion READ mpvVersion NOTIFY mpvVersionChanged)
public:
    explicit SpikeVideo(QQuickItem *parent = nullptr);
    QString mpvVersion() const { return m_mpvVersion; }

Q_SIGNALS:
    void mpvVersionChanged();

private:
    QString m_mpvVersion;
};
