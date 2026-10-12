// The frame's menu: rows of text and a glyph on a raised tile. Every row is as wide as the
// widest one, so the hover highlight is one even column. Opened by the frame at a point;
// closes on a pick, a click outside, or Escape through the escape stack. Up, Down, Home
// and End move the highlight, Return picks; it takes key focus while open and gives it back to
// whatever had it.
import QtQuick

Item {
    id: root
    objectName: "frame-menu"
    property var items: []
    property bool open: false
    property real originX: 0
    property real originY: 0
    // The keyboard highlight; -1 until Up or Down, and the hovered row follows the pointer
    property int hi: -1
    visible: open
    anchors.fill: parent
    z: 1000

    property Item prevFocus: null
    function openAt(x, y, list) {
        if (!open) prevFocus = frame.hostWindow ? frame.hostWindow.activeFocusItem : null
        items = list; originX = x; originY = y; hi = -1; open = true
        frame.escapeStack.push("popover", root)
        root.forceActiveFocus()
    }
    function close() { if (!open) return; open = false; frame.escapeStack.pop(root); frame.focusPage(root, prevFocus); prevFocus = null }
    // The mouse and the keyboard share one path: close first, so an action that navigates
    // or opens another layer starts from a closed menu.
    function pick(i) {
        if (i < 0 || i >= items.length) return
        var action = items[i].action
        close()
        if (action) action()
    }
    // Escape is not handled here: it propagates to the frame, which closes the top entry.
    Keys.onUpPressed: hi = hi <= 0 ? items.length - 1 : hi - 1
    Keys.onDownPressed: hi = (hi + 1) % items.length
    Keys.onPressed: function(e) {
        if (e.key === Qt.Key_Home) { hi = 0; e.accepted = true }
        else if (e.key === Qt.Key_End) { hi = items.length - 1; e.accepted = true }
    }
    Keys.onReturnPressed: pick(hi)
    Keys.onEnterPressed: pick(hi)

    readonly property real pad: theme.space(1.5)
    readonly property real rowWidth: {
        var w = theme.space(40)
        for (var i = 0; i < rows.count; i++) {
            var row = rows.itemAt(i)
            if (row) w = Math.max(w, row.contentWidth + theme.space(6))
        }
        return w
    }

    // The shield: a press of any button outside the panel closes the menu, and neither it
    // nor a wheel or a hover reaches the page under the menu.
    Blocker { anchors.fill: parent; scrim: false; dismissOnPress: true; onDismissed: root.close() }
    Frost {
        id: panel
        objectName: "frame-menu-panel"
        readonly property real margin: theme.space(2)
        // Clamped both ways, so a window narrower or shorter than the menu never pushes it
        // off the top or left edge
        x: Math.max(margin, Math.min(root.originX, root.width - width - margin))
        y: Math.max(margin, Math.min(root.originY, root.height - height - margin))
        width: root.rowWidth + root.pad * 2
        height: column.implicitHeight + root.pad * 2
        radius: theme.radiusTile
        // A step up from what it opens over: the strong hairline always, and the float
        // fill when it opens over the drawer, itself a raised surface
        readonly property bool overRaised: {
            var d = frame.drawerPanel
            if (!root.open || !d.open) return false
            var p = d.mapToItem(root, 0, 0)
            return root.originY >= p.y && root.originX >= p.x && root.originX <= p.x + d.width
        }
        color: overRaised ? theme.raisedFloat : theme.raised
        borderColor: theme.lineStrong
        // Takes presses on the padding and the gaps between rows, which would otherwise
        // reach the shield and close the menu; the rows are declared after it, on top.
        MouseArea { anchors.fill: parent; acceptedButtons: Qt.AllButtons }
        Column {
            id: column
            x: root.pad; y: root.pad
            Repeater {
                id: rows
                model: root.items
                Corner {
                    required property var modelData
                    required property int index
                    objectName: "frame-menu-row-" + index
                    readonly property bool highlighted: index === root.hi
                    readonly property real contentWidth: row.implicitWidth
                    width: root.rowWidth
                    height: theme.controlHeight
                    radius: theme.radiusInner(root.pad)
                    smoothing: theme.cornerSmoothing
                    color: highlighted ? theme.tone(panel.color, theme.text, 0.07) : "transparent"
                    Row {
                        id: row
                        anchors.left: parent.left; anchors.leftMargin: theme.space(3)
                        anchors.verticalCenter: parent.verticalCenter
                        spacing: theme.space(2)
                        Icon { visible: !!modelData.icon; glyph: modelData.icon || ""; anchors.verticalCenter: parent.verticalCenter; size: theme.space(4); color: highlighted ? theme.text : theme.textDim }
                        Label { text: modelData.text; role: "ui"; anchors.verticalCenter: parent.verticalCenter }
                    }
                    MouseArea { id: m; anchors.fill: parent; hoverEnabled: true; cursorShape: Qt.PointingHandCursor
                        onEntered: root.hi = index; onClicked: root.pick(index) }
                }
            }
        }
    }
}
