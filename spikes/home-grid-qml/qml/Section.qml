// A settings section: a bold heading over a hairline, an optional helper, then its rows.
import QtQuick

Column {
    id: root
    property string title: ""
    property string helper: ""
    default property alias rows: body.data

    width: parent ? parent.width : theme.space(100)
    spacing: theme.space(2)

    Text {
        text: root.title
        color: theme.text
        font.family: theme.fontSans
        font.pointSize: theme.typeNormal
        font.weight: Font.Bold
    }
    Rectangle { width: parent.width; height: 1; color: theme.line }
    Text {
        visible: root.helper !== ""
        width: parent.width
        text: root.helper
        color: theme.textDim
        wrapMode: Text.Wrap
        font.family: theme.fontSans
        font.pointSize: theme.typeSmall
    }
    Column {
        id: body
        width: parent.width
        spacing: theme.space(1)
    }
}
