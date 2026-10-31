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
        if (page.undo) { page.undo = null; return true }
        return false
    }

    // "Now" or "Next", as the core spells SeasonPick
    property string season: "Now"
    // 0 everything, 1 on my lists, 2 in my library
    property int filter: 0
    property var listed: null
    property bool refreshing: false
    // The season read this page waits on: events of the other season's read are not its own
    property int refreshJob: -1
    property bool loadFailed: false
    property real nowMs: Date.now()
    // The show just marked, while its Undo is up: { id, title }
    property var undo: null
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
    readonly property string summary: {
        if (!listed || listed.shows.length === 0) return ""
        var line = Fmt.plural(listed.shows.length, "show", "shows") + (page.season === "Now" ? " on air" : "")
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
        page.allOut = false
        page.refreshing = false
        page.refreshJob = -1
        page.listed = null
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
    function dismiss(s) {
        var r = Door.setNotInterested(s.anilist_id, true)
        if (r.error) { frame.toast(r.error.message); return }
        page.undo = { id: s.anilist_id, title: s.title }
        undoTimer.restart()
        page.reload()
    }
    function takeBack() {
        if (!page.undo) return
        var r = Door.setNotInterested(page.undo.id, false)
        if (r.error) { frame.toast(r.error.message); return }
        page.undo = null
        page.reload()
    }
    function plan(s) {
        if (!page.trackerConnected) { frame.openConnect("Anilist"); return }
        var r = Door.planToWatch(s.anilist_id)
        if (r.error) { frame.toast(r.error.message); return }
        if (!r.reply || r.reply.job === undefined) frame.toast(s.title + " is on your list already")
    }
    function getEpisodes(s) {
        if (page.owned(s)) { frame.go("downloads", { series: s.owned }, "Downloads"); return }
        frame.go("downloads", { hit: { id: s.anilist_id, title: s.title, romaji: s.romaji || "", english: s.english || "", native: s.native || "",
                                       cover_url: s.poster || "", episodes: s.episodes || 0, format: s.format || "", year: 0 } }, "Downloads")
    }
    function siteOf(s) { return s.site_url || "https://anilist.co/anime/" + s.anilist_id }
    function actionsFor(s) {
        var items = []
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
                    options: [{ text: "Everything", name: "all" }, { text: "On my lists", name: "lists" }, { text: "In my library", name: "library" }]
                    index: page.filter
                    onPicked: function(i) { page.filter = i }
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

    // ---- Undo, at the foot, for the show just marked
    Rectangle {
        objectName: "airing-undo"
        visible: !!page.undo
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
                text: page.undo ? page.undo.title + " won't show up here again" : ""
                role: "ui"
                elide: Text.ElideRight
            }
            Button { objectName: "airing-undo-button"; small: true; text: "Undo"; icon: "rotate-ccw"; onClicked: page.takeBack() }
        }
    }
}
