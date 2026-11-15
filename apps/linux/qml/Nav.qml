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
    property var current: entry("home", {}, "")
    property real pendingScroll: -1
    signal changed()

    // Each page's name by message ID; a page given its own label (a series title) keeps it
    readonly property var labels: ({ home: "nav-home", library: "nav-library", airing: "nav-airing", feed: "nav-feed", watching: "nav-watching", stats: "nav-stats", metadata: "nav-metadata", downloads: "nav-downloads",
                                     settings: "nav-settings", series: "nav-series", player: "nav-player", show: "nav-show", recommended: "nav-recommended" })
    readonly property var railPages: ["home", "library", "airing", "feed", "watching", "metadata", "downloads", "settings", "stats"]
    readonly property string backLabel: trail.length ? trail[trail.length - 1].label : words.t("nav-home")
    readonly property int railIndex: {
        var i = railPages.indexOf(current.page)
        if (i >= 0) return i
        for (var k = 0; k < trail.length; k++) { var j = railPages.indexOf(trail[k].page); if (j >= 0) return j }
        return 0
    }

    function key(e) { return e.page + ":" + JSON.stringify(e.props || {}) }
    function labelOf(page, label) { return label || words.t(labels[page] || "action-back") }
    // An entry: `own` says the label came with it (a series title) and is kept as it came;
    // without it the label is the page's name from the catalog, looked up again when the
    // language changes, across the trail and Forward too.
    function entry(page, props, label, scrollY) {
        return { page: page, props: props || {}, label: labelOf(page, label), own: !!label, scrollY: scrollY || 0 }
    }
    function keep(e, scrollY) { return entry(e.page, e.props, e.own ? e.label : "", scrollY) }
    readonly property string lang: words.lang
    onLangChanged: {
        function again(e) { return e.own ? e : Object.assign({}, e, { label: labelOf(e.page, "") }) }
        trail = trail.map(again)
        ahead = ahead.map(again)
        current = again(current)
    }

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
        var here = keep(current, leavingScroll)
        var t = trail.filter(function(e) { return key(e) !== key(here) })
        t.push(here)
        trail = t.slice(-12)
        current = entry(page, props, label)
        pendingScroll = 0
        changed()
    }
    // Sideways: the trail stays, so Back leaves the level rather than the episode.
    function replace(page, props, label) {
        current = entry(page, props, label)
        pendingScroll = 0
        changed()
    }
    function back(leavingScroll) {
        remember(leavingScroll)
        var here = keep(current, leavingScroll)
        if (!trail.length) {
            if (current.page === "home") return
            ahead = ahead.concat([here])
            current = entry("home", {}, "")
            pendingScroll = 0
            changed()
            return
        }
        var t = trail.slice()
        var target = t.pop()
        trail = t
        ahead = ahead.concat([here]).slice(-12)
        current = keep(target)
        pendingScroll = target.scrollY || 0
        changed()
    }
    function forward(leavingScroll) {
        if (!ahead.length) return
        remember(leavingScroll)
        var a = ahead.slice()
        var target = a.pop()
        trail = trail.concat([keep(current, leavingScroll)]).slice(-12)
        ahead = a
        current = keep(target)
        pendingScroll = target.scrollY || 0
        changed()
    }
    // A page changed its own label (a series title arrived)
    function relabel(label) { current = entry(current.page, current.props, label) }
}
