// The left rail: brand, the five pages with their Lucide glyphs, and the title-language
// switch with the version at the foot.
import QtQuick
import com.marcusrosado.AniBeam

Item {
    id: root
    property int active: 0
    signal picked(int index)

    readonly property var pages: [
        { text: "Library", icon: "house" }, { text: "Feed", icon: "rss" }, { text: "Watching", icon: "eye" },
        { text: "Metadata", icon: "database" }, { text: "Settings", icon: "settings" }
    ]
    width: theme.space(30)

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
                source: "qrc:/qt/qml/com/marcusrosado/AniBeam/assets/icon.png"
                sourceSize: Qt.size(96, 96)
                smooth: true
            }
        }
        Item { width: 1; height: theme.space(4) }

        Repeater {
            model: root.pages
            Corner {
                id: entry
                required property int index
                required property var modelData
                readonly property bool on: index === root.active
                width: root.width - theme.space(3)
                height: theme.controlHeight
                anchors.horizontalCenter: parent.horizontalCenter
                radius: theme.radiusMd
                smoothing: theme.cornerSmoothing
                color: on ? theme.surfaceRaised : (m.containsMouse ? theme.surface : "transparent")
                borderColor: entry.activeFocus ? theme.focusRing : "transparent"
                borderWidth: entry.activeFocus ? theme.space(0.5) : 0
                activeFocusOnTab: true
                Keys.onReturnPressed: root.picked(entry.index)
                Keys.onSpacePressed: root.picked(entry.index)
                Row {
                    anchors.left: parent.left
                    anchors.leftMargin: theme.space(3)
                    anchors.verticalCenter: parent.verticalCenter
                    spacing: theme.space(2)
                    Icon {
                        anchors.verticalCenter: parent.verticalCenter
                        glyph: entry.modelData.icon
                        color: entry.on ? theme.text : theme.textDim
                        size: theme.space(4.5)
                    }
                    Text {
                        anchors.verticalCenter: parent.verticalCenter
                        text: entry.modelData.text
                        color: entry.on ? theme.text : theme.textDim
                        font.family: theme.fontSans
                        font.pointSize: theme.typeNormal
                        font.weight: entry.on ? Font.DemiBold : Font.Normal
                    }
                }
                MouseArea { id: m; anchors.fill: parent; hoverEnabled: true; cursorShape: Qt.PointingHandCursor; onClicked: root.picked(entry.index) }
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
            index: Door.preferences.title_language === "English" ? 1 : 0
            onPicked: function(i) {
                var p = JSON.parse(JSON.stringify(Door.preferences))
                p.title_language = i === 1 ? "English" : "Romaji"
                Door.setPreferences(p)
            }
        }
        Text {
            anchors.horizontalCenter: parent.horizontalCenter
            width: root.width - theme.space(3)
            horizontalAlignment: Text.AlignHCenter
            elide: Text.ElideMiddle
            text: Shell.version
            color: theme.textFaint
            font.family: theme.fontMono
            font.pointSize: theme.typeSmall
            // Elided in the rail; the full string on hover through the frame's own tip,
            // never a native ToolTip.
            Tooltip { text: Shell.version }
        }
    }
}
