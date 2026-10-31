// The one-line status strip at the foot of every page, a tile across the content width set
// in the grout, as tall as the rail's foot tile beside it (Frame sets the height): the
// latest activity line with its stage chip and time, a progress line along the tile's top
// edge while a job runs, and the unseen error count at the right. Clicking it toggles the
// activity drawer.
import QtQuick

Item {
    id: root
    property string stage: "scan"
    property string message: ""
    property string time: ""
    property bool running: false
    property real fraction: 0
    property int unseenErrors: 0
    signal clicked()

    height: theme.space(7) + theme.grout

    Corner {
        id: tileBox
        x: 0; y: 0
        width: parent.width - theme.grout
        height: parent.height - theme.grout
        radius: theme.radiusTile
        smoothing: theme.cornerSmoothing
        // Flat and quiet: no pane until the pointer asks for the drawer
        color: mouse.containsMouse ? theme.tileHover : "transparent"

        Rectangle {
            visible: root.running
            anchors.top: parent.top
            x: tileBox.radius
            width: (parent.width - tileBox.radius * 2) * Math.max(0, Math.min(1, root.fraction))
            height: 2
            color: theme.accent
            z: 1
        }

        Row {
            id: line
            anchors.left: parent.left
            anchors.leftMargin: theme.space(4)
            anchors.right: errors.left
            anchors.rightMargin: theme.space(4)
            anchors.verticalCenter: parent.verticalCenter
            spacing: theme.space(2)
            Chip {
                id: stageChip
                visible: root.stage !== ""
                anchors.verticalCenter: parent.verticalCenter
                text: root.stage
                small: true
                color: theme.raised
                textColor: theme.textDim
            }
            Label {
                anchors.verticalCenter: parent.verticalCenter
                width: Math.max(0, Math.min(implicitWidth, line.width - (stageChip.visible ? stageChip.width + line.spacing : 0) - stamp.width - line.spacing))
                text: root.message
                role: "ui"
                elide: Text.ElideRight
            }
            Label {
                id: stamp
                anchors.verticalCenter: parent.verticalCenter
                text: root.time
                color: theme.textFaint
                role: "ledger"
            }
        }
        Chip {
            id: errors
            anchors.right: parent.right
            anchors.rightMargin: theme.space(4)
            anchors.verticalCenter: parent.verticalCenter
            visible: root.unseenErrors > 0
            width: visible ? implicitWidth : 0
            text: root.unseenErrors + (root.unseenErrors === 1 ? " error" : " errors")
            icon: "circle-alert"
            small: true
            color: theme.redSoft
            textColor: theme.red
        }
    }
    MouseArea {
        id: mouse
        anchors.fill: tileBox
        hoverEnabled: true
        cursorShape: Qt.PointingHandCursor
        onClicked: root.clicked()
    }
}
