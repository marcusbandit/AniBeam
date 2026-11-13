// Airing now (#202): every show AniList has on air, not only the library's, as one page of
// uniform cards, most popular first. The core owns the season (ListAiringNow answers the
// cached read at once; RefreshAiringNow goes to AniList on open when that read is over half
// an hour old, and from Refresh). Along the top, Out now: the episodes that came out in the
// last 24 hours across everything on air, newest first. A seg picks Airing now or the next
// season, another narrows the cards to the user's lists or the library.
//
// The x on a card marks the show not interested: it leaves the page at once, with Undo in a
// bar at the foot. One AniList entry and nothing related to it, so a new season of the same
// show comes back on its own. Settings, Library lists the marked shows with a way back.
// A click on a card opens what can be done with it: the series when the library has it,
// Plan to watch, Get episodes, AniList, Not interested.
//
// Several at once: press and hold a card, or Ctrl+click it, and the page starts picking.
// Every click then picks or unpicks, and a bar at the foot offers the two things that make
// sense for many shows, Plan to watch and Not interested. Escape, Cancel or unpicking the
// last one ends it.
//
// No poll. The page re-reads the cache (never AniList) when the next episode it knows of is
// due, so that episode moves into Out now on time, and a half-minute clock keeps "2h ago"
// honest, the way Watching keeps its countdowns.
import QtQuick
import QtQuick.Shapes
import QtQuick.Window
import QtCore
import com.marcusrosado.AniBeam

Item {
    id: page
    objectName: "airing-page"
    property var props: ({})
    property string title: words.t("airing-title")
    property alias scrollY: scroll.contentY
    function escapePressed() {
        if (page.selecting) { page.clearPicks(); return true }
        if (page.undo) { page.undo = null; return true }
        return false
    }

    // "Now", "Next", or a season and its year ("Spring 2019"), as the bridge spells SeasonPick;
    // kept in the page's props, so the rail brings the page back on the season and filter it
    // was left on
    property string season: props.season || "Now"
    // 0 everything, 1 on my lists, 2 in my library
    property int filter: props.filter || 0
    // What to leave out, on top of the filter: the shows on the user's lists, the library's
    property bool hideListed: !!props.hideListed
    property bool hideOwned: !!props.hideOwned
    // The season grid's order, any field AniList gives a show. The pick and its direction are
    // kept across launches; the value is what ascending runs over, `asc` the direction a field
    // starts in and `words` what the direction button says for ascending and descending, each
    // by message ID.
    readonly property var sorts: [
        { key: "popular", label: "airing-sort-popular", asc: true, words: ["airing-dir-most-popular", "airing-dir-least-popular"] },
        { key: "score", label: "airing-sort-score", asc: false, words: ["airing-dir-lowest", "airing-dir-highest"] },
        { key: "trending", label: "airing-sort-trending", asc: false, words: ["airing-dir-least-trending", "airing-dir-most-trending"] },
        { key: "favourites", label: "airing-sort-favourites", asc: false, words: ["airing-dir-fewest", "airing-dir-most"] },
        { key: "soonest", label: "airing-sort-soonest", asc: true, words: ["airing-dir-soonest", "airing-dir-latest"] },
        { key: "aired", label: "airing-sort-aired", asc: false, words: ["airing-dir-fewest", "airing-dir-most"] },
        { key: "episodes", label: "airing-sort-episodes", asc: false, words: ["airing-dir-fewest", "airing-dir-most"] },
        { key: "start", label: "airing-sort-start", asc: false, words: ["airing-dir-oldest", "airing-dir-newest"] },
        { key: "duration", label: "airing-sort-duration", asc: false, words: ["airing-dir-shortest", "airing-dir-longest"] },
        { key: "title", label: "airing-sort-title", asc: true, words: ["airing-dir-a-to-z", "airing-dir-z-to-a"] }]
    function sortIndex(key) {
        for (var i = 0; i < page.sorts.length; i++) if (page.sorts[i].key === key) return i
        return 0
    }
    readonly property int sortAt: page.sortIndex(sortPref.sortKey)
    readonly property bool sortAsc: sortPref.sortAsc
    function pickSort(i) {
        sortPref.sortKey = page.sorts[i].key
        sortPref.sortAsc = page.sorts[i].asc
        scroll.contentY = 0
    }
    function flipSort() { sortPref.sortAsc = !sortPref.sortAsc }
    // What a show is ordered by, or null when AniList gave none: those sort last either way
    function sortValue(key, s, i) {
        switch (key) {
        case "popular": return i
        case "score": return s.average_score
        case "trending": return s.trending
        case "favourites": return s.favourites
        case "soonest": return s.next_airing ? s.next_airing.at : null
        // The last episode out: the newest one seen airing, else the one before the next, else
        // all of them for a show with none due and none just out
        case "aired": return s.latest ? s.latest.episode : s.next_airing ? s.next_airing.episode - 1 : (s.episodes || null)
        case "episodes": return s.episodes
        case "start": return s.start_date
        case "duration": return s.duration
        default: return String(s.title).toLocaleLowerCase()
        }
    }
    function keepView() {
        frame.nav.current.props = { season: page.season, filter: page.filter, hideListed: page.hideListed, hideOwned: page.hideOwned }
    }
    property var listed: null
    property bool refreshing: false
    // The season read this page waits on: events of the other season's read are not its own
    property int refreshJob: -1
    property bool loadFailed: false
    property real nowMs: Date.now()
    // The shows just marked, while their Undo is up: { ids, text }
    property var undo: null
    // The cards picked, by AniList id; picking runs while any is
    property var picked: ({})
    readonly property int pickedCount: Object.keys(picked).length
    readonly property bool selecting: pickedCount > 0
    function togglePick(s) {
        var next = Object.assign({}, page.picked)
        if (next[s.anilist_id]) delete next[s.anilist_id]
        else next[s.anilist_id] = s
        page.picked = next
    }
    function clearPicks() { page.picked = ({}) }
    function pickedShows() {
        var out = []
        for (var k in page.picked) out.push(page.picked[k])
        return out
    }
    property bool allOut: false

    // A list status as the core names it, to its words by message ID
    readonly property var statusWords: ({ Watching: "status-watching", Repeating: "status-rewatching", Completed: "status-completed",
                                          Paused: "status-on-hold", Dropped: "status-dropped", Planning: "status-planning" })
    // The season after this one, by AniList's quarters, for the seg before its first read. A
    // code as the bridge spells it ("Spring 2027"); seasonText() gives its words
    readonly property string nextLabel: {
        var d = new Date(nowMs), q = Math.floor(d.getUTCMonth() / 3)
        return ["Winter", "Spring", "Summer", "Fall"][(q + 1) % 4] + " " + (d.getUTCFullYear() + (q === 3 ? 1 : 0))   // i18n: none
    }
    // A season code ("Spring 2027") in words
    function seasonText(code) {
        var parts = String(code).split(" ")
        return parts.length === 2 ? words.t("airing-season", { season: parts[0], year: parts[1] }) : String(code)
    }
    readonly property bool trackerConnected: !!(Door.trackers && ((Door.trackers.anilist && Door.trackers.anilist.connected)
                                                                  || (Door.trackers.mal && Door.trackers.mal.connected)))

    function owned(s) { return s.owned !== null && s.owned !== undefined }
    function fresh(s) { return !!s.latest && nowMs / 1000 - s.latest.at < 86400 }
    function ago(at) {
        var secs = Math.max(0, nowMs / 1000 - at)
        if (secs < 3600) return words.t("airing-ago-minutes", { minutes: Math.max(1, Math.floor(secs / 60)) })
        if (secs < 86400) return words.t("airing-ago-hours", { hours: Math.floor(secs / 3600) })
        return words.t("airing-ago-days", { days: Math.floor(secs / 86400) })
    }
    // The ledger line: an episode out in the last day says so; else the countdown runs (an
    // empty override); else what kind of show it is
    function metaOf(s) {
        if (fresh(s)) return words.t("airing-episode-out", { episode: Fmt.episodeName("", s.latest.episode), ago: ago(s.latest.at) })
        if (s.next_airing) return ""
        if (s.latest) return words.t("airing-episode-out", { episode: Fmt.episodeName("", s.latest.episode), ago: ago(s.latest.at) })
        return s.episodes ? words.t("count-episodes", { count: s.episodes }) : words.t("airing-no-date")
    }
    function cardItem(s) {
        return {
            id: s.anilist_id,
            title: s.title,
            poster: s.poster || "",
            watched: null,
            total_episodes: s.episodes || null,
            total_is_estimate: false,
            // The accent New tag: an episode out in the last day
            watched_state: fresh(s) ? "Behind" : "Unknown",
            strip: null,
            code: s.list_status ? (page.statusWords[s.list_status] ? words.t(page.statusWords[s.list_status]) : "") : owned(s) ? words.t("airing-in-library") : "",
            community_score: s.average_score !== null && s.average_score !== undefined ? s.average_score / 10 : null,
            my_score: null,
            hidden: false,
            next_airing: s.next_airing || null,
            episodes_on_disk: 0,
            list_status: null,
            site_url: ""
        }
    }

    // The season control's second cell: the season showing when one was picked, else the next
    readonly property string seasonLabel: page.season === "Now" || page.season === "Next" ? page.nextLabel : page.season
    function kept(s) {
        if (page.hideListed && !!s.list_status) return false
        if (page.hideOwned && page.owned(s)) return false
        return true
    }
    readonly property bool narrowed: page.filter !== 0 || page.hideListed || page.hideOwned
    // What the filter alone holds, before the hides: tells the empty state which one emptied it
    readonly property int filterCount: page.filter === 1 ? listedRaw : page.filter === 2 ? ownedRaw : (listed ? listed.shows.length : 0)
    readonly property int listedRaw: listed ? listed.shows.filter(function(s) { return !!s.list_status }).length : 0
    readonly property int ownedRaw: listed ? listed.shows.filter(function(s) { return page.owned(s) }).length : 0
    readonly property bool hidesEmptied: page.filterCount > 0
    readonly property var shows: {
        if (!listed) return []
        var f = page.filter
        var out = listed.shows.filter(function(s) {
            if (f === 1 && !s.list_status) return false
            if (f === 2 && !page.owned(s)) return false
            return page.kept(s)
        })
        // A show missing the value sorts after every show that has it, whichever way the order
        // runs; ties keep AniList's popularity order
        var key = page.sorts[page.sortAt].key
        if (key === "popular" && page.sortAsc) return out
        var dir = page.sortAsc ? 1 : -1
        var rows = out.map(function(s, i) {
            var v = page.sortValue(key, s, i)
            return { s: s, k: v === undefined ? null : v, i: i }
        })
        rows.sort(function(a, b) {
            if (a.k === null || b.k === null) return a.k === b.k ? a.i - b.i : a.k === null ? 1 : -1
            if (a.k < b.k) return -dir
            if (a.k > b.k) return dir
            return a.i - b.i
        })
        return rows.map(function(r) { return r.s })
    }
    // The strip follows the narrowing too: On my lists means the episodes of those shows
    readonly property var aired: {
        if (page.season !== "Now" || !listed) return []
        if (!page.narrowed) return listed.aired
        // A filter keeps only its shows' episodes; a hide drops only the episodes of the shows
        // it hides, so a show that just finished, and is no longer listed, stays
        if (page.filter !== 0) {
            var keep = {}
            for (var i = 0; i < page.shows.length; i++) keep[page.shows[i].anilist_id] = true
            return listed.aired.filter(function(a) { return keep[a.anilist_id] === true })
        }
        var drop = {}
        for (var j = 0; j < listed.shows.length; j++) if (!page.kept(listed.shows[j])) drop[listed.shows[j].anilist_id] = true
        return listed.aired.filter(function(a) { return drop[a.anilist_id] !== true })
    }
    // Out now's grid, small cards or list rows by Settings, Appearance, until All opens the rest
    readonly property real outGap: outList ? theme.space(2) : theme.space(3)
    Settings { id: sortPref; location: Shell.settingsUrl; category: "airing"; property string sortKey: "popular"; property bool sortAsc: true }
    // Cards come columns by rows, from Settings, Appearance. The grid wins over the image: a
    // banner is cropped to its cell, and a column goes only when a cell would drop under
    // outMinCard, too small to read a title in
    readonly property real outMinCard: theme.space(32)
    Settings { id: kept; location: Shell.settingsUrl; category: "airing"; property string outLayout: "4x2"; property string outStyle: "cards" }
    readonly property var outLayout: {
        var m = /^(\d)x(\d)$/.exec(kept.outLayout)
        return m ? [parseInt(m[1]), parseInt(m[2])] : [4, 2]
    }
    // Cards unless Settings picks the list: rows of a small poster, title, episode and age,
    // as many columns as fit at a readable title width, three rows deep
    readonly property bool outList: kept.outStyle === "list"
    readonly property int outRows: outList ? 3 : outLayout[1]
    readonly property int outCols: outList
        ? Math.max(1, Math.floor((scroll.innerWidth + outGap) / (theme.space(84) + outGap)))
        : Math.max(1, Math.min(outLayout[0], Math.floor((scroll.innerWidth + outGap) / (outMinCard + outGap))))
    // Not floored: the cells then sum to the full width, so the grid ends where the posters do
    readonly property real outCellWidth: (scroll.innerWidth - outGap * (outCols - 1)) / outCols
    // Tall enough for two lines of words, and close to a banner's wide shape as the tile narrows
    readonly property real outCellHeight: outList ? theme.space(17) : Math.max(theme.space(20), Math.round(outCellWidth / 3.4))
    readonly property int outFirst: outCols * outRows
    readonly property var outShown: page.allOut ? page.aired : page.aired.slice(0, page.outFirst)
    readonly property int outToday: {
        var n = 0
        for (var i = 0; i < aired.length; i++) if (nowMs / 1000 - aired[i].at < 86400) n++
        return n
    }
    // How many each filter holds with the hides applied, for its pill
    readonly property int allCount: listed ? listed.shows.filter(function(s) { return page.kept(s) }).length : 0
    readonly property int listedCount: listed ? listed.shows.filter(function(s) { return !!s.list_status && page.kept(s) }).length : 0
    readonly property int ownedCount: listed ? listed.shows.filter(function(s) { return page.owned(s) && page.kept(s) }).length : 0
    // The line says what the pills do not: how many came out today (the show count is on the
    // filter pill already)
    readonly property string summary: {
        if (!listed || page.shows.length === 0 || page.season !== "Now" || outToday === 0) return ""
        return words.t("airing-out-today", { count: outToday })
    }
    function showOf(id) {
        if (!listed) return null
        for (var i = 0; i < listed.shows.length; i++) if (listed.shows[i].anilist_id === id) return listed.shows[i]
        return null
    }

    function reload() {
        var r = Door.listAiringNow(page.season)
        if (r.error) { page.loadFailed = !page.listed; frame.fail(r.error, page.reload); return }
        page.loadFailed = false
        page.nowMs = Date.now()
        page.listed = r.reply.now
        if (r.reply.refreshing !== null && r.reply.refreshing !== undefined) { page.refreshing = true; page.refreshJob = r.reply.refreshing }
        page.armDue()
    }
    // To AniList: `force` from Refresh, else only when the cached read is stale
    property int refreshAsked: 0   // for a --drive eval: the open asked once
    function refresh(force) {
        refreshAsked++
        var r = Door.refreshAiringNow(page.season, force === true)
        if (r.error) { page.loadFailed = !page.listed || page.listed.shows.length === 0; frame.fail(r.error, function() { page.refresh(force) }); return }
        if (r.reply && r.reply.job !== undefined) { page.refreshing = true; page.refreshJob = r.reply.job }
    }
    function pickSeason(s) {
        // The next season by name is the next season
        if (s === page.nextLabel) s = "Next"
        if (s === page.season) return
        page.season = s
        page.keepView()
        page.allOut = false
        page.refreshing = false
        page.refreshJob = -1
        page.listed = null
        page.clearPicks()
        page.reload()
        page.refresh(false)
        scroll.contentY = 0
    }
    // The next episode due among the cards: when it comes, the cache read moves it to Out now
    function armDue() {
        var soonest = 0, now = Date.now() / 1000
        var list = listed ? listed.shows : []
        for (var i = 0; i < list.length; i++) {
            var n = list[i].next_airing
            if (n && n.at > now && (soonest === 0 || n.at < soonest)) soonest = n.at
        }
        if (soonest === 0) { dueTimer.stop(); return }
        dueTimer.interval = Math.min(2147483000, Math.ceil((soonest - now) * 1000) + 1500)
        dueTimer.restart()
    }

    // ---- What a card can do
    function dismiss(s) { page.dismissAll([s]) }
    function dismissAll(list) {
        var ids = []
        for (var i = 0; i < list.length; i++) {
            var r = Door.setNotInterested(list[i].anilist_id, true)
            if (r.error) { frame.fail(r.error); break }
            ids.push(list[i].anilist_id)
        }
        if (ids.length === 0) return
        page.undo = { ids: ids, text: ids.length === 1 ? words.t("airing-dismissed-one", { title: list[0].title })
                                                      : words.t("airing-dismissed", { count: ids.length }) }
        undoTimer.restart()
        page.reload()
    }
    function takeBack() {
        if (!page.undo) return
        for (var i = 0; i < page.undo.ids.length; i++) {
            var r = Door.setNotInterested(page.undo.ids[i], false)
            if (r.error) { frame.fail(r.error); return }
        }
        page.undo = null
        page.reload()
    }
    function plan(s) { page.planAll([s]) }
    function planAll(list) {
        if (!page.trackerConnected) { frame.openConnect("Anilist"); return }
        var sent = 0, already = 0
        for (var i = 0; i < list.length; i++) {
            if (list[i].list_status) { already++; continue }
            var r = Door.planToWatch(list[i].anilist_id)
            if (r.error) { frame.fail(r.error); return }
            if (r.reply && r.reply.job !== undefined) sent++
            else already++
        }
        if (list.length === 1) { if (sent === 0) frame.toast(words.t("airing-planned-already-one", { title: list[0].title })); return }
        frame.toast(sent === 0 ? words.t("airing-planned-already-all")
                               : already > 0 ? words.t("airing-planned-some-already", { count: sent, already: already })
                               : words.t("airing-planned", { count: sent }))
    }
    // The bar's two actions: they act on every card picked, then picking ends
    function planPicked() { var l = page.pickedShows(); page.clearPicks(); page.planAll(l) }
    function dismissPicked() { var l = page.pickedShows(); page.clearPicks(); page.dismissAll(l) }
    function getEpisodes(s) {
        if (page.owned(s)) { frame.go("downloads", { series: s.owned }); return }
        frame.go("downloads", { hit: { id: s.anilist_id, title: s.title, romaji: s.romaji || "", english: s.english || "", native: s.native || "",
                                       cover_url: s.poster || "", episodes: s.episodes || 0, format: s.format || "", year: 0 } })
    }
    function siteOf(s) { return s.site_url || "https://anilist.co/anime/" + s.anilist_id }
    function actionsFor(s) {
        var items = []
        if (page.owned(s)) items.push({ text: words.t("airing-play-next"), icon: "play", action: function() { frame.playNext(s.owned, s.title) } })
        if (page.owned(s)) items.push({ text: words.t("action-open-series"), icon: "layers", action: function() { frame.go("series", { id: s.owned }, s.title) } })
        if (!s.list_status) items.push({ text: words.t("airing-plan-to-watch"), icon: "bookmark", action: function() { page.plan(s) } })
        items.push({ text: page.owned(s) ? words.t("airing-get-more-episodes") : words.t("airing-get-episodes"), icon: "download", action: function() { page.getEpisodes(s) } })
        items.push({ text: words.t("airing-open-on-anilist"), icon: "external-link", action: function() { frame.openLink(page.siteOf(s)) } })
        items.push({ text: words.t("airing-not-interested"), icon: "eye-off", action: function() { page.dismiss(s) } })
        return items
    }
    function openActions(s, anchor) {
        var p = anchor.mapToItem(frame, theme.space(3), anchor.height * 0.55)
        frame.openMenu(p.x, p.y, page.actionsFor(s))
    }

    Timer { id: debounce; interval: 400; onTriggered: page.reload() }
    Timer { id: dueTimer; onTriggered: page.reload() }
    Timer { interval: 30000; running: !!page.listed; repeat: true; onTriggered: page.nowMs = Date.now() }
    Timer { id: undoTimer; interval: 8000; onTriggered: page.undo = null }
    Connections {
        target: Door
        function onAiringNowRefreshed(season) {
            if (season !== page.season) return
            page.refreshing = false
            page.refreshJob = -1
            page.loadFailed = false
            page.reload()
        }
        function onJobFailed(job, kind, error) {
            if (job !== page.refreshJob) return
            page.refreshing = false
            page.refreshJob = -1
            page.loadFailed = !page.listed || page.listed.shows.length === 0
            frame.fail(error, function() { page.refresh(true) })
        }
        function onJobCancelled(job, kind) { if (job === page.refreshJob) { page.refreshing = false; page.refreshJob = -1 } }
        // Plan to watch lands, a download makes a show the library's: the cache read says so
        function onJobFinished(job, kind, ok) { if (kind === "SetStatus" || kind === "Scan") debounce.restart() }
        function onTrackersChanged() { debounce.restart() }
    }
    Component.onCompleted: { reload(); refresh(false) }

    // ---- The grid's geometry: as many columns as the width holds at the poster width
    readonly property real gap: theme.space(4)
    readonly property int cols: Math.max(1, Math.floor((scroll.innerWidth + gap) / (theme.posterWidth + gap)))
    readonly property real cardWidth: (scroll.innerWidth - gap * (cols - 1)) / cols

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
                text: page.season === "Now" ? words.t("airing-title") : page.seasonText(page.seasonLabel)
                role: "display"
                font.pointSize: theme.typePage
            }
            Label {
                objectName: "airing-summary"
                anchors.left: titleLabel.right
                anchors.leftMargin: theme.space(3)
                anchors.right: notInterested.left
                anchors.rightMargin: theme.space(4)
                anchors.baseline: titleLabel.baseline
                text: page.summary
                role: "ui"
                color: theme.textDim
                elide: Text.ElideRight
            }
            Button {
                id: notInterested
                objectName: "airing-not-interested"
                visible: !!page.listed && page.listed.not_interested > 0
                width: visible ? implicitWidth : 0
                anchors.right: refreshBtn.left
                anchors.rightMargin: visible ? theme.space(2) : 0
                anchors.verticalCenter: parent.verticalCenter
                flat: true
                icon: "eye-off"
                text: page.listed ? words.t("airing-not-interested-count", { count: page.listed.not_interested }) : ""
                onClicked: frame.go("settings", { tab: "library", focus: "not-interested" })
                Tooltip { msg: "airing-not-interested-tip" }
            }
            Button {
                id: refreshBtn
                objectName: "airing-refresh"
                anchors.right: parent.right
                anchors.verticalCenter: parent.verticalCenter
                text: page.refreshing ? words.t("airing-refreshing") : words.t("action-refresh")
                icon: "refresh-cw"
                busy: page.refreshing
                flat: true
                onClicked: page.refresh(true)
            }
        }

        // ---- The season, the filter, and what to leave out
        Item {
            width: parent.width
            height: theme.controlHeight + theme.space(2)
            Row {
                anchors.left: parent.left
                anchors.verticalCenter: parent.verticalCenter
                spacing: theme.space(3)
                Seg {
                    id: seasonSeg
                    objectName: "airing-season"
                    // The third cell opens every season; it is never the one showing
                    options: [{ text: words.t("airing-title"), icon: "radio", name: "now" },
                              { text: page.seasonText(page.seasonLabel), icon: "calendar-clock", name: "next" },
                              { icon: "chevron-down", name: "seasons", tip: words.t("airing-pick-season") }]
                    index: page.season === "Now" ? 0 : 1
                    onPicked: function(i) {
                        if (i === 0) page.pickSeason("Now")
                        else if (i === 1) { if (page.season === "Now") page.pickSeason("Next") }
                        else picker.open()
                    }
                    SeasonPicker {
                        id: picker
                        anchors.fill: parent
                        current: page.season === "Now" ? "" : page.seasonLabel
                        nowMs: page.nowMs
                        onPicked: function(name, year) { page.pickSeason(name + " " + year) }
                    }
                }
                Seg {
                    objectName: "airing-filter"
                    options: [{ text: words.t("airing-filter-all"), name: "all", count: page.listed ? page.allCount : undefined },
                              { text: words.t("airing-filter-lists"), name: "lists", count: page.listed ? page.listedCount : undefined },
                              { text: words.t("airing-filter-library"), name: "library", count: page.listed ? page.ownedCount : undefined }]
                    index: page.filter
                    onPicked: function(i) {
                        page.filter = i
                        // A filter to the very shows a hide leaves out takes that hide off
                        if (i === 1) page.hideListed = false
                        if (i === 2) page.hideOwned = false
                        page.keepView()
                        page.clearPicks()
                    }
                }
            }
            Row {
                anchors.right: parent.right
                anchors.verticalCenter: parent.verticalCenter
                spacing: theme.space(2)
                ToggleChip {
                    objectName: "airing-hide-lists"
                    msg: "airing-hide-lists"
                    icon: "eye-off"
                    checked: page.hideListed
                    // On my lists with them hidden would be nothing at all
                    interactive: page.filter !== 1
                    onToggled: function(on) { page.hideListed = on; page.keepView(); page.clearPicks() }
                    Tooltip { msg: "airing-hide-lists-tip" }
                }
                ToggleChip {
                    objectName: "airing-hide-library"
                    msg: "airing-hide-library"
                    icon: "eye-off"
                    checked: page.hideOwned
                    interactive: page.filter !== 2
                    onToggled: function(on) { page.hideOwned = on; page.keepView(); page.clearPicks() }
                    Tooltip { msg: "airing-hide-library-tip" }
                }
            }
        }

        // ---- Out now: the episodes of the last 24 hours, newest first, as an even grid of
        // rows (thumb, title, episode and age), three rows deep until All opens the rest
        Column {
            objectName: "airing-out-now"
            visible: page.aired.length > 0
            width: parent.width
            spacing: theme.space(3)
            topPadding: theme.space(2)
            bottomPadding: theme.space(6)
            Item {
                width: parent.width
                height: outTitle.implicitHeight
                Row {
                    spacing: theme.space(3)
                    Label {
                        id: outTitle
                        msg: "airing-out-now"
                        role: "display"
                        font.pointSize: theme.typeHeading
                        font.variableAxes: ({ "wdth": 70, "wght": 420 })
                        color: theme.accent
                    }
                    Label {
                        anchors.baseline: outTitle.baseline
                        msg: "airing-out-last-day"; msgArgs: ({ count: page.aired.length })
                        role: "ui"
                        color: theme.textDim
                    }
                }
                Button {
                    objectName: "airing-out-all"
                    visible: page.aired.length > page.outFirst
                    anchors.right: parent.right
                    anchors.verticalCenter: parent.verticalCenter
                    small: true
                    flat: true
                    icon: page.allOut ? "chevron-up" : "chevron-down"
                    text: page.allOut ? words.t("airing-out-fewer") : words.t("airing-out-all", { count: page.aired.length })
                    onClicked: page.allOut = !page.allOut
                }
            }
            Flow {
                // Half a pixel over, so a row of exact cells never wraps its last one
                width: parent.width + 0.5
                spacing: page.outGap
                Repeater {
                    model: page.outShown
                    // One episode, as a small card or a list row
                    Item {
                        id: pill
                        required property var modelData
                        readonly property var show: page.showOf(modelData.anilist_id)
                        readonly property bool today: page.nowMs / 1000 - modelData.at < 86400
                        objectName: "airing-out-" + modelData.anilist_id + "-" + modelData.episode
                        width: page.outCellWidth
                        height: page.outCellHeight
                        activeFocusOnTab: true
                        scale: theme.reducedMotion || page.outList ? 1 : (pm.pressed ? 0.985 : 1)
                        function open() {
                            if (pill.show) page.openActions(pill.show, pill)
                            else frame.openLink("https://anilist.co/anime/" + pill.modelData.anilist_id)
                        }
                        Keys.onReturnPressed: pill.open()
                        // One look per pill, built only for the style Settings picked
                        Loader {
                            anchors.fill: parent
                            sourceComponent: page.outList ? outRow : outCard
                        }
                        // The card: the banner across the whole tile, rounded like the cards
                        // below, darkened toward the foot so the words read
                        Component {
                            id: outCard
                            Item {
                                Image {
                                    id: outPoster
                                    visible: false
                                    width: pill.width; height: pill.height
                                    // AniList's wide banner, made for this shape; the tall cover only
                                    // when a show has none
                                    source: !pill.show ? "" : pill.show.banner ? Fmt.fileUrl(pill.show.banner)
                                          : pill.show.poster ? Fmt.fileUrl(pill.show.poster) : ""
                                    // Decoded at the tile's width, its height following the image's own shape
                                    sourceSize.width: Math.ceil(pill.width * (Screen.devicePixelRatio || 1))
                                    fillMode: Image.PreserveAspectCrop
                                    smooth: true
                                    asynchronous: true
                                    cache: true
                                }
                                Corner {
                                    anchors.fill: parent
                                    radius: theme.radiusTile
                                    smoothing: theme.cornerSmoothing
                                    color: theme.tileSunken
                                    fillItem: outPoster.status === Image.Ready ? outPoster : null
                                }
                                // The shade: light at the top, near black under the words
                                Corner {
                                    anchors.fill: parent
                                    radius: theme.radiusTile
                                    smoothing: theme.cornerSmoothing
                                    // No outline: on a light banner it read as a gap round the tile.
                                    // Only keyboard focus draws a ring; hover lifts the shade instead
                                    borderColor: pill.activeFocus ? theme.focusRing : "transparent"
                                    borderWidth: pill.activeFocus ? 2 : 0
                                    fillGradient: LinearGradient {
                                        x1: 0; y1: 0; x2: 0; y2: pill.height
                                        GradientStop { position: 0; color: Qt.rgba(0, 0, 0, pm.containsMouse ? 0.05 : 0.2) }
                                        GradientStop { position: 0.45; color: Qt.rgba(0, 0, 0, pm.containsMouse ? 0.35 : 0.5) }
                                        GradientStop { position: 1; color: Qt.rgba(0, 0, 0, 0.88) }
                                    }
                                }
                                Column {
                                    anchors.left: parent.left
                                    anchors.right: parent.right
                                    anchors.bottom: parent.bottom
                                    anchors.margins: theme.space(3)
                                    spacing: theme.space(0.5)
                                    Label {
                                        width: parent.width
                                        text: pill.modelData.title
                                        role: "ui"
                                        color: "white"
                                        elide: Text.ElideRight
                                    }
                                    Row {
                                        spacing: theme.space(2)
                                        Label { text: Fmt.episodeName("", pill.modelData.episode); role: "ledger"; color: Qt.rgba(1, 1, 1, 0.72) }
                                        Label { text: page.ago(pill.modelData.at); role: "ledger"; color: pill.today ? theme.accent : Qt.rgba(1, 1, 1, 0.55) }
                                    }
                                }
                            }
                        }
                        // The list row: a small poster, then the title, episode and age
                        Component {
                            id: outRow
                            Item {

                                Rectangle {
                                    anchors.fill: parent
                                    radius: theme.radiusSm
                                    color: pm.containsMouse ? theme.raised : "transparent"
                                    border.color: pill.activeFocus ? theme.focusRing : "transparent"
                                    border.width: 1
                                }
                                Corner {
                                    id: thumb
                                    x: theme.space(1.5)
                                    anchors.verticalCenter: parent.verticalCenter
                                    height: parent.height - theme.space(3)
                                    width: Math.round(height / 1.5)
                                    radius: theme.radiusSm; smoothing: 0
                                    color: theme.tileSunken
                                    borderColor: theme.line; borderWidth: 1
                                    Image {
                                        id: thumbImage
                                        visible: false
                                        width: thumb.width; height: thumb.height
                                        source: pill.show && pill.show.poster ? Fmt.fileUrl(pill.show.poster) : ""
                                        sourceSize: theme.posterDecodeSize
                                        fillMode: Image.PreserveAspectCrop
                                        smooth: true
                                        asynchronous: true
                                        cache: true
                                    }
                                    fillItem: thumbImage.status === Image.Ready ? thumbImage : null
                                }
                                Column {
                                    anchors.left: thumb.right
                                    anchors.leftMargin: theme.space(3)
                                    anchors.right: parent.right
                                    anchors.rightMargin: theme.space(2)
                                    anchors.verticalCenter: parent.verticalCenter
                                    spacing: theme.space(1)
                                    Label {
                                        width: parent.width
                                        text: pill.modelData.title
                                        role: "ui"
                                        elide: Text.ElideRight
                                    }
                                    Row {
                                        spacing: theme.space(2)
                                        Label { text: Fmt.episodeName("", pill.modelData.episode); role: "ledger"; color: theme.textDim }
                                        Label { text: page.ago(pill.modelData.at); role: "ledger"; color: pill.today ? theme.accent : theme.textFaint }
                                    }
                                }
                            }
                        }
                        MouseArea { id: pm; anchors.fill: parent; hoverEnabled: true; cursorShape: Qt.PointingHandCursor; onClicked: pill.open(); Accessible.role: Accessible.Button; Accessible.name: theme.nameNear(parent)
                            Accessible.onPressAction: pill.open() }
                    }
                }
            }
        }

        // ---- The season's order: sorts the grid below and nothing above it
        Item {
            objectName: "airing-sort-row"
            visible: page.shows.length > 0
            width: parent.width
            height: theme.controlHeight
            Row {
                spacing: theme.space(2)
                Dropdown {
                    objectName: "airing-sort"
                    prefix: words.t("airing-sort")
                    minWidth: theme.space(58)
                    options: page.sorts.map(function(s) { return words.t(s.label) })
                    index: page.sortAt
                    onPicked: function(i) { page.pickSort(i) }
                }
                Button {
                    objectName: "airing-sort-direction"
                    anchors.verticalCenter: parent.verticalCenter
                    width: Math.max(implicitWidth, theme.space(48))
                    flat: true
                    icon: page.sortAsc ? "arrow-up-narrow-wide" : "arrow-down-wide-narrow"
                    text: words.t(page.sorts[page.sortAt].words[page.sortAsc ? 0 : 1])
                    onClicked: page.flipSort()
                }
            }
            Label {
                anchors.right: parent.right
                anchors.verticalCenter: parent.verticalCenter
                msg: "count-shows"; msgArgs: ({ count: page.shows.length })
                role: "ui"
                color: theme.textDim
            }
        }

        // ---- The shows, in the order above
        Flow {
            objectName: "airing-grid"
            // Half a pixel over, so a row of exact cards never wraps its last one
            width: parent.width + 0.5
            spacing: page.gap
            Repeater {
                model: page.shows
                LazyTile {
                    id: tile
                    required property var modelData
                    width: page.cardWidth
                    height: Math.round(page.cardWidth * 1.5)
                    sourceComponent: Card {
                        id: card
                        objectName: "tile-" + tile.modelData.anilist_id
                        item: page.cardItem(tile.modelData)
                        metaOverride: page.metaOf(tile.modelData)
                        posterWidth: page.cardWidth
                        nowMs: page.nowMs
                        dismissible: true
                        selectable: true
                        selecting: page.selecting
                        selected: !!page.picked[tile.modelData.anilist_id]
                        onSelectToggled: page.togglePick(tile.modelData)
                        onOpened: page.openActions(tile.modelData, card)
                        onPlayed: page.openActions(tile.modelData, card)
                        onDismissed: page.dismiss(tile.modelData)
                    }
                }
            }
        }

        // ---- Nothing to show: why, and what to do
        Item {
            visible: page.shows.length === 0
            width: parent.width
            height: visible ? theme.space(60) : 0
            EmptyState {
                visible: page.loadFailed
                icon: "triangle-alert"
                title: words.t("airing-load-failed-title")
                body: words.t("airing-load-failed-body")
                Button { msg: "action-try-again"; icon: "refresh-cw"; onClicked: { page.reload(); page.refresh(true) } }
            }
            EmptyState {
                visible: !page.loadFailed && (!page.listed || page.listed.shows.length === 0)
                // Read and still empty is a season AniList lists nothing for, not a page never read
                readonly property bool readEmpty: !!page.listed && page.listed.fetched_at !== null && page.listed.fetched_at !== undefined
                icon: page.refreshing ? "loader-circle" : "radio"
                title: page.refreshing ? words.t("airing-reading-title")
                     : readEmpty ? (page.season === "Now" ? words.t("airing-nothing-now") : words.t("airing-nothing-for", { season: page.seasonText(page.listed.label) }))
                     : words.t("airing-nothing-read")
                body: page.refreshing ? words.t("airing-reading-body")
                    : readEmpty ? words.t("airing-nothing-body")
                    : words.t("airing-nothing-read-body")
                Button { visible: !page.refreshing; msg: "action-refresh"; icon: "refresh-cw"; onClicked: page.refresh(true) }
            }
            EmptyState {
                visible: !page.loadFailed && !!page.listed && page.listed.shows.length > 0 && page.narrowed
                icon: page.hidesEmptied ? "eye-off" : page.filter === 1 ? "bookmark" : "layers"
                title: page.hidesEmptied ? words.t("airing-hides-empty-title")
                     : page.filter === 1 ? words.t("airing-lists-empty-title")
                     : words.t("airing-library-empty-title")
                body: page.hidesEmptied ? words.t("airing-hides-empty-body")
                    : page.filter === 1 ? words.t("airing-lists-empty-body")
                    : words.t("airing-library-empty-body")
                Button {
                    msg: "airing-show-everything"
                    onClicked: { page.filter = 0; page.hideListed = false; page.hideOwned = false; page.keepView() }
                }
            }
        }
    }

    // ---- Picking: how many, and the two things to do with them
    Rectangle {
        objectName: "airing-picked"
        visible: page.selecting
        anchors.horizontalCenter: parent.horizontalCenter
        anchors.bottom: parent.bottom
        anchors.bottomMargin: theme.space(6)
        width: Math.min(parent.width - theme.space(8), pickRow.implicitWidth + theme.space(6))
        height: theme.controlHeight + theme.space(3)
        radius: theme.radiusTile
        color: theme.raised
        border.color: theme.accent
        border.width: 1
        Row {
            id: pickRow
            anchors.centerIn: parent
            spacing: theme.space(3)
            Label {
                anchors.verticalCenter: parent.verticalCenter
                msg: "airing-picked"; msgArgs: ({ count: page.pickedCount })
                role: "ui"
            }
            Button { objectName: "airing-picked-plan"; small: true; msg: "airing-plan-to-watch"; icon: "bookmark"; onClicked: page.planPicked() }
            Button { objectName: "airing-picked-dismiss"; small: true; msg: "airing-not-interested"; icon: "eye-off"; onClicked: page.dismissPicked() }
            Button { objectName: "airing-picked-cancel"; small: true; flat: true; msg: "action-cancel"; onClicked: page.clearPicks() }
        }
    }

    // ---- Undo, at the foot, for the shows just marked
    Rectangle {
        objectName: "airing-undo"
        visible: !!page.undo && !page.selecting
        anchors.horizontalCenter: parent.horizontalCenter
        anchors.bottom: parent.bottom
        anchors.bottomMargin: theme.space(6)
        width: Math.min(parent.width - theme.space(8), undoRow.implicitWidth + theme.space(6))
        height: theme.controlHeight + theme.space(3)
        radius: theme.radiusTile
        color: theme.raised
        border.color: theme.line
        border.width: 1
        Row {
            id: undoRow
            anchors.centerIn: parent
            spacing: theme.space(4)
            Label {
                anchors.verticalCenter: parent.verticalCenter
                width: Math.min(implicitWidth, theme.space(110))
                text: page.undo ? page.undo.text : ""
                role: "ui"
                elide: Text.ElideRight
            }
            Button { objectName: "airing-undo-button"; small: true; msg: "airing-undo"; icon: "rotate-ccw"; onClicked: page.takeBack() }
        }
    }
}
