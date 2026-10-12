// A track picker: Off (for subtitles) plus every track, the current one in the accent.
// Opens above its anchor (above the whole controls tile in the player), scrolling when
// the list is taller than the room there; a pick closes it; Escape closes it through the frame.
import QtQuick
import QtQuick.Controls.Basic as QC

Item {
    id: root
    property string title: ""
    property var tracks: []
    property int selected: -1
    property bool offRow: false
    property bool open: false
    signal picked(int id)
    anchors.fill: parent
    visible: open
    z: 900

    // The anchor is kept so the panel can be placed again on every size change while open:
    // the F key toggles fullscreen with a picker up, and a position computed once would
    // strand the panel away from its button. mapToItem is not a tracked binding, so the
    // explicit size handlers below are what keep it right.
    property var anchorItem: null
    // The tallest the panel may be; place() lowers it to the room above the anchor, so a
    // long list scrolls instead of running off the window's top.
    property real maxHeight: theme.space(80)
    // The tile the anchor sits in (the player's controls island), so the panel clears all
    // of it, the seek bar included, not only the button.
    function tileOf(item) {
        for (var p = item; p; p = p.parent) if (p.objectName === "player-controls") return p
        return null
    }
    function place() {
        if (!anchorItem) return
        var p = anchorItem.mapToItem(root, anchorItem.width / 2, 0)
        var tile = tileOf(anchorItem)
        var floor = (tile ? tile.mapToItem(root, 0, 0).y : p.y) - theme.space(2)
        maxHeight = Math.max(theme.space(20), Math.min(theme.space(80), floor - theme.space(2)))
        panel.x = Math.max(theme.space(2), Math.min(p.x - panel.width / 2, root.width - panel.width - theme.space(2)))
        panel.y = Math.max(theme.space(2), floor - panel.height)
    }
    // The current track, scrolled into the middle of the list when the list scrolls
    function revealSelected() {
        var rows = (offRow ? [{ id: -1 }] : []).concat(tracks)
        for (var i = 0; i < rows.length; i++) if (rows[i].id === selected) break
        if (i >= rows.length) return
        var y = theme.space(6) + i * theme.controlHeight
        trackList.contentY = Math.max(0, Math.min(trackList.contentHeight - trackList.height, y - (trackList.height - theme.controlHeight) / 2))
    }
    // Deferred a tick: the picker resizes with the window before the chrome lays its
    // buttons out again, so placing at once reads the anchor's old position.
    onWidthChanged: if (open) Qt.callLater(root.place)
    onHeightChanged: if (open) Qt.callLater(root.place)
    // Guarded the same way close() is: a second openAt on a picker already up would count
    // itself twice into page.openMenus and the chrome would never hide again.
    function openAt(anchor) {
        if (open) return
        anchorItem = anchor
        place()
        open = true
        revealSelected()
        page.openMenus++
        frame.escapeStack.push("popover", root)
    }
    function close() { if (!open) return; open = false; page.openMenus--; frame.escapeStack.pop(root); page.showChrome() }
    // A navigation destroys the page and this picker with it, without ever calling close(),
    // so the count and the escape stack would keep an entry whose closer is gone. close()
    // is already guarded for the never-opened case, so this is safe unconditionally.
    Component.onDestruction: root.close()

    // The shield: any press outside the panel closes the picker, and no press, wheel or
    // hover reaches what is under it
    Blocker { anchors.fill: parent; scrim: false; dismissOnPress: true; onDismissed: root.close() }
    Corner {
        id: panel
        objectName: root.objectName !== "" ? root.objectName + "-panel" : "track-picker-panel"
        width: theme.space(70); height: Math.min(root.maxHeight, column.implicitHeight + theme.space(4))
        radius: theme.radiusMd; smoothing: theme.cornerSmoothing
        // One fill step above raised: it floats over a raised surface (the player's
        // controls tile, the rating prompt), and the same fill would merge with it
        color: theme.raisedFloat; borderColor: theme.lineStrong; borderWidth: 1
        // The list's length can change while open (tracks arrive late); keep the foot put
        onHeightChanged: if (root.open) root.place()
        MouseArea { anchors.fill: parent; acceptedButtons: Qt.AllButtons }
        Flickable {
            id: trackList
            anchors.fill: parent; anchors.margins: theme.space(2); clip: true
            contentWidth: width; contentHeight: column.implicitHeight
            boundsBehavior: Flickable.StopAtBounds
            QC.ScrollBar.vertical: ThinScrollBar {}
            WheelStep { flick: trackList }
            Column {
                id: column
                width: parent.width
                Text { text: root.title; color: theme.textDim; font.family: theme.fontSans; font.pointSize: theme.typeSmall; font.weight: Font.DemiBold; height: theme.space(6); verticalAlignment: Text.AlignVCenter; leftPadding: theme.space(2) }
                Repeater {
                    model: (root.offRow ? [{ id: -1, label: "Off" }] : []).concat(root.tracks)
                    Corner {
                        required property var modelData
                        width: column.width; height: theme.controlHeight
                        radius: theme.radiusSm; smoothing: theme.cornerSmoothing
                        color: m.containsMouse ? theme.tone(theme.raisedFloat, theme.text, 0.07) : "transparent"
                        Text { anchors.left: parent.left; anchors.leftMargin: theme.space(2); anchors.verticalCenter: parent.verticalCenter; text: modelData.label; color: modelData.id === root.selected ? theme.accent : theme.text; font.family: theme.fontSans; font.pointSize: theme.typeNormal; font.weight: modelData.id === root.selected ? Font.DemiBold : Font.Normal; elide: Text.ElideRight; width: parent.width - theme.space(4) }
                        MouseArea { id: m; anchors.fill: parent; hoverEnabled: true; cursorShape: Qt.PointingHandCursor; onClicked: { root.picked(modelData.id); root.close() } }
                    }
                }
            }
        }
    }
}
