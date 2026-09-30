// The pill search field: placeholder in faint text, a "/  Ctrl K" hint at rest, an X once
// there is text. Escape clears and leaves it.
import QtQuick

Corner {
    id: root
    property alias text: input.text
    property string placeholder: "Search romaji, english or folder"
    property string hint: "/  Ctrl K"
    signal cleared()
    width: Math.min(parent ? parent.width : theme.space(120), theme.space(120))
    height: theme.controlHeight
    radius: height / 2
    smoothing: theme.cornerSmoothing
    color: theme.surfaceSunken
    borderColor: input.activeFocus ? theme.focusRing : theme.line
    borderWidth: 1
    function focusInput() { input.forceActiveFocus(); input.selectAll() }
    TextInput {
        id: input
        anchors.fill: parent
        anchors.leftMargin: theme.space(4); anchors.rightMargin: theme.space(10)
        verticalAlignment: TextInput.AlignVCenter
        color: theme.text
        font.family: theme.fontSans; font.pointSize: theme.typeNormal
        selectionColor: theme.accentSoft; selectedTextColor: theme.text
        clip: true
        Keys.onEscapePressed: { text = ""; focus = false; root.cleared() }
        Text { anchors.fill: parent; verticalAlignment: Text.AlignVCenter; visible: !input.text; text: root.placeholder; color: theme.textFaint; font: input.font }
    }
    Text {
        anchors.right: parent.right; anchors.rightMargin: theme.space(4); anchors.verticalCenter: parent.verticalCenter
        visible: !input.activeFocus && !input.text
        text: root.hint; color: theme.textFaint; font.family: theme.fontMono; font.pointSize: theme.typeSmall
    }
    Icon {
        visible: input.text !== ""
        anchors.right: parent.right; anchors.rightMargin: theme.space(3); anchors.verticalCenter: parent.verticalCenter
        glyph: "x"; size: theme.space(4); color: theme.textDim
        MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: { input.text = ""; root.cleared() } }
    }
}
