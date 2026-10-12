// A page as a mosaic wall (DESIGN.md, Layout), for the pages that are not the library: the
// library's module (mosaic.js columns, the half-column unit, the grout as joint and page
// margin), a header row of tiles, and under it either packed series tiles (`sections`,
// packed by mosaic.js with the library's label stones, heroes and live window) or a `body`
// of the page's own tiles laid on the same columns. LibraryPage is the reference this
// follows; it keeps its own copy because its header and sweep are its alone.
//
// Header tiles are MosaicWall.HeaderTile children: each says the width its content needs
// (`need`) and whether it takes the row's spare columns (`flex`); the wall gives it whole
// column spans, one unit tall, wrapping on a narrow window.
//
// sections: [{ label, count, accent, heroes: [entry], cards: [entry] }]; an entry is
// { id, item (the Card's series), meta, countdown, ghost }. An empty label has no stone.
// When no section holds a tile, `body` (if set) fills the rest of the viewport.
import QtQuick
import QtQuick.Controls.Basic as QC
import com.marcusrosado.AniBeam
import "mosaic.js" as Mosaic

Item {
    id: wall
    property var sections: []
    property real nowMs: Date.now()
    property Component body: null
    default property alias headerData: header.data
    property alias scrollY: flick.contentY
    property alias flick: flick
    signal opened(var entry)
    signal played(var entry)

    // One header tile, cut by the wall
    component HeaderTile: Corner {
        property real need: 0
        property bool flex: false
        property int minSpan: 1
        readonly property real pad: theme.space(3)
        readonly property var slot: parent && parent.slotOf ? parent.slotOf(this) : null
        x: slot ? slot.x : 0
        y: slot ? slot.y : 0
        width: slot ? slot.w : 0
        height: slot ? slot.h : 0
        radius: theme.radiusTile
        smoothing: theme.cornerSmoothing
        color: theme.tile
    }

    // ---- The module
    readonly property real grout: theme.grout
    readonly property var geo: Mosaic.columns(Math.max(1, width - grout), theme.posterWidth, grout)
    readonly property real unit: geo.u
    readonly property real step: unit + grout
    function px(units) { return Mosaic.unitsToPx(units, unit, grout) }
    function spanX(col) { return geo.xs[Math.min(col, geo.cols - 1)] }
    function spanW(col, span) { var last = Math.min(geo.cols - 1, col + span - 1); return geo.xs[last] + geo.ws[last] - geo.xs[col] }
    function spanFor(w) { return Math.max(1, Math.ceil((w + theme.space(6) + grout) / (geo.w + grout))) }
    Card { id: probe; visible: false; live: false; posterWidth: wall.geo.w }
    readonly property int unitsSmall: Mosaic.unitsFor(probe.posterHeight + probe.minPlate, unit, grout)
    readonly property int unitsHero: unitsSmall * 2

    readonly property var packed: Mosaic.pack(sections, geo.cols, unitsSmall, unitsHero)
    readonly property var tiles: packed.tiles
    function packedOverlaps() { return Mosaic.overlaps(tiles) }
    function tileCount() { return tiles.filter(function(t) { return t.kind !== "label" }).length }
    function heroCount() { return tiles.filter(function(t) { return t.kind === "hero" }).length }

    readonly property var headerLayout: {
        var items = []
        for (var i = 0; i < header.children.length; i++) {
            var c = header.children[i]
            if (c.need === undefined || !c.visible) continue
            items.push({ key: i, span: Math.max(c.minSpan, spanFor(c.need)), flex: c.flex })
        }
        return Mosaic.header(items, geo.cols)
    }
    readonly property real headerHeight: header.children.length ? px(headerLayout.rows) : 0

    Flickable {
        id: flick
        objectName: "wall-flick"   // a --drive target: the page's scroll lives here
        y: wall.grout
        width: wall.width - wall.grout
        height: wall.height - wall.grout * 2
        clip: true
        boundsBehavior: Flickable.StopAtBounds
        contentWidth: width
        readonly property real wallTop: wall.headerHeight > 0 ? wall.headerHeight + wall.grout : 0
        readonly property real bodyHeight: bodyLoader.item ? Math.max(bodyLoader.item.implicitHeight, height - wallTop) : 0
        contentHeight: wallTop + Math.max(wall.px(wall.packed.rows), bodyHeight)
        QC.ScrollBar.vertical: scrollBar
        WheelStep { flick: flick }

        Item {
            id: header
            width: flick.width
            height: wall.headerHeight
            function slotOf(item) {
                var t = wall.headerLayout.tiles
                for (var i = 0; i < t.length; i++) {
                    if (header.children[t[i].key] !== item) continue
                    return { x: wall.spanX(t[i].col), y: t[i].row * wall.step, w: wall.spanW(t[i].col, t[i].colSpan), h: wall.unit }
                }
                return null
            }
        }

        // The live window, as on the library: the tiles are sorted by row
        readonly property int firstLive: {
            var t = wall.tiles, lo = 0, hi = t.length
            var lim = (contentY - wallTop - height - wall.px(wall.unitsHero)) / wall.step
            while (lo < hi) { var m = (lo + hi) >> 1; if (t[m].row < lim) lo = m + 1; else hi = m }
            return lo
        }
        readonly property int lastLive: {
            var t = wall.tiles, lo = 0, hi = t.length
            var lim = (contentY - wallTop + height * 2) / wall.step
            while (lo < hi) { var m = (lo + hi) >> 1; if (t[m].row <= lim) lo = m + 1; else hi = m }
            return lo - 1
        }

        Repeater {
            model: wall.tiles
            delegate: Item {
                id: cell
                required property var modelData
                required property int index
                readonly property bool stone: modelData.kind === "label"
                readonly property bool hero: modelData.kind === "hero"
                readonly property var section: wall.sections[modelData.section]
                readonly property bool live: index >= flick.firstLive && index <= flick.lastLive
                readonly property bool inView: y >= flick.contentY && y + height <= flick.contentY + flick.height
                objectName: stone ? "stone-" + String(section.label).toLowerCase().replace(/ /g, "-") : "tile-" + modelData.card.id
                x: wall.spanX(modelData.col)
                y: flick.wallTop + modelData.row * wall.step
                width: wall.spanW(modelData.col, modelData.colSpan)
                height: wall.px(modelData.rowSpan)
                Loader {
                    anchors.fill: parent
                    active: cell.live
                    sourceComponent: cell.stone ? labelStone : seriesTile
                }
                Component {
                    id: seriesTile
                    Card {
                        readonly property var entry: cell.modelData.card
                        objectName: cell.inView ? (cell.hero ? "hero-tile" : "small-tile") : ""
                        item: entry.item || ({})
                        metaOverride: entry.meta || ""
                        countdown: entry.countdown || null
                        ghost: !!entry.ghost
                        hero: cell.hero
                        posterWidth: cell.width
                        tileHeight: cell.height
                        live: true
                        nowMs: wall.nowMs
                        onOpened: wall.opened(entry)
                        onPlayed: wall.played(entry)
                    }
                }
                Component {
                    id: labelStone
                    Corner {
                        readonly property bool accent: !!cell.section.accent
                        radius: theme.radiusTile
                        smoothing: theme.cornerSmoothing
                        color: accent ? theme.plate(theme.accent) : theme.raised
                        Label {
                            x: theme.space(4); y: theme.space(4)
                            width: parent.width - theme.space(8)
                            text: cell.section.label
                            role: "title"
                            wrapMode: Text.Wrap
                            maximumLineCount: 3
                            elide: Text.ElideRight
                        }
                        Label {
                            anchors.right: parent.right; anchors.rightMargin: theme.space(4)
                            anchors.bottom: parent.bottom; anchors.bottomMargin: theme.space(3)
                            text: cell.section.count
                            role: "ledger"
                            font.pointSize: theme.typeDisplay * 1.6
                            font.weight: 700
                            font.variableAxes: ({ "wdth": 88, "wght": 700 })
                            color: parent.accent ? theme.text : theme.textDim
                        }
                    }
                }
            }
        }

        // The page's own tiles when there are no series tiles: at least the rest of the
        // viewport, so an empty state is a tile that fills the page
        Loader {
            id: bodyLoader
            y: flick.wallTop
            width: flick.width
            height: flick.bodyHeight
            active: wall.body !== null && wall.tiles.length === 0
            sourceComponent: wall.body
        }
    }
    ThinScrollBar {
        id: scrollBar
        x: wall.width - wall.grout + Math.max(0, (wall.grout - width) / 2)
        y: flick.y
        height: flick.height
        width: Math.min(wall.grout, theme.space(1))
    }
}
