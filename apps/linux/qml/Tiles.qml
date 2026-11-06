// A row of figures set flat on the ground: the figure in big tabular ledger figures over
// its caption in dim ui text, no box around either. As many per line as the width holds
// at theme.space(36) a figure, the lines evened out so four never split three and one.
// `stone` is accepted for old callers and ignored: nothing here is a tile any more.
import QtQuick
import QtQuick.Layouts

Grid {
    id: root
    property var tiles: []                 // [{ value, caption }]
    property color stone: "transparent"
    readonly property real gap: theme.space(8)
    // The figures are text, which density does not shrink, so neither does their width
    readonly property real minTile: Math.round(4 * 36 * Math.max(1, theme.textScale))
    readonly property int fit: Math.max(1, Math.floor((width + gap) / (minTile + gap)))
    readonly property int lines: Math.max(1, Math.ceil(tiles.length / fit))
    readonly property real tileWidth: (width - (columns - 1) * gap) / columns
    width: parent ? parent.width : theme.space(100)
    Layout.fillWidth: true
    columns: Math.max(1, Math.ceil(tiles.length / lines))
    columnSpacing: gap
    rowSpacing: theme.space(4)
    Repeater {
        model: root.tiles
        Column {
            required property var modelData
            width: root.tileWidth
            spacing: theme.space(0.5)
            Label {
                width: parent.width
                elide: Text.ElideRight
                text: modelData.value
                role: "ledger"
                font.pointSize: theme.typeDisplay
                font.weight: 700
                font.variableAxes: ({ "wdth": 88, "wght": 700 })
            }
            Label {
                width: parent.width
                text: modelData.caption
                role: "ui"
                color: theme.textDim
                elide: Text.ElideRight
            }
        }
    }
}
