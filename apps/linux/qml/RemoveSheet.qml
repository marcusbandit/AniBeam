// The one confirmation every removal on the downloads page goes through: a small sheet by
// the button that asked, saying what always goes, with up to two check marks for what can
// go with it. Unsubscribing always takes the feed and its rule; "Also remove the torrents"
// takes the client's torrents (the files stay); "Also delete the files" moves the files to
// the trash and takes the torrents with it, so ticking it ticks and locks the first. Without
// a subscription the base is the torrents and only the files can be added. Nothing is ever
// deleted for good here: files go to the trash, on their own drive.
//
// Opened through frame.askRemoval(anchor, spec, done). `spec`: { heading, base, rule (bool),
// torrentCount, canFiles, filesWhere, note }; `done` gets { torrents, files }. A click
// outside, Escape or Cancel closes it without a word; Return confirms.
import QtQuick
import com.marcusrosado.AniBeam

Item {
    id: root
    anchors.fill: parent
    visible: false
    property var spec: ({})
    property var done: null
    // What held the keyboard before the sheet took it, to hand it back
    property Item prevFocus: null
    property bool torrents: false
    property bool files: false
    // Where the asking button sat when the sheet opened, in the sheet's own coordinates: the
    // sheet is placed again whenever its height settles, and the button may be gone by then.
    property var anchorRect: null
    readonly property bool rule: spec.rule === true
    readonly property int torrentCount: spec.torrentCount || 0
    readonly property bool canTorrents: rule && torrentCount > 0
    readonly property bool canFiles: spec.canFiles === true
    readonly property string confirmText: {
        if (rule) return files ? "Unsubscribe and delete" : torrents ? "Unsubscribe and remove torrents" : "Unsubscribe"
        return files ? "Remove and delete files" : (torrentCount === 1 ? "Remove the torrent" : "Remove torrents")
    }

    function openAt(anchor, s, cb) {
        root.spec = s || {}
        root.done = cb
        root.torrents = false
        root.files = false
        root.prevFocus = frame.hostWindow ? frame.hostWindow.activeFocusItem : null
        if (anchor) {
            var p = anchor.mapToItem(root, 0, 0)
            root.anchorRect = { x: p.x, y: p.y, width: anchor.width, height: anchor.height }
        } else root.anchorRect = null
        root.visible = true
        panel.place()
        frame.escapeStack.push("sheet", root)
        panel.forceActiveFocus()
    }
    function close() {
        if (!root.visible) return
        root.visible = false
        root.done = null
        frame.escapeStack.pop(root)
        frame.focusPage(root, root.prevFocus)
        root.prevFocus = null
    }
    // Without a subscription the torrents are what goes, ticked or not; the check that adds
    // them is only there beside a subscription.
    function confirm() {
        var cb = root.done, choice = { torrents: !root.rule || root.torrents || root.files, files: root.files }
        root.close()
        if (cb) cb(choice)
    }

    // A press anywhere else closes it, and stops there
    MouseArea { anchors.fill: parent; acceptedButtons: Qt.AllButtons; onPressed: root.close(); onWheel: function(w) { root.close(); w.accepted = false } }

    Frost {
        id: panel
        width: Math.min(theme.space(120), root.width - theme.space(8))
        height: body.implicitHeight + theme.space(5) * 2
        radius: theme.radiusTile
        focus: true
        Keys.onReturnPressed: root.confirm()
        Keys.onEnterPressed: root.confirm()
        Keys.onEscapePressed: root.close()
        // Below the anchor, or above it when the window ends first; never over it unless
        // neither fits, and always wholly inside the window. Placed again when the height
        // settles: the Column reports its height only once it has laid out, so the first
        // placement can see a short sheet and pick a slot the real one runs out of.
        onHeightChanged: if (root.visible) place()
        function place() {
            var w = root.width, h = root.height, a = root.anchorRect, edge = theme.space(4)
            if (!a) { x = (w - width) / 2; y = Math.max(edge, (h - height) / 2); return }
            // From the anchor's left edge when the room is there, else ending at its right
            var left = a.x + width <= w - edge ? a.x : a.x + a.width - width
            x = Math.max(edge, Math.min(left, w - width - edge))
            var below = a.y + a.height + theme.space(2), above = a.y - height - theme.space(2)
            var top = below + height <= h - edge ? below : above >= edge ? above : h - height - edge
            y = Math.max(edge, top)
        }
        MouseArea { anchors.fill: parent }   // a press inside stays inside

        Column {
            id: body
            x: theme.space(5)
            y: theme.space(5)
            width: parent.width - theme.space(10)
            spacing: theme.space(3)
            Label { width: parent.width; text: root.spec.heading || ""; role: "title"; font.pointSize: theme.typeNormal * 1.08; wrapMode: Text.Wrap }
            Label { width: parent.width; visible: text !== ""; text: root.spec.base || ""; role: "ui"; color: theme.textDim; wrapMode: Text.Wrap }
            Check {
                visible: root.canTorrents
                checked: root.torrents || root.files
                locked: root.files
                text: root.torrentCount === 1 ? "Also remove its torrent" : "Also remove its " + root.torrentCount + " torrents"
                detail: root.files ? "Deleting the files takes the torrents with them." : "They leave the client; the files stay."
                onToggled: root.torrents = !root.torrents
            }
            Check {
                objectName: "remove-sheet-files"
                visible: root.canFiles
                checked: root.files
                text: "Also delete the files"
                detail: "Moves " + (root.spec.filesWhere || "what the torrents downloaded") + " to the trash" + (root.rule ? ", and removes the torrents." : ".")
                onToggled: root.files = !root.files
            }
            Label { width: parent.width; visible: text !== ""; text: root.spec.note || ""; role: "ui"; font.pointSize: theme.typeSmall; color: theme.textFaint; wrapMode: Text.Wrap }
            Item {
                width: parent.width
                height: theme.controlHeight
                Row {
                    anchors.right: parent.right
                    spacing: theme.space(2)
                    Button { objectName: "remove-sheet-cancel"; text: "Cancel"; flat: true; onClicked: root.close() }
                    Button { objectName: "remove-sheet-confirm"; text: root.confirmText; icon: root.files ? "trash-2" : root.rule ? "rss" : "x"; danger: true; onClicked: root.confirm() }
                }
            }
        }
    }

    // A check mark with its line and a dim line under it
    component Check: Item {
        id: check
        property bool checked: false
        property bool locked: false
        property string text: ""
        property string detail: ""
        signal toggled()
        width: parent ? parent.width : 0
        height: Math.max(box.height, words.implicitHeight) + theme.space(1)
        opacity: locked ? 0.7 : 1
        Rectangle {
            id: box
            y: theme.space(0.25)
            width: theme.space(5); height: width
            radius: theme.radiusSm
            color: check.checked ? theme.red : "transparent"
            border.color: check.checked ? theme.red : (cm.containsMouse ? theme.textDim : theme.lineStrong)
            border.width: 1.5
            Icon { visible: check.checked; anchors.centerIn: parent; glyph: "check"; size: theme.space(3.5); color: theme.contrast(theme.red, Qt.rgba(1, 1, 1, 1)) >= 4.5 ? "#ffffff" : "#0d0e11" }
        }
        Column {
            id: words
            anchors.left: box.right
            anchors.leftMargin: theme.space(2.5)
            anchors.right: parent.right
            spacing: theme.space(0.5)
            Label { width: parent.width; text: check.text; role: "ui"; color: theme.text; wrapMode: Text.Wrap }
            Label { width: parent.width; text: check.detail; role: "ui"; font.pointSize: theme.typeSmall; color: theme.textDim; wrapMode: Text.Wrap }
        }
        MouseArea { id: cm; anchors.fill: parent; hoverEnabled: true; cursorShape: check.locked ? Qt.ArrowCursor : Qt.PointingHandCursor; enabled: !check.locked; onClicked: check.toggled() }
    }
}
