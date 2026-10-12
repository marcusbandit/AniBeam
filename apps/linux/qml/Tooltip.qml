// Hover text for one item. The tip itself is drawn by the frame in its overlay after a
// 600 ms hover intent, so it is never clipped by the item's own parent.
//
// An Item with a HoverHandler, not a MouseArea: a hovering MouseArea takes the hover from
// every sibling under it, so a button's own MouseArea declared before its Tooltip never saw
// containsMouse and its hover fill never showed. A HoverHandler reads the hover without
// taking it. It takes no presses either, so clicks reach whatever is under it.
//
// No tip while a popover is up (a dropdown's list, a picker, the menu): the click that
// opened it would otherwise bring the tip up over it 600 ms later.
import QtQuick

Item {
    id: root
    property string text: ""
    readonly property bool hovered: hh.hovered
    anchors.fill: parent

    function popoverUp() {
        if (typeof frame === "undefined" || !frame.escapeStack) return false
        var e = frame.escapeStack.entries
        for (var i = 0; i < e.length; i++) if (e[i].kind === "popover") return true
        return false
    }

    HoverHandler {
        id: hh
        onHoveredChanged: {
            if (hovered) { if (root.text !== "") intent.restart() }
            else { intent.stop(); frame.hideTip() }
        }
    }
    onTextChanged: if (hh.hovered && text === "") { intent.stop(); frame.hideTip() }
    // A tip already up when a popover opens (a click on the hovered button) goes
    Connections {
        target: hh.hovered && typeof frame !== "undefined" ? frame.escapeStack : null
        function onEntriesChanged() { if (root.popoverUp()) { intent.stop(); frame.hideTip() } }
    }
    Timer {
        id: intent
        interval: 600
        onTriggered: if (hh.hovered && root.text !== "" && !root.popoverUp()) frame.showTip(root, root.text)
    }
}
