// The sidebar: one flat column a step up from the ground. The brand at the top, then the two
// places the app is for, Home and Library, each a glyph over its name. The rest of the app
// (Feed, Watching, Metadata, Downloads, Settings) is a quiet stack of round glyph buttons near the
// foot, named by their tips, and the foot itself holds the title-language switch and the
// version. The active place takes the soft accent; nothing else is lit.
import QtQuick
import com.marcusrosado.AniBeam

Item {
    id: root
    // Indexes into Nav.railPages: home, library, feed, watching, metadata, downloads, settings
    property int active: 0
    // Complete subscriptions waiting to be cleared: a red count on Downloads
    property int downloadsBadge: 0
    signal picked(int index)

    readonly property var primary: [
        { index: 0, text: "Home", icon: "house" }, { index: 1, text: "Library", icon: "layers" }
    ]
    readonly property var secondary: [
        { index: 2, text: "Feed", icon: "rss" }, { index: 3, text: "Watching", icon: "eye" },
        { index: 4, text: "Metadata", icon: "database" }, { index: 5, text: "Downloads", icon: "download" },
        { index: 6, text: "Settings", icon: "settings" }
    ]
    readonly property real pad: theme.space(2)
    readonly property real itemWidth: Math.max(theme.space(18), labels.implicitWidth + pad * 2, lang.implicitWidth + pad)
    width: itemWidth + pad * 2
    // The frame cuts its status strip to this, so the two share one band along the foot
    readonly property real footHeight: theme.space(9)

    // The labels at the size the items set them, never shown: their widest sets the column
    Column {
        id: labels
        visible: false
        Repeater { model: root.primary; Label { required property var modelData; role: "title"; font.pointSize: theme.typeSmall; text: modelData.text } }
    }

    Rectangle { anchors.fill: parent; color: theme.tile }
    Rectangle { anchors.right: parent.right; width: 1; height: parent.height; color: theme.line }

    Column {
        x: root.pad
        y: theme.space(4)
        spacing: theme.space(2)

        Image {
            x: (root.itemWidth - width) / 2
            width: theme.space(10); height: width
            source: "qrc:/qt/qml/com/marcusrosado/AniBeam/assets/icon.png"
            sourceSize: Qt.size(96, 96)
            smooth: true
        }
        Item { width: 1; height: theme.space(3) }

        Repeater {
            model: root.primary
            Rectangle {
                id: entry
                required property var modelData
                objectName: "rail-" + modelData.text.toLowerCase()   // a --drive target
                readonly property bool on: modelData.index === root.active
                width: root.itemWidth
                height: theme.space(17)
                radius: theme.radiusTile
                color: on ? theme.accentSoft : m.containsMouse ? theme.raised : "transparent"
                border.color: entry.activeFocus ? theme.focusRing : "transparent"
                border.width: 1
                scale: m.pressed ? 0.97 : 1
                activeFocusOnTab: true
                Keys.onReturnPressed: root.picked(modelData.index)
                Keys.onSpacePressed: root.picked(modelData.index)
                Column {
                    anchors.centerIn: parent
                    spacing: theme.space(1.5)
                    Icon {
                        anchors.horizontalCenter: parent.horizontalCenter
                        glyph: entry.modelData.icon
                        size: theme.space(6)
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
                MouseArea { id: m; anchors.fill: parent; hoverEnabled: true; cursorShape: Qt.PointingHandCursor; onClicked: root.picked(entry.modelData.index) }
            }
        }
    }

    // The rest of the app, and the foot
    Column {
        id: lower
        x: root.pad
        anchors.bottom: parent.bottom
        anchors.bottomMargin: theme.space(4)
        spacing: theme.space(1.5)

        Repeater {
            model: root.secondary
            Rectangle {
                id: minor
                required property var modelData
                objectName: "rail-" + modelData.text.toLowerCase()
                readonly property bool on: modelData.index === root.active
                x: (root.itemWidth - width) / 2
                width: theme.space(11); height: width
                radius: width / 2
                color: on ? theme.accentSoft : mm.containsMouse ? theme.raised : "transparent"
                border.color: minor.activeFocus ? theme.focusRing : "transparent"
                border.width: 1
                scale: mm.pressed ? 0.95 : 1
                activeFocusOnTab: true
                Keys.onReturnPressed: root.picked(modelData.index)
                Keys.onSpacePressed: root.picked(modelData.index)
                Icon {
                    anchors.centerIn: parent
                    glyph: minor.modelData.icon
                    size: theme.space(5)
                    color: minor.on ? theme.accent : (mm.containsMouse ? theme.text : theme.textFaint)
                }
                MouseArea { id: mm; anchors.fill: parent; hoverEnabled: true; cursorShape: Qt.PointingHandCursor; onClicked: root.picked(minor.modelData.index) }
                Tooltip { text: minor.badge > 0 ? minor.modelData.text + ": " + minor.badge + (minor.badge === 1 ? " subscription is" : " subscriptions are") + " complete" : minor.modelData.text }
                readonly property int badge: modelData.index === 5 ? root.downloadsBadge : 0
                // The badge: red, its count in it, over the glyph's top right
                Rectangle {
                    objectName: "rail-downloads-badge"
                    visible: minor.badge > 0
                    height: theme.space(4.5)
                    width: Math.max(height, badgeText.implicitWidth + theme.space(2))
                    radius: height / 2
                    x: minor.width - width * 0.7
                    y: -height * 0.2
                    color: theme.red
                    border.color: theme.tile
                    border.width: 2
                    Label {
                        id: badgeText
                        anchors.centerIn: parent
                        text: minor.badge > 99 ? "99+" : String(minor.badge)
                        role: "ledger"
                        font.pointSize: theme.typeSmall * 0.85
                        font.weight: 600
                        color: "#ffffff"
                    }
                }
            }
        }

        Item { width: 1; height: theme.space(3) }

        // The title language: one pill naming the current one; a click switches
        Rectangle {
            id: lang
            objectName: "title-language"
            readonly property bool english: Door.preferences.title_language === "English"
            x: (root.itemWidth - width) / 2
            implicitWidth: langWidest.implicitWidth + theme.space(4)
            width: implicitWidth
            height: theme.space(6)
            radius: height / 2
            color: langMouse.containsMouse ? theme.raisedHover : theme.raised
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
            Tooltip { text: "Title language" }
        }
        Label {
            width: root.itemWidth
            horizontalAlignment: Text.AlignHCenter
            wrapMode: Text.Wrap
            text: Shell.release
            role: "ledger"
            color: theme.textFaint
            Tooltip { text: Shell.version }
        }
    }
}
