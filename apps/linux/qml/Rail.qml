// The sidebar (#1012): one flat column a step up from the ground, holding four places and two
// round buttons, glyphs only, each named by its tip. The brand at the top, then Home, Library,
// Airing and Downloads as square tiles, and at the foot Join room and Settings. The column is
// one tile wide plus the same gap on each side as between the tiles, so a lit tile sits with
// an even margin all round. The pages that used to have glyphs of their own live inside a
// place: the feed opens from Home's week, and Watching, Stats and Show info are tabs of
// Library, so the rail lights Library while one of them is open. The title language is a
// row in Settings, Appearance, and the version with What's new is in Settings, About. The
// active place takes the soft accent; nothing else is lit.
import QtQuick
import com.marcusrosado.Anibeam

Item {
    id: root
    // The page open now, or the rail page the trail last passed through; the rail lights the
    // place that holds it
    property string page: "home"
    // Complete subscriptions waiting to be cleared: a red count on Downloads
    property int downloadsBadge: 0
    // An update waits: a red dot on Settings until it is installed, and its line in the tip
    property bool settingsBadge: false
    property string settingsNote: ""
    // A place picked, by the page it opens
    signal picked(string page)
    // Watch together (#846): lit while in a room or while its panel is up
    property bool inRoom: false
    property bool roomOpen: false
    signal roomClicked()

    // `key` names the --drive target (rail-<key>) and the page it opens, `msg` the words
    readonly property var places: [
        { key: "home", msg: "rail-home", icon: "house" },
        { key: "library", msg: "rail-library", icon: "layers" },
        { key: "airing", msg: "rail-airing", icon: "radio" },
        { key: "downloads", msg: "rail-downloads", icon: "download" }
    ]
    // Which place holds each page that is not a place itself. Library's three must match the
    // pages in LibraryTabs.qml
    readonly property var holders: ({ feed: "home", watching: "library", stats: "library", metadata: "library" })
    readonly property string lit: holders[page] || page

    // One gap everywhere: beside the tiles, between them and above the brand
    readonly property real pad: theme.space(2)
    readonly property real itemWidth: theme.space(12)
    width: itemWidth + pad * 2
    // The frame cuts its status strip to this, so the two share one band along the foot
    readonly property real footHeight: theme.space(9)
    readonly property real roundSize: itemWidth
    // A short window shrinks the place tiles before they run under the round buttons at the
    // foot; a tall one leaves them square. Upper: the brand, a gap and four tiles, a gap between
    // each; lower: two round buttons over the bottom gap.
    readonly property real tileSize: {
        var brand = itemWidth - theme.space(2)
        var free = height - (pad + theme.space(1)) - brand - pad * 6 - pad - (roundSize * 2 + pad * 2)
        return Math.max(theme.space(8), Math.min(itemWidth, Math.floor(free / 4)))
    }

    Rectangle { anchors.fill: parent; color: theme.tile }
    Rectangle { anchors.right: parent.right; width: 1; height: parent.height; color: theme.line }

    Column {
        id: upper
        x: root.pad
        // The brand sits as far from the top as from the sides
        y: root.pad + theme.space(1)
        spacing: root.pad

        Image {
            x: (root.itemWidth - width) / 2
            width: root.itemWidth - theme.space(2); height: width
            source: "qrc:/qt/qml/com/marcusrosado/Anibeam/assets/" + (Theme.appIcon === "dark" ? "icon-dark.png" : Shell.defaultIcon)
            sourceSize: Qt.size(96, 96)
            // Decoded off the Qt thread, not inside the frame's first build
            asynchronous: true
            smooth: true
        }
        Item { width: 1; height: root.pad }

        Repeater {
            model: root.places
            Rectangle {
                id: entry
                required property var modelData
                objectName: "rail-" + modelData.key   // a --drive target
                readonly property bool on: modelData.key === root.lit
                readonly property int badge: modelData.key === "downloads" ? root.downloadsBadge : 0
                Accessible.role: Accessible.PageTab
                Accessible.name: badge > 0 ? words.t("rail-downloads-complete", { count: badge }) : words.t(modelData.msg)
                Accessible.selected: on
                Accessible.onPressAction: root.picked(modelData.key)
                width: root.itemWidth
                height: root.tileSize
                radius: theme.radiusTile
                color: on ? theme.accentSoft : m.containsMouse ? theme.raised : "transparent"
                border.color: entry.activeFocus ? theme.focusRing : "transparent"
                border.width: 1
                scale: theme.reducedMotion ? 1 : (m.pressed ? 0.97 : 1)
                activeFocusOnTab: true
                Keys.onReturnPressed: root.picked(modelData.key)
                Keys.onSpacePressed: root.picked(modelData.key)
                Icon {
                    anchors.centerIn: parent
                    glyph: entry.modelData.icon
                    size: theme.space(6)
                    color: entry.on ? theme.accent : (m.containsMouse ? theme.text : theme.textDim)
                }
                MouseArea { id: m; anchors.fill: parent; hoverEnabled: true; cursorShape: Qt.PointingHandCursor; onClicked: root.picked(entry.modelData.key) }
                Tooltip { text: entry.Accessible.name }
                // Complete subscriptions on Downloads: a red count over the glyph's top right
                Rectangle {
                    objectName: "rail-downloads-badge"
                    visible: entry.badge > 0
                    height: theme.space(4.5)
                    width: Math.max(height, badgeText.implicitWidth + theme.space(2))
                    radius: height / 2
                    x: entry.width - width * 0.8
                    y: -height * 0.2
                    color: theme.red
                    border.color: theme.tile
                    border.width: 2
                    Label {
                        id: badgeText
                        anchors.centerIn: parent
                        text: entry.badge > 99 ? "99+" : String(entry.badge)
                        role: "ledger"
                        font.pointSize: theme.typeSmall * 0.85
                        font.weight: 600
                        color: theme.contrast(theme.red, Qt.rgba(1, 1, 1, 1)) >= 4.5 ? "#ffffff" : "#0d0e11"
                    }
                }
            }
        }
    }

    // The foot: Join room and Settings, round and named by their tips
    Column {
        id: lower
        x: root.pad
        anchors.bottom: parent.bottom
        anchors.bottomMargin: root.pad
        spacing: root.pad

        // Watch together (#888): not a page, a panel; Ctrl+J opens it too
        Rectangle {
            id: roomEntry
            objectName: "rail-room"
            readonly property bool on: root.inRoom || root.roomOpen
            Accessible.role: Accessible.Button
            Accessible.name: words.t(root.inRoom ? "rail-in-room" : "rail-join-room")
            Accessible.onPressAction: root.roomClicked()
            x: (root.itemWidth - width) / 2
            width: root.roundSize; height: width
            radius: width / 2
            color: on ? theme.accentSoft : rm.containsMouse ? theme.raised : "transparent"
            border.color: roomEntry.activeFocus ? theme.focusRing : "transparent"
            border.width: 1
            scale: theme.reducedMotion ? 1 : (rm.pressed ? 0.95 : 1)
            activeFocusOnTab: true
            Keys.onReturnPressed: root.roomClicked()
            Keys.onSpacePressed: root.roomClicked()
            Icon {
                anchors.centerIn: parent
                glyph: "users"
                size: theme.space(5)
                color: roomEntry.on ? theme.accent : (rm.containsMouse ? theme.text : theme.textDim)
            }
            MouseArea { id: rm; anchors.fill: parent; hoverEnabled: true; cursorShape: Qt.PointingHandCursor; onClicked: root.roomClicked() }
            Tooltip { text: root.inRoom ? words.t("rail-room-in") : words.t("rail-room") }
        }

        Rectangle {
            id: settingsEntry
            objectName: "rail-settings"
            readonly property bool on: root.lit === "settings"
            Accessible.role: Accessible.PageTab
            Accessible.name: words.t("rail-settings")
            Accessible.selected: on
            Accessible.onPressAction: root.picked("settings")
            x: (root.itemWidth - width) / 2
            width: root.roundSize; height: width
            radius: width / 2
            color: on ? theme.accentSoft : sm.containsMouse ? theme.raised : "transparent"
            border.color: settingsEntry.activeFocus ? theme.focusRing : "transparent"
            border.width: 1
            scale: theme.reducedMotion ? 1 : (sm.pressed ? 0.95 : 1)
            activeFocusOnTab: true
            Keys.onReturnPressed: root.picked("settings")
            Keys.onSpacePressed: root.picked("settings")
            Icon {
                anchors.centerIn: parent
                glyph: "settings"
                size: theme.space(5)
                color: settingsEntry.on ? theme.accent : (sm.containsMouse ? theme.text : theme.textDim)
            }
            MouseArea { id: sm; anchors.fill: parent; hoverEnabled: true; cursorShape: Qt.PointingHandCursor; onClicked: root.picked("settings") }
            Tooltip { text: root.settingsBadge && root.settingsNote !== "" ? words.t("rail-settings-note", { note: root.settingsNote }) : words.t("rail-settings") }
            // An update waits: a red dot over the glyph's top right
            Rectangle {
                objectName: "rail-settings-badge"
                visible: root.settingsBadge
                width: theme.space(3.5); height: width
                radius: width / 2
                x: settingsEntry.width - width * 1.1
                y: width * 0.1
                color: theme.red
                border.color: theme.tile
                border.width: 2
            }
        }
    }
}
