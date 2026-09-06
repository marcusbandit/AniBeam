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
    property var trackList: []
    property int aid: -1
    property int sid: -1
    // The subtitle track C turns back on, so switching off and on again lands where it was
    // rather than on the file's first track.
    property int lastSid: -1
    property real subDelay: 0
    // Task 12 wires SkipWindowsReady onto the same property; until then the session's own
    // windows are the ones the chapters gave.
    property var windows: session ? session.skip_windows : []

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
    // Spec 4.4's subtitle table as mpv options. These are set before the file, so they are
    // in force for its first frame.
    function applyDefaults() {
        var opts = Player.subtitleOptions(session.subtitle_defaults)
        for (var i = 0; i < opts.length; i++) video.setProperty(opts[i][0], opts[i][1])
    }
    // Every sidecar the core found beside the file. `sub-auto=no` is one of the owned
    // options, so mpv adds none of its own and the list the core reports is the list.
    // These go on after the file is loaded, not with the options before it: sub-add acts
    // on the file mpv is playing, and before loadfile there is none, so mpv answers
    // MPV_ERROR_COMMAND and the sidecar never reaches the list the pick reads.
    function addSidecars() {
        for (var j = 0; j < session.sidecars.length; j++) {
            var s = session.sidecars[j]
            video.commandBlocking(["sub-add", s.path, "auto", s.title || "", s.language || ""])
        }
    }
    // commandBlocking's own answer says nothing usable: mpv returns an empty node on
    // success, which reaches JS as undefined, and MpvQt wraps a failure in an ErrorReturn
    // struct, which reaches JS as NaN, so no comparison against it can be true. The list
    // mpv ended up with is the answer, so the check is made against that instead.
    function checkSidecars(list) {
        for (var j = 0; j < session.sidecars.length; j++) {
            var p = session.sidecars[j].path
            var on = list.some(function(t) { return t.type === "sub" && t.external && t["external-filename"] === p })
            if (!on) console.warn("anibeam: sub-add did not load", p)
        }
    }
    // A settings change reaches a file already playing: the styling options are re-set on
    // the running core. The language orders only decide the next pick, so nothing re-picks.
    Connections {
        target: Door
        function onSettingsChanged() {
            // The door writes its own settings property before it emits, so the new value
            // is already here and a second GetSettings would only ask for what we hold.
            var d = page.session ? Door.settings.subtitle_defaults : null
            if (!d) return
            page.session.subtitle_defaults = d
            var opts = Player.subtitleOptions(d)
            for (var i = 0; i < opts.length; i++) video.setProperty(opts[i][0], opts[i][1])
        }
    }

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
    // The one place the track list is asked for rather than observed: the pick has to be
    // made before the first frame, and the observation for `track-list` may not have
    // arrived yet. Everything after this comes from `changed`.
    function onFileLoaded() {
        addSidecars()
        trackList = asArray(video.getProperty("track-list"))
        checkSidecars(trackList)
        var p = Player.pickTracks(trackList, session.track_choice, session.subtitle_defaults)
        video.setProperty("aid", p.aid >= 0 ? String(p.aid) : "no")
        video.setProperty("sid", p.sid >= 0 ? String(p.sid) : "no")
        if (p.sid >= 0) lastSid = p.sid
    }
    function onObserved(name, value) {
        if (name === "track-list") trackList = asArray(value)
        else if (name === "aid") aid = value === "no" || value === null ? -1 : Number(value)
        else if (name === "sid") { sid = value === "no" || value === null ? -1 : Number(value); if (sid >= 0) lastSid = sid }
        else if (name === "sub-delay") { subDelay = Number(value || 0); hud.update(delayLine(subDelay), "subDelay") }
        else if (name === "chapter-list") {}
        else onObservedMore(name, value)                      // Task 12
    }
    function onObservedMore(name, value) {}
    function onEnded() { close("Ended") }                        // Task 12 adds the replay and the pill

    // ---- Tracks
    // mpv hands the list back as a QVariantList, which QML wraps in a sequence object:
    // Array.isArray says no and every element read converts a QVariant afresh. One copy
    // here and every reader after this point works on a plain array, and a null becomes
    // an empty one rather than a crash at the first filter.
    function asArray(v) {
        if (!v) return []
        var out = []
        for (var i = 0; i < v.length; i++) out.push(v[i])
        return out
    }
    readonly property var audioTracks: trackList.filter(function(t) { return t.type === "audio" }).map(function(t) { return { id: t.id, label: Player.trackLabel(t), track: t } })
    readonly property var subTracks: trackList.filter(function(t) { return t.type === "sub" }).map(function(t) { return { id: t.id, label: Player.trackLabel(t), track: t } })
    // The session's own track_choice keeps serde's shape, the string "Off" or
    // { Track: { track } }; the door takes {} for none and { off: true } for Off.
    function subtitleArg() {
        var s = session.track_choice.subtitle
        if (!s) return ({})
        return s === "Off" ? { off: true } : s
    }
    function audioArg() { return session.track_choice.audio ? session.track_choice.audio : ({}) }
    function storeChoice(audio, subtitle) {
        var r = Door.setTrackChoice(session.series, audio, subtitle)
        if (r.error) console.warn("anibeam: track choice:", r.error.message)
        return !r.error
    }
    // The id comes off a list that mpv can have replaced since the picker drew it, so a
    // track that is no longer there changes nothing rather than storing an empty choice.
    function pickAudio(id) {
        var t = trackList.find(function(x) { return x.type === "audio" && x.id === id })
        if (!t) return
        video.setProperty("aid", String(id))
        var ref = Player.trackRef(t)
        storeChoice(ref, subtitleArg())
        session.track_choice.audio = ref
        showChrome()
    }
    function pickSubtitle(id) {
        var t = id < 0 ? null : trackList.find(function(x) { return x.type === "sub" && x.id === id })
        if (id >= 0 && !t) return
        video.setProperty("sid", id < 0 ? "no" : String(id))
        var choice = id < 0 ? { off: true } : { Track: { track: Player.trackRef(t) } }
        storeChoice(audioArg(), choice)
        session.track_choice.subtitle = id < 0 ? "Off" : choice
        showChrome()
    }
    // C: off, then back to the track it was on. The choice is not stored, because a key
    // pressed to read one sign is not a decision about the series.
    function toggleSubtitles() {
        if (sid >= 0) video.setProperty("sid", "no")
        else if (lastSid >= 0) video.setProperty("sid", String(lastSid))
        else if (subTracks.length) video.setProperty("sid", String(subTracks[0].id))
        showChrome()
    }
    // z and Z. The page's own value moves first so a second press within the round trip
    // adds to the first rather than repeating it; the observation then rewrites the line
    // with what mpv actually settled on, so the HUD reports the player, not the intent.
    function delayLine(v) { return "subtitle delay " + (v >= 0 ? "+" : "") + v.toFixed(1) + " s" }
    function nudgeDelay(d) {
        var v = Math.round((subDelay + d) * 10) / 10
        subDelay = v
        video.setProperty("sub-delay", v)
        hud.flash(delayLine(v), "subDelay")
        showChrome()
    }

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

    // ---- The seek preview: a second mpv core, nothing audible, moved by time-pos. It is
    // its own core rather than a thumbnail file, so it costs nothing on disk and answers
    // any position; it is declared above the video and below the island so it can never
    // cover the controls it sits over.
    Corner {
        id: preview
        // It hides with the chrome as well as on exit: the island can go while the pointer
        // rests on the bar, and the bar goes with it, so no exit would ever arrive.
        property bool shown: false
        visible: shown && page.chromeVisible
        width: theme.space(60); height: width * 9 / 16 + theme.space(6)
        radius: theme.radiusMd; smoothing: theme.cornerSmoothing
        color: theme.scrim; borderColor: theme.line; borderWidth: 1
        y: controls.y - height - theme.space(2)
        property bool loaded: false
        function show(secs, centerX) {
            x = Math.max(theme.space(2), Math.min(centerX - width / 2, page.width - width - theme.space(2)))
            stamp.text = Fmt.clock(secs)
            shown = true
            if (loaded) previewVideo.setPropertyAsync("time-pos", secs)
        }
        function hide() { shown = false }
        VideoItem {
            id: previewVideo
            anchors.left: parent.left; anchors.right: parent.right; anchors.top: parent.top; anchors.margins: theme.space(1)
            height: width * 9 / 16
            onReady: {
                var o = Player.previewOptions
                for (var i = 0; i < o.length; i++) setProperty(o[i][0], o[i][1])
                if (page.session) command(["loadfile", page.session.path])
            }
            onLoaded: preview.loaded = true
        }
        Text { id: stamp; anchors.bottom: parent.bottom; anchors.bottomMargin: theme.space(1); anchors.horizontalCenter: parent.horizontalCenter; color: theme.text; font.family: theme.fontMono; font.pointSize: theme.typeSmall }
    }

    // ---- Controls island
    Corner {
        id: controls
        anchors.horizontalCenter: parent.horizontalCenter
        anchors.bottom: parent.bottom; anchors.bottomMargin: theme.space(6)
        width: Math.min(parent.width - theme.space(12), theme.space(220))
        height: bottomRow.height + seekSlot.height + theme.space(9)
        radius: theme.radiusLg; smoothing: theme.cornerSmoothing
        color: theme.scrim; borderColor: theme.line; borderWidth: 1
        opacity: page.chromeVisible ? 1 : 0
        visible: opacity > 0
        Behavior on opacity { NumberAnimation { duration: theme.motionNormal } }
        MouseArea { anchors.fill: parent; hoverEnabled: true; onPositionChanged: page.showChrome() }
        // The bar is above the island's own hover MouseArea, so the hover that keeps the
        // chrome up while the pointer sits on the bar comes from the bar itself.
        Item { id: seekSlot; anchors.left: parent.left; anchors.right: parent.right; anchors.top: parent.top; anchors.margins: theme.space(3); height: seek.height
            SeekBar { id: seek; width: parent.width; position: page.timePos; duration: page.duration; windows: page.windows
                onSeeked: function(s) { page.seekTo(s) }
                onHovered: function(s) { page.showChrome(); preview.show(s, seek.mapToItem(page, page.duration > 0 ? s / page.duration * seek.width : 0, 0).x) }
                onUnhovered: preview.hide() } }
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
                    PlayerButton { id: audioBtn; glyph: "audio-lines"; tip: "Audio track"; visible: page.audioTracks.length > 1; onClicked: audioPicker.openAt(audioBtn) }
                    PlayerButton { id: subBtn; glyph: "captions"; tip: page.sid >= 0 ? "Subtitles" : "Subtitles off"; active: page.sid >= 0; visible: page.subTracks.length > 0; onClicked: subPicker.openAt(subBtn) }
                    PlayerButton { glyph: frame.hostWindow.visibility === Window.FullScreen ? "minimize" : "maximize"; tip: "Fullscreen"; onClicked: page.toggleFullscreen() } } }
        }
    }

    // ---- The pickers. They fill the page and hold the chrome while they are up.
    TrackPicker { id: audioPicker; title: "Audio"; tracks: page.audioTracks; selected: page.aid; onPicked: function(id) { page.pickAudio(id) } }
    TrackPicker { id: subPicker; title: "Subtitles"; tracks: page.subTracks; selected: page.sid; offRow: true; onPicked: function(id) { page.pickSubtitle(id) } }

    // ---- The HUD line: one message over the picture, gone after 1.2 s. Task 12's frame
    // step shares it.
    Corner {
        id: hud
        visible: false
        anchors.top: parent.top; anchors.topMargin: theme.space(20); anchors.horizontalCenter: parent.horizontalCenter
        width: hudText.implicitWidth + theme.space(6); height: theme.controlHeight
        radius: height / 2; smoothing: theme.cornerSmoothing; color: theme.scrim; borderColor: theme.line; borderWidth: 1
        // Which key owns the line on screen, so an observation that lands afterwards can
        // rewrite its own line and no other. An omitted kind belongs to nobody.
        property string kind: ""
        Text { id: hudText; anchors.centerIn: parent; color: theme.text; font.family: theme.fontMono; font.pointSize: theme.typeNormal }
        Timer { id: hudTimer; interval: 1200; onTriggered: hud.clear() }
        function flash(text, kind) { hud.kind = kind || ""; hudText.text = text; visible = true; hudTimer.restart() }
        // The text alone, with the countdown left alone: a confirmation is not a new message.
        function update(text, kind) { if (visible && hud.kind === (kind || "")) hudText.text = text }
        function clear() { visible = false; hud.kind = ""; hudTimer.stop() }
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
        else if (e.key === Qt.Key_C) page.toggleSubtitles()
        else if (e.key === Qt.Key_Z && !(e.modifiers & Qt.ShiftModifier)) page.nudgeDelay(-0.1)
        else if (e.key === Qt.Key_Z) page.nudgeDelay(0.1)
        // A picker is open: the press belongs to the frame's escape stack, which closes it.
        // Accepting it here would leave the player and take the picker with it.
        else if (e.key === Qt.Key_Escape) {
            if (page.openMenus > 0) e.accepted = false
            else if (frame.hostWindow.visibility === Window.FullScreen) frame.hostWindow.visibility = Window.Windowed
            else page.leave()
        }
        else e.accepted = false
    }
}
