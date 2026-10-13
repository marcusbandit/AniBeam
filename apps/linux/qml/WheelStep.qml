// The shell's mouse-wheel step for any Flickable. Every notch adds a fixed step to a target
// and the content follows it with exponential smoothing, so a spin of five notches goes
// five steps, not one: the Flickable's own flick() replaced the velocity on each notch, and
// a fast spin barely moved further than a single click. A touchpad hands pixel deltas,
// which add straight to the same target. The target is clamped to the content's bounds
// and a drag takes over the moment it starts.
import QtQuick
import com.marcusrosado.AniBeam

WheelHandler {
    id: root
    required property Flickable flick
    // One notch of a mouse wheel, in px: three of the usual desktop steps (Chromium and
    // GTK scroll about 100 px a notch), so one tap here is three taps elsewhere
    property real step: theme.space(75)
    property real target: 0
    property bool following: false
    function bound(y) { return Math.max(0, Math.min(y, Math.max(0, flick.contentHeight - flick.height))) }
    // The first wheel events of a run go to the log with everything the platform said about
    // them, so a report of a crawling wheel can be read against the real device
    property int logged: 0
    acceptedDevices: PointerDevice.AllDevices
    onWheel: function(ev) {
        try {
            if (!following) target = flick.contentY
            // A mouse notch is the angle delta (120 a notch, in fragments on a high-resolution
            // wheel). Qt on Wayland fills pixelDelta for a plain mouse wheel too, a dozen px a
            // notch, which read as a crawl, so pixels count only when no angle came at all (a
            // touchpad, or a wheel the platform reports in pixels: libinput's 15 a notch).
            var angle = ev.angleDelta.y, pixel = ev.pixelDelta.y
            var dy = angle !== 0 ? -angle / 120 * step : -pixel / 15 * step
            if (logged < 40) {
                logged++
                var dev = "none"
                try { dev = ev.device ? (ev.device.name + " type=" + ev.device.type) : "none" } catch (e) { dev = "unreadable" }
                console.warn("wheel: device=" + dev + " angle=" + angle + " pixel=" + pixel + " phase=" + ev.phase + " -> dy=" + dy + " step=" + step)
            }
            target = bound(target + dy)
            following = true
            flick.cancelFlick()
            ev.accepted = true
        } catch (e) {
            console.warn("wheel: handler failed: " + e)
        }
    }
    // A handler holds no children, so the follower is a property
    property FrameAnimation follower: FrameAnimation {
        running: root.following
        onTriggered: {
            if (root.flick.dragging) { root.following = false; return }
            var y = root.bound(root.target)
            var next = theme.follow(root.flick.contentY, y, frameTime)
            if (Math.abs(next - y) < 0.5) { root.flick.contentY = y; root.following = false }
            else root.flick.contentY = next
        }
    }
}
