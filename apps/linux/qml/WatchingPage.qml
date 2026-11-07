// The watching page: what the user is watching or rewatching on AniList, as sections of
// uniform cards. The core owns the cache and the refresh (ListWatching replies the cached
// list at once plus the id of a RefreshWatching job when it started one; WatchingRefreshed
// brings the fresh list). An entry the library owns opens the series in-app; the rest are
// ghost cards that open AniList. ListWatching reads the cache only; RefreshWatching goes
// to AniList, on open when the cache is over five minutes old and from Refresh when asked.
// The page reads on open and on tracker and watching events, never on library changes.
// The sections: Airing (a next episode is scheduled), Episodes waiting (behind on a
// finished run), Up to date, and Not in the library (the ghosts). Without AniList the
// page is one empty state with the connect button.
//
// Planning (#210), the second tab: the entries on AniList's Planning list, off the same
// refresh (ListPlanning reads the cache). On disk first, then the rest. A card opens its
// actions: Start watching moves it to Watching on every connected tracker and plays the
// first episode; one not in the library offers Get episodes, which opens Downloads on it.
import QtQuick
import com.marcusrosado.AniBeam

Item {
    id: page
    objectName: "watching-page"
    property var props: ({})
    property string title: "Watching"
    property alias scrollY: scroll.contentY
    function escapePressed() { return false }

    property var entries: []
    // "watching" or "planning", kept in the page's props so Back returns to the tab
    property string tab: props.tab === "planning" ? "planning" : "watching"
    property var planned: []
    property bool planningFailed: false
    // The list was read by the last reload or refresh; a switch only shows it, from the top
    function pickTab(t) {
        page.tab = t
        frame.nav.current.props = { tab: t }
        scroll.contentY = 0
        if (t === "planning" && page.planningFailed) page.readPlanning()
    }
    property bool anilistConnected: false
    property bool loadFailed: false
    property real nowMs: Date.now()
    property real cutAt: Date.now()
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
    function shortCountdown(secs) {
        var t = Math.max(0, Math.floor(secs))
        var d = Math.floor(t / 86400), h = Math.floor(t % 86400 / 3600), m = Math.floor(t % 3600 / 60)
        return d > 0 ? d + "d " + h + "h" : h > 0 ? h + "h " + m + "m" : Math.max(1, m) + "m"
    }

    // The cards, cut once per reload against that moment's clock. Owned entries go by
    // state: a next episode scheduled is Airing, a finished run with episodes left is
    // waiting, the rest up to date. Every entry the library does not own is a ghost.
    readonly property var sections: tab === "planning" ? planningSections : watchingSections
    readonly property var planningSections: {
        if (!anilistConnected || planned.length === 0) return []
        var here = [], away = []
        for (var i = 0; i < planned.length; i++) {
            var e = planned[i]
            var item = entryItem(e)
            var x = { id: item.id, item: item, raw: e, ghost: !owned(e), planning: true,
                      meta: owned(e) ? "On disk" : (e.total ? Fmt.plural(e.total, "episode", "episodes") : "On AniList") }
            if (owned(e)) here.push(x)
            else away.push(x)
        }
        var out = []
        if (here.length) out.push({ title: "On disk", cards: here, accent: true, fact: "Ready to start" })
        if (away.length) out.push({ title: "Not in the library", cards: away, accent: false, fact: "Get episodes to start" })
        return out
    }
    readonly property var watchingSections: {
        if (!anilistConnected || entries.length === 0) return []
        var now = page.cutAt / 1000
        var airing = [], waiting = [], done = [], ghosts = []
        for (var i = 0; i < entries.length; i++) {
            var e = entries[i]
            var item = entryItem(e)
            var airs = item.next_airing && item.next_airing.at > now
            // The airing countdown says more than "updated 3d ago" when one runs
            var x = { id: item.id, item: item, raw: e, ghost: !owned(e),
                      meta: airs ? "" : (e.updated_at ? "AniList updated " + Fmt.relative(e.updated_at, nowMs / 1000) : "") }
            if (!owned(e)) ghosts.push(x)
            else if (airs) airing.push(x)
            else if (e.total && e.progress >= e.total) done.push(x)
            else waiting.push(x)
        }
        airing.sort(function(a, b) { return a.item.next_airing.at - b.item.next_airing.at })
        var out = []
        var next = airing.length ? airing[0] : null
        if (airing.length) out.push({ title: "Airing", cards: airing, accent: true,
                                      fact: next ? "Next: " + next.item.title + " in " + shortCountdown(next.item.next_airing.at - now) : "" })
        if (waiting.length) out.push({ title: "Episodes waiting", cards: waiting, accent: false,
                                       fact: Fmt.plural(waiting.length, "series", "series") + " behind" })
        if (done.length) out.push({ title: "Up to date", cards: done, accent: false, fact: "" })
        if (ghosts.length) out.push({ title: "Not in the library", cards: ghosts, accent: false, fact: "On AniList only" })
        return out
    }
    // The head's one line: what the list has to say right now
    readonly property string summary: tab === "planning" ? planningSummary : watchingSummary
    readonly property string planningSummary: {
        if (!anilistConnected) return "What you plan to watch on AniList."
        if (planned.length === 0) return ""
        var here = planned.filter(function(e) { return page.owned(e) }).length
        return Fmt.plural(planned.length, "show", "shows") + " planned, " + here + " on disk."
    }
    readonly property string watchingSummary: {
        if (!anilistConnected) return "What you're watching or rewatching on AniList."
        if (entries.length === 0) return ""
        var now = nowMs / 1000, waiting = 0, next = null
        for (var i = 0; i < entries.length; i++) {
            var e = entries[i]
            var n = e.next_airing
            if (n && n.at > now) { if (!next || n.at < next.at) next = { at: n.at, title: e.title } }
            else if (owned(e) && !(e.total && e.progress >= e.total)) waiting++
        }
        if (waiting > 0) return Fmt.plural(waiting, "series has", "series have") + " episodes waiting."
        if (next) return "Next: " + next.title + " in " + shortCountdown(next.at - now)
        return "All caught up."
    }

    function reload() {
        var t = Door.getTrackers()
        // A failed read keeps what the frame last knew, rather than calling AniList not connected
        page.anilistConnected = !t.error ? !!(t.reply.state.anilist.connected) : !!(Door.trackers.anilist && Door.trackers.anilist.connected)
        if (!page.anilistConnected) { page.entries = []; return }
        var r = Door.listWatching()
        if (r.error) { page.loadFailed = true; frame.fail(r.error, page.reload); return }
        page.loadFailed = false
        page.cutAt = Date.now()
        page.nowMs = page.cutAt
        page.entries = r.reply.list.entries || []
        if (r.reply.refreshing !== null && r.reply.refreshing !== undefined) page.refreshing = true
        page.readPlanning()
    }
    function readPlanning() {
        if (!page.anilistConnected) { page.planned = []; return }
        var r = Door.call("ListPlanning", {})
        if (r.error) {
            // Said once, on the tab that shows it; the tab then shows the failure, not old cards
            if (!page.planningFailed && page.tab === "planning") frame.fail(r.error)
            page.planningFailed = true
            page.planned = []
            return
        }
        page.planningFailed = false
        page.planned = r.reply.list.entries || []
    }
    // Start watching (#210): Watching on every connected tracker, then the first episode
    function startWatching(e) {
        if (frame.setListStatus(e.owned, "Watching", "Watching")) frame.playNext(e.owned, e.title)
    }
    // Downloads searches by the latin titles, so each goes over as AniList has it
    function getEpisodes(e) {
        frame.go("downloads", { hit: { id: e.anilist_id, title: e.title, romaji: e.romaji || "", english: e.english || "",
                                       native: e.native || "", cover_url: e.poster || "", episodes: e.total || 0,
                                       format: "", year: 0 } }, "Downloads")
    }
    function planningActions(x) {
        var e = x.raw, items = []
        if (page.owned(e)) {
            items.push({ text: "Start watching", icon: "play", action: function() { page.startWatching(e) } })
            items.push({ text: "Open series", icon: "layers", action: function() { frame.go("series", { id: e.owned }, e.title) } })
        } else {
            items.push({ text: "Get episodes", icon: "download", action: function() { page.getEpisodes(e) } })
        }
        items.push({ text: "Open on AniList", icon: "external-link", action: function() { frame.openLink(e.site_url || "https://anilist.co/anime/" + e.anilist_id) } })
        return items
    }
    function openPlanning(x, card) {
        var p = card.mapToItem(frame, card.width / 2, card.height / 2)
        frame.openMenu(p.x, p.y, page.planningActions(x))
    }
    // To AniList: `force` from Refresh, else only when the cache is stale. The core
    // answers Ok without a fetch when no account is connected, so the page always asks.
    property int refreshAsked: 0   // for a --drive eval: the open asked once
    function refresh(force) {
        refreshAsked++
        var r = Door.refreshWatching(force === true)
        if (r.error) { page.loadFailed = page.entries.length === 0; frame.fail(r.error, function() { page.refresh(force) }); return }
        if (r.reply && r.reply.job !== undefined) page.refreshing = true
    }
    function playSeries(id, title) {
        var r = Door.getSeries(id)
        var d = !r.error ? r.reply.detail : null
        if (d && d.next_up) frame.go("player", { file: d.next_up }, title)
        else frame.go("series", { id: id }, title)
    }
    function open(x) {
        if (!x.ghost) frame.go("series", { id: x.raw.owned }, x.raw.title)
        else frame.openLink(x.item.site_url)
    }
    // Tracker events come in bursts (connect, then the state push): one read for them
    Timer { id: debounce; interval: 400; onTriggered: page.reload() }
    Timer { interval: 30000; running: page.entries.length > 0; repeat: true; onTriggered: page.nowMs = Date.now() }
    Connections {
        target: Door
        function onWatchingRefreshed(list) { page.refreshing = false; page.loadFailed = false; page.cutAt = Date.now(); page.nowMs = page.cutAt; page.entries = list.entries || []; page.readPlanning() }
        // Start watching, or a status changed anywhere, moves an entry between the tabs
        function onJobFinished(job, kind, ok) { if (kind === "SetStatus") page.reload() }
        function onTrackersChanged() { debounce.restart() }
        function onJobFailed(job, kind, error) { if (kind === "RefreshWatching") { page.refreshing = false; page.loadFailed = page.entries.length === 0; frame.fail(error, function() { page.refresh(true) }) } }
        function onJobCancelled(job, kind) { if (kind === "RefreshWatching") page.refreshing = false }
    }
    Component.onCompleted: { reload(); refresh(false) }

    // ---- The grid's geometry: as many columns as the width holds at the poster width
    readonly property real gap: theme.space(4)
    readonly property int cols: Math.max(1, Math.floor((scroll.innerWidth + gap) / (theme.posterWidth + gap)))
    readonly property real cardWidth: Math.floor((scroll.innerWidth - gap * (cols - 1)) / cols)

    PageScroll {
        id: scroll
        anchors.fill: parent
        readonly property real innerWidth: column - theme.space(16)

        // ---- The head: the title, the one line beside it, Refresh at the right
        Item {
            width: parent.width
            height: theme.space(20)
            Label {
                id: titleLabel
                anchors.left: parent.left
                anchors.verticalCenter: parent.verticalCenter
                text: page.tab === "planning" ? "Planning" : "Watching"
                role: "display"
                font.pointSize: theme.typePage
            }
            Label {
                objectName: "watching-summary"
                anchors.left: titleLabel.right
                anchors.leftMargin: theme.space(3)
                anchors.right: tabSeg.visible ? tabSeg.left : refreshBtn.visible ? refreshBtn.left : parent.right
                anchors.rightMargin: theme.space(4)
                anchors.baseline: titleLabel.baseline
                text: page.summary
                role: "ui"
                color: theme.textDim
                elide: Text.ElideRight
            }
            Seg {
                id: tabSeg
                objectName: "watching-tab"
                visible: page.anilistConnected
                anchors.right: refreshBtn.visible ? refreshBtn.left : parent.right
                anchors.rightMargin: theme.space(4)
                anchors.verticalCenter: parent.verticalCenter
                options: [{ text: "Watching", name: "watching", count: page.entries.length },
                          { text: "Planning", name: "planning", count: page.planned.length }]
                index: page.tab === "planning" ? 1 : 0
                onPicked: function(i) { page.pickTab(i === 1 ? "planning" : "watching") }
            }
            Button {
                id: refreshBtn
                objectName: "watching-refresh"
                visible: page.anilistConnected
                anchors.right: parent.right
                anchors.verticalCenter: parent.verticalCenter
                text: page.refreshing ? "Refreshing" : "Refresh"
                icon: "refresh-cw"
                busy: page.refreshing
                flat: true
                onClicked: if (!page.refreshing) page.refresh(true)
            }
        }

        // ---- The sections: a heading in the display cut with its count and one fact,
        // over a grid of uniform cards
        Repeater {
            model: page.sections
            Column {
                id: section
                required property var modelData
                required property int index
                objectName: "watching-section-" + String(modelData.title).toLowerCase().replace(/[^a-z0-9]+/g, "-")
                width: parent.width
                spacing: theme.space(3)
                topPadding: index > 0 ? theme.space(2) : 0
                Item {
                    width: parent.width
                    height: theme.space(12)
                    Row {
                        anchors.left: parent.left
                        anchors.bottom: parent.bottom
                        anchors.bottomMargin: theme.space(2)
                        spacing: theme.space(3)
                        Label {
                            id: sectionTitle
                            text: section.modelData.title
                            role: "display"
                            font.pointSize: theme.typeHeading
                            font.variableAxes: ({ "wdth": 70, "wght": 420 })
                            color: section.modelData.accent ? theme.accent : theme.text
                        }
                        Label { anchors.baseline: sectionTitle.baseline; text: section.modelData.cards.length; role: "ledger"; color: theme.textFaint }
                        Label { anchors.baseline: sectionTitle.baseline; text: section.modelData.fact; role: "ui"; color: theme.textDim }
                    }
                }
                Flow {
                    width: parent.width
                    spacing: page.gap
                    Repeater {
                        model: section.modelData.cards
                        LazyTile {
                            id: tile
                            required property var modelData
                            width: page.cardWidth
                            height: Math.round(page.cardWidth * 1.5)
                            sourceComponent: Card {
                                id: watchCard
                                objectName: "tile-" + tile.modelData.id
                                item: tile.modelData.item
                                metaOverride: tile.modelData.meta
                                ghost: tile.modelData.ghost
                                posterWidth: page.cardWidth
                                nowMs: page.nowMs
                                onOpened: tile.modelData.planning ? page.openPlanning(tile.modelData, watchCard) : page.open(tile.modelData)
                                // A show the library does not have has nothing to play: Space opens it.
                                // On Planning, Space is Start watching, or Get episodes off disk
                                onPlayed: tile.modelData.planning ? (tile.modelData.ghost ? page.getEpisodes(tile.modelData.raw) : page.startWatching(tile.modelData.raw))
                                          : tile.modelData.ghost ? page.open(tile.modelData) : page.playSeries(tile.modelData.raw.owned, tile.modelData.raw.title)
                            }
                        }
                    }
                }
            }
        }

        // ---- No list to show: why, and what to do, in a plain box in the column
        Item {
            visible: page.sections.length === 0
            width: parent.width
            height: visible ? theme.space(60) : 0
            EmptyState {
                objectName: "watching-connect-tile"
                visible: !page.anilistConnected
                icon: "log-in"
                title: "AniList is not connected"
                body: "Connect it and this page lists the shows you are watching or rewatching on AniList, with Play on the ones you have and a countdown on the ones airing."
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
            EmptyState {
                visible: page.anilistConnected && page.loadFailed
                icon: "triangle-alert"
                title: "The watching list did not load"
                body: "AniList could not be reached. The list comes back on the next refresh."
                Button { text: "Try again"; icon: "refresh-cw"; onClicked: { page.reload(); page.refresh(true) } }
            }
            EmptyState {
                objectName: "planning-failed"
                visible: page.anilistConnected && !page.loadFailed && page.tab === "planning" && page.planningFailed
                icon: "triangle-alert"
                title: "The planning list did not load"
                body: "Its cache could not be read. Try again, or it comes back on the next refresh."
                Button { text: "Try again"; icon: "refresh-cw"; onClicked: page.readPlanning() }
            }
            EmptyState {
                objectName: "planning-empty"
                visible: page.anilistConnected && !page.loadFailed && page.tab === "planning" && !page.planningFailed
                icon: "bookmark"
                title: "Nothing on your planning list"
                body: "Plan to watch on Airing now, or Planning in a show's list menu, puts a show here."
                Button { text: "Airing now"; icon: "radio"; flat: true; onClicked: frame.go("airing") }
            }
            EmptyState {
                visible: page.anilistConnected && !page.loadFailed && page.tab !== "planning"
                icon: "eye"
                title: "Nothing on your watching list"
                body: "Mark a show as Watching on AniList and it lands here."
                Button { text: "Open AniList"; icon: "external-link"; flat: true; onClicked: frame.openLink("https://anilist.co/home") }
            }
        }
    }
}
