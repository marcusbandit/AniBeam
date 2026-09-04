#include "helpers.h"
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
