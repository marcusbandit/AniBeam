// Spec 4.5: Settings on the mosaic. The header is a row of tiles at the page margin (the
// title, the four tabs), on the library's module; under it the current tab's tiles fill the
// rest of the viewport, set in the same joint. The tab and each tab's scroll are session
// state kept on the frame.
import QtQuick
import com.marcusrosado.AniBeam
import "mosaic.js" as Mosaic

FocusScope {
    id: page
    objectName: "settings-page"
    property var props: ({})
    property string title: "Settings"
    property real scrollY: 0
    readonly property var tabNames: ["Library", "Appearance", "Playback", "Data"]
    readonly property var tabIcons: ["folder", "palette", "play", "hard-drive"]
    property int tab: frame.settingsTab
    onTabChanged: frame.settingsTab = tab
    Component.onCompleted: {
        var want = tabNames.map(function(n) { return n.toLowerCase() }).indexOf(String(props.tab || "").toLowerCase())
        if (want >= 0) tab = want
        forceActiveFocus()
    }
    // Named escapePressed, not escape: QML's compiler reserves "escape" and refuses it as a
    // property, method or signal name on any object.
    function escapePressed() { return false }

    // ---- The module: the library's columns and unit
    readonly property real grout: theme.grout
    readonly property var geo: Mosaic.columns(Math.max(1, width - grout), theme.posterWidth, grout)
    readonly property real unit: geo.u
    readonly property real step: unit + grout
    function px(units) { return Mosaic.unitsToPx(units, unit, grout) }
    function spanX(col) { return geo.xs[Math.min(col, geo.cols - 1)] }
    function spanW(col, span) { var last = Math.min(geo.cols - 1, col + span - 1); return geo.xs[last] + geo.ws[last] - geo.xs[col] }
    readonly property real hpad: theme.space(2.5)
    function spanFor(w) { return Math.max(1, Math.ceil((w + hpad * 2 + grout) / (geo.w + grout))) }
    readonly property var headerLayout: Mosaic.header([
        { key: "title", span: Math.max(2, spanFor(titleLabel.implicitWidth)), flex: true },
        { key: "tabs", span: spanFor(tabsSeg.implicitWidth) }
    ], geo.cols)
    function headerTile(key) {
        var t = headerLayout.tiles
        for (var i = 0; i < t.length; i++) if (t[i].key === key) return t[i]
        return { col: 0, row: 0, colSpan: 1 }
    }
    readonly property real headerHeight: px(headerLayout.rows)

    component HeaderTile: Corner {
        property string key: ""
        readonly property var place: page.headerTile(key)
        x: page.spanX(place.col)
        y: page.grout + place.row * page.step
        width: page.spanW(place.col, place.colSpan)
        height: page.unit
        radius: theme.radiusTile
        smoothing: theme.cornerSmoothing
        color: theme.tile
    }

    HeaderTile {
        key: "title"
        Label {
            id: titleLabel
            x: page.hpad
            y: parent.height - page.hpad - height
            text: "Settings"
            role: "display"
        }
    }
    HeaderTile {
        key: "tabs"
        Label { x: page.hpad; y: page.hpad; text: "Section"; role: "ui"; font.pointSize: theme.typeSmall; color: theme.textDim }
        Seg {
            id: tabsSeg
            objectName: "settings-tabs"
            x: page.hpad
            y: parent.height - page.hpad - height
            options: page.tabNames.map(function(n, i) { return { text: n, icon: page.tabIcons[i] } })
            index: page.tab
            onPicked: function(i) { page.tab = i }
        }
    }
    Loader {
        id: content
        x: 0
        y: page.grout + page.headerHeight + page.grout
        width: page.width - page.grout
        height: page.height - y - page.grout
        sourceComponent: [libraryTab, appearanceTab, playbackTab, dataTab][page.tab]
        onLoaded: { item.contentY = frame.settingsScroll[page.tab] || 0 }
        Connections { target: content.item; function onContentYChanged() { var s = frame.settingsScroll.slice(); s[page.tab] = content.item.contentY; frame.settingsScroll = s } }
    }
    Component { id: libraryTab; SettingsLibraryTab {} }
    Component { id: appearanceTab; SettingsAppearanceTab {} }
    Component { id: playbackTab; SettingsPlaybackTab {} }
    Component { id: dataTab; SettingsDataTab {} }
}
