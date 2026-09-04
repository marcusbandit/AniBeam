// A pill button with a text label. Hover lifts the fill one surface step, press one more,
// both with exponential smoothing. `flat` draws no fill at rest (a text button), `danger`
// sets the label and the soft fill in red. Space and Return press it.
import QtQuick

Corner {
    id: root
    property string text: ""
    property bool danger: false
    property bool flat: false
    property bool small: false
    signal clicked()

    implicitWidth: label.implicitWidth + theme.space(small ? 3 : 4) * 2
    implicitHeight: small ? theme.space(6) : theme.controlHeight
    radius: height / 2
    smoothing: theme.cornerSmoothing
    activeFocusOnTab: true

    // 0 rest, 1 hover, 2 press
    property real lift: 0
    readonly property real liftTarget: mouse.pressed ? 2 : (mouse.containsMouse ? 1 : 0)
    FrameAnimation {
        running: Math.abs(root.lift - root.liftTarget) > 0.002
        onTriggered: root.lift += (root.liftTarget - root.lift) * (1 - Math.exp(-18 * frameTime))
    }
    readonly property color restFill: danger ? theme.redSoft : (flat ? "transparent" : theme.surface)
    readonly property color hoverFill: danger ? theme.tone(theme.redSoft, theme.red, 0.15) : (flat ? theme.surface : theme.surfaceRaised)
    readonly property color pressFill: danger ? theme.tone(theme.redSoft, theme.red, 0.3) : (flat ? theme.surfaceRaised : theme.surfacePressed)
    color: lift < 1 ? theme.tone(restFill, hoverFill, lift) : theme.tone(hoverFill, pressFill, lift - 1)
    borderColor: activeFocus ? theme.focusRing : (flat ? "transparent" : theme.line)
    borderWidth: activeFocus ? theme.space(0.5) : (flat ? 0 : 1)

    Keys.onSpacePressed: clicked()
    Keys.onReturnPressed: clicked()
    Keys.onEnterPressed: clicked()

    Text {
        id: label
        anchors.centerIn: parent
        text: root.text
        color: root.danger ? theme.red : (root.flat && root.lift < 0.5 ? theme.textDim : theme.text)
        font.family: theme.fontSans
        font.pointSize: root.small ? theme.typeSmall : theme.typeNormal
        font.weight: Font.Medium
    }
    MouseArea {
        id: mouse
        anchors.fill: parent
        hoverEnabled: true
        cursorShape: Qt.PointingHandCursor
        onClicked: { root.forceActiveFocus(); root.clicked() }
    }
}
