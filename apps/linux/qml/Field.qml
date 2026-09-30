// A one line text field in a pill: placeholder in faint text, focus on the border,
// `mono` sets the fixed face for numbers and codes.
import QtQuick

Corner {
    id: root
    property alias text: input.text
    property string placeholder: ""
    property bool mono: false
    signal edited(string text)

    implicitWidth: theme.space(40)
    implicitHeight: theme.controlHeight
    radius: height / 2
    smoothing: theme.cornerSmoothing
    color: theme.surfaceSunken
    borderColor: input.activeFocus ? theme.focusRing : (hover.containsMouse ? theme.lineStrong : theme.line)
    borderWidth: input.activeFocus ? theme.space(0.5) : 1

    function focusInput() { input.forceActiveFocus() }

    MouseArea {
        id: hover
        anchors.fill: parent
        hoverEnabled: true
        cursorShape: Qt.IBeamCursor
        onClicked: input.forceActiveFocus()
    }
    TextInput {
        id: input
        anchors.fill: parent
        anchors.leftMargin: theme.space(4)
        anchors.rightMargin: theme.space(4)
        verticalAlignment: TextInput.AlignVCenter
        color: theme.text
        font.family: root.mono ? theme.fontMono : theme.fontSans
        font.pointSize: theme.typeNormal
        selectionColor: theme.accentSoft
        selectedTextColor: theme.text
        clip: true
        onEditingFinished: root.edited(text)
        Text {
            anchors.fill: parent
            verticalAlignment: Text.AlignVCenter
            visible: !input.text
            text: root.placeholder
            color: theme.textFaint
            font: input.font
        }
    }
}
