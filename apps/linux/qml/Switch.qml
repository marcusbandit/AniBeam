// An on/off switch: a square glass track, a square thumb that slides with exponential
// smoothing (theme.followRate) and carries the accent fill with it. Space toggles it; hover and focus show on the border.
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
    color: theme.tone(theme.tile, theme.accentSoft, on)
    borderColor: activeFocus ? theme.focusRing : (hover.containsMouse ? theme.lineStrong : theme.tone(theme.line, Qt.rgba(theme.accent.r, theme.accent.g, theme.accent.b, 0.55), on))
    borderWidth: 1
    activeFocusOnTab: true

    // 0 off, 1 on, smoothed; drives the thumb and both fills
    property real on: 0
    readonly property real onTarget: checked ? 1 : 0
    Component.onCompleted: on = onTarget
    FrameAnimation {
        running: Math.abs(root.on - root.onTarget) > 0.002
        onTriggered: root.on = theme.follow(root.on, root.onTarget, frameTime)
    }

    function toggle() { checked = !checked; toggled(checked) }
    Keys.onSpacePressed: toggle()
    Keys.onReturnPressed: toggle()
    Keys.onEnterPressed: toggle()

    Corner {
        readonly property real size: root.height - root.pad * 2
        x: root.pad + (root.width - root.pad * 2 - size) * root.on
        y: root.pad
        width: size
        height: size
        radius: size / 2
        smoothing: theme.cornerSmoothing
        color: theme.tone(theme.textDim, theme.accent, root.on)
    }
    MouseArea {
        id: hover
        anchors.fill: parent
        hoverEnabled: true
        cursorShape: Qt.PointingHandCursor
        onClicked: { root.forceActiveFocus(); root.toggle() }
    }
}
