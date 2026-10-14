// Downloads: anirss's front, built around what you subscribe to and what you have, with
// the transfers in their place: up front only while something is moving. The head names
// the page and says how anirss and the torrent client stand (each a chip that opens the
// Downloads settings). Under it the search, then, while a search is up, its query line,
// groups, Narrow and Best fit, the releases and the actions pane, as before.
//
// With no search up the page is three bands. Downloading, only while a torrent is moving:
// the moving ones, speeds in the band's line. Subscriptions: every rule anirss registered
// as a poster of the library series its folder fills, the complete ones first (finished
// airing, every episode on disk: nothing left for the rule to do) under a line that says
// how many there are and offers to remove their RSS in one go. Downloaded: every folder
// the client holds torrents for, one row per show with its poster, Remove torrents and
// Remove files. A poster or a row opens the show inside the page (ShowDetail): its
// episodes, its feed as the feed reads now, its torrents. Every removal asks twice in
// place (ConfirmButton).
//
// The subscriptions are the frame's (it reads them for the rail's badge); this page asks
// for a fresh read on open. The transfers are the one place in the app that polls: every
// two seconds while something is downloading and the page is up, every fifteen while
// nothing is. There is no event for a byte arriving.
import QtQuick
import com.marcusrosado.AniBeam

Item {
    id: page
    objectName: "downloads-page"
    property var props: ({})
    property string title: "Downloads"
    property alias scrollY: scroll.contentY
    // Escape closes the show, then the search
    function escapePressed() {
        if (page.detail !== null) { page.closeDetail(); return true }
        if (page.state !== null || page.searchError !== "") { page.state = null; page.history = []; page.searchError = ""; page.notice = ""; return true }
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
    property real connectJob: -1
    property bool connecting: false
    property int endpointIndex: 0

    // ---- The search: the state anirss asks the client to hold, and the trail for Back
    property var state: null
    property var history: []
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
    readonly property int shown: Math.min(items.length, 200)

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
    property int subFilter: 0
    property bool allGroups: false
    // Rule names whose removal is running, and the jobs that carry them
    property var removing: ({})
    property var removeJobs: ({})
    // Group keys whose torrents are going
    property var clearing: ({})
    // Remove RSS for every complete subscription: one at a time, the next on each answer
    property var bulk: []
    property string bulkCurrent: ""
    property real nowMs: Date.now()
    Timer { interval: 30000; repeat: true; running: page.visible && frame.armedConfirm === null; onTriggered: page.nowMs = Date.now() }
    // A removed rule stays dimmed until the read after it no longer lists it
    onRulesChanged: {
        var keep = {}
        for (var i = 0; i < rules.length; i++) if (page.removing[rules[i].name]) keep[rules[i].name] = true
        page.removing = keep
    }

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
    function runSearch() {
        var q = search.text.trim()
        if (q === "") { search.focusInput(); return }
        var endpoint = page.endpoints.length > page.endpointIndex ? page.endpoints[page.endpointIndex].name : ""
        var r = Door.searchReleases(q, endpoint)
        if (r.error) { page.searchError = r.error.message; return }
        page.searchJob = r.reply.job
        page.searching = true
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
    function refine(op) {
        if (!page.state || page.searching) return
        var r = Door.refineReleases(page.state.query, page.state.endpoint, page.state.items, op)
        if (r.error) { page.searchError = r.error.message; return }
        page.searchJob = r.reply.job
        page.searching = true
        page.searchError = ""
    }
    function narrow(text) {
        var t = text.trim()
        if (t === "") return
        if (t.charAt(0) === "-" && t.length > 1) refine({ type: "exclude", term: t.slice(1).trim() })
        else refine({ type: "term", term: t })
    }
    function adopt(s) {
        page.state = { query: s.query, endpoint: s.endpoint, items: s.items || [], groups: s.groups || [], bestFit: s.best_fit || { available: false }, clientSide: !!s.client_side }
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
    function subscribe() { if (page.state) act(Door.subscribe(page.state.query, page.state.endpoint, page.state.items, page.nameText)) }
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
    function folderOf(path) { return String(path || "").replace(/\/+$/, "") }
    function lastName(path) { var f = folderOf(path); var i = f.lastIndexOf("/"); return i >= 0 ? f.slice(i + 1) : f }
    function torrentKey(t) { return t.series_id !== null && t.series_id !== undefined ? "s" + t.series_id : "p" + folderOf(t.save_path) }
    function ruleKey(r) { return r.show ? "s" + r.show.series_id : "p" + folderOf(r.save_path) }
    function titleOf(r) { return r.show ? r.show.title : r.name }
    readonly property var ruleByKey: { var m = {}; for (var i = 0; i < rules.length; i++) m[ruleKey(rules[i])] = rules[i]; return m }

    // Active is what is moving or waiting to: not done, not stopped, not broken
    function rank(t) {
        var s = String(t.state || "")
        if (s === "error" || s === "missingFiles") return 3
        if (s.indexOf("paused") === 0 || s.indexOf("stopped") === 0) return (t.progress || 0) >= 1 ? 1 : 2
        return (t.progress || 0) >= 1 ? 1 : 0
    }
    readonly property var activeTorrents: torrents.filter(function(t) { return page.rank(t) === 0 })
        .sort(function(a, b) { return (b.dlspeed || 0) - (a.dlspeed || 0) || (b.added_on || 0) - (a.added_on || 0) })
    readonly property var clientIndex: {
        var byHash = {}, byName = {}
        for (var i = 0; i < torrents.length; i++) { byHash[String(torrents[i].hash).toLowerCase()] = torrents[i]; byName[torrents[i].name] = torrents[i] }
        return { byHash: byHash, byName: byName }
    }
    // One group per show folder, the moving ones first, then the newest
    readonly property var showGroups: {
        var m = {}, order = []
        for (var i = 0; i < torrents.length; i++) {
            var t = torrents[i], k = page.torrentKey(t), g = m[k]
            if (!g) {
                var show = t.series_id !== null && t.series_id !== undefined ? (page.shows[t.series_id] || null) : null
                g = m[k] = { key: k, show: show, title: show ? show.title : page.lastName(t.save_path), folder: page.folderOf(t.save_path),
                             rule: page.ruleByKey[k] || null, torrents: [], size: 0, active: 0, done: 0, broken: 0, upspeed: 0, latest: 0 }
                order.push(g)
            }
            g.torrents.push(t)
            g.size += t.size || 0
            var r = page.rank(t)
            if (r === 0) g.active++
            if (r === 3) g.broken++
            if ((t.progress || 0) >= 1) g.done++
            g.upspeed += t.upspeed || 0
            g.latest = Math.max(g.latest, t.completion_on || 0, t.added_on || 0)
        }
        order.sort(function(a, b) { return (b.active > 0) - (a.active > 0) || b.latest - a.latest })
        return order
    }
    readonly property int groupCap: 40
    readonly property real downloadedSize: { var n = 0; for (var i = 0; i < torrents.length; i++) n += torrents[i].size || 0; return n }

    // The subscriptions: complete first, then airing (soonest first), then the rest
    function ruleRank(r) {
        if (r.show && r.show.complete) return 0
        if (r.show && (r.show.next_airing || r.show.status === "Releasing")) return 1
        return 2
    }
    readonly property var completeRules: rules.filter(function(r) { return r.show && r.show.complete })
    readonly property var airingRules: rules.filter(function(r) { return page.ruleRank(r) === 1 })
    readonly property var shownRules: (subFilter === 1 ? completeRules : subFilter === 2 ? airingRules : rules).slice().sort(function(a, b) {
        var d = page.ruleRank(a) - page.ruleRank(b)
        if (d !== 0) return d
        if (page.ruleRank(a) === 1) {
            var an = a.show.next_airing ? a.show.next_airing.at : 1e12, bn = b.show.next_airing ? b.show.next_airing.at : 1e12
            if (an !== bn) return an - bn
        }
        return page.titleOf(a).localeCompare(page.titleOf(b))
    })

    // ---- The show inside the page
    function openRule(r) {
        page.detail = { key: page.ruleKey(r), rule: r.name, folder: page.folderOf(r.save_path), seriesId: r.show ? r.show.series_id : -1, title: page.titleOf(r) }
        page.readFeed()
        scroll.contentY = 0
    }
    function openGroup(g) {
        page.detail = { key: g.key, rule: g.rule ? g.rule.name : "", folder: g.folder, seriesId: g.show ? g.show.series_id : -1, title: g.title }
        page.readFeed()
        scroll.contentY = 0
    }
    function closeDetail() { page.detail = null; page.feed = null }
    readonly property var detailRule: {
        if (!detail || !detail.rule) return null
        for (var i = 0; i < rules.length; i++) if (rules[i].name === detail.rule) return rules[i]
        return null
    }
    readonly property var detailShow: detailRule && detailRule.show ? detailRule.show : (detail && detail.seriesId >= 0 ? (shows[detail.seriesId] || null) : null)
    readonly property var detailTorrents: {
        if (!detail) return []
        var ruleFolder = detailRule ? page.folderOf(detailRule.save_path) : ""
        return torrents.filter(function(t) { return page.torrentKey(t) === page.detail.key || (ruleFolder !== "" && page.folderOf(t.save_path) === ruleFolder) })
            .sort(function(a, b) { return String(a.name).localeCompare(String(b.name)) })
    }
    function readFeed() {
        var r = page.detailRule
        if (!r || !r.feed_url) { page.feed = null; return }
        page.feed = { url: r.feed_url, entries: page.feed && page.feed.url === r.feed_url ? page.feed.entries : [], loading: true, error: "" }
        var res = Door.readFeed(r.feed_url)
        if (res.error) page.feed = { url: r.feed_url, entries: [], loading: false, error: page.errorWords(res.error) }
    }
    function downloadEntry(e) {
        var name = page.detailRule ? page.detailRule.name : page.lastName(page.detail.folder)
        page.act(Door.downloadReleases([{ title: e.title, link: e.link, seeders: e.seeders, leechers: e.leechers, downloads: e.downloads, size: e.size, page: e.page }], name))
        frame.toast("Sent " + e.title + " to " + page.clientName)
    }

    // ---- Removals
    function withKey(obj, key, on) { var o = JSON.parse(JSON.stringify(obj)); if (on) o[key] = true; else delete o[key]; return o }
    function removeRss(r) {
        var res = Door.removeSubscription(r.name, true, false)
        if (res.error) { frame.toast(page.errorWords(res.error)); return false }
        page.removing = page.withKey(page.removing, r.name, true)
        var j = JSON.parse(JSON.stringify(page.removeJobs)); j[String(res.reply.job)] = r.name; page.removeJobs = j
        return true
    }
    function removeAllComplete() {
        page.bulk = page.completeRules.map(function(r) { return r.name })
        page.nextBulk()
    }
    function nextBulk() {
        if (!page.bulk.length) { page.bulkCurrent = ""; return }
        var b = page.bulk.slice(), name = b.shift()
        page.bulk = b
        page.bulkCurrent = name
        var rule = null
        for (var i = 0; i < page.rules.length; i++) if (page.rules[i].name === name) rule = page.rules[i]
        if (!rule || !page.removeRss(rule)) page.nextBulk()
    }
    function removalEnded(job, name) {
        var j = JSON.parse(JSON.stringify(page.removeJobs)); delete j[String(job)]; page.removeJobs = j
        if (page.detail && page.detail.rule === name) page.closeDetail()
        if (name === page.bulkCurrent) page.nextBulk()
    }
    function clearTorrents(key, list, files) {
        var hashes = list.map(function(t) { return t.hash })
        if (!hashes.length) return
        var res = Door.controlTorrents(hashes, "Delete", files === true)
        if (res.error) { frame.toast(page.errorWords(res.error)); return }
        page.clearing = page.withKey(page.clearing, key, true)
    }
    function control(hash, action, files) {
        var r = Door.controlTorrents([hash], action, files === true)
        if (r.error) frame.toast(page.errorWords(r.error))
    }

    // ---- The head's one line
    readonly property string summary: {
        if (!page.status) return "Connecting."
        if (page.connecting && !page.anirssReady) return "Starting anirss."
        var a = page.anirss
        if (a.state === "Missing") return "anirss is not installed. " + a.message
        if (a.state === "Failed") return "anirss is not running: " + a.message
        if (a.state === "Off") return "anirss is not running yet."
        var t = page.torrentOn ? page.clientName + " " + page.torrent.version + " is connected." : page.clientName + " is not connected: " + (page.torrent.message || "no answer") + "."
        return "Searches run through anirss " + a.version + ". " + t
    }

    Connections {
        target: Door
        function onDownloadsStatusChanged(status) {
            page.status = status
            page.connecting = false
            if (page.anirssReady && !frame.downloadsRulesRead) page.loadRules()
        }
        function onReleasesFound(job, search) {
            if (job !== page.searchJob) return
            page.searching = false
            page.adopt(search)
            var lines = []
            if (search.fell_back) {
                var asked = search.tried && search.tried.length ? search.tried[0].endpoint : ""
                lines.push(asked + ": no results, switched to " + search.endpoint + " (" + search.items.length + ")")
            }
            if (search.auto_resolution) lines.push("Narrowed to " + search.auto_resolution + ", the sharpest on offer")
            page.notice = lines.join(". ")
        }
        function onReleasesRefined(job, refinement) {
            if (job !== page.searchJob) return
            page.searching = false
            if (refinement.applied) {
                var h = page.history.slice(); h.push(page.state); page.history = h
                page.adopt(refinement)
                page.notice = ""
            } else {
                page.notice = page.reasonWords(refinement.reason)
            }
        }
        function onSubscriptionPreviewed(job, spec) { if (job === page.previewJob) page.preview = spec }
        function onSubscribed(job, result) {
            if (job !== page.actionJob) return
            page.acting = false
            page.actionLine = "Subscribed as " + result.name + ". New episodes save to " + result.save_path + "."
        }
        function onTorrentsAdded(job, result) {
            if (job !== page.actionJob) return
            page.acting = false
            page.actionLine = "Added " + Fmt.plural(result.count, "torrent", "torrents") + " under " + result.save_path + "."
            page.selected = {}
            page.pollTorrents()
        }
        function onTorrentsRead(torrents, transfer, shows) {
            if (frame.armedConfirm !== null) return
            var m = {}
            for (var i = 0; i < shows.length; i++) m[shows[i].series_id] = shows[i]
            page.shows = m
            page.torrents = torrents
            page.transfer = transfer
            page.torrentsRead = true
            page.torrentsError = ""
            page.clearing = ({})
        }
        function onFeedRead(url, entries) {
            if (!page.feed || page.feed.url !== url) return
            var sorted = entries.slice().sort(function(a, b) { return (b.published || 0) - (a.published || 0) })
            page.feed = { url: url, entries: sorted, loading: false, error: "" }
        }
        function onDownloadsUnavailable(what, error) {
            if (what === "torrents") page.torrentsError = page.errorWords(error)
            else if (what === "feed" && page.feed) page.feed = { url: page.feed.url, entries: page.feed.entries, loading: false, error: page.errorWords(error) }
        }
        function onSubscriptionRemoved(name) {
            for (var k in page.removeJobs) if (page.removeJobs[k] === name) { page.removalEnded(Number(k), name); break }
            page.pollTorrents()
        }
        function onTorrentsControlled(hashes, action) { page.pollTorrents() }
        function onJobFailed(job, kind, error) {
            if (job === page.searchJob) { page.searching = false; page.searchError = page.errorWords(error) }
            else if (job === page.actionJob) { page.acting = false; page.actionError = page.errorWords(error) }
            else if (job === page.connectJob) { page.connecting = false; frame.toast(page.errorWords(error)) }
            else if (job === page.previewJob) { page.previewJob = -1 }
            else if (page.removeJobs[String(job)] !== undefined) {
                var name = page.removeJobs[String(job)]
                frame.toast(page.errorWords(error))
                page.bulk = []
                page.removing = page.withKey(page.removing, name, false)
                page.removalEnded(job, name)
            } else if (kind === "ControlTorrents") { page.clearing = ({}); frame.toast(page.errorWords(error)) }
        }
        function onJobCancelled(job, kind) {
            if (job === page.searchJob) page.searching = false
            else if (job === page.actionJob) page.acting = false
            else if (job === page.connectJob) page.connecting = false
        }
    }
    // The one poll in the app: the transfers, while this page is up; quick only while
    // something is moving
    // Held while a removal is armed: a fresh list rebuilds the rows, and the armed button
    // with them, between the two clicks
    Timer { interval: page.activeTorrents.length ? 2000 : 15000; running: page.visible && Door.ready && page.torrentOn && frame.armedConfirm === null; repeat: true; onTriggered: page.pollTorrents() }
    Component.onCompleted: {
        reload()
        connect()
        loadRules()
        pollTorrents()
        if (props.query) { search.text = props.query; Qt.callLater(page.runSearch) }
    }

    // ---- The geometry
    readonly property real gap: theme.space(4)
    readonly property real paneWidth: theme.space(90)
    readonly property real tileGap: theme.grout
    readonly property int cols: Math.max(1, Math.floor((scroll.innerWidth + tileGap) / (theme.posterWidth + tileGap)))
    readonly property real cardWidth: Math.floor((scroll.innerWidth - tileGap * (cols - 1)) / cols)
    readonly property bool home: page.detail === null && page.state === null && !page.searching && page.searchError === ""

    PageScroll {
        id: scroll
        anchors.fill: parent
        readonly property real innerWidth: width - theme.space(16)

        // ---- The head: the title, the one line, the two halves as chips
        Item {
            width: parent.width
            height: theme.space(14)
            Label {
                id: titleLabel
                anchors.left: parent.left
                anchors.verticalCenter: parent.verticalCenter
                text: "Downloads"
                role: "display"
            }
            Label {
                objectName: "downloads-summary"
                anchors.left: titleLabel.right
                anchors.leftMargin: theme.space(3)
                anchors.right: chips.left
                anchors.rightMargin: theme.space(4)
                anchors.baseline: titleLabel.baseline
                text: page.summary
                role: "ui"
                color: theme.textDim
                elide: Text.ElideRight
            }
            Row {
                id: chips
                anchors.right: parent.right
                anchors.verticalCenter: parent.verticalCenter
                spacing: theme.space(2)
                Chip {
                    objectName: "downloads-anirss-chip"
                    icon: "rss"
                    text: page.anirssReady ? "anirss " + page.anirss.version : page.connecting ? "anirss starting" : "anirss off"
                    textColor: page.anirssReady ? theme.caughtUp : page.connecting ? theme.textDim : theme.red
                    clickable: true
                    onClicked: frame.go("settings", { tab: "downloads" })
                    Tooltip { text: page.anirssReady ? "anirss at " + page.anirss.url : (page.anirss && page.anirss.message ? page.anirss.message : "Open the Downloads settings") }
                }
                Chip {
                    objectName: "downloads-client-chip"
                    icon: "download"
                    text: page.torrentOn ? page.clientName + " " + page.torrent.version : page.clientName + " off"
                    textColor: page.torrentOn ? theme.caughtUp : theme.red
                    clickable: true
                    onClicked: frame.go("settings", { tab: "downloads" })
                    Tooltip { text: page.torrentOn ? (page.status.settings.torrent_url) : (page.torrent && page.torrent.message ? page.torrent.message : "Open the Downloads settings") }
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

        // ---- The search row: the field, the endpoint, Search
        Row {
            width: parent.width
            spacing: theme.space(2)
            SearchField {
                id: search
                objectName: "downloads-search"
                width: parent.width - endpointPick.width - go.width - parent.spacing * 2
                placeholder: "Search the release feeds"
                hint: "/ then Return"
                onAccepted: page.runSearch()
                onCleared: {}
            }
            Dropdown {
                id: endpointPick
                objectName: "downloads-endpoint"
                options: page.endpoints.length ? page.endpoints.map(function(e) { return e.name }) : ["(no endpoints)"]
                index: page.endpointIndex
                minWidth: theme.space(28)
                interactive: page.endpoints.length > 0
                onPicked: function(i) { page.endpointIndex = i; if (page.state) page.runSearch() }
                Tooltip { text: page.endpoints.length > page.endpointIndex ? page.endpoints[page.endpointIndex].url : "The search endpoints come from anirss's config" }
            }
            Button {
                id: go
                objectName: "downloads-go"
                text: page.searching ? "Searching" : "Search"
                icon: "search"
                primary: true
                enabled: !page.searching
                onClicked: page.runSearch()
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
                Button { text: "Back"; icon: "arrow-left"; flat: true; small: true; enabled: page.history.length > 0; anchors.verticalCenter: parent.verticalCenter; onClicked: page.back() }
                Label { visible: !!page.state; text: page.state ? page.state.query : ""; role: "title"; font.pointSize: theme.typeNormal; anchors.verticalCenter: parent.verticalCenter; elide: Text.ElideMiddle; width: Math.min(implicitWidth, parent.width * 0.5) }
                Label { visible: !!page.state; text: page.state ? Fmt.plural(page.items.length, "release", "releases") + " on " + page.state.endpoint : ""; role: "ledger"; color: theme.textDim; anchors.verticalCenter: parent.verticalCenter }
                Label { visible: page.searching; text: page.state ? "Narrowing" : "Searching"; role: "ui"; color: theme.textDim; anchors.verticalCenter: parent.verticalCenter }
                Label { visible: page.notice !== "" && !page.searching; text: page.notice; role: "ui"; color: theme.textDim; anchors.verticalCenter: parent.verticalCenter; elide: Text.ElideRight }
                Label { visible: page.searchError !== ""; text: page.searchError; role: "ui"; color: theme.red; anchors.verticalCenter: parent.verticalCenter; elide: Text.ElideRight }
            }
            // The groups: a token or a few that split the list, a count on each
            Flow {
                visible: page.groups.length > 0
                width: parent.width
                spacing: theme.space(1.5)
                Repeater {
                    model: page.groups
                    Chip {
                        required property var modelData
                        required property int index
                        text: modelData.label + "  " + modelData.member_count
                        small: true
                        clickable: !page.searching
                        selected: page.hoveredGroup === index
                        onClicked: page.refine({ type: "tokens", tokens: modelData.tokens })
                        HoverHandler { onHoveredChanged: page.hoveredGroup = hovered ? index : (page.hoveredGroup === index ? -1 : page.hoveredGroup) }
                    }
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
            // The releases at the left, the actions at the right
            Item {
                width: parent.width
                height: Math.max(list.implicitHeight, pane.implicitHeight)
                Column {
                    id: list
                    x: 0
                    width: parent.width - page.paneWidth - page.gap
                    Repeater {
                        model: page.items.slice(0, page.shown)
                        ReleaseRow {
                            required property var modelData
                            required property int index
                            release: modelData
                            selected: page.selected[modelData.link] === true
                            inGroup: page.hoveredGroup >= 0 && page.groups[page.hoveredGroup] && page.groups[page.hoveredGroup].members.indexOf(index) >= 0
                            onToggled: page.toggle(modelData)
                            onDownloadOne: page.download([modelData])
                        }
                    }
                    Label {
                        visible: page.items.length > page.shown
                        topPadding: theme.space(2)
                        text: "and " + (page.items.length - page.shown) + " more. Narrow the list to see them."
                        role: "ui"
                        color: theme.textDim
                    }
                    Item {
                        visible: page.state && page.items.length === 0 && !page.searching
                        width: parent.width
                        height: visible ? theme.space(40) : 0
                        EmptyState { icon: "search"; title: "No releases"; body: "Nothing on " + (page.state ? page.state.endpoint : "") + " matches. Try another endpoint or a shorter query." }
                    }
                }
                Tile {
                    id: pane
                    x: parent.width - page.paneWidth
                    width: page.paneWidth
                    implicitHeight: paneBody.implicitHeight + padding * 2
                    height: implicitHeight
                    padding: theme.space(4)
                    lit: false
                    Column {
                        id: paneBody
                        width: parent.width
                        spacing: theme.space(2.5)
                        Label { text: "Name"; role: "ledger"; color: theme.textDim }
                        Field {
                            objectName: "downloads-name"
                            width: parent.width
                            placeholder: "anirss picks one from the first release"
                            text: page.nameText
                            onEdited: function(t) { page.nameText = t }
                        }
                        Label { width: parent.width; text: "The folder under anirss's save base. A subscription keeps this name as its rule."; role: "ui"; font.pointSize: theme.typeSmall; color: theme.textFaint; wrapMode: Text.Wrap }
                        Item { width: 1; height: theme.space(1) }
                        Button {
                            objectName: "downloads-subscribe"
                            width: parent.width
                            text: page.acting ? "Working" : "Subscribe to this search"
                            icon: "rss"
                            primary: true
                            enabled: !!page.state && page.items.length > 0 && !page.acting && !page.searching
                            onClicked: page.subscribe()
                        }
                        Button {
                            width: parent.width
                            text: page.preview ? "Preview again" : "Preview the rule"
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
                            Label { width: parent.width; text: page.preview ? "Feed " + page.preview.feed_url : ""; role: "ledger"; color: theme.textDim; elide: Text.ElideMiddle }
                            Label { visible: !!(page.preview && page.preview.must_contain); width: parent.width; text: page.preview ? "Must contain " + page.preview.must_contain : ""; role: "ledger"; color: theme.textDim; elide: Text.ElideRight }
                            Label { visible: !!(page.preview && page.preview.must_not_contain); width: parent.width; text: page.preview ? "Must not contain " + page.preview.must_not_contain : ""; role: "ledger"; color: theme.textDim; elide: Text.ElideRight }
                        }
                        Rectangle { width: parent.width; height: 1; color: theme.line }
                        Button {
                            objectName: "downloads-download"
                            width: parent.width
                            text: page.selectedCount > 0 ? "Download " + Fmt.plural(page.selectedCount, "selected release", "selected releases") : "Download all " + page.items.length
                            icon: "download"
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
                            Tooltip { text: "One selected release, straight into the movie folder" }
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
                clientIndex: page.clientIndex
                nowMs: page.nowMs
                removing: !!page.detailRule && page.removing[page.detailRule.name] === true
                onBack: page.closeDetail()
                onRemoveRss: page.removeRss(page.detailRule)
                onRemoveTorrents: function(files) { page.clearTorrents(page.detail.key, page.detailTorrents, files) }
                onControl: function(hash, action, files) { page.control(hash, action, files) }
                onDownloadEntry: function(entry) { page.downloadEntry(entry) }
                onRefreshFeed: page.readFeed()
            }
        }

        // ---- Downloading: only while something moves
        Column {
            visible: page.detail === null && page.activeTorrents.length > 0
            width: parent.width
            spacing: theme.space(1)
            SectionHead {
                title: "Downloading"
                count: page.activeTorrents.length
                line: {
                    var parts = []
                    if (page.transfer && page.transfer.dl_speed > 0) parts.push(Fmt.bytes(page.transfer.dl_speed) + "/s down")
                    if (page.transfer && page.transfer.up_speed > 0) parts.push(Fmt.bytes(page.transfer.up_speed) + "/s up")
                    return parts.length ? parts.join(", ") : "Waiting for peers."
                }
            }
            Repeater {
                model: page.activeTorrents.slice(0, 50)
                TorrentRow {
                    required property var modelData
                    torrent: modelData
                    onStop: page.control(modelData.hash, "Stop", false)
                    onStart: page.control(modelData.hash, "Start", false)
                    onRemove: function(files) { page.control(modelData.hash, "Delete", files) }
                }
            }
        }

        // ---- Subscriptions: a poster each, the complete ones first
        Column {
            visible: page.home
            width: parent.width
            spacing: theme.space(3)
            Item {
                width: parent.width
                height: theme.space(12)
                SectionHead {
                    anchors.left: parent.left
                    anchors.right: subSeg.left
                    anchors.rightMargin: theme.space(4)
                    title: "Subscriptions"
                    count: page.rules.length
                    line: page.rulesError !== "" ? page.rulesError : page.rulesLoaded ? (page.rules.length ? "Feeds anirss watches, as the shows they fill." : "None yet.") : (page.anirssReady ? "Reading." : "")
                    alarm: page.rulesError !== ""
                }
                Seg {
                    id: subSeg
                    objectName: "downloads-sub-filter"
                    anchors.right: parent.right
                    anchors.bottom: parent.bottom
                    anchors.bottomMargin: theme.space(1)
                    small: true
                    options: ["All " + page.rules.length, "Complete " + page.completeRules.length, "Airing " + page.airingRules.length]
                    index: page.subFilter
                    onPicked: function(i) { page.subFilter = i }
                }
            }
            // The things to do: complete subscriptions, their RSS can go
            Tile {
                visible: page.completeRules.length > 0
                width: parent.width
                implicitHeight: theme.space(20)
                height: implicitHeight
                padding: theme.space(4)
                lit: false
                Rectangle { x: 0; anchors.verticalCenter: parent.verticalCenter; width: theme.space(3); height: width; radius: width / 2; color: theme.red }
                Column {
                    x: theme.space(6)
                    anchors.verticalCenter: parent.verticalCenter
                    width: parent.width - x - attentionButtons.width - theme.space(8)
                    spacing: theme.space(0.75)
                    Label {
                        width: parent.width
                        text: page.bulkCurrent !== "" ? "Removing " + page.bulkCurrent + (page.bulk.length ? ", " + page.bulk.length + " to go" : "") + "."
                              : Fmt.plural(page.completeRules.length, "subscription is complete", "subscriptions are complete") + "."
                        role: "title"
                        font.pointSize: theme.typeNormal
                        elide: Text.ElideRight
                    }
                    Label {
                        width: parent.width
                        text: "Every episode is in the library and nothing more will air. Removing their RSS takes the feed, the rule and the torrents out of the client; the files stay."
                        role: "ui"
                        color: theme.textDim
                        elide: Text.ElideRight
                    }
                }
                Row {
                    id: attentionButtons
                    anchors.right: parent.right
                    anchors.verticalCenter: parent.verticalCenter
                    spacing: theme.space(2)
                    Button { visible: page.subFilter !== 1; text: "Show them"; icon: "list-filter"; flat: true; onClicked: page.subFilter = 1 }
                    ConfirmButton {
                        objectName: "downloads-remove-complete"
                        label: page.completeRules.length === 1 ? "Remove its RSS" : "Remove RSS for all " + page.completeRules.length
                        confirmText: "Confirm, remove " + page.completeRules.length
                        icon: "rss"
                        enabled: page.bulkCurrent === ""
                        onConfirmed: page.removeAllComplete()
                    }
                }
            }
            Flow {
                width: parent.width
                spacing: page.tileGap
                Repeater {
                    model: page.shownRules
                    SubscriptionCard {
                        required property var modelData
                        width: page.cardWidth
                        height: implicitHeight
                        rule: modelData
                        nowMs: page.nowMs
                        removing: page.removing[modelData.name] === true
                        onOpened: page.openRule(modelData)
                        onRemoveRss: page.removeRss(modelData)
                    }
                }
            }
            Item {
                visible: page.rulesLoaded && page.shownRules.length === 0 && page.rulesError === ""
                width: parent.width
                height: visible ? theme.space(30) : 0
                EmptyState {
                    icon: "rss"
                    title: page.rules.length ? "None here" : "No subscriptions"
                    body: page.rules.length ? "No subscription fits this filter." : "Search above and subscribe to keep a show coming."
                }
            }
        }

        // ---- Downloaded: one row per show folder the client holds torrents for
        Column {
            visible: page.home
            width: parent.width
            spacing: theme.space(1)
            Item { width: 1; height: theme.space(4) }
            SectionHead {
                width: parent.width
                title: "Downloaded"
                count: page.showGroups.length
                line: !page.torrentOn ? (page.torrent && page.torrent.message ? page.torrent.message : page.clientName + " is not connected.")
                      : page.torrentsError !== "" ? page.torrentsError
                      : page.torrentsRead ? Fmt.plural(page.torrents.length, "torrent", "torrents") + ", " + Fmt.bytes(page.downloadedSize) + " in " + page.clientName + "." : "Reading."
                alarm: !page.torrentOn || page.torrentsError !== ""
            }
            Repeater {
                model: page.allGroups ? page.showGroups : page.showGroups.slice(0, page.groupCap)
                DownloadedRow {
                    required property var modelData
                    group: modelData
                    busy: page.clearing[modelData.key] === true
                    onOpened: page.openGroup(modelData)
                    onRemoveTorrents: function(files) { page.clearTorrents(modelData.key, modelData.torrents, files) }
                }
            }
            Button {
                visible: page.showGroups.length > page.groupCap
                text: page.allGroups ? "Show fewer" : "Show all " + page.showGroups.length
                flat: true
                small: true
                onClicked: page.allGroups = !page.allGroups
            }
            Item {
                visible: page.torrentsRead && page.showGroups.length === 0
                width: parent.width
                height: visible ? theme.space(30) : 0
                EmptyState {
                    icon: page.torrentOn ? "download" : "circle-alert"
                    title: page.torrentOn ? "Nothing downloaded" : page.clientName + " is not connected"
                    body: page.torrentOn ? "What you download or subscribe to shows up here, a row per show." : (page.torrent && page.torrent.message ? page.torrent.message : "Check its address and password in Settings.")
                    Button { visible: !page.torrentOn; text: "Settings"; icon: "settings"; onClicked: frame.go("settings", { tab: "downloads" }) }
                }
            }
        }
    }

    // A band's head: the title in the display cut, its count, one line beside it
    component SectionHead: Item {
        id: head
        property string title: ""
        property int count: 0
        property string line: ""
        property bool alarm: false
        width: parent ? parent.width : 0
        height: theme.space(12)
        Row {
            anchors.left: parent.left
            anchors.right: parent.right
            anchors.bottom: parent.bottom
            anchors.bottomMargin: theme.space(2)
            spacing: theme.space(3)
            Label { id: headTitle; text: head.title; role: "display"; font.pointSize: theme.typeLarge; font.variableAxes: ({ "wdth": 74, "wght": 500 }) }
            Label { anchors.baseline: headTitle.baseline; visible: head.count > 0; text: head.count; role: "ledger"; color: theme.textFaint }
            Label { anchors.baseline: headTitle.baseline; text: head.line; role: "ui"; color: head.alarm ? theme.red : theme.textDim; elide: Text.ElideRight; width: Math.max(0, Math.min(implicitWidth, parent.width - headTitle.width - theme.space(20))) }
        }
    }
}
