// The Diablo section idiom: a numbered label in letter-spaced caps mono — "02 / AIRING" —
// with the count in the accent and an action slot on the right.
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
            text: (index !== "" ? index + " / " : "") + title.toUpperCase()
            color: theme.textDim
            font.family: theme.fontMono
            font.pointSize: theme.typeSmall
            font.letterSpacing: theme.typeSmall * 0.14
            anchors.verticalCenter: parent.verticalCenter
        }
        Text {
            visible: count >= 0
            text: count.toString()
            color: theme.accent
            font.family: theme.fontMono
            font.pointSize: theme.typeSmall
            anchors.verticalCenter: parent.verticalCenter
        }
    }
    Row { id: right; anchors.right: parent.right; anchors.verticalCenter: parent.verticalCenter; spacing: theme.space(2) }
}
