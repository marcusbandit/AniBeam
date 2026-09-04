// The left rail: brand, the five pages, and the title-language switch with the version at
// the foot. Text labels only in the prototype; no icon set yet.
import QtQuick

Item {
    id: root
    property int active: 0
    property string titleLang: "jp"
    signal langPicked(string lang)

    width: theme.space(23)

    Rectangle {
        anchors.fill: parent
        color: theme.surfaceSunken
    }
    Rectangle {
        anchors.right: parent.right
        width: 1
        height: parent.height
        color: theme.line
    }

    Column {
        anchors.top: parent.top
        anchors.topMargin: theme.space(5)
        anchors.horizontalCenter: parent.horizontalCenter
        spacing: theme.space(1)

        Corner {
            width: theme.space(11); height: width
            radius: theme.radiusMd
            smoothing: theme.cornerSmoothing
            color: theme.accentSoft
            anchors.horizontalCenter: parent.horizontalCenter
            Image {
                anchors.centerIn: parent
                width: parent.width * 0.7; height: width
                source: "qrc:/qt/qml/dev/anibeam/proto/assets/icon.png"
                sourceSize: Qt.size(96, 96)
                smooth: true
            }
        }
        Item { width: 1; height: theme.space(4) }

        Repeater {
            model: ["Library", "Feed", "Watching", "Metadata", "Settings"]
            Corner {
                width: root.width - theme.space(3)
                height: theme.controlHeight
                anchors.horizontalCenter: parent.horizontalCenter
                radius: theme.radiusMd
                smoothing: theme.cornerSmoothing
                color: index === root.active ? theme.surfaceRaised : (m.containsMouse ? theme.surface : "transparent")
                Text {
                    anchors.centerIn: parent
                    text: modelData
                    color: index === root.active ? theme.text : theme.textDim
                    font.family: theme.fontSans
                    font.pointSize: theme.typeNormal
                    font.weight: index === root.active ? Font.DemiBold : Font.Normal
                }
                MouseArea { id: m; anchors.fill: parent; hoverEnabled: true; cursorShape: Qt.PointingHandCursor; onClicked: root.active = index }
            }
        }
    }

    Column {
        anchors.bottom: parent.bottom
        anchors.bottomMargin: theme.space(5)
        anchors.horizontalCenter: parent.horizontalCenter
        spacing: theme.space(3)
        Seg {
            anchors.horizontalCenter: parent.horizontalCenter
            small: true
            options: ["JP", "EN"]
            index: root.titleLang === "en" ? 1 : 0
            onPicked: function(i) { root.langPicked(i === 1 ? "en" : "jp") }
        }
        Text {
            anchors.horizontalCenter: parent.horizontalCenter
            text: "2.0.0-proto"
            color: theme.textFaint
            font.family: theme.fontMono
            font.pointSize: theme.typeSmall
        }
    }
}
