// A square button with a text label and an optional Lucide glyph before it. At rest it is a
// raised pane of glass (edge and top highlight); `flat` sets it as resting glass without the
// highlight (a quieter button), `primary` fills it with the accent under ground-coloured
// ink, and `danger` sets the label and the soft fill in red. Hover lifts the fill one step, press one more and scales it to
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
    // False for a button that must never hold the keyboard: the player's, where a focused
    // button would swallow Space and Return before the page's own keys see them, and Tab
    // would walk a focus ring through the island.
    property bool takesFocus: true
    signal clicked()

    implicitWidth: content.implicitWidth + theme.space(small ? 3 : 4) * 2
    implicitHeight: small ? theme.space(6) : theme.controlHeight
    radius: height / 2
    smoothing: theme.cornerSmoothing
    activeFocusOnTab: takesFocus
    scale: mouse.pressed ? 0.98 : 1
    // A disabled button takes no clicks (its MouseArea inherits `enabled`); this says so.
    opacity: enabled ? 1 : 0.45

    // 0 rest, 1 hover, 2 press
    property real lift: 0
    readonly property real liftTarget: mouse.pressed ? 2 : (mouse.containsMouse ? 1 : 0)
    FrameAnimation {
        running: Math.abs(root.lift - root.liftTarget) > 0.002
        onTriggered: root.lift = theme.follow(root.lift, root.liftTarget, frameTime)
    }
    readonly property color restFill: danger ? theme.redSoft : primary ? theme.accent : flat ? theme.tile : theme.raised
    readonly property color hoverFill: danger ? theme.tone(theme.redSoft, theme.red, 0.15)
                                     : primary ? theme.tone(theme.accent, theme.ink, 0.15)
                                     : flat ? theme.tileHover : theme.raisedHover
    readonly property color pressFill: danger ? theme.tone(theme.redSoft, theme.red, 0.3)
                                     : primary ? theme.tone(theme.accent, theme.ink, 0.25)
                                     : theme.glassOf(theme.dark ? 0.12 : 0.15)
    color: lift < 1 ? theme.tone(restFill, hoverFill, lift) : theme.tone(hoverFill, pressFill, lift - 1)
    borderColor: activeFocus ? theme.focusRing : primary ? "transparent" : danger ? Qt.rgba(theme.red.r, theme.red.g, theme.red.b, 0.3) : theme.line
    borderWidth: activeFocus ? 2 : 1
    highlight: !flat && !primary && !danger ? theme.glassHighlight : "transparent"

    Keys.onSpacePressed: clicked()
    Keys.onReturnPressed: clicked()
    Keys.onEnterPressed: clicked()

    readonly property color ink: danger ? theme.red : primary ? (theme.dark ? theme.ground : theme.accentText) : (flat && lift < 0.5 ? theme.textDim : theme.text)
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
            // An icon-only button has no label, and no spacing after its glyph either.
            visible: root.text !== ""
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
        onClicked: { if (root.takesFocus) root.forceActiveFocus(); root.clicked() }
    }
}
