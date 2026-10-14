// A button that asks twice, in place. The first click arms it: it fills red and its label
// turns to Confirm. The second click does the thing. A click anywhere else, Escape, focus
// moving on or the button leaving the screen disarms it, so one stray click removes
// nothing and no question pushes the row's own text aside. The frame catches the click
// that lands elsewhere (Frame.armConfirm) and lets it through to whatever it hit.
//
// The width holds still between the two labels: the wider of them sets it.
import QtQuick

Button {
    id: root
    property string label: ""
    property string confirmText: "Confirm"
    property bool armed: false
    // At rest on art (a poster), white on faint glass; armed it is red like any other
    property bool onArt: false
    signal confirmed()

    text: armed ? confirmText : label
    primary: armed
    tint: theme.red
    flat: !armed && !onArt
    ghost: !armed && onArt
    implicitWidth: Math.max(restText.implicitWidth, armedText.implicitWidth)
                   + (icon !== "" ? theme.space(small ? 3.5 : 4.5) + theme.space(1.5) : 0)
                   + theme.space(small ? 3 : 4) * 2

    function arm() { if (armed) return; armed = true; frame.armConfirm(root) }
    function disarm() { if (!armed) return; armed = false; frame.disarmConfirm(root) }
    // The escape stack's closer
    function close() { disarm() }

    onClicked: { if (armed) { disarm(); confirmed() } else arm() }
    onActiveFocusChanged: if (!activeFocus && armed && takesFocus) disarm()
    onVisibleChanged: if (!visible) disarm()
    onEnabledChanged: if (!enabled) disarm()
    Component.onDestruction: frame.disarmConfirm(root)

    Label { id: restText; visible: false; text: root.label; role: "ui"; font.pointSize: root.small ? theme.typeSmall : theme.typeNormal }
    Label { id: armedText; visible: false; text: root.confirmText; role: "ui"; font.pointSize: root.small ? theme.typeSmall : theme.typeNormal; font.weight: 600; font.variableAxes: ({ "wdth": 100, "wght": 600 }) }
}
