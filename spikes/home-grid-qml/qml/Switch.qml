// An on/off switch: a pill track, a round thumb that slides with exponential smoothing and
// carries the accent fill with it. Space toggles it; hover and focus show on the border.
import QtQuick

Corner {
    id: root
    property bool checked: false
    signal toggled(bool on)

    readonly property real pad: theme.space(0.75)
    implicitWidth: theme.space(11)
    implicitHeight: theme.space(6)
    radius: height / 2
    smoothing: theme.cornerSmoothing
    color: theme.tone(theme.surfaceSunken, theme.accent, on)
    borderColor: activeFocus ? theme.focusRing : (hover.containsMouse ? theme.lineStrong : theme.tone(theme.line, theme.accent, on))
    borderWidth: activeFocus ? theme.space(0.5) : 1
    activeFocusOnTab: true

    // 0 off, 1 on, smoothed; drives the thumb and both fills
    property real on: 0
    readonly property real onTarget: checked ? 1 : 0
    Component.onCompleted: on = onTarget
    FrameAnimation {
        running: Math.abs(root.on - root.onTarget) > 0.002
        onTriggered: root.on += (root.onTarget - root.on) * (1 - Math.exp(-14 * frameTime))
    }

    function toggle() { checked = !checked; toggled(checked) }
    Keys.onSpacePressed: toggle()

    Corner {
        readonly property real size: root.height - root.pad * 2
        x: root.pad + (root.width - root.pad * 2 - size) * root.on
        y: root.pad
        width: size
        height: size
        radius: size / 2
        smoothing: theme.cornerSmoothing
        color: theme.tone(theme.textDim, theme.accentText, root.on)
    }
    MouseArea {
        id: hover
        anchors.fill: parent
        hoverEnabled: true
        cursorShape: Qt.PointingHandCursor
        onClicked: { root.forceActiveFocus(); root.toggle() }
    }
}
