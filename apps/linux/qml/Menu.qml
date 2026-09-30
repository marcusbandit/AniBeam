// The frame's menu: rows of text and a glyph on a raised surface. Opened by the frame at a
// point; closes on a pick, a click outside, or Escape through the escape stack.
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

    MouseArea { anchors.fill: parent; acceptedButtons: Qt.LeftButton | Qt.RightButton; onPressed: root.close() }
    Corner {
        readonly property real margin: theme.space(2)
        x: Math.min(root.originX, root.width - width - margin)
        y: Math.min(root.originY, root.height - height - margin)
        width: column.implicitWidth + theme.space(2) * 2
        height: column.implicitHeight + theme.space(2) * 2
        radius: theme.radiusMd
        smoothing: theme.cornerSmoothing
        color: theme.surfaceRaised
        borderColor: theme.lineStrong
        borderWidth: 1
        Column {
            id: column
            x: theme.space(2); y: theme.space(2)
            Repeater {
                model: root.items
                Corner {
                    required property var modelData
                    width: Math.max(theme.space(40), row.implicitWidth + theme.space(6))
                    height: theme.controlHeight
                    radius: theme.radiusSm
                    smoothing: theme.cornerSmoothing
                    color: m.containsMouse ? theme.surfacePressed : "transparent"
                    Row {
                        id: row
                        anchors.left: parent.left; anchors.leftMargin: theme.space(3)
                        anchors.verticalCenter: parent.verticalCenter
                        spacing: theme.space(2)
                        Icon { visible: !!modelData.icon; glyph: modelData.icon || ""; anchors.verticalCenter: parent.verticalCenter; size: theme.space(4) }
                        Text { text: modelData.text; color: theme.text; font.family: theme.fontSans; font.pointSize: theme.typeNormal; anchors.verticalCenter: parent.verticalCenter }
                    }
                    MouseArea { id: m; anchors.fill: parent; hoverEnabled: true; cursorShape: Qt.PointingHandCursor
                        onClicked: { root.close(); if (modelData.action) modelData.action() } }
                }
            }
        }
    }
}
