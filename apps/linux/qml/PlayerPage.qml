// Spec 4.4: the player. The one page that takes the whole window. libmpv draws through
// VideoItem; the shell draws every overlay and handles every key. The core owns the
// rules; the shell sends ticks and shows what comes back.
import QtQuick
import QtQuick.Window
import com.marcusrosado.AniBeam

FocusScope {
    id: page
    property var props: ({})
    readonly property bool fullWindow: true
    property var session: null
    property string title: session ? session.series_title : "Player"
    property real scrollY: 0
    readonly property alias video: video

    // Observed mpv state
    property real timePos: 0
    property real duration: 0
    property bool paused: false
    property bool seeking: false
    property bool ended: false
    property bool loaded: false
    property string hwdec: ""
    property int drops: 0

    // Chrome
    property bool chromeVisible: true
    property int openMenus: 0            // pickers and the help list hold the chrome
    // A menu holds the chrome: the timer runs again rather than falling through, so the
    // chrome hides 2.5 s after the menu closes instead of staying up until the next move.
    Timer { id: hideTimer; interval: 2500; onTriggered: { if (page.openMenus > 0) restart(); else page.chromeVisible = false } }
    function showChrome() { chromeVisible = true; hideTimer.restart() }

    // ---- Session
    Component.onCompleted: {
        var r = Door.openPlayback(props.file)
        if (r.error) { frame.toast(r.error.message); Qt.callLater(frame.nav.back); return }
        session = r.reply.session
        if (video.isReady) start()
        showChrome()
        forceActiveFocus()
    }
    Component.onDestruction: close("Stopped")
    property bool closed: false
    function close(reason) {
        if (!session || closed) return
        closed = true
        tickTimer.stop()
        Door.closePlayback(session.session, timePos, reason)
    }
    function leave() { close("Stopped"); frame.nav.back() }
    function openNeighbour(file) { if (!file) return; close("Switched"); frame.nav.replace("player", { file: file }, page.title) }
    // Named escapePressed, not escape: the QML compiler reserves "escape" as a member name
    // on any object, and the frame calls this one by that name.
    function escapePressed() { leave(); return true }

    // The layers, the owned options, then the file, seeking to the resume point before
    // the first frame through the start option.
    function start() {
        var layers = Player.configLayers
        for (var i = 0; i < layers.length; i++) video.include(layers[i])
        var owned = Player.ownedOptions
        for (var j = 0; j < owned.length; j++) video.setProperty(owned[j][0], owned[j][1])
        applyDefaults()                                          // Task 11 fills this
        video.setProperty("volume", Player.volume)
        video.setProperty("mute", Player.mute)
        video.setProperty("start", session.resume_from ? String(session.resume_from) : "none")
        video.command(["loadfile", session.path])
    }
    function applyDefaults() {}

    VideoItem {
        id: video
        anchors.fill: parent
        property bool isReady: false
        onReady: {
            isReady = true
            observe("time-pos", VideoItem.Double); observe("duration", VideoItem.Double); observe("pause", VideoItem.Flag)
            observe("eof-reached", VideoItem.Flag); observe("seeking", VideoItem.Flag); observe("volume", VideoItem.Double)
            observe("mute", VideoItem.Flag); observe("hwdec-current", VideoItem.String); observe("frame-drop-count", VideoItem.Int64)
            observe("track-list", VideoItem.Node); observe("chapter-list", VideoItem.Node); observe("aid", VideoItem.String)
            observe("sid", VideoItem.String); observe("sub-delay", VideoItem.Double); observe("estimated-frame-number", VideoItem.Int64)
            if (page.session) page.start()
        }
        onLoaded: {
            page.loaded = true
            setProperty("start", "none")
            var chapters = getProperty("chapter-list") || []
            var list = []
            for (var i = 0; i < chapters.length; i++) list.push({ title: String(chapters[i].title || ""), start: Number(chapters[i].time || 0) })
            page.duration = Number(getProperty("duration") || 0)
            if (page.session && page.duration > 0) {
                var rc = Door.reportChapters(page.session.session, list, page.duration)
                if (rc.error) console.warn("anibeam: chapters:", rc.error.message)
            }
            page.onFileLoaded()                                  // Task 11 picks the tracks here
            tickTimer.start()
        }
        onChanged: function(name, value) {
            if (name === "time-pos") { if (value !== null && value !== undefined) page.timePos = value }
            else if (name === "duration") { if (value) page.duration = value }
            else if (name === "pause") { var was = page.paused; page.paused = !!value; if (was !== page.paused) page.tick() }
            else if (name === "seeking") { var wasSeeking = page.seeking; page.seeking = !!value; if (wasSeeking && !page.seeking) page.tick() }
            else if (name === "eof-reached") { if (value && !page.ended) { page.ended = true; page.onEnded() } }
            else if (name === "hwdec-current") page.hwdec = value ? String(value) : ""
            else if (name === "frame-drop-count") page.drops = Number(value || 0)
            else page.onObserved(name, value)                   // Tasks 11 and 12 read the rest
        }
        MouseArea { anchors.fill: parent; onClicked: page.togglePause(); hoverEnabled: true; onPositionChanged: page.showChrome(); cursorShape: page.chromeVisible ? Qt.ArrowCursor : Qt.BlankCursor }
    }
    function onFileLoaded() {}
    function onObserved(name, value) {}
    function onEnded() { close("Ended") }                        // Task 12 adds the replay and the pill

    // ---- Ticks: once a second while playing, once on pause, once after a seek, once on
    // close. The timer never asks mpv for anything: it sends the last observed time-pos,
    // which arrives on every frame, so this is the sampler, not a poll.
    Timer { id: tickTimer; interval: 1000; repeat: true; running: false; onTriggered: if (!page.paused) page.tick() }
    function tick() { if (session && !closed) Door.tick(session.session, timePos, paused) }

    // ---- Transport
    function togglePause() { video.setProperty("pause", !paused); showChrome() }
    function seekTo(secs) { var t = Math.max(0, Math.min(duration > 0 ? duration : secs, secs)); video.command(["seek", String(t), "absolute"]); showChrome() }
    function setVolume(v) { v = Math.max(0, Math.min(100, v)); video.setProperty("volume", v); if (v > 0 && Player.mute) setMute(false); Player.setVolume(v); showChrome() }
    function setMute(m) { video.setProperty("mute", m); Player.setMute(m); showChrome() }
    function toggleFullscreen() { frame.hostWindow.visibility = frame.hostWindow.visibility === Window.FullScreen ? Window.Windowed : Window.FullScreen }

    // ---- Header
    Rectangle {
        id: header
        anchors.left: parent.left; anchors.right: parent.right; anchors.top: parent.top
        // The bar is as tall as its three lines and one padding step either side, so a
        // larger system font or a denser scale grows it instead of clipping inside it.
        height: headerRow.implicitHeight + theme.space(4) * 2
        color: theme.scrim
        opacity: page.chromeVisible ? 1 : 0
        // Zero opacity still takes clicks, so the hidden chrome has to leave the scene.
        visible: opacity > 0
        Behavior on opacity { NumberAnimation { duration: theme.motionNormal } }
        Row {
            id: headerRow
            anchors.left: parent.left; anchors.leftMargin: theme.space(4); anchors.verticalCenter: parent.verticalCenter
            spacing: theme.space(4)
            PlayerButton { glyph: "arrow-left"; tip: "Back"; onClicked: page.leave() }
            Column {
                id: headerText
                anchors.verticalCenter: parent.verticalCenter
                width: header.width - parent.x - x - theme.space(4)
                Text { width: parent.width; elide: Text.ElideRight; text: page.session ? page.session.series_title : ""; color: theme.textDim; font.family: theme.fontSans; font.pointSize: theme.typeSmall }
                Text { width: parent.width; elide: Text.ElideRight; text: page.session ? (page.session.episode_title || page.session.path.split("/").pop()) : ""; color: theme.text; font.family: theme.fontSans; font.pointSize: theme.typeNormal; font.weight: Font.DemiBold }
                Text { width: parent.width; elide: Text.ElideRight; text: page.session ? page.session.code : ""; color: theme.textFaint; font.family: theme.fontMono; font.pointSize: theme.typeSmall }
            }
        }
    }

    // ---- Controls island
    Corner {
        id: controls
        anchors.horizontalCenter: parent.horizontalCenter
        anchors.bottom: parent.bottom; anchors.bottomMargin: theme.space(6)
        width: Math.min(parent.width - theme.space(12), theme.space(220))
        height: bottomRow.height + theme.space(6)                // Task 11 adds the seek row above
        radius: theme.radiusLg; smoothing: theme.cornerSmoothing
        color: theme.scrim; borderColor: theme.line; borderWidth: 1
        opacity: page.chromeVisible ? 1 : 0
        visible: opacity > 0
        Behavior on opacity { NumberAnimation { duration: theme.motionNormal } }
        MouseArea { anchors.fill: parent; hoverEnabled: true; onPositionChanged: page.showChrome() }
        Item { id: seekSlot; anchors.left: parent.left; anchors.right: parent.right; anchors.top: parent.top; anchors.margins: theme.space(3); height: 0 }
        Row {
            id: bottomRow
            anchors.left: parent.left; anchors.right: parent.right; anchors.bottom: parent.bottom; anchors.margins: theme.space(3)
            // No height: a Row is as tall as its tallest child, so the island follows the
            // buttons and the readout rather than a number that has to be kept in step.
            spacing: theme.space(1)
            PlayerButton { glyph: "skip-back"; interactive: !!(page.session && page.session.prev && !page.session.is_extra); tip: interactive ? "Previous episode" : "No previous episode"; onClicked: page.openNeighbour(page.session.prev) }
            PlayerButton { glyph: page.paused ? "play" : "pause"; tip: page.paused ? "Play" : "Pause"; onClicked: page.togglePause() }
            PlayerButton { glyph: "skip-forward"; interactive: !!(page.session && page.session.next && !page.session.is_extra); tip: interactive ? "Next episode" : "No next episode"; onClicked: page.openNeighbour(page.session.next) }
            Text { anchors.verticalCenter: parent.verticalCenter; text: Fmt.clock(page.timePos) + " / " + Fmt.clock(page.duration); color: theme.text; font.family: theme.fontMono; font.pointSize: theme.typeSmall; leftPadding: theme.space(2); rightPadding: theme.space(2) }
            PlayerButton { glyph: Player.mute || Player.volume === 0 ? "volume-x" : "volume-2"; tip: Player.mute ? "Unmute" : "Mute"; onClicked: page.setMute(!Player.mute) }
            SliderRow { anchors.verticalCenter: parent.verticalCenter; from: 0; to: 100; value: Player.mute ? 0 : Player.volume; stepSize: 1; trackWidth: theme.space(24); onMoved: function(v) { page.setVolume(v) } }
            // The slot takes its height from what it holds, not from the row: the row now
            // takes its own height from its children, and parent.height here would be a loop.
            Item { id: rightSlot; width: parent.width - x - theme.space(1); height: rightGroup.height   // Task 11 and 12 add the pickers, mark, help
                anchors.verticalCenter: parent.verticalCenter
                Row { id: rightGroup; anchors.right: parent.right; spacing: theme.space(1)
                    PlayerButton { glyph: frame.hostWindow.visibility === Window.FullScreen ? "minimize" : "maximize"; tip: "Fullscreen"; onClicked: page.toggleFullscreen() } } }
        }
    }

    // ---- Keys (the base set; Task 12 completes the map)
    // The keys a held press repeats: the seeks and the volume ramp, as every player does.
    // Task 12 adds the two frame step keys, which repeat as well.
    readonly property var repeatKeys: [Qt.Key_Left, Qt.Key_Right, Qt.Key_Up, Qt.Key_Down]
    Keys.onPressed: function(e) {
        // Escape swallows its own repeat rather than falling through: unaccepted, the frame
        // would take the second press and leave the player on a key that was held, not hit.
        if (e.isAutoRepeat && e.key === Qt.Key_Escape) { e.accepted = true; return }
        if (e.isAutoRepeat && page.repeatKeys.indexOf(e.key) < 0) { e.accepted = false; return }
        // Ctrl, Alt and Meta belong to the frame's shortcuts, so every branch below is the
        // plain key. Shift passes through: Task 12's z and Z differ by it. The one
        // combination the player claims, Ctrl+Right, is Task 12's and goes above this line.
        if (e.modifiers & (Qt.ControlModifier | Qt.AltModifier | Qt.MetaModifier)) { e.accepted = false; return }
        e.accepted = true
        if (e.key === Qt.Key_Space || e.key === Qt.Key_K) page.togglePause()
        else if (e.key === Qt.Key_Left) page.seekTo(page.timePos - 5)
        else if (e.key === Qt.Key_Right) page.seekTo(page.timePos + 5)
        else if (e.key === Qt.Key_M) page.setMute(!Player.mute)
        else if (e.key === Qt.Key_F) page.toggleFullscreen()
        else if (e.key === Qt.Key_Up) page.setVolume(Player.volume + 5)
        else if (e.key === Qt.Key_Down) page.setVolume(Player.volume - 5)
        else if (e.key === Qt.Key_Escape) { if (frame.hostWindow.visibility === Window.FullScreen) frame.hostWindow.visibility = Window.Windowed; else page.leave() }
        else e.accepted = false
    }
}
