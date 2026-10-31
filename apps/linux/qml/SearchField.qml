// The search field: a square pane of glass with the glass edge and an accent line along its foot on focus, placeholder
// in faint text, a "/ or Ctrl+K" hint at rest, an X once there is text. Escape clears and
// leaves it, unless a layer is open, which Escape closes first.
import QtQuick

Corner {
    id: root
    property alias text: input.text
    property string placeholder: "Search titles or folders"
    property string hint: "/ or Ctrl+K"
    signal cleared()
    // Return or Enter in the field: a page that searches on demand rather than as you type
    signal accepted()
    // The Downloads page's one wide field: taller, its text at the large size
    property bool large: false
    // A round search button inside the field's right end, for a field whose Return runs a
    // search worth a button of its own (the Downloads page's release search)
    property bool goButton: false
    property bool goBusy: false
    property string goTip: ""
    signal go()
    readonly property bool editing: input.activeFocus
    // Up and Down in the field, for a list of suggestions under it
    signal stepped(int by)
    width: Math.min(parent ? parent.width : theme.space(120), theme.space(120))
    height: large ? theme.space(15) : theme.controlHeight
    radius: Math.min(theme.radiusTile, height / 2)
    smoothing: theme.cornerSmoothing
    color: input.activeFocus ? theme.raised : theme.tile
    // Focus is the accent edge
    borderColor: input.activeFocus ? theme.focusRing : hover.containsMouse ? theme.lineStrong : theme.line
    borderWidth: 1
    function focusInput() { input.forceActiveFocus(); input.selectAll() }
    MouseArea { id: hover; anchors.fill: parent; hoverEnabled: true; cursorShape: Qt.IBeamCursor; onClicked: input.forceActiveFocus() }
    Icon {
        id: glass
        anchors.left: parent.left; anchors.leftMargin: root.large ? theme.space(5) : theme.space(3); anchors.verticalCenter: parent.verticalCenter
        glyph: "search"; size: root.large ? theme.space(5.5) : theme.space(4); color: input.activeFocus ? theme.text : theme.textDim
    }
    TextInput {
        id: input
        anchors.fill: parent
        anchors.leftMargin: root.large ? theme.space(14) : theme.space(9); anchors.rightMargin: root.goButton ? theme.space(20) : theme.space(10)
        verticalAlignment: TextInput.AlignVCenter
        color: theme.text
        font.family: theme.fontFamily; font.pointSize: root.large ? theme.typeLarge : theme.typeNormal
        font.weight: theme.roleWeight("ui"); font.variableAxes: theme.roleAxes("ui")
        selectionColor: theme.accentSoft; selectedTextColor: theme.text
        clip: true
        onAccepted: root.accepted()
        Keys.onUpPressed: root.stepped(-1)
        Keys.onDownPressed: root.stepped(1)
        // With a layer open (a menu, a picker, a sheet) Escape is left to propagate up to
        // the frame, which closes that layer; the query stays.
        Keys.onEscapePressed: function(e) {
            if (frame.escapeStack.top()) { e.accepted = false; return }
            text = ""; focus = false; root.cleared()
        }
        Text { anchors.fill: parent; verticalAlignment: Text.AlignVCenter; visible: !input.text; text: root.placeholder; color: theme.textFaint; font: input.font }
    }
    Button {
        id: goButton
        objectName: root.objectName !== "" ? root.objectName + "-go" : ""
        visible: root.goButton
        anchors.right: parent.right; anchors.rightMargin: theme.space(4); anchors.verticalCenter: parent.verticalCenter
        width: theme.space(8); height: width
        radius: width / 2
        primary: true
        icon: "search"
        busy: root.goBusy
        takesFocus: false
        onClicked: root.go()
        Tooltip { text: root.goTip }
    }
    Label {
        anchors.right: root.goButton ? goButton.left : parent.right; anchors.rightMargin: theme.space(3); anchors.verticalCenter: parent.verticalCenter
        visible: !input.activeFocus && !input.text && root.hint !== ""
        text: root.hint; color: theme.textFaint; role: "ledger"
    }
    // Clear: a round hover behind the x, a press darker still, and its tip
    Rectangle {
        objectName: root.objectName !== "" ? root.objectName + "-clear" : ""
        visible: input.text !== ""
        anchors.right: root.goButton ? goButton.left : parent.right; anchors.rightMargin: theme.space(1.5); anchors.verticalCenter: parent.verticalCenter
        width: theme.space(7); height: width; radius: width / 2
        color: clearArea.pressed ? theme.surfacePressed : clearArea.containsMouse ? theme.tileHover : "transparent"
        Icon { anchors.centerIn: parent; glyph: "x"; size: theme.space(4); color: clearArea.containsMouse ? theme.text : theme.textDim }
        MouseArea { id: clearArea; anchors.fill: parent; hoverEnabled: true; cursorShape: Qt.PointingHandCursor; onClicked: { input.text = ""; root.cleared(); input.forceActiveFocus() } }
        Tooltip { text: "Clear the search" }
    }
}
