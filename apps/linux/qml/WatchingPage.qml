// The watching page: what the user is watching or rewatching on AniList, as a mosaic wall
// on the library's module. The core owns the cache and the refresh (ListWatching replies
// the cached list at once plus the id of a RefreshWatching job when it started one;
// WatchingRefreshed brings the fresh list). An entry the library owns opens the series
// in-app, and is a hero with Play when it airs this week; the rest are ghost cards that
// open AniList. ListWatching reads the cache only; RefreshWatching goes to AniList, on
// open when the cache is over five minutes old and from the Refresh tile when asked. The
// page reads on open and on tracker and watching events, never on library changes.
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

    readonly property var sections: {
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
        if (!page.anilistConnected) { page.entries = []; return }
        var r = Door.listWatching()
        if (r.error) { page.loadFailed = true; frame.toast(r.error.message); return }
        page.loadFailed = false
        page.packedAt = Date.now()
        page.entries = r.reply.list.entries || []
        if (r.reply.refreshing !== null && r.reply.refreshing !== undefined) page.refreshing = true
    }
    // To AniList: `force` from the Refresh tile, else only when the cache is stale
    function refresh(force) {
        if (!page.anilistConnected) return
        var r = Door.refreshWatching(force)
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
            visible: page.entries.length > 0
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
                    icon: "log-in"
                    title: "AniList is not connected"
                    body: "Connect AniList in Settings to see your watching list."
                    // tracker-one-click merge: ConnectButton + frame.openConnect("Anilist")
                    Button { objectName: "watching-connect"; text: "Open Settings"; icon: "settings"; primary: true; onClicked: frame.go("settings", { tab: "Library" }) }
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
