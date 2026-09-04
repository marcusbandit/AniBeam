// A settings panel: a surface with a hairline edge, a Lucide glyph beside a bold heading,
// an optional helper line, then its rows. Fills whatever column it sits in.
import QtQuick

Corner {
    id: root
    property string title: ""
    property string icon: ""
    property string helper: ""
    default property alias rows: body.data

    readonly property real pad: theme.space(6)
    width: parent ? parent.width : theme.space(100)
    implicitHeight: inner.height + pad * 2
    radius: theme.radiusLg
    smoothing: theme.cornerSmoothing
    color: theme.surface
    borderColor: theme.line
    borderWidth: 1

    Column {
        id: inner
        x: root.pad
        y: root.pad
        width: root.width - root.pad * 2
        spacing: theme.space(3)

        Row {
            spacing: theme.space(2)
            Icon {
                visible: root.icon !== ""
                anchors.verticalCenter: parent.verticalCenter
                glyph: root.icon
                color: theme.text
            }
            Text {
                anchors.verticalCenter: parent.verticalCenter
                text: root.title
                color: theme.text
                font.family: theme.fontSans
                font.pointSize: theme.typeNormal
                font.weight: Font.Bold
            }
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
        Column {
            id: body
            width: parent.width
            spacing: theme.space(2)
        }
    }
}
