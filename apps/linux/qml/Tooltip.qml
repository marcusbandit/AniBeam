// Hover text for one item. The tip itself is drawn by the frame in its overlay after a
// 600 ms hover intent, so it is never clipped by the item's own parent.
//
// A HoverHandler on the item it describes, not a MouseArea over it. A hovering MouseArea
// takes the hover from every sibling under it, so a button's own MouseArea declared before
// its Tooltip never saw containsMouse and its hover fill never showed. A HoverHandler laid
// over it as a sibling still did (measured: an overlapping Item with a HoverHandler kept the
// MouseArea under it at containsMouse false), so this item covers nothing at all and its
// handler sits on the parent: hover reaches every ancestor of what is under the pointer, so
// the parent's handler sees it whichever child takes it.
//
// No tip while a popover is up (a dropdown's list, a picker, the menu): the click that
// opened it would otherwise bring the tip up over it 600 ms later.
import QtQuick

Item {
    id: root
    property string text: ""
    readonly property bool hovered: hh.hovered
    readonly property Item target: parent
    // The frame that draws the tip. Under Main's tokens page `frame` is Main's Loader, not
    // the Frame, so the Frame is its item there.
    readonly property var host: typeof frame === "undefined" || !frame ? null
        : frame.showTip ? frame : (frame.item && frame.item.showTip ? frame.item : null)
    width: 0; height: 0

    function popoverUp() {
        if (!host || !host.escapeStack) return false
        var e = host.escapeStack.entries
        for (var i = 0; i < e.length; i++) if (e[i].kind === "popover") return true
        return false
    }
    function hide() { intent.stop(); if (host) host.hideTip() }

    HoverHandler {
        id: hh
        parent: root.target
        onHoveredChanged: if (hovered) { if (root.text !== "") intent.restart() } else root.hide()
    }
    onTextChanged: if (hh.hovered && text === "") hide()
    // A tip already up when a popover opens (a click on the hovered button) goes
    Connections {
        target: hh.hovered && root.host ? root.host.escapeStack : null
        function onEntriesChanged() { if (root.popoverUp()) root.hide() }
    }
    Timer {
        id: intent
        interval: 600
        onTriggered: if (hh.hovered && root.host && root.text !== "" && !root.popoverUp()) root.host.showTip(root.target, root.text)
    }
}
