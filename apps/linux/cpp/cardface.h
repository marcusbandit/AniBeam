#pragma once
// The poster card's face, drawn natively (#289). Everything a card writes and draws over
// its poster: the scrim, the tags, the title, the ledger line or the airing countdown, the
// score, the episode cubes, the hover outline and the focus ring. One QQuickItem builds it
// all as two geometry nodes and a text node per colour, where the QML card it replaces
// made about sixty objects, each with its own bindings run by the JavaScript engine; a
// card went from about 2.4 ms to build to a fraction of that.
//
// CardStyle holds what every card draws with: the fonts of the four roles, the theme's
// inks, the cube colours and the spacing unit. Main.qml hands it the tokens once, and
// again whenever the theme changes; each face reads it when it lays out.
#include <QtCore/QHash>
#include <QtCore/QPointer>
#include <QtCore/QVariant>
#include <QtGui/QColor>
#include <QtGui/QFont>
#include <QtGui/QTextLayout>
#include <QtQml/qqmlregistration.h>
#include <QtQuick/QQuickItem>
#include <memory>
#include <vector>

class CardStyle : public QObject
{
    Q_OBJECT
    QML_ELEMENT
    QML_SINGLETON
public:
    explicit CardStyle(QObject *parent = nullptr);
    ~CardStyle() override;
    static CardStyle *instance();

    // The tokens, as Main.qml gathers them: fontFamily, fontFigure, typeSmall, typeNormal,
    // typeLarge, space (one spacing unit), radiusTile, radiusSm, scrimReach, the inks
    // (text, textDim, textFaint, accent, focusRing, onPoster, onPosterDim, accentOnPoster,
    // posterScrim, downloadInk) and cubes, a map from a cube code to { fill, edge }.
    Q_INVOKABLE void set(const QVariantMap &tokens);

    struct Data {
        QFont title, heroTitle, ledger, ledgerStrong, heroScore, ui, heroUi, tag, heroTag;
        QColor text, textDim, textFaint, accent, focusRing, onPoster, onPosterDim, accentOnPoster, posterScrim, downloadInk;
        QColor tagFill{8, 9, 11, 158}, tagEdge{255, 255, 255, 36};
        QHash<QChar, QPair<QColor, QColor>> cubes;
        qreal unit = 4, radiusTile = 18, radiusSm = 6, scrimReach = 0.55;
    };
    const Data &data() const { return m_data; }

signals:
    void changed();

private:
    Data m_data;
};

// Which of a page's waiting LazyTiles are within reach of the viewport (#289): one call
// that measures every tile in C++, where a JavaScript loop over mapToItem cost a page of
// 300 series about 30 ms a visit.
class TileReach : public QObject
{
    Q_OBJECT
    QML_ELEMENT
    QML_SINGLETON
public:
    explicit TileReach(QObject *parent = nullptr) : QObject(parent) {}
    // For each tile (in `tiles`), measured in `content`'s coordinates: 2 when it overlaps
    // [viewTop, viewBottom], 1 when it overlaps [reachTop, reachBottom] only, 0 otherwise
    Q_INVOKABLE QList<int> measure(QQuickItem *content, const QVariantList &tiles, qreal viewTop, qreal viewBottom,
                                   qreal reachTop, qreal reachBottom) const;
};

class CardFace : public QQuickItem
{
    Q_OBJECT
    QML_ELEMENT
    Q_PROPERTY(QVariantMap item READ item WRITE setItem NOTIFY itemChanged)
    Q_PROPERTY(QString metaOverride READ metaOverride WRITE setMetaOverride NOTIFY metaOverrideChanged)
    Q_PROPERTY(QVariant countdown READ countdown WRITE setCountdown NOTIFY countdownChanged)
    Q_PROPERTY(bool ghost READ ghost WRITE setGhost NOTIFY ghostChanged)
    Q_PROPERTY(bool hero READ hero WRITE setHero NOTIFY heroChanged)
    Q_PROPERTY(double nowMs READ nowMs WRITE setNowMs NOTIFY nowMsChanged)
    Q_PROPERTY(qreal lift READ lift WRITE setLift NOTIFY liftChanged)
    Q_PROPERTY(bool focused READ focused WRITE setFocused NOTIFY focusedChanged)
    Q_PROPERTY(qreal rightInset READ rightInset WRITE setRightInset NOTIFY rightInsetChanged)
    // The decoded poster: an Image whose texture the face draws, cropped to fill the tile and
    // rounded by a shader; null until it is Ready. posterOpacity dims a ghost's or a dropped
    // series'.
    Q_PROPERTY(QQuickItem *poster READ poster WRITE setPoster NOTIFY posterChanged)
    Q_PROPERTY(qreal posterOpacity READ posterOpacity WRITE setPosterOpacity NOTIFY posterChanged)
    // Read by the card: where the hero's Play row sits, and what a tip names
    Q_PROPERTY(qreal playY READ playY NOTIFY laidOut)
    Q_PROPERTY(qreal pad READ pad NOTIFY laidOut)
    Q_PROPERTY(bool bare READ bare NOTIFY itemChanged)
    Q_PROPERTY(QString folderName READ folderName NOTIFY itemChanged)
    Q_PROPERTY(QString title READ title NOTIFY itemChanged)

public:
    explicit CardFace(QQuickItem *parent = nullptr);
    ~CardFace() override;

    QVariantMap item() const { return m_item; }
    void setItem(const QVariantMap &v);
    QString metaOverride() const { return m_meta; }
    void setMetaOverride(const QString &v);
    QVariant countdown() const { return m_countdown; }
    void setCountdown(const QVariant &v);
    bool ghost() const { return m_ghost; }
    void setGhost(bool v);
    bool hero() const { return m_hero; }
    void setHero(bool v);
    double nowMs() const { return m_nowMs; }
    void setNowMs(double v);
    qreal lift() const { return m_lift; }
    void setLift(qreal v);
    bool focused() const { return m_focused; }
    void setFocused(bool v);
    qreal rightInset() const { return m_rightInset; }
    void setRightInset(qreal v);
    QQuickItem *poster() const { return m_poster; }
    void setPoster(QQuickItem *v);
    qreal posterOpacity() const { return m_posterOpacity; }
    void setPosterOpacity(qreal v);
    qreal playY() const { return m_playY; }
    qreal pad() const;
    bool bare() const;
    QString folderName() const;
    QString title() const;

    // "2d 5h", "4h 12m", "12m": short enough to share a line with the episode
    static QString shortCountdown(double secs);

signals:
    void itemChanged();
    void metaOverrideChanged();
    void countdownChanged();
    void ghostChanged();
    void heroChanged();
    void nowMsChanged();
    void liftChanged();
    void focusedChanged();
    void rightInsetChanged();
    void posterChanged();
    void laidOut();

protected:
    void updatePolish() override;
    QSGNode *updatePaintNode(QSGNode *old, UpdatePaintNodeData *) override;
    void geometryChange(const QRectF &now, const QRectF &before) override;

private:
    struct Fill {
        QRectF rect;
        qreal radius;
        QColor top, mid, bottom;   // a vertical gradient: top, at `midAt`, bottom
        qreal midAt = -1;          // -1 for a flat fill in `top`
    };
    struct Stroke {
        QRectF rect;
        qreal radius, width;
        QColor color;
    };
    struct Run {
        int layout;
        QPointF at;
        QColor color;
    };
    struct Line {
        QPointF a, b;
        qreal width;
        QColor color;
    };

    void relayout();
    int addText(const QString &text, const QFont &font, qreal maxWidth, qreal *width, qreal *height);
    int addTitle(const QString &text, const QFont &font, qreal width, int maxLines, qreal *height);
    void addTag(const QString &text, const QColor &ink, qreal x, qreal y, bool alignRight, qreal *width);
    void addClock(qreal cx, qreal cy, qreal size, const QColor &c);

    QVariantMap m_item;
    QString m_meta;
    QVariant m_countdown;
    bool m_ghost = false, m_hero = false, m_focused = false;
    double m_nowMs = 0;
    qreal m_lift = 0, m_rightInset = 0, m_playY = 0, m_posterOpacity = 1;
    QPointer<QQuickItem> m_poster;
    QMetaObject::Connection m_textureWatch;

    std::vector<std::unique_ptr<QTextLayout>> m_layouts;
    std::vector<Fill> m_fills;
    std::vector<Stroke> m_strokes, m_overStrokes;
    std::vector<Line> m_lines;
    std::vector<Run> m_runs;
    bool m_dirty = true;
    bool m_styled = false;
};
