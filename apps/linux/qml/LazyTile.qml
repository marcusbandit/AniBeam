// A card that is built only when it is close to being seen (#289). Until then it is one
// empty Item of the card's size, so the page lays out exactly as it will look and the
// scroll bar is right from the first frame.
//
// The page's PageScroll hands out a budget of about a screen and a tenth of area: tiles
// created while the budget lasts build at once, synchronously, so the first frame after a
// click already shows every card in view. The rest wait for that first frame (when the
// layout is settled) and then build as they come within one screen of the viewport,
// asynchronously, so a fast scroll never stalls on a row of cards. A built card stays
// built. Outside a PageScroll the tile builds at once, as a plain card would.
import QtQuick

Item {
    id: root
    property Component sourceComponent
    readonly property alias item: loader.item
    property var flick: null
    property bool seen: false
    property bool late: false

    function findFlick() {
        for (var p = parent; p; p = p.parent)
            if (p.lazyArmed !== undefined && p.takeLazy !== undefined) return p
        return null
    }
    // A Repeater parents its delegate after the delegate completes, so the tile decides
    // when it first has a parent, still inside the same synchronous build
    property bool decided: false
    function decide() {
        if (decided || !parent) return
        decided = true
        flick = findFlick()
        // The page may not know its column width yet, so its cards are still 0 wide: the
        // theme's card size stands in until they are laid out
        var guess = theme.posterWidth * 1.3
        var area = width * height > 0 ? width * height : guess * guess * 1.5
        if (!flick || flick.takeLazy(area)) seen = true
        else late = true
    }
    Component.onCompleted: decide()
    onParentChanged: decide()
    // Read only once the page has painted; contentHeight is named so a layout change asks
    // again, since mapToItem itself is not something a binding can follow
    readonly property bool near: {
        if (seen || !flick || !flick.lazyArmed || flick.contentHeight < 0) return false
        var y = root.mapToItem(flick.contentItem, 0, 0).y
        var reach = flick.height
        return y + height > flick.contentY - reach && y < flick.contentY + flick.height + reach
    }
    onNearChanged: if (near) seen = true

    Loader {
        id: loader
        anchors.fill: parent
        active: root.seen
        asynchronous: root.late
        sourceComponent: root.sourceComponent
    }
}
