#include "mpvitem.h"
#include <MpvQt/MpvController>
#include <QJsonDocument>
#include <QJsonArray>
#include <QTimer>
#include <QTextStream>
#include <QDir>

SpikeConfig &spikeConfig() { static SpikeConfig c; return c; }

static QJsonValue jv(const QVariant &v) { return QJsonValue::fromVariant(v); }

MpvItem::MpvItem(QQuickItem *parent) : MpvAbstractItem(parent)
{
    m_clock.start();
    connect(this, &MpvAbstractItem::ready, this, &MpvItem::onReady);
    connect(mpvController(), &MpvController::fileLoaded, this, &MpvItem::onFileLoaded, Qt::QueuedConnection);
    connect(mpvController(), &MpvController::propertyChanged, this, &MpvItem::onPropertyChanged, Qt::QueuedConnection);
    connect(mpvController(), &MpvController::endFile, this, [this](const QString &r) { log(QStringLiteral("end-file"), {{"reason", r}}); }, Qt::QueuedConnection);
    connect(mpvController(), &MpvController::videoReconfig, this, [this] { log(QStringLiteral("video-reconfig"), {}); }, Qt::QueuedConnection);
}

void MpvItem::setPreviewMode(bool on) { if (m_preview != on) { m_preview = on; Q_EMIT previewModeChanged(); } }

void MpvItem::log(const QString &tag, const QJsonObject &o)
{
    QJsonObject out = o;
    out.insert(QStringLiteral("t_ms"), m_clock.elapsed());
    out.insert(QStringLiteral("who"), m_preview ? QStringLiteral("preview") : QStringLiteral("player"));
    QTextStream ts(stdout);
    ts << "SPIKE " << tag << ' ' << QString::fromUtf8(QJsonDocument(out).toJson(QJsonDocument::Compact)) << '\n';
    ts.flush();
}

void MpvItem::at(int ms, std::function<void()> f) { QTimer::singleShot(ms, this, std::move(f)); }

void MpvItem::onReady()
{
    const auto &cfg = spikeConfig();
    const QString who = m_preview ? QStringLiteral("preview") : QStringLiteral("player");
    setProperty(QStringLiteral("log-file"), QDir(cfg.outDir).filePath(QStringLiteral("mpv-%1.log").arg(who)));
    setProperty(QStringLiteral("hwdec"), cfg.hwdec);
    setProperty(QStringLiteral("keep-open"), QStringLiteral("yes"));
    setProperty(QStringLiteral("mute"), QStringLiteral("yes"));
    setProperty(QStringLiteral("osd-level"), 0);
    setProperty(QStringLiteral("screenshot-format"), QStringLiteral("png"));
    setProperty(QStringLiteral("hr-seek"), QStringLiteral("yes"));
    if (m_preview) {
        setProperty(QStringLiteral("pause"), true);
        setProperty(QStringLiteral("aid"), QStringLiteral("no"));
        setProperty(QStringLiteral("sid"), QStringLiteral("no"));
        setProperty(QStringLiteral("sub-auto"), QStringLiteral("no"));
        setProperty(QStringLiteral("audio-file-auto"), QStringLiteral("no"));
    }
    observeProperty(QStringLiteral("time-pos"), MPV_FORMAT_DOUBLE);
    observeProperty(QStringLiteral("pause"), MPV_FORMAT_FLAG);
    observeProperty(QStringLiteral("hwdec-current"), MPV_FORMAT_STRING);
    observeProperty(QStringLiteral("frame-drop-count"), MPV_FORMAT_INT64);
    observeProperty(QStringLiteral("decoder-frame-drop-count"), MPV_FORMAT_INT64);
    observeProperty(QStringLiteral("mistimed-frame-count"), MPV_FORMAT_INT64);
    observeProperty(QStringLiteral("chapter"), MPV_FORMAT_INT64);
    observeProperty(QStringLiteral("seeking"), MPV_FORMAT_FLAG);
    observeProperty(QStringLiteral("vo-configured"), MPV_FORMAT_FLAG);
    log(QStringLiteral("ready"), {{"hwdec", cfg.hwdec}, {"file", cfg.file}});
    command({QStringLiteral("loadfile"), cfg.file});
}

void MpvItem::onFileLoaded()
{
    QJsonObject o;
    o.insert(QStringLiteral("track-list"), jv(getProperty(QStringLiteral("track-list"))));
    o.insert(QStringLiteral("chapter-list"), jv(getProperty(QStringLiteral("chapter-list"))));
    o.insert(QStringLiteral("video-params"), jv(getProperty(QStringLiteral("video-params"))));
    o.insert(QStringLiteral("video-codec"), jv(getProperty(QStringLiteral("video-codec"))));
    o.insert(QStringLiteral("container-fps"), jv(getProperty(QStringLiteral("container-fps"))));
    o.insert(QStringLiteral("duration"), jv(getProperty(QStringLiteral("duration"))));
    log(QStringLiteral("file-loaded"), o);
    if (m_preview) {
        at(50000, [this] { previewSeek(300); });
        at(54000, [this] { previewSeek(900); });
        at(58000, [this] { previewSeek(1200); });
        at(62000, [this] { previewSeek(600); });
        at(65000, [this] { report(QStringLiteral("preview-final")); });
        return;
    }
    if (spikeConfig().script) runScript();
}

void MpvItem::previewSeek(double t)
{
    m_seekTarget = t;
    m_seekClock.start();
    setProperty(QStringLiteral("time-pos"), t);
}

void MpvItem::onPropertyChanged(const QString &name, const QVariant &value)
{
    if (name == QLatin1String("time-pos")) { m_timePos = value.toDouble(); if (m_paused && !m_preview) log(QStringLiteral("time-pos-paused"), {{"value", m_timePos}}); refreshStatus(); return; }
    if (name == QLatin1String("pause")) { m_paused = value.toBool(); if (!m_preview) log(QStringLiteral("pause"), {{"value", m_paused}, {"time-pos", m_timePos}}); refreshStatus(); return; }
    if (name == QLatin1String("hwdec-current")) { m_hwdecCurrent = value.toString(); log(QStringLiteral("hwdec-current"), {{"value", m_hwdecCurrent}}); refreshStatus(); return; }
    if (name == QLatin1String("frame-drop-count")) { m_drops = value.toLongLong(); log(QStringLiteral("frame-drop-count"), {{"value", double(m_drops)}, {"time-pos", m_timePos}}); refreshStatus(); return; }
    if (name == QLatin1String("decoder-frame-drop-count")) { log(QStringLiteral("decoder-frame-drop-count"), {{"value", jv(value)}, {"time-pos", m_timePos}}); return; }
    if (name == QLatin1String("vo-drop-frame-count")) { m_voDrops = value.toLongLong(); refreshStatus(); return; }
    if (name == QLatin1String("mistimed-frame-count")) { m_mistimed = value.toLongLong(); refreshStatus(); return; }
    if (name == QLatin1String("chapter")) { log(QStringLiteral("chapter"), {{"value", jv(value)}}); return; }
    if (name == QLatin1String("vo-configured")) { log(QStringLiteral("vo-configured"), {{"value", jv(value)}}); return; }
    if (name == QLatin1String("seeking")) {
        if (!value.toBool() && m_seekTarget >= 0) {
            log(QStringLiteral("seek-done"), {{"target", m_seekTarget}, {"latency_ms", double(m_seekClock.elapsed())}, {"time-pos", jv(getProperty(QStringLiteral("time-pos")))}});
            m_seekTarget = -1;
        }
        return;
    }
}

void MpvItem::refreshStatus()
{
    m_status = QStringLiteral("%1  t=%2  %3  hwdec=%4  drops=%5 vo-drops=%6 mistimed=%7")
        .arg(m_preview ? QStringLiteral("preview") : QStringLiteral("player"))
        .arg(m_timePos, 0, 'f', 3).arg(m_paused ? QStringLiteral("paused") : QStringLiteral("playing"))
        .arg(m_hwdecCurrent).arg(m_drops).arg(m_voDrops).arg(m_mistimed);
    Q_EMIT statusLineChanged();
}

void MpvItem::report(const QString &tag)
{
    static const char *props[] = {
        "time-pos", "pause", "hwdec-current", "hwdec-interop", "video-codec", "video-params/pixelformat", "video-params/hw-pixelformat",
        "video-params/w", "video-params/h", "container-fps", "estimated-vf-fps", "display-fps", "estimated-display-fps", "vsync-ratio", "vsync-jitter",
        "frame-drop-count", "decoder-frame-drop-count", "mistimed-frame-count", "vo-delayed-frame-count", "chapter", "current-vo", "sid", "aid",
        "sub-text", "estimated-frame-number", "video-sync", "gpu-context", "gpu-api", "vo-passes/fresh/count", "avsync"
    };
    QJsonObject o;
    QElapsedTimer et; et.start();
    for (const char *p : props) o.insert(QString::fromLatin1(p), jv(getProperty(QString::fromLatin1(p))));
    o.insert(QStringLiteral("report_ms"), double(et.nsecsElapsed()) / 1e6);
    o.insert(QStringLiteral("props"), int(std::size(props)));
    log(tag, o);
}

void MpvItem::frameStep(int dir)
{
    command({dir > 0 ? QStringLiteral("frame-step") : QStringLiteral("frame-back-step")});
}

void MpvItem::togglePause() { setProperty(QStringLiteral("pause"), !m_paused); }
void MpvItem::toggleMute() { setProperty(QStringLiteral("mute"), !getProperty(QStringLiteral("mute")).toBool()); }

void MpvItem::shot(const QString &name)
{
    const QString path = QDir(spikeConfig().outDir).filePath(QStringLiteral("shot-%1.png").arg(name));
    commandAsync({QStringLiteral("screenshot-to-file"), path, QStringLiteral("subtitles")});
    log(QStringLiteral("shot"), {{"path", path}});
}

void MpvItem::runScript()
{
    // Phase A: undisturbed playback from the OP. Observed properties log themselves.
    at(12000, [this] { shot(QStringLiteral("op")); });
    at(20000, [this] { setProperty(QStringLiteral("chapter"), 1); log(QStringLiteral("set-chapter"), {{"chapter", 1}}); });
    at(24000, [this] { shot(QStringLiteral("part-a")); });
    at(26000, [this] { setProperty(QStringLiteral("pause"), true); log(QStringLiteral("set-pause"), {{"pause", true}}); });
    for (int i = 0; i < 5; ++i)
        at(27000 + i * 1000, [this, i] { log(QStringLiteral("cmd-frame-step"), {{"n", i + 1}}); frameStep(+1); });
    for (int i = 0; i < 5; ++i)
        at(32000 + i * 1000, [this, i] { log(QStringLiteral("cmd-frame-back-step"), {{"n", i + 1}}); frameStep(-1); });
    at(37500, [this] { shot(QStringLiteral("after-steps")); });
    at(38500, [this] { setProperty(QStringLiteral("pause"), false); log(QStringLiteral("set-pause"), {{"pause", false}}); });
    at(41000, [this] { Q_EMIT fullscreenRequested(true); log(QStringLiteral("fullscreen"), {{"on", true}}); });
    at(45000, [this] { shot(QStringLiteral("fullscreen")); });
    at(46500, [this] { Q_EMIT fullscreenRequested(false); log(QStringLiteral("fullscreen"), {{"on", false}}); });
    at(66000, [this] { report(QStringLiteral("final")); shot(QStringLiteral("final")); });
    at(67500, [this] { Q_EMIT quitRequested(); });
}
