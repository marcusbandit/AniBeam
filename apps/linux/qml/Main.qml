// The window. The first frame is the ground alone: Hyprland answers the first configure
// with 0x0 and sends the tile's size only after the window has mapped, so everything else
// is built once the window is settled and laid out once, at the compositor's size.
import QtQuick
import QtQuick.Window
import com.marcusrosado.AniBeam

Window {
    id: window
    width: Shell.shootWidth > 0 ? Shell.shootWidth : 1280   // a hint; the compositor sizes the window
    height: Shell.shootHeight > 0 ? Shell.shootHeight : 800
    visible: true
    title: "AniBeam"
    color: "#101216"                                        // Task 5 binds this to theme.bg

    // Window.color is the clear colour the compositor paints with; grabToImage renders only
    // painted items, so without this the offscreen capture below comes back transparent.
    Rectangle { anchors.fill: parent; color: window.color }

    property bool firstFrame: false
    property bool settled: false
    onAfterAnimating: if (!firstFrame) { firstFrame = true; settle.start() }
    onWidthChanged: if (firstFrame && !settled) settled = true
    onHeightChanged: if (firstFrame && !settled) settled = true
    Timer { id: settle; interval: 200; onTriggered: window.settled = true }

    // Task 7 replaces this with Frame { anchors.fill: parent; visible: window.settled }
    Text {
        visible: window.settled
        anchors.centerIn: parent
        text: "AniBeam " + Shell.version
        color: "#e4e7ee"
    }

    // --shoot <png>: one capture of the frame after settle, then quit. grabToImage renders
    // the scene into an image, so it works under QT_QPA_PLATFORM=offscreen.
    onSettledChanged: if (settled && Shell.shoot !== "") shootTimer.start()
    Timer {
        id: shootTimer
        interval: 400
        onTriggered: window.contentItem.grabToImage(function(result) {
            result.saveToFile(Shell.shoot)
            Qt.quit()
        })
    }
}
