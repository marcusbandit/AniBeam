// The feed: what landed and what lands next, as sections of uniform cards. The core builds
// the cards (ListFeed replies Feed(Vec<FeedCard>), one per series with files, sorted in
// Rust); this page picks the sort (Recently released, Coming soon, persisted in the core's
// preferences), cuts the sections and ticks the countdowns. A card "says which": Aired
// names the broadcast, Downloaded the file's arrival, and a Scheduled card carries its
// countdown. Recent is "This week" over "Earlier"; Coming soon is the scheduled ones over
// "Everything else". The head is the library's: the title, the one-line summary beside
// it, the sort at the right.
import QtQuick
import com.marcusrosado.AniBeam

Item {
    id: page
    objectName: "feed-page"
    property var props: ({})
    property string title: "Feed"
    property alias scrollY: scroll.contentY
    function escapePressed() { return false }

    property string sort: Door.preferences.feed_sort === "Upcoming" ? "Upcoming" : "Recent"
    property var cards: []
    property real nowMs: Date.now()
    // The clock the sections are cut against: set on reload, so the lists do not move
    // between This week and Earlier as the countdowns tick
    property real cutAt: Date.now()

    function reasonText(reason) {
        var kind = reason ? Object.keys(reason)[0] : "None"
        if (kind === "Aired") return Fmt.episodeName("", reason.Aired.episode) + " aired " + Fmt.relative(reason.Aired.at, nowMs / 1000)
        if (kind === "Downloaded") return "File downloaded " + Fmt.relative(reason.Downloaded.at, nowMs / 1000)
        return ""
    }
    function reasonAt(reason) {
        if (!reason) return 0
        if (reason.Aired) return reason.Aired.at
        if (reason.Downloaded) return reason.Downloaded.at
        return 0
    }
    function scheduledIn(card) {
        return card.reason && card.reason.Scheduled !== undefined ? card.reason.Scheduled : null
    }
    function entry(card) {
        var s = card.series || {}
        var sched = scheduledIn(card)
        return { id: s.id, item: s, meta: sched ? "" : reasonText(card.reason), countdown: sched, at: sched ? sched.at : reasonAt(card.reason) }
    }
    function shortCountdown(secs) {
        var t = Math.max(0, Math.floor(secs))
        var d = Math.floor(t / 86400), h = Math.floor(t % 86400 / 3600), m = Math.floor(t % 3600 / 60)
        return d > 0 ? d + "d " + h + "h" : h > 0 ? h + "h " + m + "m" : Math.max(1, m) + "m"
    }

    // The sections, cut once per reload. Recent: this week's over the earlier ones.
    // Upcoming: the scheduled ones (soonest first, in the accent) over everything else.
    readonly property var sections: {
        var now = page.cutAt / 1000
        var week = 7 * 86400
        var head = [], tail = []
        for (var i = 0; i < cards.length; i++) {
            var e = entry(cards[i])
            if (e.id === undefined) continue
            if (sort === "Upcoming" && e.countdown) head.push(e); else tail.push(e)
        }
        var out = []
        if (sort === "Recent") {
            var fresh = tail.filter(function(e) { return e.at > 0 && now - e.at <= week })
            var earlier = tail.filter(function(e) { return !(e.at > 0 && now - e.at <= week) })
            if (fresh.length) out.push({ title: "This week", cards: fresh, accent: true, fact: Fmt.plural(fresh.length, "new episode", "new episodes") })
            if (earlier.length) out.push({ title: "Earlier", cards: earlier, accent: false,
                                           fact: earlier[0].at > 0 ? "Latest out " + Fmt.relative(earlier[0].at, now) : "" })
            return out
        }
        // The head line already names the next one; the section counts this week's
        var thisWeek = head.filter(function(e) { return e.at - now <= week }).length
        var recent = tail.filter(function(e) { return e.at > 0 && now - e.at <= week }).length
        if (head.length) out.push({ title: "Coming soon", cards: head, accent: true,
                                    fact: thisWeek > 0 ? thisWeek + " this week" : "" })
        if (tail.length) out.push({ title: "Everything else", cards: tail, accent: false,
                                    fact: recent > 0 ? recent + " out this week"
                                        : tail[0].at > 0 ? "Latest out " + Fmt.relative(tail[0].at, now) : "" })
        return out
    }

    // The head's one line: what this sort has to say right now
    readonly property string summary: {
        var now = nowMs / 1000
        if (sort === "Upcoming") {
            var next = null
            for (var i = 0; i < cards.length; i++) {
                var sc = scheduledIn(cards[i])
                if (sc && sc.at > now && (!next || sc.at < next.at)) next = { at: sc.at, title: cards[i].series ? cards[i].series.title : "" }
            }
            if (!next) return "Nothing scheduled yet."
            return "Next: " + next.title + " in " + shortCountdown(next.at - now)
        }
        var fresh = 0
        for (var j = 0; j < cards.length; j++) {
            var at = reasonAt(cards[j].reason)
            if (at > 0 && now - at <= 7 * 86400) fresh++
        }
        return fresh === 0 ? "Nothing new this week." : fresh === 1 ? "1 new episode this week." : fresh + " new episodes this week."
    }

    function reload() {
        var keep = scroll.contentY
        var r = Door.listFeed(page.sort)
        if (r.error) { frame.toast(r.error.message); return }
        page.cutAt = Date.now()
        page.nowMs = page.cutAt
        page.cards = r.reply.cards
        scroll.contentY = Math.max(0, Math.min(keep, Math.max(0, scroll.contentHeight - scroll.height)))
    }
    function persist() {
        var p = JSON.parse(JSON.stringify(Door.preferences))
        p.feed_sort = page.sort
        Door.setPreferences(p)
    }
    // Play: the series' next episode, read only now; nothing next opens the series
    function playSeries(s) {
        var r = Door.getSeries(s.id)
        var d = !r.error ? r.reply.detail : null
        var file = d ? (d.next_up ? d.next_up : (s.kind === "Movie" && d.episodes && d.episodes.length ? d.episodes[0].file : null)) : null
        if (file) frame.go("player", { file: file }, s.title)
        else frame.go("series", { id: s.id }, s.title)
    }
    function openSeries(s) { frame.go("series", { id: s.id }, s.title) }
    Timer { id: debounce; interval: 250; onTriggered: page.reload() }
    Timer { interval: 30000; running: page.cards.length > 0; repeat: true; onTriggered: page.nowMs = Date.now() }
    Connections {
        target: Door
        function onSeriesChanged(changed) { debounce.restart() }
        function onSeriesRemoved(ids) { debounce.restart() }
        function onPreferencesChanged(p) {
            var want = p.feed_sort === "Upcoming" ? "Upcoming" : "Recent"
            if (want !== page.sort) { page.sort = want; debounce.restart() }
        }
    }
    Component.onCompleted: reload()

    // ---- The grid's geometry: as many columns as the width holds at the poster width
    readonly property real gap: theme.space(4)
    readonly property int cols: Math.max(1, Math.floor((scroll.innerWidth + gap) / (theme.posterWidth + gap)))
    readonly property real cardWidth: Math.floor((scroll.innerWidth - gap * (cols - 1)) / cols)

    PageScroll {
        id: scroll
        anchors.fill: parent
        readonly property real innerWidth: width - theme.space(16)

        // ---- The head: the title, this sort's one line beside it, the sort at the right
        Item {
            width: parent.width
            height: theme.space(20)
            Label {
                id: titleLabel
                anchors.left: parent.left
                anchors.verticalCenter: parent.verticalCenter
                text: "Feed"
                role: "display"
                font.pointSize: theme.typePage
            }
            Label {
                objectName: "feed-summary"
                anchors.left: titleLabel.right
                anchors.leftMargin: theme.space(3)
                anchors.right: sortSeg.left
                anchors.rightMargin: theme.space(4)
                anchors.baseline: titleLabel.baseline
                text: page.summary
                role: "ui"
                color: theme.textDim
                elide: Text.ElideRight
            }
            Seg {
                id: sortSeg
                objectName: "feed-sort"
                anchors.right: parent.right
                anchors.verticalCenter: parent.verticalCenter
                options: [{ text: "Recently released", icon: "clock", name: "recent" }, { text: "Coming soon", icon: "calendar-clock", name: "upcoming" }]
                index: page.sort === "Upcoming" ? 1 : 0
                onPicked: function(i) { page.sort = i === 1 ? "Upcoming" : "Recent"; page.persist(); page.reload() }
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
                objectName: "feed-section-" + String(modelData.title).toLowerCase().replace(/[^a-z0-9]+/g, "-")
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
                        Card {
                            required property var modelData
                            objectName: "tile-" + modelData.id
                            item: modelData.item
                            metaOverride: modelData.meta
                            countdown: modelData.countdown
                            posterWidth: page.cardWidth
                            nowMs: page.nowMs
                            onOpened: page.openSeries(modelData.item)
                            onPlayed: page.playSeries(modelData.item)
                        }
                    }
                }
            }
        }

        // Nothing to show: what to do, in a plain box in the column
        Item {
            visible: page.sections.length === 0
            width: parent.width
            height: visible ? theme.space(60) : 0
            EmptyState {
                icon: "rss"
                title: "Nothing here yet"
                body: "Add a folder in Settings to build your feed."
                Button { text: "Settings"; icon: "settings"; onClicked: frame.go("settings") }
            }
        }
    }
}
