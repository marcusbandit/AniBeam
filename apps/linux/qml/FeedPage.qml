// The feed: what landed and what lands next. The core builds the cards
// (ListFeed replies Feed(Vec<FeedCard>), one per series with files, sorted in
// Rust); this page picks the sort — Recently released, Coming soon, persisted
// in the core's preferences — lays the grid out, and ticks the scheduled
// countdowns. The card "says which": Aired names the broadcast, Downloaded the
// file's arrival, and a Scheduled card badges what is on disk while its
// countdown chip owns the future.
import QtQuick
import com.marcusrosado.AniBeam

PageScroll {
    id: page
    property var props: ({})
    property string title: "Feed"
    property real scrollY: 0
    function escapePressed() { return false }

    property string sort: Door.preferences.feed_sort === "Upcoming" ? "Upcoming" : "Recent"
    property var cards: []
    property var scheduled: []   // the Upcoming sort's head group
    property var rest: []        // the Upcoming sort's tail group
    property real nowMs: Date.now()

    function reasonText(reason) {
        var kind = reason ? Object.keys(reason)[0] : "None"
        if (kind === "Aired") return "Episode aired " + Fmt.relative(reason.Aired.at, nowMs / 1000)
        if (kind === "Downloaded") return "File downloaded " + Fmt.relative(reason.Downloaded.at, nowMs / 1000)
        return ""
    }
    function scheduledIn(card) {
        return card.reason && card.reason.Scheduled !== undefined ? card.reason.Scheduled : null
    }

    function reload() {
        var r = Door.listFeed(page.sort)
        if (r.error) { frame.toast(r.error.message); return }
        page.cards = r.reply.cards
        var head = [], tail = []
        for (var i = 0; i < page.cards.length; i++) {
            if (scheduledIn(page.cards[i])) head.push(page.cards[i]); else tail.push(page.cards[i])
        }
        page.scheduled = head
        page.rest = tail
    }
    function persist() {
        var p = JSON.parse(JSON.stringify(Door.preferences))
        p.feed_sort = page.sort
        Door.setPreferences(p)
    }
    Timer { id: debounce; interval: 250; onTriggered: page.reload() }
    Timer { interval: 30000; running: page.cards.length > 0; repeat: true; onTriggered: page.nowMs = Date.now() }
    Connections {
        target: Door
        function onSeriesChanged(changed) { debounce.restart() }
        function onPreferencesChanged(p) {
            if (p.feed_sort !== page.sort) { page.sort = p.feed_sort; debounce.restart() }
        }
    }
    Component.onCompleted: {
        page.sort = Door.preferences.feed_sort === "Upcoming" ? "Upcoming" : "Recent"
        reload()
    }

    // One card of the feed grid. The story under the poster is the reason's;
    // a scheduled card's countdown rides the card's own accent slot.
    component FeedCardDelegate: Card {
        required property var modelData
        readonly property var card: modelData || ({})
        item: card.series || ({})
        nowMs: page.nowMs
        readonly property var sched: page.scheduledIn(card)
        metaOverride: sched ? "" : page.reasonText(card.reason)
        countdownOverride: sched ? "Episode " + sched.episode + " in " + shortCountdown(sched.at - page.nowMs / 1000) : ""
        onOpened: card.series ? frame.go("series", { id: card.series.id }, card.series.title) : undefined
    }

    // The head: the title in the voice, the sub line under it, the sort beside it
    Item {
        id: head
        width: parent.width
        height: Math.max(titleText.height + sub.height + theme.space(1), sortRow.height)
        Text {
            id: titleText
            anchors.left: parent.left; anchors.top: parent.top
            text: "Feed"
            color: theme.text
            font.family: theme.fontDisplay
            font.pointSize: theme.typeLarge
            font.weight: Font.DemiBold
        }
        Text {
            id: sub
            anchors.left: parent.left; anchors.top: titleText.bottom; anchors.topMargin: theme.space(1)
            text: page.sort === "Upcoming" ? "What airs next, soonest first." : "Your library, by latest episode."
            color: theme.textDim
            font.family: theme.fontUi
            font.pointSize: theme.typeSmall
        }
        Seg {
            id: sortRow
            anchors.right: parent.right; anchors.verticalCenter: parent.verticalCenter
            options: [{ text: "Recently released", icon: "clock" }, { text: "Coming soon", icon: "calendar-clock" }]
            index: page.sort === "Upcoming" ? 1 : 0
            onPicked: function(i) { page.sort = i === 1 ? "Upcoming" : "Recent"; page.persist(); page.reload() }
        }
    }

    // EmptyState centers itself, and an anchored child inside a positioner
    // breaks the positioner's layout, so each one sits in a plain wrapper the
    // Column can position.
    Item {
        width: parent.width
        height: emptyFeed.height
        visible: emptyFeed.visible
        EmptyState {
            id: emptyFeed
            visible: page.cards.length === 0
            icon: "rss"
            title: "Nothing here yet"
            body: "Add a folder in Settings to build your feed."
        }
    }

    // Upcoming: the scheduled head soonest first, then the rest in recent order
    Column {
        width: parent.width
        spacing: theme.space(4)
        visible: page.sort === "Upcoming" && page.scheduled.length > 0
        SectionHeader { title: "Coming soon"; count: page.scheduled.length }
        Flow {
            width: parent.width
            spacing: theme.space(5)
            Repeater {
                model: page.scheduled
                FeedCardDelegate { }
            }
        }
    }
    Column {
        width: parent.width
        spacing: theme.space(4)
        visible: page.sort === "Upcoming" && page.rest.length > 0
        SectionHeader { title: "Everything else"; count: page.rest.length }
        Flow {
            width: parent.width
            spacing: theme.space(5)
            Repeater {
                model: page.rest
                FeedCardDelegate { }
            }
        }
    }

    // Recent: one grid, newest first
    Flow {
        width: parent.width
        spacing: theme.space(5)
        visible: page.sort === "Recent" && page.cards.length > 0
        Repeater {
            model: page.cards
            FeedCardDelegate { }
        }
    }
}
