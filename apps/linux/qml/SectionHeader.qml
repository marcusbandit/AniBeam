// The section idiom: a title in the title cut with its count beside it in the ledger, and an
// action slot on the right.
import QtQuick

Item {
    property string title: ""
    property string index: ""
    property int count: -1
    default property alias actions: right.data
    width: parent ? parent.width : implicitWidth
    implicitHeight: theme.controlHeight
    Row {
        anchors.left: parent.left; anchors.verticalCenter: parent.verticalCenter
        spacing: theme.space(2)
        Label {
            id: heading
            text: title
            role: "title"
            anchors.verticalCenter: parent.verticalCenter
        }
        Label {
            visible: count >= 0
            text: count.toString()
            color: theme.textFaint
            role: "ledger"
            anchors.baseline: heading.baseline
        }
    }
    Row { id: right; anchors.right: parent.right; anchors.verticalCenter: parent.verticalCenter; spacing: theme.space(2) }
}
