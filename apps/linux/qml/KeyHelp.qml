// The keyboard map, shown by ?. The list is the page's handler written out: a key that
// PlayerPage's Keys.onPressed does not act on has no line here, and every line here is a key
// that works. Escape or a click outside closes it, and it holds the chrome open while it is
// up. The panel is as wide as the list rather than a fixed box, so a longer line grows it.
import QtQuick
import QtQuick.Controls.Basic as QC

Item {
    id: root
    property bool open: false
    anchors.fill: parent
    visible: open
    z: 950

    readonly property var keys: [
        ["Space / K", "Play or pause"],
        ["Left / Right", "Seek 5 s"],
        ["Ctrl+Right", "Skip the intro or outro, else 90 s"],
        [", / .", "One frame back or forward"],
        ["M", "Mute"],
        ["F", "Fullscreen"],
        ["C", "Subtitles off and back"],
        ["z / Z", "Subtitle delay 100 ms earlier or later"],
        ["Up / Down", "Volume 5"],
        ["Escape", "Leave the player"],
        ["?", "Show or hide this list"]
    ]

    // Guarded the way TrackPicker's openAt is: a second ? on a list already up would count
    // itself twice into page.openMenus and the chrome would never hide again.
    function show() { if (open) return; open = true; page.openMenus++; frame.escapeStack.push("popover", root) }
    function close() { if (!open) return; open = false; page.openMenus--; frame.escapeStack.pop(root); page.showChrome() }
    // A navigation destroys the page and this list with it without ever calling close(), so
    // the count and the escape stack would keep an entry whose closer is gone. close() is
    // already guarded for the never-opened case, so this is safe unconditionally.
    Component.onDestruction: root.close()

    MouseArea { anchors.fill: parent; onPressed: root.close() }
    Corner {
        anchors.centerIn: parent
        // As wide and as tall as the list, up to what the window has room for. Past that the
        // list scrolls rather than spilling out of the panel, which is what a small window at
        // density 1.25 would otherwise do.
        width: Math.min(parent.width - theme.space(8), column.implicitWidth + theme.space(10))
        height: Math.min(parent.height - theme.space(8), column.implicitHeight + theme.space(10))
        radius: theme.radiusLg; smoothing: theme.cornerSmoothing
        color: theme.surfaceRaised; borderColor: theme.lineStrong; borderWidth: 1
        MouseArea { anchors.fill: parent }
        Flickable {
            anchors.fill: parent; anchors.margins: theme.space(5)
            clip: true
            contentWidth: column.implicitWidth; contentHeight: column.implicitHeight
            boundsBehavior: Flickable.StopAtBounds
            QC.ScrollBar.vertical: ThinScrollBar {}
            Column {
                id: column
                spacing: theme.space(2)
                Text { text: "Keyboard shortcuts"; color: theme.text; font.family: theme.fontSans; font.pointSize: theme.typeLarge; font.weight: Font.DemiBold; bottomPadding: theme.space(2) }
                Repeater {
                    model: root.keys
                    Row {
                        required property var modelData
                        spacing: theme.space(3)
                        // One key column, so the descriptions line up; a cap wider than the
                        // column widens its own row rather than running under the description.
                        Item {
                            anchors.verticalCenter: parent.verticalCenter
                            width: Math.max(theme.space(30), cap.implicitWidth); height: cap.height
                            Chip { id: cap; text: modelData[0]; small: true; color: theme.surface; textColor: theme.text }
                        }
                        Text { anchors.verticalCenter: parent.verticalCenter; text: modelData[1]; color: theme.textDim; font.family: theme.fontSans; font.pointSize: theme.typeNormal }
                    }
                }
            }
        }
    }
}
