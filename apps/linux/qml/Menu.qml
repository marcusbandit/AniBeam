// The frame's menu: rows of text and a glyph on a raised tile. Every row is as wide as the
// widest one, so the hover highlight is one even column. Opened by the frame at a point;
// closes on a pick, a click outside, or Escape through the escape stack.
import QtQuick

Item {
    id: root
    property var items: []
    property bool open: false
    property real originX: 0
    property real originY: 0
    visible: open
    anchors.fill: parent
    z: 1000

    function openAt(x, y, list) {
        items = list; originX = x; originY = y; open = true
        frame.escapeStack.push("popover", root)
    }
    function close() { if (!open) return; open = false; frame.escapeStack.pop(root) }

    readonly property real pad: theme.space(1.5)
    readonly property real rowWidth: {
        var w = theme.space(40)
        for (var i = 0; i < rows.count; i++) {
            var row = rows.itemAt(i)
            if (row) w = Math.max(w, row.contentWidth + theme.space(6))
        }
        return w
    }

    MouseArea { anchors.fill: parent; acceptedButtons: Qt.LeftButton | Qt.RightButton; onPressed: root.close() }
    Frost {
        readonly property real margin: theme.space(2)
        x: Math.min(root.originX, root.width - width - margin)
        y: Math.min(root.originY, root.height - height - margin)
        width: root.rowWidth + root.pad * 2
        height: column.implicitHeight + root.pad * 2
        radius: theme.radiusTile
        Column {
            id: column
            x: root.pad; y: root.pad
            Repeater {
                id: rows
                model: root.items
                Corner {
                    required property var modelData
                    readonly property real contentWidth: row.implicitWidth
                    width: root.rowWidth
                    height: theme.controlHeight
                    radius: theme.radiusInner(root.pad)
                    smoothing: theme.cornerSmoothing
                    color: m.containsMouse ? theme.raisedHover : "transparent"
                    Row {
                        id: row
                        anchors.left: parent.left; anchors.leftMargin: theme.space(3)
                        anchors.verticalCenter: parent.verticalCenter
                        spacing: theme.space(2)
                        Icon { visible: !!modelData.icon; glyph: modelData.icon || ""; anchors.verticalCenter: parent.verticalCenter; size: theme.space(4); color: m.containsMouse ? theme.text : theme.textDim }
                        Label { text: modelData.text; role: "ui"; anchors.verticalCenter: parent.verticalCenter }
                    }
                    MouseArea { id: m; anchors.fill: parent; hoverEnabled: true; cursorShape: Qt.PointingHandCursor
                        onClicked: { root.close(); if (modelData.action) modelData.action() } }
                }
            }
        }
    }
}
