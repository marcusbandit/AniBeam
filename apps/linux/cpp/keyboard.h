// Whether Ctrl is held, app-wide, for QML that previews what a Ctrl+click would do before
// the click (the Downloads show view lights every episode a Ctrl+click picks). QML sees a
// modifier only on the event in hand, so a press with the pointer standing still would go
// unseen; this watches the keys and the pointer for the whole app instead.
//
// It also says when focus is moving because of Tab (#627): `tab` is 1 for Tab and -1 for
// Shift+Tab from the moment the press arrives until it has been handled, and 0 otherwise.
// A plain Item has no focus reason in QML, so this is how a page tells a Tab move from a
// click or a focus put back by code.
#pragma once

#include <QObject>
#include <QtQml/qqmlregistration.h>

class Keyboard : public QObject
{
    Q_OBJECT
    QML_ELEMENT
    QML_SINGLETON
    Q_PROPERTY(bool ctrl READ ctrl NOTIFY ctrlChanged)
    Q_PROPERTY(int tab READ tab NOTIFY tabChanged)

public:
    explicit Keyboard(QObject *parent = nullptr);
    bool ctrl() const { return m_ctrl; }
    int tab() const { return m_tab; }

signals:
    void ctrlChanged();
    void tabChanged();

protected:
    bool eventFilter(QObject *watched, QEvent *event) override;

private:
    void set(bool ctrl);
    void setTab(int tab);
    bool m_ctrl = false;
    int m_tab = 0;
};
