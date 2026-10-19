// A Flickable's wheel, scrolled the way a browser does it. Qt's own wheel moves 72 px a
// notch as a short glide, and the next notch cancels whatever that glide had left, so a
// fast spin barely went further than one notch. Here every notch adds 100 px to a target
// (per 120 of angle, so the compositor's scroll factor still applies) and the content
// glides there. A touchpad's pixels go on 1:1. A drag takes over the moment it starts.
import QtQuick
import com.marcusrosado.AniBeam

WheelHandler {
    id: root
    required property Flickable flick
    readonly property real notch: 100
    function bound(y) { return Math.max(0, Math.min(y, Math.max(0, flick.contentHeight - flick.height))) }
    // The keys scroll through here too, so a key and a notch add to the same target
    function scrollTo(y) {
        flick.cancelFlick()
        glide.to = bound(y)
        glide.restart()
    }
    acceptedDevices: PointerDevice.Mouse | PointerDevice.TouchPad
    acceptedPointerTypes: PointerDevice.AllPointerTypes
    onWheel: function(ev) {
        // Qt's Wayland plugin calls a mouse wheel a touchpad, so the phase is what tells them
        // apart: a finger on a touchpad comes as a begin, update, end run, a notch with none
        if (ev.phase === Qt.NoScrollPhase && ev.angleDelta.y !== 0) {
            scrollTo((glide.running ? glide.to : flick.contentY) - ev.angleDelta.y / 120 * notch)
        } else {
            glide.stop()
            flick.cancelFlick()
            flick.contentY = bound(flick.contentY - ev.pixelDelta.y)
        }
    }
    // A handler holds no children, so the glide and the drag hook are properties
    property NumberAnimation glide: NumberAnimation {
        target: root.flick
        property: "contentY"
        duration: theme.reducedMotion ? 0 : theme.motionNormal
        easing.type: Easing.OutCubic
    }
    property Connections dragHook: Connections {
        target: root.flick
        function onMovementStarted() { root.glide.stop() }
    }
}
