// The navigation trail. Back goes to the page you came from, labelled with its name, and
// restores its scroll; Forward goes back to where Back left, until a new page is opened;
// sideways moves keep the trail; twelve entries at most. The frame reads the leaving page's
// scrollY and restores it on the way back.
//
// The rail's pages remember how they were left, their props (a search, a season) and their
// scroll, so going back to one from the rail finds it as it was.
import QtQuick

QtObject {
    id: nav
    property var trail: []
    // What Back left, newest last, for Forward
    property var ahead: []
    // Per rail page, how it was left: { props, scrollY }
    property var remembered: ({})
    readonly property bool canBack: trail.length > 0 || current.page !== "home"
    readonly property bool canForward: ahead.length > 0
    property var current: ({ page: "home", props: {}, label: "Home" })
    property real pendingScroll: -1
    signal changed()

    readonly property var labels: ({ home: "Home", library: "Library", airing: "Airing now", feed: "Feed", watching: "Watching", metadata: "Metadata", downloads: "Downloads",
                                     settings: "Settings", series: "Series", player: "Player" })
    readonly property var railPages: ["home", "library", "airing", "feed", "watching", "metadata", "downloads", "settings"]
    readonly property string backLabel: trail.length ? trail[trail.length - 1].label : "Home"
    readonly property int railIndex: {
        var i = railPages.indexOf(current.page)
        if (i >= 0) return i
        for (var k = 0; k < trail.length; k++) { var j = railPages.indexOf(trail[k].page); if (j >= 0) return j }
        return 0
    }

    function key(e) { return e.page + ":" + JSON.stringify(e.props || {}) }
    function labelOf(page, label) { return label || labels[page] || "Back" }

    function remember(scrollY) {
        if (railPages.indexOf(current.page) < 0) return
        var m = Object.assign({}, remembered)
        m[current.page] = { props: current.props, scrollY: scrollY || 0 }
        remembered = m
    }
    // A rail page, as it was left
    function openRail(page, leavingScroll) {
        var m = remembered[page]
        open(page, m ? m.props : {}, undefined, leavingScroll)
        if (m) pendingScroll = m.scrollY
    }
    // Descend: the page we leave joins the trail with its scroll, de-duplicated and capped.
    function open(page, props, label, leavingScroll) {
        remember(leavingScroll)
        ahead = []
        var here = { page: current.page, props: current.props, label: current.label, scrollY: leavingScroll || 0 }
        var t = trail.filter(function(e) { return key(e) !== key(here) })
        t.push(here)
        trail = t.slice(-12)
        current = { page: page, props: props || {}, label: labelOf(page, label) }
        pendingScroll = 0
        changed()
    }
    // Sideways: the trail stays, so Back leaves the level rather than the episode.
    function replace(page, props, label) {
        current = { page: page, props: props || {}, label: labelOf(page, label) }
        pendingScroll = 0
        changed()
    }
    function back(leavingScroll) {
        remember(leavingScroll)
        var here = { page: current.page, props: current.props, label: current.label, scrollY: leavingScroll || 0 }
        if (!trail.length) {
            if (current.page === "home") return
            ahead = ahead.concat([here])
            current = { page: "home", props: {}, label: "Home" }
            pendingScroll = 0
            changed()
            return
        }
        var t = trail.slice()
        var target = t.pop()
        trail = t
        ahead = ahead.concat([here]).slice(-12)
        current = { page: target.page, props: target.props, label: target.label }
        pendingScroll = target.scrollY || 0
        changed()
    }
    function forward(leavingScroll) {
        if (!ahead.length) return
        remember(leavingScroll)
        var a = ahead.slice()
        var target = a.pop()
        trail = trail.concat([{ page: current.page, props: current.props, label: current.label, scrollY: leavingScroll || 0 }]).slice(-12)
        ahead = a
        current = { page: target.page, props: target.props, label: target.label }
        pendingScroll = target.scrollY || 0
        changed()
    }
    // A page changed its own label (a series title arrived)
    function relabel(label) { current = { page: current.page, props: current.props, label: label } }
}
