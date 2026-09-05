// A pill of text on a scrim or a surface, with an optional Lucide glyph before the text.
// Numbers set in the fixed face.
import QtQuick

Corner {
    id: root
    property string text: ""
    property string icon: ""
    property color textColor: theme.text
    property bool small: false
    property bool mono: true
    property bool selected: false
    property bool clickable: false
    signal clicked()

    implicitWidth: content.implicitWidth + theme.space(small ? 2 : 2.5) * 2
    implicitHeight: label.implicitHeight + theme.space(small ? 0.75 : 1.25) * 2
    radius: height / 2
    smoothing: theme.cornerSmoothing
    color: selected ? theme.accentSoft : theme.scrim
    borderColor: hover.containsMouse ? theme.lineStrong : "transparent"
    borderWidth: clickable ? 1 : 0

    readonly property color ink: selected ? theme.accent : textColor
    Row {
        id: content
        anchors.centerIn: parent
        spacing: theme.space(1)
        Icon {
            visible: root.icon !== ""
            anchors.verticalCenter: parent.verticalCenter
            glyph: root.icon
            color: root.ink
            size: theme.space(root.small ? 3.5 : 4)
        }
        Text {
            id: label
            anchors.verticalCenter: parent.verticalCenter
            text: root.text
            color: root.ink
            font.family: root.mono ? theme.fontMono : theme.fontSans
            font.pointSize: root.small ? theme.typeSmall : theme.typeNormal
            font.weight: Font.Medium
        }
    }
    MouseArea {
        id: hover
        anchors.fill: parent
        enabled: root.clickable
        hoverEnabled: root.clickable
        cursorShape: Qt.PointingHandCursor
        onClicked: root.clicked()
    }
}
