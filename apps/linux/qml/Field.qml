// A one line text field: a pane of glass with its edge, placeholder in faint text, the
// edge in the accent on focus. `mono` sets the ledger cut, tabular figures for numbers and codes.
import QtQuick

Corner {
    id: root
    property alias text: input.text
    property string placeholder: ""
    property bool mono: false
    // A password: the characters are hidden as typed
    property bool secret: false
    signal edited(string text)

    implicitWidth: theme.space(40)
    implicitHeight: theme.controlHeight
    radius: Math.min(theme.radiusTile, height / 2)
    smoothing: theme.cornerSmoothing
    color: input.activeFocus ? theme.raised : theme.tile
    // Focus is the accent edge
    borderColor: input.activeFocus ? theme.focusRing : hover.containsMouse ? theme.lineStrong : theme.line
    borderWidth: 1

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
        anchors.leftMargin: theme.space(3)
        anchors.rightMargin: theme.space(3)
        verticalAlignment: TextInput.AlignVCenter
        echoMode: root.secret ? TextInput.Password : TextInput.Normal
        color: theme.text
        font.family: theme.fontFamily
        font.pointSize: root.mono ? theme.typeSmall : theme.typeNormal
        font.weight: theme.roleWeight(root.mono ? "ledger" : "ui")
        font.variableAxes: theme.roleAxes(root.mono ? "ledger" : "ui")
        font.features: theme.roleFeatures(root.mono ? "ledger" : "ui")
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
