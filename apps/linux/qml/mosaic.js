.pragma library
// The library's mosaic, as plain arithmetic: the column module, which series earn a hero
// tile, and a first-fit skyline packer. No Qt in here, so test-mosaic.mjs runs it under node
// and a --drive eval can call it on the live page.
//
// Units: a column is one cell width; a row is one half-row unit u (DESIGN.md, Layout). A tile
// spanning k rows is k*u + (k-1)*grout tall, so every edge on the page lands on the module.

// The column geometry for a width: the count, then each column's x and width in whole
// pixels, the remainder handed out one pixel at a time from the left so the right edge is
// exact. u is half the base cell width.
function columns(width, cellMin, grout) {
    var cols = Math.max(1, Math.floor((width + grout) / (cellMin + grout)))
    var free = Math.max(0, width - (cols - 1) * grout)
    var base = Math.floor(free / cols)
    var rem = free - base * cols
    var xs = [], ws = [], x = 0
    for (var c = 0; c < cols; c++) {
        var w = base + (c < rem ? 1 : 0)
        xs.push(x); ws.push(w)
        x += w + grout
    }
    return { cols: cols, xs: xs, ws: ws, w: base, u: Math.round(base * 0.5) }
}

// Pixel extent of a span of units starting at unit 0
function unitsToPx(k, u, grout) { return k <= 0 ? 0 : k * u + (k - 1) * grout }
// The smallest whole number of units whose height holds `px`
function unitsFor(px, u, grout) { return Math.max(1, Math.ceil((px + grout) / (u + grout))) }

// Heroes per section: a third of the columns, one at least from four columns, none below
function heroCap(cols) { return cols < 4 ? 0 : Math.max(1, Math.floor(cols / 3)) }

// Does a series earn a hero tile? In progress (behind, or started and not caught up) and
// viewed in the last 14 days, or its next episode airs in the next 7. Returns the rank key
// (bigger first) or -1. Recently watched outrank airing-only, most recent first; airing-only
// go soonest first.
var DAY = 86400
function heroRank(c, nowSecs) {
    var watched = c.watched !== null && c.watched !== undefined ? c.watched : 0
    var inProgress = c.watched_state === "Behind" || (watched > 0 && c.watched_state !== "CaughtUp")
    var viewed = c.last_viewed_at ? c.last_viewed_at : 0
    if (inProgress && viewed > 0 && nowSecs - viewed <= 14 * DAY) return 2e10 + viewed
    var at = c.next_airing && c.next_airing.at ? c.next_airing.at : 0
    if (at > nowSecs && at - nowSecs <= 7 * DAY) return 1e10 - (at - nowSecs)
    return -1
}

// The section's hero candidates: the qualifying cards, best first, capped; `taken` (an
// object keyed by id), when given, skips series already picked and records the new ones.
// pack() takes them in order, sets each series large at most once on the wall, and
// demotes the tail while the section would leave a hole.
function pickHeroes(cards, nowSecs, cap, taken) {
    if (cap <= 0) return []
    var ranked = []
    for (var i = 0; i < cards.length; i++) {
        var c = cards[i]
        if (taken && taken[c.id]) continue
        var r = heroRank(c, nowSecs)
        if (r >= 0) ranked.push({ r: r, c: c })
    }
    ranked.sort(function(a, b) { return b.r - a.r })
    var out = []
    for (var j = 0; j < ranked.length && out.length < cap; j++) {
        out.push(ranked[j].c)
        if (taken) taken[ranked[j].c.id] = true
    }
    return out
}

// First-fit skyline packing of the whole wall. sections: [{ label, count, heroes: [card],
// cards: [card] }] where `cards` may include the heroes (they are skipped there). The wall is
// one continuous skyline: a section's label stone goes where the previous section's last
// tile left off (no fresh row, no section-end gap), then its heroes, then its small tiles,
// each at the lowest row its span fits, left-most on ties. Because the skyline only grows
// and a section's span-1 tiles are placed after its label, a later section's tiles never
// precede an earlier one's in reading order. A section with an empty label has no stone.
// Heroes past the cap, or with fewer than two columns, are set as small tiles; while the
// wall would leave a hole (see firstHole()), the least important hero of the section that
// caused it is set small and the wall is packed again. Only the very last row may run
// short. Returns { tiles: [{ kind, section, card, col, row, colSpan, rowSpan }] sorted by
// row then col, rows: the total height in units }.
function pack(sections, cols, unitsSmall, unitsHero) {
    cols = Math.max(1, cols)
    var cap = cols >= 2 ? heroCap(cols) : 0
    var allowed = []
    for (var a = 0; a < sections.length; a++) allowed.push(cap)

    function packWall() {
        var sky = []
        for (var c = 0; c < cols; c++) sky.push(0)
        var tiles = []
        var used = []
        // The reading-order floor: no tile may sit before the last tile of an earlier
        // section (row, then column), so sections never interleave. A free cell the floor
        // skips is a hole, and firstHole() sends the wall back for a demotion.
        var floorRow = -1, floorCol = -1, lastRow = -1, lastCol = -1
        function place(kind, si, card, span, height) {
            span = Math.min(span, cols)
            var best = -1, bestRow = Infinity
            for (var x = 0; x + span <= cols; x++) {
                var y = 0
                for (var k = x; k < x + span; k++) y = Math.max(y, sky[k])
                if (y < floorRow || (y === floorRow && x <= floorCol)) y = x > floorCol ? floorRow : floorRow + unitsSmall
                if (y < bestRow) { bestRow = y; best = x }
            }
            if (bestRow > lastRow || (bestRow === lastRow && best > lastCol)) { lastRow = bestRow; lastCol = best }
            for (var m = best; m < best + span; m++) sky[m] = bestRow + height
            tiles.push({ kind: kind, section: si, card: card, col: best, row: bestRow, colSpan: span, rowSpan: height })
        }
        // A series is a hero once on the wall: the first section that sets it large keeps it
        var heroed = {}
        for (var si = 0; si < sections.length; si++) {
            var s = sections[si]
            floorRow = lastRow; floorCol = lastCol
            if (s.label) place("label", si, null, 1, unitsSmall)
            var heroes = (s.heroes || []).filter(function(h) { return !heroed[h.id] }).slice(0, allowed[si])
            used.push(heroes.length)
            var heroIds = {}
            for (var h = 0; h < heroes.length; h++) {
                heroIds[heroes[h].id] = heroed[heroes[h].id] = true
                place("hero", si, heroes[h], 2, unitsHero)
            }
            var cards = s.cards || []
            var inCards = {}
            for (var i = 0; i < cards.length; i++) {
                inCards[cards[i].id] = true
                if (!heroIds[cards[i].id]) place("small", si, cards[i], 1, unitsSmall)
            }
            // demoted heroes (past the cap, or to close a hole) that were not in cards
            var extra = s.heroes || []
            for (var e = 0; e < extra.length; e++)
                if (!heroIds[extra[e].id] && !inCards[extra[e].id]) place("small", si, extra[e], 1, unitsSmall)
        }
        var rows = 0
        for (var r = 0; r < cols; r++) rows = Math.max(rows, sky[r])
        return { tiles: tiles, rows: rows, used: used }
    }

    var out
    for (;;) {
        out = packWall()
        var bad = firstHole(out.tiles, cols, 0, unitsSmall)
        if (bad < 0) break
        // Demote in the latest section with a hero at or above the first hole: that is the
        // hero whose span opened it (an earlier one would have shown a hole sooner)
        var pick = -1
        for (var t = 0; t < out.tiles.length; t++) {
            var tt = out.tiles[t]
            if (tt.kind === "hero" && tt.row <= bad && tt.section > pick) pick = tt.section
        }
        // (or, when the floor skipped cells above a later hero, the latest section with one)
        if (pick < 0) for (var q = 0; q < out.used.length; q++) if (out.used[q] > 0) pick = q
        if (pick < 0) break   // no heroes left: span-1 tiles cannot leave a hole
        allowed[pick] = out.used[pick] - 1
    }
    out.tiles.sort(function(p, q) { return p.row - q.row || p.col - q.col })
    return { tiles: out.tiles, rows: out.rows }
}

// The first unit row at which the tiles leave a hole, or -1. Every cell above the final
// small row must be filled, and the final row may only run short on the right (a ragged
// last row, filled left to right). `tiles` start at unit row `top`.
function firstHole(tiles, cols, top, unitsSmall) {
    var bottom = top
    for (var i = 0; i < tiles.length; i++) bottom = Math.max(bottom, tiles[i].row + tiles[i].rowSpan)
    var taken = {}
    for (var j = 0; j < tiles.length; j++) {
        var t = tiles[j]
        for (var x = t.col; x < t.col + t.colSpan; x++)
            for (var y = t.row; y < t.row + t.rowSpan; y++) taken[x + ":" + y] = true
    }
    var finalRow = bottom - unitsSmall
    for (var r = top; r < bottom; r++) {
        var gap = false
        for (var c = 0; c < cols; c++) {
            var full = taken[c + ":" + r] === true
            if (!full && r < finalRow) return r
            if (!full) gap = true
            else if (gap) return r
        }
    }
    return -1
}
function holes(tiles, cols, top, unitsSmall) { return firstHole(tiles, cols, top, unitsSmall) >= 0 }

// Do any two tiles share a cell? (the drive script's check, and the test's)
function overlaps(tiles) {
    var seen = {}
    for (var i = 0; i < tiles.length; i++) {
        var t = tiles[i]
        for (var x = t.col; x < t.col + t.colSpan; x++)
            for (var y = t.row; y < t.row + t.rowSpan; y++) {
                var key = x + ":" + y
                if (seen[key] !== undefined) return true
                seen[key] = i
            }
    }
    return false
}

// The header: tiles of whole column spans laid into rows of `cols`, wrapping when a tile
// does not fit; each row's spare columns go to its flexible tile (the first one flagged
// `flex` in the row, else the row's first). items: [{ key, span, flex }] in order.
// Returns [{ key, col, row, colSpan }] and the row count.
function header(items, cols) {
    var rows = [[]], used = 0
    for (var i = 0; i < items.length; i++) {
        var span = Math.min(cols, Math.max(1, items[i].span))
        if (used + span > cols && rows[rows.length - 1].length) { rows.push([]); used = 0 }
        rows[rows.length - 1].push({ key: items[i].key, span: span, flex: !!items[i].flex })
        used += span
    }
    var out = []
    for (var r = 0; r < rows.length; r++) {
        var row = rows[r], total = 0, f = -1
        for (var j = 0; j < row.length; j++) { total += row[j].span; if (f < 0 && row[j].flex) f = j }
        if (f < 0) f = 0
        row[f].span += cols - total
        var col = 0
        for (var k = 0; k < row.length; k++) {
            out.push({ key: row[k].key, col: col, row: r, colSpan: row[k].span })
            col += row[k].span
        }
    }
    return { tiles: out, rows: rows.length }
}

// ---- Boxes (DESIGN.md, Layout: Arrange). The library arranged into framed, labelled boxes,
// SpaceSniffer's lesson taken for grouping, not for style: each group of two or more is a
// rigid rectangle with a slim header strip across its top and its members tiled under it,
// boxes and lone tiles packed edge to edge on one wall.
//
// The wall counts in slots: one column by one small tile's height (unitsSmall units). A box
// is c columns by k slots, so its outer edges stay on the module beside the lone tiles. Its
// strip (one strip unit, the same for every box) is cut from that height and the members
// share the rest, so a member stands a few pixels shorter than a lone tile. The members fill
// the box exactly: with a hero (2x2) only when it makes an exact fit, else spread over the k
// rows as evenly as they go (fuller rows first), a short row's tiles widened to the box's
// width. That widening is the box's "stretch", kept small by the shape cost, so no box has a
// hole and none needs a filler cell.

// A box's inner layout at width c: rows of members, x and w in columns (fractions where a
// row is stretched), row and rows in inner rows. null when the shape does not hold.
function boxLayout(n, hero, c) {
    var area = n + (hero ? 3 : 0)
    if (c < 1 || n < 1) return null
    var k = Math.ceil(area / c)
    var cells = []
    if (hero) {
        if (c < 2 || k < 2 || c * k !== area) return null
        cells.push({ kind: "hero", x: 0, w: 2, row: 0, rows: 2 })
        var grid = {}
        grid["0:0"] = grid["0:1"] = grid["1:0"] = grid["1:1"] = true
        for (var r = 0; r < k; r++)
            for (var x = 0; x < c; x++)
                if (!grid[r + ":" + x]) cells.push({ kind: "small", x: x, w: 1, row: r, rows: 1 })
        return { c: c, k: k, hero: true, stretch: 1, cells: cells }
    }
    var base = Math.floor(n / k), extra = n % k, stretch = 1
    for (var row = 0; row < k; row++) {
        var m = base + (row < extra ? 1 : 0)
        if (m < 1) return null
        stretch = Math.max(stretch, c / m)
        for (var i = 0; i < m; i++) cells.push({ kind: "small", x: i * c / m, w: c / m, row: row, rows: 1 })
    }
    return { c: c, k: k, hero: false, stretch: stretch, cells: cells }
}

// A shape's cost: how far its pixel rectangle is from a square, plus its stretch (a widened
// member crops its poster), less a little for a hero. geo: { slotW, slotH, grout } in pixels.
var STRETCH_COST = 1.5, HERO_BONUS = 0.15, BAND_COST = 1
function shapeCost(L, geo) {
    var W = L.c * geo.slotW + (L.c - 1) * geo.grout
    var H = L.k * geo.slotH + (L.k - 1) * geo.grout
    return Math.abs(Math.log(W / H)) + STRETCH_COST * (L.stretch - 1) - (L.hero ? HERO_BONUS : 0)
}
// A box's shapes at most maxC wide, best first
function boxShapes(n, heroOk, maxC, geo) {
    var out = []
    var tries = heroOk ? [true, false] : [false]
    for (var t = 0; t < tries.length; t++) {
        var area = n + (tries[t] ? 3 : 0)
        for (var c = 1; c <= Math.min(maxC, area); c++) {
            var L = boxLayout(n, tries[t], c)
            if (!L || L.stretch > MAX_STRETCH) continue
            L.score = shapeCost(L, geo)
            out.push(L)
        }
    }
    out.sort(function(a, b) { return a.score - b.score || a.c - b.c || (a.hero ? -1 : 1) })
    if (!out.length) out.push(boxLayout(n, false, Math.min(maxC, n)))
    return out
}
function boxShape(n, heroOk, maxC, geo) { return boxShapes(n, heroOk, maxC, geo)[0] }

// Pack the arranged wall. items, in reading order: [{ cards: [card], box: bool, hero: card
// or null, heroRank: number }]. An item is a box when `box` is set or it holds two or more
// cards; a box's `hero` is its one hero candidate (cap 1 per box, and only from four
// columns), a lone tile's `hero` sets it 2x2 (a third of the columns at most, the best
// ranked kept). geo sizes a slot in pixels for the box shapes.
//
// The packer is a skyline that always fills the lowest, left-most run of columns, so every
// placement sits on a flat run and nothing is ever left under a tile. Each step takes a
// decision, in order of preference: a box takes its best shape when the run holds it, else
// the next lone tile comes forward to fill the run (singles fill the gaps around boxes),
// else the box takes the best shape the run allows; a lone tile is placed as it comes (a
// lone hero at 2x2). Only the bottom can come out uneven, and only the very last row may
// run short, on the right. When the greedy path would not close, it is tried again with
// lone tiles held back to the end (they level the bottom) and lone heroes set small,
// cheapest repair first; then a depth-first search goes back over
// the decisions, latest first, trying the next preference: another width for a box, a lone
// tile held back to the end, a lone hero set small. The search has a budget. The wall is
// also laid in bands (see bands()), which always closes, and the cheaper of the two by box
// shape cost stands.
// Returns { tiles: [{ kind: frame|hero|small, item, card, airing, box, inner, col, row,
// colSpan, rowSpan }] sorted by row then col (a frame before what it holds), rows } in units;
// a member's `box` is its frame tile and `inner` its place in the box (see the mapping).
var MAX_NODES = 2000, MAX_GREEDY = 120
function packGroups(items, cols, unitsSmall, geo) {
    cols = Math.max(1, cols)
    var boxHeroes = heroCap(cols) > 0
    var isBox = [], heroOk = [], ranked = []
    for (var i = 0; i < items.length; i++) {
        var it = items[i]
        isBox.push(!!it.box || it.cards.length >= 2)
        heroOk.push(false)
        if (isBox[i]) heroOk[i] = !!it.hero && boxHeroes
        else if (it.hero && cols >= 2) ranked.push(i)
    }
    ranked.sort(function(a, b) { return (items[b].heroRank || 0) - (items[a].heroRank || 0) || a - b })
    for (var h = 0; h < ranked.length && h < heroCap(cols); h++) heroOk[ranked[h]] = true

    var memo = {}
    function shapesFor(j, maxC) {
        var key = j + ":" + maxC
        return memo[key] || (memo[key] = boxShapes(items[j].cards.length, heroOk[j], maxC, geo))
    }
    function level(sky) {
        var bottom = 0
        for (var b = 0; b < cols; b++) bottom = Math.max(bottom, sky[b])
        var gap = false
        for (var e = 0; e < cols; e++) {
            if (sky[e] < bottom - 1) return -1
            if (sky[e] === bottom - 1) gap = true
            else if (gap) return -1
        }
        return bottom
    }

    var nodes = 0, limit = 0
    // placed: a linked list (prev, entry) so a branch shares its prefix
    function dfs(queue, sky, placed) {
        if (++nodes > limit) return null
        if (!queue.length) { var bottom = level(sky); return bottom >= 0 ? { placed: placed, slots: bottom } : null }
        var y = Infinity, x = 0
        for (var c = 0; c < cols; c++) if (sky[c] < y) { y = sky[c]; x = c }
        var g = 0
        while (x + g < cols && sky[x + g] === y) g++
        var j = queue[0]
        var choices = []
        var pull = -1
        for (var q = 1; q < queue.length; q++) if (!isBox[queue[q]]) { pull = q; break }
        if (isBox[j]) {
            var pref = shapesFor(j, cols)[0]
            var fits = shapesFor(j, g)
            if (pref.c <= g) choices.push({ at: 0, shape: pref })
            if (pull > 0) choices.push({ at: pull, span: heroOk[queue[pull]] && g >= 2 ? 2 : 1 })
            if (pull > 0 && heroOk[queue[pull]] && g >= 2) choices.push({ at: pull, span: 1 })
            for (var f = 0; f < fits.length; f++) if (fits[f] !== pref) choices.push({ at: 0, shape: fits[f] })
        } else {
            if (heroOk[j] && g >= 2) choices.push({ at: 0, span: 2 })
            choices.push({ at: 0, span: 1 })
            var later = false
            for (var lq = 1; lq < queue.length; lq++) if (isBox[queue[lq]]) { later = true; break }
            if (later) choices.push({ defer: true })
        }
        for (var k = 0; k < choices.length; k++) {
            var ch = choices[k]
            var next = queue.slice()
            if (ch.defer) { next.push(next.shift()); var r0 = dfs(next, sky, placed); if (r0) return r0; continue }
            var jj = next.splice(ch.at, 1)[0]
            var sky2 = sky.slice()
            var entry
            if (ch.shape) {
                entry = { j: jj, col: x, row: y, shape: ch.shape }
                for (var m = x; m < x + ch.shape.c; m++) sky2[m] = y + ch.shape.k
            } else {
                entry = { j: jj, col: x, row: y, span: ch.span }
                for (var m2 = x; m2 < x + ch.span; m2++) sky2[m2] = y + ch.span
            }
            var r = dfs(next, sky2, { prev: placed, entry: entry })
            if (r) return r
            if (nodes > limit) return null
        }
        return null
    }
    var singles = []
    for (var o = 0; o < items.length; o++) if (!isBox[o]) singles.push(o)
    var lone = ranked.slice(0, heroCap(cols)).reverse()   // least important first
    var sky0 = []
    for (var z = 0; z < cols; z++) sky0.push(0)
    // the queue with the last `reserve` lone tiles held back after every box
    function queueOf(reserve) {
        var held = {}, main = [], tail = []
        for (var r = 0; r < reserve && r < singles.length; r++) held[singles[singles.length - 1 - r]] = true
        for (var q = 0; q < items.length; q++) (held[q] ? tail : main).push(q)
        return main.concat(tail)
    }
    // First the greedy path alone (the search's first branch) under each repair, cheapest
    // first: a held-back lone tile costs 1, a lone hero set small 4; then the search proper
    var tries = []
    for (var dm = 0; dm <= lone.length; dm++)
        for (var rv = 0; rv <= singles.length && rv <= cols * 4; rv++) tries.push({ r: rv, d: dm, cost: rv + 4 * dm })
    tries.sort(function(a, b) { return a.cost - b.cost || a.d - b.d })
    var found = null, spent = 0
    for (var ti = 0; ti < tries.length && ti < MAX_GREEDY && !found; ti++) {
        for (var d0 = 0; d0 < lone.length; d0++) heroOk[lone[d0]] = d0 >= tries[ti].d
        nodes = 0; limit = items.length + 1
        found = dfs(queueOf(tries[ti].r), sky0, null)
        spent += nodes
    }
    if (!found) {
        for (var d1 = 0; d1 < lone.length; d1++) heroOk[lone[d1]] = true
        nodes = 0; limit = MAX_NODES
        found = dfs(queueOf(0), sky0, null)
        spent += nodes
    }
    nodes = spent
    // The search's wall, if it closed, against the bands, by their boxes' shape costs (a
    // box squeezed into a narrow run can close a wall and still look wrong); the bands pay
    // a little more when they move lone tiles behind the boxes
    var placedList = []
    var slots = 0
    function shapesCost(list) {
        var t = 0
        for (var w = 0; w < list.length; w++) if (list[w].shape) t += shapeCost(list[w].shape, geo)
        return t
    }
    if (found) {
        for (var n0 = found.placed; n0; n0 = n0.prev) placedList.push(n0.entry)
        placedList.reverse()
        slots = found.slots
    }
    var banded = bands(items, isBox, heroOk, cols, geo)
    var bandCost = shapesCost(banded.placed) + (singles.length ? BAND_COST : 0)
    var usedBands = !found || bandCost < shapesCost(placedList)
    if (usedBands) {
        placedList = banded.placed
        slots = banded.slots
    }

    // Slots to units: a box to its frame (which draws the strip) and its members, each
    // member carrying its place inside the box (`inner`: x and w in columns, row and rows in
    // inner rows, and the box's c and k) and, for the live window, the unit row it falls on
    var us = unitsSmall
    var tiles = []
    for (var k2 = 0; k2 < placedList.length; k2++) {
        var pl = placedList[k2], item = items[pl.j]
        if (pl.shape) {
            var S = pl.shape
            var frameTile = { kind: "frame", item: pl.j, card: null, airing: !!item.airing, col: pl.col, row: pl.row * us, colSpan: S.c, rowSpan: S.k * us }
            tiles.push(frameTile)
            var heroCard = S.hero ? item.hero : null
            var members = heroCard ? item.cards.filter(function(c) { return c.id !== heroCard.id }) : item.cards
            var mi = 0
            for (var ci = 0; ci < S.cells.length; ci++) {
                var cell = S.cells[ci]
                tiles.push({ kind: cell.kind, item: pl.j, card: cell.kind === "hero" ? heroCard : members[mi++], airing: !!item.airing, box: frameTile,
                             col: pl.col + Math.floor(cell.x), row: (pl.row + cell.row) * us, colSpan: Math.max(1, Math.round(cell.w)), rowSpan: cell.rows * us,
                             inner: { x: cell.x, w: cell.w, row: cell.row, rows: cell.rows, c: S.c, k: S.k } })
            }
        } else {
            tiles.push({ kind: pl.span === 2 ? "hero" : "small", item: pl.j, card: item.cards[0], airing: false, box: null, col: pl.col, row: pl.row * us, colSpan: pl.span, rowSpan: pl.span * us })
        }
    }
    tiles.sort(function(a, b) { return a.row - b.row || a.col - b.col || (a.kind === "frame" ? -1 : b.kind === "frame" ? 1 : 0) })
    return { tiles: tiles, rows: slots * us, searched: nodes, closed: !usedBands }
}

// The wall in bands, when the skyline search runs out of budget: the boxes in order, in
// rows that each fill the width exactly, every box in a band the band's height; then the
// lone tiles, small, row by row, the last row ragged on the right. A band is chosen by
// dynamic programming over the boxes (at most BAND_MAX to a band) and the band heights,
// each band's widths by a knapsack to the column count, for the least total shape cost.
// A box alone at the full width is always a band, so this always closes.
var BAND_MAX = 8, MAX_STRETCH = 1.5
function bands(items, isBox, heroOk, cols, geo) {
    var boxes = []
    for (var i = 0; i < items.length; i++) if (isBox[i]) boxes.push(i)
    var B = boxes.length
    var heights = {}
    for (var hk = 1; hk <= 12; hk++) heights[hk] = true
    // per box and height: the widths that come out exactly that tall, with their layout
    var optsMemo = {}
    function opts(b, K) {
        var key = b + ":" + K
        if (optsMemo[key]) return optsMemo[key]
        var j = boxes[b], n = items[j].cards.length, out = []
        for (var c = 1; c <= cols; c++) {
            var L = heroOk[j] ? boxLayout(n, true, c) : null
            if (!L || L.k !== K) L = boxLayout(n, false, c)
            if (!L || L.k !== K || L.stretch > MAX_STRETCH) continue
            out.push({ c: c, L: L, cost: shapeCost(L, geo) })
        }
        // alone at the full width a box always makes a band, whatever its stretch
        if (!out.some(function(o) { return o.c === cols })) {
            var full = boxLayout(n, false, cols)
            if (full && full.k === K) out.push({ c: cols, L: full, cost: shapeCost(full, geo) })
        }
        return (optsMemo[key] = out)
    }
    for (var b0 = 0; b0 < B; b0++) {
        var n0 = items[boxes[b0]].cards.length + (heroOk[boxes[b0]] ? 3 : 0)
        heights[Math.ceil(items[boxes[b0]].cards.length / cols)] = true
        heights[Math.ceil(n0 / cols)] = true
    }
    var Ks = Object.keys(heights).map(Number)
    // best[i]: the least cost to lay boxes 0..i-1; via[i]: { from, K, picks }
    var best = [0], via = [null]
    for (var e = 1; e <= B; e++) { best.push(Infinity); via.push(null) }
    for (var from = 0; from < B; from++) {
        if (best[from] === Infinity) continue
        for (var ki = 0; ki < Ks.length; ki++) {
            var K = Ks[ki]
            // knapsack over the band's boxes: width used -> { cost, picks }
            var table = {}
            table[0] = { cost: 0, picks: [] }
            for (var to = from; to < B && to - from < BAND_MAX; to++) {
                var o = opts(to, K)
                var nextT = {}
                for (var used in table) {
                    var u = +used, cur = table[used]
                    for (var q = 0; q < o.length; q++) {
                        var w = u + o[q].c
                        if (w > cols) continue
                        var cost = cur.cost + o[q].cost
                        if (!nextT[w] || cost < nextT[w].cost) nextT[w] = { cost: cost, picks: cur.picks.concat([o[q]]) }
                    }
                }
                table = nextT
                var full = table[cols]
                if (full) {
                    var total = best[from] + full.cost + 0.05   // a little per band: fewer, fuller bands
                    if (total < best[to + 1] - 1e-9) { best[to + 1] = total; via[to + 1] = { from: from, K: K, picks: full.picks } }
                }
                if (!Object.keys(table).length) break
            }
        }
    }
    var chain = []
    for (var at = B; at > 0; at = via[at].from) chain.push(via[at])
    chain.reverse()
    var placed = [], y = 0, bi = 0
    for (var bn = 0; bn < chain.length; bn++) {
        var x = 0
        for (var pk = 0; pk < chain[bn].picks.length; pk++) {
            var pick = chain[bn].picks[pk]
            placed.push({ j: boxes[bi++], col: x, row: y, shape: pick.L })
            x += pick.c
        }
        y += chain[bn].K
    }
    var cc = 0
    for (var si = 0; si < items.length; si++) {
        if (isBox[si]) continue
        placed.push({ j: si, col: cc, row: y, span: 1 })
        if (++cc === cols) { cc = 0; y++ }
    }
    return { placed: placed, slots: y + (cc > 0 ? 1 : 0) }
}
