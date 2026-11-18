// The live side of `anibeam ctl` (#1205): requests come in over the control socket
// (control.rs) as a JSON object { steps, out }, one at a time, and run through a Driver of
// their own against the window on screen. The steps are the --drive steps, so a script
// written for one runs in the other. Each request gets one reply: { ok, result, log } or
// { ok: false, error, log }. A page step closes whatever is open over the page first, so
// `go` reaches any page from anywhere, and takes two shorthands the drive steps lack:
// props.find on the series page opens the first series whose title holds those words, and
// a page name is checked against Frame.pages before anything moves.
import QtQuick
import com.marcusrosado.Anibeam

QtObject {
    id: control
    property var hostWindow
    property var hostFrame
    property var evaluate
    property var queue: []
    property var running: null

    property Component driverComponent: Component { Driver {} }
    property Timer startWait: Timer { interval: 100; onTriggered: control.runNext() }
    property Connections link: Connections {
        target: Shell
        function onControlRequested(id, request) { control.take(id, request) }
        function onControlCancelled(id) { control.cancel(id) }
    }

    function reply(id, r) {
        var text
        try { text = JSON.stringify(r) } catch (e) { text = JSON.stringify({ ok: false, error: "the reply is not JSON: " + e.message, log: r.log }) }
        Shell.controlReply(id, text)
    }

    function take(id, text) {
        var req
        try { req = JSON.parse(text) } catch (e) { reply(id, { ok: false, error: "bad JSON: " + e.message, log: [] }); return }
        queue = queue.concat([{ id: id, req: req }])
        if (!running) Qt.callLater(control.runNext)
    }

    // The caller stopped waiting: a queued request is dropped, a running one fails where it is
    function cancel(id) {
        queue = queue.filter(function(j) { return j.id !== id })
        if (running && running.jobId === id) running.fail("cancelled: the caller stopped waiting")
    }

    function runNext() {
        if (running || !queue.length) return
        // A window just started by `ctl open` answers before its frame is built: the request
        // waits for it, up to 30 s
        var f = hostFrame.item
        if (!f || !f.nav) {
            if (!queue[0].since) queue[0].since = Date.now()
            if (Date.now() - queue[0].since < 30000) { startWait.start(); return }
        }
        var job = queue[0]
        queue = queue.slice(1)
        if (!f || !f.nav) { reply(job.id, { ok: false, error: "the window did not finish starting", log: [] }); Qt.callLater(control.runNext); return }
        var steps
        try { steps = prepare(job.req.steps) } catch (e) { reply(job.id, { ok: false, error: e.message || String(e), log: [] }); Qt.callLater(control.runNext); return }
        var d = driverComponent.createObject(control, { hostWindow: hostWindow, hostFrame: hostFrame, evaluate: evaluate,
                                                         outDir: String(job.req.out || ""), live: true,
                                                         beforePage: control.closeOverlays })
        d.jobId = job.id
        running = d
        d.finish = function(ok, message) {
            var r = ok ? { ok: true, result: plain(d.result, 0), log: d.lines } : { ok: false, error: message, log: d.lines }
            reply(job.id, r)
            running = null
            d.destroy(100)
            Qt.callLater(control.runNext)
        }
        d.run(steps)
    }

    // Checks the steps' outer shape and turns the shorthands into what the driver takes
    function prepare(steps) {
        if (!Array.isArray(steps) || steps.length === 0) throw new Error("steps is a non-empty JSON array")
        return steps.map(function(s, i) {
            if (s === null || typeof s !== "object" || Array.isArray(s)) throw new Error("step " + (i + 1) + " is not an object")
            if (s.page === undefined) return s
            var page = String(s.page)
            if (!hostFrame.item.pages[page]) throw new Error("no page " + page + "; pages: " + Object.keys(hostFrame.item.pages).join(", "))
            var props = Object.assign({}, s.props || {})
            if (page === "series" && props.id === undefined && props.find !== undefined) {
                var r = Door.listSeries("All", String(props.find), "Alpha", "Asc", false)
                if (r.error || !r.reply.series.length) throw new Error("no series in the library matches " + JSON.stringify(props.find))
                props = { id: r.reply.series[0].id }
            }
            return Object.assign({}, s, { page: page, props: props })
        })
    }

    // Everything opened over the page, newest first: sheets, pickers, the drawer, the first
    // run. Bounded, in case a closer keeps itself on the stack.
    function closeOverlays() {
        var stack = hostFrame.item.escapeStack
        for (var n = 0; n < 12; n++) {
            var top = stack.top()
            if (!top) return
            top.closer.close()
            stack.pop(top.closer)
        }
    }

    // A result as JSON can carry it: a QML item becomes its type name, and depth is capped,
    // since an object's properties can lead back up the whole item tree
    function plain(v, depth) {
        if (v === undefined) return null
        if (v === null || typeof v === "number" || typeof v === "boolean" || typeof v === "string") return v
        if (typeof v === "function") return String(v)
        if (depth > 6) return "..."
        if (Array.isArray(v)) return v.slice(0, 2000).map(function(x) { return plain(x, depth + 1) })
        // A QObject prints as Type(0x...): its type name stands for it
        var s = String(v)
        if (/\(0x[0-9a-f]+/.test(s)) return s.replace(/\(0x[0-9a-f]+.*$/, "")
        var o = {}
        Object.keys(v).forEach(function(k) { o[k] = plain(v[k], depth + 1) })
        return o
    }

    // ---- what the commands' evals call

    // True once the page the trail is on is the one built and shown
    function settled() {
        var f = hostFrame.item
        return !!(running && f && f.nav && running.pageReady(f.nav.current.page))
    }
    // The first visible item with this objectName, open popups included
    function find(name) { return running ? running.findItem(name) : null }


    function where() {
        var f = hostFrame.item, c = f.nav.current
        var r = { page: c.page, props: c.props, label: c.label, canBack: f.nav.canBack, canForward: f.nav.canForward,
                  window: { width: hostWindow.width, height: hostWindow.height, active: hostWindow.active } }
        if (c.page === "settings" && f.pageItem && f.pageItem.current !== undefined) r.tab = f.pageItem.current
        var open = f.escapeStack.entries.map(function(e) { return e.kind })
        if (open.length) r.open = open
        return r
    }
    // Each page in Frame.pages and the props it opens with, where they are known
    function pages() {
        var known = {
            home: {}, library: { q: "search", sort: "", group: "", genre: "", arrange: "", box: "" },
            airing: { season: "", filter: "" }, feed: {}, watching: { tab: "" }, stats: { year: "" },
            metadata: { q: "search" }, downloads: { query: "search", series: "series id" },
            settings: { tab: "library accounts appearance playback downloads keys about developer", focus: "a setting's id" },
            series: { id: "series id", find: "words of the title" }, player: { file: "path", speed: "" },
            show: { anilist: "AniList id" }, recommended: {}
        }
        var r = {}
        Object.keys(hostFrame.item.pages).forEach(function(p) { r[p] = known[p] || {} })
        return r
    }
}
