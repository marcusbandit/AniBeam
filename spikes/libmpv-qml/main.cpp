#include "mpvitem.h"
#include <QGuiApplication>
#include <QQmlApplicationEngine>
#include <QQmlContext>
#include <QQuickWindow>
#include <QCommandLineParser>
#include <QDir>
#include <QTextStream>

int main(int argc, char *argv[])
{
    QQuickWindow::setGraphicsApi(QSGRendererInterface::OpenGL);
    QGuiApplication app(argc, argv);
    app.setApplicationName(QStringLiteral("mpvspike"));
    app.setDesktopFileName(QStringLiteral("mpvspike"));

    QCommandLineParser p;
    p.addPositionalArgument(QStringLiteral("file"), QStringLiteral("video file"));
    QCommandLineOption hwdec(QStringLiteral("hwdec"), QStringLiteral("mpv hwdec value"), QStringLiteral("value"), QStringLiteral("auto"));
    QCommandLineOption out(QStringLiteral("out"), QStringLiteral("output dir for logs and shots"), QStringLiteral("dir"), QDir::currentPath());
    QCommandLineOption script(QStringLiteral("script"), QStringLiteral("run the scripted sequence then quit"));
    QCommandLineOption preview(QStringLiteral("preview"), QStringLiteral("also run a second item as a seek preview"));
    p.addOptions({hwdec, out, script, preview});
    p.process(app);
    auto &cfg = spikeConfig();
    cfg.file = p.positionalArguments().value(0);
    cfg.hwdec = p.value(hwdec);
    cfg.outDir = p.value(out);
    cfg.script = p.isSet(script);
    cfg.preview = p.isSet(preview);
    QDir().mkpath(cfg.outDir);

    QQmlApplicationEngine engine;
    engine.rootContext()->setContextProperty(QStringLiteral("spikePreview"), cfg.preview);
    QObject::connect(&engine, &QQmlApplicationEngine::objectCreationFailed, &app, [] { QCoreApplication::exit(1); }, Qt::QueuedConnection);
    engine.loadFromModule("Spike", "Main");
    return app.exec();
}
