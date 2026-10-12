// The left rail: a column of tiles set in the grout. The brand tile at the top, one tile per
// page (a Lucide glyph over a short label), and a foot tile with the title-language switch
// and the version. The active page's tile is raised and its glyph is in the accent; there is
// no accent bar. The tiles are as wide as the widest label (or the switch) needs at the
// current density, so a label never clips.
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
    readonly property real pad: theme.space(2)
    readonly property real tileWidth: Math.max(theme.space(17), labels.implicitWidth + pad * 2, lang.implicitWidth + pad * 2)
    readonly property real tileHeight: Math.round(tileWidth * 0.72)
    width: tileWidth + theme.grout * 2
    // The foot tile's height: the frame's status strip is cut to it, so the two sit in one
    // band along the window's foot
    readonly property real footHeight: footTile.height

    // The labels at the size the tiles set them, never shown: their widest is the tile's
    Column {
        id: labels
        visible: false
        Repeater { model: root.pages; Label { required property var modelData; role: "title"; font.pointSize: theme.typeSmall; text: modelData.text } }
    }

    Rectangle { anchors.fill: parent; color: theme.ground }

    Column {
        x: theme.grout
        y: theme.grout
        spacing: theme.grout

        // Brand
        Corner {
            width: root.tileWidth; height: root.tileHeight
            radius: theme.radiusTile
            smoothing: theme.cornerSmoothing
            color: theme.tile
            Image {
                anchors.centerIn: parent
                width: Math.round(parent.height * 0.62); height: width
                source: "qrc:/qt/qml/com/marcusrosado/AniBeam/assets/icon.png"
                sourceSize: Qt.size(96, 96)
                smooth: true
            }
        }

        Repeater {
            model: root.pages
            Corner {
                id: entry
                required property int index
                required property var modelData
                objectName: "rail-" + modelData.text.toLowerCase()   // a --drive target
                readonly property bool on: index === root.active
                width: root.tileWidth
                height: root.tileHeight
                radius: theme.radiusTile
                smoothing: theme.cornerSmoothing
                color: on ? theme.raised : theme.tile
                borderColor: entry.activeFocus ? theme.focusRing : (m.containsMouse && !on ? theme.line : "transparent")
                borderWidth: entry.activeFocus ? theme.space(0.5) : (m.containsMouse && !on ? 1 : 0)
                scale: m.pressed ? 0.98 : 1
                activeFocusOnTab: true
                Keys.onReturnPressed: root.picked(entry.index)
                Keys.onSpacePressed: root.picked(entry.index)
                Column {
                    anchors.centerIn: parent
                    spacing: theme.space(1.5)
                    Icon {
                        anchors.horizontalCenter: parent.horizontalCenter
                        glyph: entry.modelData.icon
                        size: theme.space(5)
                        color: entry.on ? theme.accent : (m.containsMouse ? theme.text : theme.textDim)
                    }
                    Label {
                        anchors.horizontalCenter: parent.horizontalCenter
                        text: entry.modelData.text
                        role: "title"
                        font.pointSize: theme.typeSmall
                        color: entry.on || m.containsMouse ? theme.text : theme.textDim
                    }
                }
                MouseArea { id: m; anchors.fill: parent; hoverEnabled: true; cursorShape: Qt.PointingHandCursor; onClicked: root.picked(entry.index) }
            }
        }
    }

    // Foot: the title language and the version (the crate's, then the commit, on two lines
    // when the tile is narrow)
    Corner {
        id: footTile
        x: theme.grout
        anchors.bottom: parent.bottom
        anchors.bottomMargin: theme.grout
        width: root.tileWidth
        height: foot.implicitHeight + root.pad * 2
        radius: theme.radiusTile
        smoothing: theme.cornerSmoothing
        color: theme.tile
        Column {
            id: foot
            anchors.centerIn: parent
            spacing: theme.space(2)
            Seg {
                id: lang
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
            Label {
                anchors.horizontalCenter: parent.horizontalCenter
                width: root.tileWidth - root.pad * 2
                horizontalAlignment: Text.AlignHCenter
                wrapMode: Text.Wrap
                text: Shell.release
                role: "ledger"
                color: theme.textDim
                // The full git description on hover, through the frame's own tip
                Tooltip { text: Shell.version }
            }
        }
    }
}
