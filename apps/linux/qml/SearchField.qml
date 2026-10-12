// The search field: a sunken tile with a hairline and the accent ring on focus, placeholder
// in faint text, a "/  Ctrl K" hint at rest, an X once there is text. Escape clears and
// leaves it, unless a layer is open, which Escape closes first.
import QtQuick

Corner {
    id: root
    property alias text: input.text
    property string placeholder: "Search romaji, english or folder"
    property string hint: "/  Ctrl K"
    signal cleared()
    width: Math.min(parent ? parent.width : theme.space(120), theme.space(120))
    height: theme.controlHeight
    radius: Math.min(theme.radiusTile, height / 2)
    smoothing: theme.cornerSmoothing
    color: theme.tileSunken
    borderColor: input.activeFocus ? theme.focusRing : (hover.containsMouse ? theme.lineStrong : theme.line)
    borderWidth: input.activeFocus ? theme.space(0.5) : 1
    function focusInput() { input.forceActiveFocus(); input.selectAll() }
    MouseArea { id: hover; anchors.fill: parent; hoverEnabled: true; cursorShape: Qt.IBeamCursor; onClicked: input.forceActiveFocus() }
    Icon {
        id: glass
        anchors.left: parent.left; anchors.leftMargin: theme.space(3); anchors.verticalCenter: parent.verticalCenter
        glyph: "search"; size: theme.space(4); color: input.activeFocus ? theme.text : theme.textDim
    }
    TextInput {
        id: input
        anchors.fill: parent
        anchors.leftMargin: theme.space(9); anchors.rightMargin: theme.space(10)
        verticalAlignment: TextInput.AlignVCenter
        color: theme.text
        font.family: theme.fontFamily; font.pointSize: theme.typeNormal
        font.weight: theme.roleWeight("ui"); font.variableAxes: theme.roleAxes("ui")
        selectionColor: theme.accentSoft; selectedTextColor: theme.text
        clip: true
        // With a layer open (a menu, a picker, a sheet) Escape is left to propagate up to
        // the frame, which closes that layer; the query stays.
        Keys.onEscapePressed: function(e) {
            if (frame.escapeStack.top()) { e.accepted = false; return }
            text = ""; focus = false; root.cleared()
        }
        Text { anchors.fill: parent; verticalAlignment: Text.AlignVCenter; visible: !input.text; text: root.placeholder; color: theme.textFaint; font: input.font }
    }
    Label {
        anchors.right: parent.right; anchors.rightMargin: theme.space(3); anchors.verticalCenter: parent.verticalCenter
        visible: !input.activeFocus && !input.text
        text: root.hint; color: theme.textFaint; role: "ledger"
    }
    Icon {
        visible: input.text !== ""
        anchors.right: parent.right; anchors.rightMargin: theme.space(3); anchors.verticalCenter: parent.verticalCenter
        glyph: "x"; size: theme.space(4); color: theme.textDim
        MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: { input.text = ""; root.cleared() } }
    }
}
