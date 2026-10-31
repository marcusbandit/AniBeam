// A settings group: a heading on the ground in the title cut, an optional helper line
// under it, then one flat pane (`theme.tile`, the tile radius) holding the rows, each
// `SettingRow` closed by a hairline except the last. One surface per group and dividers
// inside it, never a pane in a pane. Three slots: `rows` (the default) flow from the top
// at their natural height, `stretch` holds one item that takes whatever is left when the
// group is taller than its content, and `foot` pins rows to the pane's bottom edge.
// `bare` drops the pane for a group whose content draws its own surfaces (the previews).
// The group is laid out by the ColumnLayout it sits in: it fills the column's width, its
// minimum is its content height, and with `grows` it takes the column's spare height.
// `icon` is accepted for old callers and not drawn: the settings index carries the glyphs.
import QtQuick
import QtQuick.Layouts

Item {
    id: root
    property string title: ""
    property string icon: ""
    property string helper: ""
    property bool grows: false
    property bool spread: false
    property bool bare: false
    // Search lands here when its words name the group rather than one row
    readonly property bool settingTarget: true
    property real flash: 0
    function pulse() { flashAnim.restart() }
    SequentialAnimation {
        id: flashAnim
        NumberAnimation { target: root; property: "flash"; to: 1; duration: 160 }
        PauseAnimation { duration: 700 }
        NumberAnimation { target: root; property: "flash"; to: 0; duration: 600 }
    }
    default property alias rows: body.data
    property alias stretch: fill.data
    property alias foot: footer.data

    // The pane's side padding, shared with the rows so their hairlines run inside it
    readonly property real inset: bare ? 0 : theme.space(5)
    readonly property real pad: bare ? 0 : theme.space(1.5)
    readonly property real gap: theme.space(3)
    readonly property Item stretchItem: fill.children.length ? fill.children[0] : null
    readonly property real naturalStretch: stretchItem ? stretchItem.implicitHeight : 0
    readonly property bool hasRows: laidRows.n > 0
    readonly property bool hasFoot: footer.children.length > 0
    readonly property bool hasHead: title !== "" || helper !== ""
    // The rows' natural height read off the children, not body.height, so `spread` can
    // widen the gaps without feeding back into the height it spreads over. A Column lays
    // out only visible children with a size, so only those count; the last of them is the
    // row that draws no hairline.
    readonly property var laidRows: {
        var n = 0, sum = 0, last = null
        for (var i = 0; i < body.children.length; i++) {
            var c = body.children[i]
            if (!c.visible || c.height <= 0 || c.width <= 0) continue
            n++; sum += c.height; last = c
        }
        return { n: n, sum: sum, last: last }
    }
    readonly property real bodyNatural: laidRows.sum
    readonly property real paneNatural: pad * 2 + (hasRows ? bodyNatural : 0)
                                        + (stretchItem ? (hasRows ? gap : 0) + naturalStretch : 0)
                                        + (hasFoot ? gap + footer.height : 0)

    implicitHeight: (hasHead ? head.height + theme.space(3) : 0) + paneNatural
    Layout.fillWidth: true
    Layout.fillHeight: grows
    Layout.minimumHeight: implicitHeight
    Layout.preferredHeight: implicitHeight

    Column {
        id: head
        visible: root.hasHead
        width: root.width
        spacing: theme.space(1)
        Label {
            visible: root.title !== ""
            text: root.title
            role: "display"
            font.pointSize: theme.typeLarge
            font.variableAxes: ({ "wdth": 74, "wght": 500 })
            color: root.flash > 0 ? Qt.tint(theme.text, Qt.rgba(theme.accent.r, theme.accent.g, theme.accent.b, root.flash)) : theme.text
        }
        Label {
            visible: root.helper !== ""
            width: Math.min(parent.width, theme.space(190))
            text: root.helper
            color: theme.textDim
            wrapMode: Text.Wrap
            role: "ui"
            font.pointSize: theme.typeSmall
        }
    }
    Rectangle {
        id: pane
        y: root.hasHead ? head.height + theme.space(3) : 0
        width: root.width
        height: root.height - y
        radius: theme.radiusTile
        color: root.bare ? "transparent" : theme.tile
        border.width: root.flash > 0 ? 1 : 0
        border.color: Qt.rgba(theme.accent.r, theme.accent.g, theme.accent.b, root.flash)

        Column {
            id: body
            // The row that closes the pane: SettingRow reads it to leave its hairline off
            readonly property Item lastRow: root.laidRows.last
            x: root.inset
            y: root.pad
            width: pane.width - root.inset * 2
            spacing: root.spread && root.laidRows.n > 1 && !root.stretchItem
                     ? Math.max(0, root.height - root.implicitHeight) / (root.laidRows.n - 1)
                     : 0
        }
        // The stretch slot: one child, anchored to fill it, that declares its own implicitHeight
        Item {
            id: fill
            x: root.inset
            y: body.y + (root.hasRows ? body.height + root.gap : 0)
            width: pane.width - root.inset * 2
            height: Math.max(0, footer.y - (root.hasFoot ? root.gap : 0) - y)
        }
        Column {
            id: footer
            x: root.inset
            y: pane.height - root.pad - height
            width: pane.width - root.inset * 2
            spacing: theme.space(2)
        }
    }
}
