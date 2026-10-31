// Airing now (#202): every show AniList has on air, not only the library's, as one page of
// uniform cards, most popular first. The core owns the season (ListAiringNow answers the
// cached read at once; RefreshAiringNow goes to AniList on open when that read is over half
// an hour old, and from Refresh). Along the top, Out now: the episodes that came out in the
// last two days across everything on air, newest first. A seg picks Airing now or the next
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
import com.marcusrosado.AniBeam

Item {
    id: page
    objectName: "airing-page"
    property var props: ({})
    property string title: "Airing now"
    property alias scrollY: scroll.contentY
    function escapePressed() {
        if (page.selecting) { page.clearPicks(); return true }
        if (page.undo) { page.undo = null; return true }
        return false
    }

    // "Now" or "Next", as the core spells SeasonPick; kept in the page's props, so the rail
    // brings the page back on the season and filter it was left on
    property string season: props.season || "Now"
    // 0 everything, 1 on my lists, 2 in my library
    property int filter: props.filter || 0
    function keepView() { frame.nav.current.props = { season: page.season, filter: page.filter } }
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

    readonly property var statusWords: ({ Watching: "Watching", Repeating: "Rewatching", Completed: "Completed",
                                          Paused: "On hold", Dropped: "Dropped", Planning: "Planning" })
    // The season after this one, by AniList's quarters, for the seg before its first read
    readonly property string nextLabel: {
        var d = new Date(nowMs), q = Math.floor(d.getUTCMonth() / 3)
        return ["Winter", "Spring", "Summer", "Fall"][(q + 1) % 4] + " " + (d.getUTCFullYear() + (q === 3 ? 1 : 0))
    }
    readonly property bool trackerConnected: !!(Door.trackers && ((Door.trackers.anilist && Door.trackers.anilist.connected)
                                                                  || (Door.trackers.mal && Door.trackers.mal.connected)))

    function owned(s) { return s.owned !== null && s.owned !== undefined }
    function fresh(s) { return !!s.latest && nowMs / 1000 - s.latest.at < 86400 }
    function ago(at) {
        var secs = Math.max(0, nowMs / 1000 - at)
        if (secs < 3600) return Math.max(1, Math.floor(secs / 60)) + "m ago"
        if (secs < 86400) return Math.floor(secs / 3600) + "h ago"
        return Math.floor(secs / 86400) + "d ago"
    }
    // The ledger line: an episode out in the last day says so; else the countdown runs (an
    // empty override); else what kind of show it is
    function metaOf(s) {
        if (fresh(s)) return Fmt.episodeName("", s.latest.episode) + " out " + ago(s.latest.at)
        if (s.next_airing) return ""
        if (s.latest) return Fmt.episodeName("", s.latest.episode) + " out " + ago(s.latest.at)
        return s.episodes ? Fmt.plural(s.episodes, "episode", "episodes") : "No date yet"
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
            code: s.list_status ? (page.statusWords[s.list_status] || "") : owned(s) ? "In library" : "",
            community_score: s.average_score !== null && s.average_score !== undefined ? s.average_score / 10 : null,
            my_score: null,
            hidden: false,
            next_airing: s.next_airing || null,
            episodes_on_disk: 0,
            list_status: null,
            site_url: ""
        }
    }

    readonly property var shows: {
        if (!listed) return []
        var f = page.filter
        return listed.shows.filter(function(s) {
            if (f === 1) return !!s.list_status
            if (f === 2) return page.owned(s)
            return true
        })
    }
    // The strip follows the narrowing too: On my lists means the episodes of those shows
    readonly property var aired: {
        if (page.season !== "Now" || !listed) return []
        if (page.filter === 0) return listed.aired
        var keep = {}
        for (var i = 0; i < page.shows.length; i++) keep[page.shows[i].anilist_id] = true
        return listed.aired.filter(function(a) { return keep[a.anilist_id] === true })
    }
    readonly property var outShown: page.allOut ? page.aired : page.aired.slice(0, 12)
    readonly property int outToday: {
        var n = 0
        for (var i = 0; i < aired.length; i++) if (nowMs / 1000 - aired[i].at < 86400) n++
        return n
    }
    // How many each filter holds, for its pill
    readonly property int listedCount: listed ? listed.shows.filter(function(s) { return !!s.list_status }).length : 0
    readonly property int ownedCount: listed ? listed.shows.filter(function(s) { return page.owned(s) }).length : 0
    // The line counts what the filter shows, not everything on air
    readonly property string summary: {
        if (!listed || page.shows.length === 0) return ""
        var line = Fmt.plural(page.shows.length, "show", "shows") + (page.season === "Now" ? " on air" : "")
        if (page.season === "Now" && outToday > 0) line += ", " + Fmt.plural(outToday, "episode", "episodes") + " out today"
        return line
    }
    function showOf(id) {
        if (!listed) return null
        for (var i = 0; i < listed.shows.length; i++) if (listed.shows[i].anilist_id === id) return listed.shows[i]
        return null
    }

    function reload() {
        var r = Door.listAiringNow(page.season)
        if (r.error) { page.loadFailed = !page.listed; frame.toast(r.error.message); return }
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
        if (r.error) { page.loadFailed = !page.listed || page.listed.shows.length === 0; frame.toast(r.error.message); return }
        if (r.reply && r.reply.job !== undefined) { page.refreshing = true; page.refreshJob = r.reply.job }
    }
    function pickSeason(s) {
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
            if (r.error) { frame.toast(r.error.message); break }
            ids.push(list[i].anilist_id)
        }
        if (ids.length === 0) return
        page.undo = { ids: ids, text: ids.length === 1 ? list[0].title + " won't show up here again"
                                                      : ids.length + " shows won't show up here again" }
        undoTimer.restart()
        page.reload()
    }
    function takeBack() {
        if (!page.undo) return
        for (var i = 0; i < page.undo.ids.length; i++) {
            var r = Door.setNotInterested(page.undo.ids[i], false)
            if (r.error) { frame.toast(r.error.message); return }
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
            if (r.error) { frame.toast(r.error.message); return }
            if (r.reply && r.reply.job !== undefined) sent++
            else already++
        }
        if (list.length === 1) { if (sent === 0) frame.toast(list[0].title + " is on your list already"); return }
        frame.toast(sent === 0 ? "They are all on your list already"
                               : Fmt.plural(sent, "show goes", "shows go") + " on your Planning list"
                                 + (already > 0 ? ", " + already + " were on it already" : ""))
    }
    // The bar's two actions: they act on every card picked, then picking ends
    function planPicked() { var l = page.pickedShows(); page.clearPicks(); page.planAll(l) }
    function dismissPicked() { var l = page.pickedShows(); page.clearPicks(); page.dismissAll(l) }
    function getEpisodes(s) {
        if (page.owned(s)) { frame.go("downloads", { series: s.owned }, "Downloads"); return }
        frame.go("downloads", { hit: { id: s.anilist_id, title: s.title, romaji: s.romaji || "", english: s.english || "", native: s.native || "",
                                       cover_url: s.poster || "", episodes: s.episodes || 0, format: s.format || "", year: 0 } }, "Downloads")
    }
    function siteOf(s) { return s.site_url || "https://anilist.co/anime/" + s.anilist_id }
    function actionsFor(s) {
        var items = []
        if (page.owned(s)) items.push({ text: "Play next", icon: "play", action: function() { frame.playNext(s.owned, s.title) } })
        if (page.owned(s)) items.push({ text: "Open series", icon: "layers", action: function() { frame.go("series", { id: s.owned }, s.title) } })
        if (!s.list_status) items.push({ text: "Plan to watch", icon: "bookmark", action: function() { page.plan(s) } })
        items.push({ text: page.owned(s) ? "Get more episodes" : "Get episodes", icon: "download", action: function() { page.getEpisodes(s) } })
        items.push({ text: "Open on AniList", icon: "external-link", action: function() { frame.openLink(page.siteOf(s)) } })
        items.push({ text: "Not interested", icon: "eye-off", action: function() { page.dismiss(s) } })
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
            frame.toast(error && error.message ? error.message : "AniList could not be reached")
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
    readonly property real cardWidth: Math.floor((scroll.innerWidth - gap * (cols - 1)) / cols)

    PageScroll {
        id: scroll
        anchors.fill: parent
        readonly property real innerWidth: width - theme.space(16)

        // ---- The head: the title, the one line beside it, Refresh at the right
        Item {
            width: parent.width
            height: theme.space(20)
            Label {
                id: titleLabel
                anchors.left: parent.left
                anchors.verticalCenter: parent.verticalCenter
                text: page.season === "Now" ? "Airing now" : (page.listed ? page.listed.label : page.nextLabel)
                role: "display"
                font.pointSize: theme.typePage
            }
            Label {
                objectName: "airing-summary"
                anchors.left: titleLabel.right
                anchors.leftMargin: theme.space(3)
                anchors.right: refreshBtn.left
                anchors.rightMargin: theme.space(4)
                anchors.baseline: titleLabel.baseline
                text: page.summary
                role: "ui"
                color: theme.textDim
                elide: Text.ElideRight
            }
            Button {
                id: refreshBtn
                objectName: "airing-refresh"
                anchors.right: parent.right
                anchors.verticalCenter: parent.verticalCenter
                text: page.refreshing ? "Refreshing" : "Refresh"
                icon: "refresh-cw"
                busy: page.refreshing
                flat: true
                onClicked: page.refresh(true)
            }
        }

        // ---- The two choices, and the way to the shows put aside
        Item {
            width: parent.width
            height: theme.controlHeight + theme.space(2)
            Row {
                anchors.left: parent.left
                anchors.verticalCenter: parent.verticalCenter
                spacing: theme.space(3)
                Seg {
                    objectName: "airing-season"
                    options: [{ text: "Airing now", icon: "radio", name: "now" }, { text: page.nextLabel, icon: "calendar-clock", name: "next" }]
                    index: page.season === "Now" ? 0 : 1
                    onPicked: function(i) { page.pickSeason(i === 0 ? "Now" : "Next") }
                }
                Seg {
                    objectName: "airing-filter"
                    options: [{ text: "Everything", name: "all", count: page.listed ? page.listed.shows.length : undefined },
                              { text: "On my lists", name: "lists", count: page.listed ? page.listedCount : undefined },
                              { text: "In my library", name: "library", count: page.listed ? page.ownedCount : undefined }]
                    index: page.filter
                    onPicked: function(i) { page.filter = i; page.keepView(); page.clearPicks() }
                }
            }
            Button {
                objectName: "airing-not-interested"
                visible: !!page.listed && page.listed.not_interested > 0
                anchors.right: parent.right
                anchors.verticalCenter: parent.verticalCenter
                flat: true
                small: true
                icon: "eye-off"
                text: page.listed ? page.listed.not_interested + " not interested" : ""
                onClicked: frame.go("settings", { tab: "library", focus: "not-interested" }, "Settings")
                Tooltip { text: "Shows you marked not interested, in Settings" }
            }
        }

        // ---- Out now: the episodes of the last two days, newest first
        Column {
            objectName: "airing-out-now"
            visible: page.aired.length > 0
            width: parent.width
            spacing: theme.space(3)
            topPadding: theme.space(2)
            bottomPadding: theme.space(4)
            Row {
                spacing: theme.space(3)
                Label {
                    id: outTitle
                    text: "Out now"
                    role: "display"
                    font.pointSize: theme.typeHeading
                    font.variableAxes: ({ "wdth": 70, "wght": 420 })
                    color: theme.accent
                }
                Label { anchors.baseline: outTitle.baseline; text: page.aired.length; role: "ledger"; color: theme.textFaint }
                Label { anchors.baseline: outTitle.baseline; text: "The last two days"; role: "ui"; color: theme.textDim }
            }
            Flow {
                width: parent.width
                spacing: theme.space(2)
                Repeater {
                    model: page.outShown
                    Rectangle {
                        id: pill
                        required property var modelData
                        readonly property var show: page.showOf(modelData.anilist_id)
                        objectName: "airing-out-" + modelData.anilist_id + "-" + modelData.episode
                        width: pillRow.implicitWidth + theme.space(5)
                        height: theme.controlHeight
                        radius: theme.radiusTile
                        color: pm.containsMouse ? theme.raised : theme.tile
                        border.color: pill.activeFocus ? theme.focusRing : theme.line
                        border.width: 1
                        activeFocusOnTab: true
                        function open() {
                            if (pill.show) page.openActions(pill.show, pill)
                            else frame.openLink("https://anilist.co/anime/" + pill.modelData.anilist_id)
                        }
                        Keys.onReturnPressed: pill.open()
                        Row {
                            id: pillRow
                            anchors.centerIn: parent
                            spacing: theme.space(2)
                            Rectangle {
                                anchors.verticalCenter: parent.verticalCenter
                                width: theme.space(2); height: width; radius: width / 2
                                color: page.nowMs / 1000 - pill.modelData.at < 86400 ? theme.accent : theme.textFaint
                            }
                            Label {
                                anchors.verticalCenter: parent.verticalCenter
                                width: Math.min(implicitWidth, theme.space(70))
                                text: pill.modelData.title
                                role: "ui"
                                elide: Text.ElideRight
                            }
                            Label { anchors.verticalCenter: parent.verticalCenter; text: Fmt.episodeName("", pill.modelData.episode); role: "ledger"; color: theme.textDim }
                            Label { anchors.verticalCenter: parent.verticalCenter; text: page.ago(pill.modelData.at); role: "ledger"; color: theme.textFaint }
                        }
                        MouseArea { id: pm; anchors.fill: parent; hoverEnabled: true; cursorShape: Qt.PointingHandCursor; onClicked: pill.open() }
                    }
                }
                Button {
                    visible: page.aired.length > 12
                    small: true
                    flat: true
                    text: page.allOut ? "Fewer" : "All " + page.aired.length
                    onClicked: page.allOut = !page.allOut
                }
            }
        }

        // ---- The shows, most popular first
        Flow {
            objectName: "airing-grid"
            width: parent.width
            spacing: page.gap
            Repeater {
                model: page.shows
                Card {
                    id: card
                    required property var modelData
                    objectName: "tile-" + modelData.anilist_id
                    item: page.cardItem(modelData)
                    metaOverride: page.metaOf(modelData)
                    posterWidth: page.cardWidth
                    nowMs: page.nowMs
                    dismissible: true
                    selectable: true
                    selecting: page.selecting
                    selected: !!page.picked[modelData.anilist_id]
                    onSelectToggled: page.togglePick(modelData)
                    onOpened: page.openActions(modelData, card)
                    onPlayed: page.openActions(modelData, card)
                    onDismissed: page.dismiss(modelData)
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
                title: "AniList could not be reached"
                body: "The shows on air come from AniList. They come back on the next refresh."
                Button { text: "Try again"; icon: "refresh-cw"; onClicked: { page.reload(); page.refresh(true) } }
            }
            EmptyState {
                visible: !page.loadFailed && (!page.listed || page.listed.shows.length === 0)
                icon: page.refreshing ? "loader-circle" : "radio"
                title: page.refreshing ? "Reading what is on air" : "Nothing read yet"
                body: page.refreshing ? "Every show on air, from AniList. This takes a few seconds the first time." : "Refresh reads every show on air from AniList."
                Button { visible: !page.refreshing; text: "Refresh"; icon: "refresh-cw"; onClicked: page.refresh(true) }
            }
            EmptyState {
                visible: !page.loadFailed && !!page.listed && page.listed.shows.length > 0 && page.filter > 0
                icon: page.filter === 1 ? "bookmark" : "layers"
                title: page.filter === 1 ? "Nothing on air is on your lists" : "Nothing on air is in your library"
                body: page.filter === 1 ? "Plan to watch on a card puts it on your list." : "Get episodes on a card downloads it into your library."
                Button { text: "Show everything"; onClicked: page.filter = 0 }
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
                text: page.pickedCount + " picked"
                role: "ui"
            }
            Button { objectName: "airing-picked-plan"; small: true; text: "Plan to watch"; icon: "bookmark"; onClicked: page.planPicked() }
            Button { objectName: "airing-picked-dismiss"; small: true; text: "Not interested"; icon: "eye-off"; onClicked: page.dismissPicked() }
            Button { objectName: "airing-picked-cancel"; small: true; flat: true; text: "Cancel"; onClicked: page.clearPicks() }
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
            Button { objectName: "airing-undo-button"; small: true; text: "Undo"; icon: "rotate-ccw"; onClicked: page.takeBack() }
        }
    }
}
