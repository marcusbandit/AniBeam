// The inline confirm: the row's controls give way to a line naming the consequence, a red
// confirm button and Keep. Escape or Keep restores the row. No modal dialog exists. A row
// that can be narrower than the line sets `maxWidth`: when the line does not fit it, the
// question takes a line of its own above the buttons and wraps (#1123).
import QtQuick

Grid {
    id: root
    property string question: ""
    property string confirmText: words.t("action-remove")
    property string confirmIcon: "trash-2"
    signal accepted()
    signal kept()
    property real maxWidth: -1
    readonly property bool split: maxWidth >= 0 && question !== ""
                                  && questionText.implicitWidth + columnSpacing + buttons.width > maxWidth
    columns: split ? 1 : 2
    columnSpacing: theme.space(3)
    rowSpacing: theme.space(2)
    verticalItemAlignment: Grid.AlignVCenter
    function close() { kept() }
    // Registered only while shown: a row toggles its confirm with `visible`
    function sync() { if (visible) frame.escapeStack.push("confirm", root); else frame.escapeStack.pop(root) }
    Component.onCompleted: sync()
    onVisibleChanged: sync()
    Component.onDestruction: frame.escapeStack.pop(root)
    Text {
        id: questionText
        visible: text !== ""
        width: root.split ? Math.min(implicitWidth, root.maxWidth) : implicitWidth
        wrapMode: root.split ? Text.WrapAtWordBoundaryOrAnywhere : Text.NoWrap
        text: root.question
        color: theme.text
        font.family: theme.fontSans
        font.pointSize: theme.typeNormal
    }
    Row {
        id: buttons
        spacing: root.columnSpacing
        Button { objectName: root.objectName !== "" ? root.objectName + "-ok" : ""; text: root.confirmText; icon: root.confirmIcon; danger: true; onClicked: root.accepted() }
        Button { msg: "inline-confirm-keep"; flat: true; onClicked: root.kept() }
    }
}
