// The delays panel: subtitle delay and audio delay, each a step down, the value mpv reports, a
// step up and a reset to 0. It opens above its button the way TrackPicker does and stays up
// while the steps are clicked; a press outside or Escape closes it. The buttons never take
// the keyboard, and the page takes it back when the panel closes, so the keys work at once.
import QtQuick

Item {
    id: root
    property bool open: false
    anchors.fill: parent
    visible: open
    z: 900

    // Kept and placed again on every size change while open, as TrackPicker's anchor is.
    property var anchorItem: null
    function tileOf(item) {
        for (var p = item; p; p = p.parent) if (p.objectName === "player-controls") return p
        return null
    }
    function place() {
        if (!anchorItem) return
        var p = anchorItem.mapToItem(root, anchorItem.width / 2, 0)
        var tile = tileOf(anchorItem)
        var floor = (tile ? tile.mapToItem(root, 0, 0).y : p.y) - theme.space(2)
        panel.x = Math.max(theme.space(2), Math.min(p.x - panel.width / 2, root.width - panel.width - theme.space(2)))
        panel.y = Math.max(theme.space(2), floor - panel.height)
    }
    onWidthChanged: if (open) Qt.callLater(root.place)
    onHeightChanged: if (open) Qt.callLater(root.place)
    function openAt(anchor) {
        if (open) return
        anchorItem = anchor
        place()
        open = true
        page.openMenus++
        frame.escapeStack.push("popover", root)
    }
    function close() { if (!open) return; open = false; page.openMenus--; frame.escapeStack.pop(root); page.showChrome() }
    Component.onDestruction: root.close()

    Blocker { anchors.fill: parent; scrim: false; dismissOnPress: true; onDismissed: root.close() }
    Corner {
        id: panel
        objectName: "player-delays-panel"
        width: column.implicitWidth + theme.space(4) * 2; height: column.implicitHeight + theme.space(2) * 2
        color: theme.glassFloat; borderColor: theme.lineStrong; borderWidth: 1; highlight: theme.glassHighlight
        MouseArea { anchors.fill: parent; acceptedButtons: Qt.AllButtons }
        Column {
            id: column
            anchors.centerIn: parent
            Text { text: "Delay"; color: theme.textDim; font.family: theme.fontSans; font.pointSize: theme.typeSmall; font.weight: Font.DemiBold; height: theme.space(6); verticalAlignment: Text.AlignVCenter }
            Repeater {
                model: [{ prop: "sub-delay", key: "sub", label: "Subtitles" },
                        { prop: "audio-delay", key: "audio", label: "Audio" }]
                Row {
                    id: delayRow
                    required property var modelData
                    readonly property real value: modelData.prop === "audio-delay" ? page.audioDelay : page.subDelay
                    spacing: theme.space(1)
                    Label { anchors.verticalCenter: parent.verticalCenter; width: theme.space(20); role: "ui"; text: delayRow.modelData.label }
                    PlayerButton { objectName: "player-" + delayRow.modelData.key + "-delay-down"; glyph: "minus"; tip: delayRow.modelData.label + " 0.1 s earlier"
                        onClicked: page.nudgeDelay(delayRow.modelData.prop, -0.1) }
                    Label { anchors.verticalCenter: parent.verticalCenter; width: theme.space(16); horizontalAlignment: Text.AlignHCenter; role: "ledger"; text: page.delayText(delayRow.value) }
                    PlayerButton { objectName: "player-" + delayRow.modelData.key + "-delay-up"; glyph: "plus"; tip: delayRow.modelData.label + " 0.1 s later"
                        onClicked: page.nudgeDelay(delayRow.modelData.prop, 0.1) }
                    PlayerButton { objectName: "player-" + delayRow.modelData.key + "-delay-reset"; glyph: "rotate-ccw"; interactive: delayRow.value !== 0
                        tip: interactive ? "Reset to 0 until the next file" : "No delay"; onClicked: page.resetDelay(delayRow.modelData.prop) }
                }
            }
        }
    }
}
