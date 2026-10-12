// node apps/linux/qml/test-mosaic.mjs: the library packer (mosaic.js) checked in isolation.
// mosaic.js is a QML `.pragma library` script, not a module, so it is read and evaluated
// with the pragma line dropped.
import { readFileSync } from "node:fs"
import assert from "node:assert/strict"

const src = readFileSync(new URL("./mosaic.js", import.meta.url), "utf8").replace(/^\.pragma library$/m, "")
const M = new Function(src + "\nreturn { columns, unitsToPx, unitsFor, heroCap, heroRank, pickHeroes, pack, holes, firstHole, overlaps, header, boxLayout, boxShape, packGroups }")()

const now = 2_000_000_000
const day = 86400
let id = 0
const card = (extra = {}) => ({ id: ++id, watched: null, watched_state: "Unknown", last_viewed_at: null, next_airing: null, ...extra })

// columns: exact right edge, whole pixels, u half the base
for (const width of [700, 1000, 1499, 2460, 4900]) {
  const g = M.columns(width, 180, 6)
  assert.equal(g.xs.length, g.cols)
  const last = g.xs[g.cols - 1] + g.ws[g.cols - 1]
  assert.equal(last, width, `right edge exact at ${width}`)
  for (const w of g.ws) assert.ok(Number.isInteger(w) && w >= g.w && w <= g.w + 1)
  assert.equal(g.u, Math.round(g.w * 0.5))
}
assert.equal(M.unitsToPx(4, 90, 6), 378)
assert.equal(M.unitsFor(378, 90, 6), 4)
assert.equal(M.unitsFor(379, 90, 6), 5)

// cap
assert.equal(M.heroCap(3), 0)
assert.equal(M.heroCap(4), 1)
assert.equal(M.heroCap(5), 1)
assert.equal(M.heroCap(6), 2)
assert.equal(M.heroCap(13), 4)

// hero rule
assert.ok(M.heroRank(card({ watched_state: "Behind", last_viewed_at: now - 2 * day }), now) > 0)
assert.ok(M.heroRank(card({ watched: 3, watched_state: "Unknown", last_viewed_at: now - day }), now) > 0)
assert.equal(M.heroRank(card({ watched: 3, watched_state: "CaughtUp", last_viewed_at: now - day }), now), -1)
assert.equal(M.heroRank(card({ watched_state: "Behind", last_viewed_at: now - 20 * day }), now), -1)
assert.ok(M.heroRank(card({ next_airing: { episode: 4, at: now + 3 * day } }), now) > 0)
assert.equal(M.heroRank(card({ next_airing: { episode: 4, at: now + 9 * day } }), now), -1)
{
  const a = card({ watched_state: "Behind", last_viewed_at: now - 5 * day })
  const b = card({ watched_state: "Behind", last_viewed_at: now - 1 * day })
  const c = card({ next_airing: { episode: 2, at: now + day } })
  const taken = {}
  assert.deepEqual(M.pickHeroes([c, a, b], now, 2, taken).map(x => x.id), [b.id, a.id])
  assert.deepEqual(M.pickHeroes([c, a, b], now, 2, taken).map(x => x.id), [c.id], "a hero is a hero once")
}

// pack
function check(sections, cols, us, uh) {
  const { tiles, rows } = M.pack(sections, cols, us, uh)
  assert.equal(M.overlaps(tiles), false, `no overlap at ${cols} cols`)
  for (const t of tiles) {
    assert.ok(t.col >= 0 && t.col + t.colSpan <= cols, "within the columns")
    assert.ok(t.row + t.rowSpan <= rows)
  }
  const cap = M.heroCap(cols)
  for (let s = 0; s < sections.length; s++) {
    const own = tiles.filter(t => t.section === s)
    assert.ok(own.filter(t => t.kind === "hero").length <= cap, "heroes within the cap")
    assert.equal(own.filter(t => t.kind === "label").length, 1)
    const label = own.find(t => t.kind === "label")
    const first = [...own].sort((p, q) => p.row - q.row || p.col - q.col)[0]
    assert.equal(first, label, "the label stone opens its section in reading order")
    // every series of the section placed exactly once
    const ids = new Set([...sections[s].cards, ...sections[s].heroes].map(c => c.id))
    assert.equal(own.filter(t => t.kind !== "label").length, ids.size)
  }
  // sections never interleave: in reading order (row, then column) the section index
  // never goes back
  const reading = [...tiles].sort((p, q) => p.row - q.row || p.col - q.col)
  for (let i = 1; i < reading.length; i++)
    assert.ok(reading[i].section >= reading[i - 1].section, `sections interleave at ${cols} cols`)
  // one continuous wall: no hole anywhere above its final small row, which runs short only
  // on the right
  assert.equal(M.holes(tiles, cols, 0, us), false, `the wall has a hole at ${cols} cols`)
  // sorted by row then col
  for (let i = 1; i < tiles.length; i++)
    assert.ok(tiles[i - 1].row < tiles[i].row || (tiles[i - 1].row === tiles[i].row && tiles[i - 1].col < tiles[i].col))
  return { tiles, rows }
}

for (const cols of [1, 2, 3, 4, 5, 7, 8, 13, 26]) {
  id = 0
  const all = Array.from({ length: 52 }, () => card())
  const airing = Array.from({ length: 6 }, () => card())
  const heroes = all.slice(0, 6)
  // the airing series are not repeated in the main section
  const sections = [
    { label: "Airing", heroes: airing.slice(0, 2), cards: airing },
    { label: "Everything", heroes, cards: all },
    { label: "Empty", heroes: [], cards: [] },
  ]
  check(sections, cols, 4, 8)
}

// a series is a hero once on the wall (the sections hold disjoint series)
{
  id = 0
  const all = Array.from({ length: 30 }, () => card())
  const { tiles } = check([
    { label: "Airing", heroes: all.slice(0, 2), cards: all.slice(0, 3) },
    { label: "Everything", heroes: all.slice(3, 7), cards: all.slice(3) },
  ], 8, 4, 8)
  const heroIds = tiles.filter(t => t.kind === "hero").map(t => t.card.id)
  assert.equal(new Set(heroIds).size, heroIds.length, "no series is a hero twice")
}

// one continuous wall: the second section's label follows the first section's last tile
// on the same row, no fresh row and no empty field beside a short section
{
  id = 0
  const airing = Array.from({ length: 6 }, () => card())
  const all = Array.from({ length: 52 }, () => card())
  const { tiles } = check([
    { label: "Airing", heroes: [], cards: airing },
    { label: "Series", heroes: [], cards: all },
  ], 14, 4, 8)
  const label = tiles.find(t => t.kind === "label" && t.section === 1)
  assert.deepEqual([label.col, label.row], [7, 0], "the Series stone sits beside Airing's last tile")
}

// the airing case: 6 cards, 13 columns, 4 of them hero-eligible. Four heroes would leave
// cells empty beside them; heroes are demoted until nothing is empty above the final
// small row and that row is ragged only on the right.
{
  id = 0
  const six = Array.from({ length: 6 }, () => card({ next_airing: { episode: 2, at: now + day } }))
  const heroes = M.pickHeroes(six, now, M.heroCap(13), {})
  assert.equal(heroes.length, 4)
  const { tiles, rows } = check([{ label: "Airing", heroes, cards: six }], 13, 4, 8)
  const taken = new Set()
  for (const t of tiles) for (let x = t.col; x < t.col + t.colSpan; x++) for (let y = t.row; y < t.row + t.rowSpan; y++) taken.add(x + ":" + y)
  for (let y = 0; y < rows - 4; y++) for (let x = 0; x < 13; x++) assert.ok(taken.has(x + ":" + y), `no empty cell above the final small row (${x}, ${y})`)
  assert.equal(M.holes(tiles, 13, 0, 4), false)
}

// the wireframe case: label, two heroes, small tiles filling in beside them
{
  id = 0
  const all = Array.from({ length: 20 }, () => card())
  const { tiles } = check([{ label: "x", heroes: all.slice(0, 2), cards: all }], 8, 4, 8)
  const heroes = tiles.filter(t => t.kind === "hero")
  assert.deepEqual(heroes.map(t => [t.col, t.row]), [[1, 0], [3, 0]])
  const smallRow0 = tiles.filter(t => t.kind === "small" && t.row === 0).map(t => t.col)
  assert.deepEqual(smallRow0, [5, 6, 7])
  const smallRow4 = tiles.filter(t => t.kind === "small" && t.row === 4).map(t => t.col)
  assert.deepEqual(smallRow4, [0, 5, 6, 7], "small tiles fill under the label and beside the heroes")
}

// header: whole spans that fill each row exactly, the spare to the flex tile
{
  const items = [{ key: "title", span: 2, flex: true }, { key: "tabs", span: 3 }, { key: "sorts", span: 3 }, { key: "count", span: 1 }]
  for (const cols of [4, 5, 9, 13]) {
    const h = M.header(items, cols)
    for (let r = 0; r < h.rows; r++) {
      const row = h.tiles.filter(t => t.row === r)
      assert.equal(row.reduce((n, t) => n + t.colSpan, 0), cols, "a header row fills the width")
    }
  }
  assert.equal(M.header(items, 13).tiles.find(t => t.key === "title").colSpan, 6)
}


// ---- packGroups: the arranged wall, boxes as rigid blocks
const geo = { slotW: 180, slotH: 290, grout: 2 }
// a seeded generator, so a failure names a case that reproduces
function rng(seed) { return () => (seed = (seed * 1103515245 + 12345) % 2147483648) / 2147483648 }
function library(rand, n, heroShare) {
  const items = []
  const all = []
  let left = n
  while (left > 0) {
    const r = rand()
    const size = Math.min(left, r < 0.6 ? 1 : r < 0.85 ? 2 : r < 0.95 ? 2 + Math.floor(rand() * 4) : 5 + Math.floor(rand() * 10))
    const cards = Array.from({ length: size }, () => card())
    all.push(...cards)
    const hero = rand() < heroShare ? cards[Math.floor(rand() * size)] : null
    items.push({ cards, hero, heroRank: hero ? rand() : 0 })
    left -= size
  }
  return { items, all }
}
function checkGroups(items, cols, us, label) {
  const { tiles, rows } = M.packGroups(items, cols, us, geo)
  const cells = tiles.filter(t => t.kind !== "frame")
  const frames = tiles.filter(t => t.kind === "frame")
  assert.equal(M.overlaps(cells), false, `${label}: no two tiles share a cell`)
  assert.equal(M.overlaps(frames), false, `${label}: boxes do not overlap`)
  for (const t of tiles) {
    assert.ok(t.col >= 0 && t.col + t.colSpan <= cols, `${label}: within the columns`)
    assert.ok(t.row + t.rowSpan <= rows, `${label}: within the rows`)
    assert.equal(t.row % us, 0, `${label}: on the slot grid`)
  }
  // a box's cells stay inside its frame and fill it exactly: no hole inside a box
  for (const f of frames) {
    const own = cells.filter(t => t.item === f.item)
    for (const t of own) assert.ok(t.col >= f.col && t.col + t.colSpan <= f.col + f.colSpan && t.row >= f.row && t.row + t.rowSpan <= f.row + f.rowSpan, `${label}: members inside their box`)
    const area = own.reduce((a, t) => a + t.colSpan * t.rowSpan, 0)
    assert.equal(area, f.colSpan * f.rowSpan, `${label}: a box is filled, no hole inside`)
    assert.equal(own.filter(t => t.kind === "label").length, 1, `${label}: one label per box`)
    const label0 = own.find(t => t.kind === "label")
    assert.deepEqual([label0.col, label0.row], [f.col, f.row], `${label}: the label opens its box`)
    assert.ok(own.filter(t => t.kind === "hero").length <= 1, `${label}: one hero per box at most`)
    if (cols < 4) assert.equal(own.filter(t => t.kind === "hero").length, 0, `${label}: no box hero below four columns`)
  }
  // only lone tiles are outside boxes, and only groups of one are lone
  for (const t of cells.filter(t => !frames.some(f => f.item === t.item)))
    assert.equal(items[t.item].cards.length, 1, `${label}: a lone tile is a group of one`)
  // every series exactly once outside the Airing box
  const seen = new Map()
  for (const t of cells) if (t.card && !t.airing) seen.set(t.card.id, (seen.get(t.card.id) || 0) + 1)
  for (const it of items) if (!it.airing) for (const c of it.cards) assert.equal(seen.get(c.id), 1, `${label}: series ${c.id} placed once`)
  // no hole above the final small row, which runs short only on the right
  assert.equal(M.holes(cells, cols, 0, us), false, `${label}: the wall has a hole`)
  // sorted by row then col, a frame before its label
  for (let i = 1; i < tiles.length; i++) assert.ok(tiles[i - 1].row < tiles[i].row || (tiles[i - 1].row === tiles[i].row && tiles[i - 1].col <= tiles[i].col), `${label}: sorted`)
  return { tiles, rows, cells, frames }
}

// box shape: near square in pixels, slack bounded, every width hole-free
for (let n = 1; n <= 40; n++) {
  for (const hero of [false, true]) {
    for (let c = 1; c <= 13; c++) {
      const L = M.boxLayout(n, hero, c)
      if (!L) continue
      assert.ok(L.s >= 1 && L.s <= c, `slack within the first row (n ${n}, c ${c})`)
      const area = L.cells.reduce((a, t) => a + t.colSpan * t.rowSpan, 0)
      assert.equal(area, c * L.k, `the box is filled (n ${n}, c ${c})`)
      assert.equal(M.overlaps(L.cells), false)
    }
  }
}
{
  // the chosen width beats every other width on squareness, give or take the header cost
  for (const n of [2, 3, 5, 8, 13, 21]) {
    const pick = M.boxShape(n, false, 13, geo)
    const aspect = L => Math.abs(Math.log((L.c * geo.slotW + (L.c - 1) * geo.grout) / (L.k * geo.slotH + (L.k - 1) * geo.grout)))
    for (let c = 1; c <= 13; c++) {
      const L = M.boxLayout(n, false, c)
      assert.ok(aspect(pick) <= aspect(L) + 1, `near square: ${n} members at ${pick.c} columns`)
    }
    assert.ok(aspect(pick) < 0.7, `${n} members come out near square, not a strip`)
  }
  // Youjo Senki: a series and its film side by side behind their label
  const two = M.boxShape(2, false, 13, geo)
  assert.deepEqual([two.c, two.k, two.s], [3, 1, 1])
  // capped at the wall's columns
  assert.ok(M.boxShape(30, false, 3, geo).c <= 3)
}

// random libraries at every width: the invariants hold, singles fill around boxes
for (let seed = 1; seed <= 60; seed++) {
  const rand = rng(seed)
  for (const cols of [1, 2, 3, 4, 5, 7, 8, 13, 26]) {
    id = 0
    const { items } = library(rand, 10 + Math.floor(rand() * 120), 0.3)
    const airing = Array.from({ length: Math.floor(rand() * 7) }, () => card())
    if (airing.length) items.unshift({ cards: airing, box: true, airing: true, hero: null })
    checkGroups(items, cols, 4, `seed ${seed} at ${cols} cols`)
  }
}

// the Airing box comes first and its series also stand in their own groups
{
  id = 0
  const a = card(), b = card(), c = card()
  const items = [{ cards: [a, b], box: true, airing: true, hero: null }, { cards: [a, c], hero: null }, { cards: [b], hero: null }, ...Array.from({ length: 12 }, () => ({ cards: [card()], hero: null }))]
  const { frames, cells } = checkGroups(items, 8, 4, "airing")
  assert.deepEqual([frames[0].item, frames[0].row, frames[0].col], [0, 0, 0], "the Airing box opens the wall")
  assert.equal(cells.filter(t => t.card && t.card.id === a.id).length, 2, "an airing series shows in Airing and in its box")
  // an Airing box of one is still a box
  const one = checkGroups([{ cards: [a], box: true, airing: true, hero: null }, { cards: [b], hero: null }], 8, 4, "airing of one")
  assert.equal(one.frames.length, 1)
}

// singles fill the gaps around boxes: a wall of one wide box and lone tiles keeps every
// lone tile above the box's foot that fits there
{
  id = 0
  const items = [{ cards: Array.from({ length: 7 }, () => card()), hero: null }, ...Array.from({ length: 20 }, () => ({ cards: [card()], hero: null }))]
  const { frames, cells } = checkGroups(items, 8, 4, "gaps")
  const f = frames[0]
  const beside = cells.filter(t => t.item > 0 && t.row < f.row + f.rowSpan)
  assert.equal(beside.length, (8 - f.colSpan) * (f.rowSpan / 4), "lone tiles fill the columns beside the box")
}

// heroes: one per box from four columns; a lone hero is 2x2; none below four columns
{
  id = 0
  const members = Array.from({ length: 4 }, () => card())
  const items = [{ cards: members, hero: members[2] }, { cards: [card()], hero: null }]
  const lone = card()
  items.push({ cards: [lone], hero: lone, heroRank: 1 })
  for (let i = 0; i < 12; i++) items.push({ cards: [card()], hero: null })
  const wide = checkGroups(items, 8, 4, "heroes")
  const heroes = wide.cells.filter(t => t.kind === "hero")
  assert.ok(heroes.some(t => t.card.id === members[2].id && t.colSpan === 2), "the box's hero is 2x2")
  assert.ok(heroes.some(t => t.card.id === lone.id), "a lone hero is 2x2")
  const narrow = checkGroups(items, 3, 4, "heroes narrow")
  assert.equal(narrow.cells.filter(t => t.kind === "hero" && narrow.frames.some(f => f.item === t.item)).length, 0)
}

// deterministic: the same input packs the same wall
{
  const rand1 = rng(7), rand2 = rng(7)
  id = 0; const a = library(rand1, 80, 0.3).items
  id = 0; const b = library(rand2, 80, 0.3).items
  const pa = M.packGroups(a, 13, 4, geo), pb = M.packGroups(b, 13, 4, geo)
  assert.deepEqual(pa.tiles.map(t => [t.kind, t.col, t.row, t.colSpan, t.rowSpan, t.card && t.card.id]), pb.tiles.map(t => [t.kind, t.col, t.row, t.colSpan, t.rowSpan, t.card && t.card.id]))
}

console.log("mosaic ok")
