// The Look tab's preview: the same sample drawn once per mode from the knobs as they are
// now, so a change to any Look control shows in both at once. Two panes side by side, or
// stacked when the area is too narrow for both.
import QtQuick

Item {
    id: root
    property var library: []
    property string titleLang: "jp"
    property real nowMs: Date.now()

    readonly property var modes: ["dark", "light"]
    readonly property real gap: theme.space(4)
    readonly property int columns: width >= modes.length * theme.space(72) + (modes.length - 1) * gap ? modes.length : 1
    readonly property real paneWidth: (width - (columns - 1) * gap) / columns
    implicitHeight: childrenRect.height

    // A series with a poster and progress under way shows every corner fact; failing that any
    // poster, failing that the no-poster state
    readonly property var sample: {
        var list = library || []
        var pick = list.find(function(i) { return i.poster && i.watched !== null && i.watched !== undefined && i.total && i.watched < i.total })
            || list.find(function(i) { return i.poster })
        if (pick) return pick
        return { folderName: "Sample series", titleRomaji: "Sample series", fileCount: 12, latestFile: 12, watched: 8, total: 12, score: 8.2 }
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
                id: label
                text: modelData
                color: theme.textDim
                font.family: theme.fontSans
                font.pointSize: theme.typeSmall
                font.capitalization: Font.AllUppercase
                font.letterSpacing: 1
            }
            LookPane {
                id: pane
                width: parent.width
                mode: modelData
                host: theme
                sample: root.sample
                titleLang: root.titleLang
                nowMs: root.nowMs
            }
        }
    }
}
