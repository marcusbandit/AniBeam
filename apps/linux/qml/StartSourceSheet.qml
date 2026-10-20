// The confirm before AniBeam starts a service: a small sheet by the chip that asked, saying
// which service and what starting it does. Start runs the core's StartEndpoint call (the page
// owns that), and the sheet stays up while it waits, since a pod can take a minute to answer;
// a failure (not installed, did not start) shows here as a line and the button becomes Try
// again. A click outside, Escape or Cancel closes it; the job carries on without it.
//
// The page opens it with openAt(anchor, name) and drives `busy` and `error`; `startRequested`
// is the confirmed click.
import QtQuick
import com.marcusrosado.AniBeam

Item {
    id: root
    anchors.fill: parent
    visible: false
    property string name: ""
    property bool busy: false
    property string error: ""
    // What the service is called to a person; the endpoint's name is what the chip says
    readonly property string service: "Jackett"
    property Item prevFocus: null
    property var anchorRect: null
    signal startRequested(string name)

    function openAt(anchor, endpoint) {
        root.name = endpoint
        root.error = ""
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
        frame.escapeStack.pop(root)
        frame.focusPage(root, root.prevFocus)
        root.prevFocus = null
    }
    function start() { if (!root.busy) root.startRequested(root.name) }
    Component.onDestruction: frame.escapeStack.pop(root)

    // A press anywhere else closes it, and stops there
    MouseArea { anchors.fill: parent; acceptedButtons: Qt.AllButtons; onPressed: root.close(); onWheel: function(w) { root.close(); w.accepted = false } }

    Frost {
        id: panel
        objectName: "start-source-sheet"
        width: Math.min(theme.space(110), root.width - theme.space(8))
        height: body.implicitHeight + theme.space(5) * 2
        radius: theme.radiusTile
        focus: true
        Keys.onReturnPressed: root.start()
        Keys.onEnterPressed: root.start()
        Keys.onEscapePressed: root.close()
        onHeightChanged: if (root.visible) place()
        // Below the chip, or above it when the window ends first; always inside the window
        function place() {
            var w = root.width, h = root.height, a = root.anchorRect, edge = theme.space(4)
            if (!a) { x = (w - width) / 2; y = Math.max(edge, (h - height) / 2); return }
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
            Label { width: parent.width; text: "Start " + root.service + " for " + root.name + "?"; role: "title"; font.pointSize: theme.typeNormal * 1.08; wrapMode: Text.Wrap }
            Label {
                width: parent.width
                text: root.busy
                      ? "Starting " + root.service + ". " + root.name + " switches on when it answers; this can take up to a minute."
                      : root.service + " is the service that searches " + root.name + " for AniBeam. Starting it runs it in the background on this computer, and it keeps running after AniBeam closes."
                role: "ui"
                color: theme.textDim
                wrapMode: Text.Wrap
            }
            Label { width: parent.width; visible: root.error !== ""; text: root.error; role: "ui"; color: theme.red; wrapMode: Text.Wrap }
            Item {
                width: parent.width
                height: theme.controlHeight
                Row {
                    anchors.right: parent.right
                    spacing: theme.space(2)
                    Button { objectName: "start-source-cancel"; text: "Cancel"; flat: true; onClicked: root.close() }
                    Button {
                        objectName: "start-source-confirm"
                        text: root.busy ? "Starting" : root.error !== "" ? "Try again" : "Start"
                        icon: "play"
                        primary: true
                        enabled: !root.busy
                        onClicked: root.start()
                    }
                }
            }
        }
    }
}
