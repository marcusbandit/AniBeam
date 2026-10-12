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

// The section's heroes: the qualifying cards, best first, capped; `taken` (an object keyed
// by id) skips series already a hero in an earlier section and records the new ones.
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

// First-fit skyline packing. sections: [{ label, count, heroes: [card], cards: [card] }]
// where `cards` may include the heroes (they are skipped there). Each section starts on a
// fresh row (the skyline flattened to its top), with its label stone at column 0, then its
// heroes, then its small tiles, each at the lowest row its span fits, left-most on ties.
// Heroes past the cap, or with fewer than two columns, are set as small tiles.
// Returns { tiles: [{ kind, section, card, col, row, colSpan, rowSpan }] sorted by row
// then col, rows: the total height in units }.
function pack(sections, cols, unitsSmall, unitsHero) {
    cols = Math.max(1, cols)
    var sky = []
    for (var c = 0; c < cols; c++) sky.push(0)
    var tiles = []
    var cap = heroCap(cols)

    function place(kind, si, card, span, height) {
        span = Math.min(span, cols)
        var best = -1, bestRow = Infinity
        for (var x = 0; x + span <= cols; x++) {
            var y = 0
            for (var k = x; k < x + span; k++) y = Math.max(y, sky[k])
            if (y < bestRow) { bestRow = y; best = x }
        }
        for (var m = best; m < best + span; m++) sky[m] = bestRow + height
        tiles.push({ kind: kind, section: si, card: card, col: best, row: bestRow, colSpan: span, rowSpan: height })
    }

    for (var si = 0; si < sections.length; si++) {
        var s = sections[si]
        var top = 0
        for (var a = 0; a < cols; a++) top = Math.max(top, sky[a])
        for (var b = 0; b < cols; b++) sky[b] = top
        place("label", si, null, 1, unitsSmall)
        var heroes = (s.heroes || []).slice(0, cols >= 2 ? cap : 0)
        var heroIds = {}
        for (var h = 0; h < heroes.length; h++) {
            heroIds[heroes[h].id] = true
            place("hero", si, heroes[h], 2, unitsHero)
        }
        var cards = s.cards || []
        for (var i = 0; i < cards.length; i++) {
            if (heroIds[cards[i].id]) continue
            place("small", si, cards[i], 1, unitsSmall)
        }
        // demoted heroes (past the cap) that were not in cards
        var extra = (s.heroes || []).slice(heroes.length)
        for (var e = 0; e < extra.length; e++) {
            var inCards = false
            for (var q = 0; q < cards.length; q++) if (cards[q].id === extra[e].id) { inCards = true; break }
            if (!inCards) place("small", si, extra[e], 1, unitsSmall)
        }
    }
    var rows = 0
    for (var r = 0; r < cols; r++) rows = Math.max(rows, sky[r])
    tiles.sort(function(p, q) { return p.row - q.row || p.col - q.col })
    return { tiles: tiles, rows: rows }
}

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
