// The navigation trail. Back goes to the page you came from, labelled with its name, and
// restores its scroll; sideways moves keep the trail; twelve entries at most. The frame
// reads the leaving page's scrollY and restores it on the way back.
import QtQuick

QtObject {
    id: nav
    property var trail: []
    property var current: ({ page: "library", props: {}, label: "Library" })
    property real pendingScroll: -1
    signal changed()

    readonly property var labels: ({ library: "Library", feed: "Feed", watching: "Watching", metadata: "Metadata", settings: "Settings",
                                     subscriptions: "Subscriptions", series: "Series", player: "Player" })
    readonly property var railPages: ["library", "feed", "watching", "metadata", "settings"]
    readonly property string backLabel: trail.length ? trail[trail.length - 1].label : "Library"
    readonly property int railIndex: {
        var i = railPages.indexOf(current.page)
        if (i >= 0) return i
        if (current.page === "subscriptions") return 4
        for (var k = 0; k < trail.length; k++) { var j = railPages.indexOf(trail[k].page); if (j >= 0) return j }
        return 0
    }

    function key(e) { return e.page + ":" + JSON.stringify(e.props || {}) }
    function labelOf(page, label) { return label || labels[page] || "Back" }

    // Descend: the page we leave joins the trail with its scroll, de-duplicated and capped.
    function open(page, props, label, leavingScroll) {
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
    function back() {
        if (!trail.length) { if (current.page !== "library") replace("library", {}, "Library"); return }
        var t = trail.slice()
        var target = t.pop()
        trail = t
        current = { page: target.page, props: target.props, label: target.label }
        pendingScroll = target.scrollY || 0
        changed()
    }
    // A page changed its own label (a series title arrived)
    function relabel(label) { current = { page: current.page, props: current.props, label: label } }
}
