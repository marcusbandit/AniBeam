// One pane of the glass mosaic: a square tile of translucent glass over the ground, a 1 px
// edge and a 1 px top highlight, with its content set inside a padding. Every surface on a
// page is one of these, laid out with theme.grout (2 px) between panes and as the page's
// own margin. Children go into `content`, already inset. `raised` is the hovered/selected
// step. `tone` is accepted for old callers and ignored: no surface takes a poster's colour.
import QtQuick

Corner {
    id: root
    property color tone: "transparent"
    property bool raised: false
    property real padding: theme.space(4)
    // False drops the top highlight (a pane flush under another, a quiet strip)
    property bool lit: true
    readonly property color fill: raised ? theme.raised : theme.tile
    default property alias contentData: content.data
    color: fill
    borderColor: theme.line
    borderWidth: 1
    highlight: lit ? theme.glassHighlight : "transparent"
    Item {
        id: content
        anchors.fill: parent
        anchors.margins: root.padding
    }
}
