// The Appearance preview: the same sample page drawn once per mode from the knobs as they
// are now, so a change to any control shows in both at once. Two panes side by side while
// the width holds two of theme.space(90) and the gap, stacked otherwise.
import QtQuick

Item {
    id: root
    property var library: []
    property string titleLang: "jp"
    property real nowMs: Date.now()

    readonly property var modes: ["dark", "light"]
    readonly property real gap: theme.space(4)
    readonly property int columns: width >= modes.length * theme.space(90) + (modes.length - 1) * gap ? modes.length : 1
    readonly property real paneWidth: (width - (columns - 1) * gap) / columns
    implicitHeight: childrenRect.height

    // Records with posters, the ones with progress under way first so every corner fact shows
    readonly property var samples: {
        var list = (library || []).filter(function(i) { return i.poster })
        var going = list.filter(function(i) { return i.watched !== null && i.watched !== undefined && i.total && i.watched < i.total })
        var rest = list.filter(function(i) { return going.indexOf(i) < 0 })
        return going.concat(rest).slice(0, 8)
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
                host: theme
                samples: root.samples
                seriesCount: (root.library || []).length
                titleLang: root.titleLang
                nowMs: root.nowMs
            }
        }
    }
}
