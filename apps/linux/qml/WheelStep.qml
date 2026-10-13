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
    onWheel: function(ev) {
        if (!following) target = flick.contentY
        // Only a touchpad's pixel delta is taken as pixels: Qt on Wayland fills pixelDelta
        // for a plain mouse wheel too (a dozen px a notch), which read as a crawl.
        var touchpad = ev.device && ev.device.type === PointerDevice.TouchPad && ev.pixelDelta.y !== 0
        var dy = touchpad ? -ev.pixelDelta.y : -ev.angleDelta.y / 120 * step
        target = bound(target + dy)
        following = true
        flick.cancelFlick()
        ev.accepted = true
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
