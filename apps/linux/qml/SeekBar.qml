// The seek bar: a square groove cut into the controls pane, the played portion in the accent,
// a brighter glass intro band and a quieter outro band where skip windows
// are known, a hover position for the preview. The knob is a square of the
// accent. A press starts a scrub: the knob and the played portion follow the pointer and the
// preview shows the frame, and the release seeks once, wherever it lands, since the pointer
// stays grabbed when a drag drifts off the groove. The main core seeks only on release: a
// live seek per move would read as playback to the auto-skip and the tracker ticks.
import QtQuick

Item {
    id: root
    property real position: 0
    property real duration: 0
    property var windows: []
    // Chapter starts, in seconds: a thin notch cut through the groove at each
    property var chapters: []
    signal seeked(real secs)
    signal hovered(real secs)
    signal unhovered()
    height: theme.space(5)
    // The scrub target while pressed, and after the release until the position reaches it,
    // so the knob does not flick back to where playback was while mpv seeks. -1 when idle.
    property real scrub: -1
    readonly property bool scrubbing: mouse.pressed
    readonly property real played: duration > 0 ? Math.min(1, (scrub >= 0 ? scrub : position) / duration) : 0
    function at(x) { return duration > 0 ? Math.max(0, Math.min(duration, x / width * duration)) : 0 }
    onPositionChanged: if (!mouse.pressed && scrub >= 0 && Math.abs(position - scrub) < 2) scrub = -1
    // A seek mpv clamps or refuses never lands within reach; the knob goes back to the truth.
    Timer { id: settle; interval: 1500; onTriggered: if (!mouse.pressed) root.scrub = -1 }

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
    Repeater {
        model: root.duration > 0 ? root.chapters : []
        Rectangle {
            required property var modelData
            visible: modelData > 1 && modelData < root.duration - 1
            anchors.verticalCenter: parent.verticalCenter
            x: Math.round(modelData / root.duration * root.width) - 1
            width: 2; height: track.height
            color: Qt.rgba(0, 0, 0, 0.7)
        }
    }
    Corner { x: track.width * root.played - width / 2; anchors.verticalCenter: parent.verticalCenter; width: theme.space(1); height: theme.space(4); color: theme.accent; visible: mouse.containsMouse || mouse.pressed }
    MouseArea {
        id: mouse
        objectName: "player-seek"
        anchors.fill: parent
        hoverEnabled: true
        preventStealing: true
        onPressed: function(m) { settle.stop(); root.scrub = root.at(m.x); root.hovered(root.scrub) }
        onPositionChanged: function(m) { if (pressed) root.scrub = root.at(m.x); root.hovered(root.at(m.x)) }
        onReleased: function(m) {
            root.scrub = root.at(m.x)
            root.seeked(root.scrub)
            settle.restart()
            if (!containsMouse) root.unhovered()
        }
        onExited: if (!pressed) root.unhovered()
        onCanceled: { root.scrub = -1; root.unhovered() }
    }
}
