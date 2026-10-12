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
    Label { text: title; role: "title"; anchors.horizontalCenter: parent.horizontalCenter }
    Label { text: body; color: theme.textDim; role: "ui"; anchors.horizontalCenter: parent.horizontalCenter; horizontalAlignment: Text.AlignHCenter; width: theme.space(100); wrapMode: Text.Wrap }
    Row { id: actionRow; spacing: theme.space(2); anchors.horizontalCenter: parent.horizontalCenter }
}
