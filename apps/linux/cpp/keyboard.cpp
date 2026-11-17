#include "keyboard.h"

#include <QGuiApplication>
#include <QKeyEvent>
#include <QMetaObject>

Keyboard::Keyboard(QObject *parent) : QObject(parent)
{
    qApp->installEventFilter(this);
}

bool Keyboard::eventFilter(QObject *watched, QEvent *event)
{
    switch (event->type()) {
    // Ctrl's own press and release carry the state from before it; the key says it plainly
    case QEvent::KeyPress:
    case QEvent::KeyRelease: {
        auto *key = static_cast<QKeyEvent *>(event);
        // Qt Quick moves focus on Tab only without Ctrl or Alt. The press is still being
        // delivered here, so the reset waits until the event loop is back
        if (event->type() == QEvent::KeyPress && !(key->modifiers() & (Qt::ControlModifier | Qt::AltModifier))
            && (key->key() == Qt::Key_Tab || key->key() == Qt::Key_Backtab)) {
            setTab(key->key() == Qt::Key_Backtab || (key->modifiers() & Qt::ShiftModifier) ? -1 : 1);
            QMetaObject::invokeMethod(this, [this] { setTab(0); }, Qt::QueuedConnection);
        }
        if (key->key() == Qt::Key_Control)
            set(event->type() == QEvent::KeyPress);
        else
            set(key->modifiers() & Qt::ControlModifier);
        break;
    }
    case QEvent::MouseMove:
    case QEvent::HoverMove:
    case QEvent::MouseButtonPress:
    case QEvent::Wheel:
        set(static_cast<QInputEvent *>(event)->modifiers() & Qt::ControlModifier);
        break;
    // A Ctrl let go while another window had the keys never reaches this one
    case QEvent::ApplicationStateChange:
    case QEvent::FocusOut:
        set(QGuiApplication::queryKeyboardModifiers() & Qt::ControlModifier);
        break;
    default:
        break;
    }
    return QObject::eventFilter(watched, event);
}

void Keyboard::set(bool ctrl)
{
    if (ctrl == m_ctrl)
        return;
    m_ctrl = ctrl;
    emit ctrlChanged();
}

void Keyboard::setTab(int tab)
{
    if (tab == m_tab)
        return;
    m_tab = tab;
    emit tabChanged();
}
