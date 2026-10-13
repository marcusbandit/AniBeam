// A settings panel: a tile (no edge, the grout around it is the edge), a Lucide glyph beside
// a heading in the title cut,
// an optional helper line, then its rows. Three slots: `rows` (the default) flow from the
// top at their natural height, `stretch` holds one item that takes whatever is left when
// the panel is taller than its content, and `foot` pins rows to the bottom edge. The panel
// is laid out by the ColumnLayout it sits in: it fills the column's width, its minimum is
// its content height so it is never squished, and with `grows` it takes the column's spare
// height (Layout.fillHeight). With `spread` the rows share that spare height as even
// gaps between them, so a growing panel of rows is never hollow at its foot.
import QtQuick
import QtQuick.Layouts

Corner {
    id: root
    property string title: ""
    property string icon: ""
    property string helper: ""
    property bool grows: false
    property bool spread: false
    default property alias rows: body.data
    property alias stretch: fill.data
    property alias foot: footer.data

    readonly property real pad: theme.space(6)
    readonly property real gap: theme.space(3)
    readonly property Item stretchItem: fill.children.length ? fill.children[0] : null
    readonly property real naturalStretch: stretchItem ? stretchItem.implicitHeight : 0
    readonly property bool hasRows: body.children.length > 0
    readonly property bool hasFoot: footer.children.length > 0
    readonly property real rowGap: theme.space(2)
    // The rows' natural height read off the children, not body.height, so `spread` can
    // widen the gaps without feeding back into the height it spreads over. A Column lays
    // out only visible children with a size, so only those count.
    readonly property var laidRows: {
        var n = 0, sum = 0
        for (var i = 0; i < body.children.length; i++) {
            var c = body.children[i]
            if (!c.visible || c.height <= 0 || c.width <= 0) continue
            n++; sum += c.height
        }
        return { n: n, sum: sum }
    }
    readonly property real bodyNatural: laidRows.sum + rowGap * Math.max(0, laidRows.n - 1)

    implicitHeight: pad * 2 + head.height
                    + (hasRows ? gap + bodyNatural : 0)
                    + (stretchItem ? gap + naturalStretch : 0)
                    + (hasFoot ? gap + footer.height : 0)
    Layout.fillWidth: true
    Layout.fillHeight: grows
    Layout.minimumHeight: implicitHeight
    Layout.preferredHeight: implicitHeight
    color: theme.tile
    borderColor: theme.line
    borderWidth: 1
    highlight: theme.glassHighlight

    Column {
        id: head
        x: root.pad
        y: root.pad
        width: root.width - root.pad * 2
        spacing: root.gap

        Row {
            spacing: theme.space(2)
            Icon {
                visible: root.icon !== ""
                anchors.verticalCenter: parent.verticalCenter
                glyph: root.icon
                color: theme.text
            }
            Label {
                anchors.verticalCenter: parent.verticalCenter
                text: root.title
                role: "title"
                font.pointSize: theme.typeNormal
            }
        }
        Label {
            visible: root.helper !== ""
            width: parent.width
            text: root.helper
            color: theme.textDim
            wrapMode: Text.Wrap
            role: "ui"
            font.pointSize: theme.typeSmall
        }
    }
    Column {
        id: body
        anchors.top: head.bottom
        anchors.topMargin: root.hasRows ? root.gap : 0
        x: root.pad
        width: root.width - root.pad * 2
        spacing: root.spread && root.laidRows.n > 1 && !root.stretchItem
                 ? root.rowGap + Math.max(0, root.height - root.implicitHeight) / (root.laidRows.n - 1)
                 : root.rowGap
    }
    // The stretch slot: one child, anchored to fill it, that declares its own implicitHeight
    Item {
        id: fill
        anchors.top: body.bottom
        anchors.topMargin: root.stretchItem ? root.gap : 0
        anchors.bottom: footer.top
        anchors.bottomMargin: root.hasFoot ? root.gap : 0
        x: root.pad
        width: root.width - root.pad * 2
    }
    Column {
        id: footer
        anchors.bottom: parent.bottom
        anchors.bottomMargin: root.pad
        x: root.pad
        width: root.width - root.pad * 2
        spacing: theme.space(2)
    }
}
