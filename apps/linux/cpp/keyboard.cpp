#include "keyboard.h"

#include <QGuiApplication>
#include <QInputEvent>

Keyboard::Keyboard(QObject *parent) : QObject(parent)
{
    qApp->installEventFilter(this);
}

bool Keyboard::eventFilter(QObject *watched, QEvent *event)
{
    switch (event->type()) {
    case QEvent::KeyPress:
    case QEvent::KeyRelease:
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
