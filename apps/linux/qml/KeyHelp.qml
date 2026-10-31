// The keyboard map, shown by ? or the keyboard button. The list is the keys that work in the
// player written out: PlayerKeys.qml's handler and the frame's own shortcuts. A key neither
// acts on has no line here, and every line here is a key that works. Escape or a press
// outside closes it, and it holds the chrome open while it is up.
//
// A floating pane of glass on a Blocker scrim, like every sheet. The groups sit side by side when the
// window has room for them and stack when it does not; past the window's height the tile
// scrolls rather than spilling.
import QtQuick
import QtQuick.Controls.Basic as QC

Item {
    id: root
    objectName: "player-keys"
    property bool open: false
    anchors.fill: parent
    visible: open
    z: 950

    readonly property var groups: [
        { name: "Playback", keys: [
            ["Space / K", "Play or pause"],
            ["Left / Right", "Seek 5 s"],
            ["J / L", "Seek 10 s"],
            ["Ctrl+Right", "Skip the intro or outro, else 90 s"],
            [", / .", "One frame back or forward"],
            ["[ / ]", "Slower or faster, in steps of 0.25"],
            ["Backspace", "Normal speed"],
            ["Up / Down", "Volume up or down by 5"],
            ["M", "Mute"],
            ["C", "Subtitles off and back"],
            ["z / Z", "Subtitle delay 100 ms earlier or later"],
            ["Hold the video", "2\u00d7 speed while held"],
            ["S", "Save a screenshot to Pictures"]
        ] },
        { name: "Navigation", keys: [
            ["N / P", "Next or previous episode"],
            ["Page Up / Down", "Previous or next chapter"],
            ["Home / End", "The start, or the outro"],
            ["F / F11", "Fullscreen"],
            ["Escape", "Close what is open, else leave fullscreen, else leave the player"],
            ["Alt+Left", "Back"],
            ["?", "Show or hide this list"]
        ] },
        { name: "Library", keys: [
            ["Ctrl+K", "Library and search"],
            ["/ or Ctrl+F", "Search this page"],
            ["Ctrl+L", "Activity"],
            ["Ctrl+,", "Settings"],
            ["Ctrl+Q", "Quit"]
        ] }
    ]

    // Guarded the way TrackPicker's openAt is: a second ? on a list already up would count
    // itself twice into page.openMenus and the chrome would never hide again.
    // The player that holds the list open, when it is the player's; the frame's own list
    // (? anywhere else) has none
    property var player: null
    function show() { if (open) return; open = true; if (player) player.openMenus++; frame.escapeStack.push("popover", root) }
    function close() { if (!open) return; open = false; if (player) { player.openMenus--; player.showChrome() } frame.escapeStack.pop(root) }
    // A navigation destroys the page and this list with it without ever calling close(), so
    // the count and the escape stack would keep an entry whose closer is gone. close() is
    // already guarded for the never-opened case, so this is safe unconditionally.
    Component.onDestruction: root.close()

    Blocker { anchors.fill: parent; dismissOnPress: true; onDismissed: root.close() }

    // The key column is one width across every group, the widest chip's, so every
    // description starts the same distance from its column's edge.
    property real capWidth: 0
    readonly property real gap: theme.space(8)
    readonly property real margin: theme.space(4)
    // Side by side when the three fit inside the window, else one above the next.
    readonly property bool wide: groupsGrid.wideWidth + panel.padding * 2 <= width - margin * 2

    Tile {
        id: panel
        color: theme.glassFloat
        borderColor: theme.lineStrong
        padding: theme.space(6)
        anchors.centerIn: parent
        width: Math.min(parent.width - root.margin * 2, column.implicitWidth + padding * 2)
        height: Math.min(parent.height - root.margin * 2, column.implicitHeight + padding * 2)
        // The Blocker dismisses on any press; one on the panel's padding must not reach it.
        MouseArea { anchors.fill: parent; anchors.margins: -panel.padding; acceptedButtons: Qt.AllButtons }
        Flickable {
            id: keyFlick
            anchors.fill: parent
            clip: true
            contentWidth: column.implicitWidth; contentHeight: column.implicitHeight
            boundsBehavior: Flickable.StopAtBounds
            QC.ScrollBar.vertical: ThinScrollBar {}
            WheelStep { flick: keyFlick }
            Column {
                id: column
                spacing: theme.space(5)
                Label { role: "title"; font.pointSize: theme.typeLarge; text: "Keyboard shortcuts" }
                Grid {
                    id: groupsGrid
                    columns: root.wide ? root.groups.length : 1
                    columnSpacing: root.gap
                    rowSpacing: theme.space(5)
                    // What the grid would be with every group in one row.
                    property real wideWidth: 0
                    Component.onCompleted: measure()
                    function measure() {
                        var w = 0
                        for (var i = 0; i < children.length; i++) if (children[i].implicitWidth !== undefined && children[i].isGroup) w += children[i].implicitWidth
                        wideWidth = w + root.gap * (root.groups.length - 1)
                    }
                    Repeater {
                        model: root.groups
                        Column {
                            required property var modelData
                            readonly property bool isGroup: true
                            spacing: theme.space(2)
                            onImplicitWidthChanged: groupsGrid.measure()
                            Label { role: "title"; color: theme.textDim; text: modelData.name; bottomPadding: theme.space(1) }
                            Repeater {
                                model: modelData.keys
                                Row {
                                    required property var modelData
                                    spacing: theme.space(3)
                                    Item {
                                        anchors.verticalCenter: parent.verticalCenter
                                        width: root.capWidth; height: cap.height
                                        // Every chip is the column's width, so the gap to
                                        // its description is the row spacing in every group.
                                        Chip { id: cap; width: root.capWidth; text: modelData[0]; color: theme.tile; textColor: theme.text
                                            onImplicitWidthChanged: root.capWidth = Math.max(root.capWidth, implicitWidth)
                                            Component.onCompleted: root.capWidth = Math.max(root.capWidth, implicitWidth) }
                                    }
                                    Label { anchors.verticalCenter: parent.verticalCenter; role: "ui"; color: theme.textDim; text: modelData[1] }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
