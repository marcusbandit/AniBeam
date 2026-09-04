// The one-line status strip at the foot of every page: the latest activity line with its
// stage chip and time, a progress line along the top edge while a job runs, and the unseen
// error count at the right. Clicking it toggles the activity drawer.
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

    height: theme.space(7)

    Rectangle { anchors.fill: parent; color: theme.surfaceSunken }
    Rectangle { anchors.top: parent.top; width: parent.width; height: 1; color: theme.line }
    Rectangle {
        visible: root.running
        anchors.top: parent.top
        width: parent.width * Math.max(0, Math.min(1, root.fraction))
        height: theme.space(0.5)
        color: theme.accent
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
            anchors.verticalCenter: parent.verticalCenter
            text: root.stage
            small: true
            color: theme.surface
            textColor: theme.textDim
        }
        Text {
            anchors.verticalCenter: parent.verticalCenter
            width: Math.max(0, Math.min(implicitWidth, line.width - stageChip.width - stamp.width - line.spacing * 2))
            text: root.message
            color: theme.text
            elide: Text.ElideRight
            font.family: theme.fontSans
            font.pointSize: theme.typeSmall
        }
        Text {
            id: stamp
            anchors.verticalCenter: parent.verticalCenter
            text: root.time
            color: theme.textFaint
            font.family: theme.fontMono
            font.pointSize: theme.typeSmall
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
    MouseArea {
        anchors.fill: parent
        cursorShape: Qt.PointingHandCursor
        onClicked: root.clicked()
    }
}
