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
//
// A board (an item with `gridBoard: true`, Home's week) holds its stops in lanes, one a child
// of the board: columns, or rows when the board says `gridAcross: true`. Inside it the keys
// walk a lane and step to the next lane at the nearest stop, and past a lane's first or last
// stop they leave the board for the rows above or below it, since its lanes' stops never
// line up into rows of their own.
import QtQuick

QtObject {
    id: root
    required property Flickable flick
    // Space kept between the focused stop and the viewport's edge when it scrolls into view
    property real margin: theme.space(4)

    function isStop(it) { return !!it && it.gridStop === true }

    // Every shown stop under the content item (or under `under`), in tree order. A stop is a
    // leaf here: its own children are never a stop of the page's.
    function stops(under) {
        var out = [], stack = [under || flick.contentItem]
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
    // The stops in rows: grouped by top edge, each row sorted by x, the rows by y. `skip`
    // leaves out a board's stops, for the keys leaving it.
    function rows(skip) {
        var byY = {}, keys = []
        stops().forEach(function(it) {
            if (skip && boardOf(it) === skip) return
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
    function boardOf(it) {
        for (var p = it.parent; p && p !== flick.contentItem; p = p.parent) if (p.gridBoard === true) return p
        return null
    }
    // A board's lanes that hold a stop, each its stops in reading order
    function lanes(board) {
        var out = []
        for (var i = 0; i < board.children.length; i++) {
            var lane = board.children[i]
            if (!lane.visible) continue
            var rs = stops(lane).map(rectOf).sort(function(a, b) { return Math.round(a.y) - Math.round(b.y) || a.x - b.x })
            if (rs.length) out.push(rs)
        }
        return out
    }
    function moveOnBoard(board, from, key) {
        var all = lanes(board), li = -1, si = -1
        for (var l = 0; l < all.length && li < 0; l++)
            for (var k = 0; k < all[l].length; k++) if (all[l][k].item === from) { li = l; si = k; break }
        if (li < 0) return null
        var across = board.gridAcross === true
        var along = across ? (key === Qt.Key_Left || key === Qt.Key_Right) : (key === Qt.Key_Up || key === Qt.Key_Down)
        var step = key === Qt.Key_Right || key === Qt.Key_Down ? 1 : -1
        var me = rectOf(from)
        if (along) {
            var at = si + step
            if (at >= 0 && at < all[li].length) return all[li][at].item
            if (across) return null
        } else {
            var to = li + step
            if (to >= 0 && to < all.length) {
                var best = null
                all[to].forEach(function(r) {
                    var d = across ? Math.abs(r.cx - me.cx) : Math.abs(r.y + r.h / 2 - (me.y + me.h / 2))
                    if (!best || d < best.d) best = { d: d, item: r.item }
                })
                return best.item
            }
            if (!across) return null
        }
        // Off the board: the nearest stop in the first row past its edge
        var edge = rectOf(board), outside = rows(board), row = null
        if (step > 0) { for (var r = 0; r < outside.length && !row; r++) if (outside[r][0].y >= edge.y + edge.h) row = outside[r] }
        else { for (var q = outside.length - 1; q >= 0 && !row; q--) if (outside[q][0].y + outside[q][0].h <= edge.y) row = outside[q] }
        return row ? nearest(row, me.cx) : null
    }
    // The stop an arrow lands on from `from`, or null when there is none that way
    function move(from, key) {
        var board = boardOf(from)
        if (board) return moveOnBoard(board, from, key)
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
        // Into a board: the first stop of each lane from above, the last from below, so the
        // keys land at a lane's end rather than on whichever stop sits lowest
        var into = boardOf(all[next][0].item)
        if (into) {
            var ends = lanes(into).map(function(l) { return key === Qt.Key_Down ? l[0] : l[l.length - 1] })
            if (into.gridAcross === true) ends = key === Qt.Key_Down ? ends.slice(0, 1) : ends.slice(-1)
            return nearest(ends, rectOf(from).cx)
        }
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
