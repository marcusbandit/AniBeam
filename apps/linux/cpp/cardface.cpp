#include "cardface.h"

#include <algorithm>
#include <array>
#include <cmath>
#include <functional>

#include <QtGui/QFontMetricsF>
#include <QtGui/QTextOption>
#include <QtQuick/QQuickWindow>
#include <QtQuick/QSGFlatColorMaterial>
#include <QtQuick/QSGGeometryNode>
#include <QtQuick/QSGTextNode>
#include <QtQuick/QSGVertexColorMaterial>

// ---- CardStyle -------------------------------------------------------------------------

namespace {
CardStyle *g_style = nullptr;

QColor colorOf(const QVariant &v, const QColor &fallback = QColor())
{
    if (!v.isValid())
        return fallback;
    QColor c = v.value<QColor>();
    if (!c.isValid() && v.canConvert<QString>())
        c = QColor::fromString(v.toString());
    return c.isValid() ? c : fallback;
}

// One cut of the face, as Tokens' roles define them: the width and weight axes, the
// weight, and tabular figures for the ledger
QFont roleFont(const QString &family, qreal size, int wdth, int wght, bool tabular)
{
    QFont f(family);
    f.setPointSizeF(size);
    f.setWeight(QFont::Weight(wght));
    f.setVariableAxis(QFont::Tag("wdth"), wdth);
    f.setVariableAxis(QFont::Tag("wght"), wght);
    if (tabular)
        f.setFeature(QFont::Tag("tnum"), 1);
    return f;
}
}

CardStyle::CardStyle(QObject *parent) : QObject(parent) { g_style = this; }
CardStyle::~CardStyle()
{
    if (g_style == this)
        g_style = nullptr;
}
CardStyle *CardStyle::instance() { return g_style; }

void CardStyle::set(const QVariantMap &t)
{
    Data d;
    const QString family = t.value(QStringLiteral("fontFamily")).toString();
    const qreal small = t.value(QStringLiteral("typeSmall"), 9.0).toReal();
    const qreal normal = t.value(QStringLiteral("typeNormal"), 10.5).toReal();
    const qreal large = t.value(QStringLiteral("typeLarge"), 15.0).toReal();
    // Tokens.roles: display 68/330, title 86/600, ui 100/450, ledger 88/500
    d.title = roleFont(family, normal, 86, 600, false);
    d.heroTitle = roleFont(family, large * 1.2, 68, 330, false);
    d.ledger = roleFont(family, small, 88, 500, true);
    d.ledgerStrong = roleFont(family, small, 88, 650, true);
    d.heroScore = roleFont(family, normal, 88, 500, true);
    d.ui = roleFont(family, small, 100, 450, false);
    d.tag = d.ledger;
    d.heroTag = roleFont(family, normal * 0.92, 88, 500, true);
    d.text = colorOf(t.value(QStringLiteral("text")), Qt::white);
    d.textDim = colorOf(t.value(QStringLiteral("textDim")), QColor(170, 170, 170));
    d.textFaint = colorOf(t.value(QStringLiteral("textFaint")), QColor(120, 120, 120));
    d.accent = colorOf(t.value(QStringLiteral("accent")), QColor(80, 200, 200));
    d.focusRing = colorOf(t.value(QStringLiteral("focusRing")), d.accent);
    d.onPoster = colorOf(t.value(QStringLiteral("onPoster")), QColor(0xee, 0xf2, 0xf5));
    d.onPosterDim = colorOf(t.value(QStringLiteral("onPosterDim")), QColor(0xaa, 0xb2, 0xbb));
    d.accentOnPoster = colorOf(t.value(QStringLiteral("accentOnPoster")), d.accent);
    d.posterScrim = colorOf(t.value(QStringLiteral("posterScrim")), QColor(8, 9, 11, 235));
    d.downloadInk = colorOf(t.value(QStringLiteral("downloadInk")), d.onPoster);
    d.unit = t.value(QStringLiteral("space"), 4.0).toReal();
    d.radiusTile = t.value(QStringLiteral("radiusTile"), 18.0).toReal();
    d.radiusSm = t.value(QStringLiteral("radiusSm"), 6.0).toReal();
    d.scrimReach = t.value(QStringLiteral("scrimReach"), 0.55).toReal();
    const QVariantMap cubes = t.value(QStringLiteral("cubes")).toMap();
    for (auto it = cubes.cbegin(); it != cubes.cend(); ++it) {
        if (it.key().isEmpty())
            continue;
        const QVariantMap c = it.value().toMap();
        d.cubes.insert(it.key().at(0), {colorOf(c.value(QStringLiteral("fill")), Qt::transparent),
                                        colorOf(c.value(QStringLiteral("edge")), Qt::transparent)});
    }
    m_data = d;
    emit changed();
}

// ---- CardFace: inputs ------------------------------------------------------------------

namespace {
const CardStyle::Data &style()
{
    static const CardStyle::Data fallback;
    return g_style ? g_style->data() : fallback;
}
bool present(const QVariant &v) { return v.isValid() && !v.isNull(); }
QString count(const QVariant &v) { return QString::number(qint64(std::llround(v.toDouble()))); }
QString plural(qint64 n, const char *one, const char *many)
{
    return QString::number(n) + QLatin1Char(' ') + QLatin1String(n == 1 ? one : many);
}
QString episodeSpan(int first, int through)
{
    return through > first ? QStringLiteral("Episodes %1 to %2").arg(first).arg(through)
                           : QStringLiteral("Episode %1").arg(first);
}
}

CardFace::CardFace(QQuickItem *parent) : QQuickItem(parent)
{
    setFlag(ItemHasContents, true);
}
CardFace::~CardFace() = default;

void CardFace::relayout()
{
    m_dirty = true;
    polish();
}

#define FACE_SETTER(name, member, type, signal) \
    void CardFace::name(type v)                 \
    {                                           \
        if (member == v)                        \
            return;                             \
        member = v;                             \
        emit signal();                          \
        relayout();                             \
    }
FACE_SETTER(setMetaOverride, m_meta, const QString &, metaOverrideChanged)
FACE_SETTER(setGhost, m_ghost, bool, ghostChanged)
FACE_SETTER(setHero, m_hero, bool, heroChanged)
FACE_SETTER(setFocused, m_focused, bool, focusedChanged)
FACE_SETTER(setRightInset, m_rightInset, qreal, rightInsetChanged)
#undef FACE_SETTER

void CardFace::setItem(const QVariantMap &v)
{
    m_item = v;
    emit itemChanged();
    relayout();
}
void CardFace::setCountdown(const QVariant &v)
{
    m_countdown = v;
    emit countdownChanged();
    relayout();
}
void CardFace::setNowMs(double v)
{
    if (m_nowMs == v)
        return;
    m_nowMs = v;
    emit nowMsChanged();
    relayout();
}
void CardFace::setLift(qreal v)
{
    if (qFuzzyCompare(m_lift, v))
        return;
    m_lift = v;
    emit liftChanged();
    relayout();
}
void CardFace::geometryChange(const QRectF &now, const QRectF &before)
{
    QQuickItem::geometryChange(now, before);
    if (now.size() != before.size())
        relayout();
}

qreal CardFace::pad() const { return style().unit * (m_hero ? 5 : 3); }
bool CardFace::bare() const { return m_item.value(QStringLiteral("poster")).toString().isEmpty(); }
QString CardFace::title() const { return m_item.value(QStringLiteral("title")).toString(); }
QString CardFace::folderName() const
{
    return m_item.value(QStringLiteral("titles")).toMap().value(QStringLiteral("folder")).toString();
}

QString CardFace::shortCountdown(double secs)
{
    const qint64 t = std::max<qint64>(0, qint64(std::floor(secs)));
    const qint64 d = t / 86400, h = t % 86400 / 3600, m = t % 3600 / 60;
    if (d > 0)
        return QStringLiteral("%1d %2h").arg(d).arg(h);
    if (h > 0)
        return QStringLiteral("%1h %2m").arg(h).arg(m);
    return QStringLiteral("%1m").arg(std::max<qint64>(1, m));
}

// ---- Layout: what goes where, worked out on the Qt thread ------------------------------

int CardFace::addText(const QString &text, const QFont &font, qreal maxWidth, qreal *width, qreal *height)
{
    const QFontMetricsF fm(font);
    const QString shown = maxWidth > 0 ? fm.elidedText(text, Qt::ElideRight, maxWidth) : text;
    auto layout = std::make_unique<QTextLayout>(shown, font);
    QTextOption opt;
    opt.setWrapMode(QTextOption::NoWrap);
    layout->setTextOption(opt);
    layout->beginLayout();
    QTextLine line = layout->createLine();
    if (line.isValid())
        line.setPosition(QPointF(0, 0));
    layout->endLayout();
    if (width)
        *width = line.isValid() ? line.naturalTextWidth() : 0;
    if (height)
        *height = line.isValid() ? line.height() : fm.height();
    m_layouts.push_back(std::move(layout));
    return int(m_layouts.size()) - 1;
}

// A title wrapped to `maxLines`, its last line elided, lines 0.95 apart as the QML card's
// proportional line height set them
int CardFace::addTitle(const QString &text, const QFont &font, qreal width, int maxLines, qreal *height)
{
    auto lay = [&](const QString &s, int *lines) {
        auto layout = std::make_unique<QTextLayout>(s, font);
        QTextOption opt;
        opt.setWrapMode(QTextOption::WrapAtWordBoundaryOrAnywhere);
        layout->setTextOption(opt);
        layout->beginLayout();
        qreal y = 0;
        int n = 0;
        for (;;) {
            QTextLine line = layout->createLine();
            if (!line.isValid())
                break;
            line.setLineWidth(width);
            line.setPosition(QPointF(0, y));
            y += line.height() * 0.95;
            n += 1;
        }
        layout->endLayout();
        *lines = n;
        return layout;
    };
    int lines = 0;
    auto layout = lay(text, &lines);
    if (lines > maxLines) {
        const int cut = layout->lineAt(maxLines - 1).textStart();
        const QString rest = QFontMetricsF(font).elidedText(text.mid(cut), Qt::ElideRight, width);
        layout = lay(text.left(cut) + rest, &lines);
    }
    qreal h = 0;
    for (int i = 0; i < layout->lineCount() && i < maxLines; ++i)
        h = layout->lineAt(i).y() + layout->lineAt(i).height();
    if (height)
        *height = h;
    m_layouts.push_back(std::move(layout));
    return int(m_layouts.size()) - 1;
}

void CardFace::addTag(const QString &text, const QColor &ink, qreal x, qreal y, bool alignRight, qreal *width)
{
    const auto &st = style();
    const QFont &font = m_hero ? st.heroTag : st.tag;
    qreal tw = 0, th = 0;
    const int at = addText(text, font, 0, &tw, &th);
    const qreal padX = st.unit * (m_hero ? 2.5 : 2), padY = st.unit * (m_hero ? 1 : 0.75);
    const qreal w = std::ceil(tw + padX * 2), h = std::ceil(th + padY * 2);
    const qreal left = alignRight ? x - w : x;
    const QRectF r(left, y, w, h);
    m_fills.push_back({r, st.radiusSm, st.tagFill, {}, {}, -1});
    m_strokes.push_back({r, st.radiusSm, 1, st.tagEdge});
    m_runs.push_back({at, QPointF(left + padX, y + padY), ink});
    if (width)
        *width = w;
}

// Lucide's clock: a circle and two hands, drawn at `size`
void CardFace::addClock(qreal cx, qreal cy, qreal size, const QColor &c)
{
    const qreal k = size / 24.0, sw = std::max<qreal>(1.2, 2 * k);
    const qreal r = 10 * k;
    m_strokes.push_back({QRectF(cx - r - sw / 2, cy - r - sw / 2, 2 * r + sw, 2 * r + sw), r + sw / 2, sw, c});
    const QPointF o(cx - 12 * k, cy - 12 * k);
    m_lines.push_back({o + QPointF(12 * k, 6 * k), o + QPointF(12 * k, 12 * k), sw, c});
    m_lines.push_back({o + QPointF(12 * k, 12 * k), o + QPointF(16 * k, 14 * k), sw, c});
}

void CardFace::updatePolish()
{
    // A face made before Main handed over the tokens follows them from its first layout
    if (!m_styled && g_style) {
        m_styled = true;
        connect(g_style, &CardStyle::changed, this, [this] { relayout(); });
    }
    if (!m_dirty)
        return;
    m_layouts.clear();
    m_fills.clear();
    m_strokes.clear();
    m_overStrokes.clear();
    m_lines.clear();
    m_runs.clear();

    const auto &st = style();
    const qreal w = width(), h = height();
    if (w <= 0 || h <= 0) {
        update();
        return;
    }
    const qreal u = st.unit, p = pad(), radius = st.radiusTile;
    const bool isBare = bare();
    const QColor inkMain = isBare ? st.text : st.onPoster;
    const QColor inkDim = isBare ? st.textDim : st.onPosterDim;
    const QColor inkAccent = isBare ? st.accent : st.accentOnPoster;
    const QVariantMap &it = m_item;
    const QString status = it.value(QStringLiteral("list_status")).toString();
    const bool dropped = status == QLatin1String("Dropped"), onHold = status == QLatin1String("Paused");

    // What the ledger and the countdown say
    const bool hasWatched = present(it.value(QStringLiteral("watched")));
    const double total = it.value(QStringLiteral("total_episodes")).toDouble();
    const bool totalKnown = total > 0;
    const QString watchedText = !hasWatched ? QString()
        : totalKnown ? count(it.value(QStringLiteral("watched"))) + QStringLiteral(" of ") + count(total)
                + (it.value(QStringLiteral("total_is_estimate")).toBool() ? QStringLiteral("+") : QString())
        : count(it.value(QStringLiteral("watched"))) + QStringLiteral(" watched");
    const QString filesText = plural(qint64(it.value(QStringLiteral("episodes_on_disk")).toDouble()), "file", "files");
    const QString ledgerText = !m_meta.isEmpty() ? m_meta : !watchedText.isEmpty() ? watchedText : filesText;
    QVariantMap airing;
    if (present(m_countdown))
        airing = m_countdown.toMap();
    else if (m_meta.isEmpty()) {
        const QVariantMap next = it.value(QStringLiteral("next_airing")).toMap();
        if (!next.isEmpty() && next.value(QStringLiteral("at")).toDouble() * 1000 > m_nowMs)
            airing = next;
    }
    const bool airs = !airing.isEmpty();
    const QString airingTime = airs ? (airing.value(QStringLiteral("predicted")).toBool() ? QStringLiteral("predicted ") : QString())
            + shortCountdown(airing.value(QStringLiteral("at")).toDouble() - m_nowMs / 1000)
                                    : QString();
    const int season = airing.value(QStringLiteral("season")).toInt();
    const QString airingEpisode = airs ? (season && season != 1 ? QStringLiteral("S%1 ").arg(season) : QString())
            + episodeSpan(airing.value(QStringLiteral("episode")).toInt(), airing.value(QStringLiteral("through")).toInt())
                                       : QString();
    const bool mine = present(it.value(QStringLiteral("my_score")));
    const bool hasScore = mine || present(it.value(QStringLiteral("community_score")));
    const QString scoreText = mine ? QString::number(it.value(QStringLiteral("my_score")).toDouble(), 'f', 1)
        : hasScore ? QString::number(it.value(QStringLiteral("community_score")).toDouble(), 'f', 1)
                   : QString();

    // ---- The episode cubes along the foot, one row, folding a long run (EpisodeCubes'
    // rules at maxRows 1)
    const QString codes = it.value(QStringLiteral("cubes")).toString();
    const bool strip = !m_ghost && !codes.isEmpty();
    qreal stripH = 0;
    if (strip) {
        const qreal cubeSize = u * 1.75, minSize = 2, gap = std::max<qreal>(2, std::round(cubeSize * 0.22));
        const qreal sw = w - p * 2;
        const int n = codes.size();
        const int perRowFull = std::max(1, int(std::floor((sw + gap) / (cubeSize + gap))));
        qreal size = cubeSize;
        if (n > perRowFull && sw > 0) {
            const qreal s = (sw + gap) / n - gap;
            size = std::max(minSize, std::min(cubeSize, std::floor(s * 2) / 2));
        }
        const int perRow = std::max(1, int(std::floor((sw + gap) / (size + gap))));
        const int fold = n > perRow ? int(std::ceil(double(n) / perRow)) : 1;
        static const QHash<QChar, int> urgency{{u'e', 0}, {u'm', 1}, {u't', 2}, {u'p', 3}, {u'g', 4},
                                               {u'a', 5}, {u'd', 6}, {u'l', 7}, {u'w', 8}, {u'f', 9}};
        stripH = size;
        const qreal y = h - p * 0.6 - size;
        int col = 0;
        for (int i = 0; i < n; i += fold) {
            QChar best = codes.at(i);
            for (int j = i + 1; j < std::min(n, i + fold); ++j)
                if (urgency.value(codes.at(j), 4) < urgency.value(best, 4))
                    best = codes.at(j);
            const auto colors = st.cubes.value(best, st.cubes.value(u'g'));
            const QRectF r(p + col * (size + gap), y, size, size);
            const qreal cr = std::max<qreal>(1, size * 0.3);
            if (colors.first.alpha() > 0)
                m_fills.push_back({r, cr, colors.first, {}, {}, -1});
            if (colors.second.alpha() > 0)
                m_strokes.push_back({r, cr, 1, colors.second});
            col += 1;
        }
    }

    // ---- The writing on the foot, bottom up: the title over one ledger line (small), or
    // the hero's title, next episode, countdown, page line, progress and Play row
    const QFont &titleFont = m_hero ? st.heroTitle : st.title;
    const qreal footW = w - p * 2;
    const QFontMetricsF lfm(st.ledger);
    const qreal ledgerH = lfm.height();
    const qreal pillH = std::ceil(ledgerH + u);
    const qreal spacing = u * (m_hero ? 1.5 : 1);
    qreal titleH = 0;
    const int titleAt = addTitle(it.value(QStringLiteral("title")).toString(), titleFont, footW, 2, &titleH);

    // The countdown row: the episode in the dim ledger, a clock and the time in accent
    auto countdownRow = [&](qreal x, qreal cy, qreal room) {
        qreal timeW = 0, timeH = 0, epW = 0, epH = 0;
        const int timeAt = addText(airingTime, st.ledgerStrong, 0, &timeW, &timeH);
        const qreal clock = std::round(ledgerH * 0.85), inner = u * 0.5, gapOut = u * 1.5;
        const qreal epRoom = std::max<qreal>(0, room - clock - timeW - gapOut - inner - u * 0.5);
        const int epAt = addText(airingEpisode, st.ledger, epRoom, &epW, &epH);
        m_runs.push_back({epAt, QPointF(x, cy - epH / 2), inkDim});
        const qreal cx = x + epW + gapOut;
        addClock(cx + clock / 2, cy, clock, inkAccent);
        m_runs.push_back({timeAt, QPointF(cx + clock + inner, cy - timeH / 2), inkAccent});
    };

    // Each row under the title: its height, and what lays it out at a given top
    struct Row { qreal h; std::function<void(qreal)> place; };
    std::vector<Row> rows;
    qreal footH = titleH;
    if (!m_hero) {
        footH += spacing + pillH;
        rows.push_back({pillH, [&, this](qreal top) {
            const qreal cy = top + pillH / 2;
            qreal sW = 0, sH = 0;
            int sAt = -1;
            if (!scoreText.isEmpty()) {
                sAt = addText(scoreText, st.ledger, 0, &sW, &sH);
                m_runs.push_back({sAt, QPointF(p + footW - sW, cy - sH / 2), mine ? inkAccent : inkDim});
            }
            const qreal room = footW - (sAt >= 0 ? sW + u * 2 : 0);
            if (airs)
                countdownRow(p, cy, room);
            else {
                qreal lw = 0, lh = 0;
                const int at = addText(ledgerText, st.ledger, room, &lw, &lh);
                m_runs.push_back({at, QPointF(p, cy - lh / 2), inkDim});
            }
        }});
    } else {
        const auto &itm = it;
        QString nextText;
        if (hasWatched && itm.value(QStringLiteral("watched_state")).toString() != QLatin1String("CaughtUp")) {
            const qint64 wn = qint64(itm.value(QStringLiteral("watched")).toDouble());
            if (!(totalKnown && wn >= total))
                nextText = QStringLiteral("Episode %1 next").arg(wn + 1);
        }
        auto line = [&, this](const QString &text, const QFont &font, const QColor &color) {
            if (text.isEmpty())
                return;
            const qreal lh = QFontMetricsF(font).height();
            footH += spacing + lh;
            rows.push_back({lh, [&, this, text, font, color](qreal top) {
                const int at = addText(text, font, footW, nullptr, nullptr);
                m_runs.push_back({at, QPointF(p, top), color});
            }});
        };
        line(nextText, st.ui, inkMain);
        if (airs) {
            footH += spacing + pillH;
            rows.push_back({pillH, [&](qreal top) { countdownRow(p, top + pillH / 2, footW); }});
        }
        line(m_meta, st.ledger, inkDim);
        line(!watchedText.isEmpty() ? watchedText : filesText, st.ledger, inkDim);
        // The spacer, then the Play row: the button is the card's, the score is drawn here
        const qreal playH = u * 8;
        footH += spacing + u * 1.5 + spacing + playH;
        rows.push_back({u * 1.5 + spacing + playH, [&, this](qreal top) {
            const qreal rowTop = top + u * 1.5 + spacing;
            m_playY = rowTop;
            if (!scoreText.isEmpty()) {
                qreal sW = 0, sH = 0;
                const int sAt = addText(scoreText, st.heroScore, 0, &sW, &sH);
                m_runs.push_back({sAt, QPointF(p + footW - sW, rowTop + playH / 2 - sH / 2), mine ? inkAccent : inkDim});
            }
        }});
    }

    const qreal footY = h - p - footH - (strip ? stripH + u : 0);

    // ---- The scrim, under the writing: rises from the foot, deepening on hover
    if (!isBare) {
        const qreal sh = std::min(h, std::max(h * (st.scrimReach + 0.06 * m_lift), footH + p * 2 + u * 6));
        QColor mid = st.posterScrim;
        mid.setAlphaF(st.posterScrim.alphaF() * (0.7 + 0.12 * m_lift));
        QColor clear = st.posterScrim;
        clear.setAlpha(0);
        // A pixel wider than the tile on the sides and the foot, so the scrim's own soft edge
        // lies over the poster's antialiased edge rather than inside it: a pale rim showed
        // in the corners where the poster's edge peeked out past the scrim's
        m_fills.insert(m_fills.begin(), {QRectF(-1, h - sh, w + 2, sh + 1), radius + 1, clear, mid, st.posterScrim, 0.45});
    } else {
        qreal nw = 0, nh = 0;
        const int at = addText(QStringLiteral("No poster"), st.ui, 0, &nw, &nh);
        m_runs.push_back({at, QPointF((w - nw) / 2, (h - nh) / 2 - h * 0.12), st.textFaint});
    }

    m_runs.push_back({titleAt, QPointF(p, footY), inkMain});
    qreal y = footY + titleH;
    for (const Row &row : rows) {
        y += spacing;
        row.place(y);
        y += row.h;
    }

    // ---- Tags along the top: the episode code left; New, Downloading, On hold, Dropped,
    // Hidden and the ghost's AniList right, in that order from the left
    const qreal ty = p * 0.8;
    const QString code = it.value(QStringLiteral("code")).toString();
    if (!code.isEmpty())
        addTag(code, st.onPoster, p * 0.8, ty, false, nullptr);
    struct Tag { QString text; QColor ink; };
    std::vector<Tag> tags;
    const int downloading = it.value(QStringLiteral("downloading")).toList().size();
    if (it.value(QStringLiteral("watched_state")).toString() == QLatin1String("Behind") && !m_ghost && !dropped && !onHold)
        tags.push_back({QStringLiteral("New"), st.accentOnPoster});
    if (downloading > 0 && !m_ghost)
        tags.push_back({QStringLiteral("Downloading"), st.downloadInk});
    if (onHold && !m_ghost)
        tags.push_back({QStringLiteral("On hold"), st.onPoster});
    if (dropped)
        tags.push_back({QStringLiteral("Dropped"), st.onPosterDim});
    if (it.value(QStringLiteral("hidden")).toBool())
        tags.push_back({QStringLiteral("Hidden"), st.onPosterDim});
    if (m_ghost && !it.value(QStringLiteral("site_url")).toString().isEmpty())
        tags.push_back({QStringLiteral("AniList"), st.onPoster});
    qreal right = w - p * 0.8 - m_rightInset;
    for (auto t = tags.rbegin(); t != tags.rend(); ++t) {
        qreal tw = 0;
        addTag(t->text, t->ink, right, ty, true, &tw);
        right -= tw + u;
    }

    // ---- Hover outline and the focus ring, over everything
    if (m_lift > 0.01 || m_focused) {
        QColor c = st.accent;
        c.setAlphaF(m_focused ? 1.0 : std::min<qreal>(1, m_lift));
        m_overStrokes.push_back({QRectF(0, 0, w, h), radius, 1, c});
    }
    if (m_focused)
        m_overStrokes.push_back({QRectF(-2, -2, w + 4, h + 4), radius + 2, 2, st.focusRing});

    m_dirty = false;
    emit laidOut();
    update();
}

// ---- Geometry: rounded shapes in vertex colour, antialiased by a one pixel fringe ------

namespace {
using Vertex = QSGGeometry::ColoredPoint2D;

struct Mesh {
    std::vector<Vertex> v;
    std::vector<quint32> i;
    quint32 add(qreal x, qreal y, const QColor &c)
    {
        Vertex p;
        const qreal a = c.alphaF();
        p.set(float(x), float(y), uchar(std::lround(c.redF() * a * 255)), uchar(std::lround(c.greenF() * a * 255)),
              uchar(std::lround(c.blueF() * a * 255)), uchar(std::lround(a * 255)));
        v.push_back(p);
        return quint32(v.size() - 1);
    }
    void quad(quint32 a, quint32 b, quint32 c, quint32 d) { i.insert(i.end(), {a, b, c, a, c, d}); }
};

QColor clear(QColor c)
{
    c.setAlpha(0);
    return c;
}
QColor mix(const QColor &a, const QColor &b, qreal t)
{
    // Premultiplied-correct mixing is done per vertex by the material; here straight
    return QColor::fromRgbF(float(a.redF() + (b.redF() - a.redF()) * t), float(a.greenF() + (b.greenF() - a.greenF()) * t),
                            float(a.blueF() + (b.blueF() - a.blueF()) * t), float(a.alphaF() + (b.alphaF() - a.alphaF()) * t));
}

int arcSteps(qreal r) { return std::clamp(int(std::ceil(r / 2.5)), 2, 12); }

// The points around a rounded rect, clockwise from the top left corner's start; the same
// count for any rect with the same `steps`, so two of them pair up into a ring
std::vector<QPointF> outline(const QRectF &r, qreal radius, int steps)
{
    radius = std::max<qreal>(0, std::min(radius, std::min(r.width(), r.height()) / 2));
    std::vector<QPointF> pts;
    const QPointF centres[4] = {{r.left() + radius, r.top() + radius}, {r.right() - radius, r.top() + radius},
                                {r.right() - radius, r.bottom() - radius}, {r.left() + radius, r.bottom() - radius}};
    for (int c = 0; c < 4; ++c) {
        const qreal a0 = M_PI + c * M_PI / 2;
        for (int k = 0; k <= steps; ++k) {
            const qreal a = a0 + (M_PI / 2) * k / steps;
            pts.push_back(centres[c] + QPointF(std::cos(a) * radius, std::sin(a) * radius));
        }
    }
    return pts;
}

void ring(Mesh &m, const std::vector<QPointF> &outer, const QColor &co, const std::vector<QPointF> &inner, const QColor &ci)
{
    const size_t n = outer.size();
    const quint32 base = quint32(m.v.size());
    for (size_t k = 0; k < n; ++k) {
        m.add(outer[k].x(), outer[k].y(), co);
        m.add(inner[k].x(), inner[k].y(), ci);
    }
    for (size_t k = 0; k < n; ++k) {
        const quint32 a = base + quint32(2 * k), b = base + quint32(2 * ((k + 1) % n));
        m.quad(a, b, b + 1, a + 1);
    }
}

// A filled rounded rect with a vertical colour: rows down the shape, each row's ends set in
// by the corner arcs, and a fringe one pixel out on every side
void fillShape(Mesh &m, const QRectF &r, qreal radius, const std::function<QColor(qreal)> &colorAt, std::vector<qreal> extraRows)
{
    radius = std::max<qreal>(0, std::min(radius, std::min(r.width(), r.height()) / 2));
    const int steps = arcSteps(radius);
    std::vector<qreal> ys{r.top(), r.bottom()};
    for (int k = 1; k <= steps; ++k) {
        const qreal a = (M_PI / 2) * k / steps;
        ys.push_back(r.top() + radius - std::cos(a) * radius);
        ys.push_back(r.bottom() - radius + std::cos(a) * radius);
    }
    for (qreal y : extraRows)
        if (y > r.top() && y < r.bottom())
            ys.push_back(y);
    std::sort(ys.begin(), ys.end());
    ys.erase(std::unique(ys.begin(), ys.end(), [](qreal a, qreal b) { return std::abs(a - b) < 0.01; }), ys.end());
    auto inset = [&](qreal y) {
        qreal dy = 0;
        if (y < r.top() + radius)
            dy = r.top() + radius - y;
        else if (y > r.bottom() - radius)
            dy = y - (r.bottom() - radius);
        else
            return qreal(0);
        return radius - std::sqrt(std::max<qreal>(0, radius * radius - dy * dy));
    };
    const qreal aa = 1.0;
    // Per row: outer left, left, right, outer right
    std::vector<std::array<quint32, 4>> rows;
    for (qreal y : ys) {
        const QColor c = colorAt(y);
        const qreal in = inset(y);
        rows.push_back({m.add(r.left() + in - aa, y, clear(c)), m.add(r.left() + in, y, c), m.add(r.right() - in, y, c),
                        m.add(r.right() - in + aa, y, clear(c))});
    }
    for (size_t k = 0; k + 1 < rows.size(); ++k) {
        const auto &a = rows[k], &b = rows[k + 1];
        m.quad(a[0], a[1], b[1], b[0]);
        m.quad(a[1], a[2], b[2], b[1]);
        m.quad(a[2], a[3], b[3], b[2]);
    }
    // The top and bottom edges' fringe
    auto cap = [&](const std::array<quint32, 4> &row, qreal y) {
        const Vertex l = m.v[row[1]], rr = m.v[row[2]];
        const QColor c = colorAt(y < r.center().y() ? r.top() : r.bottom());
        const quint32 a = m.add(l.x, y, clear(c)), b = m.add(rr.x, y, clear(c));
        if (y < r.center().y())
            m.quad(a, b, row[2], row[1]);
        else
            m.quad(row[1], row[2], b, a);
    };
    cap(rows.front(), r.top() - aa);
    cap(rows.back(), r.bottom() + aa);
}

void strokeShape(Mesh &m, const QRectF &r, qreal radius, qreal width, const QColor &c)
{
    const int steps = arcSteps(radius + 1);
    const qreal aa = 1.0;
    const auto outerAA = outline(r.adjusted(-aa / 2, -aa / 2, aa / 2, aa / 2), radius + aa / 2, steps);
    const auto outer = outline(r.adjusted(aa / 2, aa / 2, -aa / 2, -aa / 2), radius - aa / 2, steps);
    const qreal inW = std::max<qreal>(aa, width);
    const auto inner = outline(r.adjusted(inW - aa / 2, inW - aa / 2, -(inW - aa / 2), -(inW - aa / 2)),
                               std::max<qreal>(0, radius - inW + aa / 2), steps);
    const auto innerAA = outline(r.adjusted(inW + aa / 2, inW + aa / 2, -(inW + aa / 2), -(inW + aa / 2)),
                                 std::max<qreal>(0, radius - inW - aa / 2), steps);
    if (width > aa) {
        ring(m, outerAA, clear(c), outer, c);
        ring(m, outer, c, inner, c);
        ring(m, inner, c, innerAA, clear(c));
    } else {
        // A hairline: the colour's full strength at its middle, scaled by its width
        QColor mid = c;
        mid.setAlphaF(c.alphaF() * std::min<qreal>(1, width));
        const auto centre = outline(r.adjusted(width / 2, width / 2, -width / 2, -width / 2), std::max<qreal>(0, radius - width / 2), steps);
        ring(m, outerAA, clear(c), centre, mid);
        ring(m, centre, mid, innerAA, clear(c));
    }
}

void lineShape(Mesh &m, QPointF a, QPointF b, qreal width, const QColor &c)
{
    const QPointF d = b - a;
    const qreal len = std::hypot(d.x(), d.y());
    if (len <= 0)
        return;
    QPointF n(-d.y() / len, d.x() / len);
    QPointF t(d.x() / len, d.y() / len);
    // A round-ish cap: the segment runs half a width past each end
    a -= t * (width / 2);
    b += t * (width / 2);
    const qreal hw = width / 2, aa = 1.0;
    const quint32 o1 = m.add((a + n * (hw + aa / 2)).x(), (a + n * (hw + aa / 2)).y(), clear(c));
    const quint32 i1 = m.add((a + n * (hw - aa / 2)).x(), (a + n * (hw - aa / 2)).y(), c);
    const quint32 i2 = m.add((a - n * (hw - aa / 2)).x(), (a - n * (hw - aa / 2)).y(), c);
    const quint32 o2 = m.add((a - n * (hw + aa / 2)).x(), (a - n * (hw + aa / 2)).y(), clear(c));
    const quint32 p1 = m.add((b + n * (hw + aa / 2)).x(), (b + n * (hw + aa / 2)).y(), clear(c));
    const quint32 j1 = m.add((b + n * (hw - aa / 2)).x(), (b + n * (hw - aa / 2)).y(), c);
    const quint32 j2 = m.add((b - n * (hw - aa / 2)).x(), (b - n * (hw - aa / 2)).y(), c);
    const quint32 p2 = m.add((b - n * (hw + aa / 2)).x(), (b - n * (hw + aa / 2)).y(), clear(c));
    m.quad(o1, p1, j1, i1);
    m.quad(i1, j1, j2, i2);
    m.quad(i2, j2, p2, o2);
}

QSGGeometryNode *meshNode(const Mesh &m)
{
    auto *g = new QSGGeometry(QSGGeometry::defaultAttributes_ColoredPoint2D(), int(m.v.size()), int(m.i.size()),
                              QSGGeometry::UnsignedIntType);
    g->setDrawingMode(QSGGeometry::DrawTriangles);
    std::copy(m.v.begin(), m.v.end(), g->vertexDataAsColoredPoint2D());
    std::copy(m.i.begin(), m.i.end(), static_cast<quint32 *>(g->indexData()));
    auto *n = new QSGGeometryNode;
    n->setGeometry(g);
    n->setFlag(QSGNode::OwnsGeometry);
    n->setMaterial(new QSGVertexColorMaterial);
    n->setFlag(QSGNode::OwnsMaterial);
    return n;
}
}

// ---- Nodes: rebuilt whole when the layout moved; the GUI thread waits meanwhile --------

QSGNode *CardFace::updatePaintNode(QSGNode *old, UpdatePaintNodeData *)
{
    delete old;
    if (width() <= 0 || height() <= 0)
        return nullptr;
    auto *root = new QSGNode;

    Mesh below;
    for (const Fill &f : m_fills) {
        if (f.midAt < 0) {
            const QColor c = f.top;
            fillShape(below, f.rect, f.radius, [c](qreal) { return c; }, {});
        } else {
            const QRectF r = f.rect;
            const qreal midY = r.top() + r.height() * f.midAt;
            const QColor top = f.top, mid = f.mid, bottom = f.bottom;
            fillShape(below, r, f.radius,
                      [=](qreal y) {
                          if (y <= midY)
                              return mix(top, mid, r.height() > 0 ? (y - r.top()) / std::max<qreal>(1e-3, midY - r.top()) : 0);
                          return mix(mid, bottom, (y - midY) / std::max<qreal>(1e-3, r.bottom() - midY));
                      },
                      {midY});
        }
    }
    for (const Stroke &s : m_strokes)
        strokeShape(below, s.rect, s.radius, s.width, s.color);
    for (const Line &l : m_lines)
        lineShape(below, l.a, l.b, l.width, l.color);
    if (!below.v.empty())
        root->appendChildNode(meshNode(below));

    // One text node per colour
    if (QQuickWindow *win = window()) {
        QHash<QRgb, QSGTextNode *> byColor;
        for (const Run &r : m_runs) {
            if (r.layout < 0 || r.layout >= int(m_layouts.size()))
                continue;
            QSGTextNode *&node = byColor[r.color.rgba()];
            if (!node) {
                node = win->createTextNode();
                node->setColor(r.color);
                root->appendChildNode(node);
            }
            node->addTextLayout(r.at, m_layouts[r.layout].get());
        }
    }

    Mesh over;
    for (const Stroke &s : m_overStrokes)
        strokeShape(over, s.rect, s.radius, s.width, s.color);
    if (!over.v.empty())
        root->appendChildNode(meshNode(over));
    return root;
}
