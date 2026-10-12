// One stone of the mosaic: a G2 rounded tile in the tile colour (or a plate cut from a
// tone), with its content set inside a padding. Every surface on a page is one of these,
// laid out with theme.grout between tiles and as the page's own margin. Children go into
// `content`, already inset; `tone` paints the tile as a plate of that colour (text on it
// should read theme.contrast against `fill`). `raised` is the hovered/selected step.
import QtQuick

Corner {
    id: root
    property color tone: "transparent"
    property bool raised: false
    property real padding: theme.space(4)
    readonly property color fill: tone.a > 0 ? theme.plate(tone) : raised ? theme.raised : theme.tile
    default property alias contentData: content.data
    color: fill
    radius: theme.radiusTile
    smoothing: theme.cornerSmoothing
    Item {
        id: content
        anchors.fill: parent
        anchors.margins: root.padding
    }
}
