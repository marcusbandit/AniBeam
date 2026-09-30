#pragma once
// The video surface: MpvQt's MpvAbstractItem, which owns the mpv core, its thread and the
// render context. This subclass only forwards the controller's signals to QML, exposes the
// observation formats, and loads a config layer through `include` after init (spec 5.2).
#include <MpvQt/MpvAbstractItem>
#include <QtQml/qqmlregistration.h>
#include <QtCore/QString>
#include <QtCore/QVariant>

class VideoItem : public MpvAbstractItem
{
    Q_OBJECT
    QML_ELEMENT
public:
    enum Format {
        Flag = MPV_FORMAT_FLAG,
        Int64 = MPV_FORMAT_INT64,
        Double = MPV_FORMAT_DOUBLE,
        String = MPV_FORMAT_STRING,
        Node = MPV_FORMAT_NODE
    };
    Q_ENUM(Format)

    explicit VideoItem(QQuickItem *parent = nullptr);

    Q_INVOKABLE void observe(const QString &name, int format);
    /// Parses a config file as if each line were set one by one; init-only lines are ignored.
    Q_INVOKABLE void include(const QString &path);

Q_SIGNALS:
    void loaded();
    void ended(const QString &reason);
    void changed(const QString &name, const QVariant &value);
    void reconfigured();
};
