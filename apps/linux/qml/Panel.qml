// A settings section, flat (DESIGN.md, the flat home pass): a Lucide glyph before a heading
// in the display cut, an optional helper line, then its rows on the ground with a hairline
// under each (SettingRow draws its own). No tile around it; the spacing between sections
// is the joint. Three slots: `rows` (the default) flow from the top at their natural
// height, `stretch` holds one item that takes whatever is left when the section is taller
// than its content, and `foot` pins rows to the bottom edge. The section is laid out by the
// ColumnLayout it sits in: it fills the column's width, its minimum is its content height
// so it is never squished, and with `grows` it takes the column's spare height
// (Layout.fillHeight). With `spread` the rows share that spare height as even gaps between
// them, so a growing section of rows is never hollow at its foot.
import QtQuick
import QtQuick.Layouts

Item {
    id: root
    property string title: ""
    property string icon: ""
    property string helper: ""
    property bool grows: false
    property bool spread: false
    default property alias rows: body.data
    property alias stretch: fill.data
    property alias foot: footer.data

    readonly property real pad: theme.space(2)
    readonly property real gap: theme.space(3)
    readonly property Item stretchItem: fill.children.length ? fill.children[0] : null
    readonly property real naturalStretch: stretchItem ? stretchItem.implicitHeight : 0
    readonly property bool hasRows: body.children.length > 0
    readonly property bool hasFoot: footer.children.length > 0
    readonly property real rowGap: theme.space(1)
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

    Column {
        id: head
        x: 0
        y: root.pad
        width: root.width
        spacing: theme.space(1.5)

        Row {
            spacing: theme.space(2.5)
            Icon {
                visible: root.icon !== ""
                anchors.verticalCenter: parent.verticalCenter
                glyph: root.icon
                size: theme.space(5)
                color: theme.text
            }
            Label {
                anchors.verticalCenter: parent.verticalCenter
                text: root.title
                role: "display"
                font.pointSize: theme.typeLarge
                font.variableAxes: ({ "wdth": 74, "wght": 500 })
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
        x: 0
        width: root.width
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
        x: 0
        width: root.width
    }
    Column {
        id: footer
        anchors.bottom: parent.bottom
        anchors.bottomMargin: root.pad
        x: 0
        width: root.width
        spacing: theme.space(2)
    }
}
