// The Appearance preview: the same sample page drawn once per mode from the knobs as they
// are now, so a change to any control shows in both at once. Two panes side by side while
// the width holds two of theme.space(90) and the gap, stacked otherwise. The sample cards
// are the library's own first eight with a poster, in-progress ones first, so every corner
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
    implicitHeight: childrenRect.height

    property var samples: []
    function reload() {
        var r = Door.listSeries("All", "", "LastViewed", "Desc", false)
        if (r.error) return
        var list = r.reply.series.filter(function(i) { return !!i.poster })
        var going = list.filter(function(i) { return i.watched !== null && i.watched !== undefined && i.total_episodes && i.watched < i.total_episodes })
        var rest = list.filter(function(i) { return going.indexOf(i) < 0 })
        root.samples = going.concat(rest).slice(0, 8)
    }
    Component.onCompleted: reload()
    Timer { id: debounce; interval: 250; onTriggered: root.reload() }
    Connections {
        target: Door
        function onSeriesChanged(c) { debounce.restart() }
        function onScanFinished(s, a, c, r) { debounce.restart() }
    }

    Repeater {
        model: root.modes
        Column {
            required property int index
            required property string modelData
            x: (index % root.columns) * (root.paneWidth + root.gap)
            y: Math.floor(index / root.columns) * (height + root.gap)
            width: root.paneWidth
            spacing: theme.space(2)
            Text {
                text: modelData
                color: theme.textDim
                font.family: theme.fontSans
                font.pointSize: theme.typeSmall
                font.capitalization: Font.AllUppercase
                font.letterSpacing: 1
            }
            LookPane {
                width: parent.width
                mode: modelData
                samples: root.samples
                nowMs: root.nowMs
            }
        }
    }
}
