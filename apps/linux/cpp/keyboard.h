// Whether Ctrl is held, app-wide, for QML that previews what a Ctrl+click would do before
// the click (the Downloads show view lights every episode a Ctrl+click picks). QML sees a
// modifier only on the event in hand, so a press with the pointer standing still would go
// unseen; this watches the keys and the pointer for the whole app instead.
#pragma once

#include <QObject>
#include <QtQml/qqmlregistration.h>

class Keyboard : public QObject
{
    Q_OBJECT
    QML_ELEMENT
    QML_SINGLETON
    Q_PROPERTY(bool ctrl READ ctrl NOTIFY ctrlChanged)

public:
    explicit Keyboard(QObject *parent = nullptr);
    bool ctrl() const { return m_ctrl; }

signals:
    void ctrlChanged();

protected:
    bool eventFilter(QObject *watched, QEvent *event) override;

private:
    void set(bool ctrl);
    bool m_ctrl = false;
};
