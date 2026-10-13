// The watching page: what the user is watching or rewatching on AniList, as a mosaic wall
// on the library's module. The core owns the cache and the refresh (ListWatching replies
// the cached list at once plus the id of a RefreshWatching job when it started one;
// WatchingRefreshed brings the fresh list). An entry the library owns opens the series
// in-app, and is a hero with Play when it airs this week; the rest are ghost cards that
// open AniList. ListWatching reads the cache only; RefreshWatching goes to AniList, on
// open when the cache is over five minutes old and from the Refresh tile when asked. The
// page reads on open and on tracker and watching events, never on library changes.
// Without AniList the page is still a wall: a connect tile and a line on what the page
// shows once connected head it, and under them what this machine knows on its own (the
// series played here, the library's series airing, and the rest newest on disk first).
import QtQuick
import com.marcusrosado.AniBeam
import "mosaic.js" as Mosaic

Item {
    id: page
    objectName: "watching-page"
    property var props: ({})
    property string title: "Watching"
    property alias scrollY: wall.scrollY
    function escapePressed() { return false }

    property var entries: []
    property bool anilistConnected: false
    property bool loadFailed: false
    property real nowMs: Date.now()
    property real packedAt: Date.now()
    property bool refreshing: false
    property var localPlayed: []    // library cards with a view on this machine, newest first
    property var localAiring: []    // the library's airing rail
    property var localLatest: []    // every library card, newest activity first

    function entryItem(e) {
        return {
            id: e.owned !== null && e.owned !== undefined ? e.owned : -e.anilist_id,
            title: e.title,
            poster: e.poster || "",
            watched: e.progress > 0 ? e.progress : null,
            total_episodes: e.total || null,
            total_is_estimate: false,
            watched_state: "Unknown",
            strip: null,
            code: e.progress > 0 ? "Ep " + String(Math.floor(e.progress)).padStart(2, "0") : "",
            community_score: e.score !== null && e.score !== undefined ? e.score : null,
            my_score: null,
            hidden: false,
            next_airing: e.next_airing || null,
            episodes_on_disk: 0,
            site_url: e.owned ? "" : (e.site_url || "https://anilist.co/anime/" + e.anilist_id)
        }
    }
    function owned(e) { return e.owned !== null && e.owned !== undefined }

    function localSections() {
        var now = packedAt / 1000, out = []
        function entry(c, meta) { return { id: c.id, item: c, raw: { owned: c.id, title: c.title }, ghost: false, meta: meta } }
        if (localPlayed.length) {
            var played = localPlayed.map(function(c) { return entry(c, "Played " + Fmt.relative(c.last_viewed_at, nowMs / 1000)) })
            out.push({ label: "Played here", count: played.length, cards: played,
                       fact: { text: "Kept on this machine" },
                       heroes: Mosaic.pickHeroes(localPlayed, now, 2, null).map(function(c) { return played[localPlayed.indexOf(c)] }) })
        }
        var playedIds = {}
        for (var i = 0; i < localPlayed.length; i++) playedIds[localPlayed[i].id] = true
        var airing = localAiring.filter(function(c) { return !playedIds[c.id] })
        if (airing.length) {
            var best = null
            for (var j = 0; j < airing.length; j++) {
                var n = airing[j].next_airing
                if (n && n.at > now && (!best || n.at < best.at)) best = { at: n.at, text: airing[j].title }
            }
            var rows = airing.map(function(c) { return entry(c, "") })
            out.push({ label: "Airing", count: rows.length, cards: rows, fact: best,
                       heroes: Mosaic.pickHeroes(airing, now, airing.length, null).map(function(c) { return entry(c, "") }) })
        }
        var shown = {}
        for (var k = 0; k < out.length; k++) for (var m = 0; m < out[k].cards.length; m++) shown[out[k].cards[m].id] = true
        var latest = localLatest.filter(function(c) { return !shown[c.id] }).map(function(c) { return entry(c, "") })
        if (latest.length) out.push({ label: "Latest on disk", count: latest.length, cards: latest,
                                      fact: { text: "Newest downloads first" }, heroes: [] })
        return out
    }
    readonly property var sections: {
        if (!anilistConnected) return localSections()
        if (entries.length === 0) return []
        var now = packedAt / 1000
        var list = [], heroes = []
        for (var i = 0; i < entries.length; i++) {
            var e = entries[i]
            var item = entryItem(e)
            // The airing countdown says more than "updated 3d ago" when one runs
            var airs = item.next_airing && item.next_airing.at > now
            var x = { id: item.id, item: item, raw: e, ghost: !owned(e),
                      meta: airs ? "" : (e.updated_at ? "AniList updated " + Fmt.relative(e.updated_at, nowMs / 1000) : "") }
            list.push(x)
            if (owned(e) && Mosaic.heroRank(item, now) >= 0) heroes.push(x)
        }
        heroes.sort(function(a, b) { return a.item.next_airing.at - b.item.next_airing.at })
        return [{ label: "", count: list.length, cards: list, heroes: heroes }]
    }

    function reload() {
        var t = Door.getTrackers()
        if (!t.error) page.anilistConnected = !!(t.reply.state.anilist.connected)
        if (!page.anilistConnected) { page.entries = []; page.loadLocal(); return }
        var r = Door.listWatching()
        if (r.error) { page.loadFailed = true; frame.toast(r.error.message); return }
        page.loadFailed = false
        page.packedAt = Date.now()
        page.entries = r.reply.list.entries || []
        if (r.reply.refreshing !== null && r.reply.refreshing !== undefined) page.refreshing = true
    }
    // What the page shows without AniList: the library's own read, no network
    function loadLocal() {
        var r = Door.listLibrary("All", "", "LastViewed", "Desc", false)
        if (r.error) { page.localPlayed = []; page.localAiring = []; return }
        page.packedAt = Date.now()
        page.localPlayed = (r.reply.series || []).filter(function(c) { return !!c.last_viewed_at })
        page.localAiring = r.reply.airing || []
        page.localLatest = (r.reply.series || []).slice().sort(function(a, b) { return b.latest_activity_at - a.latest_activity_at })
    }
    // To AniList: `force` from the Refresh tile, else only when the cache is stale. The core
    // answers Ok without a fetch when no account is connected, so the page always asks.
    property int refreshAsked: 0   // for a --drive eval: the open asked once
    function refresh(force) {
        refreshAsked++
        var r = Door.refreshWatching(force === true)
        if (r.error) { page.loadFailed = page.entries.length === 0; frame.toast(r.error.message); return }
        if (r.reply && r.reply.job !== undefined) page.refreshing = true
    }
    function playSeries(id, title) {
        var r = Door.getSeries(id)
        var d = !r.error ? r.reply.detail : null
        if (d && d.next_up) frame.go("player", { file: d.next_up }, title)
        else frame.go("series", { id: id }, title)
    }
    // Tracker events come in bursts (connect, then the state push): one read for them
    Timer { id: debounce; interval: 400; onTriggered: page.reload() }
    Timer { interval: 30000; running: page.entries.length > 0; repeat: true; onTriggered: page.nowMs = Date.now() }
    Connections {
        target: Door
        function onWatchingRefreshed(list) { page.refreshing = false; page.loadFailed = false; page.packedAt = Date.now(); page.entries = list.entries || [] }
        function onTrackersChanged() { debounce.restart() }
        function onViewed(series, episode) { if (!page.anilistConnected) page.loadLocal() }
        function onJobFailed(job, kind, error) { if (kind === "RefreshWatching") { page.refreshing = false; page.loadFailed = page.entries.length === 0; frame.toast(error && error.message ? error.message : "AniList could not be reached") } }
        function onJobCancelled(job, kind) { if (kind === "RefreshWatching") page.refreshing = false }
    }
    Component.onCompleted: { reload(); refresh(false) }

    MosaicWall {
        id: wall
        anchors.fill: parent
        sections: page.sections
        nowMs: page.nowMs
        onOpened: function(x) {
            if (!x.ghost) frame.go("series", { id: x.raw.owned }, x.raw.title)
            else Qt.openUrlExternally(x.item.site_url)
        }
        onPlayed: function(x) { page.playSeries(x.raw.owned, x.raw.title) }

        MosaicWall.HeaderTile {
            need: titleCol.implicitWidth
            minSpan: 2
            flex: true
            Column {
                id: titleCol
                x: parent.pad
                anchors.verticalCenter: parent.verticalCenter
                spacing: theme.space(1)
                Label { text: "Watching"; role: "display" }
                Label { text: "What you're watching or rewatching on AniList."; role: "ui"; color: theme.textDim }
            }
        }
        MosaicWall.HeaderTile {
            objectName: "watching-connect-tile"
            visible: !page.anilistConnected
            need: connectCol.implicitWidth
            Column {
                id: connectCol
                x: parent.pad
                anchors.verticalCenter: parent.verticalCenter
                spacing: theme.space(1.5)
                Label { text: "AniList is not connected"; role: "title" }
                // One click: the browser opens on it when the build carries AniList's id
                ConnectButton {
                    objectName: "watching-connect"
                    tracker: "Anilist"
                    text: frame.connecting["Anilist"] !== undefined ? "Waiting for the browser" : "Connect AniList"
                    icon: frame.connecting["Anilist"] !== undefined ? "clock" : "log-in"
                    primary: true
                    onClicked: frame.openConnect("Anilist")
                }
            }
        }
        MosaicWall.HeaderTile {
            objectName: "watching-explain-tile"
            visible: !page.anilistConnected
            need: theme.space(120)
            flex: true
            Label {
                x: parent.pad
                width: parent.width - parent.pad * 2
                anchors.verticalCenter: parent.verticalCenter
                text: "Connected, this page lists your Watching and Rewatching series on AniList: Play on the ones you own, a countdown on the ones airing. Until then it shows what this machine knows."
                role: "ui"
                color: theme.textDim
                wrapMode: Text.Wrap
                maximumLineCount: 3
                elide: Text.ElideRight
            }
        }
        MosaicWall.HeaderTile {
            visible: page.anilistConnected
            need: refreshBtn.implicitWidth
            Button {
                id: refreshBtn
                objectName: "watching-refresh"
                x: parent.pad
                anchors.verticalCenter: parent.verticalCenter
                text: page.refreshing ? "Refreshing" : "Refresh"
                icon: "refresh-cw"
                enabled: !page.refreshing
                onClicked: page.refresh(true)
            }
        }
        MosaicWall.HeaderTile {
            visible: page.anilistConnected && page.entries.length > 0
            need: countRow.implicitWidth
            Row {
                id: countRow
                anchors.right: parent.right; anchors.rightMargin: parent.pad
                anchors.verticalCenter: parent.verticalCenter
                spacing: theme.space(1.5)
                Label {
                    id: countFigure
                    text: page.entries.length
                    role: "ledger"
                    font.pointSize: theme.typeDisplay
                    font.weight: 700
                    font.variableAxes: ({ "wdth": 88, "wght": 700 })
                }
                Label {
                    anchors.baseline: countFigure.baseline
                    text: "series"
                    role: "ui"
                    font.pointSize: theme.typeSmall
                    color: theme.textDim
                }
            }
        }

        // No list to show: one tile filling the page, saying why and what to do
        body: Component {
            Tile {
                implicitHeight: theme.space(60)
                EmptyState {
                    visible: !page.anilistConnected
                    icon: "play"
                    title: "Nothing played here yet"
                    body: "Play an episode from the library and it shows here, with the series airing this week."
                    Button { text: "Open the library"; icon: "house"; onClicked: frame.go("library", {}, "Library") }
                }
                EmptyState {
                    visible: page.anilistConnected && page.loadFailed
                    icon: "triangle-alert"
                    title: "The watching list did not load"
                    body: "AniList could not be reached. The list comes back on the next refresh."
                    Button { text: "Try again"; icon: "refresh-cw"; onClicked: { page.reload(); page.refresh(true) } }
                }
                EmptyState {
                    visible: page.anilistConnected && !page.loadFailed
                    icon: "eye"
                    title: "Nothing on your watching list"
                    body: "Mark a show as Watching on AniList and it lands here."
                }
            }
        }
    }
}
