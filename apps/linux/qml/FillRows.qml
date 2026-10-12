// A list for a tile that takes spare height: as many rows of `model` as the height holds,
// no scroll and no half row, so a growing tile is full of rows instead of hollow. Its
// implicit height is `minRows` rows, the least it asks of its column. The delegate gets
// `modelData` and `index` like a Repeater's, and is `rowHeight` tall.
import QtQuick

Item {
    id: root
    property var model: []
    property real rowHeight: theme.space(10)
    property int minRows: 3
    property Component delegate: null
    readonly property int fit: Math.max(0, Math.floor(height / rowHeight))
    readonly property int shown: Math.min(model.length, fit)
    implicitHeight: Math.min(model.length, minRows) * rowHeight
    clip: true
    Column {
        width: parent.width
        Repeater { model: root.model.slice(0, root.shown); delegate: root.delegate }
    }
}
