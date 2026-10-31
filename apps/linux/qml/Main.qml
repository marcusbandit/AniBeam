// The window. The frame and the first page are built as soon as the engine and the core
// are up, while the render thread sets up the GPU. Hyprland answers the first configure with
// 0x0 and sends the tile's size only after the window has mapped; that arrives as one
// resize and one relayout. `settled` still gates the --shoot and --drive captures.
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
    // The tokens the native poster cards draw with (CardFace, #289), handed over at start and
    // again whenever the theme moves
    QtObject {
        id: cardTokens
        readonly property var tokens: ({
            fontFamily: theme.fontFamily, typeSmall: theme.typeSmall, typeNormal: theme.typeNormal, typeLarge: theme.typeLarge,
            space: 4 * theme.densityFactor, radiusTile: theme.radiusTile, radiusSm: theme.radiusSm, scrimReach: theme.scrimReach,
            text: theme.text, textDim: theme.textDim, textFaint: theme.textFaint, accent: theme.accent, focusRing: theme.focusRing,
            onPoster: theme.onPoster, onPosterDim: theme.onPosterDim, accentOnPoster: theme.accentOnPoster, posterScrim: theme.posterScrim,
            downloadInk: theme.tone(theme.cubeState("t").fill, Qt.rgba(1, 1, 1, 1), 0.3),
            cubes: theme.cubeStates
        })
        onTokensChanged: CardStyle.set(tokens)
        Component.onCompleted: CardStyle.set(tokens)
    }
    color: Theme.ready ? theme.bg : "#101216"

    // The poster pool: every poster decoded once and held, so a page's first
    // paint finds its pictures in the engine's cache instead of on the disk.
    // Filled the way the pages show them — airing soonest, then recently
    // viewed, then the shelf — and refilled (debounced) when the library says
    // a series changed, only ever replacing the pool when the path set moved.
    // Developer options can switch the pool off, and switching images off empties it too.
    PosterPool {
        id: posterPool
        capacity: frame.item && frame.item.devOn && (!frame.item.devKept.posterPool || !frame.item.devKept.images) ? 0 : 60
    }
    // One read for both lists: listLibrary carries the airing rail with the shelf, and
    // the Door keeps the answer until the library changes, so a page reading the same
    // thing in the same beat pays nothing.
    function fillPool() {
        var r = Door.listLibrary("All", "", "LastViewed", "Desc", false)
        if (r.error) return
        var out = [], seen = {}
        function push(c) { if (c.poster && !seen[c.poster]) { seen[c.poster] = true; out.push(c.poster) } }
        r.reply.airing.forEach(function(c) { if (c.next_airing) push(c) })
        r.reply.series.forEach(push)
        var paths = out.slice(0, posterPool.capacity)
        if (paths.length !== posterPool.paths.length || paths.join("\u0001") !== posterPool.paths.join("\u0001"))
            posterPool.paths = paths
    }
    Timer { id: poolDebounce; interval: 2000; onTriggered: window.fillPool() }
    // Measuring sends posters through the image://dev provider, a different url from the
    // file the pool decoded, so the pool would miss every page; it starts over instead.
    Connections {
        target: frame.item && frame.item.devOn !== undefined ? frame.item : null
        function onDevOnChanged() { posterPool.paths = []; poolDebounce.restart() }
    }
    Connections { target: Door; function onSeriesChanged(cards) { poolDebounce.restart() } }
    // The first fill waits a beat after the library is ready, so the first page's own
    // posters reach the single pixmap reader thread ahead of the pool's.
    Timer { id: poolStart; interval: 1500; onTriggered: window.fillPool() }
    Connections { target: Door; function onReadyChanged() { if (Door.ready) poolStart.start() } }

    // Touching the Player singleton here is what constructs it, and constructing it is what
    // registers the Qt thread an MPRIS command is queued on. Without this the first media
    // key before a file plays would have nowhere to go.
    readonly property real bootVolume: Player.volume

    // A second launch, or an MPRIS Raise, asks for this window; Quit closes it. The token
    // is the launcher's, and it goes into the environment before the activation request.
    Connections {
        target: Shell
        function onActivateRequested(token) { Shell.raiseWindow(window, token) }
        function onQuitRequested() { Qt.quit() }
    }

    // Window.color is the clear colour the compositor paints with; grabToImage renders only
    // painted items, so without this the offscreen capture below comes back transparent.
    Rectangle { anchors.fill: parent; color: window.color }

    property bool firstFrame: false
    property bool settled: false
    onAfterAnimating: if (!firstFrame) { firstFrame = true; Dev.mark("first frame (empty window)"); settle.start() }
    onWidthChanged: if (firstFrame && !settled) settled = true
    onHeightChanged: if (firstFrame && !settled) settled = true
    // The compositor's size arrives as a resize right after the window maps, and the line
    // above settles on it at once; this is only the fallback for a window that is never
    // resized (a tiling WM that keeps the hint, a floating window, offscreen). It was 200 ms,
    // which was most of a launch (#289).
    Timer { id: settle; interval: 30; onTriggered: window.settled = true }

    // --page takes an optional ":<action>" suffix for a shoot capture that needs the frame
    // in a state a bare page name cannot reach, such as the activity drawer open over it;
    // pageName is the page nav opens, pageAction the part after the colon.
    readonly property string pageName: Shell.page.indexOf(":") >= 0 ? Shell.page.slice(0, Shell.page.indexOf(":")) : Shell.page
    readonly property string pageAction: Shell.page.indexOf(":") >= 0 ? Shell.page.slice(Shell.page.indexOf(":") + 1) : ""

    Loader {
        id: frame
        anchors.fill: parent
        // Built as soon as the engine and the core are up, not after the first empty frame:
        // the render thread sets up the GPU meanwhile, and a compositor's late size is one
        // relayout, where waiting for it was most of a launch (#289)
        active: Theme.ready && Door.ready
        sourceComponent: window.pageName === "tokens" ? tokensPage : frameComponent
        onLoaded: {
            Dev.mark("frame built")
            // Home is the default page, so its props ride only when a launch actually
            // handed some over; every other page always replaces.
            var homeWithProps = window.pageName === "home" && Shell.props !== ""
            if (window.pageName !== "tokens" && (homeWithProps || (window.pageName !== "home" && item.nav))) {
                var props = {}
                if (Shell.props !== "") { try { props = JSON.parse(Shell.props) } catch (e) { console.warn("--props is not valid JSON:", e.message); props = {} } }
                // A --shoot of the series page with no id in --props opens the first
                // series alphabetically, so the page has something real to draw.
                if (window.pageName === "series" && props.id === undefined) {
                    var r = Door.listSeries("All", "", "Alpha", "Asc", false)
                    if (!r.error && r.reply.series.length) props = { id: r.reply.series[0].id }
                }
                item.nav.replace(window.pageName, props, undefined)
            }
            // The suffix action runs once the page is up and before the grab, so the
            // drawer's rise has the whole shoot delay to settle at its open height.
            if (window.pageAction === "drawer" && item.toggleDrawer) item.toggleDrawer()
            if (window.pageAction === "connect" && item.openConnect) item.openConnect("Anilist")
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
    // --drive <script.json> starts its steps at the same moment instead of grabbing; see
    // Driver.qml. driveOut is set only on a drive run, so it is the mode flag.
    function maybeShoot() {
        if (window.settled && Theme.ready && frame.item && (Shell.shoot !== "" || Shell.driveOut !== ""))
            shootTimer.start()
    }
    onSettledChanged: { if (window.settled) Dev.mark("window settled"); window.maybeShoot() }
    Connections { target: Theme; function onReadyChanged() { window.maybeShoot() } }
    Timer {
        id: shootTimer
        interval: 400
        onTriggered: {
            if (Shell.driveOut !== "") { if (driver.item) driver.item.start(); return }
            window.contentItem.grabToImage(function(result) {
                result.saveToFile(Shell.shoot)
                Qt.quit()
            })
        }
    }
    Loader {
        id: driver
        active: Shell.driveOut !== ""
        sourceComponent: Driver { hostWindow: window; hostFrame: frame; outDir: Shell.driveOut; evaluate: window.driveEval }
    }
    // A drive script's {"eval": ...}: direct eval here, so window, frame and theme resolve,
    // and `vars` holds what earlier evals stored with "as".
    function driveEval(src, vars) { return eval(src) }
}
