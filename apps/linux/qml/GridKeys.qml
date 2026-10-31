// Arrow keys across a page's grids. PageScroll owns one of these, so every page that sits on
// it gets the same keys from the same code. A stop is any visible item in the scroll's
// content that declares `gridStop: true` (a Card, Home's Resume button); nothing is
// registered, the content tree is read at the key press, so a reflowed grid, a hidden shelf
// or a new section is simply what is there at that moment.
//
// Left and Right go to the previous and next stop among the same parent's children (a
// Flow's order, so the end of a row runs on to the next). Up and Down go to the row above
// or below, whichever stop there is nearest in x by centre; rows are the stops grouped by
// their top edge in content coordinates, so a section boundary, a day row of the week and
// the hero's button are all just rows. Nothing is hardcoded about columns. With no stop
// focused (the page or the scroll itself has focus), Down and Right go to the first stop.
// Escape on a stop hands focus to the scroll, so the page keeps focus with no card lit.
import QtQuick

QtObject {
    id: root
    required property Flickable flick
    // Space kept between the focused stop and the viewport's edge when it scrolls into view
    property real margin: theme.space(4)

    function isStop(it) { return !!it && it.gridStop === true }

    // Every shown stop under the content item, in tree order. A stop is a leaf here: its
    // own children are never a stop of the page's.
    function stops() {
        var out = [], stack = [flick.contentItem]
        while (stack.length) {
            var it = stack.pop()
            if (isStop(it)) { if (it.width > 0 && it.height > 0) out.push(it); continue }
            var kids = it.children
            for (var i = kids.length - 1; i >= 0; i--) if (kids[i].visible) stack.push(kids[i])
        }
        return out
    }
    function rectOf(it) {
        var p = it.mapToItem(flick.contentItem, 0, 0)
        return { item: it, x: p.x, y: p.y, w: it.width, h: it.height, cx: p.x + it.width / 2 }
    }
    // The stops in rows: grouped by top edge, each row sorted by x, the rows by y
    function rows() {
        var byY = {}, keys = []
        stops().forEach(function(it) {
            var r = rectOf(it), k = Math.round(r.y)
            if (!byY[k]) { byY[k] = []; keys.push(k) }
            byY[k].push(r)
        })
        keys.sort(function(a, b) { return a - b })
        return keys.map(function(k) { return byY[k].sort(function(a, b) { return a.x - b.x }) })
    }
    function nearest(row, cx) {
        var best = null
        row.forEach(function(r) { if (!best || Math.abs(r.cx - cx) < Math.abs(best.cx - cx)) best = r })
        return best ? best.item : null
    }
    // The stop an arrow lands on from `from`, or null when there is none that way
    function move(from, key) {
        if (key === Qt.Key_Left || key === Qt.Key_Right) {
            // A card a LazyTile built sits inside the tile's Loader, so its row is the
            // tile's siblings, each standing for the card it holds (#289)
            var sib = []
            var slot = from.parent && from.parent.parent && from.parent.parent.build !== undefined && from.parent.parent.item === from ? from.parent.parent : from
            var kids = slot.parent ? slot.parent.children : []
            for (var i = 0; i < kids.length; i++) {
                var k = kids[i].build !== undefined && kids[i].sourceComponent !== undefined ? kids[i].item : kids[i]
                if (k && isStop(k) && k.visible) sib.push(k)
            }
            var at = sib.indexOf(from) + (key === Qt.Key_Right ? 1 : -1)
            return at >= 0 && at < sib.length ? sib[at] : null
        }
        var all = rows(), here = -1
        for (var r = 0; r < all.length && here < 0; r++)
            for (var c = 0; c < all[r].length; c++) if (all[r][c].item === from) { here = r; break }
        if (here < 0) return null
        var next = here + (key === Qt.Key_Down ? 1 : -1)
        if (next < 0 || next >= all.length) return null
        return nearest(all[next], rectOf(from).cx)
    }
    function first() { var s = stops(); return s.length ? s[0] : null }

    // Scrolls so the stop sits wholly inside the viewport, with the margin, through the
    // scroll's own smoothed target
    function reveal(it) {
        var r = rectOf(it)
        var top = r.y - margin, bottom = r.y + r.h + margin
        // Measured from where a glide under way will end, not from where it is now
        var at = flick.scrollTarget !== undefined ? flick.scrollTarget : flick.contentY
        var y = at
        if (top < y) y = top
        else if (bottom > y + flick.height) y = bottom - flick.height
        if (y !== at) flick.scrollTo(y)
    }
    function go(it) {
        if (!it) return false
        it.forceActiveFocus()
        reveal(it)
        return true
    }

    // The key handler: true (and the event accepted) when the key moved focus
    function handle(event, focused) {
        var k = event.key
        var onStop = isStop(focused)
        if (k === Qt.Key_Escape) {
            if (!onStop) return false
            flick.forceActiveFocus()
            event.accepted = true
            return true
        }
        var arrow = k === Qt.Key_Left || k === Qt.Key_Right || k === Qt.Key_Up || k === Qt.Key_Down
        if (!arrow || event.modifiers !== Qt.NoModifier) return false
        var moved = false
        if (onStop) moved = go(move(focused, k))
        else if (focused === flick || focused === flick.parent || (focused && focused.scrollY !== undefined))
            moved = (k === Qt.Key_Down || k === Qt.Key_Right) && go(first())
        if (moved) event.accepted = true
        return moved
    }
}
