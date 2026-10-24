// The show view on the Downloads page: how a show, or a feed for it, gets added. Opening a
// show never downloads anything. The band says what the show is and what the library has;
// "Available sources" lists the providers (anirss's endpoints), each saying whether it can
// answer now without asking it anything; clicking one fetches what it has for this show.
// Its releases open as a grid: one row per release group and quality, one cube per episode,
// under the library's own row so the gaps line up. Clicking a group picks its releases for
// every episode the library lacks; clicking a cube picks or drops one, and that includes an
// episode the library already has: picking one asks to replace it, and the plan then carries a
// "Replace current episodes" switch, off by default. The plan at the right says what Get will
// do, and whether new episodes will follow. When the show has torrents moving, a Downloading
// section above all of this says what, from where, how far along, and lets you stop or cancel
// them or have them fetched in order.
//
// The page owns the searches and the actions (DownloadsPage.getShow*); this view draws them
// and reports clicks.
import QtQuick
import com.marcusrosado.AniBeam

Item {
    id: root
    // { title, query, poster, cubes, total, seriesId, status, format, year, nextAiring, airing }
    property var show: null
    // This show's torrents from the page's poll, to tell downloading from on disk
    property var torrents: []
    property var endpoints: []
    // name -> { state: "idle" | "searching" | "done" | "error", items, facts, error, label, elapsed }
    property var results: ({})
    property string active: ""
    property var sourceOn: function(name) { return true }
    property bool acting: false
    // Set when a Get went through; it clears the picks and is not drawn
    property string actionLine: ""
    property string actionError: ""
    property real nowMs: Date.now()
    signal back()
    signal fetch(string name)
    signal sourceToggled(string name, bool on)
    signal checkSources()
    signal openList(string name)
    signal getAndFollow(var plan)
    // Stop, start or reorder a torrent of this show; the page runs it (Door.controlTorrents)
    signal control(string hash, string action)
    // What a torrent's buttons wait on ("Stop", "Start", "Cancel", or ""), from the page
    property var busyOf: function(hash) { return "" }
    // The page asks before it cancels, and moves what the torrent wrote to the trash
    signal cancelTorrent(Item anchor, var torrent)
    // The same for every torrent in the Downloading list at once
    signal controlAll(var hashes, string action)
    signal cancelAll(Item anchor, var torrents)

    implicitHeight: body.implicitHeight

    readonly property real planWidth: theme.space(88)
    readonly property real colGap: theme.space(6)
    readonly property bool wide: width > theme.space(220)
    readonly property bool inLibrary: !!show && show.seriesId >= 0

    // ---- The library's row and the episode count the grid runs to
    readonly property var libCodes: show && show.cubes ? String(show.cubes).split("") : []
    readonly property var current: active !== "" && results[active] ? ofShow(results[active]) : null
    // ---- Only the releases that name this show. A search matches a release group as well
    // as a title ("Naruto" finds Naruto-Kun.Hu's One Piece), so a release must carry at
    // least half of the show's words outside its leading [group]
    // One word list per name the show goes by; a release passes on any of them
    readonly property var showWords: {
        if (!show) return []
        var names = show.queries && show.queries.length ? show.queries : [show.query || show.title || ""]
        var stop = { the: 1, and: 1, season: 1, part: 1, movie: 1 }
        return names.map(function(q) { return String(q).toLowerCase().split(/[^a-z0-9]+/).filter(function(w) { return w.length >= 3 && !stop[w] }) })
                    .filter(function(ws) { return ws.length > 0 })
    }
    function namesShow(title) {
        if (!showWords.length) return true
        var t = " " + String(title || "").toLowerCase().replace(/^\s*\[[^\]]*\]/, "").replace(/[^a-z0-9]+/g, " ") + " "
        return showWords.some(function(ws) {
            var hit = ws.filter(function(w) { return t.indexOf(" " + w) >= 0 }).length
            return hit >= Math.ceil(ws.length / 2)
        })
    }
    function ofShow(r) {
        if (!r || !r.items) return r
        var items = [], facts = []
        for (var i = 0; i < r.items.length; i++) if (namesShow(r.items[i].title)) { items.push(r.items[i]); facts.push(r.facts[i]) }
        if (items.length === r.items.length) return r
        var out = {}
        for (var k in r) out[k] = r[k]
        out.items = items; out.facts = facts; out.dropped = r.items.length - items.length
        return out
    }
    readonly property int total: {
        var t = show && show.total > 0 ? show.total : libCodes.length
        // A finished show's own count is the count: a stray number in a release name
        // (Naruto - 1080 something) must not stretch the grid past the last episode
        // (an episode a torrent is actually fetching still counts)
        if (show && show.total > 0 && !show.airing) {
            var fixed = show.total
            for (var x in inFlight) fixed = Math.max(fixed, Number(x))
            return Math.min(fixed, 2000)
        }
        if (current && current.items) for (var i = 0; i < current.facts.length; i++) {
            var f = current.facts[i]
            if (f && !f.batch && f.episode !== null && f.episode !== undefined && f.episode === Math.floor(f.episode)) t = Math.max(t, f.episode)
            if (f && f.batch && f.last_episode) t = Math.max(t, f.last_episode)
            var inside = f && f.batch && !f.first_episode ? contentsOf(current.items[i]) : null
            if (inside && inside.length) t = Math.max(t, inside[inside.length - 1])
        }
        // A show made from a torrent has no count of its own: its torrent's episodes are the count
        for (var k in inFlight) t = Math.max(t, Number(k))
        return Math.min(t, 2000)
    }
    // Episode number to how far along its torrent is, for every episode a torrent of this
    // show carries and has not finished; a batch is read file by file
    readonly property var inFlight: {
        var out = {}
        for (var i = 0; i < torrents.length; i++) {
            var t = torrents[i]
            if ((t.progress || 0) >= 1) continue
            var files = t.files || []
            if (files.length) {
                for (var j = 0; j < files.length; j++) {
                    var f = files[j], n = f.episode
                    if (n === null || n === undefined || n !== Math.floor(n) || f.priority === 0) continue
                    out[n] = f.progress || 0
                }
            } else if (t.episode !== null && t.episode !== undefined) out[t.episode] = t.progress || 0
        }
        return out
    }
    // The torrents of this show that have not finished, in episode order
    function stoppedState(t) { var st = String(t.state || ""); return st.indexOf("paused") === 0 || st.indexOf("stopped") === 0 }
    readonly property var activeTorrents: torrents.filter(function(t) { return (t.progress || 0) < 1 })
        .sort(function(a, b) { return root.firstEpisode(a) - root.firstEpisode(b) || String(a.name || "").localeCompare(String(b.name || "")) })
    // The lowest episode a torrent carries, from the page, so Activity orders the same way
    property var firstEpisode: function(t) { return 1e9 }
    readonly property var runningTorrents: activeTorrents.filter(function(t) { return !root.stoppedState(t) })
    readonly property bool downloading: Object.keys(inFlight).some(function(n) { return inFlight[n] < 1 })
    readonly property int downloadingCount: Object.keys(inFlight).filter(function(n) { return inFlight[n] < 1 }).length
    // The episodes with a file here, from the card. The cubes mark one watched elsewhere as
    // watched with no file here, so with this list only what is in it counts as had (#156).
    // The same rule as the core's releases::grid::library, which the phone uses; keep the two
    // alike.
    readonly property var here: show && show.here ? show.here : null
    readonly property int lastHere: here && here.length ? here[here.length - 1] : 0
    readonly property var library: {
        var out = []
        for (var n = 1; n <= total; n++) {
            var c = n <= libCodes.length ? libCodes[n - 1] : "g"
            if ((c === "w" || c === "l") && here && here.indexOf(n) < 0) c = n < lastHere ? "m" : "g"
            var p = inFlight[n]
            if (p !== undefined && c !== "w" && c !== "l") c = p >= 1 ? "d" : "t"
            out.push(c)
        }
        return out
    }
    readonly property var libraryFills: library.map(function(c, i) { var p = root.inFlight[i + 1]; return p === undefined ? null : p })
    // Had: on disk, or on its way (a torrent already carries it). A had episode can still be
    // picked, on purpose, to replace it; the automatic picks never take one.
    function have(n) { var c = library[n - 1]; return c === "d" || c === "l" || c === "w" || c === "t" }
    function onTheWay(n) { return library[n - 1] === "t" }

    // ---- The active provider's releases as rows: group and quality, best-seeded per episode.
    // A batch is a row of its own: one torrent that lights every episode it names (all of
    // them when it names none, a season pack), picked and dropped as one.
    function seedsOf(item) { return item && item.seeders !== null && item.seeders !== undefined ? item.seeders : -1 }
    // ---- What a batch with no range in its title holds, counted from its torrent's file
    // list by the core: link -> { state: "asking" | "done", episodes }
    property var contents: ({})
    function contentsOf(item) { var c = item && root.contents[item.link]; return c && c.state === "done" ? c.episodes : null }
    function inspectBatches() {
        if (!current || !current.items) return
        var asked = false, next = JSON.parse(JSON.stringify(root.contents))
        for (var i = 0; i < current.items.length; i++) {
            var f = current.facts[i] || {}, link = current.items[i].link
            if (!f.batch || f.first_episode || !link || next[link]) continue
            next[link] = { state: "asking", episodes: [] }
            Door.call("InspectRelease", { link: link })
            asked = true
        }
        if (asked) root.contents = next
    }
    onCurrentChanged: inspectBatches()
    Connections {
        target: Door
        function onReleaseInspected(link, episodes, files) {
            if (root.contents[link] === undefined) return
            var next = JSON.parse(JSON.stringify(root.contents))
            next[link] = { state: "done", episodes: episodes.map(Number).sort(function(a, b) { return a - b }) }
            root.contents = next
        }
    }
    // Episode numbers as short runs: 1-12, 14, 20-24
    function runs(list) {
        var out = [], i = 0
        while (i < list.length) {
            var j = i
            while (j + 1 < list.length && list[j + 1] === list[j] + 1) j++
            out.push(j > i ? list[i] + "-" + list[j] : String(list[i]))
            i = j + 1
        }
        return out.join(", ")
    }
    readonly property var rows: {
        if (!current || !current.items) return []
        var by = {}, order = [], packs = []
        for (var i = 0; i < current.items.length; i++) {
            var f = current.facts[i] || {}, it = current.items[i]
            var gname = f.group || root.active
            var key = gname + "|" + (f.resolution || "")
            if (f.batch) {
                // The episodes it holds: the title's range, else the counted file list, else
                // (still counting, or a magnet with no list) the whole show by its title
                var list = []
                if (f.first_episode) for (var m = f.first_episode; m <= Math.min(f.last_episode || total, total); m++) list.push(m)
                else {
                    var inside = contentsOf(it)
                    if (inside && inside.length) list = inside.filter(function(x) { return x >= 1 && x <= total })
                    else for (var w = 1; w <= total; w++) list.push(w)
                }
                var state = f.first_episode ? "title" : contentsOf(it) && contentsOf(it).length ? "counted" : root.contents[it.link] && root.contents[it.link].state === "asking" ? "counting" : "unknown"
                var b = { key: "batch|" + i, group: gname, resolution: f.resolution || "", batch: true, list: list, state: state,
                          first: list.length ? list[0] : 0, last: list.length ? list[list.length - 1] : 0, label: runs(list),
                          eps: {}, count: 0, seeds: Math.max(0, seedsOf(it)), size: f.size_bytes || 0 }
                for (var q = 0; q < list.length; q++) { b.eps[list[q]] = i; b.count++ }
                packs.push(b)
                continue
            }
            if (f.episode === null || f.episode === undefined || f.episode !== Math.floor(f.episode)) continue
            var r = by[key]
            if (!r) {
                r = by[key] = { key: key, group: gname, resolution: f.resolution || "", batch: false, eps: {}, count: 0, seeds: 0, size: 0 }
                order.push(r)
            }
            var n = f.episode, prev = r.eps[n]
            if (prev === undefined || seedsOf(it) > seedsOf(current.items[prev])) r.eps[n] = i
        }
        for (var k = 0; k < order.length; k++) {
            var row = order[k]
            for (var e in row.eps) { row.count++; row.seeds += Math.max(0, seedsOf(current.items[row.eps[e]])); var fb = current.facts[row.eps[e]]; row.size += fb && fb.size_bytes ? fb.size_bytes : 0 }
        }
        order = order.concat(packs)
        order.sort(function(a, b) { return b.count - a.count || (a.batch ? 1 : 0) - (b.batch ? 1 : 0) || b.seeds - a.seeds })
        return order
    }
    // The feed a pick follows: a batch row follows its group's episode row, when there is one
    function followOf(row) {
        if (!row || !row.batch) return row
        for (var i = 0; i < rows.length; i++) if (!rows[i].batch && rows[i].key === row.group + "|" + row.resolution) return rows[i]
        return null
    }

    // ---- The picks: episode number -> item index, and the row the plan follows
    property var picks: ({})
    property string chosen: ""
    property bool follow: !!show && !!show.airing
    // Swap the episodes the library has for the picked releases; off until asked for
    property bool replaceOn: false
    // Fetch the new torrents in order, as qBittorrent's option does
    property bool sequential: false
    // Get was clicked with a download of this show to cancel: waiting for a yes
    property bool confirming: false
    function clearPicks() { picks = ({}); chosen = ""; replaceOn = false; confirming = false }
    onActiveChanged: clearPicks()
    onShowChanged: { clearPicks(); sequential = false; follow = !!show && !!show.airing }
    // When the action went through, the picks are spent
    onActionLineChanged: if (actionLine !== "") clearPicks()
    onPicksChanged: confirming = false
    onReplaceOnChanged: confirming = false
    // The download it would cancel finished or went: nothing left to ask about
    onHadOnTheWayChanged: if (!hadOnTheWay.length) confirming = false
    // When a provider's releases arrive, pick the group that covers the most gaps
    onRowsChanged: if (chosen === "" && rows.length) pickRow(bestRow(), false)
    function wanted(row) {
        var out = []
        for (var n = 1; n <= total; n++) if (!have(n) && row.eps[n] !== undefined) out.push(n)
        return out
    }
    function bestRow() {
        var best = rows[0], score = -1
        for (var i = 0; i < rows.length; i++) { var s = wanted(rows[i]).length * 1000 + rows[i].count; if (s > score) { score = s; best = rows[i] } }
        return best
    }
    function covered(row) {
        var out = []
        for (var n = 1; n <= total; n++) if (row.eps[n] !== undefined) out.push(n)
        return out
    }
    // `explicit` is a click on the row: with nothing missing it takes everything the row
    // covers, to replace it. The automatic pick never does.
    function pickRow(row, explicit) {
        if (!row) return
        var p = {}, w = wanted(row)
        if (!w.length && explicit) w = covered(row)
        for (var i = 0; i < w.length; i++) p[w[i]] = row.eps[w[i]]
        picks = p
        chosen = row.key
    }
    function toggleCube(row, n) {
        if (row.eps[n] === undefined) return
        var p = JSON.parse(JSON.stringify(picks))
        // A batch comes whole, so a click on a gap takes or drops every gap it fills; a click
        // on an episode the library has picks just that one, to replace it
        if (row.batch && !have(n)) {
            // One torrent: a click on any of its episodes takes or drops all of them
            var on = p[n] === row.eps[n], w = wanted(row)
            for (var i = 0; i < w.length; i++) { if (on) { if (p[w[i]] === row.eps[w[i]]) delete p[w[i]] } else p[w[i]] = row.eps[w[i]] }
            picks = p
            chosen = row.key
            return
        }
        if (p[n] === row.eps[n]) delete p[n]; else p[n] = row.eps[n]
        picks = p
        chosen = row.key
    }
    // A folded cube (a long show) stands for a few episodes: a click takes them all, or
    // drops them all when every one of them is already picked from this row
    function toggleRange(row, a, z) {
        if (row.batch) { for (var m = a; m <= z; m++) if (row.eps[m] !== undefined) { toggleCube(row, m); return } return }
        var p = JSON.parse(JSON.stringify(picks)), all = true, any = false
        for (var n = a; n <= z; n++) if (row.eps[n] !== undefined) { any = true; if (p[n] !== row.eps[n]) all = false }
        if (!any) return
        for (var k = a; k <= z; k++) if (row.eps[k] !== undefined) { if (all) delete p[k]; else p[k] = row.eps[k] }
        picks = p
        chosen = row.key
    }
    function codesOf(row) {
        var out = []
        for (var n = 1; n <= total; n++) {
            if (row.eps[n] === undefined) out.push("n")
            else if (picks[n] === row.eps[n]) out.push("p")
            else if (have(n)) out.push(library[n - 1] === "t" ? "t" : "o")
            else out.push("a")
        }
        return out
    }
    readonly property var pickList: {
        var out = []
        for (var n in picks) out.push({ n: Number(n), i: picks[n] })
        out.sort(function(a, b) { return a.n - b.n })
        return out
    }
    // The picks of episodes the library has, and whether replacing them is on: with it off
    // they are left out of the plan, and the plan says so
    readonly property var hadPicked: pickList.filter(function(p) { return root.have(p.n) })
    readonly property var hadOnDisk: hadPicked.filter(function(p) { return !root.onTheWay(p.n) })
    readonly property var hadOnTheWay: hadPicked.filter(function(p) { return root.onTheWay(p.n) })
    readonly property bool replacing: replaceOn && hadPicked.length > 0
    // What Get will fetch
    readonly property var planList: replacing ? pickList : pickList.filter(function(p) { return !root.have(p.n) })
    // The torrents behind the picks: a batch is one however many episodes it covers
    readonly property var pickItems: {
        var seen = {}, out = []
        for (var k = 0; k < planList.length; k++) { var i = planList[k].i; if (!seen[i]) { seen[i] = true; out.push(i) } }
        return out
    }
    readonly property real pickBytes: {
        var s = 0
        for (var k = 0; k < pickItems.length; k++) { var f = current && current.facts[pickItems[k]]; s += f && f.size_bytes ? f.size_bytes : 0 }
        return s
    }
    readonly property var chosenRow: { for (var i = 0; i < rows.length; i++) if (rows[i].key === chosen) return rows[i]; return null }
    readonly property var followRow: followOf(chosenRow)
    readonly property var pickGroups: {
        var seen = {}, out = []
        for (var k = 0; k < pickItems.length; k++) { var f = current && current.facts[pickItems[k]]; var g = (f && f.group ? f.group : root.active) + (f && f.batch ? "'s batch" : ""); if (!seen[g]) { seen[g] = true; out.push(g) } }
        return out
    }
    function episodesPhrase(list) {
        var ns = list.map(function(p) { return p.n })
        if (!ns.length) return ""
        // Runs: 1 to 4, 7, 9 to 12
        var parts = [], a = ns[0], b = ns[0]
        for (var i = 1; i <= ns.length; i++) {
            if (i < ns.length && ns[i] === b + 1) { b = ns[i]; continue }
            parts.push(a === b ? String(a) : a + " to " + b)
            if (i < ns.length) { a = ns[i]; b = ns[i] }
        }
        return (ns.length === 1 ? "Episode " : "Episodes ") + parts.join(", ")
    }
    // The unfinished torrents of this show that carry an episode being replaced; the core
    // decides, file by file, whether each goes whole or only loses those files
    function cancelHashes() {
        var eps = {}, out = []
        for (var k = 0; k < hadOnTheWay.length; k++) eps[hadOnTheWay[k].n] = true
        for (var i = 0; i < activeTorrents.length; i++) {
            var t = activeTorrents[i], files = t.files || [], hit = false
            if (files.length) { for (var j = 0; j < files.length; j++) if (files[j].episode !== null && files[j].episode !== undefined && eps[files[j].episode]) hit = true }
            else if (t.episode !== null && t.episode !== undefined && eps[t.episode]) hit = true
            if (hit) out.push(t.hash)
        }
        return out
    }
    // What the download asks for beyond the releases: order, and the episodes replaced, each
    // under the position of its release in `items`
    function options() {
        var replace = []
        if (replacing) {
            var by = {}
            for (var k = 0; k < hadPicked.length; k++) {
                var pos = pickItems.indexOf(hadPicked[k].i)
                if (pos < 0) continue
                if (!by[pos]) by[pos] = []
                by[pos].push(hadPicked[k].n)
            }
            for (var key in by) replace.push({ item: Number(key), episodes: by[key] })
        }
        return { sequential: sequential, provider: active, series: inLibrary ? show.seriesId : null, replace: replace, cancel: replacing ? cancelHashes() : [] }
    }
    // Get: a download that cancels another asks first
    function go() {
        if (replacing && hadOnTheWay.length && !confirming) { confirming = true; return }
        confirming = false
        getAndFollow(plan())
    }
    function plan() {
        var items = pickItems.map(function(i) { return current.items[i] })
        var row = followRow
        return { endpoint: active, query: current ? current.query : "", items: items, options: options(), follow: follow && !!row,
                 followQuery: row ? (current.query + " " + row.group + (row.resolution ? " " + row.resolution : "")).trim() : "",
                 followItems: row ? Object.keys(row.eps).map(function(n) { return current.items[row.eps[n]] }) : [],
                 name: show ? show.folderName || show.saveName || show.title : "" }
    }
    readonly property string goText: {
        var n = planList.length
        if (acting) return "Working"
        var swap = replacing ? " and replace" : ""
        if (n && follow && followRow) return "Get " + Fmt.plural(n, "episode", "episodes") + swap + " and subscribe"
        if (n) return "Get " + Fmt.plural(n, "episode", "episodes") + swap
        if (follow && followRow) return "Subscribe for new episodes"
        return pickList.length ? "Nothing new to get" : "Pick an episode"
    }
    function epsWords(list) { var w = episodesPhrase(list); return w.charAt(0).toLowerCase() + w.slice(1) }
    // The line under the Replace switch: what it does now, on or off
    readonly property string replaceWords: {
        if (!hadPicked.length) return ""
        var eps = epsWords(hadPicked)
        if (!replaceOn) return "Off. The picks for " + eps + " are left out, since the library has them. Turn it on to swap them."
        var out = "The files you have for " + eps + " go to the trash once the new ones finish."
        if (hadOnTheWay.length) out += " " + episodesPhrase(hadOnTheWay) + (hadOnTheWay.length === 1 ? " is" : " are") + " downloading now: that download is cancelled and what it wrote goes to the trash."
        return out
    }

    // ---- Provider words
    function kindWords(e) {
        if (e.on_demand) return "Jackett, started on demand"
        if (e.kind === "nyaa") return "Anime torrents"
        if (e.kind === "rss") return "RSS feed"
        return e.kind
    }
    function readyWords(e) {
        if (!e.on_demand) return "Ready"
        return e.running === true ? "Running" : "Not running"
    }
    function resultWords(e) {
        if (!root.sourceOn(e.name)) return e.on_demand && e.running !== true ? "Not running. Click its source above to start it." : "Switched off"
        var r = ofShow(results[e.name])
        if (!r || r.state === "idle") return "Click to see what it has for this show"
        if (r.state === "searching") return (r.label || "Searching") + ", " + Math.floor(r.elapsed || 0) + " s"
        if (r.state === "error") return r.error
        // Each release marks what it holds; a batch by its range, its counted files, or the show
        var has = {}
        for (var i = 0; i < r.items.length; i++) {
            var f = r.facts[i]
            if (!f) continue
            if (!f.batch) { if (f.episode >= 1 && f.episode <= total) has[f.episode] = true; continue }
            var inside = f.first_episode ? null : root.contentsOf(r.items[i])
            if (inside && inside.length) inside.forEach(function(x) { has[x] = true })
            else for (var n = f.first_episode || 1; n <= Math.min(f.last_episode || total, total); n++) has[n] = true
        }
        var cover = 0
        for (var k in has) if (Number(k) <= total) cover++
        return Fmt.plural(r.items.length, "release", "releases") + (total ? ", " + cover + " of " + total + " episodes covered" : "")
    }

    Column {
        id: body
        width: parent.width
        spacing: theme.space(6)

        // ---- The band: poster, title, what the library has
        Item {
            width: parent.width
            height: Math.max(poster.height, foot.y + foot.height) + theme.space(2)
            Corner {
                id: poster
                width: theme.space(34); height: width * 1.5
                radius: theme.radiusMd
                smoothing: theme.cornerSmoothing
                color: theme.tileSunken
                Image {
                    id: posterImage
                    visible: false
                    anchors.fill: parent
                    source: !root.show || !root.show.poster ? "" : String(root.show.poster).indexOf("://") > 0 ? root.show.poster : "file://" + root.show.poster
                    sourceSize: theme.posterDecodeSize
                    fillMode: Image.PreserveAspectCrop
                    asynchronous: true
                }
                fillItem: posterImage.status === Image.Ready ? posterImage : null
            }
            // The title and its facts at the poster's top; where the show stands, and its
            // episodes, at its foot
            // The series' own page: everything about it, and where its episodes play
            Button {
                id: seriesButton
                objectName: "get-open-series"
                visible: root.inLibrary
                anchors.right: parent.right
                anchors.top: poster.top
                height: theme.space(11.75)
                text: "Open series"
                icon: "arrow-up-right"
                onClicked: frame.go("series", { id: root.show.seriesId }, root.show.title)
                Tooltip { text: "The series page: its details, episodes and playing" }
            }
            Column {
                id: bandText
                anchors.left: poster.right
                anchors.leftMargin: theme.space(7)
                anchors.right: seriesButton.visible ? seriesButton.left : parent.right
                anchors.rightMargin: seriesButton.visible ? theme.space(4) : 0
                anchors.top: poster.top
                spacing: theme.space(3)
                Label {
                    width: parent.width
                    text: root.show ? root.show.title : ""
                    role: "display"
                    font.pointSize: theme.typePage
                    wrapMode: Text.Wrap
                    maximumLineCount: 2
                    elide: Text.ElideRight
                    lineHeight: 0.9
                }
                Flow {
                    width: parent.width
                    spacing: theme.space(5)
                    Repeater {
                        model: {
                            var s = root.show, out = []
                            if (!s) return out
                            if (s.format) out.push(s.format)
                            if (s.year) out.push(String(s.year))
                            if (s.airing) out.push(s.nextAiring ? "Episode " + s.nextAiring.episode + " in " + Fmt.countdown(s.nextAiring.at - root.nowMs / 1000) : "Airing")
                            else if (s.status === "Finished") out.push("Finished")
                            if (s.total > 0) out.push(s.total + " episodes")
                            return out
                        }
                        Label { required property string modelData; text: modelData; role: "ui"; font.pointSize: theme.typeNormal * 1.1; color: theme.textDim }
                    }
                }
            }
            // At the poster's foot when the title leaves room, else right under the facts
            Column {
                id: foot
                anchors.left: bandText.left
                anchors.right: parent.right
                y: Math.max(bandText.y + bandText.height + theme.space(5), poster.y + poster.height - height)
                spacing: theme.space(3)
                Label {
                    objectName: "get-standing"
                    text: root.downloading ? "Downloading " + Fmt.plural(root.downloadingCount, "episode", "episodes")
                        : root.activeTorrents.length ? "Downloading " + Fmt.plural(root.activeTorrents.length, "torrent", "torrents")
                        : root.inLibrary ? "In your library" : "Not in your library"
                    role: "ui"; font.pointSize: theme.typeNormal * 1.1; color: theme.textDim
                }
                EpisodeCubes {
                    width: Math.min(parent.width, theme.space(220))
                    height: implicitHeight
                    visible: root.library.length > 0
                    codes: root.library
                    fills: root.libraryFills
                    cubeSize: theme.space(7.75)
                    minSize: theme.space(2.5)
                    maxRows: 2
                }
            }
        }

        // ---- What is downloading now, from where, and how far along
        Column {
            id: dlSection
            objectName: "get-downloading"
            visible: root.activeTorrents.length > 0
            width: parent.width
            spacing: theme.space(2)
            Item {
                width: parent.width
                height: theme.space(14)
                Label { id: dlTitle; anchors.left: parent.left; anchors.bottom: parent.bottom; anchors.bottomMargin: theme.space(2); text: "Downloading"; role: "display"; font.pointSize: theme.typeHeading; font.variableAxes: ({ "wdth": 70, "wght": 420 }) }
                // Every torrent at once, when there is more than one
                Row {
                    visible: root.activeTorrents.length > 1
                    anchors.right: parent.right
                    anchors.bottom: parent.bottom
                    anchors.bottomMargin: theme.space(2)
                    spacing: theme.space(2)
                    Button {
                        objectName: "get-all-toggle"
                        readonly property bool stopping: root.runningTorrents.length > 0
                        readonly property var targets: stopping ? root.runningTorrents : root.activeTorrents
                        busy: targets.some(function(t) { var w = root.busyOf(t.hash); return w === "Stop" || w === "Start" })
                        primary: !stopping && !busy
                        text: stopping ? "Stop all" : "Start all"
                        icon: stopping ? "pause" : "play"
                        onClicked: root.controlAll(targets.map(function(t) { return t.hash }), stopping ? "Stop" : "Start")
                        Tooltip { text: parent.stopping ? "Stop every download of this show; what they have stays" : "Start every download of this show again" }
                    }
                    Button {
                        id: cancelAllButton
                        objectName: "get-all-cancel"
                        danger: true
                        busy: root.activeTorrents.some(function(t) { return root.busyOf(t.hash) === "Cancel" })
                        text: "Cancel all"
                        icon: "x"
                        onClicked: root.cancelAll(cancelAllButton, root.activeTorrents)
                        Tooltip { text: "Cancel every download of this show and move what they wrote to the trash" }
                    }
                }
            }
            Repeater {
                model: root.activeTorrents
                TorrentCard { required property var modelData; t: modelData }
            }
        }

        // ---- Available sources, and the plan beside it
        Item {
            width: parent.width
            // Narrow, the plan sits under the grid, so the page has to scroll to it
            height: root.wide ? Math.max(left.implicitHeight, planTile.height) : planTile.y + planTile.height
            Column {
                id: left
                width: root.wide ? parent.width - root.planWidth - root.colGap : parent.width
                spacing: theme.space(5)

                Item {
                    width: parent.width
                    height: theme.space(12)
                    Label { id: whereTitle; anchors.left: parent.left; anchors.bottom: parent.bottom; anchors.bottomMargin: theme.space(2); text: "Available sources"; role: "display"; font.pointSize: theme.typeHeading; font.variableAxes: ({ "wdth": 70, "wght": 420 }) }
                    Button { id: checkButton; anchors.right: parent.right; anchors.bottom: parent.bottom; anchors.bottomMargin: theme.space(1); text: "Check again"; icon: "refresh-cw"; flat: true; small: true; onClicked: root.checkSources(); Tooltip { text: "Asks each on-demand provider whether it is running. Starts nothing." } }
                }

                Tile {
                    width: parent.width
                    implicitHeight: providerList.implicitHeight + padding * 2
                    height: implicitHeight
                    padding: theme.space(2)
                    lit: false
                    Column {
                        id: providerList
                        width: parent.width
                        Repeater {
                            model: root.endpoints
                            Item {
                                id: prow
                                required property var modelData
                                required property int index
                                readonly property bool on: root.sourceOn(modelData.name)
                                readonly property bool isActive: root.active === modelData.name
                                readonly property var res: root.results[modelData.name] || null
                                objectName: "provider-" + modelData.name
                                width: parent.width
                                height: theme.space(19)
                                Rectangle {
                                    anchors.fill: parent
                                    radius: theme.radiusMd
                                    color: prow.isActive ? theme.accentSoft : rowMouse.containsMouse && prow.on ? theme.glassFloat : "transparent"
                                }
                                MouseArea {
                                    id: rowMouse
                                    anchors.fill: parent
                                    hoverEnabled: true
                                    enabled: prow.on
                                    cursorShape: prow.on ? Qt.PointingHandCursor : Qt.ArrowCursor
                                    onClicked: root.fetch(prow.modelData.name)
                                }
                                // Ready, running or not: a dot in its colour
                                Rectangle {
                                    id: readyDot
                                    x: theme.space(4)
                                    anchors.verticalCenter: parent.verticalCenter
                                    width: theme.space(2.5); height: width; radius: width / 2
                                    color: !prow.modelData.on_demand || prow.modelData.running === true ? theme.green : theme.textFaint
                                }
                                Column {
                                    anchors.left: readyDot.right
                                    anchors.leftMargin: theme.space(4)
                                    anchors.right: stateCol.left
                                    anchors.rightMargin: theme.space(4)
                                    anchors.verticalCenter: parent.verticalCenter
                                    spacing: theme.space(1)
                                    opacity: prow.on ? 1 : 0.6
                                    Row {
                                        spacing: theme.space(3)
                                        Label { id: pname; text: prow.modelData.name; role: "title"; font.pointSize: theme.typeLarge }
                                        Label { anchors.baseline: pname.baseline; text: root.kindWords(prow.modelData); role: "ui"; color: theme.textDim }
                                    }
                                    Label {
                                        width: parent.width
                                        text: root.resultWords(prow.modelData)
                                        role: "ui"
                                        color: prow.res && prow.res.state === "error" ? theme.red : prow.res && prow.res.state === "searching" ? theme.text : theme.textDim
                                        elide: Text.ElideRight
                                    }
                                }
                                Row {
                                    id: stateCol
                                    anchors.right: parent.right
                                    anchors.rightMargin: theme.space(4)
                                    anchors.verticalCenter: parent.verticalCenter
                                    spacing: theme.space(4)
                                    Label { anchors.verticalCenter: parent.verticalCenter; text: root.readyWords(prow.modelData); role: "ui"; color: !prow.modelData.on_demand || prow.modelData.running === true ? theme.text : theme.textFaint }
                                    Switch {
                                        objectName: "provider-switch-" + prow.modelData.name
                                        anchors.verticalCenter: parent.verticalCenter
                                        checked: prow.on
                                        onToggled: function(on) { root.sourceToggled(prow.modelData.name, on) }
                                    }
                                    Icon { anchors.verticalCenter: parent.verticalCenter; glyph: "chevron-right"; size: theme.space(5); color: prow.on ? theme.textDim : "transparent" }
                                }
                                Rectangle { visible: prow.index < root.endpoints.length - 1; anchors.bottom: parent.bottom; x: theme.space(4); width: parent.width - theme.space(8); height: 1; color: theme.line }
                            }
                        }
                    }
                }

                // ---- The active provider's releases, as a grid of cubes
                Column {
                    id: grid
                    visible: !!root.current && root.current.state === "done"
                    width: parent.width
                    spacing: theme.space(2)
                    readonly property real whoWidth: theme.space(62)
                    readonly property real cubeWidth: width - whoWidth - theme.space(4)
                    Item {
                        width: parent.width
                        height: theme.space(14)
                        Label { id: gridTitle; anchors.left: parent.left; anchors.bottom: parent.bottom; anchors.bottomMargin: theme.space(2); text: root.active; role: "display"; font.pointSize: theme.typeHeading; font.variableAxes: ({ "wdth": 70, "wght": 420 }) }
                        Button { id: listButton; anchors.right: parent.right; anchors.bottom: parent.bottom; anchors.bottomMargin: theme.space(1); text: "Every release as a list"; icon: "list"; flat: true; small: true; onClicked: root.openList(root.active) }
                    }
                    // The library's own row first, so the gaps line up
                    GridRow {
                        title: "Your library"
                        line: {
                            var have = root.library.filter(function(c) { return c === "d" || c === "l" || c === "w" }).length
                            var parts = []
                            if (have > 0) parts.push(have + " of " + root.total + " on disk")
                            if (root.downloadingCount > 0) parts.push(root.downloadingCount + " downloading")
                            return parts.length ? parts.join(", ") : "Nothing yet"
                        }
                        codes: root.library
                        fills: root.libraryFills
                        lit: true
                    }
                    Repeater {
                        model: root.rows.filter(function(r) { return r.count > 0 })
                        GridRow {
                            required property var modelData
                            title: modelData.batch ? modelData.group + ", batch" : modelData.group
                            line: (modelData.resolution ? modelData.resolution + ", " : "")
                                + (modelData.batch ? (modelData.state === "counting" ? "counting its episodes" : modelData.state === "unknown" ? "episodes not listed, the whole show by its title" : Fmt.plural(modelData.count, "episode", "episodes")) + ", one torrent" : Fmt.plural(modelData.count, "episode", "episodes"))
                                + (modelData.size > 0 ? ", " + Fmt.bytes(modelData.size) : "")
                            codes: root.codesOf(modelData)
                            chosen: root.chosen === modelData.key
                            clickable: true
                            rowData: modelData
                            onPicked: root.pickRow(modelData, true)
                            onCubePicked: function(i) { root.toggleCube(modelData, i + 1) }
                            onRangePicked: function(a, z) { root.toggleRange(modelData, a + 1, z + 1) }
                        }
                    }
                }
                // What the colours mean, read from the same table the cubes are
                Flow {
                    visible: grid.visible
                    width: parent.width
                    spacing: theme.space(5)
                    Repeater {
                        model: ["w", "l", "d", "g", "m", "a", "p", "o"]
                        Row {
                            required property string modelData
                            spacing: theme.space(2)
                            Rectangle {
                                anchors.verticalCenter: parent.verticalCenter
                                width: theme.space(3); height: width; radius: width * 0.3
                                color: theme.cubeState(parent.modelData).fill
                                border.width: theme.cubeState(parent.modelData).edge === "transparent" ? 0 : 1
                                border.color: theme.cubeState(parent.modelData).edge
                            }
                            Label { text: theme.cubeState(parent.modelData).label; role: "ui"; font.pointSize: theme.typeSmall; color: theme.textDim }
                        }
                    }
                }
                // Searching or failed: one line where the grid will be
                Label {
                    visible: !!root.current && root.current.state !== "done" && root.current.state !== "idle"
                    width: parent.width
                    text: root.current ? (root.current.state === "error" ? root.current.error : (root.current.label || "Searching " + root.active) + ", " + Math.floor(root.current.elapsed || 0) + " s") : ""
                    role: "ui"
                    font.pointSize: theme.typeLarge
                    color: root.current && root.current.state === "error" ? theme.red : theme.textDim
                    wrapMode: Text.Wrap
                }
            }

            // ---- The plan: what Get will do
            Tile {
                id: planTile
                objectName: "get-plan"
                x: root.wide ? parent.width - root.planWidth : 0
                y: root.wide ? 0 : left.implicitHeight + theme.space(6)
                width: root.wide ? root.planWidth : parent.width
                implicitHeight: planBody.implicitHeight + padding * 2
                height: implicitHeight
                padding: theme.space(6)
                lit: false
                Column {
                    id: planBody
                    width: parent.width
                    spacing: theme.space(4)
                    Label {
                        width: parent.width
                        text: root.planList.length ? "Get " + Fmt.plural(root.planList.length, "episode", "episodes") : root.pickList.length ? "Nothing new picked" : root.current && root.current.state === "done" ? "Nothing picked" : "Pick a provider"
                        role: "display"; font.pointSize: theme.typeHeading; font.variableAxes: ({ "wdth": 70, "wght": 420 })
                        wrapMode: Text.Wrap
                    }
                    Label {
                        width: parent.width
                        text: {
                            if (root.planList.length) return root.episodesPhrase(root.planList) + " from " + root.pickGroups.join(" and ") + " on " + root.active + "."
                            if (root.pickList.length) return "The library has " + root.epsWords(root.pickList) + ". Turn on Replace current episodes below to swap " + (root.pickList.length === 1 ? "it" : "them") + "."
                            return ""
                        }
                        visible: text !== ""
                        role: "ui"; color: theme.textDim; wrapMode: Text.Wrap
                    }
                    Flow {
                        visible: root.planList.length > 0
                        width: parent.width
                        spacing: theme.space(6)
                        Column { Label { text: Fmt.bytes(root.pickBytes); role: "figure"; font.pointSize: theme.typeLarge }
                            Label { text: "together"; role: "ui"; font.pointSize: theme.typeSmall; color: theme.textFaint } }
                        Column { Label { text: String(root.pickItems.length); role: "figure"; font.pointSize: theme.typeLarge }
                            Label { text: root.pickItems.length === 1 ? "torrent" : "torrents"; role: "ui"; font.pointSize: theme.typeSmall; color: theme.textFaint } }
                    }
                    Rectangle { width: parent.width; height: 1; color: theme.line }
                    // The picks include episodes the library has: say what replacing them does
                    Item {
                        visible: root.hadPicked.length > 0
                        width: parent.width
                        height: visible ? Math.max(replaceSwitch.height, replaceText.implicitHeight) : 0
                        Switch {
                            id: replaceSwitch
                            objectName: "get-replace"
                            checked: root.replaceOn
                            onToggled: function(on) { root.replaceOn = on }
                        }
                        Column {
                            id: replaceText
                            anchors.left: replaceSwitch.right
                            anchors.leftMargin: theme.space(4)
                            anchors.right: parent.right
                            spacing: theme.space(1)
                            Label { text: "Replace current episodes"; role: "title"; font.pointSize: theme.typeNormal }
                            Label { objectName: "get-replace-words"; width: parent.width; text: root.replaceWords; role: "ui"; font.pointSize: theme.typeSmall; color: root.replaceOn ? theme.text : theme.textDim; wrapMode: Text.Wrap }
                        }
                    }
                    OptionRow {
                        objectName: "get-sequential"
                        title: "Download in order"
                        checked: root.sequential
                        onToggled: function(on) { root.sequential = on }
                        tip: root.sequential ? "On: each new torrent fetches its pieces from the start, so the first episodes finish first and you can watch while the rest comes. A little slower overall."
                            : "Off: the client fetches pieces in whatever order the swarm offers them."
                    }
                    OptionRow {
                        objectName: "get-follow"
                        title: "Subscribe for new episodes"
                        checked: root.follow
                        onToggled: function(on) { root.follow = on }
                        tip: !root.follow ? "Off: new episodes show up here as gaps, and nothing downloads by itself."
                            : !root.chosenRow ? "Pick a group first: a subscription follows one group and quality."
                            : !root.followRow ? root.chosenRow.group + " has only the batch here, so there is no feed of single episodes to subscribe to."
                            : "On: a feed for " + root.followRow.group + (root.followRow.resolution ? " " + root.followRow.resolution : "") + " on " + root.active + ". Each new episode downloads as it appears."
                    }
                    // Cancelling a download is asked for here, once, before it goes
                    Column {
                        visible: root.confirming
                        width: parent.width
                        spacing: theme.space(3)
                        Label {
                            objectName: "get-confirm-words"
                            width: parent.width
                            text: root.episodesPhrase(root.hadOnTheWay) + (root.hadOnTheWay.length === 1 ? " is" : " are") + " downloading now. Replacing " + (root.hadOnTheWay.length === 1 ? "it" : "them") + " cancels that download and moves what it wrote to the trash."
                            role: "ui"; wrapMode: Text.Wrap
                        }
                        Button { objectName: "get-confirm"; width: parent.width; text: "Cancel it and replace"; icon: "trash-2"; primary: true; onClicked: root.go() }
                        Button { objectName: "get-confirm-back"; width: parent.width; text: "Back"; flat: true; onClicked: root.confirming = false }
                    }
                    Button {
                        objectName: "get-go"
                        visible: !root.confirming
                        width: parent.width
                        text: root.goText
                        icon: root.follow && root.followRow ? "rss" : "download"
                        primary: true
                        enabled: !root.acting && (root.planList.length > 0 || (root.follow && !!root.followRow))
                        onClicked: root.go()
                    }
                    Label { visible: root.actionError !== ""; width: parent.width; text: root.actionError; role: "ui"; font.pointSize: theme.typeSmall; color: theme.red; wrapMode: Text.Wrap }
                }
            }
        }
    }

    // A plan switch: its name, and an info glyph whose hover tip says what on and off do,
    // so the panel stays two lines instead of two paragraphs
    component OptionRow: Item {
        id: opt
        property string title: ""
        property string tip: ""
        property bool checked: false
        signal toggled(bool on)
        width: parent ? parent.width : 0
        height: Math.max(optSwitch.height, optTitle.implicitHeight)
        Switch {
            id: optSwitch
            anchors.verticalCenter: parent.verticalCenter
            checked: opt.checked
            onToggled: function(on) { opt.toggled(on) }
        }
        Label {
            id: optTitle
            anchors.left: optSwitch.right
            anchors.leftMargin: theme.space(4)
            anchors.verticalCenter: parent.verticalCenter
            width: Math.min(implicitWidth, parent.width - optSwitch.width - info.width - theme.space(6))
            text: opt.title; role: "title"; font.pointSize: theme.typeNormal; elide: Text.ElideRight
            MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: optSwitch.toggle() }
        }
        Item {
            id: info
            objectName: opt.objectName !== "" ? opt.objectName + "-info" : ""
            anchors.left: optTitle.right
            anchors.leftMargin: theme.space(2)
            anchors.verticalCenter: parent.verticalCenter
            width: theme.space(7); height: theme.space(7)
            Icon { anchors.centerIn: parent; glyph: "info"; size: theme.space(4); color: infoTip.hovered ? theme.text : theme.textFaint }
            Tooltip { id: infoTip; text: opt.tip }
        }
    }

    // One word for what the client says a torrent is doing
    function stateWords(t) {
        var st = String(t.state || "")
        if (st === "error") return "Error"
        if (st === "missingFiles") return "Files missing"
        if (stoppedState(t)) return "Stopped"
        if (st === "stalledDL") return "Waiting for peers"
        if (st === "metaDL" || st === "forcedMetaDL") return "Fetching metadata"
        if (st === "queuedDL") return "Queued"
        if (st === "checkingDL" || st === "checkingResumeData" || st === "checkingUP") return "Checking files"
        if (st === "allocating") return "Allocating"
        if (st === "moving") return "Moving files"
        if (st === "downloading" || st === "forcedDL") return "Downloading"
        return st === "" ? "Waiting" : st
    }
    // One torrent of this show: its name, where it comes from and goes, how far along, the
    // episodes as cubes, and the controls
    component TorrentCard: Tile {
        id: card
        property var t: ({})
        readonly property bool stopped: root.stoppedState(t)
        readonly property bool broken: t.state === "error" || t.state === "missingFiles"
        readonly property var files: t.files || []
        readonly property var eps: {
            var out = []
            for (var i = 0; i < files.length; i++) {
                var n = files[i].episode
                if (n === null || n === undefined || n !== Math.floor(n) || n < 1) continue
                out.push({ n: n, p: files[i].progress || 0, skip: files[i].priority === 0 })
            }
            return out.sort(function(a, b) { return a.n - b.n })
        }
        readonly property int epsDone: eps.filter(function(e) { return e.p >= 1 }).length
        readonly property var live: eps.filter(function(e) { return !e.skip })
        readonly property bool seq: !!t.seq_dl
        // The episodes in words: "Episodes 1 to 12", or the one a single file carries
        readonly property string episodeWords: {
            if (live.length > 1) return root.episodesPhrase(live)
            if (live.length === 1) return "Episode " + live[0].n
            if (t.episode !== null && t.episode !== undefined) return "Episode " + t.episode
            return t.batch ? "A batch" : ""
        }
        readonly property string release: {
            var bits = []
            if (t.group) bits.push(t.group)
            if (t.resolution) bits.push(t.resolution)
            return bits.length ? bits.join(", ") : "Group not in the name"
        }
        // The endpoint AniBeam picked it on, when it did; else the one whose name the release
        // group carries (Erai-raws on erai), said to be a guess
        readonly property string guess: {
            var g = String(t.group || "").toLowerCase()
            if (g === "") return ""
            for (var i = 0; i < root.endpoints.length; i++) if (g.indexOf(String(root.endpoints[i].name).toLowerCase()) >= 0) return root.endpoints[i].name
            return ""
        }
        readonly property string source: {
            var bits = []
            if (t.provider) bits.push(t.provider)
            else if (guess !== "") bits.push("likely " + guess)
            if (t.tracker) bits.push("tracker " + t.tracker)
            return bits.length ? bits.join(", ") : "Not known yet"
        }
        width: parent ? parent.width : 0
        implicitHeight: cardBody.implicitHeight + padding * 2
        height: implicitHeight
        padding: theme.space(5)
        lit: false
        Column {
            id: cardBody
            width: parent.width
            spacing: theme.space(5)
            // What it carries and how far along on the left; its controls on the right
            // The controls hang over the tile row's right end, so the tiles start right under
            // the figures
            Item {
                width: parent.width
                height: headCol.implicitHeight
                z: 1
                Column {
                    id: headCol
                    anchors.left: parent.left
                    anchors.right: controls.left
                    anchors.rightMargin: theme.space(4)
                    spacing: theme.space(1.5)
                    Label { visible: card.episodeWords !== ""; text: card.episodeWords; role: "ui"; color: theme.textDim }
                    Row {
                        spacing: theme.space(3)
                        Label {
                            objectName: "get-torrent-state"
                            text: card.broken ? root.stateWords(card.t) : Math.floor((card.t.progress || 0) * 100) + "%"
                            role: "title"; font.pointSize: theme.typeLarge
                            color: card.broken ? theme.red : theme.text
                        }
                        Label {
                            objectName: "get-torrent-figures"
                            anchors.baseline: parent.children[0].baseline
                            text: {
                                var t = card.t, bits = []
                                bits.push(Fmt.bytes(Math.round((t.size || 0) * (t.progress || 0))) + " of " + Fmt.bytes(t.size || 0))
                                if ((t.dlspeed || 0) > 0) bits.push(Fmt.bytes(t.dlspeed) + "/s")
                                if (!card.stopped && t.eta > 0 && t.eta < 8640000) bits.push(Fmt.countdown(t.eta) + " left")
                                return bits.join(", ")
                            }
                            role: "figure"; font.pointSize: theme.typeLarge; color: theme.textDim
                        }
                    }
                }
                Column {
                    id: controls
                    anchors.right: parent.right
                    anchors.top: parent.top
                    spacing: theme.space(3)
                    Row {
                        anchors.right: parent.right
                        spacing: theme.space(2)
                        Button {
                            objectName: "get-torrent-toggle"
                            readonly property string waiting: root.busyOf(card.t.hash)
                            busy: waiting === "Stop" || waiting === "Start"
                            primary: card.stopped && !busy
                            text: waiting === "Stop" ? "Stopping" : waiting === "Start" ? "Starting" : card.stopped ? "Start" : "Stop"
                            icon: card.stopped ? "play" : "pause"
                            onClicked: root.control(card.t.hash, card.stopped ? "Start" : "Stop")
                            Tooltip { text: card.stopped ? "Start downloading again" : "Stop downloading; what it has stays" }
                        }
                        Button {
                            id: cancelButton
                            objectName: "get-torrent-cancel"
                            danger: true
                            busy: root.busyOf(card.t.hash) === "Cancel"
                            text: busy ? "Cancelling" : "Cancel"
                            icon: "x"
                            onClicked: root.cancelTorrent(cancelButton, card.t)
                            Tooltip { text: "Cancel this download and move what it wrote to the trash" }
                        }
                    }
                    // The client's own option, per torrent
                    Row {
                        anchors.right: parent.right
                        spacing: theme.space(3)
                        Label { anchors.verticalCenter: parent.verticalCenter; text: "Force download order"; role: "title"; font.pointSize: theme.typeNormal }
                        Switch {
                            id: seqSwitch
                            objectName: "get-torrent-sequential"
                            anchors.verticalCenter: parent.verticalCenter
                            checked: card.seq
                            onToggled: function(on) { root.control(card.t.hash, on ? "Sequential" : "Unsequential"); settle.restart() }
                            // The poll after the change brings the client's answer; if the change did not
                            // take, the switch goes back to what the client says
                            Binding { target: seqSwitch; property: "checked"; value: card.seq }
                            Timer { id: settle; interval: 4000; onTriggered: seqSwitch.checked = card.seq }
                            Tooltip { text: card.seq ? "On: pieces come from the start of each file, so the first episodes finish first." : "Off: the client fetches pieces in whatever order the swarm offers them." }
                        }
                    }
                }
            }
            // One tile per episode, filling from the bottom as its file comes in; the one
            // furthest along that is not done says how far
            EpisodeCubes {
                visible: card.eps.length > 1
                width: parent.width - controls.width - theme.space(8)
                height: implicitHeight
                codes: card.eps.map(function(e) { return e.skip ? "o" : e.p >= 1 ? "d" : "t" })
                fills: card.eps.map(function(e) { return e.p })
                cubeSize: theme.space(12.25)
                minSize: theme.space(4)
                maxRows: 3
                badgeIndex: {
                    var best = -1, p = 0
                    for (var i = 0; i < card.eps.length; i++) { var e = card.eps[i]; if (!e.skip && e.p < 1 && e.p > p) { p = e.p; best = i } }
                    return best
                }
                badgeText: badgeIndex >= 0 ? Math.floor(card.eps[badgeIndex].p * 100) + "%" : ""
                tipFor: function(i, code) { var e = card.eps[i]; return "Episode " + e.n + ": " + (e.skip ? "skipped" : e.p >= 1 ? "done" : Math.floor(e.p * 100) + "%") }
            }
            EpisodeCubes {
                visible: card.eps.length <= 1
                width: parent.width
                height: implicitHeight
                codes: theme.cubeSteps(card.t.progress || 0, 40, card.broken ? "e" : "t", "g")
                cubeSize: theme.space(2.5)
                minSize: 2
                maxRows: 1
                tipLimit: 0
            }
            // What it is, where from, where to
            Column {
                width: parent.width
                spacing: theme.space(1.5)
                Fact { objectName: "get-torrent-name"; key: "Torrent"; value: card.t.name || "" }
                Fact { key: "Release"; value: card.release }
                Fact { key: "From"; value: card.source }
                Fact { key: "Saving to"; value: card.t.save_path || card.t.content_path || "" }
            }
        }
    }

    // A key and its value on one line
    component Fact: Item {
        property string key: ""
        property string value: ""
        width: parent ? parent.width : 0
        height: Math.max(keyLabel.implicitHeight, valueLabel.implicitHeight)
        Label { id: keyLabel; width: theme.space(18); text: parent.key; role: "ui"; font.pointSize: theme.typeSmall; color: theme.textFaint }
        Label { id: valueLabel; x: theme.space(18); width: parent.width - x; text: parent.value; role: "ui"; color: theme.textDim; elide: Text.ElideMiddle }
    }

    // One row of the grid: who, one line, and the cubes; a click on the name picks the row
    component GridRow: Item {
        id: gr
        property string title: ""
        property string line: ""
        property var codes: []
        property var fills: null
        property bool chosen: false
        property bool lit: false
        property bool clickable: false
        property var rowData: null
        signal picked()
        signal cubePicked(int index)
        signal rangePicked(int first, int last)
        width: grid.width
        height: Math.max(theme.space(15), cubes.implicitHeight + theme.space(5))
        Rectangle {
            anchors.fill: parent
            radius: theme.radiusMd
            color: gr.chosen ? theme.accentSoft : gr.lit ? theme.glassFloat : whoMouse.containsMouse ? theme.glassFloat : "transparent"
            border.width: gr.chosen ? 1 : 0
            border.color: theme.accentEdge
        }
        MouseArea {
            id: whoMouse
            width: grid.whoWidth; height: parent.height
            enabled: gr.clickable
            hoverEnabled: enabled
            cursorShape: enabled ? Qt.PointingHandCursor : Qt.ArrowCursor
            onClicked: gr.picked()
        }
        Column {
            x: theme.space(4)
            width: grid.whoWidth - theme.space(6)
            anchors.verticalCenter: parent.verticalCenter
            spacing: theme.space(0.5)
            Label { width: parent.width; text: gr.title; role: "title"; font.pointSize: theme.typeNormal; elide: Text.ElideRight }
            Label { width: parent.width; text: gr.line; role: "ui"; font.pointSize: theme.typeSmall; color: theme.textDim; elide: Text.ElideRight }
        }
        EpisodeCubes {
            id: cubes
            x: grid.whoWidth
            anchors.verticalCenter: parent.verticalCenter
            width: grid.cubeWidth
            height: implicitHeight
            codes: gr.codes
            fills: gr.fills
            // A long show trades cube size for rows before it folds episodes together: a
            // season keeps big cubes, a few hundred episodes stay one cube each
            cubeSize: root.total <= 60 ? theme.space(6) : root.total <= 160 ? theme.space(4) : theme.space(3)
            minSize: theme.space(2.5)
            maxRows: root.total <= 60 ? 4 : 10
            clickable: gr.clickable
            spanFirst: gr.rowData && gr.rowData.batch ? gr.rowData.first : 0
            spanLast: gr.rowData && gr.rowData.batch ? gr.rowData.last : 0
            spanLabel: !gr.rowData || !gr.rowData.batch ? "" : gr.rowData.state === "counting" ? "Counting" : gr.rowData.state === "unknown" ? gr.rowData.label + "?" : "Episodes " + gr.rowData.label
            spanChosen: gr.chosen
            tipFor: gr.rowData ? function(i, code) {
                var idx = gr.rowData.eps[i + 1]
                if (idx === undefined) return "Episode " + (i + 1) + ": no release from " + gr.rowData.group
                var it = root.current.items[idx], f = root.current.facts[idx]
                var bits = [it.title]
                if (it.seeders !== null && it.seeders !== undefined) bits.push(it.seeders + " seeders")
                if (f && f.size_bytes) bits.push(Fmt.bytes(f.size_bytes))
                if (code === "o") bits.push("already on disk, click to replace it")
                if (code === "t") bits.push("downloading now, click to replace it")
                return bits.join("\n")
            } : null
            onCubeClicked: function(i) { gr.cubePicked(i) }
            onRangeClicked: function(a, z) { gr.rangePicked(a, z) }
        }
    }
}
