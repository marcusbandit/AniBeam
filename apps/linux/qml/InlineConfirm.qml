// The inline confirm: the row's controls give way to a line naming the consequence, a red
// confirm button and Keep. Escape or Keep restores the row. No modal dialog exists.
import QtQuick

Row {
    id: root
    property string question: ""
    property string confirmText: "Remove"
    property string confirmIcon: "trash-2"
    signal accepted()
    signal kept()
    spacing: theme.space(3)
    function close() { kept() }
    // Registered only while shown: a row toggles its confirm with `visible`
    function sync() { if (visible) frame.escapeStack.push("confirm", root); else frame.escapeStack.pop(root) }
    Component.onCompleted: sync()
    onVisibleChanged: sync()
    Component.onDestruction: frame.escapeStack.pop(root)
    Text { anchors.verticalCenter: parent.verticalCenter; text: root.question; color: theme.text; font.family: theme.fontSans; font.pointSize: theme.typeNormal }
    Button { text: root.confirmText; icon: root.confirmIcon; danger: true; onClicked: root.accepted() }
    Button { text: "Keep"; flat: true; onClicked: root.kept() }
}
