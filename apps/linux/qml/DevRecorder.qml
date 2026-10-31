// Developer options (#287): times every page load and keeps the record, so the Developer
// tab and the overlay can say where the time and the memory went. A load runs from the
// navigation to the moment the page has painted and every image it asked for has arrived:
//
//   built       the page file loaded and its Component.onCompleted ran (the Qt thread is
//               blocked for all of it; this is the click-to-anything delay)
//   firstFrame  the first frame painted after that
//   settled     no image still loading, polled every 100 ms, at most 15 s
//
// Each record carries the core calls made in that window with their cost and reply size,
// the decodes with their cost, the x-ray of the finished page and the memory before and
// after. The tour visits every page a few times with images on, then again with them off,
// so the two can be set side by side.
import QtQuick
import QtQuick.Window
import com.marcusrosado.AniBeam

QtObject {
    id: rec
    property Window hostWindow
    property var hostFrame
    property bool enabled: false
    property bool images: true
    property var loads: []
    readonly property var last: loads.length ? loads[0] : null
    property var current: null
    property var tourState: null
    property var tourResult: null
    signal finished(var load)
    signal tourFinished(var result)

    function totals() { return JSON.parse(Dev.images(Number.MAX_VALUE)) }
    function memory() { return JSON.parse(Dev.memory()) }

    function begin(page, label) {
        if (!enabled) return
        var t = totals()
        current = { page: page, label: label || page, at: Dev.now(), images: rec.images, built: -1, firstFrame: -1, settled: -1,
                    frames: 0, maxGap: 0, lastFrame: -1, before: { requests: t.requests, blocked: t.blocked }, memBefore: memory() }
    }
    function built() {
        if (!current || current.built >= 0) return
        current.built = Dev.now() - current.at
        current.lastFrame = Dev.now()
        poll.restart()
    }
    function frame() {
        if (!current || current.built < 0) return
        var now = Dev.now()
        current.frames += 1
        if (current.firstFrame < 0) current.firstFrame = now - current.at
        current.maxGap = Math.max(current.maxGap, now - current.lastFrame)
        current.lastFrame = now
    }
    function check() {
        var c = current
        if (!c || !hostFrame || !hostFrame.pageItem) return
        var scene = JSON.parse(Dev.scene(hostFrame.pageItem, hostWindow))
        var t = totals()
        var waited = Dev.now() - c.at
        if (c.firstFrame >= 0 && scene.images.loading === 0 && t.pending === 0) finish(c, scene, t)
        else if (waited > 15000) { c.timedOut = true; finish(c, scene, t) }
    }
    function finish(c, scene, t) {
        poll.stop()
        c.settled = Dev.now() - c.at
        var calls = JSON.parse(Dev.calls(c.at))
        var decodes = JSON.parse(Dev.images(c.at)).list
        var sum = function(list, key) { var s = 0; list.forEach(function(x) { s += x[key] }); return s }
        c.calls = {
            n: calls.length, ms: sum(calls, "total"), bytes: sum(calls, "bytes"),
            hits: calls.filter(function(x) { return x.hit }).length,
            list: calls.slice().sort(function(a, b) { return b.total - a.total })
        }
        var decodeMs = sum(decodes, "ms")
        c.decodes = {
            n: decodes.length, ms: decodeMs, bytes: sum(decodes, "w") === 0 ? 0 : decodes.reduce(function(s, d) { return s + d.w * d.h * 4 }, 0),
            fileBytes: sum(decodes, "bytes"),
            requests: t.requests - c.before.requests, blocked: t.blocked - c.before.blocked,
            slowest: decodes.slice().sort(function(a, b) { return b.ms - a.ms }).slice(0, 10),
            // The reader is one thread: the decodes ran one after another, so their sum is
            // the floor on how long the last poster waited.
            span: decodes.length ? (decodes[decodes.length - 1].at + decodes[decodes.length - 1].ms) - decodes[0].at : 0
        }
        c.decodes.cached = Math.max(0, c.decodes.requests - c.decodes.blocked - c.decodes.n)
        c.scene = scene
        c.memAfter = memory()
        delete c.lastFrame
        delete c.before
        current = null
        loads = [c].concat(loads).slice(0, 60)
        finished(c)
        if (tourState) tourNext.restart()
    }
    property Timer poll: Timer { interval: 100; repeat: true; onTriggered: rec.check() }

    // ---- The tour: every page, `rounds` times with images on, then the same with them off
    readonly property var tourPages: ["home", "library", "airing", "feed", "watching", "metadata", "downloads", "settings", "series"]
    signal setImages(bool on)
    function tour(rounds) {
        if (!enabled || tourState) return
        var r = Door.listLibrary("All", "", "LastViewed", "Desc", false)
        var series = !r.error && r.reply.series.length ? r.reply.series[0].id : 0
        var steps = []
        ;[true, false].forEach(function(on) {
            for (var i = 0; i < rounds; i++) rec.tourPages.forEach(function(p) {
                if (p === "series" && !series) return
                steps.push({ page: p, props: p === "series" ? { id: series } : {}, images: on, round: i })
            })
        })
        tourState = { steps: steps, at: 0, was: rec.images, results: [] }
        tourResult = null
        tourNext.restart()
    }
    function cancelTour() { if (tourState) { var was = tourState.was; tourState = null; setImages(was) } }
    property Timer tourNext: Timer {
        interval: 350
        onTriggered: {
            var s = rec.tourState
            if (!s) return
            if (s.at > 0 && rec.last) s.results.push({ step: s.steps[s.at - 1], load: rec.last })
            if (s.at >= s.steps.length) { rec.endTour(); return }
            var step = s.steps[s.at++]
            if (step.images !== rec.images) rec.setImages(step.images)
            rec.hostFrame.go(step.page, step.props)
        }
    }
    function median(xs) { if (!xs.length) return 0; var s = xs.slice().sort(function(a, b) { return a - b }); return s[Math.floor(s.length / 2)] }
    function endTour() {
        var s = tourState
        tourState = null
        setImages(s.was)
        var pages = {}
        s.results.forEach(function(r) {
            var p = pages[r.step.page] || (pages[r.step.page] = { page: r.step.page, on: [], off: [] })
            ;(r.step.images ? p.on : p.off).push(r.load)
        })
        var row = function(list) {
            var pick = function(f) { return rec.median(list.map(f)) }
            return {
                built: pick(function(l) { return l.built }), firstFrame: pick(function(l) { return l.firstFrame }),
                settled: pick(function(l) { return l.settled }), calls: pick(function(l) { return l.calls.n }),
                callMs: pick(function(l) { return l.calls.ms }), bytes: pick(function(l) { return l.calls.bytes }),
                decodes: pick(function(l) { return l.decodes.n }), decodeMs: pick(function(l) { return l.decodes.ms }),
                decodedBytes: pick(function(l) { return l.decodes.bytes }), items: pick(function(l) { return l.scene.items }),
                images: pick(function(l) { return l.scene.images.total }), offscreen: pick(function(l) { return l.scene.images.offscreen }),
                rss: pick(function(l) { return l.memAfter.VmRSS || 0 })
            }
        }
        var out = rec.tourPages.filter(function(p) { return pages[p] }).map(function(p) { return { page: p, on: row(pages[p].on), off: row(pages[p].off) } })
        tourResult = { at: Date.now(), rows: out, peak: memory().VmHWM || 0 }
        tourFinished(tourResult)
        hostFrame.go("settings", { tab: "developer" })
    }

    // ---- Words for the clipboard: one block a developer can paste into an issue
    function kb(b) { return (b / 1024).toFixed(b < 10240 ? 1 : 0) + " KB" }
    function mb(b) { return (b / 1048576).toFixed(1) + " MB" }
    function ms(v) { return Math.round(v) + " ms" }
    function describe(l) {
        var lines = [l.label + (l.images ? "" : " (images off)") + ": built " + ms(l.built) + ", first frame " + ms(l.firstFrame) + ", settled " + ms(l.settled) + ", longest frame gap " + ms(l.maxGap),
                     "  calls " + l.calls.n + " (" + l.calls.hits + " cached), " + ms(l.calls.ms) + ", " + kb(l.calls.bytes),
                     "  images asked " + l.decodes.requests + ", decoded " + l.decodes.n + " in " + ms(l.decodes.ms) + " (" + mb(l.decodes.bytes) + " decoded from " + mb(l.decodes.fileBytes) + " on disk), from cache " + l.decodes.cached + ", blocked " + l.decodes.blocked,
                     "  scene " + l.scene.items + " items, " + l.scene.texts + " texts, " + l.scene.images.total + " images (" + l.scene.images.offscreen + " outside the window), " + l.scene.shapes + " shapes",
                     "  memory " + mb((l.memBefore.VmRSS || 0) * 1024) + " to " + mb((l.memAfter.VmRSS || 0) * 1024)]
        l.calls.list.slice(0, 6).forEach(function(c) { lines.push("    " + c.name + " " + c.total.toFixed(1) + " ms, " + kb(c.bytes) + (c.hit ? " cached" : "")) })
        return lines.join("\n")
    }
    function report() {
        var out = ["AniBeam " + Shell.build + ", page loads, newest first", ""]
        if (tourResult) {
            out.push("Tour (median per page, images on / off):")
            tourResult.rows.forEach(function(r) {
                out.push("  " + r.page + ": built " + ms(r.on.built) + " / " + ms(r.off.built) + ", settled " + ms(r.on.settled) + " / " + ms(r.off.settled)
                         + ", " + r.on.items + " items, " + r.on.decodes + " decodes " + ms(r.on.decodeMs) + ", calls " + kb(r.on.bytes))
            })
            out.push("  peak memory " + mb(tourResult.peak * 1024), "")
        }
        loads.forEach(function(l) { out.push(describe(l)) })
        return out.join("\n")
    }
}
