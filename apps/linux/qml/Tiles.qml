// A row of stat stones set straight on the grout: a caption at the top left and the figure
// in big tabular ledger figures at the bottom right, the way the library's section stones
// carry their counts. As many per line as the width holds at theme.space(44) a stone, the
// lines evened out so four never split three and one; one grout between them.
import QtQuick
import QtQuick.Layouts

Grid {
    id: root
    property var tiles: []                 // [{ value, caption }]
    // The stone's colour: the resting tile on the grout, raised when set inside a tile
    property color stone: theme.tile
    readonly property real gap: theme.grout
    readonly property real minTile: theme.space(44)
    readonly property int fit: Math.max(1, Math.floor((width + gap) / (minTile + gap)))
    readonly property int lines: Math.max(1, Math.ceil(tiles.length / fit))
    readonly property real tileWidth: (width - (columns - 1) * gap) / columns
    width: parent ? parent.width : theme.space(100)
    Layout.fillWidth: true
    columns: Math.max(1, Math.ceil(tiles.length / lines))
    columnSpacing: gap
    rowSpacing: gap
    Repeater {
        model: root.tiles
        Corner {
            required property var modelData
            width: root.tileWidth
            height: theme.space(26)
            color: root.stone
            borderColor: theme.line
            borderWidth: 1
            highlight: theme.glassHighlight
            Label {
                x: theme.space(4); y: theme.space(4)
                width: parent.width - theme.space(8)
                text: modelData.caption
                role: "ui"
                font.pointSize: theme.typeSmall
                color: theme.textDim
                elide: Text.ElideRight
            }
            Label {
                anchors.right: parent.right; anchors.rightMargin: theme.space(4)
                anchors.bottom: parent.bottom; anchors.bottomMargin: theme.space(3)
                width: Math.min(implicitWidth, parent.width - theme.space(8))
                horizontalAlignment: Text.AlignRight
                elide: Text.ElideRight
                text: modelData.value
                role: "ledger"
                font.pointSize: theme.typeDisplay
                font.weight: 700
                font.variableAxes: ({ "wdth": 88, "wght": 700 })
            }
        }
    }
}
