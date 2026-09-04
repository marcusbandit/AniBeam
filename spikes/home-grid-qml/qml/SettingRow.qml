// One setting: a label, an optional status line in the fixed face and an optional helper
// under it, and a control slot on the right. Never shorter than theme.space(10).
import QtQuick

Item {
    id: root
    property string label: ""
    property string line: ""
    property string helper: ""
    default property alias control: slot.data

    width: parent ? parent.width : theme.space(100)
    height: Math.max(theme.space(10), words.implicitHeight + theme.space(2) * 2, slot.height + theme.space(2) * 2)

    Column {
        id: words
        anchors.left: parent.left
        anchors.right: slot.left
        anchors.rightMargin: theme.space(6)
        anchors.verticalCenter: parent.verticalCenter
        spacing: theme.space(0.5)
        Text {
            width: parent.width
            text: root.label
            color: theme.text
            wrapMode: Text.Wrap
            font.family: theme.fontSans
            font.pointSize: theme.typeNormal
        }
        Text {
            visible: root.line !== ""
            width: parent.width
            text: root.line
            color: theme.textDim
            wrapMode: Text.Wrap
            font.family: theme.fontMono
            font.pointSize: theme.typeSmall
        }
        Text {
            visible: root.helper !== ""
            width: parent.width
            text: root.helper
            color: theme.textDim
            wrapMode: Text.Wrap
            font.family: theme.fontSans
            font.pointSize: theme.typeSmall
        }
    }
    Item {
        id: slot
        anchors.right: parent.right
        anchors.verticalCenter: parent.verticalCenter
        width: childrenRect.width
        height: childrenRect.height
    }
}
