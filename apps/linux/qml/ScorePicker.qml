// The score picker: 101 values from 0.0 to 10.0, Save, and Clear when a score exists.
// Opens under an anchor in the frame's overlay; Escape or a click outside closes it.
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
        var p = anchor.mapToItem(root, 0, anchor.height)
        panel.x = Math.min(p.x, root.width - panel.width - theme.space(2))
        panel.y = Math.min(p.y + theme.space(1), root.height - panel.height - theme.space(2))
        open = true
        frame.escapeStack.push("popover", root)
        list.positionViewAtIndex(Math.round(draft * 10), ListView.Center)
    }
    function close() { if (!open) return; open = false; frame.escapeStack.pop(root) }
    // A keyboard navigation (Alt+Left, Ctrl+K, Ctrl+comma) while open destroys the page,
    // and this picker with it, without ever calling close(); close() is already guarded
    // for the never-opened case, so this is safe to call unconditionally on destruction.
    Component.onDestruction: root.close()

    MouseArea { anchors.fill: parent; onPressed: root.close() }
    Corner {
        id: panel
        width: theme.space(48); height: theme.space(60)
        radius: theme.radiusMd; smoothing: theme.cornerSmoothing
        color: theme.surfaceRaised; borderColor: theme.lineStrong; borderWidth: 1
        MouseArea { anchors.fill: parent }
        ListView {
            id: list
            anchors.left: parent.left; anchors.right: parent.right; anchors.top: parent.top
            anchors.bottom: buttons.top; anchors.margins: theme.space(2)
            clip: true
            model: root.values
            delegate: Corner {
                required property string modelData
                width: list.width; height: theme.space(6)
                radius: theme.radiusSm; smoothing: theme.cornerSmoothing
                color: Number(modelData) === Number(root.draft.toFixed(1)) ? theme.accentSoft : (m.containsMouse ? theme.surfacePressed : "transparent")
                Text { anchors.centerIn: parent; text: modelData; color: Number(modelData) === Number(root.draft.toFixed(1)) ? theme.accent : theme.text; font.family: theme.fontMono; font.pointSize: theme.typeNormal }
                MouseArea { id: m; anchors.fill: parent; hoverEnabled: true; onClicked: root.draft = Number(modelData) }
            }
            QC.ScrollBar.vertical: ThinScrollBar {}
        }
        Row {
            id: buttons
            anchors.bottom: parent.bottom; anchors.right: parent.right; anchors.margins: theme.space(2)
            spacing: theme.space(2)
            Button { visible: root.current >= 0; text: "Clear"; danger: true; small: true; onClicked: { root.close(); root.cleared() } }
            Button { text: "Save"; small: true; onClicked: { root.close(); root.saved(root.draft) } }
        }
    }
}
