// A bold title, a count chip and an action slot on the right.
import QtQuick

Item {
    property string title: ""
    property int count: -1
    default property alias actions: right.data
    width: parent ? parent.width : implicitWidth
    implicitHeight: theme.controlHeight
    Row {
        anchors.left: parent.left; anchors.verticalCenter: parent.verticalCenter
        spacing: theme.space(3)
        Text { text: title; color: theme.text; font.family: theme.fontSans; font.pointSize: theme.typeNormal; font.weight: Font.Bold; anchors.verticalCenter: parent.verticalCenter }
        Chip { visible: count >= 0; text: String(count); small: true; color: theme.surface; textColor: theme.textDim; anchors.verticalCenter: parent.verticalCenter }
    }
    Row { id: right; anchors.right: parent.right; anchors.verticalCenter: parent.verticalCenter; spacing: theme.space(2) }
}
