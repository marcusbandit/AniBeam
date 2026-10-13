// The shell's mouse-wheel step for any Flickable. A notch starts the Flickable's own
// flick at a velocity that glides roughly three reading lines before its deceleration
// settles — Chromium's step, carried by the Flickable's native curve, so it is smooth,
// clamped at the bounds, and interrupted the moment a drag takes over. Mice and pens
// only: a touchpad keeps the Flickable's pixel-precise handling and momentum.
import QtQuick
import com.marcusrosado.AniBeam

WheelHandler {
    id: root
    required property Flickable flick
    // How far a notch travels, relative to the base step of ~3 reading lines. Glide distance
    // goes with velocity squared, so the velocity (and the cap a fast spin runs into) scales
    // by its square root.
    property real multiplier: 9
    // velocity for one base notch: ~3 lines of glide under the default 1500 px/s^2 deceleration
    readonly property real notchVelocity: 600 * Math.sqrt(multiplier)
    Component.onCompleted: flick.maximumFlickVelocity = 2500 * Math.sqrt(multiplier)
    onWheel: function(ev) {
        var notches = ev.angleDelta.y / 120
        flick.flick(0, notches * notchVelocity)
        ev.accepted = true
    }
}
