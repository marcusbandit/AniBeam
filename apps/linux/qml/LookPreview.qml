// The Appearance preview: the same sample page drawn once per mode from the knobs as they
// are now, so a change to any control shows in both at once. Two panes side by side while
// the width holds two of theme.space(90) and the gap, stacked otherwise. The sample cards
// are the library's own first two dozen with a poster, in-progress ones first, so every corner
// of a card has something real to show; an empty library falls back to LookPane's own
// placeholder record instead of leaving the panes blank. Reloads, debounced, whenever the
// core says a series or a scan changed, the same idiom SettingsLibraryTab.qml uses for its
// own Door-driven stats.
import QtQuick
import com.marcusrosado.AniBeam

Item {
    id: root
    anchors.fill: parent
    property real nowMs: Date.now()

    readonly property var modes: ["dark", "light"]
    readonly property real gap: theme.space(4)
    readonly property int columns: width >= modes.length * theme.space(90) + (modes.length - 1) * gap ? modes.length : 1
    readonly property real paneWidth: (width - (columns - 1) * gap) / columns
    // Natural: the label and one row of cards per pane. Side by side the panes take the
    // whole height they are given and fill it with rows of cards; stacked, they keep it.
    readonly property real labelHeight: theme.space(2) + labelProbe.implicitHeight
    readonly property real paneNatural: rep.count && rep.itemAt(0) ? rep.itemAt(0).pane.naturalHeight : 0
    implicitHeight: columns > 1 ? labelHeight + paneNatural : (labelHeight + paneNatural) * modes.length + gap * (modes.length - 1)
    Text { id: labelProbe; visible: false; text: "Dark"; font.family: theme.fontUi; font.pointSize: theme.typeSmall; font.weight: Font.DemiBold }

    property var samples: []
    function reload() {
        // Samples describe the library, so the Watched tab rides along:
        // a finished-heavy library still has posters to preview with.
        var r = Door.listSeries("All", "", "LastViewed", "Desc", false)
        if (r.error) return
        var list = r.reply.series
        var w = Door.listSeries("Watched", "", "LastViewed", "Desc", false)
        if (!w.error) list = list.concat(w.reply.series)
        list = list.filter(function(i) { return !!i.poster })
        var going = list.filter(function(i) { return i.watched !== null && i.watched !== undefined && i.total_episodes && i.watched < i.total_episodes })
        var rest = list.filter(function(i) { return going.indexOf(i) < 0 })
        root.samples = going.concat(rest).slice(0, 24)
    }
    Component.onCompleted: reload()
    Timer { id: debounce; interval: 250; onTriggered: root.reload() }
    Connections {
        target: Door
        function onSeriesChanged(c) { debounce.restart() }
        function onScanFinished(s, a, c, r) { debounce.restart() }
    }

    Repeater {
        id: rep
        model: root.modes
        Column {
            property alias pane: lookPane
            required property int index
            required property string modelData
            x: (index % root.columns) * (root.paneWidth + root.gap)
            y: Math.floor(index / root.columns) * (height + root.gap)
            width: root.paneWidth
            spacing: theme.space(2)
            Text {
                text: modelData === "dark" ? "Dark" : "Light"
                color: theme.textDim
                font.family: theme.fontUi
                font.pointSize: theme.typeSmall
                font.weight: Font.DemiBold
            }
            LookPane {
                id: lookPane
                width: parent.width
                targetHeight: root.columns > 1 ? root.height - root.labelHeight : 0
                mode: modelData
                samples: root.samples
                nowMs: root.nowMs
            }
        }
    }
}
