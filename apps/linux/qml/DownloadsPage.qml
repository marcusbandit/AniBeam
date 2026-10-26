// Downloads: anirss's front, shows first. Two columns: the page, and Activity at the right
// (ActivityColumn: the rate, what is moving, what finished, today's lines of the log), so a
// click anywhere has somewhere to show its result.
//
// The head names the page and says how anirss and the torrent client stand. Under it one
// wide field: typing lists shows (the library's first, then AniList's), Return searches
// release names the old way (the table, filters, Subscribe), asking every source that is
// switched on. Under the field the sources: a chip per provider, lit when on and greyed when
// off, a click flips it. A plain endpoint is on by default; an on-demand one (1337x, a pod
// behind Jackett) is on only while it runs. Clicking it while it does not run asks, in a
// small sheet, whether to start Jackett; AniBeam starts it on that confirm and never before,
// and the core refuses a search that would start it behind your back.
//
// With nothing searched the page is four lenses: Gaps in your library (aired episodes not
// on disk), Airing, Following (the subscriptions, in sections by where the show stands) and
// Downloaded (the torrents, a show at a time). Every show, wherever it is clicked, opens the
// show view (GetShow): its providers, a click on one fetches what it has, its releases as a
// grid of episode cubes, and a plan that says what Get and Follow will do. Opening a show
// never downloads anything. A subscription or a downloaded show opens the older ShowDetail,
// with its feed and torrents. Every removal goes through the frame's removal sheet.
//
// The subscriptions are the frame's (it reads them for the rail's badge); this page asks
// for a fresh read on open. The transfers are the one place in the app that polls: every
// two seconds while something is downloading and the page is up, every fifteen while
// nothing is, and not at all while the removal sheet is open.
import QtQuick
import com.marcusrosado.AniBeam

Item {
    id: page
    objectName: "downloads-page"
    property var props: ({})
    property string title: "Downloads"
    property alias scrollY: scroll.contentY
    // Escape closes the show, then the search; out of a release list that came from a show's
    // grid, it goes back to the grid
    function escapePressed() {
        if (page.searching) { page.cancelSearch(); return true }
        if (page.detail !== null) { page.closeDetail(); return true }
        if (page.getShow !== null) { page.closeGet(); return true }
        if (page.state !== null && page.gridReturn !== null) { page.backToGrid(); return true }
        if (page.state !== null || page.searchError !== "") { page.clearSearch(); return true }
        return false
    }
    function focusSearch() { scroll.contentY = 0; search.focusInput() }

    // ---- The connection
    property var status: null
    readonly property var anirss: status ? status.anirss : null
    readonly property var torrent: status ? status.torrent : null
    readonly property bool anirssReady: !!anirss && anirss.state === "Ready"
    readonly property bool anirssStarting: !!anirss && anirss.state === "Starting"
    readonly property bool torrentOn: !!torrent && torrent.connected
    readonly property var endpoints: status && status.endpoints ? status.endpoints : []
    readonly property string clientName: status && status.settings ? (status.settings.torrent_client === "Qbittorrent" ? "qBittorrent" : String(status.settings.torrent_client)) : "qBittorrent"
    readonly property var savedClients: status && status.settings && status.settings.saved_clients ? status.settings.saved_clients : []
    // A remote anirss keeps the client its own config names, so the chip cannot switch it
    readonly property bool remoteAnirss: !!status && !!status.settings && status.settings.anirss_mode === "Remote"
    // The client in use by the name it was saved under, once there is more than one to
    // tell apart (the saved ones and AniBeam's own); with one, plain "qBittorrent"
    readonly property string activeClientName: {
        var s = status ? status.settings : null
        if (!s || savedClients.length === 0) return clientName
        if (s.torrent_mode === "Managed") return "AniBeam's own"
        if (savedClients.length < 2) return clientName
        for (var i = 0; i < savedClients.length; i++) if (savedClients[i].id === s.active_client) return savedClients[i].name
        return clientName
    }
    property real connectJob: -1
    property bool connecting: false
    // Set by a switch of client: the list is the new client's once it answers, not at the
    // poll timer's next turn
    property bool repoll: false

    // ---- The sources: which providers a search may ask. A plain endpoint is on unless
    // switched off; an on-demand one is on only while it runs, and switching it on while it
    // does not asks first (askStart). The flips live in the frame for the session.
    function endpointNamed(name) { for (var i = 0; i < endpoints.length; i++) if (endpoints[i].name === name) return endpoints[i]; return null }
    function sourceOn(name) {
        var e = page.endpointNamed(name)
        if (e && e.on_demand && e.running !== true) return false
        var o = frame.downloadsSources[name]
        return o !== undefined ? o : true
    }
    function storeSource(name, on) {
        var m = JSON.parse(JSON.stringify(frame.downloadsSources))
        m[name] = on
        frame.downloadsSources = m
    }
    // Switching on a service that is down does not switch it on: it opens the sheet that
    // asks whether to start it. Everything else flips at once.
    function setSource(name, on) {
        var e = page.endpointNamed(name)
        if (on && e && e.on_demand && e.running !== true) { page.askStart(name); return }
        page.storeSource(name, on)
    }
    function toggleSource(name) { page.setSource(name, !page.sourceOn(name)) }
    readonly property var enabledEndpoints: { var m = frame.downloadsSources; return endpoints.filter(function(e) { return page.sourceOn(e.name) }) }
    property real checkJob: -1
    function checkSources() {
        var r = Door.call("CheckEndpoints", {})
        if (r.error) { frame.toast(page.errorWords(r.error)); return }
        page.checkJob = r.reply.job
    }

    // ---- Starting a source's service (Jackett, behind 1337x): ask, then the core starts it
    property real startJob: -1
    property string startName: ""
    function askStart(name) {
        var at = -1
        for (var i = 0; i < page.endpoints.length; i++) if (page.endpoints[i].name === name) at = i
        var chip = at >= 0 ? sourceChips.itemAt(at) : null
        if (page.startJob < 0) startSheet.error = ""
        startSheet.openAt(chip, name)
    }
    function startService(name) {
        var r = Door.call("StartEndpoint", { name: name })
        if (r.error) { startSheet.error = page.errorWords(r.error); return }
        page.startJob = r.reply.job
        page.startName = name
        startSheet.error = ""
        startSheet.busy = true
    }
    function startDone(ok, error) {
        var name = page.startName
        page.startJob = -1
        page.startName = ""
        startSheet.busy = false
        if (ok) {
            page.storeSource(name, true)
            startSheet.close()
            frame.toast(name + " is running and switched on")
        } else if (startSheet.visible) startSheet.error = page.errorWords(error)
        else frame.toast(page.errorWords(error), 6)
    }

    // ---- The library, for the lenses and the suggestions
    property var libCards: []
    function loadLibrary() {
        var r = Door.listLibrary("All", "", "LastViewed", "Desc", false)
        if (!r.error) page.libCards = r.reply.series || []
    }
    function hasGap(c) {
        var s = String(c.cubes || "")
        return c.kind === "Show" && !c.hidden && (s.indexOf("m") >= 0 || (s.indexOf("g") >= 0 && /[dlw]/.test(s)))
    }
    readonly property var gapCards: libCards.filter(hasGap)
    readonly property var airingCards: libCards.filter(function(c) { return c.kind === "Show" && !c.hidden && (c.status === "Releasing" || !!c.next_airing) })
    // "Episodes 4 and 7 missing, 8 to 12 not downloaded"
    function runs(ns) {
        if (!ns.length) return ""
        var parts = [], a = ns[0], b = ns[0]
        for (var i = 1; i <= ns.length; i++) {
            if (i < ns.length && ns[i] === b + 1) { b = ns[i]; continue }
            parts.push(a === b ? String(a) : a + " to " + b)
            if (i < ns.length) { a = ns[i]; b = ns[i] }
        }
        if (parts.length === 2) return parts[0] + " and " + parts[1]
        return parts.join(", ")
    }
    function gapWords(c) {
        var s = String(c.cubes || ""), miss = [], tail = []
        for (var i = 0; i < s.length; i++) { if (s[i] === "m") miss.push(i + 1); else if (s[i] === "g") tail.push(i + 1) }
        var out = []
        if (miss.length) out.push((miss.length === 1 ? "Episode " : "Episodes ") + page.runs(miss) + " missing")
        if (tail.length) out.push((out.length ? "" : tail.length === 1 ? "Episode " : "Episodes ") + page.runs(tail) + " not downloaded")
        return out.join(", ")
    }
    property string lens: frame.downloadsLens
    onLensChanged: frame.downloadsLens = lens

    // ---- The suggestions under the field: library shows, then AniList's
    property var aniHits: []
    property real aniJob: -1
    property int suggestIndex: 0
    property bool suggestClosed: false
    readonly property string query: search.text.trim()
    readonly property var libHits: {
        var q = page.query.toLowerCase()
        if (q.length < 2) return []
        var out = []
        for (var i = 0; i < libCards.length && out.length < 5; i++) {
            var c = libCards[i], t = c.titles || {}
            var hay = [c.title, t.romaji, t.english, t.native, t.folder].join(" ").toLowerCase()
            if (hay.indexOf(q) >= 0) out.push(c)
        }
        return out
    }
    readonly property var aniOnly: {
        var ids = {}
        for (var i = 0; i < libCards.length; i++) if (libCards[i].match_info && libCards[i].match_info.anilist_id) ids[libCards[i].match_info.anilist_id] = libCards[i]
        return aniHits.filter(function(h) { return !ids[h.id] }).slice(0, 5)
    }
    readonly property var suggestRows: libHits.map(function(c) { return { lib: c } }).concat(aniOnly.map(function(h) { return { ani: h } }))
    readonly property bool suggesting: !suggestClosed && query.length >= 2 && search.editing && page.state === null && !page.searching
    Timer {
        id: aniDebounce
        interval: 300
        onTriggered: {
            if (page.query.length < 2) { page.aniHits = []; return }
            var r = Door.searchOn("Anilist", page.query, 8)
            if (!r.error) page.aniJob = r.reply.job
        }
    }
    function pickSuggestion(row) {
        if (!row) return
        page.suggestClosed = true
        if (row.lib) page.openGet(page.showFromCard(row.lib, page.query))
        else page.openGet(page.showFromHit(row.ani, page.query))
    }

    // ---- Which name a show goes by here. The one that was typed, when it is one of its
    // names (search "Kino no Tabi" with the app in English and Kino no Tabi is what shows);
    // otherwise the app's title language. Releases are searched by the romaji name, which is
    // what release files carry, and by its other names when that finds nothing.
    function norm(t) { return String(t || "").toLowerCase().replace(/[^a-z0-9\u00c0-\uffff]+/g, " ").trim() }
    function namesOfHit(h) { return [h.romaji, h.english, h.title, h.native].concat(h.synonyms || []).filter(function(n) { return !!n }) }
    function namesOfCard(c) { var t = c.titles || {}; return [t.romaji, t.english, c.title, t.native].filter(function(n) { return !!n }) }
    function preferredOfHit(h) {
        var lang = Door.preferences.title_language || "Romaji"
        if (lang === "Native" && h.native) return h.native
        return lang === "English" ? (h.english || h.romaji || h.title) : (h.romaji || h.title || h.english)
    }
    function typedName(names, typed) {
        var q = page.norm(typed)
        if (q.length < 2) return ""
        for (var i = 0; i < names.length; i++) if (page.norm(names[i]) === q) return names[i]
        for (var j = 0; j < names.length; j++) { var n = page.norm(names[j]); if (n.indexOf(q) === 0 || (q.length >= 4 && n.indexOf(q) >= 0)) return names[j] }
        return ""
    }
    function nameOfHit(h, typed) { return page.typedName(page.namesOfHit(h), typed) || page.preferredOfHit(h) }
    function nameOfCard(c, typed) { return page.typedName(page.namesOfCard(c), typed) || c.title }
    // The release queries, best first: the romaji, then whatever else it is called
    function queriesOf(names, first) {
        var out = [], seen = {}
        ;[first].concat(names).forEach(function(n) { var k = page.norm(n); if (n && k && !seen[k]) { seen[k] = true; out.push(String(n)) } })
        return out
    }

    // ---- The show view: a show, its providers' releases, the plan
    property var getShow: null
    property var getResults: ({})
    property string getActive: ""
    property var getJobs: ({})
    property real getJob: -1
    property real followJob: -1
    property var pendingFollow: null
    property bool getActing: false
    property string getLine: ""
    property string getError: ""
    function lastPart(p) { var f = String(p || "").replace(/[\\/]+$/, ""); var i = Math.max(f.lastIndexOf("/"), f.lastIndexOf("\\")); return i >= 0 ? f.slice(i + 1) : f }
    // The torrents that fill a show: linked to its series, or saved under a folder named
    // as it is (a show not scanned into the library yet)
    function keyOf(s) { return String(s || "").toLowerCase().replace(/[^a-z0-9]+/g, "") }
    function torrentsOfShow(show) {
        if (!show) return []
        var want = [keyOf(show.folderName), keyOf(show.saveName), keyOf(show.title)].filter(function(k) { return k !== "" })
        var named = show.hashes || []
        return torrents.filter(function(t) {
            if (show.seriesId >= 0 && t.series_id === show.seriesId) return true
            if (named.indexOf(t.hash) >= 0) return true
            var k = keyOf(page.lastPart(t.save_path)), c = keyOf(page.lastPart(t.content_path))
            return want.indexOf(k) >= 0 || want.indexOf(c) >= 0
        })
    }
    function showFromCard(c, typed) {
        var t = c.titles || {}
        var queries = page.queriesOf(page.namesOfCard(c), t.romaji || c.title)
        return { title: page.nameOfCard(c, typed || ""), query: queries[0], queries: queries, poster: c.poster || "", cubes: c.cubes || "", here: c.episodes_here || null, total: c.total_episodes || 0,
                 seriesId: c.id, status: c.status || "", format: c.format || "", year: c.year || 0, nextAiring: c.next_airing || null,
                 airing: c.status === "Releasing" || !!c.next_airing, folderName: page.lastPart(c.path) }
    }
    function showFromHit(h, typed) {
        var queries = page.queriesOf(page.namesOfHit(h), h.romaji || h.title)
        var title = page.nameOfHit(h, typed || "")
        // A show not in the library is saved under its own name, but never the native one:
        // the library links folders by their latin titles
        return { title: title, saveName: title === h.native ? (h.romaji || h.english || h.title) : title, query: queries[0], queries: queries, poster: h.cover_url || "", cubes: h.episodes ? "g".repeat(Math.min(h.episodes, 400)) : "", total: h.episodes || 0,
                 seriesId: -1, status: "", format: h.format || "", year: h.year || 0, nextAiring: null, airing: false, folderName: "" }
    }
    function showFromLink(s) {
        for (var i = 0; i < libCards.length; i++) if (libCards[i].id === s.series_id) return page.showFromCard(libCards[i])
        return { title: s.title, query: s.title, poster: s.poster || "", cubes: s.cubes || "", total: s.total_episodes || 0, seriesId: s.series_id,
                 status: s.status || "", format: "", year: 0, nextAiring: s.next_airing || null, airing: s.status === "Releasing" || !!s.next_airing, folderName: page.lastPart(s.path) }
    }
    // A show made from a torrent the library does not know: the name as a release title says
    // it, cut down to the show, unless the folder it saves into is named for the show
    function showName(name) {
        var n = String(name || "").replace(/\.(mkv|mp4|avi)$/i, "")
        n = n.replace(/\[[^\]]*\]|\([^)]*\)/g, " ").replace(/[._]/g, " ")
        n = n.replace(/\sS\d{1,2}(E\d{1,4})?\b.*$/i, " ").replace(/\s-\s\d{1,4}(v\d)?\b.*$/, " ")
        n = n.replace(/\b(batch|complete|2160p|1080p|720p|480p|web-?dl|bluray|bd|x264|x265|hevc|aac|multi-?sub).*$/i, " ")
        return n.replace(/\s+/g, " ").replace(/[\s-]+$/, "").trim()
    }
    function showFromTorrent(t) {
        var folder = page.lastPart(t.save_path)
        var title = page.showName(t.name)
        if (folder !== "" && keyOf(folder) !== "" && keyOf(t.name).indexOf(keyOf(folder)) >= 0) title = folder
        if (title === "") title = folder !== "" ? folder : String(t.name || "")
        var total = 0, files = t.files || []
        for (var i = 0; i < files.length; i++) if (files[i].episode !== null && files[i].episode !== undefined) total = Math.max(total, files[i].episode)
        if (t.episode !== null && t.episode !== undefined) total = Math.max(total, t.episode)
        return { title: title, query: title, poster: "", cubes: "", total: total, seriesId: -1, status: "", format: "", year: 0, nextAiring: null, airing: false,
                 folderName: folder, hashes: [t.hash] }
    }
    // An Activity row: the show it fills, or one made from the torrent's name and folder
    function openFromTorrent(t) {
        if (t.series_id !== null && t.series_id !== undefined && page.shows[t.series_id]) page.openGet(page.showFromLink(page.shows[t.series_id]))
        else page.openGet(page.showFromTorrent(t))
    }
    function openGet(show) {
        page.getShow = show
        page.getResults = ({})
        page.getActive = ""
        page.getLine = ""
        page.getError = ""
        page.gridReturn = null
        page.dropSearches()
        page.state = null
        page.detail = null
        page.history = []
        scroll.contentY = 0
    }
    function closeGet() { page.getShow = null; page.getResults = ({}); page.getActive = ""; page.getJobs = ({}) }
    function setResult(name, r) { var m = Object.assign({}, page.getResults); m[name] = r; page.getResults = m }
    function fetchGet(name) {
        if (!page.getShow || !page.sourceOn(name)) return
        page.getActive = name
        var have = page.getResults[name]
        if (have && (have.state === "done" || have.state === "searching")) return
        page.searchGetAs(name, 0)
    }
    // One provider's search by the show's `at`th name; an empty answer moves on to the next
    function searchGetAs(name, at) {
        var qs = page.getShow.queries || [page.getShow.query]
        var r = Door.searchReleases(qs[at], name)
        if (r.error) { page.setResult(name, { state: "error", error: page.errorWords(r.error) }); return }
        var j = Object.assign({}, page.getJobs); j[r.reply.job] = name; page.getJobs = j
        page.setResult(name, { state: "searching", label: at > 0 ? "Trying " + qs[at] : "", elapsed: 0, start: Date.now(), at: at })
    }
    Timer {
        interval: 500; repeat: true
        running: { for (var k in page.getResults) if (page.getResults[k].state === "searching") return true; return false }
        onTriggered: {
            var m = Object.assign({}, page.getResults)
            for (var k in m) if (m[k].state === "searching") m[k] = Object.assign({}, m[k], { elapsed: (Date.now() - m[k].start) / 1000 })
            page.getResults = m
        }
    }
    function getAndFollow(plan) {
        page.getLine = ""
        page.getError = ""
        page.pendingFollow = plan.follow ? plan : null
        if (plan.items.length) {
            var r = Door.downloadReleasesWith(plan.items, plan.name, plan.options || {})
            if (r.error) { page.getError = page.errorWords(r.error); return }
            page.getJob = r.reply.job
            page.getActing = true
        } else if (plan.follow) page.follow(plan)
    }
    function follow(plan) {
        page.pendingFollow = null
        var r = Door.subscribe(plan.followQuery, plan.endpoint, plan.followItems, plan.name, plan.query !== plan.followQuery ? plan.query : "")
        if (r.error) { page.getActing = false; page.getError = page.errorWords(r.error); return }
        page.followJob = r.reply.job
        page.getActing = true
    }
    // The release list a show's grid opens is a step away from the grid: the show, what each
    // provider found and the field's text are kept, so Back to the grid puts them back
    property var gridReturn: null
    function listFromGet(name) {
        var res = page.getResults[name]
        if (!res || !res.search) return
        var kept = {}
        for (var k in page.getResults) if (page.getResults[k].state === "done") kept[k] = page.getResults[k]
        page.gridReturn = { show: page.getShow, results: kept, active: page.getActive, text: search.text }
        search.text = res.query
        page.closeGet()
        page.adopt(res.search, res.search.fetched_query || res.search.query)
        scroll.contentY = 0
    }
    function backToGrid() {
        var back = page.gridReturn
        if (!back) return
        search.text = back.text
        page.openGet(back.show)
        page.getResults = back.results
        page.getActive = back.active
    }

    // ---- The search: the state anirss asks the client to hold, and the trail for Back
    property var state: null
    property var history: []
    // A release search asks every source that is on, a job each. `searchJobs` is job to
    // source; `searchResults` is source to { state: "searching" | "done" | "error" | "idle",
    // search, error }, kept so a click on a source's chip shows what it found. `state` holds
    // the one on screen; narrowing it is a step on that source alone (`searchJob`).
    property var searchJobs: ({})
    property var searchResults: ({})
    property string adoptedFrom: ""
    property real searchJob: -1
    property bool searching: false
    property string notice: ""
    property string searchError: ""
    property var selected: ({})
    property int hoveredGroup: -1
    property string nameText: ""
    property var preview: null
    property real previewJob: -1
    property real actionJob: -1
    property bool acting: false
    property string actionLine: ""
    property string actionError: ""
    readonly property var items: state && state.items ? state.items : []
    readonly property var groups: state && state.groups ? state.groups : []
    readonly property int selectedCount: { var n = 0; for (var k in selected) if (selected[k]) n++; return n }
    readonly property var facts: state && state.facts ? state.facts : []
    readonly property var searchShow: state && state.show ? state.show : null
    // The search's own progress: the phase the core reports, the seconds since it began,
    // the ceiling an on-demand endpoint's start has (0 for any other), the endpoint asked
    property string searchLabel: ""
    property real searchTotal: 0
    property real searchStartMs: 0
    property real searchElapsed: 0
    property string searchEndpoint: ""
    Timer { interval: 250; repeat: true; running: page.searching; onTriggered: page.searchElapsed = (Date.now() - page.searchStartMs) / 1000 }
    // What the core read off the endpoint's own feed: link -> entry and title -> entry,
    // gathered over the whole search (a step back keeps its dates)
    property var extras: ({})
    property var extrasByTitle: ({})
    property string datesState: "loading"
    property string sortKey: ""
    property bool sortDesc: true
    property bool showDismissed: false
    readonly property real selectedBytes: {
        var n = 0
        for (var i = 0; i < items.length; i++) if (selected[items[i].link]) {
            var f = facts[i], e = extras[items[i].link]
            n += f && f.size_bytes ? f.size_bytes : e && e.size_bytes ? e.size_bytes : 0
        }
        return n
    }
    // The releases for the episodes the library lacks: for each episode in the list that is
    // not on disk, the one with the most seeders
    readonly property var missingPicks: {
        if (!searchShow) return []
        var best = {}
        for (var i = 0; i < items.length; i++) {
            var f = facts[i]
            if (!f || f.batch || f.on_disk || f.episode === null || f.episode === undefined) continue
            var k = String(f.episode), seeds = items[i].seeders
            if (seeds === null || seeds === undefined) { var e = extras[items[i].link]; seeds = e && e.seeders !== null && e.seeders !== undefined ? e.seeders : -1 }
            if (!(k in best) || seeds > best[k].seeds) best[k] = { i: i, seeds: seeds }
        }
        var out = []
        for (var key in best) out.push(best[key].i)
        return out
    }
    readonly property int onDiskCount: { var n = 0; for (var i = 0; i < facts.length; i++) if (facts[i] && facts[i].on_disk) n++; return n }

    // ---- The lists
    readonly property var rules: frame.downloadsRules
    readonly property bool rulesLoaded: frame.downloadsRulesRead
    readonly property string rulesError: frame.downloadsRulesError
    property var torrents: []
    property var transfer: null
    // Series id to the ShowLink the last torrents read named
    property var shows: ({})
    property bool torrentsRead: false
    property string torrentsError: ""
    // What is being removed, by key: "r:" a rule name, "g:" a group key, "t:" a hash
    property var removing: ({})
    // Removals waiting their turn ({ target, choice }), and the one running
    property var queue: []
    property var running: null
    property real runningJob: -1
    property real nowMs: Date.now()
    Timer { interval: 30000; repeat: true; running: page.visible && !frame.removalOpen; onTriggered: page.nowMs = Date.now() }
    // A removal's marks stay until the read after it no longer lists what it took
    onRulesChanged: page.pruneRemoving()
    onTorrentsChanged: page.pruneRemoving()

    // ---- The show open inside the page: { key, rule, folder, seriesId, title }
    property var detail: null
    // Its feed as last read: { url, entries, loading, error }
    property var feed: null

    function reload() {
        var r = Door.getDownloads()
        if (r.error) { frame.toast(r.error.message); return }
        page.status = r.reply.status
    }
    function connect() {
        var r = Door.connectDownloads()
        if (r.error) { frame.toast(r.error.message); return }
        page.connectJob = r.reply.job !== undefined ? r.reply.job : -1
        page.connecting = true
    }
    function loadRules() { frame.readDownloadsRules() }
    // The client chip's menu: every saved client and AniBeam's own, the one in use ticked,
    // then the settings. A pick switches the downloads to it and reconnects.
    function clientMenu() {
        var s = page.status ? page.status.settings : null
        var items = []
        for (var i = 0; i < page.savedClients.length; i++) {
            var c = page.savedClients[i]
            var inUse = !!s && s.torrent_mode === "Existing" && c.id === s.active_client
            items.push({ text: c.name, icon: inUse ? "check" : "hard-drive", action: page.useClient.bind(null, c.id) })
        }
        var own = !!s && s.torrent_mode === "Managed"
        items.push({ text: "AniBeam's own", icon: own ? "check" : "play", action: page.useClient.bind(null, "") })
        items.push({ text: "Torrent client settings", icon: "settings", action: function() { frame.go("settings", { tab: "downloads" }) } })
        return items
    }
    function useClient(id) {
        var r = Door.useTorrentClient(id)
        if (r.error) { frame.toast(r.error.message); return }
        frame.forgetDownloadsRules()
        page.torrents = []
        page.repoll = true
        page.connect()
    }
    function pollTorrents() {
        var r = Door.listTorrents()
        if (r.error) page.torrentsError = r.error.message
    }
    function errorWords(e) {
        var m = String(e && e.message ? e.message : "")
        if (e && e.kind === "Service" && e.code === "qbt_auth_required") return "anirss cannot log in to " + page.clientName + ". Save its password in Settings."
        if (e && e.kind === "Service" && e.code === "unauthorized") return m + ". Check the address and password in Settings."
        return m === "" ? "Something went wrong." : m
    }

    // ---- The search
    function pendingNames() { var out = []; for (var k in page.searchResults) if (page.searchResults[k].state === "searching") out.push(k); return out }
    // Cancels what is still out and forgets what came back; the state on screen stays
    function dropSearches() {
        for (var j in page.searchJobs) Door.cancelJob(Number(j))
        page.searchJobs = ({})
        page.searchResults = ({})
        page.adoptedFrom = ""
    }
    function clearSearch() {
        page.dropSearches()
        page.state = null
        page.history = []
        page.searchError = ""
        page.notice = ""
    }
    // One search per source that is on (or per name in `only`), all at once
    function runSearch(only) {
        var q = search.text.trim()
        if (q === "") { search.focusInput(); return }
        var names = only && only.length ? only : page.enabledEndpoints.map(function(e) { return e.name })
        if (!names.length) { page.searchError = "Every source is switched off. Switch one on under the search."; return }
        page.closeGet()
        page.gridReturn = null
        page.suggestClosed = true
        page.dropSearches()
        page.searchJob = -1
        var jobs = {}, results = {}, failed = []
        for (var i = 0; i < names.length; i++) {
            var r = Door.searchReleases(q, names[i])
            if (r.error) { results[names[i]] = { state: "error", error: page.errorWords(r.error) }; failed.push(r.error.message); continue }
            jobs[r.reply.job] = names[i]
            results[names[i]] = { state: "searching" }
        }
        page.searchJobs = jobs
        page.searchResults = results
        if (!Object.keys(jobs).length) { page.searchError = failed.join(" "); return }
        page.searching = true
        page.searchStartMs = Date.now()
        page.searchElapsed = 0
        page.searchEndpoint = ""
        page.searchLabel = ""
        page.retarget()
        page.extras = ({})
        page.extrasByTitle = ({})
        page.datesState = "loading"
        page.sortKey = ""
        page.showDismissed = false
        page.searchError = ""
        page.notice = ""
        page.detail = null
        page.history = []
        page.state = null
        page.selected = {}
        page.preview = null
        page.actionLine = ""
        page.actionError = ""
    }
    // The progress card follows one source: an on-demand one still coming up, else the first
    // still out
    function retarget() {
        if (page.searchJob >= 0) return
        var pend = page.pendingNames(), pick = pend.length ? pend[0] : ""
        for (var i = 0; i < pend.length; i++) { var e = page.endpointNamed(pend[i]); if (e && e.on_demand) { pick = pend[i]; break } }
        if (pick === page.searchEndpoint) return
        page.searchEndpoint = pick
        var picked = page.endpointNamed(pick)
        page.searchTotal = picked && picked.on_demand ? picked.on_demand : 0
        page.searchLabel = ""
    }
    // Puts one source's result on screen, with the trail for Back starting afresh
    function adoptFrom(name) {
        var r = page.searchResults[name]
        if (!r || r.state !== "done") return
        var s = r.search
        page.adoptedFrom = name
        page.history = []
        page.adopt(s, s.fetched_query || s.query)
        page.notice = s.auto_resolution ? "Narrowed to " + s.auto_resolution + ", the sharpest on offer" : ""
    }
    // After a source answers, fails or is cancelled: the first result with releases goes up
    // (in the sources' order), and when none has any, the first that answered
    function settleSearch() {
        var pend = page.pendingNames().length, withItems = "", firstDone = "", errors = []
        for (var i = 0; i < page.endpoints.length; i++) {
            var n = page.endpoints[i].name, r = page.searchResults[n]
            if (!r) continue
            if (r.state === "error") errors.push(n + ": " + r.error)
            if (r.state !== "done") continue
            if (firstDone === "") firstDone = n
            if (withItems === "" && r.search.items.length > 0) withItems = n
        }
        var cur = page.state
        if (withItems !== "" && (cur === null || (cur.items.length === 0 && page.history.length === 0 && page.adoptedFrom !== withItems))) page.adoptFrom(withItems)
        else if (cur === null && pend === 0 && firstDone !== "") page.adoptFrom(firstDone)
        if (page.searchJob < 0 && (page.state !== null || pend === 0)) page.searching = false
        if (page.state === null && pend === 0 && errors.length) page.searchError = errors.join(". ")
        page.retarget()
    }
    function refine(op) {
        if (!page.state || page.searching) return
        var r = Door.refineReleases(page.state.query, page.state.endpoint, page.state.items, op)
        if (r.error) { page.searchError = r.error.message; return }
        page.searchJob = r.reply.job
        page.searching = true
        page.searchStartMs = Date.now()
        page.searchElapsed = 0
        page.searchLabel = "Narrowing on " + page.state.endpoint
        page.searchTotal = 0
        page.searchEndpoint = page.state.endpoint
        page.searchError = ""
    }
    function narrow(text) {
        var t = text.trim()
        if (t === "") return
        if (t.charAt(0) === "-" && t.length > 1) refine({ type: "exclude", term: t.slice(1).trim() })
        else refine({ type: "term", term: t })
    }
    // `wire` is the query the endpoint itself was last asked: a search's own, a refetch's,
    // and a step anirss took on its side keeps the one before it
    function adopt(s, wire) {
        page.state = { query: s.query, endpoint: s.endpoint, items: s.items || [], groups: s.groups || [], bestFit: s.best_fit || { available: false }, clientSide: !!s.client_side,
                       facts: s.facts || [], show: s.show || null, wireQuery: wire || s.query }
        page.selected = {}
        page.preview = null
        page.hoveredGroup = -1
    }
    function back() {
        if (!page.history.length) return
        var h = page.history.slice()
        page.state = h.pop()
        page.history = h
        page.selected = {}
        page.preview = null
        page.notice = ""
    }
    // The anirss endpoint a feed belongs to, by its host; empty when none matches
    function endpointFor(url) {
        var host = page.feedHost(url)
        for (var i = 0; i < page.endpoints.length; i++) if (page.feedHost(page.endpoints[i].url) === host) return page.endpoints[i].name
        return ""
    }
    function feedHost(url) { var m = String(url || "").match(/^[a-z]+:\/\/([^\/?#]+)/i); return m ? m[1] : String(url || "") }
    function cancelSearch() {
        if (!page.searching) return
        for (var j in page.searchJobs) Door.cancelJob(Number(j))
        if (page.searchJob >= 0) Door.cancelJob(page.searchJob)
    }
    // A source's chip, or a Try button: its result when it has one, else a search on it alone
    function searchOn(name) {
        var have = page.searchResults[name]
        if (have && have.state === "done") { page.adoptFrom(name); return }
        page.cancelSearch()
        Qt.callLater(function() { page.runSearch([name]) })
    }
    function sortBy(key) {
        if (page.sortKey === key) page.sortDesc = !page.sortDesc
        else { page.sortKey = key; page.sortDesc = key !== "title" }
    }
    function toggleAll(on) {
        var s = {}
        if (on) for (var i = 0; i < page.items.length; i++) s[page.items[i].link] = true
        page.selected = s
    }
    function selectMissing() {
        var s = {}
        for (var i = 0; i < page.missingPicks.length; i++) s[page.items[page.missingPicks[i]].link] = true
        page.selected = s
    }
    // anirss's groups, sorted into what a person picks by: release groups, quality, the rest
    property bool moreFilters: false
    readonly property var facets: {
        var out = { groups: [], quality: [], other: [] }
        var quality = /^(\d{3,4}p|4k|hevc|avc|x26[45]|h\.?26[45]|10-?bit|8-?bit|aac|flac|opus|ac3|eac3|ddp?|web-?dl|webrip|web|bd|bdrip|bluray|remux|cr|nf|amzn|dsnp|hidive|multi-?subs?|multisub|dual(-audio)?|s\d{1,2}|batch|hdr|dv)/i
        for (var i = 0; i < groups.length; i++) {
            var g = groups[i], label = String(g.label || "")
            var entry = { index: i, label: label.replace(/\+/g, " "), count: g.member_count }
            if (label.charAt(0) === "[") out.groups.push(entry)
            else if (quality.test(label)) out.quality.push(entry)
            else out.other.push(entry)
        }
        var byCount = function(a, b) { return b.count - a.count }
        out.groups.sort(byCount); out.quality.sort(byCount); out.other.sort(byCount)
        return out
    }
    function reasonWords(reason) {
        if (reason === "zero_results") return "That would leave nothing, so the list stays as it is."
        if (reason === "no_match") return "No release matches that term."
        if (reason === "empty_term") return "Type a term to narrow by."
        return "That would change nothing."
    }
    function toggle(item) {
        var s = JSON.parse(JSON.stringify(page.selected))
        if (s[item.link]) delete s[item.link]; else s[item.link] = true
        page.selected = s
    }
    function selectedItems() { return page.items.filter(function(i) { return page.selected[i.link] === true }) }
    function act(r) {
        if (r.error) { page.actionError = page.errorWords(r.error); return }
        page.actionJob = r.reply.job
        page.acting = true
        page.actionLine = ""
        page.actionError = ""
    }
    function subscribe() {
        if (!page.state) return
        var wire = page.state.wireQuery && page.state.wireQuery !== page.state.query ? page.state.wireQuery : ""
        act(Door.subscribe(page.state.query, page.state.endpoint, page.state.items, page.nameText, wire))
    }
    function download(items) { if (items.length) act(Door.downloadReleases(items, page.nameText)) }
    function movie() {
        var one = selectedItems()
        if (one.length !== 1) { page.actionError = "Select exactly one release to download as a movie."; return }
        act(Door.downloadMovie(one[0].link))
    }
    function previewSubscription() {
        if (!page.state) return
        var r = Door.previewSubscription(page.state.query, page.state.endpoint, page.state.items)
        if (r.error) { page.actionError = page.errorWords(r.error); return }
        page.previewJob = r.reply.job
    }

    // ---- Folders and shows. A torrent and a rule meet on the series their folder fills,
    // or on the folder itself when the library has no series for it.
    function folderOf(path) { return String(path || "").replace(/[\\/]+$/, "") }
    function lastName(path) { var f = folderOf(path); var i = Math.max(f.lastIndexOf("/"), f.lastIndexOf("\\")); return i >= 0 ? f.slice(i + 1) : f }
    function torrentKey(t) { return t.series_id !== null && t.series_id !== undefined ? "s" + t.series_id : "p" + folderOf(t.save_path) }
    function ruleKey(r) { return r.show ? "s" + r.show.series_id : "p" + folderOf(r.save_path) }
    function titleOf(r) { return r.show ? r.show.title : r.name }
    function ruleForSeries(id) { for (var i = 0; i < rules.length; i++) if (rules[i].show && rules[i].show.series_id === id) return rules[i]; return null }
    readonly property var ruleByKey: { var m = {}; for (var i = 0; i < rules.length; i++) m[ruleKey(rules[i])] = rules[i]; return m }

    // Active is what is moving or waiting to: not done, not stopped, not broken
    function rank(t) {
        var s = String(t.state || "")
        if (s === "error" || s === "missingFiles") return 3
        if (s.indexOf("paused") === 0 || s.indexOf("stopped") === 0) return (t.progress || 0) >= 1 ? 1 : 2
        return (t.progress || 0) >= 1 ? 1 : 0
    }
    function seeding(t) { var s = String(t.state || ""); return s === "uploading" || s === "stalledUP" || s === "forcedUP" || s === "queuedUP" }
    readonly property var activeTorrents: torrents.filter(function(t) { return page.rank(t) === 0 })
    readonly property var clientIndex: {
        var byHash = {}, byName = {}
        for (var i = 0; i < torrents.length; i++) { byHash[String(torrents[i].hash).toLowerCase()] = torrents[i]; byName[torrents[i].name] = torrents[i] }
        return { byHash: byHash, byName: byName }
    }
    // One group per show folder
    readonly property var showGroups: {
        var m = {}, order = []
        for (var i = 0; i < torrents.length; i++) {
            var t = torrents[i], k = page.torrentKey(t), g = m[k]
            if (!g) {
                var show = t.series_id !== null && t.series_id !== undefined ? (page.shows[t.series_id] || null) : null
                g = m[k] = { key: k, show: show, title: show ? show.title : page.lastName(t.save_path), folder: page.folderOf(t.save_path),
                             rule: page.ruleByKey[k] || null, torrents: [], size: 0, have: 0, active: 0, done: 0, broken: 0, seeding: 0,
                             dlspeed: 0, upspeed: 0, eta: 0, latest: 0 }
                order.push(g)
            }
            g.torrents.push(t)
            g.size += t.size || 0
            g.have += Math.round((t.size || 0) * (t.progress || 0))
            var r = page.rank(t)
            if (r === 0) { g.active++; if (t.eta > 0 && t.eta < 8640000) g.eta = Math.max(g.eta, t.eta) }
            if (r === 3) g.broken++
            if ((t.progress || 0) >= 1) g.done++
            if (page.seeding(t)) g.seeding++
            g.dlspeed += t.dlspeed || 0
            g.upspeed += t.upspeed || 0
            g.latest = Math.max(g.latest, t.completion_on || 0, t.added_on || 0)
        }
        order.sort(function(a, b) { return b.latest - a.latest })
        return order
    }
    function groupState(g) {
        if (g.active > 0) return "downloading"
        if (g.broken > 0) return "broken"
        if (g.seeding > 0) return "seeding"
        if (g.done === g.torrents.length) return "done"
        return "stopped"
    }
    readonly property var downloading: showGroups.filter(function(g) { return page.groupState(g) === "downloading" })
    readonly property var dlSections: {
        var defs = [
            { key: "downloading", title: "Downloading", dot: theme.purple, line: "Moving now; Activity at the right has the detail." },
            { key: "seeding", title: "Seeding", dot: theme.green, line: "Downloaded and still sharing." },
            { key: "done", title: "Done", dot: theme.textDim, line: "Downloaded; the client is not sharing them." },
            { key: "stopped", title: "Stopped", dot: theme.textFaint, line: "Stopped before they finished." },
            { key: "broken", title: "Files missing", dot: theme.red, line: "The client cannot find their files: they point at a folder that is not there. Removing the torrents tidies the client." }
        ]
        var by = {}
        for (var i = 0; i < defs.length; i++) { defs[i].items = []; by[defs[i].key] = defs[i] }
        for (var j = 0; j < showGroups.length; j++) { var st = page.groupState(showGroups[j]); if (by[st]) by[st].items.push(showGroups[j]) }
        return defs.filter(function(d) { return d.items.length > 0 })
    }
    readonly property int downloadedCount: { var n = 0; for (var i = 0; i < dlSections.length; i++) n += dlSections[i].items.length; return n }
    readonly property real downloadedSize: { var n = 0; for (var i = 0; i < torrents.length; i++) n += torrents[i].size || 0; return n }

    // The subscriptions, by where the show stands
    function subState(r) {
        var s = r.show
        if (!s) return "missing"
        if (s.complete) return "complete"
        if (s.next_airing || s.status === "Releasing" || s.status === "NotYetReleased" || s.status === "Hiatus") return "airing"
        return "incomplete"
    }
    readonly property var completeRules: rules.filter(function(r) { return page.subState(r) === "complete" })
    readonly property var subSections: {
        var defs = [
            { key: "complete", title: "Complete", dot: theme.red, line: "Every episode is in the library and nothing more will air: these subscriptions have nothing left to do." },
            { key: "airing", title: "Airing", dot: theme.accent, line: "New episodes still to come, the soonest first." },
            { key: "incomplete", title: "Incomplete", dot: theme.orange, line: "Finished airing with episodes not here, or nothing says where the show stands." },
            { key: "missing", title: "Not in the library", dot: theme.textFaint, line: "Their folder holds nothing the library has found: nothing has arrived yet, or it saves where the library does not look." }
        ]
        var by = {}
        for (var i = 0; i < defs.length; i++) { defs[i].items = []; by[defs[i].key] = defs[i] }
        for (var j = 0; j < rules.length; j++) by[page.subState(rules[j])].items.push(rules[j])
        by.airing.items.sort(function(a, b) {
            var an = a.show.next_airing ? a.show.next_airing.at : 1e12, bn = b.show.next_airing ? b.show.next_airing.at : 1e12
            return an - bn || page.titleOf(a).localeCompare(page.titleOf(b))
        })
        ;["complete", "incomplete", "missing"].forEach(function(k) { by[k].items.sort(function(a, b) { return page.titleOf(a).localeCompare(page.titleOf(b)) }) })
        return defs.filter(function(d) { return d.items.length > 0 })
    }
    // What a rule's card and row say
    function ruleLedger(r) {
        var s = r.show
        if (!s) return Fmt.plural(r.torrent_count || 0, "torrent", "torrents")
        if (s.kind === "Movie") return s.complete ? "Film, on disk" : "Film"
        var total = s.total_episodes && !s.total_is_estimate ? s.total_episodes : 0, have = s.episodes_on_disk || 0
        if (s.complete) return "All " + total + " here"
        if (s.next_airing && s.next_airing.at * 1000 > page.nowMs) return "Ep " + s.next_airing.episode + (s.next_airing.through ? " to " + s.next_airing.through : "") + (s.next_airing.predicted ? " predicted in " : " in ") + Fmt.countdown(s.next_airing.at - page.nowMs / 1000)
        if (total > 0) return have + " of " + total + " here"
        return Fmt.plural(have, "episode", "episodes") + " here"
    }
    function ruleTag(r) {
        var st = page.subState(r)
        if (st === "complete") return "Complete"
        if (st === "airing") return r.rule_enabled === false ? "Paused" : "Airing"
        if (st === "incomplete") return r.show.missing && r.show.missing.length ? r.show.missing.length + " missing" : "Unknown"
        return ""
    }
    function ruleProgress(r) {
        var s = r.show
        if (!s || s.kind === "Movie" || !s.total_episodes || s.total_is_estimate) return -1
        return (s.episodes_on_disk || 0) / s.total_episodes
    }
    function groupLedger(g) {
        if (page.groupState(g) === "downloading") {
            var parts = [Math.round(100 * g.have / Math.max(1, g.size)) + "%"]
            if (g.dlspeed > 0) parts.push(Fmt.bytes(g.dlspeed) + "/s")
            if (g.eta > 0) parts.push(Fmt.countdown(g.eta) + " left")
            return parts.join(", ")
        }
        return Fmt.plural(g.torrents.length, "torrent", "torrents") + ", " + Fmt.bytes(g.size)
    }
    function stateDot(key) {
        return key === "complete" || key === "broken" ? theme.red : key === "airing" || key === "downloading" ? theme.accent
             : key === "incomplete" ? theme.orange : key === "seeding" ? theme.green : theme.textFaint
    }
    function groupWord(key) { return key === "downloading" ? "Downloading" : key === "seeding" ? "Seeding" : key === "done" ? "Done" : key === "broken" ? "Files missing" : "Stopped" }

    // ---- The show inside the page
    function openRule(r) {
        page.detail = { key: page.ruleKey(r), rule: r.name, folder: page.folderOf(r.save_path), seriesId: r.show ? r.show.series_id : -1, title: page.titleOf(r) }
        page.ruleView = null
        Door.readRule(r.name)
        page.readFeed()
        scroll.contentY = 0
    }
    function openGroup(g) {
        page.detail = { key: g.key, rule: g.rule ? g.rule.name : "", folder: g.folder, seriesId: g.show ? g.show.series_id : -1, title: g.title }
        page.readFeed()
        scroll.contentY = 0
    }
    function closeDetail() { page.detail = null; page.feed = null; page.ruleView = null; page.watchUntil = 0 }
    // ---- A fresh subscription: the show opens on it, and for a minute and a half the page
    // reads the torrents every two seconds and the rule every four, so what qBittorrent
    // takes from the feed shows up as it happens
    property var ruleView: null
    property real repairJob: -1
    property string repairLabel: ""
    function repairRule() {
        if (page.detail === null || !page.detail.rule) return
        var r = Door.repairSubscription(page.detail.rule)
        if (r.error) { frame.toast(page.errorWords(r.error)); return }
        page.repairJob = r.reply.job
        page.repairLabel = "Trying shorter searches"
    }
    property real watchUntil: 0
    property real subscribedAt: 0
    readonly property bool watching: watchUntil > nowTick
    property real nowTick: Date.now()
    Timer { interval: 2000; repeat: true; running: page.watchUntil > Date.now() || page.watching; onTriggered: { page.nowTick = Date.now(); if (page.watching && page.detail && page.detail.rule) Door.readRule(page.detail.rule) } }
    function watchSubscription(result) {
        page.detail = { key: page.searchShow ? "s" + page.searchShow.series_id : "p" + page.folderOf(result.save_path), rule: result.name,
                        folder: page.folderOf(result.save_path), seriesId: page.searchShow ? page.searchShow.series_id : -1,
                        title: page.searchShow ? page.searchShow.title : result.name }
        page.ruleView = null
        page.subscribedAt = Date.now()
        page.watchUntil = Date.now() + 90000
        page.nowTick = Date.now()
        page.feed = { url: result.feed_url, entries: [], loading: true, error: "" }
        var r = Door.readFeed(result.feed_url)
        if (r.error) page.feed = { url: result.feed_url, entries: [], loading: false, error: page.errorWords(r.error) }
        Door.readRule(result.name)
        page.loadRules()
        page.pollTorrents()
        scroll.contentY = 0
    }
    readonly property var detailRule: {
        if (!detail || !detail.rule) return null
        for (var i = 0; i < rules.length; i++) if (rules[i].name === detail.rule) return rules[i]
        return null
    }
    readonly property var detailShow: detailRule && detailRule.show ? detailRule.show : (detail && detail.seriesId >= 0 ? (shows[detail.seriesId] || null) : null)
    function torrentsFor(key, folder) {
        return torrents.filter(function(t) { return page.torrentKey(t) === key || (folder !== "" && page.folderOf(t.save_path) === folder) })
    }
    readonly property var detailTorrents: detail ? torrentsFor(detail.key, detailRule ? folderOf(detailRule.save_path) : (detail.folder || ""))
        .sort(function(a, b) { return String(a.name).localeCompare(String(b.name)) }) : []
    function readFeed() {
        var r = page.detailRule
        if (!r || !r.feed_url) { if (!page.watching) page.feed = null; return }
        page.feed = { url: r.feed_url, entries: page.feed && page.feed.url === r.feed_url ? page.feed.entries : [], loading: true, error: "" }
        var res = Door.readFeed(r.feed_url)
        if (res.error) page.feed = { url: r.feed_url, entries: [], loading: false, error: page.errorWords(res.error) }
    }
    function downloadEntry(e) {
        var name = page.detailRule ? page.detailRule.name : page.lastName(page.detail.folder)
        page.act(Door.downloadReleases([{ title: e.title, link: e.link, seeders: e.seeders, leechers: e.leechers, downloads: e.downloads, size: e.size, page: e.page }], name))
        frame.toast("Sent " + e.title + " to " + page.clientName)
    }
    function control(hash, action) { controlMany([hash], action) }
    // The lowest episode a torrent carries, skipped files aside, so the show view and
    // Activity both list torrents in episode order; one that names none goes last
    function firstEpisode(t) {
        var n = Infinity
        if (t.episode !== null && t.episode !== undefined) n = t.episode
        var files = t.files || []
        for (var i = 0; i < files.length; i++) {
            var e = files[i].episode
            if (e !== null && e !== undefined && files[i].priority !== 0) n = Math.min(n, e)
        }
        return n === Infinity ? 1e9 : n
    }
    // Several torrents at once (Stop all and Start all). The client takes a moment: each
    // button spins until a read shows the change
    function controlMany(hashes, action) {
        if (!hashes.length) return
        var r = Door.controlTorrents(hashes, action, false)
        if (r.error) { frame.toast(page.errorWords(r.error)); return }
        var p = Object.assign({}, page.pending), now = Date.now()
        hashes.forEach(function(h) { p[h] = { action: action, at: now } })
        page.pending = p
        Door.listTorrents()
    }
    // ---- Torrent actions on their way: hash -> { action, at }, cleared by the read that
    // shows them done, or after 20 s so a button never spins forever
    property var pending: ({})
    function stoppedNow(t) { var st = String(t.state || ""); return st.indexOf("paused") === 0 || st.indexOf("stopped") === 0 }
    function landed(action, t) {
        if (action === "Stop") return page.stoppedNow(t)
        if (action === "Start") return !page.stoppedNow(t)
        if (action === "Sequential" || action === "Unsequential") return !!t.seq_dl === (action === "Sequential")
        if (action === "FirstLast" || action === "UnFirstLast") return !!t.f_l_piece_prio === (action === "FirstLast")
        return true
    }
    function settlePending() {
        var keys = Object.keys(page.pending)
        if (!keys.length) return
        var by = {}
        for (var i = 0; i < page.torrents.length; i++) by[page.torrents[i].hash] = page.torrents[i]
        var p = {}, now = Date.now()
        keys.forEach(function(h) { var e = page.pending[h], t = by[h]; if (t && now - e.at < 20000 && !page.landed(e.action, t)) p[h] = e })
        page.pending = p
    }
    // What a torrent's buttons wait on: "Stop", "Start", "Cancel" (its removal is queued or
    // running), or nothing
    function busyOf(hash) { return page.pending[hash] ? page.pending[hash].action : page.isRemoving("t:" + hash) ? "Cancel" : "" }
    Timer { interval: 700; repeat: true; running: Object.keys(page.pending).length > 0 && page.visible; onTriggered: Door.listTorrents() }

    // ---- Removals: one sheet asks, a queue runs them one at a time
    function sumSize(list) { var n = 0; for (var i = 0; i < list.length; i++) n += list[i].size || 0; return n }
    function targetForRule(r) {
        var ts = page.torrentsFor(page.ruleKey(r), page.folderOf(r.save_path))
        return { title: page.titleOf(r), rule: r.name, hashes: ts.map(function(t) { return t.hash }), series: r.show ? r.show.series_id : -1,
                 rkey: "r:" + r.name, tkeys: ts.map(function(t) { return "t:" + t.hash }), size: page.sumSize(ts), folder: r.show ? r.show.path : "" }
    }
    function targetForGroup(g) {
        return { title: g.title, rule: "", hashes: g.torrents.map(function(t) { return t.hash }), series: g.show ? g.show.series_id : -1,
                 rkey: "", tkeys: ["g:" + g.key].concat(g.torrents.map(function(t) { return "t:" + t.hash })), size: g.size, folder: g.show ? g.show.path : "" }
    }
    function targetForTorrent(t) {
        return { title: t.name, rule: "", hashes: [t.hash], series: -1, rkey: "", tkeys: ["t:" + t.hash], size: t.size || 0, folder: "" }
    }
    // A client on another machine keeps its files there: it deletes them itself, the trash cannot
    readonly property bool filesHere: {
        var s = status ? status.settings : null
        if (!s || s.torrent_mode === "Managed") return true
        var host = String(s.torrent_url).replace(/^[a-z]+:\/\//i, "").replace(/[\/?#].*$/, "").replace(/^.*@/, "")
        host = host.charAt(0) === "[" ? host.slice(1, host.indexOf("]")) : host.replace(/:\d*$/, "")
        return ["127.0.0.1", "localhost", "::1"].indexOf(host.toLowerCase()) >= 0
    }
    function filesWhere(t) {
        var torrents = page.filesHere ? "what the torrents downloaded" : "what the torrents downloaded, deleted by " + page.activeClientName
        return t.folder !== "" ? t.folder + " and " + torrents : torrents
    }
    function askUnsubscribe(anchor, r) {
        var t = page.targetForRule(r)
        frame.askRemoval(anchor, {
            heading: "Unsubscribe from " + t.title + "?",
            base: "Its feed and download rule leave " + page.clientName + ", so nothing new arrives.",
            rule: true, torrentCount: t.hashes.length,
            canFiles: t.hashes.length > 0 || t.series >= 0,
            filesWhere: page.filesWhere(t)
        }, function(choice) { page.enqueue([t], choice) })
    }
    function askUnsubscribeComplete(anchor) {
        var ts = page.completeRules.map(page.targetForRule), n = 0
        for (var i = 0; i < ts.length; i++) n += ts[i].hashes.length
        frame.askRemoval(anchor, {
            heading: "Unsubscribe from all " + ts.length + " complete shows?",
            base: "Their feeds and download rules leave " + page.clientName + ". Every episode of them is in the library.",
            rule: true, torrentCount: n, canFiles: true,
            filesWhere: "their folders and what their torrents downloaded",
            note: "They go one at a time; the page shows each as it goes."
        }, function(choice) { page.enqueue(ts, choice) })
    }
    function askRemoveGroup(anchor, g) {
        var t = page.targetForGroup(g)
        frame.askRemoval(anchor, {
            heading: "Remove " + t.title + "?",
            base: (t.hashes.length === 1 ? "Its torrent leaves " : "Its " + t.hashes.length + " torrents leave ") + page.clientName + "; the files stay unless you tick below."
                  + (g.rule ? " The subscription stays." : ""),
            rule: false, torrentCount: t.hashes.length, canFiles: true,
            filesWhere: page.filesWhere(t)
        }, function(choice) { page.enqueue([t], choice) })
    }
    function askRemoveBroken(anchor, groups) {
        var ts = groups.map(page.targetForGroup), n = 0
        for (var i = 0; i < ts.length; i++) n += ts[i].hashes.length
        frame.askRemoval(anchor, {
            heading: "Remove the torrents of all " + ts.length + " shows with files missing?",
            base: "Their " + n + " torrents leave " + page.clientName + ". They point at folders that are not there, so no file is touched; subscriptions stay.",
            rule: false, torrentCount: n, canFiles: false,
            note: "They go one at a time; the page shows each as it goes."
        }, function(choice) { page.enqueue(ts, { torrents: true, files: false }) })
    }
    function askRemoveTorrent(anchor, torrent) {
        var t = page.targetForTorrent(torrent)
        frame.askRemoval(anchor, {
            heading: "Remove this torrent?",
            base: torrent.name,
            rule: false, torrentCount: 1, canFiles: true,
            filesWhere: "what it downloaded"
        }, function(choice) { page.enqueue([t], choice) })
    }
    // Stopping a download part way: the torrent goes, and what it wrote so far with it
    // unless unticked, since half an episode is no use to anyone
    function askCancelTorrent(anchor, torrent) {
        var t = page.targetForTorrent(torrent)
        frame.askRemoval(anchor, {
            heading: "Cancel this download?",
            base: torrent.name,
            rule: false, torrentCount: 1, canFiles: true, filesOn: true,
            filesWhere: "what it has downloaded so far",
            confirmText: "Cancel the download", keepText: "Keep downloading"
        }, function(choice) { page.enqueue([t], choice) })
    }
    function askCancelTorrents(anchor, list) {
        if (list.length === 1) { page.askCancelTorrent(anchor, list[0]); return }
        frame.askRemoval(anchor, {
            heading: "Cancel " + Fmt.plural(list.length, "download", "downloads") + "?",
            base: "Every download of this show stops and leaves " + page.clientName + ".",
            rule: false, torrentCount: list.length, canFiles: true, filesOn: true,
            filesWhere: "what they have downloaded so far",
            confirmText: "Cancel them all", keepText: "Keep downloading"
        }, function(choice) { page.enqueue(list.map(function(x) { return page.targetForTorrent(x) }), choice) })
    }
    function askRemoveDetail(anchor) {
        if (page.detail === null) return
        var t = { title: page.detail.title, rule: "", hashes: page.detailTorrents.map(function(x) { return x.hash }), series: page.detail.seriesId,
                  rkey: "", tkeys: ["g:" + page.detail.key].concat(page.detailTorrents.map(function(x) { return "t:" + x.hash })), size: page.sumSize(page.detailTorrents),
                  folder: page.detailShow ? page.detailShow.path : "" }
        frame.askRemoval(anchor, {
            heading: "Remove " + t.title + "?",
            base: "Its " + Fmt.plural(t.hashes.length, "torrent leaves", "torrents leave") + " " + page.clientName + "; the files stay unless you tick below." + (page.detailRule ? " The subscription stays." : ""),
            rule: false, torrentCount: t.hashes.length, canFiles: true, filesWhere: page.filesWhere(t)
        }, function(choice) { page.enqueue([t], choice) })
    }
    function mark(keys, on) {
        var o = JSON.parse(JSON.stringify(page.removing))
        for (var i = 0; i < keys.length; i++) { if (on) o[keys[i]] = true; else delete o[keys[i]] }
        page.removing = o
    }
    function isRemoving(key) { return page.removing[key] === true }
    function pruneRemoving() {
        var names = {}, hashes = {}, groups = {}, live = {}
        for (var i = 0; i < rules.length; i++) names[rules[i].name] = true
        for (var j = 0; j < torrents.length; j++) { hashes[torrents[j].hash] = true; groups[page.torrentKey(torrents[j])] = true }
        if (page.running) page.running.keys.forEach(function(k) { live[k] = true })
        page.queue.forEach(function(q) { q.keys.forEach(function(k) { live[k] = true }) })
        var o = {}
        for (var k in page.removing) {
            var kind = k.slice(0, 2), v = k.slice(2)
            var still = kind === "r:" ? names[v] : kind === "t:" ? hashes[v] : groups[v]
            if (still || live[k]) o[k] = true
        }
        page.removing = o
    }
    function enqueue(targets, choice) {
        var q = page.queue.slice()
        for (var i = 0; i < targets.length; i++) {
            var t = targets[i], keys = (t.rkey ? [t.rkey] : []).concat(choice.torrents || choice.files ? t.tkeys : [])
            q.push({ target: t, choice: choice, keys: keys })
            page.mark(keys, true)
        }
        page.queue = q
        page.startNext()
    }
    function startNext() {
        if (page.running !== null || !page.queue.length) return
        var q = page.queue.slice(), next = q.shift()
        page.queue = q
        var t = next.target, c = next.choice
        var r = Door.removeShow(t.title, t.rule, t.hashes, c.torrents, c.files, t.series)
        if (r.error) {
            frame.toast(page.errorWords(r.error))
            page.mark(next.keys, false)
            page.startNext()
            return
        }
        page.running = next
        page.runningJob = r.reply.job
    }
    function finishRunning(ok) {
        var done = page.running
        page.running = null
        page.runningJob = -1
        if (done && !ok) page.mark(done.keys, false)
        if (done && ok && page.detail !== null && ((done.target.rule !== "" && page.detail.rule === done.target.rule) || done.keys.indexOf("g:" + page.detail.key) >= 0)) page.closeDetail()
        page.startNext()
    }
    readonly property int queued: queue.length + (running !== null ? 1 : 0)

    // ---- The head's one line
    readonly property string summary: {
        if (!page.status) return "Connecting."
        if (page.connecting && !page.anirssReady) return "Starting anirss."
        var a = page.anirss
        if (a.state === "Missing") return "anirss is not installed. " + a.message
        if (a.state === "Failed") return "anirss is not running: " + a.message
        if (a.state === "Off") return "anirss is not running yet."
        var t = page.torrentOn ? page.activeClientName + " " + page.torrent.version + " is connected." : page.activeClientName + " is not connected: " + (page.torrent.message || "no answer") + "."
        return "Searches run through anirss " + a.version + ". " + t
    }

    Connections {
        target: Door
        function onSearchFinished(job, res) {
            if (job !== page.aniJob) return
            page.aniHits = res || []
        }
        function onDownloadsStatusChanged(status) {
            page.status = status
            page.connecting = false
            page.checkJob = -1
            if (page.anirssReady && !frame.downloadsRulesRead) page.loadRules()
            if (page.repoll && page.torrentOn) { page.repoll = false; page.pollTorrents() }
        }
        function onJobProgress(job, kind, done, total, label) {
            if (job === page.repairJob) { page.repairLabel = label; return }
            if (page.getJobs[job] !== undefined) {
                var n = page.getJobs[job], cur = page.getResults[n]
                if (cur && cur.state === "searching") page.setResult(n, Object.assign({}, cur, { label: label }))
                return
            }
            var sn = page.searchJobs[job]
            if (sn !== undefined) { if (sn === page.searchEndpoint && page.searchJob < 0) page.searchLabel = label; return }
            if (job !== page.searchJob || !page.searching) return
            page.searchLabel = label
        }
        function onJobFinished(job, kind, ok) {
            if (job === page.startJob && ok) page.startDone(true, null)
        }
        function onReleasesEnriched(query, endpoint, entries) {
            if (!page.state && !page.searching) return
            var byLink = JSON.parse(JSON.stringify(page.extras)), byTitle = JSON.parse(JSON.stringify(page.extrasByTitle))
            for (var i = 0; i < entries.length; i++) {
                if (entries[i].link) byLink[entries[i].link] = entries[i]
                if (entries[i].title) byTitle[entries[i].title] = entries[i]
            }
            page.extras = byLink
            page.extrasByTitle = byTitle
            page.datesState = "ready"
        }
        function onReleasesFound(job, search) {
            if (page.getJobs[job] !== undefined) {
                var name = page.getJobs[job]
                var j = Object.assign({}, page.getJobs); delete j[job]; page.getJobs = j
                // A fallback answered from another endpoint: none of it is this provider's
                var own = search.endpoint === name
                var tried = page.getResults[name] ? page.getResults[name].at || 0 : 0
                var qs = page.getShow ? page.getShow.queries || [] : []
                if ((!own || !search.items.length) && tried + 1 < qs.length) { page.searchGetAs(name, tried + 1); return }
                page.setResult(name, { state: "done", items: own ? search.items : [], facts: own ? search.facts : [], query: search.query, search: search })
                return
            }
            var from = page.searchJobs[job]
            if (from === undefined) return
            var rest = Object.assign({}, page.searchJobs); delete rest[job]; page.searchJobs = rest
            // A fallback answered from another endpoint: none of it is this source's, and the
            // source it fell back to is searched on its own anyway
            var shown = search.endpoint === from ? search : Object.assign({}, search, { endpoint: from, items: [], groups: [], facts: [], show: null, best_fit: { available: false } })
            var res = Object.assign({}, page.searchResults); res[from] = { state: "done", search: shown }; page.searchResults = res
            page.settleSearch()
        }
        function onReleasesRefined(job, refinement) {
            if (job !== page.searchJob) return
            page.searchJob = -1
            page.searching = false
            if (refinement.applied) {
                var h = page.history.slice(); h.push(page.state); page.history = h
                page.adopt(refinement, refinement.mode === "refetch" ? refinement.query : (page.state.wireQuery || page.state.query))
                page.notice = ""
            } else {
                page.notice = page.reasonWords(refinement.reason)
            }
        }
        function onSubscriptionPreviewed(job, spec) { if (job === page.previewJob) page.preview = spec }
        function onSubscribed(job, result) {
            if (job === page.followJob) {
                page.followJob = -1
                page.getActing = false
                page.getLine = (page.getLine !== "" ? page.getLine + " " : "") + "Subscribed as " + result.name + "."
                page.loadRules()
                return
            }
            if (job !== page.actionJob) return
            page.acting = false
            page.actionLine = "Subscribed as " + result.name + "."
            page.watchSubscription(result)
        }
        function onSubscriptionRepaired(job, name, feedUrl, mustContain, found) {
            if (job !== page.repairJob) return
            page.repairJob = -1
            page.repairLabel = ""
            frame.toast("Repaired " + name + ": the feed finds " + Fmt.plural(found, "release", "releases") + (mustContain ? " and the rule keeps \u201c" + mustContain + "\u201d" : ""), 6)
            page.loadRules()
            if (page.detail !== null && page.detail.rule === name) {
                page.feed = { url: feedUrl, entries: [], loading: true, error: "" }
                Door.readFeed(feedUrl)
                Door.readRule(name)
                page.subscribedAt = Date.now()
                page.watchUntil = Date.now() + 90000
                page.nowTick = Date.now()
                page.pollTorrents()
            }
        }
        function onRuleRead(rule) {
            if (page.detail === null || page.detail.rule !== rule.name) return
            page.ruleView = rule
        }
        function onTorrentsAdded(job, result) {
            if (job === page.getJob) {
                page.getJob = -1
                page.getLine = "Sent " + Fmt.plural(result.count, "torrent", "torrents") + " to " + page.clientName + "."
                page.pollTorrents()
                if (page.pendingFollow) page.follow(page.pendingFollow)
                else page.getActing = false
                return
            }
            if (job !== page.actionJob) return
            page.acting = false
            page.actionLine = "Added " + Fmt.plural(result.count, "torrent", "torrents") + "."
            page.selected = {}
            page.pollTorrents()
        }
        function onTorrentsRead(torrents, transfer, shows) {
            if (frame.removalOpen) return
            var m = {}
            for (var i = 0; i < shows.length; i++) m[shows[i].series_id] = shows[i]
            page.shows = m
            page.torrents = torrents
            page.transfer = transfer
            page.torrentsRead = true
            page.torrentsError = ""
            page.settlePending()
        }
        function onFeedRead(url, entries) {
            if (!page.feed || page.feed.url !== url) return
            var sorted = entries.slice().sort(function(a, b) { return (b.published || 0) - (a.published || 0) })
            page.feed = { url: url, entries: sorted, loading: false, error: "" }
        }
        function onDownloadsUnavailable(what, error) {
            if (what === "dates") { if (page.datesState === "loading") page.datesState = "none"; return }
            if (what === "torrents") page.torrentsError = page.errorWords(error)
            else if (what === "feed" && page.feed) page.feed = { url: page.feed.url, entries: page.feed.entries, loading: false, error: page.errorWords(error) }
        }
        function onShowRemoved(job, title, rule, hashes, trashed) {
            if (job !== page.runningJob) return
            if (trashed.length) frame.toast(Fmt.plural(trashed.length, "path", "paths") + " of " + title + " moved to the trash")
            page.finishRunning(true)
            page.pollTorrents()
        }
        function onTorrentsControlled(hashes, action) { page.pollTorrents() }
        function onJobFailed(job, kind, error) {
            if (page.getJobs[job] !== undefined) {
                var gn = page.getJobs[job]
                var gj = Object.assign({}, page.getJobs); delete gj[job]; page.getJobs = gj
                page.setResult(gn, { state: "error", error: page.errorWords(error) })
                return
            }
            if (job === page.getJob || job === page.followJob) { page.getJob = -1; page.followJob = -1; page.getActing = false; page.pendingFollow = null; page.getError = page.errorWords(error); return }
            if (job === page.checkJob) { page.checkJob = -1; frame.toast(page.errorWords(error)); return }
            if (page.searchJobs[job] !== undefined) {
                var fn = page.searchJobs[job]
                var fj = Object.assign({}, page.searchJobs); delete fj[job]; page.searchJobs = fj
                var fr = Object.assign({}, page.searchResults); fr[fn] = { state: "error", error: page.errorWords(error) }; page.searchResults = fr
                page.settleSearch()
                return
            }
            if (job === page.startJob) { page.startDone(false, error); return }
            if (job === page.searchJob) { page.searchJob = -1; page.searching = false; page.searchError = page.errorWords(error) }
            else if (job === page.actionJob) { page.acting = false; page.actionError = page.errorWords(error) }
            else if (job === page.connectJob) { page.connecting = false; frame.toast(page.errorWords(error)) }
            else if (job === page.previewJob) { page.previewJob = -1 }
            else if (job === page.repairJob) { page.repairJob = -1; page.repairLabel = ""; frame.toast(page.errorWords(error), 6) }
            else if (job === page.runningJob) {
                frame.toast(page.errorWords(error))
                // A failure stops the run: what is left is not removed on a guess
                page.queue.forEach(function(q) { page.mark(q.keys, false) })
                page.queue = []
                page.finishRunning(false)
            } else if (kind === "ControlTorrents") frame.toast(page.errorWords(error))
        }
        function onJobCancelled(job, kind) {
            if (page.getJobs[job] !== undefined) { var cn = page.getJobs[job]; var cj = Object.assign({}, page.getJobs); delete cj[job]; page.getJobs = cj; page.setResult(cn, { state: "idle" }) }
            if (job === page.runningJob) { page.queue.forEach(function(q) { page.mark(q.keys, false) }); page.queue = []; page.finishRunning(false) }
            if (page.searchJobs[job] !== undefined) {
                var sn = page.searchJobs[job]
                var sj = Object.assign({}, page.searchJobs); delete sj[job]; page.searchJobs = sj
                var sr = Object.assign({}, page.searchResults); sr[sn] = { state: "idle" }; page.searchResults = sr
                page.settleSearch()
            }
            if (job === page.startJob) { page.startJob = -1; page.startName = ""; startSheet.busy = false }
            if (job === page.searchJob) { page.searchJob = -1; page.searching = false }
            else if (job === page.actionJob) page.acting = false
            else if (job === page.connectJob) page.connecting = false
        }
    }
    // The one poll in the app: the transfers, while this page is up; quick only while
    // something moves, held while the removal sheet is open (a fresh list would rebuild
    // what it was opened from)
    Timer { interval: page.activeTorrents.length || page.watching ? 2000 : 15000; running: page.visible && Door.ready && page.torrentOn && !frame.removalOpen; repeat: true; onTriggered: page.pollTorrents() }
    Component.onCompleted: {
        reload()
        connect()
        loadRules()
        pollTorrents()
        loadLibrary()
        if (props.query) { search.text = props.query; Qt.callLater(page.runSearch) }
        // A series page's Get episodes (#169): straight to that show's view
        if (props.series !== undefined && props.series !== null) page.openSeries(props.series)
    }
    function openSeries(id) {
        var r = Door.getSeries(id)
        if (r.error || !r.reply || !r.reply.detail) { frame.toast("That show is no longer in the library"); return }
        page.openGet(page.showFromCard(r.reply.detail.card))
    }

    // ---- The geometry
    readonly property real gap: theme.space(4)
    readonly property real paneWidth: theme.space(90)
    readonly property real tileGap: theme.grout
    readonly property int cols: Math.max(1, Math.floor((scroll.innerWidth + tileGap) / (theme.posterWidth + tileGap)))
    readonly property real cardWidth: Math.floor((scroll.innerWidth - tileGap * (cols - 1)) / cols)
    readonly property bool home: page.detail === null && page.getShow === null && page.state === null && !page.searching && page.searchError === ""

    // ---- Activity at the right, while the window has the width for it. Narrower, it folds
    // to a slim strip: a button with what is moving, which slides the column in over the page
    readonly property bool activityShown: page.width >= theme.space(250)
    property bool activityOpen: false
    onActivityShownChanged: if (activityShown) activityOpen = false
    readonly property int movingCount: page.torrents.filter(function(t) { return activity.moving(t) }).length
    Item {
        id: activityStrip
        objectName: "activity-strip"
        visible: !page.activityShown
        anchors.right: parent.right
        anchors.top: parent.top
        anchors.bottom: parent.bottom
        width: visible ? theme.space(16) : 0
        Rectangle { anchors.left: parent.left; anchors.top: parent.top; anchors.bottom: parent.bottom; width: 1; color: theme.line }
        Column {
            anchors.horizontalCenter: parent.horizontalCenter
            y: theme.space(6)
            spacing: theme.space(2)
            Button {
                objectName: "activity-open"
                anchors.horizontalCenter: parent.horizontalCenter
                width: theme.space(11); height: width
                icon: "activity"
                primary: page.activityOpen
                onClicked: page.activityOpen = !page.activityOpen
                Tooltip { text: page.activityOpen ? "Hide activity" : "Activity" }
            }
            Label {
                visible: page.movingCount > 0
                anchors.horizontalCenter: parent.horizontalCenter
                text: String(page.movingCount)
                role: "figure"; font.pointSize: theme.typeSmall; color: theme.textDim
                Tooltip { text: Fmt.plural(page.movingCount, "download", "downloads") + " moving" }
            }
        }
    }
    // Over the page, a click beside the column folds it away again
    MouseArea {
        visible: page.activityOpen && !page.activityShown
        z: 4
        anchors.left: parent.left
        anchors.top: parent.top
        anchors.bottom: parent.bottom
        anchors.right: activityStrip.left
        onClicked: page.activityOpen = false
        onWheel: function(w) { w.accepted = false }
    }
    Rectangle {
        visible: activity.visible && !page.activityShown
        z: 5
        anchors.fill: activity
        color: theme.bg
    }
    ActivityColumn {
        id: activity
        visible: page.activityShown || page.activityOpen
        z: 5
        anchors.right: page.activityShown ? parent.right : activityStrip.left
        anchors.top: parent.top
        anchors.bottom: parent.bottom
        width: visible ? theme.space(88) : 0
        torrents: page.torrents
        transfer: page.transfer
        shows: page.shows
        connected: page.torrentOn
        clientName: page.clientName
        busyOf: function(hash) { return page.busyOf(hash) }
        firstEpisode: function(t) { return page.firstEpisode(t) }
        onCancelTorrent: function(anchor, t) { page.askCancelTorrent(anchor, t) }
        onControl: function(hash, action) { page.control(hash, action) }
        onControlAll: function(hashes, action) { page.controlMany(hashes, action) }
        onCancelAll: function(anchor, list) { page.askCancelTorrents(anchor, list) }
        onOpenShow: function(t) { page.openFromTorrent(t) }
    }
    Rectangle { visible: activity.visible; z: 5; anchors.right: activity.left; anchors.top: parent.top; anchors.bottom: parent.bottom; width: 1; color: theme.line }

    PageScroll {
        id: scroll
        anchors.left: parent.left
        anchors.top: parent.top
        anchors.bottom: parent.bottom
        anchors.right: page.activityShown ? activity.left : activityStrip.left
        readonly property real innerWidth: width - theme.space(16)

        // ---- The head: back (with a show open), the title, the two halves in plain words
        Item {
            width: parent.width
            height: theme.space(20)
            Button {
                id: backButton
                objectName: "downloads-back"
                visible: page.getShow !== null
                anchors.left: parent.left
                anchors.verticalCenter: parent.verticalCenter
                width: visible ? theme.space(14) : 0
                height: theme.space(13)
                radius: theme.space(5)
                flat: true
                onClicked: page.closeGet()
                Icon { anchors.centerIn: parent; glyph: "chevron-left"; size: theme.space(8); color: theme.text }
                Tooltip { text: "Back to Downloads" }
            }
            Label {
                id: titleLabel
                anchors.left: backButton.visible ? backButton.right : parent.left
                anchors.leftMargin: backButton.visible ? theme.space(4) : 0
                anchors.verticalCenter: parent.verticalCenter
                text: "Downloads"
                role: "display"
                font.pointSize: theme.typePage
            }
            Row {
                id: chips
                anchors.right: parent.right
                anchors.verticalCenter: parent.verticalCenter
                spacing: theme.space(4)
                Label {
                    objectName: "downloads-anirss-chip"
                    anchors.verticalCenter: parent.verticalCenter
                    text: page.anirssReady ? "anirss " + page.anirss.version : page.connecting ? "anirss starting" : "anirss off"
                    role: "ledger"; font.pointSize: theme.typeNormal
                    color: page.anirssReady || page.connecting ? theme.textFaint : theme.red
                    MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: frame.go("settings", { tab: "downloads" }) }
                    Tooltip { text: page.anirssReady ? "anirss at " + page.anirss.url + ". " + page.summary : (page.anirss && page.anirss.message ? page.anirss.message : "Open the Downloads settings") }
                }
                Label {
                    objectName: "downloads-client-chip"
                    anchors.verticalCenter: parent.verticalCenter
                    text: page.torrentOn ? page.activeClientName + " " + page.torrent.version : page.activeClientName + " off"
                    role: "ledger"; font.pointSize: theme.typeNormal
                    color: page.torrentOn ? theme.textFaint : theme.red
                    MouseArea {
                        anchors.fill: parent
                        cursorShape: Qt.PointingHandCursor
                        onClicked: {
                            if (page.remoteAnirss) { frame.go("settings", { tab: "downloads" }); return }
                            var p = parent.mapToItem(frame, 0, parent.height); frame.openMenu(p.x, p.y, page.clientMenu())
                        }
                    }
                    Tooltip { text: (page.torrentOn ? page.status.settings.torrent_url : (page.torrent && page.torrent.message ? page.torrent.message : "Not connected")) + (page.remoteAnirss ? "\nA remote anirss uses the client its own config names" : "\nClick to switch clients") }
                }
                Button {
                    text: page.connecting ? "Connecting" : "Reconnect"
                    icon: "refresh-cw"
                    flat: true
                    enabled: !page.connecting
                    onClicked: page.connect()
                }
            }
        }

        // ---- The search row: one wide field (shows as you type, release names on Return or
        // its round button)
        SearchField {
            id: search
            objectName: "downloads-search"
            large: true
            width: parent.width
            placeholder: "Search for a show, or a release name"
            hint: ""
            goButton: true
            goBusy: page.searching
            goTip: page.enabledEndpoints.length ? "Search release names on " + page.enabledEndpoints.map(function(e) { return e.name }).join(", ") : "Every source is switched off."
            onGo: page.runSearch()
            onAccepted: {
                if (page.suggesting && page.suggestIndex > 0 && page.suggestRows.length >= page.suggestIndex) page.pickSuggestion(page.suggestRows[page.suggestIndex - 1])
                else page.runSearch()
            }
            onStepped: function(by) { if (page.suggesting) page.suggestIndex = Math.max(0, Math.min(page.suggestRows.length, page.suggestIndex + by)) }
            onTextChanged: { page.suggestClosed = false; page.suggestIndex = 0; aniDebounce.restart() }
            onCleared: { page.aniHits = [] }
        }

        // ---- Shows that match what is typed: the library's own, then AniList's
        Tile {
            id: suggestTile
            objectName: "downloads-suggestions"
            visible: page.suggesting
            width: parent.width
            implicitHeight: suggestBody.implicitHeight + padding * 2
            height: visible ? implicitHeight : 0
            padding: theme.space(2)
            lit: false
            Column {
                id: suggestBody
                width: parent.width
                Repeater {
                    model: page.suggestRows
                    Item {
                        id: srow
                        required property var modelData
                        required property int index
                        readonly property var c: modelData.lib || null
                        readonly property var h: modelData.ani || null
                        readonly property bool current: page.suggestIndex === index + 1
                        width: parent.width
                        height: theme.space(22)
                        Rectangle { anchors.fill: parent; radius: theme.radiusMd; color: srow.current || sMouse.containsMouse ? theme.glassFloat : "transparent" }
                        MouseArea { id: sMouse; anchors.fill: parent; hoverEnabled: true; cursorShape: Qt.PointingHandCursor; onClicked: page.pickSuggestion(srow.modelData) }
                        Corner {
                            id: sPoster
                            x: theme.space(3)
                            anchors.verticalCenter: parent.verticalCenter
                            width: theme.space(12); height: width * 1.45
                            radius: theme.radiusSm
                            smoothing: theme.cornerSmoothing
                            color: theme.tileSunken
                            Image {
                                id: sImage
                                visible: false
                                anchors.fill: parent
                                source: srow.c ? (srow.c.poster ? Fmt.fileUrl(srow.c.poster) : "") : (srow.h && srow.h.cover_url ? srow.h.cover_url : "")
                                sourceSize: Qt.size(96, 140)
                                fillMode: Image.PreserveAspectCrop
                                asynchronous: true
                            }
                            fillItem: sImage.status === Image.Ready ? sImage : null
                        }
                        Column {
                            anchors.left: sPoster.right
                            anchors.leftMargin: theme.space(5)
                            anchors.right: sRight.left
                            anchors.rightMargin: theme.space(5)
                            anchors.verticalCenter: parent.verticalCenter
                            spacing: theme.space(1)
                            Label { width: parent.width; text: srow.c ? page.nameOfCard(srow.c, page.query) : page.nameOfHit(srow.h, page.query); role: "display"; font.pointSize: theme.typeHeading; font.variableAxes: ({ "wdth": 70, "wght": 420 }); elide: Text.ElideRight }
                            Label {
                                width: parent.width
                                text: {
                                    if (srow.c) {
                                        var parts = []
                                        if (srow.c.next_airing) parts.push("Airing, " + Fmt.episodeSpan(srow.c.next_airing.episode, srow.c.next_airing.through || 0).toLowerCase() + (srow.c.next_airing.predicted ? " predicted in " : " in ") + Fmt.countdown(srow.c.next_airing.at - page.nowMs / 1000))
                                        else if (srow.c.status === "Finished") parts.push("Finished")
                                        var gw = page.gapWords(srow.c)
                                        if (gw) parts.push(gw)
                                        return parts.join(". ")
                                    }
                                    var b = []
                                    if (srow.h.format) b.push(srow.h.format)
                                    if (srow.h.year) b.push(String(srow.h.year))
                                    if (srow.h.episodes) b.push(srow.h.episodes + " episodes")
                                    return b.join(", ")
                                }
                                role: "ui"; color: theme.textDim; elide: Text.ElideRight
                            }
                        }
                        Column {
                            id: sRight
                            anchors.right: parent.right
                            anchors.rightMargin: theme.space(4)
                            anchors.verticalCenter: parent.verticalCenter
                            width: theme.space(60)
                            spacing: theme.space(1.5)
                            EpisodeCubes {
                                visible: srow.c !== null && String(srow.c.cubes || "") !== ""
                                anchors.right: parent.right
                                width: Math.min(implicitWidth, parent.width)
                                height: implicitHeight
                                codes: srow.c ? srow.c.cubes || "" : ""
                                cubeSize: theme.space(2)
                                minSize: 2
                                maxRows: 2
                                tipLimit: 0
                            }
                            Label { anchors.right: parent.right; text: srow.c ? "In your library" : "On AniList, not in your library"; role: "ui"; font.pointSize: theme.typeSmall; color: theme.textFaint }
                        }
                    }
                }
                Item {
                    width: parent.width
                    height: theme.space(14)
                    Rectangle { anchors.fill: parent; radius: theme.radiusMd; color: page.suggestIndex === 0 ? theme.glassFloat : "transparent" }
                    Label {
                        x: theme.space(4)
                        anchors.verticalCenter: parent.verticalCenter
                        text: "Search release names for \u201c" + page.query + "\u201d on " + (page.enabledEndpoints.length ? page.enabledEndpoints.map(function(e) { return e.name }).join(", ") : "no source") + " instead"
                        role: "ui"; color: theme.textDim
                    }
                    Label { anchors.right: parent.right; anchors.rightMargin: theme.space(4); anchors.verticalCenter: parent.verticalCenter; text: "Return"; role: "ledger"; color: theme.textFaint }
                    MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: page.runSearch() }
                }
            }
        }

        // ---- The sources: a chip per provider, lit when on, greyed when off; a click flips it.
        // An on-demand one that is down says so, and switching it on asks to start it first
        Flow {
            objectName: "downloads-sources"
            visible: page.endpoints.length > 0
            width: parent.width
            spacing: theme.space(2)
            // Check stands as tall as the chips beside it
            readonly property real chipHeight: sourceChips.count > 0 && sourceChips.itemAt(0) ? sourceChips.itemAt(0).height : theme.space(7)
            Label { height: parent.chipHeight; verticalAlignment: Text.AlignVCenter; text: "Sources"; role: "ui"; color: theme.textFaint; rightPadding: theme.space(2) }
            Repeater {
                id: sourceChips
                model: page.endpoints
                Chip {
                    id: src
                    required property var modelData
                    readonly property bool on: page.sourceOn(modelData.name)
                    readonly property bool down: !!modelData.on_demand && modelData.running !== true
                    readonly property bool starting: page.startJob >= 0 && page.startName === modelData.name
                    objectName: "downloads-source-" + modelData.name
                    text: modelData.name
                    mono: false
                    selected: on
                    dimmed: !on
                    clickable: true
                    hint: starting ? "starting" : down ? "not running" : ""
                    onClicked: page.toggleSource(modelData.name)
                    Tooltip {
                        text: src.down
                            ? modelData.name + " runs behind Jackett, which is not running. Click to start it."
                            : (src.on ? "Searches ask " + modelData.name + ". Click to switch it off." : "Switched off. Click to ask " + modelData.name + " again.")
                              + (modelData.on_demand ? "" : " " + modelData.url)
                    }
                }
            }
            Button {
                visible: page.endpoints.some(function(e) { return !!e.on_demand })
                height: parent.chipHeight
                text: page.checkJob >= 0 ? "Checking" : "Check"
                icon: "refresh-cw"
                flat: true
                small: true
                enabled: page.checkJob < 0
                onClicked: page.checkSources()
                Tooltip { text: "Asks whether each on-demand source is running. Starts nothing." }
            }
        }

        // ---- The search up: the query line, the groups, Narrow and Best fit, then the
        // releases and the actions pane
        Column {
            visible: page.detail === null && (page.state !== null || page.searching || page.searchError !== "")
            width: parent.width
            spacing: theme.space(3)
            Row {
                width: parent.width
                height: theme.controlHeight
                spacing: theme.space(3)
                Button {
                    objectName: "downloads-back-to-grid"
                    visible: page.gridReturn !== null
                    text: "Back to the grid"
                    icon: "layout-grid"
                    small: true
                    anchors.verticalCenter: parent.verticalCenter
                    onClicked: page.backToGrid()
                    Tooltip { text: page.gridReturn ? "Back to " + page.gridReturn.show.title + " and what each provider found. Escape does the same." : "" }
                }
                Button { text: "Back"; icon: "arrow-left"; flat: true; small: true; enabled: page.history.length > 0; anchors.verticalCenter: parent.verticalCenter; onClicked: page.back() }
                Label { visible: !!page.state; text: page.state ? page.state.query : ""; role: "title"; font.pointSize: theme.typeNormal; anchors.verticalCenter: parent.verticalCenter; elide: Text.ElideMiddle; width: Math.min(implicitWidth, parent.width * 0.5) }
                Label { visible: !!page.state; text: page.state ? Fmt.plural(page.items.length, "release", "releases") + " on " + page.state.endpoint : ""; role: "ledger"; color: theme.textDim; anchors.verticalCenter: parent.verticalCenter }
                Label { visible: page.notice !== "" && !page.searching; text: page.notice; role: "ui"; color: theme.textDim; anchors.verticalCenter: parent.verticalCenter; elide: Text.ElideRight }
                Label { visible: page.searchError !== ""; text: page.searchError; role: "ui"; color: theme.red; anchors.verticalCenter: parent.verticalCenter; elide: Text.ElideRight }
            }
            // While it runs: what it is doing, for how long, and the ways out
            SearchProgress {
                visible: page.searching
                width: parent.width
                label: page.searchLabel
                elapsed: page.searchElapsed
                total: page.searchTotal
                endpoint: page.searchEndpoint
                onCancel: page.cancelSearch()
            }
            // Each source the search asked: what it found, and a click shows it
            Row {
                objectName: "downloads-search-sources"
                visible: Object.keys(page.searchResults).length > 1 && (page.state !== null || page.searching)
                spacing: theme.space(2)
                Label { anchors.verticalCenter: parent.verticalCenter; text: "Found on"; role: "ui"; color: theme.textFaint; rightPadding: theme.space(1) }
                Repeater {
                    model: page.endpoints.map(function(e) { return e.name }).filter(function(n) { return page.searchResults[n] !== undefined })
                    Chip {
                        id: found
                        required property string modelData
                        readonly property var res: page.searchResults[modelData]
                        objectName: "downloads-found-" + modelData
                        text: modelData
                        mono: false
                        small: true
                        selected: page.adoptedFrom === modelData && page.state !== null
                        dimmed: !res || res.state === "idle" || res.state === "error"
                        clickable: !!res && res.state === "done"
                        hint: !res ? "" : res.state === "searching" ? "searching" : res.state === "error" ? "failed" : res.state === "idle" ? "cancelled" : Fmt.plural(res.search.items.length, "release", "releases")
                        onClicked: page.adoptFrom(modelData)
                        Tooltip { text: found.res && found.res.state === "error" && found.res.error ? found.res.error : "" }
                    }
                }
            }
            // anirss's groups as filters, by kind: a click narrows to one
            Column {
                visible: page.groups.length > 0
                width: parent.width
                spacing: theme.space(1.5)
                FacetRow { title: "Release group"; entries: page.facets.groups.slice(0, 14) }
                FacetRow { title: "Quality"; entries: page.facets.quality.slice(0, 14) }
                FacetRow { visible: page.moreFilters; title: "Words"; entries: page.facets.other.slice(0, 30) }
                Button {
                    visible: page.facets.other.length > 0
                    text: page.moreFilters ? "Fewer filters" : "More filters (" + page.facets.other.length + ")"
                    icon: page.moreFilters ? "chevron-up" : "chevron-down"
                    flat: true
                    small: true
                    onClicked: page.moreFilters = !page.moreFilters
                }
            }
            Row {
                visible: !!page.state
                width: parent.width
                spacing: theme.space(2)
                Field {
                    id: narrow
                    objectName: "downloads-narrow"
                    width: Math.min(theme.space(90), parent.width - bestFit.width - parent.spacing)
                    placeholder: "Narrow by a term, or -term to drop"
                    onEdited: function(t) { if (t.trim() !== "") { page.narrow(t); text = "" } }
                }
                Button {
                    id: bestFit
                    objectName: "downloads-best-fit"
                    text: "Best fit"
                    icon: "sparkles"
                    flat: true
                    enabled: !!page.state && page.state.bestFit.available && !page.searching
                    onClicked: page.refine({ type: "best_fit" })
                    Tooltip { text: page.state && page.state.bestFit.available ? "Pins the best release's profile: " + page.state.bestFit.query : "Best fit has nothing to pin here" }
                }
            }
            // The show the search names, when the library has it
            Tile {
                visible: !!page.searchShow && !page.showDismissed && !!page.state
                width: parent.width
                implicitHeight: theme.space(36)
                height: visible ? implicitHeight : 0
                padding: theme.space(4)
                lit: false
                Corner {
                    id: stripPoster
                    width: theme.space(18); height: width * 1.5
                    anchors.verticalCenter: parent.verticalCenter
                    radius: theme.radiusSm
                    color: theme.tileSunken
                    Image {
                        id: stripImage
                        visible: false
                        anchors.fill: parent
                        source: page.searchShow && page.searchShow.poster ? Fmt.fileUrl(page.searchShow.poster) : ""
                        sourceSize: theme.posterDecodeSize
                        fillMode: Image.PreserveAspectCrop
                        asynchronous: true
                    }
                    fillItem: stripImage.status === Image.Ready ? stripImage : null
                }
                Column {
                    anchors.left: stripPoster.right
                    anchors.leftMargin: theme.space(5)
                    anchors.right: stripButtons.left
                    anchors.rightMargin: theme.space(5)
                    anchors.verticalCenter: parent.verticalCenter
                    spacing: theme.space(1.5)
                    Label { text: "In your library"; role: "ledger"; color: theme.textFaint }
                    Label { width: parent.width; text: page.searchShow ? page.searchShow.title : ""; role: "title"; font.pointSize: theme.typeLarge * 0.8; elide: Text.ElideRight }
                    Label {
                        width: parent.width
                        text: {
                            var s = page.searchShow
                            if (!s) return ""
                            var total = s.total_episodes && !s.total_is_estimate ? s.total_episodes : 0
                            var parts = [total ? s.episodes_on_disk + " of " + total + " episodes on disk" : Fmt.plural(s.episodes_on_disk, "episode", "episodes") + " on disk"]
                            if (s.next_airing && s.next_airing.at * 1000 > page.nowMs) parts.push(Fmt.episodeSpan(s.next_airing.episode, s.next_airing.through || 0).toLowerCase() + (s.next_airing.predicted ? " predicted in " : " in ") + Fmt.countdown(s.next_airing.at - page.nowMs / 1000))
                            else if (s.complete) parts.push("complete")
                            return parts.join(", ") + "."
                        }
                        role: "ui"
                        color: theme.textDim
                        elide: Text.ElideRight
                    }
                    Label {
                        width: parent.width
                        text: page.onDiskCount > 0 ? Fmt.plural(page.onDiskCount, "release here is", "releases here are") + " for episodes you already have, marked On disk." : "None of these releases is for an episode you already have."
                        role: "ui"
                        font.pointSize: theme.typeSmall
                        color: theme.textFaint
                        elide: Text.ElideRight
                    }
                }
                Row {
                    id: stripButtons
                    anchors.right: parent.right
                    anchors.verticalCenter: parent.verticalCenter
                    spacing: theme.space(2)
                    Button {
                        objectName: "downloads-select-missing"
                        visible: page.missingPicks.length > 0
                        text: "Tick the " + Fmt.plural(page.missingPicks.length, "missing episode", "missing episodes")
                        icon: "check-check"
                        primary: true
                        onClicked: page.selectMissing()
                        Tooltip { text: "For each episode here the library does not have, the release with the most seeders" }
                    }
                    Button { text: "Open in library"; icon: "layers"; flat: true; onClicked: frame.go("series", { id: page.searchShow.series_id }, page.searchShow.title) }
                    Button { icon: "x"; flat: true; small: true; onClicked: page.showDismissed = true; Tooltip { text: "Not this show" } }
                }
            }
            // The releases as a table at the left, the actions at the right, keeping in view
            Item {
                id: resultsArea
                visible: !!page.state
                width: parent.width
                height: page.items.length > 0 ? Math.max(table.implicitHeight, pane.implicitHeight) : noReleases.height
                ReleaseTable {
                    id: table
                    visible: page.items.length > 0
                    width: parent.width - page.paneWidth - page.gap
                    items: page.items
                    facts: page.facts
                    extras: page.extras
                    extrasByTitle: page.extrasByTitle
                    datesState: page.datesState
                    selected: page.selected
                    highlight: page.hoveredGroup >= 0 && page.groups[page.hoveredGroup] ? page.groups[page.hoveredGroup].members : []
                    sortKey: page.sortKey
                    sortDesc: page.sortDesc
                    nowMs: page.nowMs
                    opacity: page.searching ? 0.5 : 1
                    onToggled: function(item) { page.toggle(item) }
                    onToggledAll: function(on) { page.toggleAll(on) }
                    onSortBy: function(key) { page.sortBy(key) }
                    onDownloadOne: function(item) { page.download([item]) }
                }
                Item {
                    id: noReleases
                    visible: !!page.state && page.items.length === 0 && !page.searching
                    width: parent.width
                    height: visible ? theme.space(48) : 0
                    EmptyState {
                        icon: "search"
                        title: "No releases"
                        body: "Nothing on " + (page.state ? page.state.endpoint : "") + " matches \u201c" + (page.state ? page.state.query : "") + "\u201d. Another endpoint, or fewer words, may find it."
                        Repeater {
                            model: page.state ? page.endpoints.map(function(e) { return e.name }).filter(function(n) { return n !== page.state.endpoint }) : []
                            Button { required property string modelData; text: "Try " + modelData; icon: "search"; flat: true; onClicked: page.searchOn(modelData) }
                        }
                    }
                }
                Tile {
                    id: pane
                    visible: page.items.length > 0
                    x: parent.width - page.paneWidth
                    // Keeps in view while the table scrolls past
                    y: {
                        var top = resultsArea.mapToItem(scroll.contentItem, 0, 0).y
                        return Math.max(0, Math.min(scroll.contentY - top + theme.space(4), resultsArea.height - height))
                    }
                    width: page.paneWidth
                    implicitHeight: paneBody.implicitHeight + padding * 2
                    height: implicitHeight
                    padding: theme.space(4)
                    lit: false
                    Column {
                        id: paneBody
                        width: parent.width
                        spacing: theme.space(2.5)
                        // What is ticked
                        Label { text: page.selectedCount ? Fmt.plural(page.selectedCount, "release", "releases") + " ticked" : "Nothing ticked"; role: "title"; font.pointSize: theme.typeNormal }
                        Label {
                            width: parent.width
                            text: page.selectedCount ? (page.selectedBytes > 0 ? Fmt.bytes(page.selectedBytes) + " together." : "") : "Tick releases in the table, or download them all."
                            role: "ui"
                            font.pointSize: theme.typeSmall
                            color: theme.textDim
                            wrapMode: Text.Wrap
                        }
                        Row {
                            spacing: theme.space(2)
                            Button { text: "Tick all"; flat: true; small: true; enabled: page.items.length > 0 && page.selectedCount < page.items.length; onClicked: page.toggleAll(true) }
                            Button { text: "Clear"; flat: true; small: true; enabled: page.selectedCount > 0; onClicked: page.toggleAll(false) }
                        }
                        Button {
                            objectName: "downloads-download"
                            width: parent.width
                            text: page.acting ? "Working" : page.selectedCount > 0 ? "Download " + Fmt.plural(page.selectedCount, "release", "releases") : "Download all " + page.items.length
                            icon: "download"
                            primary: true
                            enabled: !!page.state && page.items.length > 0 && !page.acting && !page.searching
                            onClicked: page.download(page.selectedCount > 0 ? page.selectedItems() : page.items)
                        }
                        Button {
                            width: parent.width
                            text: "Download as a movie"
                            icon: "clapperboard"
                            flat: true
                            enabled: page.selectedCount === 1 && !page.acting
                            onClicked: page.movie()
                            Tooltip { text: "One ticked release, straight into the movie folder" }
                        }
                        Rectangle { width: parent.width; height: 1; color: theme.line }
                        // Keeping the show coming
                        Label { text: "Subscribe"; role: "title"; font.pointSize: theme.typeNormal }
                        Label {
                            width: parent.width
                            text: "New releases that match this search download as they appear, into the folder named here."
                            role: "ui"
                            font.pointSize: theme.typeSmall
                            color: theme.textDim
                            wrapMode: Text.Wrap
                        }
                        Field {
                            objectName: "downloads-name"
                            width: parent.width
                            placeholder: page.searchShow ? page.searchShow.title : "anirss picks one from the first release"
                            text: page.nameText
                            onEdited: function(t) { page.nameText = t }
                        }
                        Button {
                            objectName: "downloads-subscribe"
                            width: parent.width
                            text: page.acting ? "Working" : "Subscribe to this search"
                            icon: "rss"
                            enabled: !!page.state && page.items.length > 0 && !page.acting && !page.searching
                            onClicked: page.subscribe()
                        }
                        Button {
                            width: parent.width
                            text: page.preview ? "Show the rule again" : "Show the rule it makes"
                            icon: "eye"
                            flat: true
                            small: true
                            enabled: !!page.state && page.items.length > 0 && !page.searching
                            onClicked: page.previewSubscription()
                        }
                        Column {
                            visible: !!page.preview
                            width: parent.width
                            spacing: theme.space(1)
                            Label { width: parent.width; text: page.preview ? "From " + page.feedHost(page.preview.feed_url) + (page.preview.endpoint_name ? " (" + page.preview.endpoint_name + ")" : "") : ""; role: "ledger"; color: theme.textDim; elide: Text.ElideRight }
                            Label { visible: !!(page.preview && page.preview.must_contain); width: parent.width; text: page.preview ? "Must contain " + page.preview.must_contain : ""; role: "ledger"; color: theme.textDim; wrapMode: Text.Wrap }
                            Label { visible: !!(page.preview && page.preview.must_not_contain); width: parent.width; text: page.preview ? "Must not contain " + page.preview.must_not_contain : ""; role: "ledger"; color: theme.textDim; wrapMode: Text.Wrap }
                        }
                        Label { visible: page.actionLine !== ""; width: parent.width; text: page.actionLine; role: "ui"; font.pointSize: theme.typeSmall; color: theme.caughtUp; wrapMode: Text.Wrap }
                        Label { visible: page.actionError !== ""; width: parent.width; text: page.actionError; role: "ui"; font.pointSize: theme.typeSmall; color: theme.red; wrapMode: Text.Wrap }
                    }
                }
            }
        }

        // ---- Nothing to search with: anirss is not here
        Item {
            visible: !!page.status && !page.anirssReady && !page.connecting && page.state === null && page.detail === null
            width: parent.width
            height: visible ? theme.space(50) : 0
            EmptyState {
                icon: page.anirss && page.anirss.state === "Missing" ? "circle-alert" : "rss"
                title: page.anirss && page.anirss.state === "Missing" ? "anirss is not installed" : "anirss is not running"
                body: page.anirss && page.anirss.message ? page.anirss.message : "Searches, subscriptions and downloads run through anirss."
                Button { text: "Try again"; icon: "refresh-cw"; onClicked: page.connect() }
                Button { text: "Settings"; icon: "settings"; flat: true; onClicked: frame.go("settings", { tab: "downloads" }) }
            }
        }

        // ---- A show to get: its providers, their releases, the plan
        Loader {
            active: page.getShow !== null
            visible: active
            width: parent.width
            height: active && item ? item.implicitHeight : 0
            sourceComponent: GetShow {
                objectName: "downloads-get"
                width: scroll.innerWidth
                show: page.getShow
                torrents: page.torrentsOfShow(page.getShow)
                busyOf: function(hash) { return page.busyOf(hash) }
                firstEpisode: function(t) { return page.firstEpisode(t) }
                endpoints: page.endpoints
                results: page.getResults
                active: page.getActive
                sourceOn: function(name) { var m = frame.downloadsSources; return page.sourceOn(name) }
                acting: page.getActing
                actionLine: page.getLine
                actionError: page.getError
                nowMs: page.nowMs
                onBack: page.closeGet()
                onFetch: function(name) { page.fetchGet(name) }
                onSourceToggled: function(name, on) { page.setSource(name, on); if (on) page.fetchGet(name) }
                onCheckSources: page.checkSources()
                onOpenList: function(name) { page.listFromGet(name) }
                onGetAndFollow: function(plan) { page.getAndFollow(plan) }
                onControl: function(hash, action) { page.control(hash, action) }
                onCancelTorrent: function(anchor, t) { page.askCancelTorrent(anchor, t) }
                onControlAll: function(hashes, action) { page.controlMany(hashes, action) }
                onCancelAll: function(anchor, list) { page.askCancelTorrents(anchor, list) }
            }
        }

        // ---- The show, inside the page
        Loader {
            active: page.detail !== null
            visible: active
            width: parent.width
            height: active && item ? item.implicitHeight : 0
            sourceComponent: ShowDetail {
                width: scroll.innerWidth
                rule: page.detailRule
                show: page.detailShow
                title: page.detailShow ? page.detailShow.title : page.detail.title
                folder: page.detailRule ? page.folderOf(page.detailRule.save_path) : page.detail.folder
                torrents: page.detailTorrents
                feed: page.feed
                ruleView: page.ruleView
                source: page.detailRule ? page.endpointFor(page.detailRule.feed_url) : ""
                repairing: page.repairJob >= 0
                repairLabel: page.repairLabel
                onRepair: page.repairRule()
                watching: page.watching
                justSubscribed: page.subscribedAt > 0 && page.detail.rule !== "" && (Date.now() - page.subscribedAt) < 600000
                clientIndex: page.clientIndex
                nowMs: page.nowMs
                removing: (!!page.detailRule && page.isRemoving("r:" + page.detailRule.name)) || page.isRemoving("g:" + page.detail.key)
                onBack: page.closeDetail()
                onUnsubscribe: function(anchor) { page.askUnsubscribe(anchor, page.detailRule) }
                onRemoveAll: function(anchor) { page.askRemoveDetail(anchor) }
                onRemoveTorrent: function(torrent, anchor) { page.askRemoveTorrent(anchor, torrent) }
                onControl: function(hash, action) { page.control(hash, action) }
                onDownloadEntry: function(entry) { page.downloadEntry(entry) }
                onRefreshFeed: page.readFeed()
            }
        }

        // ---- The lenses: four ways into what there is to get
        Item {
            visible: page.home
            width: parent.width
            height: visible ? theme.space(14) : 0
            Seg {
                id: lensSeg
                objectName: "downloads-lens"
                anchors.left: parent.left
                anchors.bottom: parent.bottom
                options: [
                    { text: "Gaps in your library  " + page.gapCards.length },
                    { text: "Airing  " + page.airingCards.length },
                    { text: "Following  " + page.rules.length },
                    { text: "Downloaded  " + page.showGroups.length }
                ]
                index: ["gaps", "airing", "following", "downloaded"].indexOf(page.lens)
                onPicked: function(i) { page.lens = ["gaps", "airing", "following", "downloaded"][i] }
            }
        }

        // ---- Gaps: aired episodes the library does not have, a show a row
        Column {
            visible: page.home && page.lens === "gaps"
            width: parent.width
            spacing: theme.space(4)
            BandHead {
                title: "Gaps in your library"
                count: page.gapCards.length
                line: page.gapCards.length ? "Aired episodes that are not on disk. Open a show to see who has them." : "Nothing is missing: every aired episode of every show is on disk."
            }
            Flow {
                width: parent.width
                spacing: page.tileGap
                Repeater {
                    model: page.gapCards
                    Corner {
                        id: gap
                        required property var modelData
                        objectName: "gap-" + modelData.id
                        readonly property real cols: Math.max(1, Math.floor((scroll.innerWidth + page.tileGap) / (theme.space(105) + page.tileGap)))
                        width: Math.floor((scroll.innerWidth - page.tileGap * (cols - 1)) / cols)
                        height: theme.space(36)
                        radius: theme.radiusTile
                        smoothing: theme.cornerSmoothing
                        color: gapMouse.containsMouse ? theme.glassFloatHover : theme.glassFloat
                        MouseArea { id: gapMouse; anchors.fill: parent; hoverEnabled: true; cursorShape: Qt.PointingHandCursor; onClicked: page.openGet(page.showFromCard(gap.modelData)) }
                        Corner {
                            id: gapPoster
                            x: theme.space(3); y: theme.space(3)
                            width: (parent.height - theme.space(6)) / 1.45; height: parent.height - theme.space(6)
                            radius: theme.radiusMd
                            smoothing: theme.cornerSmoothing
                            color: theme.tileSunken
                            Image {
                                id: gapImage
                                visible: false
                                anchors.fill: parent
                                source: gap.modelData.poster ? Fmt.fileUrl(gap.modelData.poster) : ""
                                sourceSize: Qt.size(160, 232)
                                fillMode: Image.PreserveAspectCrop
                                asynchronous: true
                            }
                            fillItem: gapImage.status === Image.Ready ? gapImage : null
                        }
                        Column {
                            anchors.left: gapPoster.right
                            anchors.leftMargin: theme.space(5)
                            anchors.right: parent.right
                            anchors.rightMargin: theme.space(5)
                            anchors.verticalCenter: parent.verticalCenter
                            spacing: theme.space(2)
                            Label { width: parent.width; text: gap.modelData.title; role: "display"; font.pointSize: theme.typeHeading; font.variableAxes: ({ "wdth": 70, "wght": 420 }); wrapMode: Text.Wrap; maximumLineCount: 2; elide: Text.ElideRight; lineHeight: 0.92 }
                            Label { width: parent.width; text: page.gapWords(gap.modelData); role: "ui"; color: String(gap.modelData.cubes).indexOf("m") >= 0 ? theme.yellow : theme.textDim; wrapMode: Text.Wrap; maximumLineCount: 2; elide: Text.ElideRight }
                            EpisodeCubes { width: parent.width; height: implicitHeight; codes: gap.modelData.cubes || ""; cubeSize: theme.space(2.5); minSize: 2; maxRows: 2 }
                        }
                    }
                }
            }
        }

        // ---- Airing: the library's airing shows, posters with their cubes
        Column {
            visible: page.home && page.lens === "airing"
            width: parent.width
            spacing: theme.space(4)
            BandHead {
                title: "Airing"
                count: page.airingCards.length
                line: "Shows in your library with episodes still to come. Open one to follow it."
            }
            Flow {
                width: parent.width
                spacing: page.tileGap
                Repeater {
                    model: page.airingCards
                    ShowCard {
                        required property var modelData
                        width: page.cardWidth
                        height: implicitHeight
                        poster: modelData.poster || ""
                        title: modelData.title
                        tag: { var r = page.ruleForSeries(modelData.id); return r ? "Following" : "" }
                        tagDot: theme.green
                        ledger: modelData.next_airing ? Fmt.episodeSpan(modelData.next_airing.episode, modelData.next_airing.through || 0) + (modelData.next_airing.predicted ? " predicted in " : " in ") + Fmt.countdown(modelData.next_airing.at - page.nowMs / 1000) : "Airing"
                        cubes: modelData.cubes || ""
                        onOpened: page.openGet(page.showFromCard(modelData))
                    }
                }
            }
        }

        // ---- Following: the subscriptions, in sections by where the show stands
        Column {
            visible: page.home && page.lens === "following"
            width: parent.width
            spacing: theme.space(4)
            BandHead {
                title: "Following"
                count: page.rules.length
                line: page.rulesError !== "" ? page.rulesError : page.rulesLoaded ? (page.rules.length ? "Feeds anirss watches, as the shows they fill." : "None yet.") : (page.anirssReady ? "Reading." : "")
                alarm: page.rulesError !== ""
                view: frame.downloadsSubView
                viewName: "downloads-sub-view"
                onViewPicked: function(v) { frame.downloadsSubView = v }
            }
            Repeater {
                model: page.subSections
                Column {
                    id: subSection
                    required property var modelData
                    width: parent.width
                    spacing: theme.space(2)
                    SectionHead {
                        title: subSection.modelData.title
                        dot: subSection.modelData.dot
                        count: subSection.modelData.items.length
                        line: subSection.modelData.line
                        Button {
                            id: allButton
                            objectName: "downloads-unsubscribe-complete"
                            visible: subSection.modelData.key === "complete"
                            text: page.queued > 1 ? "Removing, " + page.queued + " to go" : "Unsubscribe all " + subSection.modelData.items.length
                            icon: "rss"
                            small: true
                            enabled: page.queued === 0
                            onClicked: page.askUnsubscribeComplete(allButton)
                        }
                    }
                    Flow {
                        visible: frame.downloadsSubView === "grid"
                        width: parent.width
                        spacing: page.tileGap
                        Repeater {
                            model: frame.downloadsSubView === "grid" ? subSection.modelData.items : []
                            ShowCard {
                                required property var modelData
                                width: page.cardWidth
                                height: implicitHeight
                                compact: !modelData.show
                                poster: modelData.show && modelData.show.poster ? modelData.show.poster : ""
                                title: page.titleOf(modelData)
                                tag: page.ruleTag(modelData)
                                tagDot: page.stateDot(page.subState(modelData))
                                ledger: page.ruleLedger(modelData)
                                progress: page.ruleProgress(modelData)
                                cubes: modelData.show && modelData.show.cubes ? modelData.show.cubes : ""
                                actionText: "Unsubscribe"
                                actionIcon: "rss"
                                busy: page.isRemoving("r:" + modelData.name)
                                onOpened: page.openRule(modelData)
                                onAction: function(anchor) { page.askUnsubscribe(anchor, modelData) }
                            }
                        }
                    }
                    Column {
                        visible: frame.downloadsSubView === "list"
                        width: parent.width
                        Repeater {
                            model: frame.downloadsSubView === "list" ? subSection.modelData.items : []
                            ShowRow {
                                required property var modelData
                                poster: modelData.show && modelData.show.poster ? modelData.show.poster : ""
                                title: page.titleOf(modelData)
                                chips: {
                                    var c = []
                                    var tag = page.ruleTag(modelData)
                                    if (tag !== "") c.push({ text: tag, dot: page.stateDot(page.subState(modelData)) })
                                    if (modelData.rule_enabled === false && tag !== "Paused") c.push({ text: "Paused", dot: theme.textFaint })
                                    return c
                                }
                                facts: page.ruleLedger(modelData) + (modelData.show ? ", " + Fmt.plural(modelData.torrent_count || 0, "torrent", "torrents") : "")
                                folder: modelData.save_path
                                progress: page.ruleProgress(modelData)
                                cubes: modelData.show && modelData.show.cubes ? modelData.show.cubes : ""
                                actionText: "Unsubscribe"
                                actionIcon: "rss"
                                busy: page.isRemoving("r:" + modelData.name)
                                onOpened: page.openRule(modelData)
                                onAction: function(anchor) { page.askUnsubscribe(anchor, modelData) }
                            }
                        }
                    }
                }
            }
            Item {
                visible: page.rulesLoaded && page.rules.length === 0 && page.rulesError === ""
                width: parent.width
                height: visible ? theme.space(30) : 0
                EmptyState { icon: "rss"; title: "No subscriptions"; body: "Search above and subscribe to keep a show coming." }
            }
        }

        // ---- Downloaded, in sections by what the client is doing with each show
        Column {
            visible: page.home && page.lens === "downloaded"
            width: parent.width
            spacing: theme.space(4)
            BandHead {
                title: "Downloaded"
                count: page.downloadedCount
                line: !page.torrentOn ? (page.torrent && page.torrent.message ? page.torrent.message : page.clientName + " is not connected.")
                      : page.torrentsError !== "" ? page.torrentsError
                      : page.torrentsRead ? Fmt.plural(page.torrents.length, "torrent", "torrents") + ", " + Fmt.bytes(page.downloadedSize) + " in " + page.clientName + "." : "Reading."
                alarm: !page.torrentOn || page.torrentsError !== ""
                view: frame.downloadsDlView
                viewName: "downloads-dl-view"
                onViewPicked: function(v) { frame.downloadsDlView = v }
            }
            Repeater {
                model: page.dlSections
                Column {
                    id: dlSection
                    required property var modelData
                    width: parent.width
                    spacing: theme.space(2)
                    SectionHead {
                        title: dlSection.modelData.title
                        dot: dlSection.modelData.dot
                        count: dlSection.modelData.items.length
                        line: dlSection.modelData.line
                        Button {
                            id: brokenButton
                            objectName: "downloads-remove-broken"
                            visible: dlSection.modelData.key === "broken"
                            text: page.queued > 1 ? "Removing, " + page.queued + " to go" : "Remove all " + dlSection.modelData.items.length
                            icon: "x"
                            small: true
                            enabled: page.queued === 0
                            onClicked: page.askRemoveBroken(brokenButton, dlSection.modelData.items)
                        }
                    }
                    GroupList { groups: dlSection.modelData.items; state: dlSection.modelData.key; grid: frame.downloadsDlView === "grid" }
                }
            }
            Item {
                visible: page.torrentsRead && page.showGroups.length === 0
                width: parent.width
                height: visible ? theme.space(30) : 0
                EmptyState {
                    icon: page.torrentOn ? "download" : "circle-alert"
                    title: page.torrentOn ? "Nothing downloaded" : page.clientName + " is not connected"
                    body: page.torrentOn ? "What you download or subscribe to shows up here, a show at a time." : (page.torrent && page.torrent.message ? page.torrent.message : "Check its address and password in Settings.")
                    Button { visible: !page.torrentOn; text: "Settings"; icon: "settings"; onClicked: frame.go("settings", { tab: "downloads" }) }
                }
            }
        }
    }

    // The confirm before AniBeam starts a source's service; over everything on the page
    StartSourceSheet {
        id: startSheet
        z: 20
        onStartRequested: function(name) { page.startService(name) }
    }

    // One kind of filter: its name, then a chip per group with its count. Hovering a chip
    // lights its releases in the table; a click narrows to them.
    component FacetRow: Row {
        id: facet
        property string title: ""
        property var entries: []
        visible: entries.length > 0
        width: parent ? parent.width : 0
        spacing: theme.space(3)
        Label { id: facetTitle; width: theme.space(26); anchors.verticalCenter: parent.verticalCenter; text: facet.title; role: "ledger"; color: theme.textFaint }
        Flow {
            width: facet.width - facetTitle.width - facet.spacing
            spacing: theme.space(1.5)
            Repeater {
                model: facet.entries
                Chip {
                    required property var modelData
                    text: modelData.label + "  " + modelData.count
                    small: true
                    maxLabelWidth: theme.space(70)
                    clickable: !page.searching
                    selected: page.hoveredGroup === modelData.index
                    onClicked: page.refine({ type: "tokens", tokens: page.groups[modelData.index].tokens })
                    HoverHandler { onHoveredChanged: page.hoveredGroup = hovered ? modelData.index : (page.hoveredGroup === modelData.index ? -1 : page.hoveredGroup) }
                }
            }
        }
    }

    // A band's head: the title in the display cut, its count, one line, and the grid or
    // list switch at the right when the band has one
    component BandHead: Item {
        id: band
        property string title: ""
        property int count: 0
        property string line: ""
        property bool alarm: false
        property string view: ""
        property string viewName: ""
        signal viewPicked(string view)
        width: parent ? parent.width : 0
        height: theme.space(12)
        Row {
            anchors.left: parent.left
            anchors.right: viewSeg.visible ? viewSeg.left : parent.right
            anchors.rightMargin: theme.space(4)
            anchors.bottom: parent.bottom
            anchors.bottomMargin: theme.space(2)
            spacing: theme.space(3)
            Label { id: bandTitle; text: band.title; role: "display"; font.pointSize: theme.typeHeading; font.variableAxes: ({ "wdth": 70, "wght": 420 }) }
            Label { anchors.baseline: bandTitle.baseline; visible: band.count > 0; text: band.count; role: "ledger"; color: theme.textFaint }
            Label { anchors.baseline: bandTitle.baseline; text: band.line; role: "ui"; color: band.alarm ? theme.red : theme.textDim; elide: Text.ElideRight; width: Math.max(0, Math.min(implicitWidth, parent.width - bandTitle.width - theme.space(20))) }
        }
        Seg {
            id: viewSeg
            objectName: band.viewName
            visible: band.view !== ""
            anchors.right: parent.right
            anchors.bottom: parent.bottom
            anchors.bottomMargin: theme.space(1)
            small: true
            options: [{ text: "Grid", icon: "layout-grid" }, { text: "List", icon: "list" }]
            index: band.view === "list" ? 1 : 0
            onPicked: function(i) { band.viewPicked(i === 1 ? "list" : "grid") }
        }
    }

    // A section's head inside a band: a dot in the section's colour, the title, the count,
    // one dim line, and anything handed to it at the right
    component SectionHead: Item {
        id: sec
        property string title: ""
        property color dot: theme.textFaint
        property int count: 0
        property string line: ""
        default property alias extra: extraRow.data
        width: parent ? parent.width : 0
        height: theme.space(10)
        Rectangle { id: secDot; anchors.verticalCenter: secTitle.verticalCenter; width: theme.space(2.5); height: width; radius: width / 2; color: sec.dot }
        Label { id: secTitle; anchors.left: secDot.right; anchors.leftMargin: theme.space(2); anchors.verticalCenter: parent.verticalCenter; text: sec.title; role: "title"; font.pointSize: theme.typeNormal * 1.15 }
        Label { id: secCount; anchors.left: secTitle.right; anchors.leftMargin: theme.space(2); anchors.baseline: secTitle.baseline; text: sec.count; role: "ledger"; color: theme.textFaint }
        Label {
            anchors.left: secCount.right
            anchors.leftMargin: theme.space(3)
            anchors.right: extraRow.left
            anchors.rightMargin: theme.space(3)
            anchors.baseline: secTitle.baseline
            text: sec.line
            role: "ui"
            color: theme.textDim
            elide: Text.ElideRight
        }
        Row { id: extraRow; anchors.right: parent.right; anchors.verticalCenter: parent.verticalCenter; spacing: theme.space(2) }
    }

    // A section of downloaded shows, as posters or as rows
    component GroupList: Item {
        id: gl
        property var groups: []
        property string state: ""
        property bool grid: true
        width: parent ? parent.width : 0
        height: grid ? glFlow.implicitHeight : glColumn.implicitHeight
        Flow {
            id: glFlow
            visible: gl.grid
            width: parent.width
            spacing: page.tileGap
            Repeater {
                model: gl.grid ? gl.groups : []
                ShowCard {
                    required property var modelData
                    width: page.cardWidth
                    height: implicitHeight
                    compact: !modelData.show
                    glyph: "folder"
                    poster: modelData.show && modelData.show.poster ? modelData.show.poster : ""
                    title: modelData.title
                    tag: page.groupWord(gl.state)
                    tagDot: page.stateDot(gl.state)
                    ledger: page.groupLedger(modelData)
                    progress: gl.state === "downloading" ? modelData.have / Math.max(1, modelData.size) : -1
                    progressCode: "t"
                    cubes: gl.state !== "downloading" && modelData.show && modelData.show.cubes ? modelData.show.cubes : ""
                    actionText: "Remove"
                    actionIcon: "x"
                    busy: page.isRemoving("g:" + modelData.key)
                    onOpened: page.openGroup(modelData)
                    onAction: function(anchor) { page.askRemoveGroup(anchor, modelData) }
                }
            }
        }
        Column {
            id: glColumn
            visible: !gl.grid
            width: parent.width
            Repeater {
                model: gl.grid ? [] : gl.groups
                ShowRow {
                    required property var modelData
                    glyph: "folder"
                    poster: modelData.show && modelData.show.poster ? modelData.show.poster : ""
                    title: modelData.title
                    chips: {
                        var c = [{ text: page.groupWord(gl.state), dot: page.stateDot(gl.state) }]
                        if (modelData.rule) c.push({ text: "Subscribed", dot: theme.accent })
                        if (modelData.show && modelData.show.complete) c.push({ text: "Complete", dot: theme.red })
                        return c
                    }
                    facts: page.groupLedger(modelData) + (gl.state === "downloading" ? ", " + Fmt.plural(modelData.torrents.length, "torrent", "torrents") : modelData.upspeed > 0 ? ", " + Fmt.bytes(modelData.upspeed) + "/s up" : "")
                    trouble: modelData.broken > 0 && modelData.broken < modelData.torrents.length ? modelData.broken + " with files missing" : ""
                    folder: modelData.folder
                    progress: gl.state === "downloading" ? modelData.have / Math.max(1, modelData.size) : -1
                    progressCode: "t"
                    cubes: gl.state !== "downloading" && modelData.show && modelData.show.cubes ? modelData.show.cubes : ""
                    actionText: "Remove"
                    actionIcon: "x"
                    busy: page.isRemoving("g:" + modelData.key)
                    onOpened: page.openGroup(modelData)
                    onAction: function(anchor) { page.askRemoveGroup(anchor, modelData) }
                }
            }
        }
    }
}
