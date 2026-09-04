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
    QCommandLineOption set(QStringLiteral("set"), QStringLiteral("mpv option as key=value, repeatable"), QStringLiteral("key=value"));
    QCommandLineOption play(QStringLiteral("play"), QStringLiteral("run the quality sequence: N seconds of undisturbed playback then stills"), QStringLiteral("seconds"), QStringLiteral("60"));
    QCommandLineOption startAt(QStringLiteral("start"), QStringLiteral("start position in seconds"), QStringLiteral("seconds"), QStringLiteral("0"));
    QCommandLineOption stills(QStringLiteral("stills"), QStringLiteral("comma separated timestamps to hold as stills"), QStringLiteral("list"), QString());
    QCommandLineOption full(QStringLiteral("fullscreen"), QStringLiteral("start fullscreen"));
    p.addOptions({hwdec, out, script, preview, set, play, startAt, stills, full});
    p.process(app);
    auto &cfg = spikeConfig();
    cfg.file = p.positionalArguments().value(0);
    cfg.hwdec = p.value(hwdec);
    cfg.outDir = p.value(out);
    cfg.script = p.isSet(script);
    cfg.preview = p.isSet(preview);
    cfg.sets = p.values(set);
    cfg.quality = p.isSet(play);
    cfg.playSeconds = p.value(play).toInt();
    cfg.startAt = p.value(startAt).toDouble();
    cfg.fullscreen = p.isSet(full);
    for (const QString &s : p.value(stills).split(QLatin1Char(','), Qt::SkipEmptyParts))
        cfg.stills.append(s.toDouble());
    QDir().mkpath(cfg.outDir);

    QQmlApplicationEngine engine;
    engine.rootContext()->setContextProperty(QStringLiteral("spikePreview"), cfg.preview);
    engine.rootContext()->setContextProperty(QStringLiteral("spikeQuality"), cfg.quality);
    engine.rootContext()->setContextProperty(QStringLiteral("spikeFullscreen"), cfg.fullscreen);
    QObject::connect(&engine, &QQmlApplicationEngine::objectCreationFailed, &app, [] { QCoreApplication::exit(1); }, Qt::QueuedConnection);
    engine.loadFromModule("Spike", "Main");
    return app.exec();
}
