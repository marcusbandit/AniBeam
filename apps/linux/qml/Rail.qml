// The left rail: a column of tiles set in the grout. The brand tile at the top, one tile per
// page (a Lucide glyph over a short label), and a foot tile with the title-language switch
// and the version. Every tile is a square pane of glass; the active page's is raised, its
// glyph in the accent, with a 2 px accent bar along its left edge. The tiles are as wide as the widest label (or the switch) needs at the
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
    // A short window: the six top tiles (brand and five pages) share what the foot leaves,
    // never overlapping it. Below the height an icon over a label needs, the label goes and
    // the tile is the icon alone; it never gets shorter than the icon and its padding.
    readonly property real labelledHeight: theme.space(5) + theme.space(1.5) + labelMetrics.height + pad * 2
    readonly property real fitHeight: Math.floor((height - footHeight - theme.grout * 8) / 6)
    readonly property real cellHeight: Math.max(theme.space(5) + pad * 2, Math.min(tileHeight, fitHeight))
    readonly property bool showLabels: cellHeight >= labelledHeight
    FontMetrics { id: labelMetrics; font.family: theme.fontFamily; font.pointSize: theme.typeSmall; font.weight: theme.roleWeight("title"); font.variableAxes: theme.roleAxes("title") }
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
            width: root.tileWidth; height: root.cellHeight
            radius: theme.radiusTile
            smoothing: theme.cornerSmoothing
            color: theme.tile
            borderColor: theme.line; borderWidth: 1
            highlight: theme.glassHighlight
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
                height: root.cellHeight
                radius: theme.radiusTile
                smoothing: theme.cornerSmoothing
                color: on || m.containsMouse ? theme.raised : theme.tile
                borderColor: entry.activeFocus ? theme.focusRing : (m.containsMouse || on ? theme.lineStrong : theme.line)
                borderWidth: 1
                highlight: theme.glassHighlight
                Rectangle { visible: entry.on; width: 2; height: parent.height; color: theme.accent }
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
                        visible: root.showLabels
                        anchors.horizontalCenter: parent.horizontalCenter
                        text: entry.modelData.text
                        role: "title"
                        font.pointSize: theme.typeSmall
                        color: entry.on || m.containsMouse ? theme.text : theme.textDim
                    }
                }
                MouseArea { id: m; anchors.fill: parent; hoverEnabled: true; cursorShape: Qt.PointingHandCursor; onClicked: root.picked(entry.index) }
                Tooltip { text: root.showLabels ? "" : entry.modelData.text }
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
        borderColor: theme.line; borderWidth: 1
        highlight: theme.glassHighlight
        Column {
            id: foot
            anchors.centerIn: parent
            spacing: theme.space(2)
            // The title language: one square naming the current one; a click switches
            Corner {
                id: lang
                objectName: "title-language"
                readonly property bool english: Door.preferences.title_language === "English"
                anchors.horizontalCenter: parent.horizontalCenter
                implicitWidth: langWidest.implicitWidth + theme.space(4)
                width: implicitWidth
                height: theme.space(6)
                radius: height / 2
                smoothing: theme.cornerSmoothing
                color: langMouse.containsMouse ? theme.raisedHover : theme.raised
                borderColor: langMouse.containsMouse ? theme.lineStrong : theme.line; borderWidth: 1
                scale: langMouse.pressed ? 0.98 : 1
                Label {
                    anchors.centerIn: parent
                    text: lang.english ? "English" : "Romaji"
                    role: "ui"
                    font.pointSize: theme.typeSmall
                }
                // the wider of the two, so the pill does not change width when switched
                Label { id: langWidest; visible: false; text: "English"; role: "ui"; font.pointSize: theme.typeSmall }
                MouseArea {
                    id: langMouse
                    anchors.fill: parent
                    hoverEnabled: true
                    cursorShape: Qt.PointingHandCursor
                    onClicked: {
                        var p = JSON.parse(JSON.stringify(Door.preferences))
                        p.title_language = lang.english ? "Romaji" : "English"
                        Door.setPreferences(p)
                    }
                }
                Tooltip { text: "Title language. Click to switch." }
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
