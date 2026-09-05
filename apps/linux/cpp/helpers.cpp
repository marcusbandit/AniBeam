#include "helpers.h"
#include <QtCore/QByteArray>
#include <QtCore/qglobal.h>
#include <QtGui/QGuiApplication>
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
