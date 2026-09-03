#pragma once
#include <MpvQt/MpvAbstractItem>
#include <QtQml/qqmlregistration.h>
#include <QJsonObject>
#include <QElapsedTimer>

struct SpikeConfig {
    QString file;
    QString hwdec = QStringLiteral("auto");
    QString outDir;
    bool script = false;
    bool preview = false;
};
SpikeConfig &spikeConfig();

class MpvItem : public MpvAbstractItem
{
    Q_OBJECT
    QML_ELEMENT
    Q_PROPERTY(bool previewMode READ previewMode WRITE setPreviewMode NOTIFY previewModeChanged)
    Q_PROPERTY(QString statusLine READ statusLine NOTIFY statusLineChanged)
public:
    explicit MpvItem(QQuickItem *parent = nullptr);
    bool previewMode() const { return m_preview; }
    void setPreviewMode(bool on);
    QString statusLine() const { return m_status; }

    Q_INVOKABLE void report(const QString &tag);
    Q_INVOKABLE void frameStep(int dir);
    Q_INVOKABLE void togglePause();
    Q_INVOKABLE void toggleMute();
    Q_INVOKABLE void shot(const QString &name);

Q_SIGNALS:
    void previewModeChanged();
    void statusLineChanged();
    void fullscreenRequested(bool on);
    void quitRequested();

private:
    void onReady();
    void onFileLoaded();
    void onPropertyChanged(const QString &name, const QVariant &value);
    void runScript();
    void log(const QString &tag, const QJsonObject &o);
    void at(int ms, std::function<void()> f);
    void previewSeek(double t);
    void refreshStatus();

    bool m_preview = false;
    QString m_status;
    QElapsedTimer m_clock;
    QElapsedTimer m_seekClock;
    double m_seekTarget = -1;
    QString m_hwdecCurrent;
    double m_timePos = 0;
    bool m_paused = false;
    qint64 m_drops = 0, m_voDrops = 0, m_mistimed = 0;
};
