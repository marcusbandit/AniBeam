// --drive <script.json>: runs a JSON array of steps against the window, offscreen, once the
// frame is up (Main.qml starts it at the moment a --shoot would grab). Input goes through
// Shell.drive*, which enter at QtGui's window-system layer the way QtTest does, so a click
// here is a click to MouseArea, the pointer handlers, Popup's press-outside and Shortcut.
// Every step prints "DRIVE STEP <n>: <summary>"; any failure prints "DRIVE FAIL: <reason>"
// and exits 1; the end of the script exits 0. Steps (one key each, besides the options):
//   {"wait": ms}                                               a fixed pause; for animation time
//   {"until": "js", "timeout": ms, "expect": <json>, "as": "k"}  polls the JS every 25 ms until it
//                                                              is truthy (or equals "expect"), then
//                                                              goes on; past "timeout" (default 5000)
//                                                              the run fails. A throw while polling
//                                                              counts as not ready yet.
//   {"click": [x, y] | "objectName", "button": "left"|"right"|"middle", "double": true}
//   {"hover": [x, y] | "objectName"}
//   {"drag": [x1, y1, x2, y2], "steps": n, "button": ...}       press, n moves held, release;
//                                                              "release": false keeps it held
//   {"release": [x, y], "button": ...}                         lets go of a held drag there
//   {"wheel": [x, y, dy] | "objectName", "dy": n, "dx": n}     angle delta; 120 is a notch
//   {"key": "Escape" | "Ctrl+K" | "a" | ...}
//   {"type": "text"}
//   {"shot": "name"}                                           <out>/<name>.png
//   {"log": "text"}
//   {"eval": "js", "expect": <json>, "as": "k"}                in Main.qml's scope; the
//                                                              result is vars.k to later evals
//   {"page": "name", "props": {...}}                           opens the page; with "shot" it waits
//                                                              for the page to be built, lets it
//                                                              settle ("settle": ms, default 300),
//                                                              and grabs <out>/<shot>.png; "until": "js"
//                                                              adds a readiness condition on top
//   {"pages": [{"page": "name", "props": {...}, "shot": "file"}, ...]}
//                                                              the same, for each entry in turn
// An objectName target is the first visible item with that name, depth first from the
// window's content item, which holds the Controls overlay and so every open Popup too.
import QtQuick
import com.marcusrosado.AniBeam

QtObject {
    id: driver
    // host, not window/frame: a property named like the id it is bound from shadows it
    property var hostWindow
    property var hostFrame
    property var evaluate          // function(src): Main.qml's eval, so ids resolve there
    property string outDir

    property var steps: []
    property var vars: ({})        // {"eval": ..., "as": "k"} results, readable in evals as vars.k
    property int at: 0
    property var queue: []
    property bool started: false
    property bool finished: false
    property var point: null       // the current step's resolved target

    property Timer timer: Timer { repeat: false; onTriggered: driver.pump() }
    property Timer poller: Timer { interval: 25; repeat: true; onTriggered: driver.poll() }
    property var waiting: null     // the {"until"} being polled

    readonly property int gap: 30  // between the move, the press and the release of a click
    readonly property int settle: 50

    function log(s) { Shell.driveLog(s) }
    function fail(reason) {
        if (finished) return
        finished = true; timer.stop(); poller.stop(); queue = []
        log("DRIVE FAIL: " + reason)
        Qt.exit(1)
    }

    function start() {
        if (started) return
        started = true
        try { steps = JSON.parse(Shell.drive) } catch (e) { fail("bad JSON: " + e.message); return }
        if (!Array.isArray(steps)) { fail("a drive script is a JSON array of steps"); return }
        try { hostWindow.requestActivate() } catch (e) { fail(e.message); return }
        next()
    }

    function next() {
        if (finished) return
        if (at >= steps.length) {
            finished = true
            log("DRIVE DONE: " + steps.length + " steps")
            Qt.exit(0)
            return
        }
        var step = steps[at]; at++
        try { queue = expand(step) } catch (e) { fail("step " + at + ": " + (e.message || e)); return }
        pump()
    }

    // Runs the step's actions in order. An action returns a delay in ms before the next
    // one, or -1 when it finishes on its own and calls pump itself (a grab).
    function pump() {
        while (!finished && queue.length) {
            var a = queue.shift(), r
            try { r = a() } catch (e) { fail("step " + at + ": " + (e.message || e)); return }
            if (r === -1) return
            if (r > 0) { timer.interval = r; timer.start(); return }
        }
        if (!finished) Qt.callLater(driver.next)
    }

    // ---- until: one probe of the condition, and the poll that repeats it

    // How a result prints: JSON where it can be (strings quoted, arrays, records); a QObject
    // or a function prints as itself. The one printer for evals, expects and timeouts.
    function show(r) {
        var s
        try { s = r === undefined ? "undefined" : JSON.stringify(r) } catch (e) { s = undefined }
        return s === undefined ? String(r) : s
    }

    // True once `name` is the page on screen. nav.current flips at once, but Frame.loadPage
    // swaps the page in on a Qt.callLater, so a script that waits on nav.current alone runs
    // ahead of a page that does not exist yet. The loaded item must be the Frame.pages file
    // for `name` (an item prints as its type name followed by _QMLTYPE).
    function pageReady(name) {
        var f = hostFrame.item
        if (!f || f.nav.current.page !== name || !f.pageItem) return false
        var file = f.pages[name] || "PagePlaceholder.qml"
        return String(f.pageItem).indexOf(file.replace(".qml", "") + "_QMLTYPE") === 0
    }

    // Evaluates the condition once; true when it holds. A throw is "not ready" (the page or
    // the item it reads may not exist yet). Only the raw result is kept per poll; it is
    // formatted when the timeout message needs it, or to compare with "expect".
    function probe(w) {
        if (w.check) return w.check(w)
        var r
        try { r = evaluate(w.src, vars); w.threw = undefined } catch (e) { w.threw = e.message || String(e); return false }
        w.last = r
        var ok = w.hasExpect ? JSON.stringify(w.expect) === show(r) : !!r
        if (ok && w.as !== undefined) vars[w.as] = r
        return ok
    }
    function poll() {
        var w = waiting
        if (!w || finished) { poller.stop(); return }
        if (probe(w)) {
            poller.stop(); waiting = null
            log("DRIVE UNTIL: ready after " + (Date.now() - w.started) + " ms")
            pump()
        } else if (Date.now() >= w.deadline) {
            poller.stop(); waiting = null
            fail("step " + w.n + ": " + (w.what || "until") + " timed out after " + w.timeout + " ms: " + w.src + " (last: " + (w.threw !== undefined ? "threw " + w.threw : show(w.last)) + ")")
        }
    }

    function summary(step) {
        var s = JSON.stringify(step)
        return s.length > 160 ? s.slice(0, 157) + "..." : s
    }

    // ---- targets

    function shown(item) {
        if (!item.visible || item.width <= 0 || item.height <= 0) return false
        for (var p = item; p; p = p.parent) if (p.opacity <= 0) return false
        return true
    }
    function findItem(name) {
        var stack = [hostWindow.contentItem]
        while (stack.length) {
            var it = stack.pop()
            if (it.objectName === name && shown(it)) return it
            var kids = it.children
            // reversed onto the stack, so the walk visits them in declaration order
            for (var i = kids.length - 1; i >= 0; i--) if (kids[i].visible) stack.push(kids[i])
        }
        return null
    }
    function resolve(target) {
        if (Array.isArray(target)) {
            if (target.length < 2 || typeof target[0] !== "number" || typeof target[1] !== "number")
                throw new Error("a point is [x, y] in window coordinates")
            return { x: target[0], y: target[1] }
        }
        if (typeof target === "string") {
            var it = findItem(target)
            if (!it) throw new Error("no visible item named " + target)
            var c = it.mapToItem(null, it.width / 2, it.height / 2)
            return { x: c.x, y: c.y }
        }
        throw new Error("a target is [x, y] or an objectName")
    }

    // ---- keys

    readonly property var named: ({
        escape: [Qt.Key_Escape, "\u001b"], esc: [Qt.Key_Escape, "\u001b"],
        return: [Qt.Key_Return, "\r"], enter: [Qt.Key_Enter, "\r"],
        tab: [Qt.Key_Tab, "\t"], backtab: [Qt.Key_Backtab, ""], space: [Qt.Key_Space, " "],
        backspace: [Qt.Key_Backspace, "\b"], delete: [Qt.Key_Delete, "\u007f"],
        up: [Qt.Key_Up, ""], down: [Qt.Key_Down, ""], left: [Qt.Key_Left, ""], right: [Qt.Key_Right, ""],
        home: [Qt.Key_Home, ""], end: [Qt.Key_End, ""], pageup: [Qt.Key_PageUp, ""], pagedown: [Qt.Key_PageDown, ""],
        f1: [Qt.Key_F1, ""], f2: [Qt.Key_F2, ""], f3: [Qt.Key_F3, ""], f4: [Qt.Key_F4, ""], f5: [Qt.Key_F5, ""],
        f6: [Qt.Key_F6, ""], f7: [Qt.Key_F7, ""], f8: [Qt.Key_F8, ""], f9: [Qt.Key_F9, ""], f10: [Qt.Key_F10, ""],
        f11: [Qt.Key_F11, ""], f12: [Qt.Key_F12, ""]
    })
    readonly property var modNames: ({
        ctrl: Qt.ControlModifier, control: Qt.ControlModifier, shift: Qt.ShiftModifier,
        alt: Qt.AltModifier, meta: Qt.MetaModifier, super: Qt.MetaModifier
    })
    // An uppercase letter typed or pressed bare is a shifted one; in a combo ("Ctrl+K") the
    // letter's case means nothing, as in a Shortcut sequence, and Shift is spelled out.
    function charKey(ch, mods, caseShifts) {
        if (ch === "\n") return { key: Qt.Key_Return, mods: mods, text: "\r" }
        if (ch === " ") return { key: Qt.Key_Space, mods: mods, text: " " }
        var upper = ch.toUpperCase()
        if (caseShifts && ch !== ch.toLowerCase()) mods |= Qt.ShiftModifier
        var typed = (mods & (Qt.ControlModifier | Qt.AltModifier | Qt.MetaModifier)) ? ""
                  : ((mods & Qt.ShiftModifier) ? upper : ch)
        return { key: upper.charCodeAt(0), mods: mods, text: typed }
    }
    function parseKey(combo) {
        if (typeof combo !== "string" || combo === "") throw new Error("a key is a name like Escape or a combo like Ctrl+K")
        var parts = combo === "+" ? ["+"] : combo.split("+")
        if (combo.length > 1 && combo.slice(-2) === "++") { parts = combo.slice(0, -2).split("+"); parts.push("+") }
        var keyPart = parts.pop(), mods = 0
        parts.forEach(function(m) {
            var v = modNames[m.toLowerCase()]
            if (v === undefined) throw new Error("unknown modifier " + m + " in " + combo)
            mods |= v
        })
        var n = named[keyPart.toLowerCase()]
        if (n) {
            if (n[0] === Qt.Key_Tab && (mods & Qt.ShiftModifier)) return { key: Qt.Key_Backtab, mods: mods, text: "" }
            var text = (mods & (Qt.ControlModifier | Qt.AltModifier | Qt.MetaModifier)) ? "" : n[1]
            return { key: n[0], mods: mods, text: text }
        }
        if (keyPart.length === 1) return charKey(keyPart, mods, parts.length === 0)
        throw new Error("unknown key " + keyPart)
    }
    function keyActions(k, out) {
        out.push(function() { Shell.driveKey(hostWindow, true, k.key, k.mods, k.text); return 0 })
        out.push(function() { Shell.driveKey(hostWindow, false, k.key, k.mods, k.text); return 10 })
    }

    // ---- steps

    function button(step) {
        var b = step.button === undefined ? "left" : step.button
        var v = ({ left: Qt.LeftButton, right: Qt.RightButton, middle: Qt.MiddleButton })[b]
        if (v === undefined) throw new Error("button is left, right or middle, not " + b)
        return v
    }
    function moveTo(target) {
        return function() {
            driver.point = resolve(target)
            Shell.driveMouse(hostWindow, 0, point.x, point.y, Qt.NoButton, 0)
            return gap
        }
    }

    // Grabs the window to <out>/<name>.png, and carries on from the callback.
    function shotAction(name) {
        if (typeof name !== "string" || name === "" || name.indexOf("/") >= 0)
            throw new Error("shot takes a bare file name")
        var file = outDir + "/" + name + ".png"
        var n0 = at
        return function() {
            var ok = hostWindow.contentItem.grabToImage(function(result) {
                if (!result.saveToFile(file)) { fail("step " + n0 + ": cannot write " + file); return }
                log("DRIVE SHOT: " + file)
                driver.pump()
            })
            if (!ok) throw new Error("grabToImage refused")
            return -1
        }
    }

    // A page step: opens the page. With "shot" or "until" it then waits for the page to be
    // built (the frame shows this page and a new page item stands in it) and for the
    // optional condition, settles, and grabs, so one run can capture a set of pages.
    function pageActions(step, out) {
        if (typeof step.page !== "string" || step.page === "") throw new Error("page takes a page name")
        var props = step.props || {}, n0 = at, prev = null
        var limit = step.timeout === undefined ? 15000 : step.timeout
        if (typeof limit !== "number" || limit <= 0) throw new Error("timeout takes milliseconds")
        var waits = step.shot !== undefined || step.until !== undefined
        if (step.until !== undefined && (typeof step.until !== "string" || step.until === ""))
            throw new Error("until takes a JS condition")
        out.push(function() {
            if (!hostFrame.item || !hostFrame.item.go) throw new Error("no frame to navigate")
            prev = hostFrame.item.pageItem
            hostFrame.item.go(step.page, props, undefined)
            return waits ? 0 : settle * 4
        })
        if (!waits) return
        out.push(function() {
            var w = { what: "page " + step.page, src: step.until || "page built", timeout: limit, n: n0,
                      started: Date.now(), deadline: Date.now() + limit, last: undefined, threw: undefined,
                      check: function(w) {
                          var f = hostFrame.item
                          if (!f || !f.nav || f.nav.current.page !== step.page || !f.pageItem || f.pageItem === prev) {
                              w.threw = "page not built yet"; return false
                          }
                          if (step.until === undefined) return true
                          var r
                          try { r = evaluate(step.until, vars) } catch (e) { w.threw = e.message || String(e); return false }
                          w.last = r
                          return !!r
                      } }
            if (probe(w)) return 0
            driver.waiting = w; poller.start()
            return -1
        })
        out.push(function() { return step.settle === undefined ? 300 : step.settle })
        if (step.shot !== undefined) out.push(shotAction(step.shot))
    }

    function expand(step) {
        if (step === null || typeof step !== "object" || Array.isArray(step)) throw new Error("a step is an object")
        var out = [], b
        log("DRIVE STEP " + at + ": " + summary(step))
        if (step.wait !== undefined) {
            if (typeof step.wait !== "number" || step.wait < 0) throw new Error("wait takes milliseconds")
            out.push(function() { return Math.max(1, step.wait) })
        } else if (step.until !== undefined) {
            if (typeof step.until !== "string" || step.until === "") throw new Error("until takes a JS condition")
            var limit = step.timeout === undefined ? 5000 : step.timeout
            if (typeof limit !== "number" || limit <= 0) throw new Error("timeout takes milliseconds")
            var n0 = at
            out.push(function() {
                var w = { src: step.until, hasExpect: step.expect !== undefined, expect: step.expect,
                          as: typeof step.as === "string" ? step.as : undefined, timeout: limit, n: n0,
                          started: Date.now(), deadline: Date.now() + limit, last: undefined, threw: undefined }
                if (probe(w)) { log("DRIVE UNTIL: ready at once"); return 0 }
                driver.waiting = w; poller.start()
                return -1
            })
        } else if (step.click !== undefined) {
            b = button(step)
            out.push(moveTo(step.click))
            out.push(function() { Shell.driveMouse(hostWindow, 1, point.x, point.y, b, 0); return gap })
            if (step.double === true) {
                out.push(function() { Shell.driveMouse(hostWindow, 2, point.x, point.y, b, 0); return gap })
                out.push(function() { Shell.driveMouse(hostWindow, 1, point.x, point.y, b, 0); return gap })
            }
            out.push(function() { Shell.driveMouse(hostWindow, 3, point.x, point.y, b, 0); return settle })
        } else if (step.drag !== undefined) {
            var d = step.drag, n = step.steps === undefined ? 10 : step.steps
            if (!Array.isArray(d) || d.length !== 4 || d.some(function(v) { return typeof v !== "number" }))
                throw new Error("drag is [x1, y1, x2, y2]")
            if (typeof n !== "number" || n < 1) throw new Error("steps takes a count of at least 1")
            b = button(step)
            out.push(moveTo([d[0], d[1]]))
            out.push(function() { Shell.driveMouse(hostWindow, 1, point.x, point.y, b, 0); return gap })
            for (var k = 1; k <= n; k++) (function(f) {
                out.push(function() {
                    driver.point = { x: d[0] + (d[2] - d[0]) * f, y: d[1] + (d[3] - d[1]) * f }
                    Shell.driveMouse(hostWindow, 0, point.x, point.y, Qt.NoButton, 0)
                    return gap
                })
            })(k / n)
            if (step.release !== false)
                out.push(function() { Shell.driveMouse(hostWindow, 3, point.x, point.y, b, 0); return settle })
        } else if (step.release !== undefined) {
            b = button(step)
            out.push(moveTo(step.release))
            out.push(function() { Shell.driveMouse(hostWindow, 3, point.x, point.y, b, 0); return settle })
        } else if (step.hover !== undefined) {
            out.push(moveTo(step.hover))
            out.push(function() { return settle })
        } else if (step.wheel !== undefined) {
            var w = step.wheel, target = w, dy = step.dy, dx = step.dx || 0
            if (Array.isArray(w)) {
                if (w.length < 3 || typeof w[2] !== "number") throw new Error("wheel is [x, y, dy]")
                target = [w[0], w[1]]; dy = w[2]
            } else if (typeof dy !== "number") throw new Error("wheel on an objectName needs \"dy\"")
            out.push(moveTo(target))
            out.push(function() { Shell.driveWheel(hostWindow, point.x, point.y, dx, dy, 0); return settle })
        } else if (step.key !== undefined) {
            keyActions(parseKey(step.key), out)
            out.push(function() { return settle })
        } else if (step.type !== undefined) {
            if (typeof step.type !== "string") throw new Error("type takes a string")
            for (var i = 0; i < step.type.length; i++) keyActions(charKey(step.type[i], 0, true), out)
            out.push(function() { return settle })
        } else if (step.shot !== undefined) {
            if (typeof step.shot !== "string" || step.shot === "" || step.shot.indexOf("/") >= 0)
                throw new Error("shot takes a bare file name")
            out.push(shotAction(step.shot))
        } else if (step.log !== undefined) {
            // the step line above is the marker
        } else if (step.eval !== undefined) {
            out.push(function() {
                var r = evaluate(String(step.eval), vars)
                if (typeof step.as === "string") vars[step.as] = r
                var shown = show(r)
                log("DRIVE EVAL: " + shown)
                if (step.expect !== undefined && JSON.stringify(step.expect) !== shown)
                    throw new Error("expected " + JSON.stringify(step.expect) + ", got " + shown)
                return 0
            })
        } else if (step.page !== undefined) {
            pageActions(step, out)
        } else if (step.pages !== undefined) {
            if (!Array.isArray(step.pages) || step.pages.length === 0) throw new Error("pages is a list of page steps")
            step.pages.forEach(function(e) {
                if (e === null || typeof e !== "object" || Array.isArray(e)) throw new Error("a pages entry is an object")
                pageActions(e, out)
            })
        } else {
            throw new Error("unknown step " + summary(step))
        }
        return out
    }
}
