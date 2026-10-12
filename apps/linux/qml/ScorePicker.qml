// The score picker: 101 values from 0.0 to 10.0, Save, and Clear when a score exists.
// Opens under an anchor in the frame's overlay; Escape or a click outside closes it.
// Up and Down move a tenth, Page Up and Page Down a point, Return saves.
import QtQuick
import QtQuick.Controls.Basic as QC

Item {
    id: root
    property bool open: false
    property real current: -1
    property real draft: 8.0
    signal saved(real value)
    signal cleared()
    anchors.fill: parent
    visible: open
    z: 900

    readonly property var values: { var v = []; for (var i = 0; i <= 100; i++) v.push((i / 10).toFixed(1)); return v }
    function openAt(anchor, currentScore) {
        current = currentScore
        draft = currentScore >= 0 ? currentScore : 8.0
        // Under the anchor, or under the whole raised tile it sits on (the player's rating
        // prompt), so the picker never covers the tile's own foot
        var foot = anchor
        for (var a = anchor.parent; a; a = a.parent) if (a.raised === true) { foot = a; break }
        var px = anchor.mapToItem(root, 0, anchor.height).x
        var py = foot.mapToItem(root, 0, foot.height).y
        panel.x = Math.max(theme.space(2), Math.min(px, root.width - panel.width - theme.space(2)))
        panel.y = Math.max(theme.space(2), Math.min(py + theme.space(1), root.height - panel.height - theme.space(2)))
        if (!open) prevFocus = frame.hostWindow ? frame.hostWindow.activeFocusItem : null
        open = true
        frame.escapeStack.push("popover", root)
        list.positionViewAtIndex(Math.round(draft * 10), ListView.Center)
        root.forceActiveFocus()
    }
    function close() { if (!open) return; open = false; frame.escapeStack.pop(root); frame.focusPage(root, prevFocus); prevFocus = null }

    // The keyboard, the way the frame menu takes it: Up and Down move the highlight a row
    // (a tenth), Page Up and Page Down a whole point, Home and End to the ends, Return
    // saves. Escape propagates to the frame, which closes the picker.
    property Item prevFocus: null
    function step(d) {
        draft = Math.max(0, Math.min(10, Math.round(draft * 10 + d) / 10))
        list.positionViewAtIndex(Math.round(draft * 10), ListView.Contain)
    }
    function save() { close(); saved(draft) }
    Keys.onUpPressed: step(-1)
    Keys.onDownPressed: step(1)
    Keys.onReturnPressed: save()
    Keys.onEnterPressed: save()
    Keys.onPressed: function(e) {
        if (e.key === Qt.Key_PageUp) step(-10)
        else if (e.key === Qt.Key_PageDown) step(10)
        else if (e.key === Qt.Key_Home) step(-100)
        else if (e.key === Qt.Key_End) step(100)
        else return
        e.accepted = true
    }
    // A keyboard navigation (Alt+Left, Ctrl+K, Ctrl+comma) while open destroys the page,
    // and this picker with it, without ever calling close(); close() is already guarded
    // for the never-opened case, so this is safe to call unconditionally on destruction.
    Component.onDestruction: root.close()

    // The shield: any press outside the panel closes the picker, and no press, wheel or
    // hover reaches what is under it
    Blocker { anchors.fill: parent; scrim: false; dismissOnPress: true; onDismissed: root.close() }
    Corner {
        id: panel
        objectName: "score-picker-panel"
        width: theme.space(48); height: theme.space(60)
        radius: theme.radiusMd; smoothing: theme.cornerSmoothing
        color: theme.surfaceRaised; borderColor: theme.lineStrong; borderWidth: 1
        MouseArea { anchors.fill: parent; acceptedButtons: Qt.AllButtons }
        ListView {
            id: list
            anchors.left: parent.left; anchors.right: parent.right; anchors.top: parent.top
            anchors.bottom: buttons.top; anchors.margins: theme.space(2)
            clip: true
            model: root.values
            delegate: Corner {
                required property string modelData
                required property int index
                objectName: "score-picker-row-" + index
                width: list.width; height: theme.space(6)
                radius: theme.radiusSm; smoothing: theme.cornerSmoothing
                color: Number(modelData) === Number(root.draft.toFixed(1)) ? theme.accentSoft : (m.containsMouse ? theme.surfacePressed : "transparent")
                Text { anchors.centerIn: parent; text: modelData; color: Number(modelData) === Number(root.draft.toFixed(1)) ? theme.accent : theme.text; font.family: theme.fontUi; font.pointSize: theme.typeNormal }
                MouseArea { id: m; anchors.fill: parent; hoverEnabled: true; onClicked: root.draft = Number(modelData) }
            }
            QC.ScrollBar.vertical: ThinScrollBar {}
        }
        Row {
            id: buttons
            anchors.bottom: parent.bottom; anchors.right: parent.right; anchors.margins: theme.space(2)
            spacing: theme.space(2)
            Button { visible: root.current >= 0; text: "Clear"; danger: true; small: true; onClicked: { root.close(); root.cleared() } }
            Button { text: "Save"; small: true; onClicked: root.save() }
        }
    }
}
