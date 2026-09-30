// A glyph, a title and one line, centred in the page.
import QtQuick

Column {
    property string icon: "info"
    property string title: ""
    property string body: ""
    default property alias actions: actionRow.data
    anchors.centerIn: parent
    spacing: theme.space(3)
    Icon { glyph: icon; size: theme.space(12); color: theme.textFaint; anchors.horizontalCenter: parent.horizontalCenter }
    Text { text: title; color: theme.text; font.family: theme.fontSans; font.pointSize: theme.typeLarge; font.weight: Font.Bold; anchors.horizontalCenter: parent.horizontalCenter }
    Text { text: body; color: theme.textDim; font.family: theme.fontSans; font.pointSize: theme.typeNormal; anchors.horizontalCenter: parent.horizontalCenter; horizontalAlignment: Text.AlignHCenter; width: theme.space(100); wrapMode: Text.Wrap }
    Row { id: actionRow; spacing: theme.space(2); anchors.horizontalCenter: parent.horizontalCenter }
}
