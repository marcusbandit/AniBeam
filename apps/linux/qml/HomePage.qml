// Home: the page the app opens on, ordered by what the app is for. First, the series to go
// on with (an episode left half-way, or the next one on disk), set on a band in that
// series' own colour, read off its poster. With a tracker connected, the series on hold
// there follow Continue in the same row whenever Continue leaves room. Then the week:
// seven day columns from today with the airing series' posters placed on the day their
// next episode lands, and a Later column for the rest. Then the series with episodes
// waiting, a row not started yet, and the library's genres as a way in. A series dropped
// on the tracker is in none of those: it closes the page on a shelf of its own, a reminder
// of what can leave the disk. Every shelf is a plain grid whose column count comes from the
// width, and a shelf that could run long stops at a row or two with "See all".
import QtQuick
import com.marcusrosado.AniBeam

Item {
    id: page
    objectName: "home-page"
    property var props: ({})
    property string title: "Home"
    property alias scrollY: scroll.contentY
    function escapePressed() { return false }
    // The arrows with nothing focused go to the scroll's grid keys (Down lands on the hero's
    // button or the first card)
    Keys.onPressed: function(event) { scroll.keyPressed(event) }

    property real nowMs: Date.now()
    property var shelf: []        // the library, last viewed first
    property var airingRows: []   // releasing, on disk, not done with
    property var resume: null     // { card, detail, episode, partial }
    property bool libraryEmpty: false
    Timer { interval: 30000; running: true; repeat: true; onTriggered: page.nowMs = Date.now() }

    readonly property real gap: theme.space(4)

    function viewedAt(c) { return c.last_viewed_at ? c.last_viewed_at : 0 }
    function watchedOf(c) { return c.watched !== null && c.watched !== undefined ? c.watched : 0 }
    function droppedOf(c) { return c.list_status === "Dropped" }
    function pausedOf(c) { return c.list_status === "Paused" }
    // On hold is a list on a tracker, so with none connected none of it shows
    readonly property bool tracked: !!Door.trackers && ((!!Door.trackers.anilist && Door.trackers.anilist.connected === true)
                                                      || (!!Door.trackers.mal && Door.trackers.mal.connected === true))

    // The hero: the series to go on with. An episode stopped short of its end comes first,
    // else the series' next episode on disk; a series with neither is caught up and is
    // passed over for the next one.
    // A series counts as started when it was played here or when the tracker says it is
    // being watched with episodes seen, since plenty get watched somewhere else. They are
    // ordered by the later of the last play here and, while an episode is out and unseen,
    // the day that episode arrived: a new episode of a show being followed is the obvious
    // next watch. The first is the hero, the rest the Continue row. At most sixteen series
    // are read; a dropped one is never among them, and one on hold sits in its own group.
    function startedOf(c) {
        if (page.droppedOf(c) || c.list_status === "Completed") return false
        if (page.tracked && page.pausedOf(c)) return false
        if (page.viewedAt(c) > 0) return true
        return page.watchedOf(c) > 0 && (c.list_status === "Watching" || c.list_status === "Repeating")
    }
    function continueAt(c) {
        var fresh = c.watched_state === "Behind" && c.latest_activity_at ? c.latest_activity_at : 0
        return Math.max(page.viewedAt(c), fresh)
    }
    function pickContinue() {
        var out = []
        var started = shelf.filter(page.startedOf).sort(function(a, b) { return page.continueAt(b) - page.continueAt(a) })
        for (var i = 0; i < started.length && i < 16; i++) {
            var c = started[i]
            var r = Door.getSeries(c.id)
            if (r.error) continue
            var d = r.reply.detail
            var eps = d.episodes || []
            var partial = null, next = null
            for (var k = 0; k < eps.length; k++) {
                var e = eps[k]
                // Stopped before the outro (85 percent when no outro is known): a stop in
                // the credits is a finished episode, a stop before them is not
                // An episode a torrent is still writing is never what to go on with
                if (e.downloading !== undefined && e.downloading !== null) continue
                if (!e.watched && e.stopped_short && e.resume && e.resume.duration > 0)
                    partial = e
                if (d.next_up && e.file === d.next_up) next = e
            }
            var ep = partial || next
            if (ep) out.push({ card: c, detail: d, episode: ep, partial: partial !== null })
        }
        return out
    }
    property var continueList: []
    function pickResume() { continueList = pickContinue(); return continueList.length ? continueList[0] : null }

    function reload() {
        var r = Door.listLibrary("All", "", "LastViewed", "Desc", false)
        if (r.error) { frame.toast(r.error.message); return }
        shelf = r.reply.series
        airingRows = r.reply.airing || []
        var n = r.reply.counts || {}
        libraryEmpty = (n.all || 0) === 0 && (n.watched || 0) === 0
        resume = pickResume()
        nowMs = Date.now()
    }
    Timer { id: debounce; interval: 500; onTriggered: page.reload() }
    Connections {
        target: Door
        function onSeriesChanged() { if (!debounce.running) debounce.start() }
        function onSeriesRemoved() { if (!debounce.running) debounce.start() }
        function onPreferencesChanged(p) { if (!debounce.running) debounce.start() }
    }
    Component.onCompleted: reload()

    function playFile(file, title) { if (file) frame.go("player", { file: file }, title) }
    function wholeFirst(d) { return d && d.episodes && d.episodes.length && (d.episodes[0].downloading === undefined || d.episodes[0].downloading === null) ? d.episodes[0].file : null }
    function openSeries(c) { frame.go("series", { id: c.id }, c.title) }
    function playSeries(c) {
        var r = Door.getSeries(c.id)
        var d = !r.error ? r.reply.detail : null
        var file = d ? (d.next_up ? d.next_up : (c.kind === "Movie" ? page.wholeFirst(d) : null)) : null
        if (file) playFile(file, c.title)
        else openSeries(c)
    }

    // ---- The lists. The hero's series shows only in the hero.
    readonly property int heroId: resume ? resume.card.id : -1
    function behindOf(c) {
        var w = watchedOf(c), reach = Math.max(c.episodes_on_disk || 0, 0)
        return c.watched_state === "Behind" ? Math.max(1, reach - w) : 0
    }
    readonly property var airingIds: {
        var m = {}
        for (var i = 0; i < airingRows.length; i++) m[airingRows[i].id] = true
        return m
    }
    // The week: today and the six days after it, each holding the airing series whose next
    // episode lands that day (soonest first); what airs later or carries no date is Later.
    readonly property var week: {
        var now = new Date(nowMs)
        var start = new Date(now.getFullYear(), now.getMonth(), now.getDate()).getTime() / 1000
        var days = []
        for (var i = 0; i < 7; i++) {
            var d = new Date((start + i * 86400) * 1000)
            days.push({ label: i === 0 ? "Today" : i === 1 ? "Tomorrow" : Qt.formatDate(d, "dddd"), date: Qt.formatDate(d, "d MMM"), today: i === 0, cards: [] })
        }
        var later = []
        var rows = airingRows.slice().sort(function(a, b) {
            var na = a.next_airing ? a.next_airing.at : 1e12, nb = b.next_airing ? b.next_airing.at : 1e12
            return na - nb
        })
        for (var r = 0; r < rows.length; r++) {
            var c = rows[r]
            var at = c.next_airing ? c.next_airing.at : 0
            var k = at > 0 ? Math.floor((at - start) / 86400) : -1
            if (k >= 0 && k < 7) days[k].cards.push(c); else later.push(c)
        }
        if (later.length) days.push({ label: "No date yet", date: "", today: false, cards: later, later: true })
        return days
    }
    readonly property int airingWaiting: airingRows.filter(function(c) { return page.behindOf(c) > 0 }).length
    readonly property string weekFact: {
        if (airingWaiting > 0) return Fmt.plural(airingWaiting, "series has", "series have") + " a new episode out"
        var now = nowMs / 1000, next = null
        for (var i = 0; i < airingRows.length; i++) {
            var n = airingRows[i].next_airing
            if (n && n.at > now && (!next || n.at < next.n.at)) next = { n: n, c: airingRows[i] }
        }
        return next ? "Next up: " + next.c.title + " in " + shortCountdown(next.n.at - now) : ""
    }
    // Episodes waiting: behind on the tracker, or watched recently and not caught up
    readonly property var catchUpList: {
        var month = 45 * 86400, now = nowMs / 1000
        return shelf.filter(function(c) {
            if (c.id === page.heroId || page.airingIds[c.id] || page.droppedOf(c)) return false
            if (c.watched_state === "Behind") return true
            return page.viewedAt(c) > now - month && c.watched_state !== "CaughtUp"
        })
    }
    // The series a file arrived in over the last fortnight, newest first
    readonly property var downloadedList: {
        var since = nowMs / 1000 - 14 * 86400
        return shelf.filter(function(c) { return c.downloaded_at && c.downloaded_at >= since })
            .sort(function(a, b) { return b.downloaded_at - a.downloaded_at })
    }
    // Never started, best rated first
    readonly property var freshList: {
        var list = shelf.filter(function(c) {
            return c.id !== page.heroId && !page.airingIds[c.id] && !page.droppedOf(c) && page.viewedAt(c) === 0
                && page.watchedOf(c) === 0 && (c.episodes_on_disk || 0) > 0
        })
        return list.sort(function(a, b) { return (b.community_score || 0) - (a.community_score || 0) })
    }
    // On hold on the tracker: the ones with an episode on disk to play first, then the one
    // touched here or given a new episode most lately
    function lastTouchOf(c) { return Math.max(page.viewedAt(c), c.latest_activity_at || 0) }
    readonly property var holdList: {
        if (!tracked) return []
        return shelf.filter(page.pausedOf).sort(function(a, b) {
            var pa = page.toWatchOf(a) > 0 ? 1 : 0, pb = page.toWatchOf(b) > 0 ? 1 : 0
            return pb - pa || page.lastTouchOf(b) - page.lastTouchOf(a)
        })
    }
    // Dropped on the tracker and still on disk
    readonly property var droppedList: shelf.filter(page.droppedOf)
    readonly property int droppedFiles: droppedList.reduce(function(n, c) { return n + (c.episodes_on_disk || 0) }, 0)
    // Every genre in the library with its count, the biggest first
    readonly property var genreList: {
        var counts = {}
        for (var i = 0; i < shelf.length; i++) {
            var g = shelf[i].genres || []
            for (var k = 0; k < g.length; k++) counts[g[k]] = (counts[g[k]] || 0) + 1
        }
        var out = Object.keys(counts).map(function(name) { return { name: name, count: counts[name] } })
        return out.sort(function(a, b) { return b.count - a.count || (a.name < b.name ? -1 : 1) })
    }

    // "4d 18h", "3h 12m", "9m"
    function shortCountdown(secs) {
        var t = Math.max(0, Math.floor(secs))
        var d = Math.floor(t / 86400), h = Math.floor(t % 86400 / 3600), m = Math.floor(t % 3600 / 60)
        return d > 0 ? d + "d " + h + "h" : h > 0 ? h + "h " + m + "m" : Math.max(1, m) + "m"
    }

    // ---- A section heading: the name in the display cut, its one fact beside it, and an
    // action at the right
    component Heading: Item {
        id: heading
        property string text: ""
        property string fact: ""
        property alias action: actionSlot.data
        width: parent ? parent.width : 0
        height: theme.space(12)
        Row {
            id: headingRow
            anchors.left: parent.left
            anchors.bottom: parent.bottom
            anchors.bottomMargin: theme.space(2)
            spacing: theme.space(3)
            Label { id: headingText; text: heading.text; role: "display"; font.pointSize: theme.typeHeading; font.variableAxes: ({ "wdth": 70, "wght": 420 }) }
            // Whole or not at all: a heading too narrow for its fact (On hold's split row)
            // drops it rather than running it into the action or the next heading
            Label {
                anchors.baseline: headingText.baseline
                text: heading.fact
                role: "ui"
                color: theme.textDim
                visible: implicitWidth <= heading.width - headingText.implicitWidth - headingRow.spacing
                                          - (actionSlot.visibleChildren.length > 0 ? actionSlot.width + theme.space(3) : 0)
            }
        }
        Item { id: actionSlot; anchors.right: parent.right; anchors.bottom: parent.bottom; anchors.bottomMargin: theme.space(2); width: childrenRect.width; height: childrenRect.height }
    }

    // The episodes on disk as a phrase: "On disk 1 to 3" while they run from the first,
    // else the count and the newest. The newest number comes off the card's code (S02E03,
    // EP 3).
    function newestOnDisk(c) {
        var m = /(\d+(?:\.\d+)?)$/.exec(c.code || "")
        return m ? Number(m[1]) : 0
    }
    function diskText(c) {
        var n = c.episodes_on_disk || 0, top = newestOnDisk(c)
        if (n === 0) return "Nothing on disk yet"
        if (n === 1 && top > 0) return "On disk: episode " + top
        if (top === n) return "On disk: episodes 1 to " + n
        return "On disk: " + n + " episodes, newest " + top
    }
    // The episodes on disk past the tracker's progress; -1 with no tracker entry
    function toWatchOf(c) {
        if (c.watched === null || c.watched === undefined) return -1
        var n = c.episodes_on_disk || 0, top = newestOnDisk(c)
        return Math.min(n, Math.max(0, top - c.watched))
    }

    // ---- One series in the week: the poster as a thumbnail, the title, which episode
    // lands and at what time, and what is on disk with how much of it is still to watch
    component WeekEntry: Rectangle {
        id: entry
        property var item: ({})
        property bool today: false
        readonly property bool gridStop: true
        readonly property var next: item.next_airing && item.next_airing.at * 1000 > page.nowMs ? item.next_airing : null
        readonly property int toWatch: page.toWatchOf(item)
        activeFocusOnTab: true
        Keys.onReturnPressed: page.openSeries(item)
        Keys.onEnterPressed: page.openSeries(item)
        Keys.onSpacePressed: page.playSeries(item)
        height: theme.space(30)
        radius: theme.radiusTile
        color: em.containsMouse ? theme.raised : theme.tile
        border.width: 1
        border.color: entry.activeFocus ? theme.accent : em.containsMouse ? theme.lineStrong : "transparent"
        scale: em.pressed ? 0.98 : 1

        Corner {
            id: entryThumb
            x: theme.space(2); anchors.verticalCenter: parent.verticalCenter
            height: parent.height - theme.space(4); width: Math.round(height * 2 / 3)
            radius: theme.radiusSm
            color: theme.tileSunken
            fillItem: entryPoster.status === Image.Ready ? entryPoster : null
            Image {
                id: entryPoster
                visible: false
                width: entryThumb.width; height: entryThumb.height
                source: entry.item.poster ? Fmt.fileUrl(entry.item.poster) : ""
                sourceSize: theme.posterDecodeSize
                fillMode: Image.PreserveAspectCrop
                asynchronous: true
            }
        }
        Column {
            anchors.left: entryThumb.right; anchors.leftMargin: theme.space(3)
            anchors.right: parent.right; anchors.rightMargin: theme.space(3)
            anchors.verticalCenter: parent.verticalCenter
            spacing: theme.space(1)
            Label {
                width: parent.width
                text: entry.item.title || ""
                role: "title"
                font.pointSize: theme.typeNormal
                color: em.containsMouse ? theme.accent : theme.text
                wrapMode: Text.Wrap
                maximumLineCount: 2
                elide: Text.ElideRight
                lineHeight: 0.95
            }
            // What lands: "Episode 4 at 17:30", today with the time left after it. A narrow
            // card wraps the time onto its own line rather than past the edge
            Flow {
                width: parent.width
                spacing: theme.space(1.5)
                Label {
                    width: Math.min(implicitWidth, parent.width)
                    elide: Text.ElideRight
                    text: entry.next ? (entry.next.season && entry.next.season !== 1 ? "S" + entry.next.season + " " : "") + Fmt.episodeSpan(entry.next.episode, entry.next.through || 0) : "No date yet"
                    role: "ui"
                    font.weight: 600
                    color: entry.next ? theme.text : theme.textDim
                }
                Label {
                    visible: !!entry.next
                    width: Math.min(implicitWidth, parent.width)
                    elide: Text.ElideRight
                    text: entry.next ? (entry.next.predicted ? "predicted, " : "at ") + Qt.formatTime(new Date(entry.next.at * 1000), "HH:mm") + (entry.today ? ", in " + page.shortCountdown(entry.next.at - page.nowMs / 1000) : "") : ""
                    role: "ui"
                    color: theme.accent
                }
            }
            // What is here: the episodes on disk, then how many of them are still to watch
            Row {
                width: parent.width
                spacing: theme.space(1.5)
                Label {
                    id: onDisk
                    width: Math.min(implicitWidth, parent.width)
                    elide: Text.ElideRight
                    text: page.diskText(entry.item); role: "ledger"; color: theme.textDim
                }
                Label {
                    visible: (entry.item.episodes_on_disk || 0) > 0 && entry.toWatch >= 0
                    width: Math.max(0, parent.width - onDisk.width - parent.spacing)
                    elide: Text.ElideRight
                    text: "\u00b7 " + (entry.toWatch > 0 ? entry.toWatch + " to watch" : "all watched")
                    role: "ledger"
                    font.weight: entry.toWatch > 0 ? 650 : 400
                    color: entry.toWatch > 0 ? theme.accent : theme.textFaint
                }
            }
        }
        MouseArea {
            id: em
            anchors.fill: parent
            hoverEnabled: true
            cursorShape: Qt.PointingHandCursor
            onClicked: { entry.forceActiveFocus(); page.openSeries(entry.item) }
        }
    }

    // ---- A shelf: a heading over a grid of cards
    component Shelf: Column {
        id: shelfBox
        property string heading: ""
        property string fact: ""
        property var list: []
        property real minWidth: theme.posterWidth
        property int maxRows: 0
        property string moreText: "See all"
        signal more()
        readonly property int cols: Math.max(1, Math.floor((width + page.gap) / (minWidth + page.gap)))
        readonly property real cardWidth: Math.floor((width - page.gap * (cols - 1)) / cols)
        readonly property var shown: maxRows > 0 ? list.slice(0, cols * maxRows) : list
        visible: list.length > 0
        spacing: theme.space(3)
        Heading {
            text: shelfBox.heading
            fact: shelfBox.fact
            action: Button {
                visible: shelfBox.list.length > shelfBox.shown.length
                text: shelfBox.moreText
                small: true
                flat: true
                onClicked: shelfBox.more()
            }
        }
        Flow {
            width: shelfBox.width
            spacing: page.gap
            Repeater {
                model: shelfBox.shown
                Card {
                    required property var modelData
                    objectName: "card-" + modelData.id   // a --drive target
                    item: modelData
                    posterWidth: shelfBox.cardWidth
                    nowMs: page.nowMs
                    onOpened: page.openSeries(modelData)
                    onPlayed: page.playSeries(modelData)
                }
            }
        }
    }

    // ---- Continue, with On hold after it in the same row: while Continue leaves room, the
    // series on hold take the rest of the row after a double gap, under a heading of their
    // own on the same line. A Continue that fills the row is the row, as it always was.
    component ContinueRow: Column {
        id: row
        property var continueCards: []
        property var holdCards: []
        readonly property int cols: Math.max(1, Math.floor((width + page.gap) / (theme.posterWidth + page.gap)))
        readonly property int lead: Math.min(continueCards.length, cols)
        readonly property bool split: holdCards.length > 0 && continueCards.length < cols
        readonly property int holdShown: split ? Math.min(holdCards.length, cols - lead) : 0
        // The second gap between the groups comes out of the cards, an equal share each, so
        // the row still ends at the page's edge with as many cards as any other
        readonly property real cardWidth: Math.floor((width - page.gap * (cols - 1) - (split && lead > 0 ? page.gap : 0)) / cols)
        readonly property real leadWidth: lead > 0 ? lead * cardWidth + (lead - 1) * page.gap : 0
        readonly property real holdX: lead > 0 ? leadWidth + page.gap * 2 : 0
        visible: lead > 0 || holdShown > 0
        spacing: theme.space(3)
        Item {
            width: row.width
            height: theme.space(12)
            Heading {
                visible: row.lead > 0
                width: row.split ? row.leadWidth : row.width
                text: "Continue"
                fact: page.continueList.length > 1 ? Fmt.plural(page.continueList.length - 1, "more series", "more series") + " to go on with" : ""
                action: Button {
                    visible: !row.split && row.continueCards.length > row.lead
                    text: "See all"
                    small: true
                    flat: true
                    onClicked: frame.go("library", { arrange: "WatchState" })
                }
            }
            Heading {
                objectName: "home-on-hold"
                visible: row.split
                x: row.holdX
                width: row.width - row.holdX
                text: "On hold"
                fact: Fmt.plural(row.holdCards.length, "series", "series") + " to come back to"
                action: Button {
                    visible: row.holdCards.length > row.holdShown
                    text: "See all"
                    small: true
                    flat: true
                    onClicked: frame.go("library", { arrange: "WatchState" })
                }
            }
        }
        // Both groups share one parent, placed by x, so the arrow keys cross from Continue
        // into On hold (GridKeys steps among one parent's children); a Row would drop the
        // empty item a second gap needs, since it lays out nothing of width nought
        Item {
            width: row.width
            height: Math.round(row.cardWidth * 1.5)
            Repeater {
                model: row.continueCards.slice(0, row.lead)
                Card {
                    required property var modelData
                    required property int index
                    objectName: "card-" + modelData.id   // a --drive target
                    x: index * (row.cardWidth + page.gap)
                    item: modelData
                    posterWidth: row.cardWidth
                    nowMs: page.nowMs
                    onOpened: page.openSeries(modelData)
                    onPlayed: page.playSeries(modelData)
                }
            }
            Repeater {
                model: row.holdCards.slice(0, row.holdShown)
                Card {
                    required property var modelData
                    required property int index
                    objectName: "card-" + modelData.id
                    x: row.holdX + index * (row.cardWidth + page.gap)
                    item: modelData
                    posterWidth: row.cardWidth
                    nowMs: page.nowMs
                    onOpened: page.openSeries(modelData)
                    onPlayed: page.playSeries(modelData)
                }
            }
        }
    }

    PageScroll {
        id: scroll
        anchors.fill: parent
        visible: !page.libraryEmpty

        // ---- The hero: the series to go on with, on a band in its own colour, the banner
        // art fading in from the right
        Item {
            id: hero
            visible: page.resume !== null
            width: parent.width
            height: visible ? theme.space(92) : 0
            readonly property var r: page.resume
            readonly property var d: r ? r.detail : null
            readonly property var c: r ? r.card : null
            readonly property var ep: r ? r.episode : null
            readonly property bool hasArt: !!(d && d.banner)
            readonly property real pad: theme.space(8)
            // The band's colour: the poster's tone over a dark base, so white ink holds in
            // both modes; the ink accent is the tone lifted toward white
            PosterTone { id: toneOf; source: hero.c && hero.c.poster ? Fmt.fileUrl(hero.c.poster) : "" }
            readonly property color tone: toneOf.tone
            readonly property color base: theme.bandBase(tone, 0.5)
            readonly property color inkAccent: theme.bandAccent(tone, base)
            readonly property color ink: "#f4f6f8"
            readonly property color inkDim: Qt.rgba(1, 1, 1, 0.68)

            // The band's colour bleeds up into the page's head and out to the window's edges,
            // so the whole top of Home takes the series' tone
            Rectangle {
                x: -scroll.gutter; y: -theme.space(7)
                width: scroll.width; height: parent.height * 0.6 + theme.space(7)
                gradient: Gradient {
                    GradientStop { position: 0; color: Qt.rgba(hero.tone.r, hero.tone.g, hero.tone.b, 0.22) }
                    GradientStop { position: 1; color: Qt.rgba(hero.tone.r, hero.tone.g, hero.tone.b, 0) }
                }
            }
            Rectangle { anchors.fill: parent; radius: theme.radiusXl; color: hero.base; antialiasing: true }
            // The art sits in the right two thirds, under a wash of the band's colour that
            // clears to the right
            Item {
                anchors.fill: parent
                clip: true
                Corner {
                    id: band
                    visible: hero.hasArt
                    x: Math.round(parent.width * 0.3); y: 0
                    width: parent.width - x; height: parent.height
                    radius: theme.radiusXl
                    color: "transparent"
                    fillItem: hero.hasArt && art.status === Image.Ready ? art : null
                    Image {
                        id: art; visible: false
                        width: parent.width; height: parent.height
                        source: hero.hasArt ? Fmt.fileUrl(hero.d.banner) : ""
                        fillMode: Image.PreserveAspectCrop; asynchronous: true
                    }
                }
                // The wash shares the band's rounding, so no square corner of it shows past
                // the band's own; it stays solid until past the art's left edge
                Rectangle {
                    visible: hero.hasArt
                    anchors.fill: parent
                    radius: theme.radiusXl
                    antialiasing: true
                    gradient: Gradient {
                        orientation: Gradient.Horizontal
                        GradientStop { position: 0.33; color: hero.base }
                        GradientStop { position: 0.62; color: Qt.rgba(hero.base.r, hero.base.g, hero.base.b, 0.55) }
                        GradientStop { position: 1; color: Qt.rgba(hero.base.r, hero.base.g, hero.base.b, 0.1) }
                    }
                }
            }

            Corner {
                id: heroPoster
                x: hero.pad
                anchors.verticalCenter: parent.verticalCenter
                height: parent.height - hero.pad * 2
                width: Math.round(height * 2 / 3)
                radius: theme.radiusTile
                color: Qt.rgba(0, 0, 0, 0.3)
                fillItem: hero.c && hero.c.poster && heroPosterImage.status === Image.Ready ? heroPosterImage : null
                Image {
                    id: heroPosterImage; visible: false
                    anchors.fill: parent
                    source: hero.c && hero.c.poster ? Fmt.fileUrl(hero.c.poster) : ""
                    sourceSize: Qt.size(theme.posterDecodeSize.width * 2, theme.posterDecodeSize.height * 2)
                    fillMode: Image.PreserveAspectCrop; asynchronous: true
                }
                MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: page.openSeries(hero.c) }
            }

            Column {
                anchors.left: heroPoster.right
                anchors.leftMargin: theme.space(8)
                anchors.right: parent.right
                anchors.rightMargin: hero.pad
                anchors.verticalCenter: parent.verticalCenter
                spacing: theme.space(2.5)

                Label {
                    text: hero.r && hero.r.partial ? "Pick up where you stopped" : "Up next"
                    role: "title"
                    font.pointSize: theme.typeNormal
                    color: hero.inkAccent
                }
                Label {
                    width: parent.width
                    text: hero.c ? hero.c.title : ""
                    role: "display"
                    color: hero.ink
                    wrapMode: Text.Wrap
                    maximumLineCount: 2
                    elide: Text.ElideRight
                    lineHeight: 0.92
                }
                Label {
                    width: parent.width
                    text: hero.ep ? Fmt.episodeName(hero.ep.title || "", hero.ep.number) : ""
                    role: "ui"
                    color: hero.ink
                    elide: Text.ElideRight
                }
                Label {
                    objectName: "hero-downloading"
                    visible: !!(hero.c && hero.c.downloading && hero.c.downloading.length > 0)
                    text: visible ? "Downloading " + Fmt.plural(hero.c.downloading.length, "episode", "episodes") : ""
                    role: "ledger"
                    font.weight: 600; font.variableAxes: ({ "wdth": 88, "wght": 600 })
                    color: theme.tone(theme.cubeState("t").fill, Qt.rgba(1, 1, 1, 1), 0.3)
                }
                Row {
                    spacing: theme.space(3)
                    readonly property real frac: hero.r && hero.r.partial
                        ? hero.ep.resume.position / hero.ep.resume.duration
                        : hero.c && hero.c.total_episodes > 0 ? page.watchedOf(hero.c) / hero.c.total_episodes : 0
                    visible: frac > 0 || !!(hero.c && hero.c.cubes)
                    EpisodeCubes {
                        visible: codes.length > 0
                        anchors.verticalCenter: parent.verticalCenter
                        width: Math.min(implicitWidth, theme.space(60))
                        height: implicitHeight
                        codes: hero.c && hero.c.cubes ? hero.c.cubes : ""
                        cubeSize: theme.space(2)
                        maxRows: 1
                    }
                    Label {
                        anchors.verticalCenter: parent.verticalCenter
                        role: "ledger"
                        color: hero.inkDim
                        text: {
                            if (!hero.r) return ""
                            if (hero.r.partial) {
                                var left = Math.max(1, Math.round((hero.ep.resume.duration - hero.ep.resume.position) / 60))
                                return left + " min left"
                            }
                            var w = page.watchedOf(hero.c)
                            return hero.c.total_episodes > 0 ? w + " of " + hero.c.total_episodes + " watched" : ""
                        }
                    }
                }
                Item { width: 1; height: theme.space(2) }
                Row {
                    spacing: theme.space(3)
                    Button {
                        objectName: "home-resume"
                        // The top of the page's vertical key order: Down from here lands on
                        // the first card, Up from the first row comes back here
                        readonly property bool gridStop: true
                        primary: true
                        tint: hero.inkAccent
                        icon: "play"
                        text: hero.ep ? (hero.r.partial ? "Resume " : "Play ") + Fmt.episodeName("", hero.ep.number).toLowerCase() : ""
                        onClicked: if (hero.ep) page.playFile(hero.ep.file, hero.c.title)
                    }
                    Button {
                        text: "Open series"
                        flat: true
                        onClicked: page.openSeries(hero.c)
                    }
                }
            }
        }

        Item { width: 1; height: hero.visible ? theme.space(4) : 0 }

        // ---- Continue: the other series with an episode to go on with, after the hero, and
        // On hold in what is left of the row
        ContinueRow {
            id: continueRow
            objectName: "home-continue"
            width: parent.width
            continueCards: page.continueList.slice(1).map(function(e) { return e.card })
            holdCards: page.holdList
        }
        Item { width: 1; height: continueRow.visible ? theme.space(4) : 0 }

        Shelf {
            objectName: "home-downloaded"
            width: parent.width
            heading: "Recently downloaded"
            fact: page.downloadedList.length ? Fmt.plural(page.downloadedList.length, "series", "series") + " with new files this fortnight" : ""
            list: page.downloadedList
            maxRows: 1
            onMore: frame.go("library", { sort: "Downloaded", arrange: "None" })
        }
        Item { width: 1; height: page.downloadedList.length ? theme.space(4) : 0 }

        // ---- The week: one row per day that has an episode landing, today always first
        // (and marked); each series says which episode lands at what time and what of it is
        // already on disk. The series without a date yet close the list
        Column {
            objectName: "home-week"
            visible: page.airingRows.length > 0
            width: parent.width
            spacing: theme.space(4)
            readonly property real labelWidth: theme.space(30)
            Heading { text: "This week"; fact: page.weekFact }
            Repeater {
                model: page.week.filter(function(d) { return d.today || d.cards.length > 0 })
                Item {
                    id: day
                    required property var modelData
                    required property int index
                    objectName: "home-day-" + index
                    width: parent.width
                    height: Math.max(dayHead.implicitHeight, dayCards.implicitHeight)
                    readonly property bool hasCards: modelData.cards.length > 0
                    // Today's line is drawn in the accent; the others sit on a hairline
                    Rectangle {
                        x: 0; y: 0
                        width: 3; height: parent.height
                        radius: 1.5
                        color: day.modelData.today ? theme.accent : theme.line
                    }
                    Column {
                        id: dayHead
                        x: theme.space(4); y: 0
                        width: parent.parent.labelWidth - x
                        spacing: theme.space(0.5)
                        Label {
                            text: day.modelData.label
                            role: "title"
                            font.pointSize: theme.typeNormal
                            color: day.modelData.today ? theme.accent : theme.text
                        }
                        Label {
                            text: day.modelData.date
                            visible: text !== ""
                            role: "ledger"
                            color: day.modelData.today ? theme.accent : theme.textDim
                        }
                        Label {
                            visible: !day.hasCards
                            text: "Nothing lands today"
                            role: "ledger"
                            color: theme.textFaint
                        }
                    }
                    Flow {
                        id: dayCards
                        x: parent.parent.labelWidth
                        width: parent.width - x
                        spacing: theme.space(3)
                        readonly property int cols: Math.max(1, Math.floor((width + spacing) / (theme.space(84) + spacing)))
                        Repeater {
                            model: day.modelData.cards
                            WeekEntry {
                                required property var modelData
                                objectName: "week-card-" + modelData.id
                                item: modelData
                                today: day.modelData.today
                                width: Math.floor((dayCards.width - dayCards.spacing * (dayCards.cols - 1)) / dayCards.cols)
                            }
                        }
                    }
                }
            }
        }
        Item { width: 1; height: page.airingRows.length ? theme.space(4) : 0 }

        Shelf {
            objectName: "home-catch-up"
            width: parent.width
            heading: "Catch up"
            fact: page.catchUpList.length ? Fmt.plural(page.catchUpList.length, "series", "series") + " with episodes waiting" : ""
            list: page.catchUpList
            maxRows: 2
            onMore: frame.go("library", { arrange: "WatchState" })
        }
        Item { width: 1; height: page.catchUpList.length ? theme.space(4) : 0 }

        Shelf {
            objectName: "home-fresh"
            width: parent.width
            heading: "Start something new"
            fact: Fmt.plural(page.freshList.length, "series", "series") + " not started"
            list: page.freshList
            maxRows: 1
            onMore: frame.go("library")
        }
        Item { width: 1; height: page.freshList.length ? theme.space(4) : 0 }

        // ---- Genres: a way into the library by kind
        Column {
            objectName: "home-genres"
            visible: page.genreList.length > 0
            width: parent.width
            spacing: theme.space(3)
            Heading {
                text: "Genres"
                action: Button {
                    text: "Open library"
                    icon: "layers"
                    small: true
                    flat: true
                    onClicked: frame.go("library")
                }
            }
            Flow {
                width: parent.width
                spacing: theme.space(2)
                Repeater {
                    model: page.genreList
                    Rectangle {
                        id: genre
                        required property var modelData
                        width: genreRow.implicitWidth + theme.space(6)
                        height: theme.space(10)
                        radius: height / 2
                        color: gm.containsMouse ? theme.raised : theme.tile
                        border.color: gm.containsMouse ? theme.lineStrong : "transparent"
                        border.width: 1
                        scale: gm.pressed ? 0.97 : 1
                        Row {
                            id: genreRow
                            anchors.centerIn: parent
                            spacing: theme.space(2)
                            Label { text: genre.modelData.name; role: "ui"; anchors.verticalCenter: parent.verticalCenter }
                            Label { text: genre.modelData.count; role: "ledger"; color: theme.textFaint; anchors.verticalCenter: parent.verticalCenter }
                        }
                        MouseArea {
                            id: gm
                            anchors.fill: parent
                            hoverEnabled: true
                            cursorShape: Qt.PointingHandCursor
                            onClicked: frame.go("library", { genre: genre.modelData.name }, genre.modelData.name)
                        }
                    }
                }
            }
        }

        // ---- Dropped: last, since nothing here is to be watched
        Item { width: 1; height: page.droppedList.length && page.genreList.length ? theme.space(4) : 0 }
        Shelf {
            objectName: "home-dropped"
            width: parent.width
            heading: "Dropped"
            fact: Fmt.plural(page.droppedFiles, "file", "files") + " still on disk"
            list: page.droppedList
            maxRows: 1
            onMore: frame.go("library")
        }
    }

    EmptyState {
        anchors.centerIn: parent
        visible: page.libraryEmpty
        icon: "folder-plus"
        title: "No series yet"
        body: "Add the folder your anime lives in and the library fills itself."
        Button { text: "Add a folder"; primary: true; onClicked: frame.go("settings") }
    }
}
