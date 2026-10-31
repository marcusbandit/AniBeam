#include "devtools.h"

#include <atomic>
#include <chrono>
#include <cmath>

#include <QtCore/QByteArray>
#include <QtCore/QFileInfo>
#include <QtCore/QHash>
#include <QtCore/QJsonArray>
#include <QtCore/QJsonDocument>
#include <QtCore/QJsonObject>
#include <QtCore/QMutex>
#include <QtCore/QMutexLocker>
#include <QtCore/QThread>
#include <QtCore/QUrl>
#include <QtCore/QVector>
#include <QtGui/QImage>
#include <QtGui/QImageReader>
#include <QtQml/QQmlAbstractUrlInterceptor>
#include <QtQml/QQmlApplicationEngine>
#include <QtQml/QQmlEngine>
#include <QtQml/QQmlDebuggingEnabler>
#include <QtQuick/QQuickImageProvider>
#include <QtQuick/QQuickItem>
#include <QtQuick/QQuickWindow>

#ifdef __GLIBC__
#include <malloc.h>
#endif

namespace {

const auto g_start = std::chrono::steady_clock::now();

std::atomic<bool> g_measure{false};
std::atomic<bool> g_images{true};
std::atomic<qint64> g_requests{0};
std::atomic<qint64> g_blocked{0};
std::atomic<int> g_pending{0};

struct Decode {
    double at;
    double ms;
    QString file;
    qint64 bytes;
    int srcW, srcH, w, h;
    int thread;
};

// The records and the running totals. The totals count every decode since the start; the
// list keeps the newest few thousand, which is a long session of page loads.
QMutex g_lock;
QVector<Decode> g_decodes;
qint64 g_decodeCount = 0;
qint64 g_decodedBytes = 0;
qint64 g_fileBytes = 0;
double g_decodeMs = 0;
QHash<Qt::HANDLE, int> g_threads;
constexpr int KEEP = 4000;
// The slot the next decode goes in; the oldest record once the ring is full
int g_next = 0;

bool isImagePath(const QString &path)
{
    static const char *exts[] = {".jpg", ".jpeg", ".png", ".webp", ".gif", ".avif"};
    for (const char *e : exts)
        if (path.endsWith(QLatin1String(e), Qt::CaseInsensitive))
            return true;
    return false;
}

// What Image itself asks the decoder for: the requested size with the aspect kept, covering
// both sides when both are given (every poster here crops), never larger than the file.
QSize loadSize(const QSize &src, const QSize &req)
{
    if (!src.isValid() || (req.width() <= 0 && req.height() <= 0))
        return src;
    double ratio;
    if (req.width() > 0 && req.height() > 0)
        ratio = std::max(double(req.width()) / src.width(), double(req.height()) / src.height());
    else if (req.width() > 0)
        ratio = double(req.width()) / src.width();
    else
        ratio = double(req.height()) / src.height();
    if (ratio >= 1.0)
        return src;
    return QSize(std::max(1, int(std::round(src.width() * ratio))), std::max(1, int(std::round(src.height() * ratio))));
}

class DevImageProvider : public QQuickImageProvider
{
public:
    DevImageProvider() : QQuickImageProvider(QQuickImageProvider::Image, QQuickImageProvider::ForceAsynchronousImageLoading) {}

    QImage requestImage(const QString &id, QSize *size, const QSize &requested) override
    {
        // Images off: one transparent pixel, the same url for every image, so the engine
        // decodes it once and every later image is a cache hit. An empty url would do
        // nothing at all, but each Image would log "Protocol is unknown", and the logging
        // costs more than the pixel.
        if (id == QLatin1String("off")) {
            if (size)
                *size = QSize(1, 1);
            QImage blank(1, 1, QImage::Format_ARGB32_Premultiplied);
            blank.fill(Qt::transparent);
            return blank;
        }
        const double at = dev_now_ms();
        g_pending.fetch_add(1);
        const QString path = QString::fromUtf8(QByteArray::fromBase64(id.toLatin1(), QByteArray::Base64UrlEncoding | QByteArray::OmitTrailingEquals));
        QImageReader reader(path);
        reader.setAutoTransform(true);
        const QSize src = reader.size();
        const QSize out = loadSize(src, requested);
        if (out.isValid() && out != src)
            reader.setScaledSize(out);
        const QImage image = reader.read();
        const double ms = dev_now_ms() - at;
        if (size)
            *size = src;
        // The file's size and name before the lock, so decoder threads never queue behind
        // each other's stat calls
        const QFileInfo info(path);
        const qint64 bytes = info.size();
        const QString name = info.fileName();
        {
            QMutexLocker held(&g_lock);
            const Qt::HANDLE t = QThread::currentThreadId();
            if (!g_threads.contains(t))
                g_threads.insert(t, g_threads.size() + 1);
            const Decode d{at, ms, name, bytes, src.width(), src.height(), image.width(), image.height(), g_threads.value(t)};
            // A ring: once full, the oldest slot is overwritten instead of shifting the rest
            if (g_decodes.size() < KEEP)
                g_decodes.push_back(d);
            else
                g_decodes[g_next] = d;
            g_next = (g_next + 1) % KEEP;
            g_decodeCount += 1;
            g_decodedBytes += image.sizeInBytes();
            g_fileBytes += bytes;
            g_decodeMs += ms;
        }
        g_pending.fetch_sub(1);
        return image;
    }
};

class DevImageInterceptor : public QQmlAbstractUrlInterceptor
{
public:
    QUrl intercept(const QUrl &url, DataType type) override
    {
        if (type != UrlString)
            return url;
        const bool measure = g_measure.load(std::memory_order_relaxed);
        const bool images = g_images.load(std::memory_order_relaxed);
        if (!measure && images)
            return url;
        const QString scheme = url.scheme();
        const bool local = scheme == QLatin1String("file");
        if (!local && scheme != QLatin1String("http") && scheme != QLatin1String("https"))
            return url;
        if (!isImagePath(url.path()))
            return url;
        g_requests.fetch_add(1);
        if (!images) {
            g_blocked.fetch_add(1);
            return QUrl(QStringLiteral("image://dev/off"));
        }
        if (local) {
            const QByteArray id = url.toLocalFile().toUtf8().toBase64(QByteArray::Base64UrlEncoding | QByteArray::OmitTrailingEquals);
            return QUrl(QStringLiteral("image://dev/") + QString::fromLatin1(id));
        }
        return url;
    }
};

struct Scene {
    int items = 0, visible = 0, offscreen = 0, texts = 0, shapes = 0, loaders = 0;
    int images = 0, ready = 0, loading = 0, error = 0, none = 0, icons = 0, imagesOffscreen = 0;
    double pixels = 0, offscreenPixels = 0;
    QHash<QString, int> types;
};

QString typeName(const QMetaObject *mo)
{
    // A QML type's class reads "Card_QMLTYPE_12"; the name before the suffix is the file.
    QString name = QString::fromLatin1(mo->className());
    const int cut = name.indexOf(QLatin1String("_QML"));
    if (cut > 0)
        name.truncate(cut);
    if (name.startsWith(QLatin1String("QQuick")))
        name = name.mid(6);
    return name;
}

void walk(QQuickItem *item, const QRectF &window, double dpr2, Scene &s)
{
    const auto children = item->childItems();
    for (QQuickItem *c : children) {
        s.items += 1;
        const bool visible = c->isVisible();
        const QRectF r = c->mapRectToScene(QRectF(0, 0, c->width(), c->height()));
        const bool onScreen = visible && r.intersects(window);
        if (visible)
            s.visible += 1;
        if (visible && !onScreen && r.width() > 0 && r.height() > 0)
            s.offscreen += 1;
        s.types[typeName(c->metaObject())] += 1;
        if (c->inherits("QQuickText"))
            s.texts += 1;
        if (c->inherits("QQuickShape"))
            s.shapes += 1;
        if (c->inherits("QQuickLoader"))
            s.loaders += 1;
        if (c->inherits("QQuickImageBase")) {
            const QUrl source = c->property("source").toUrl();
            const QString scheme = source.scheme();
            // A poster Image is often hidden and drawn through a Shape that uses it as a
            // texture, so whether it is on screen is its parent's visibility and its own
            // rectangle, not its own visibility.
            QQuickItem *holder = c->parentItem();
            const bool imageOnScreen = (holder ? holder->isVisible() : true) && r.intersects(window);
            if (scheme == QLatin1String("qrc") || source.toString() == QLatin1String("image://dev/off")) {
                s.icons += 1;
            } else {
                s.images += 1;
                const int status = c->property("status").toInt();
                if (status == 1) {
                    s.ready += 1;
                    const double px = c->implicitWidth() * c->implicitHeight() * dpr2;
                    s.pixels += px;
                    if (!imageOnScreen) {
                        s.imagesOffscreen += 1;
                        s.offscreenPixels += px;
                    }
                } else if (status == 2) {
                    s.loading += 1;
                } else if (status == 3) {
                    s.error += 1;
                } else {
                    s.none += 1;
                }
            }
        }
        walk(c, window, dpr2, s);
    }
}

}

void dev_enable_qml_profiling()
{
    if (qgetenv("ANIBEAM_QML_PROFILE") == "1")
        QQmlDebuggingEnabler::enableDebugging(true);
}

void install_dev_tools(QQmlApplicationEngine &engine)
{
    (void)dev_now_ms();
    engine.addImageProvider(QStringLiteral("dev"), new DevImageProvider);
    engine.addUrlInterceptor(new DevImageInterceptor);
}

void dev_set_image_flags(bool measure, bool images)
{
    g_measure.store(measure);
    g_images.store(images);
}

double dev_now_ms()
{
    return std::chrono::duration<double, std::milli>(std::chrono::steady_clock::now() - g_start).count();
}

QString dev_image_stats(double since)
{
    QJsonObject o;
    QJsonArray list;
    QMutexLocker held(&g_lock);
    o.insert(QStringLiteral("requests"), double(g_requests.load()));
    o.insert(QStringLiteral("blocked"), double(g_blocked.load()));
    o.insert(QStringLiteral("pending"), g_pending.load());
    o.insert(QStringLiteral("decodes"), double(g_decodeCount));
    o.insert(QStringLiteral("decodedBytes"), double(g_decodedBytes));
    o.insert(QStringLiteral("fileBytes"), double(g_fileBytes));
    o.insert(QStringLiteral("decodeMs"), g_decodeMs);
    // Oldest first: from the slot after the newest once the ring has wrapped
    const int n = g_decodes.size();
    const int oldest = n < KEEP ? 0 : g_next;
    auto at = [&](int k) -> const Decode & { return g_decodes[(oldest + k) % n]; };
    int first = n;
    while (first > 0 && at(first - 1).at >= since)
        first -= 1;
    first = std::max(first, n - 400);
    for (int k = first; k < n; ++k) {
        const Decode &d = at(k);
        list.append(QJsonObject{{QStringLiteral("at"), d.at},
                                {QStringLiteral("ms"), d.ms},
                                {QStringLiteral("file"), d.file},
                                {QStringLiteral("bytes"), double(d.bytes)},
                                {QStringLiteral("srcW"), d.srcW},
                                {QStringLiteral("srcH"), d.srcH},
                                {QStringLiteral("w"), d.w},
                                {QStringLiteral("h"), d.h},
                                {QStringLiteral("thread"), d.thread}});
    }
    o.insert(QStringLiteral("list"), list);
    return QString::fromUtf8(QJsonDocument(o).toJson(QJsonDocument::Compact));
}

QString dev_scene(QObject *item, QObject *window)
{
    auto *root = qobject_cast<QQuickItem *>(item);
    auto *w = qobject_cast<QQuickWindow *>(window);
    if (!root)
        return QStringLiteral("{}");
    Scene s;
    const double dpr = w ? w->effectiveDevicePixelRatio() : 1.0;
    const QRectF area = w ? QRectF(0, 0, w->width(), w->height()) : QRectF(0, 0, 1e9, 1e9);
    walk(root, area, dpr * dpr, s);
    QVector<QPair<QString, int>> types;
    for (auto it = s.types.cbegin(); it != s.types.cend(); ++it)
        types.push_back({it.key(), it.value()});
    std::sort(types.begin(), types.end(), [](const auto &a, const auto &b) { return a.second > b.second; });
    QJsonArray top;
    for (int i = 0; i < types.size() && i < 16; ++i)
        top.append(QJsonArray{types[i].first, types[i].second});
    QJsonObject images{{QStringLiteral("total"), s.images},
                       {QStringLiteral("ready"), s.ready},
                       {QStringLiteral("loading"), s.loading},
                       {QStringLiteral("error"), s.error},
                       {QStringLiteral("none"), s.none},
                       {QStringLiteral("offscreen"), s.imagesOffscreen},
                       {QStringLiteral("pixels"), s.pixels},
                       {QStringLiteral("offscreenPixels"), s.offscreenPixels}};
    QJsonObject o{{QStringLiteral("items"), s.items},
                  {QStringLiteral("visible"), s.visible},
                  {QStringLiteral("offscreen"), s.offscreen},
                  {QStringLiteral("texts"), s.texts},
                  {QStringLiteral("shapes"), s.shapes},
                  {QStringLiteral("loaders"), s.loaders},
                  {QStringLiteral("icons"), s.icons},
                  {QStringLiteral("images"), images},
                  {QStringLiteral("types"), top}};
    return QString::fromUtf8(QJsonDocument(o).toJson(QJsonDocument::Compact));
}

void dev_release(QObject *window)
{
    if (auto *w = qobject_cast<QQuickWindow *>(window)) {
        w->releaseResources();
        if (QQmlEngine *e = qmlEngine(w))
            e->collectGarbage();
    }
#ifdef __GLIBC__
    malloc_trim(0);
#endif
}
