// A passing notice over the picture: one line, an optional button, gone after four seconds.
// The Skipped intro line with its Undo and the tracker outcomes after a mark all come
// through here. The line is capped at the page's width and elides, since a tracker's own
// error text is as long as the tracker cares to make it. A pane of the player's glass, like every island on
// the player; its Undo never takes the keyboard, so Space after it still pauses.
import QtQuick

Corner {
    id: root
    property string text: ""
    property string action: ""
    signal acted()

    objectName: "player-notice"
    visible: false
    anchors.horizontalCenter: parent.horizontalCenter
    anchors.bottom: parent.bottom; anchors.bottomMargin: theme.space(30)
    width: row.implicitWidth + theme.space(6); height: theme.controlHeight
    color: theme.glassVideo
    borderColor: Qt.rgba(1, 1, 1, 0.1); borderWidth: 1
    highlight: theme.glassHighlight

    readonly property real maxWidth: parent ? parent.width - theme.space(8) : theme.space(100)
    Row {
        id: row
        anchors.centerIn: parent; spacing: theme.space(3)
        Label {
            anchors.verticalCenter: parent.verticalCenter
            role: "ui"
            // The button's own width is what is left over, so the line takes the rest.
            width: Math.min(implicitWidth, root.maxWidth - theme.space(6) - (act.visible ? act.width + theme.space(3) : 0))
            elide: Text.ElideRight
            text: root.text
        }
        Button { id: act; anchors.verticalCenter: parent.verticalCenter; visible: root.action !== ""; text: root.action; small: true; flat: true; takesFocus: false; onClicked: { root.hide(); root.acted() } }
    }
    // The pointer on the line holds it, so a reach for its button is never too late
    HoverHandler { id: hold }
    Timer { id: life; interval: 4000; onTriggered: if (hold.hovered) restart(); else root.visible = false }
    function show(text, action, seconds) { root.text = text; root.action = action || ""; life.interval = (seconds || 4) * 1000; visible = true; life.restart() }
    function hide() { visible = false; life.stop() }
}
