// The feed: what landed and what lands next, as a mosaic wall on the library's module. The
// core builds the cards (ListFeed replies Feed(Vec<FeedCard>), one per series with files,
// sorted in Rust); this page picks the sort (Recently released, Coming soon, persisted in
// the core's preferences), cuts the sections and ticks the countdowns. A card "says which":
// Aired names the broadcast, Downloaded the file's arrival, and a Scheduled card carries
// its countdown pill. The newest episodes (this week's) are the heroes, with Play.
import QtQuick
import com.marcusrosado.AniBeam
import "mosaic.js" as Mosaic

Item {
    id: page
    objectName: "feed-page"
    property var props: ({})
    property string title: "Feed"
    property alias scrollY: wall.scrollY
    function escapePressed() { return false }

    property string sort: Door.preferences.feed_sort === "Upcoming" ? "Upcoming" : "Recent"
    property var cards: []
    property real nowMs: Date.now()
    // The clock heroes are picked against: set on reload, so the wall is not repacked as
    // the countdowns tick
    property real packedAt: Date.now()

    function reasonText(reason) {
        var kind = reason ? Object.keys(reason)[0] : "None"
        if (kind === "Aired") return "Episode aired " + Fmt.relative(reason.Aired.at, nowMs / 1000)
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

    // One wall, built once: Recent is a single unlabelled section, Upcoming the scheduled
    // head ("Coming soon", in the accent) and the rest. The countdown text is the Card's
    // own, so the sections do not depend on the ticking clock.
    readonly property var sections: {
        var now = page.packedAt / 1000
        var week = 7 * 86400
        var head = [], tail = []
        for (var i = 0; i < cards.length; i++) {
            var e = entry(cards[i])
            if (e.id === undefined) continue
            if (sort === "Upcoming" && e.countdown) head.push(e); else tail.push(e)
        }
        if (sort === "Recent") {
            var fresh = tail.filter(function(e) { return e.at > 0 && now - e.at <= week })
            return tail.length ? [{ label: "", count: tail.length, cards: tail, heroes: fresh }] : []
        }
        var out = []
        if (head.length) out.push({ label: "Coming soon", count: head.length, accent: true, cards: head,
                                    heroes: head.filter(function(e) { return e.at - now <= week }) })
        if (tail.length) out.push({ label: "Everything else", count: tail.length, cards: tail, heroes: [] })
        return out
    }

    function reload() {
        var r = Door.listFeed(page.sort)
        if (r.error) { frame.toast(r.error.message); return }
        page.packedAt = Date.now()
        page.nowMs = page.packedAt
        page.cards = r.reply.cards
    }
    function persist() {
        var p = JSON.parse(JSON.stringify(Door.preferences))
        p.feed_sort = page.sort
        Door.setPreferences(p)
    }
    // A hero's Play: the series' next episode, read only now; nothing next opens the series
    function playSeries(s) {
        var r = Door.getSeries(s.id)
        var d = !r.error ? r.reply.detail : null
        var file = d ? (d.next_up ? d.next_up : (s.kind === "Movie" && d.episodes && d.episodes.length ? d.episodes[0].file : null)) : null
        if (file) frame.go("player", { file: file }, s.title)
        else frame.go("series", { id: s.id }, s.title)
    }
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

    MosaicWall {
        id: wall
        anchors.fill: parent
        sections: page.sections
        nowMs: page.nowMs
        onOpened: function(e) { frame.go("series", { id: e.id }, e.item.title) }
        onPlayed: function(e) { page.playSeries(e.item) }

        MosaicWall.HeaderTile {
            need: feedTitle.implicitWidth
            minSpan: 2
            flex: true
            Label {
                id: feedTitle
                x: parent.pad
                anchors.verticalCenter: parent.verticalCenter
                text: "Feed"
                role: "display"
            }
        }
        MosaicWall.HeaderTile {
            need: sortSeg.implicitWidth
            Seg {
                id: sortSeg
                objectName: "feed-sort"
                x: parent.pad
                anchors.verticalCenter: parent.verticalCenter
                options: [{ text: "Recently released", icon: "clock" }, { text: "Coming soon", icon: "calendar-clock" }]
                index: page.sort === "Upcoming" ? 1 : 0
                onPicked: function(i) { page.sort = i === 1 ? "Upcoming" : "Recent"; page.persist(); page.reload() }
            }
        }
        MosaicWall.HeaderTile {
            need: countRow.implicitWidth
            Row {
                id: countRow
                anchors.right: parent.right; anchors.rightMargin: parent.pad
                anchors.verticalCenter: parent.verticalCenter
                spacing: theme.space(1.5)
                Label {
                    id: countFigure
                    text: page.cards.length
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

        // Nothing to show: one tile filling the page, saying what to do
        body: Component {
            Tile {
                implicitHeight: theme.space(60)
                EmptyState {
                    icon: "rss"
                    title: "Nothing here yet"
                    body: "Add a folder in Settings to build your feed."
                    Button { text: "Settings"; icon: "settings"; onClicked: frame.go("settings") }
                }
            }
        }
    }
}
