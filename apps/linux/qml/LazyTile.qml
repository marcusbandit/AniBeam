// A card that is built only when it is close to being seen (#289). Until then it is one
// empty Item of the card's size, so the page lays out exactly as it will look and the
// scroll bar is right from the first frame.
//
// The page's PageScroll hands out a budget of one viewport of card area each
// frame: tiles created while it lasts build at once, synchronously, so the first frame
// after a click, a search or a re-sort shows every card in view. The rest wait in the
// PageScroll's list, which measures them only after a frame has painted (when the layout
// is settled) and as the view scrolls, and builds those within a screen of the viewport:
// synchronously when already in view, asynchronously ahead of it. A built card stays
// built. Outside a PageScroll the tile builds at once, as a plain card would.
import QtQuick

Item {
    id: root
    property Component sourceComponent
    readonly property alias item: loader.item
    property var flick: null
    property bool seen: false
    // false for a tile that holds tiles (a section of cards): it builds at once while the
    // first frame's budget lasts but spends none of it, so the budget goes to its cards
    property bool spends: true

    function findFlick() {
        for (var p = parent; p; p = p.parent)
            if (p.takeLazy !== undefined && p.lazyAdd !== undefined) return p
        return null
    }
    // Asked by the PageScroll's sweep: `now` builds in this frame, else across the next few
    function build(now) {
        if (seen) return
        loader.asynchronous = !now
        seen = true
    }
    // A Repeater parents its delegate after the delegate completes, so the tile decides
    // when it first has a parent, still inside the same synchronous build
    property bool decided: false
    function decide() {
        if (decided || !parent) return
        decided = true
        flick = findFlick()
        // The page may not know its column width yet, so its cards are still 0 wide: a
        // card a little wider than the theme's stands in, since columns stretch to fill
        var guess = theme.posterWidth * 1.3
        var area = width * height > 0 ? width * height : guess * guess * 1.5
        if (!flick || flick.takeLazy(spends ? area : 0)) seen = true
        else flick.lazyAdd(root)
    }
    Component.onCompleted: decide()
    onParentChanged: decide()
    Component.onDestruction: if (flick && !seen) flick.lazyRemove(root)

    Loader {
        id: loader
        anchors.fill: parent
        active: root.seen
        asynchronous: false
        sourceComponent: root.sourceComponent
    }
}
