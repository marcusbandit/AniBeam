// A pill button with a text label and an optional Lucide glyph before it. Hover lifts the
// fill one surface step, press one more, both with exponential smoothing. `flat` draws no
// fill at rest (a text button), `danger` sets the label and the soft fill in red. Space and
// Return press it.
import QtQuick

Corner {
    id: root
    property string text: ""
    property string icon: ""
    property bool danger: false
    property bool flat: false
    property bool small: false
    signal clicked()

    implicitWidth: content.implicitWidth + theme.space(small ? 3 : 4) * 2
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
    // A step above the panel it sits on, so it reads as a button on a surface as well as on bg
    readonly property color restFill: danger ? theme.redSoft : (flat ? "transparent" : theme.surfaceRaised)
    readonly property color hoverFill: danger ? theme.tone(theme.redSoft, theme.red, 0.15) : (flat ? theme.surfaceRaised : theme.surfacePressed)
    readonly property color pressFill: danger ? theme.tone(theme.redSoft, theme.red, 0.3) : (flat ? theme.surfacePressed : theme.tone(theme.surfacePressed, theme.text, 0.08))
    color: lift < 1 ? theme.tone(restFill, hoverFill, lift) : theme.tone(hoverFill, pressFill, lift - 1)
    borderColor: activeFocus ? theme.focusRing : (flat ? "transparent" : theme.line)
    borderWidth: activeFocus ? theme.space(0.5) : (flat ? 0 : 1)

    Keys.onSpacePressed: clicked()
    Keys.onReturnPressed: clicked()
    Keys.onEnterPressed: clicked()

    readonly property color ink: danger ? theme.red : (flat && lift < 0.5 ? theme.textDim : theme.text)
    Row {
        id: content
        anchors.centerIn: parent
        spacing: theme.space(1.5)
        Icon {
            visible: root.icon !== ""
            anchors.verticalCenter: parent.verticalCenter
            glyph: root.icon
            color: root.ink
            size: theme.space(root.small ? 3.5 : 4.5)
        }
        Text {
            id: label
            anchors.verticalCenter: parent.verticalCenter
            text: root.text
            color: root.ink
            font.family: theme.fontSans
            font.pointSize: root.small ? theme.typeSmall : theme.typeNormal
            font.weight: Font.Medium
        }
    }
    MouseArea {
        id: mouse
        anchors.fill: parent
        hoverEnabled: true
        cursorShape: Qt.PointingHandCursor
        onClicked: { root.forceActiveFocus(); root.clicked() }
    }
}
