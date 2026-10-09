// The section idiom: a title in the voice (Fraunces) with its count beside it, and an
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
        Text {
            text: title
            color: theme.text
            font.family: theme.fontDisplay
            font.pointSize: theme.typeNormal * 1.15
            font.weight: Font.DemiBold
            anchors.verticalCenter: parent.verticalCenter
        }
        Text {
            visible: count >= 0
            text: count.toString()
            color: theme.textFaint
            font.family: theme.fontUi
            font.pointSize: theme.typeSmall
            anchors.verticalCenter: parent.verticalCenter
        }
    }
    Row { id: right; anchors.right: parent.right; anchors.verticalCenter: parent.verticalCenter; spacing: theme.space(2) }
}
