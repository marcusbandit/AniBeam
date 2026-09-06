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

    // The tokens every component below reaches through the context chain. A plain Window
    // has no font property, so the face and the size are set per Text from these tokens.
    // Until the engine's first push every token is absent, so the ground is the neutral one
    // until Theme.ready rather than Tokens' missing-token magenta.
    Tokens { id: theme }
    color: Theme.ready ? theme.bg : "#101216"

    // Window.color is the clear colour the compositor paints with; grabToImage renders only
    // painted items, so without this the offscreen capture below comes back transparent.
    Rectangle { anchors.fill: parent; color: window.color }

    property bool firstFrame: false
    property bool settled: false
    onAfterAnimating: if (!firstFrame) { firstFrame = true; settle.start() }
    onWidthChanged: if (firstFrame && !settled) settled = true
    onHeightChanged: if (firstFrame && !settled) settled = true
    Timer { id: settle; interval: 200; onTriggered: window.settled = true }

    Loader {
        id: frame
        anchors.fill: parent
        active: window.settled && Theme.ready && Door.ready
        sourceComponent: Shell.page === "tokens" ? tokensPage : frameComponent
        onLoaded: {
            if (Shell.page !== "tokens" && Shell.page !== "library" && item.nav) {
                var props = {}
                if (Shell.props !== "") { try { props = JSON.parse(Shell.props) } catch (e) { console.warn("--props is not valid JSON:", e.message); props = {} } }
                // A --shoot of the series page with no id in --props opens the first
                // series alphabetically, so the page has something real to draw.
                if (Shell.page === "series" && props.id === undefined) {
                    var r = Door.listSeries("All", "", "Alpha", "Asc", false)
                    if (!r.error && r.reply.series.length) props = { id: r.reply.series[0].id }
                }
                item.nav.replace(Shell.page, props, undefined)
            }
            window.maybeShoot()
        }
    }
    Component { id: frameComponent; Frame { hostWindow: window } }
    Component { id: tokensPage; TokensPage {} }
    title: frame.item && frame.item.windowTitle ? frame.item.windowTitle : "AniBeam"

    // --shoot <png>: one capture of the frame after settle, then quit. grabToImage renders
    // the scene into an image, so it works under QT_QPA_PLATFORM=offscreen. The colours are
    // the engine's first push, so the shot waits for Theme.ready and the frame's item as
    // well as for settle.
    function maybeShoot() {
        if (window.settled && Theme.ready && frame.item && Shell.shoot !== "")
            shootTimer.start()
    }
    onSettledChanged: window.maybeShoot()
    Connections { target: Theme; function onReadyChanged() { window.maybeShoot() } }
    Timer {
        id: shootTimer
        interval: 400
        onTriggered: window.contentItem.grabToImage(function(result) {
            result.saveToFile(Shell.shoot)
            Qt.quit()
        })
    }
}
