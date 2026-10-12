// node apps/linux/qml/test-mosaic.mjs: the library packer (mosaic.js) checked in isolation.
// mosaic.js is a QML `.pragma library` script, not a module, so it is read and evaluated
// with the pragma line dropped.
import { readFileSync } from "node:fs"
import assert from "node:assert/strict"

const src = readFileSync(new URL("./mosaic.js", import.meta.url), "utf8").replace(/^\.pragma library$/m, "")
const M = new Function(src + "\nreturn { columns, unitsToPx, unitsFor, heroCap, heroRank, pickHeroes, pack, overlaps, header }")()

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
    assert.equal(label.col, 0)
    assert.ok(own.every(t => t.row >= label.row), "the label stone opens its section")
    // every series of the section placed exactly once
    const ids = new Set([...sections[s].cards, ...sections[s].heroes].map(c => c.id))
    assert.equal(own.filter(t => t.kind !== "label").length, ids.size)
  }
  // sections never interleave: each starts below the bottom of every earlier tile
  for (let s = 1; s < sections.length; s++) {
    const start = Math.min(...tiles.filter(t => t.section === s).map(t => t.row))
    const prevBottom = Math.max(...tiles.filter(t => t.section < s).map(t => t.row + t.rowSpan))
    assert.ok(start >= prevBottom, "a section starts on a fresh row")
  }
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
  const sections = [
    { label: "Airing", heroes: airing.slice(0, 2), cards: airing },
    { label: "Everything", heroes, cards: all },
    { label: "Empty", heroes: [], cards: [] },
  ]
  check(sections, cols, 4, 8)
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

console.log("mosaic ok")
