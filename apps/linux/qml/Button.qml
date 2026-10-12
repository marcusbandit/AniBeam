// A pill button with a text label and an optional Lucide glyph before it. At rest it is a
// raised pill; `flat` sets it as a resting tile instead (a quieter button on the ground),
// `primary` fills it with the accent under the accent's own ink, and `danger` sets the label
// and the soft fill in red. Hover lifts the fill one step, press one more and scales it to
// 0.98, both with exponential smoothing. Space and Return press it.
import QtQuick

Corner {
    id: root
    property string text: ""
    property string icon: ""
    property bool danger: false
    property bool flat: false
    property bool primary: false
    property bool small: false
    signal clicked()

    implicitWidth: content.implicitWidth + theme.space(small ? 3 : 4) * 2
    implicitHeight: small ? theme.space(6) : theme.controlHeight
    radius: height / 2
    smoothing: theme.cornerSmoothing
    activeFocusOnTab: true
    scale: mouse.pressed ? 0.98 : 1

    // 0 rest, 1 hover, 2 press
    property real lift: 0
    readonly property real liftTarget: mouse.pressed ? 2 : (mouse.containsMouse ? 1 : 0)
    FrameAnimation {
        running: Math.abs(root.lift - root.liftTarget) > 0.002
        onTriggered: root.lift = theme.follow(root.lift, root.liftTarget, frameTime)
    }
    readonly property color restFill: danger ? theme.redSoft : primary ? theme.accent : flat ? theme.tile : theme.raised
    readonly property color hoverFill: danger ? theme.tone(theme.redSoft, theme.red, 0.15)
                                     : primary ? theme.tone(theme.accent, theme.text, 0.15)
                                     : flat ? theme.tileHover : theme.raisedHover
    readonly property color pressFill: danger ? theme.tone(theme.redSoft, theme.red, 0.3)
                                     : primary ? theme.tone(theme.accent, theme.text, 0.25)
                                     : theme.tone(hoverFill, theme.text, 0.06)
    color: lift < 1 ? theme.tone(restFill, hoverFill, lift) : theme.tone(hoverFill, pressFill, lift - 1)
    borderColor: activeFocus ? theme.focusRing : "transparent"
    borderWidth: activeFocus ? theme.space(0.5) : 0

    Keys.onSpacePressed: clicked()
    Keys.onReturnPressed: clicked()
    Keys.onEnterPressed: clicked()

    readonly property color ink: danger ? theme.red : primary ? theme.accentText : (flat && lift < 0.5 ? theme.textDim : theme.text)
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
        Label {
            id: label
            anchors.verticalCenter: parent.verticalCenter
            text: root.text
            color: root.ink
            role: "ui"
            font.pointSize: root.small ? theme.typeSmall : theme.typeNormal
            font.weight: root.primary ? 600 : theme.roleWeight("ui")
            font.variableAxes: ({ "wdth": 100, "wght": root.primary ? 600 : theme.roleWeight("ui") })
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
