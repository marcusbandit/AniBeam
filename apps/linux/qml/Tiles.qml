// A row of stat tiles: a big fixed-face number over a caption, as many per line as the
// width holds at theme.space(36) a tile, the lines evened out so four tiles never split
// three and one.
import QtQuick

Grid {
    id: root
    property var tiles: []                 // [{ value, caption }]
    readonly property real gap: theme.space(2)
    readonly property real minTile: theme.space(36)
    readonly property int fit: Math.max(1, Math.floor((width + gap) / (minTile + gap)))
    readonly property int lines: Math.max(1, Math.ceil(tiles.length / fit))
    readonly property real tileWidth: (width - (columns - 1) * gap) / columns
    width: parent ? parent.width : theme.space(100)
    columns: Math.max(1, Math.ceil(tiles.length / lines))
    columnSpacing: gap
    rowSpacing: gap
    Repeater {
        model: root.tiles
        Corner {
            required property var modelData
            width: root.tileWidth
            height: words.height + theme.space(3) * 2
            radius: theme.radiusMd
            smoothing: theme.cornerSmoothing
            color: theme.surfaceSunken
            borderColor: theme.line
            borderWidth: 1
            Column {
                id: words
                x: theme.space(4)
                anchors.verticalCenter: parent.verticalCenter
                width: parent.width - theme.space(4) * 2
                spacing: theme.space(0.5)
                Text {
                    text: modelData.value
                    color: theme.text
                    font.family: theme.fontMono
                    font.pointSize: theme.typeLarge
                    font.weight: Font.Bold
                }
                Text {
                    width: parent.width
                    text: modelData.caption
                    color: theme.textDim
                    elide: Text.ElideRight
                    font.family: theme.fontSans
                    font.pointSize: theme.typeSmall
                }
            }
        }
    }
}
