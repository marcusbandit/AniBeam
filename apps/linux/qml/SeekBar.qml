// The seek bar: a square groove cut into the controls pane, the played portion in the accent,
// a brighter glass intro band and a quieter outro band where skip windows
// are known, a hover position for the preview. The knob is a square of the
// accent. Dragging seeks on release, since a
// MouseArea's click is its release and the drag only moves the hover.
import QtQuick

Item {
    id: root
    property real position: 0
    property real duration: 0
    property var windows: []
    signal seeked(real secs)
    signal hovered(real secs)
    signal unhovered()
    height: theme.space(5)
    readonly property real played: duration > 0 ? Math.min(1, position / duration) : 0
    function at(x) { return duration > 0 ? Math.max(0, Math.min(duration, x / width * duration)) : 0 }

    Corner { id: track; anchors.verticalCenter: parent.verticalCenter; width: parent.width; height: theme.space(1); color: Qt.rgba(1, 1, 1, 0.14) }
    Repeater {
        model: root.windows
        Corner {
            required property var modelData
            anchors.verticalCenter: parent.verticalCenter
            x: root.duration > 0 ? modelData.start / root.duration * root.width : 0
            width: root.duration > 0 ? Math.max(2, (modelData.end - modelData.start) / root.duration * root.width) : 0
            height: track.height
            color: "#ffffff"
            opacity: modelData.kind === "Intro" ? 0.38 : 0.24
        }
    }
    Corner { anchors.verticalCenter: parent.verticalCenter; width: track.width * root.played; height: track.height; color: theme.accent }
    Corner { x: track.width * root.played - width / 2; anchors.verticalCenter: parent.verticalCenter; width: theme.space(1); height: theme.space(4); color: theme.accent; visible: mouse.containsMouse || mouse.pressed }
    MouseArea {
        id: mouse
        objectName: "player-seek"
        anchors.fill: parent
        hoverEnabled: true
        onPositionChanged: function(m) { root.hovered(root.at(m.x)) }
        onExited: root.unhovered()
        onClicked: function(m) { root.seeked(root.at(m.x)) }
    }
}
