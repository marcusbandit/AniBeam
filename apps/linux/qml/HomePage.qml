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
    readonly property var airingIds: {
        var m = {}
        for (var i = 0; i < airingRows.length; i++) m[airingRows[i].id] = true
        return m
    }
    // The week (#236): nine day rows, two days back, today and six ahead, every one shown
    // even when nothing is on it. Each row holds the episodes the schedule puts on that day,
    // the ones that already aired included, so this morning's episode stays on Today and
    // says whether it landed. A predicted next episode is placed by its guess; a series with
    // nothing in the nine days and no date at all is "No date yet".
    // The calendar day the week is built for: it changes once a day, not on every tick of
    // nowMs, so the rows and their entries are rebuilt only when the day turns. The day turning
    // reloads too, so each card's window of the schedule moves with it.
    readonly property string dayKey: Qt.formatDate(new Date(nowMs), "yyyy-MM-dd")
    onDayKeyChanged: if (shelf.length) reload()
    readonly property var week: {
        var now = new Date(dayKey + "T00:00:00")
        var start = new Date(now.getFullYear(), now.getMonth(), now.getDate()).getTime() / 1000
        var days = []
        for (var i = -2; i < 7; i++) {
            var d = new Date(now.getFullYear(), now.getMonth(), now.getDate() + i)
            days.push({ label: i === 0 ? "Today" : i === 1 ? "Tomorrow" : i === -1 ? "Yesterday" : Qt.formatDate(d, "dddd"),
                        date: Qt.formatDate(d, "d MMM"), today: i === 0, past: i < 0,
                        from: d.getTime() / 1000, cards: [] })
        }
        var end = new Date(now.getFullYear(), now.getMonth(), now.getDate() + 7).getTime() / 1000
        function dayOf(at) {
            for (var k = days.length - 1; k >= 0; k--) if (at >= days[k].from) return at < end ? k : -1
            return -1
        }
        var later = []
        for (var r = 0; r < airingRows.length; r++) {
            var c = airingRows[r]
            var drops = (c.week_airings || []).slice()
            // The next episode the schedule window does not hold: a predicted one, or a TVmaze
            // show's next season, which is not the card's season
            var n = c.next_airing
            if (n && !drops.some(function(a) { return a.episode === n.episode && (a.season || 1) === (n.season || 1) }))
                drops.push({ episode: n.episode, through: n.through, season: n.season, at: n.at, predicted: n.predicted, here: false })
            var placed = false
            for (var j = 0; j < drops.length; j++) {
                var k = dayOf(drops[j].at)
                if (k < 0) continue
                days[k].cards.push({ card: c, drop: drops[j] })
                placed = true
            }
            if (!placed) later.push({ card: c, drop: null })
        }
        for (var q = 0; q < days.length; q++) days[q].cards.sort(function(a, b) { return a.drop.at - b.drop.at })
        if (later.length) days.push({ label: "No date yet", date: "", today: false, past: false, cards: later, later: true })
        return days
    }
    // Where an episode that is due stands: "future" until its time, then "here" with its file
    // of that season on disk or watched already (the core says), "downloading" while a torrent
    // writes it, and "late" when it is out and nothing has arrived, or a predicted one that is
    // not out yet
    function dropState(c, drop, nowMs) {
        if (!drop) return "none"
        if (drop.at * 1000 > nowMs) return "future"
        if (drop.here) return "here"
        var last = drop.through || drop.episode
        if ((c.downloading || []).some(function(d) { return d.number >= drop.episode && d.number <= last })) return "downloading"
        return drop.predicted ? "overdue" : "late"
    }
    // This week split by the two questions (#326): what is out and waiting (here and not
    // watched, downloading, or aired and not downloaded), the next episode to air, and the
    // rest still to come in air order. An episode here and watched has nothing left to say.
    // Built from the episodes' standings, not from the clock: the clock ticks every 30 s, and
    // a fresh split would rebuild every card under it, dropping focus and hover. `weekKey`
    // reads the clock and changes only when a standing does; the split follows the key.
    function weekEntries() {
        var out = [], ahead = [], later = [], seen = {}
        for (var i = 0; i < week.length; i++) {
            var day = week[i]
            for (var k = 0; k < day.cards.length; k++) {
                var e = day.cards[k]
                if (day.later) { later.push({ card: e.card, drop: null, state: "none", day: i }); continue }
                var st = page.dropState(e.card, e.drop, nowMs)
                if (st === "here" || st === "downloading" || st === "late") {
                    if (st === "here" && page.dropWatched(e.card, e.drop)) continue
                    out.push({ card: e.card, drop: e.drop, day: i, state: st })
                } else ahead.push({ card: e.card, drop: e.drop, day: i, state: st })
            }
        }
        // One card a series: its latest episode out stands for the rest
        out.sort(function(a, b) { return b.drop.at - a.drop.at })
        out = out.filter(function(o) { if (seen[o.card.id]) return false; seen[o.card.id] = true; return true })
        out.sort(function(a, b) { return a.drop.at - b.drop.at })
        ahead.sort(function(a, b) { return a.drop.at - b.drop.at })
        return { out: out, ahead: ahead, later: later }
    }
    // Watched already: the tracker's progress reaches it, or with no tracker the core reads
    // the series caught up
    function dropWatched(c, drop) {
        if (c.watched !== null && c.watched !== undefined) return c.watched >= (drop.through || drop.episode)
        return c.watched_state === "CaughtUp"
    }
    // The key is each entry whole, its card, its episode and its standing, so anything a tile
    // shows changing rebuilds the week: a standing, and also a title when the title language
    // switches, a score, a hold or a new file on disk (#355)
    readonly property string weekKey: {
        var e = weekEntries()
        function entries(list) { return list.map(function(x) { return [x.card, x.drop, x.state] }) }
        return JSON.stringify([dayKey, entries(e.out), entries(e.ahead), entries(e.later)])
    }
    property var weekSplit: ({ out: [], ahead: [], next: null, later: [] })
    function splitWeek() {
        var e = weekEntries(), next = null
        for (var j = 0; j < e.ahead.length; j++) if (e.ahead[j].state === "future") { next = e.ahead.splice(j, 1)[0]; break }
        weekSplit = { out: e.out, ahead: e.ahead, next: next, later: e.later }
    }
    onWeekKeyChanged: splitWeek()
    // The board (#352): today and the six days after as columns, each with its episodes
    // still to come in air order, the next one first in its day. A day with nothing on it
    // stays as a narrow column, so the week keeps its shape and a gap reads as a gap. A
    // guessed episode past its time and not out yet sits on today, after the next one; the
    // series with no date close the board in a column of their own.
    readonly property var board: {
        // Built from the day and the split alone, not from `week`, so a reload that leaves
        // every standing as it was keeps the board's tiles, their focus and hover
        var cols = [], split = weekSplit, now = new Date(dayKey + "T00:00:00")
        var overdue = split.ahead.filter(function(a) { return a.state === "overdue" })
        for (var o = 0; o < 7; o++) {
            var i = o + 2   // the week's index: it starts two days back
            var items = split.ahead.filter(function(a) { return a.day === i && a.state !== "overdue" })
            if (o === 0) items = items.concat(overdue)
            // A copy marked as the next, since a model hands the tiles copies, never the same object
            var next = split.next && split.next.day === i ? { card: split.next.card, drop: split.next.drop, day: i, state: split.next.state, next: true } : null
            if (next) items.unshift(next)
            var d = new Date(now.getFullYear(), now.getMonth(), now.getDate() + o)
            var name = o === 0 ? "Today" : o === 1 ? "Tomorrow" : Qt.formatDate(d, "dddd")
            cols.push({ name: name, short: o === 0 ? name : Qt.formatDate(d, "ddd"),
                        date: o < 2 ? Qt.formatDate(d, "ddd d MMM") : Qt.formatDate(d, "d MMM"), shortDate: Qt.formatDate(d, "d MMM"),
                        today: o === 0, later: false, items: items, next: next })
        }
        if (split.later.length) cols.push({ name: "No date yet", short: "No date yet", date: "", shortDate: "", today: false, later: true, items: split.later, next: null })
        return cols
    }
    // "1 day 20 h 36 m" as parts, for the countdown's large figures
    function countdownParts(secs) {
        var t = Math.max(0, Math.floor(secs)), d = Math.floor(t / 86400), h = Math.floor(t % 86400 / 3600), m = Math.floor(t % 3600 / 60)
        var parts = []
        if (d > 0) parts.push({ n: d, unit: d === 1 ? "day" : "days" })
        if (d > 0 || h > 0) parts.push({ n: h, unit: "h" })
        parts.push({ n: Math.max(d > 0 || h > 0 ? 0 : 1, m), unit: "m" })
        return parts
    }
    readonly property int lateCount: {
        var n = 0
        for (var i = 0; i < week.length; i++)
            for (var k = 0; k < week[i].cards.length; k++) {
                var s = page.dropState(week[i].cards[k].card, week[i].cards[k].drop, nowMs)
                if (s === "late" || s === "overdue") n++
            }
        return n
    }
    readonly property bool weekFactCounts: lateCount === 0 && weekSplit.out.every(function(o) { return o.state !== "here" })
    readonly property string weekFact: {
        if (lateCount > 0) return Fmt.plural(lateCount, "episode is", "episodes are") + " late"
        var waiting = weekSplit.out.filter(function(o) { return o.state === "here" }).length
        if (waiting > 0) return Fmt.plural(waiting, "episode", "episodes") + " out to watch"
        // The board names the next episode itself; the fact counts what is on it instead
        var coming = weekSplit.ahead.length + (weekSplit.next ? 1 : 0)
        if (coming > 0) return Fmt.plural(coming, "episode", "episodes") + " in the next seven days"
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

    // The newest episode number on disk, off the card's code (S02E03, EP 3)
    function newestOnDisk(c) {
        var m = /(\d+(?:\.\d+)?)$/.exec(c.code || "")
        return m ? Number(m[1]) : 0
    }
    // The episodes on disk past the tracker's progress; -1 with no tracker entry
    function toWatchOf(c) {
        if (c.watched === null || c.watched === undefined) return -1
        var n = c.episodes_on_disk || 0, top = newestOnDisk(c)
        return Math.min(n, Math.max(0, top - c.watched))
    }

    // What an Out now poster says under its title: when it aired, or why it cannot play yet
    function outLine(o) {
        var t = Qt.formatTime(new Date(o.drop.at * 1000), "HH:mm"), day = week[o.day] ? week[o.day].label : ""
        if (o.state === "late") return "Not downloaded yet"
        if (o.state === "downloading") return "Downloading"
        return day + " " + t
    }
    // A sub-heading inside the week: a word and a dim count
    component WeekLabel: Row {
        property alias text: wl.text
        property alias fact: wf.text
        spacing: theme.space(2)
        Label { id: wl; role: "title"; font.pointSize: theme.typeNormal }
        Label { id: wf; anchors.baseline: wl.baseline; role: "ui"; color: theme.textDim; visible: text !== "" }
    }
    // The next episode to air, first in its day on the board: the poster on a band in its
    // own tone, the time left in large figures, then the title and when. The one loud thing
    // on the board. Opens the series.
    component NextTile: Corner {
        id: nextTile
        objectName: "home-week-next"
        property var entry: null
        readonly property bool gridStop: true
        readonly property var card: entry ? entry.card : ({})
        readonly property var drop: entry ? entry.drop : null
        readonly property var parts: drop ? page.countdownParts(drop.at - page.nowMs / 1000) : []
        readonly property color base: theme.bandBase(nextTone.tone, 0.5)
        readonly property color inkAccent: theme.bandAccent(nextTone.tone, base)
        readonly property color ink: "#f4f6f8"
        readonly property color inkDim: Qt.rgba(1, 1, 1, 0.68)
        radius: theme.radiusTile
        smoothing: theme.cornerSmoothing
        color: base
        borderColor: activeFocus ? theme.focusRing : nm.containsMouse ? Qt.rgba(1, 1, 1, 0.3) : "transparent"
        borderWidth: 1
        activeFocusOnTab: true
        Keys.onReturnPressed: page.openSeries(card)
        Keys.onEnterPressed: page.openSeries(card)
        Keys.onSpacePressed: page.playSeries(card)
        scale: theme.reducedMotion ? 1 : (nm.pressed ? 0.99 : 1)
        PosterTone { id: nextTone; source: nextTile.card.poster ? Fmt.fileUrl(nextTile.card.poster) : "" }
        // Wide, the poster stands at the left and the words beside it; in a narrow column the
        // time left runs across the top and a smaller poster sits beside the title under it.
        // Every size here follows the width, so the height never feeds back into it.
        readonly property bool compact: width < theme.space(110)
        readonly property real pad: compact ? theme.space(4) : theme.space(3)
        readonly property real gap: theme.space(2)
        readonly property real wordsH: lead.height + gap + info.height
        readonly property real wideH: Math.max(nextPoster.height, wordsH)
        height: compact ? nextPoster.y + Math.max(nextPoster.height, info.height) + pad : wideH + pad * 2
        Corner {
            id: nextPoster
            x: nextTile.pad
            y: nextTile.compact ? lead.y + lead.height + theme.space(3) : nextTile.pad + Math.round((nextTile.wideH - height) / 2)
            height: nextTile.compact ? theme.space(24) : theme.space(40); width: Math.round(height * 2 / 3)
            radius: theme.radiusInner(theme.space(3))
            color: Qt.rgba(0, 0, 0, 0.3)
            fillItem: nextArt.status === Image.Ready ? nextArt : null
            Image {
                id: nextArt
                visible: false
                width: nextPoster.width; height: nextPoster.height
                source: nextTile.card.poster ? Fmt.fileUrl(nextTile.card.poster) : ""
                sourceSize: theme.posterDecodeSize
                fillMode: Image.PreserveAspectCrop
                asynchronous: true
            }
        }
        // The time left
        Column {
            id: lead
            x: nextTile.compact ? nextTile.pad : nextPoster.x + nextPoster.width + theme.space(5)
            y: nextTile.compact ? nextTile.pad : nextTile.pad + Math.round((nextTile.wideH - nextTile.wordsH) / 2)
            width: nextTile.width - x - nextTile.pad
            spacing: theme.space(1)
            Label { text: "Next up"; role: "ledger"; color: nextTile.inkAccent }
            Flow {
                width: parent.width
                spacing: theme.space(2)
                Repeater {
                    model: nextTile.parts
                    Row {
                        required property var modelData
                        spacing: theme.space(0.75)
                        Label { id: fig; text: modelData.n; role: "display"; font.pointSize: theme.typeHeading * (nextTile.compact ? 1 : 1.2); font.variableAxes: ({ "wdth": 70, "wght": 560 }); color: nextTile.ink; font.features: { "tnum": 1 } }
                        Label { anchors.baseline: fig.baseline; text: modelData.unit; role: "title"; font.pointSize: theme.typeNormal; color: nextTile.inkDim }
                    }
                }
            }
        }
        // The series and when
        Column {
            id: info
            x: nextTile.compact ? nextPoster.x + nextPoster.width + theme.space(3) : lead.x
            y: nextTile.compact ? nextPoster.y : lead.y + lead.height + nextTile.gap
            width: nextTile.width - x - nextTile.pad
            spacing: theme.space(1)
            Label {
                width: parent.width
                text: nextTile.card.title || ""
                role: "title"
                font.pointSize: theme.typeNormal * (nextTile.compact ? 1 : 1.15)
                color: nextTile.ink
                wrapMode: Text.Wrap
                maximumLineCount: 3
                elide: Text.ElideRight
            }
            Label {
                width: parent.width
                text: nextTile.drop ? (nextTile.drop.season && nextTile.drop.season !== 1 ? "Season " + nextTile.drop.season + ", " : "")
                      + Fmt.episodeSpan(nextTile.drop.episode, nextTile.drop.through || 0) + ", "
                      + Qt.formatDateTime(new Date(nextTile.drop.at * 1000), "dddd HH:mm")
                      + (nextTile.drop.predicted ? ", time estimated" : "") : ""
                role: "ui"
                color: nextTile.inkDim
                wrapMode: Text.Wrap
            }
        }
        MouseArea {
            id: nm
            anchors.fill: parent
            hoverEnabled: true
            cursorShape: Qt.PointingHandCursor
            onClicked: { nextTile.forceActiveFocus(); page.openSeries(nextTile.card) }
            onContainsMouseChanged: frame.hoverCard(nextTile, nextTile.card, containsMouse)
        }
        Component.onDestruction: frame.hoverCard(nextTile, null, false)
    }
    // One episode on the board: a poster thumb, then the time, the title and the episode. A
    // guessed time says so beside the time, a late one says it is not out yet, and how many of
    // the series wait on disk follows the episode. Opens the series, Space plays it.
    component BoardTile: Corner {
        id: tile
        property var entry: ({})
        readonly property bool gridStop: true
        objectName: "week-" + (card.id || 0) + (drop ? "-" + (drop.season ? drop.season + "x" : "") + drop.episode : "")
        readonly property var card: entry.card || ({})
        readonly property var drop: entry.drop || null
        readonly property bool overdue: entry.state === "overdue"
        readonly property bool guessed: entry.state === "future" && !!drop && !!drop.predicted
        readonly property int toWatch: page.toWatchOf(card)
        height: Math.max(theme.space(21), tileText.implicitHeight + theme.space(4))
        radius: theme.radiusMd
        smoothing: theme.cornerSmoothing
        color: tm.containsMouse ? theme.raised : theme.tile
        borderColor: tile.activeFocus ? theme.focusRing : tm.containsMouse ? theme.lineStrong : "transparent"
        borderWidth: 1
        activeFocusOnTab: true
        Keys.onReturnPressed: page.openSeries(card)
        Keys.onEnterPressed: page.openSeries(card)
        Keys.onSpacePressed: page.playSeries(card)
        Corner {
            id: thumb
            x: theme.space(2)
            anchors.verticalCenter: parent.verticalCenter
            height: theme.space(17); width: Math.round(height * 2 / 3)
            radius: theme.radiusSm
            color: theme.tileSunken
            fillItem: thumbImg.status === Image.Ready ? thumbImg : null
            Image {
                id: thumbImg
                visible: false
                width: thumb.width; height: thumb.height
                source: tile.card.poster ? Fmt.fileUrl(tile.card.poster) : ""
                sourceSize: Qt.size(Math.ceil(thumb.width * 2), Math.ceil(thumb.height * 2))
                fillMode: Image.PreserveAspectCrop
                asynchronous: true
            }
        }
        Column {
            id: tileText
            anchors.left: thumb.right; anchors.leftMargin: theme.space(3)
            anchors.right: parent.right; anchors.rightMargin: theme.space(3)
            anchors.verticalCenter: parent.verticalCenter
            spacing: theme.space(0.5)
            Row {
                width: parent.width
                spacing: theme.space(1.5)
                Label {
                    id: when
                    text: !tile.drop ? "No date" : Qt.formatDateTime(new Date(tile.drop.at * 1000), tile.overdue ? "ddd HH:mm" : "HH:mm")
                    role: "ledger"
                    font.pointSize: theme.typeNormal
                    color: tile.overdue ? theme.red : tile.guessed ? theme.yellow : tile.drop ? theme.text : theme.textDim
                }
                Label {
                    anchors.baseline: when.baseline
                    width: Math.min(implicitWidth, parent.width - when.width - parent.spacing)
                    elide: Text.ElideRight
                    visible: tile.overdue || tile.guessed
                    text: tile.overdue ? "not out yet" : "estimated"
                    role: "ledger"
                    color: tile.overdue ? theme.red : theme.yellow
                    opacity: 0.8
                }
            }
            Label {
                width: parent.width
                text: tile.card.title || ""
                role: "ui"
                font.weight: 600
                color: tm.containsMouse ? theme.accent : theme.text
                wrapMode: Text.Wrap
                maximumLineCount: 2
                elide: Text.ElideRight
            }
            Row {
                width: parent.width
                spacing: theme.space(2)
                Label {
                    width: Math.min(implicitWidth, parent.width - (waits.visible ? waits.implicitWidth + parent.spacing : 0))
                    elide: Text.ElideRight
                    text: tile.drop ? (tile.drop.season && tile.drop.season !== 1 ? "S" + tile.drop.season + " " : "") + Fmt.episodeSpan(tile.drop.episode, tile.drop.through || 0) : ""
                    visible: text !== ""
                    role: "ledger"
                    color: theme.textDim
                }
                Label { id: waits; visible: tile.toWatch > 0; text: tile.toWatch + " to watch"; role: "ledger"; color: theme.accent }
            }
        }
        MouseArea {
            id: tm
            anchors.fill: parent
            hoverEnabled: true
            cursorShape: Qt.PointingHandCursor
            onClicked: { tile.forceActiveFocus(); page.openSeries(tile.card) }
            onContainsMouseChanged: frame.hoverCard(tile, tile.card, containsMouse)
        }
        Component.onDestruction: frame.hoverCard(tile, null, false)
    }
    // A day on the board: its name and date over a rule, the accent's on today, then its
    // episodes. A day with nothing says so once, small. `across` is the narrow window's
    // layout: the name at the left of a row, the episodes flowing beside it, the rule on top.
    component BoardDay: Item {
        id: bday
        property var day: ({})
        property bool narrow: false
        property bool across: false
        readonly property real headW: across ? theme.space(28) : width
        // Across, as many tiles on a line as hold a readable title, sharing it evenly
        readonly property real tileMin: theme.space(64)
        readonly property int perLine: Math.max(1, Math.floor((list.width + list.spacing) / (tileMin + list.spacing)))
        readonly property real tileW: across ? (list.width - (perLine - 1) * list.spacing) / perLine : list.width
        height: across ? Math.max(rule.height + theme.space(3) + head.height, list.y + list.height) : list.y + list.height
        Rectangle {
            id: rule
            y: bday.across ? 0 : head.height + theme.space(2)
            width: parent.width
            height: bday.across && !bday.day.today ? 1 : 2
            radius: 1
            color: bday.day.today ? theme.accent : theme.line
        }
        Column {
            id: head
            y: bday.across ? rule.height + theme.space(3) : 0
            width: bday.headW - (bday.across ? theme.space(3) : 0)
            Label {
                width: parent.width
                text: bday.narrow && !bday.across ? bday.day.short : bday.day.name
                role: "title"
                font.pointSize: theme.typeNormal
                color: bday.day.today ? theme.accent : bday.day.later ? theme.textDim : theme.text
                elide: Text.ElideRight
            }
            Label {
                width: parent.width
                text: (bday.narrow && !bday.across ? bday.day.shortDate : bday.day.date) || " "
                role: "ledger"
                color: theme.textDim
                elide: Text.ElideRight
            }
        }
        Flow {
            id: list
            x: bday.across ? bday.headW : 0
            y: bday.across ? rule.height + theme.space(3) : rule.y + rule.height + theme.space(2)
            width: bday.width - x
            spacing: theme.space(2)
            Label { visible: !(bday.day.items && bday.day.items.length); text: "Nothing"; role: "ledger"; color: theme.textFaint }
            NextTile {
                visible: !!bday.day.next
                entry: bday.day.next
                width: bday.across ? Math.min(list.width, bday.tileW * 2 + list.spacing) : bday.tileW
            }
            Repeater {
                model: (bday.day.items || []).filter(function(e) { return !e.next })
                BoardTile { required property var modelData; entry: modelData; width: bday.tileW }
            }
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
                    onClicked: frame.go("library", { arrange: "WatchState", box: "Watching" })
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
                    onClicked: frame.go("library", { arrange: "WatchState", box: "On hold" })
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

        // ---- The week (#326, #352): Out now first, the posters of what aired and waits to be
        // watched (a click plays it; one not downloaded opens the series to get it); then the
        // board of the days ahead, a column a day. The board takes the whole row: days with
        // episodes share it, empty days keep a narrow column, and a window too narrow for all
        // of them wraps the days onto a second line.
        Column {
            id: weekBox
            objectName: "home-week"
            visible: page.airingRows.length > 0
            width: parent.width
            spacing: theme.space(3)
            readonly property var split: page.weekSplit
            readonly property int outCount: split.out.length
            readonly property bool hasAhead: page.board.some(function(d) { return d.items.length > 0 })
            readonly property real cardW: theme.posterWidth
            Heading {
                text: "This week"; fact: page.weekFact
                action: Button { objectName: "home-week-airing"; flat: true; small: true; text: "Airing now"; icon: "chevron-right"; onClicked: frame.go("airing") }
            }
            // Out now
            Column {
                objectName: "home-week-out"
                visible: weekBox.outCount > 0
                width: parent.width
                spacing: theme.space(3)
                WeekLabel { text: "Out now"; fact: Fmt.plural(weekBox.outCount, "episode", "episodes") }
                Flow {
                    width: parent.width
                    spacing: page.gap
                    Repeater {
                        model: weekBox.split.out
                        Card {
                            required property var modelData
                            objectName: "week-out-" + modelData.card.id
                            item: modelData.card
                            posterWidth: weekBox.cardW
                            nowMs: page.nowMs
                            metaOverride: page.outLine(modelData)
                            onOpened: modelData.state === "late" ? page.openSeries(modelData.card) : page.playSeries(modelData.card)
                            onPlayed: page.playSeries(modelData.card)
                        }
                    }
                }
            }
            Item { visible: weekBox.outCount > 0 && weekBox.hasAhead; width: 1; height: theme.space(3) }
            WeekLabel {
                visible: weekBox.outCount > 0 && weekBox.hasAhead
                text: "Coming up"
                // The heading counts them already when it has nothing more pressing to say
                fact: { var n = weekBox.split.ahead.length + (weekBox.split.next ? 1 : 0); return n && !page.weekFactCounts ? Fmt.plural(n, "episode", "episodes") : "" }
            }
            // The board
            Flow {
                id: board
                objectName: "home-week-board"
                visible: weekBox.hasAhead
                width: parent.width
                spacing: across ? theme.space(4) : theme.space(3)
                readonly property real narrowW: theme.space(20)
                readonly property real minDayW: theme.space(52)
                readonly property int busy: page.board.filter(function(d) { return d.items.length > 0 }).length
                readonly property int idle: page.board.length - busy
                readonly property real dayW: busy ? Math.floor((width - idle * narrowW - (page.board.length - 1) * theme.space(3)) / busy) : 0
                // Too narrow for a column a day: a row a day instead, the days down the page
                readonly property bool across: dayW < minDayW
                // Its days are lanes for the arrow keys (GridKeys)
                readonly property bool gridBoard: true
                readonly property bool gridAcross: across
                Repeater {
                    model: page.board
                    BoardDay {
                        required property var modelData
                        day: modelData
                        narrow: modelData.items.length === 0
                        across: board.across
                        width: across ? board.width : narrow ? board.narrowW : board.dayW
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
            onMore: frame.go("library", { arrange: "WatchState", box: "Watching" })
        }
        Item { width: 1; height: page.catchUpList.length ? theme.space(4) : 0 }

        Shelf {
            objectName: "home-fresh"
            width: parent.width
            heading: "Start something new"
            fact: Fmt.plural(page.freshList.length, "series", "series") + " not started"
            list: page.freshList
            maxRows: 1
            onMore: frame.go("library", { arrange: "WatchState", box: "Not started" })
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
                        scale: theme.reducedMotion ? 1 : (gm.pressed ? 0.97 : 1)
                        activeFocusOnTab: true
                        function open() { frame.go("library", { genre: genre.modelData.name }, genre.modelData.name) }
                        Keys.onReturnPressed: open()
                        Keys.onSpacePressed: open()
                        Rectangle { anchors.fill: parent; anchors.margins: -2; radius: height / 2; visible: genre.activeFocus; color: "transparent"; border.width: 2; border.color: theme.focusRing }
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
                            onClicked: genre.open()
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
            onMore: frame.go("library", { arrange: "WatchState", box: "Dropped" })
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
    // A new install has no library to hold the tracker nudge, so the empty Home carries it
    TrackerNudge {
        objectName: "home-tracker-nudge"
        visible: page.libraryEmpty && wanted
        anchors.horizontalCenter: parent.horizontalCenter
        anchors.bottom: parent.bottom; anchors.bottomMargin: theme.space(10)
        width: Math.min(parent.width - theme.space(16), theme.space(220))
        height: implicitHeight
    }
}
