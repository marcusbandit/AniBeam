// The seek bar: the played portion, an amber intro band and a teal outro band where skip
// windows are known, a hover position for the preview. Dragging seeks on release, since a
// MouseArea's click is its release and the drag only moves the hover.
import QtQuick

Item {
    id: root
    property real position: 0
    property real duration: 0
    property var windows: []
    property real hoverAt: -1
    signal seeked(real secs)
    signal hovered(real secs)
    signal unhovered()
    height: theme.space(5)
    readonly property real played: duration > 0 ? Math.min(1, position / duration) : 0
    function at(x) { return duration > 0 ? Math.max(0, Math.min(duration, x / width * duration)) : 0 }

    Corner { id: track; anchors.verticalCenter: parent.verticalCenter; width: parent.width; height: theme.space(1.25); radius: height / 2; smoothing: theme.cornerSmoothing; color: theme.surfaceSunken; borderColor: theme.line; borderWidth: 1 }
    Repeater {
        model: root.windows
        Corner {
            required property var modelData
            anchors.verticalCenter: parent.verticalCenter
            x: root.duration > 0 ? modelData.start / root.duration * root.width : 0
            width: root.duration > 0 ? Math.max(2, (modelData.end - modelData.start) / root.duration * root.width) : 0
            height: track.height; radius: height / 2; smoothing: theme.cornerSmoothing
            color: modelData.kind === "Intro" ? theme.yellow : theme.cyan
            opacity: 0.7
        }
    }
    Corner { anchors.verticalCenter: parent.verticalCenter; width: track.width * root.played; height: track.height; radius: height / 2; smoothing: theme.cornerSmoothing; color: theme.accent }
    Corner { x: track.width * root.played - width / 2; anchors.verticalCenter: parent.verticalCenter; width: theme.space(3.5); height: width; radius: width / 2; smoothing: theme.cornerSmoothing; color: theme.accent; borderColor: theme.bg; borderWidth: theme.space(0.5); visible: mouse.containsMouse || mouse.pressed }
    MouseArea {
        id: mouse
        anchors.fill: parent
        hoverEnabled: true
        onPositionChanged: function(m) { root.hoverAt = root.at(m.x); root.hovered(root.hoverAt) }
        onExited: { root.hoverAt = -1; root.unhovered() }
        onClicked: function(m) { root.seeked(root.at(m.x)) }
    }
}
