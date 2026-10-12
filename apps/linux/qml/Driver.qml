// --drive <script.json>: runs a JSON array of steps against the window, offscreen, once the
// frame is up (Main.qml starts it at the moment a --shoot would grab). Input goes through
// Shell.drive*, which enter at QtGui's window-system layer the way QtTest does, so a click
// here is a click to MouseArea, the pointer handlers, Popup's press-outside and Shortcut.
// Every step prints "DRIVE STEP <n>: <summary>"; any failure prints "DRIVE FAIL: <reason>"
// and exits 1; the end of the script exits 0. Steps (one key each, besides the options):
//   {"wait": ms}
//   {"click": [x, y] | "objectName", "button": "left"|"right"|"middle", "double": true}
//   {"hover": [x, y] | "objectName"}
//   {"wheel": [x, y, dy] | "objectName", "dy": n, "dx": n}     angle delta; 120 is a notch
//   {"key": "Escape" | "Ctrl+K" | "a" | ...}
//   {"type": "text"}
//   {"shot": "name"}                                           <out>/<name>.png
//   {"log": "text"}
//   {"eval": "js", "expect": <json>, "as": "k"}                in Main.qml's scope; the
//                                                              result is vars.k to later evals
//   {"page": "name", "props": {...}}
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

    readonly property int gap: 30  // between the move, the press and the release of a click
    readonly property int settle: 50

    function log(s) { Shell.driveLog(s) }
    function fail(reason) {
        if (finished) return
        finished = true; timer.stop(); queue = []
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

    function expand(step) {
        if (step === null || typeof step !== "object" || Array.isArray(step)) throw new Error("a step is an object")
        var out = [], b
        log("DRIVE STEP " + at + ": " + summary(step))
        if (step.wait !== undefined) {
            if (typeof step.wait !== "number" || step.wait < 0) throw new Error("wait takes milliseconds")
            out.push(function() { return Math.max(1, step.wait) })
        } else if (step.click !== undefined) {
            b = button(step)
            out.push(moveTo(step.click))
            out.push(function() { Shell.driveMouse(hostWindow, 1, point.x, point.y, b, 0); return gap })
            if (step.double === true) {
                out.push(function() { Shell.driveMouse(hostWindow, 2, point.x, point.y, b, 0); return gap })
                out.push(function() { Shell.driveMouse(hostWindow, 1, point.x, point.y, b, 0); return gap })
            }
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
            var file = outDir + "/" + step.shot + ".png"
            out.push(function() {
                var ok = hostWindow.contentItem.grabToImage(function(result) {
                    if (!result.saveToFile(file)) { fail("step " + at + ": cannot write " + file); return }
                    log("DRIVE SHOT: " + file)
                    driver.pump()
                })
                if (!ok) throw new Error("grabToImage refused")
                return -1
            })
        } else if (step.log !== undefined) {
            // the step line above is the marker
        } else if (step.eval !== undefined) {
            out.push(function() {
                var r = evaluate(String(step.eval), vars)
                if (typeof step.as === "string") vars[step.as] = r
                // JSON where it can be (strings quoted, arrays, records); a QObject or a
                // function prints as itself
                var shown
                try { shown = r === undefined ? "undefined" : JSON.stringify(r) } catch (e) { shown = undefined }
                if (shown === undefined) shown = String(r)
                log("DRIVE EVAL: " + shown)
                if (step.expect !== undefined && JSON.stringify(step.expect) !== shown)
                    throw new Error("expected " + JSON.stringify(step.expect) + ", got " + shown)
                return 0
            })
        } else if (step.page !== undefined) {
            out.push(function() {
                if (!hostFrame.item || !hostFrame.item.go) throw new Error("no frame to navigate")
                hostFrame.item.go(String(step.page), step.props || {}, undefined)
                return settle * 4
            })
        } else {
            throw new Error("unknown step " + summary(step))
        }
        return out
    }
}
