// Spec 4.4: the player. The one page that takes the whole window. libmpv draws through
// VideoItem; the shell draws every overlay and handles every key. The core owns the
// rules; the shell sends ticks and shows what comes back.
//
// The page holds the session, the mpv wiring, the rules and the keys; PlayerChrome.qml holds
// the header, the island, the auto-next pill and the replay button, and reads its state from
// here. The page contract the frame relies on (props, fullWindow, title, escapePressed,
// scrollY) stays on this object.
import QtQuick
import QtQuick.Window
import com.marcusrosado.AniBeam

FocusScope {
    id: page
    // The player is a dark room in either mode: its own tokens, forced dark, so the glass
    // over the picture and the text on it read the same on a light theme. Everything the
    // page and PlayerChrome build resolves `theme` here first.
    Tokens { id: theme; mode: "dark" }
    property var props: ({})
    readonly property bool fullWindow: true
    property var session: null
    property string title: session ? session.series_title : "Player"
    property real scrollY: 0
    readonly property alias video: video
    // For the drive script: the rating prompt, which otherwise needs a tracked last episode.
    readonly property alias ratingPrompt: rating

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
    // Seeded by the observation and refreshed at each step, since mpv notifies this one only
    // once. See the frame step section.
    property real frameNumber: 0

    // Chrome
    property bool chromeVisible: true
    property int openMenus: 0            // pickers, the rating picker and the key list
    // A popover's own button or row may have held the keyboard while it was up; the page
    // takes it back when the last one closes, so Space and the arrows work again at once.
    onOpenMenusChanged: if (openMenus === 0) forceActiveFocus()
    // A menu or a held scrub holds the chrome: the timer runs again rather than falling
    // through, so the chrome hides 2.5 s after either ends instead of staying up until the next move.
    Timer { id: hideTimer; interval: 2500; onTriggered: { if (page.openMenus > 0 || chrome.scrubbing) restart(); else page.chromeVisible = false } }
    function showChrome() { chromeVisible = true; hideTimer.restart() }
    // The pointer goes with the chrome, as an application override (see helpers.h)
    onChromeVisibleChanged: Shell.setCursorHidden(!chromeVisible)

    // ---- Session
    Component.onCompleted: {
        var r = Door.openPlayback(props.file)
        if (r.error) { frame.toast(r.error.message); Qt.callLater(frame.nav.back); return }
        session = r.reply.session
        readSeries()
        if (video.isReady) start()
        showChrome()
        forceActiveFocus()
    }
    // The session carries the series and the file, not the episode number and not the
    // tracker ids, so the detail is read once here. A failure costs the mark button and the
    // rating prompt and nothing else, so it warns rather than leaving the page.
    // `seriesTitles` is for Task 13's MPRIS metadata, which wants the untranslated titles.
    property bool trackerKnown: false
    property var seriesTitles: ({})
    property real episodeNumber: -1
    // The header's second line: the episode's title, else "Episode N" through the shared
    // helper, else (an extra or an unmatched file, which have no number) the file's name.
    readonly property string episodeLine: !session ? ""
        : session.episode_title ? session.episode_title
        : !session.is_extra && episodeNumber >= 0 ? Fmt.episodeName("", episodeNumber)
        : session.path.split("/").pop()
    readonly property bool canMark: trackerKnown && !!session && !session.is_extra && episodeNumber >= 0
    function readSeries() {
        var d = Door.getSeries(session.series)
        if (d.error) { console.warn("anibeam: series detail:", d.error.message); return }
        var card = d.reply.detail.card
        trackerKnown = !!(card.match_info && (card.match_info.anilist_id || card.match_info.mal_id))
        seriesTitles = card.titles || {}
        var ep = d.reply.detail.episodes.find(function(e) { return e.file === session.file })
        episodeNumber = ep ? ep.number : -1
    }
    Component.onDestruction: {
        Shell.setCursorHidden(false)
        flushVolume()
        close("Stopped")
        // The page going away is the one release keepAwake cannot see
        Player.setKeepAwake(false)
        // The page is the only thing that ever publishes; leaving it leaves nothing playing.
        Player.mprisUpdate("Stopped", "", "", "", 0, false, false)
    }
    property bool closed: false
    function close(reason) {
        if (!session || closed) return
        closed = true
        tickTimer.stop()
        // The auto-next countdown is deliberately not stopped here: the end of the file
        // closes the session and the countdown is what carries us into the next episode.
        // Every other caller destroys the page, and the timer goes with it.
        Door.closePlayback(session.session, timePos, reason)
    }
    function leave() { close("Stopped"); frame.nav.back() }
    function openNeighbour(file) { if (!file) return; close("Switched"); frame.nav.replace("player", { file: file }, page.title) }
    // N and P: the same neighbours, and the same rule, as the island's two buttons. An extra
    // has no place in the episode order, so it has neither.
    readonly property bool hasNext: !!(session && session.next && !session.is_extra)
    readonly property bool hasPrevious: !!(session && session.prev && !session.is_extra)
    function openNext() { if (hasNext) openNeighbour(session.next) }
    function openPrevious() { if (hasPrevious) openNeighbour(session.prev) }
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
        applyDefaults()
        video.setProperty("volume", page.volume)
        video.setProperty("mute", Player.mute)
        video.setProperty("start", session.resume_from ? String(session.resume_from) : "none")
        video.command(["loadfile", session.path])
    }
    // Spec 4.4's subtitle table as mpv options. These are set before the file, so they are
    // in force for its first frame.
    function applyDefaults() { applySubtitleOptions(session.subtitle_defaults) }
    // The styling options, and the subtitle position they set noted for the lift below:
    // the lift is relative to the viewer's own position, and has to be set again after
    // them, since they would put the position back.
    function applySubtitleOptions(d) {
        var opts = Player.subtitleOptions(d)
        for (var i = 0; i < opts.length; i++) {
            if (opts[i][0] === "sub-pos") subPosSetting = Number(opts[i][1])
            video.setProperty(opts[i][0], opts[i][1])
        }
        video.setProperty("sub-pos", subPos)
    }
    // ---- The subtitle lift. While the controls tile is up it covers the foot of the
    // picture, where the subtitles are, so mpv's sub-pos (percent of the picture's height,
    // ASS included) is raised until the bottom line clears the tile by a grout, and put back
    // to the viewer's own setting when the controls go.
    property real subPosSetting: 100
    readonly property int subPos: chromeVisible && height > 0 && chrome.controlsTop > 0
        ? Math.max(0, Math.min(subPosSetting, Math.floor(100 * (chrome.controlsTop - theme.grout) / height)))
        : subPosSetting
    onSubPosChanged: if (video.isReady) video.setProperty("sub-pos", subPos)
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
            page.applySubtitleOptions(d)
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
            observe("speed", VideoItem.Double)
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
            page.onFileLoaded()
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
            else page.onObserved(name, value)
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
        // mpv sends this one exactly once, when the observation is registered: it is not on
        // mpv's tick change list, so an observed value alone would read the number the file
        // opened on for the whole session. Kept because the spec's observe list has it and
        // it seeds the value; the number the HUD shows is read at the step. See readFrame().
        else if (name === "estimated-frame-number") frameNumber = Number(value || 0)
        else if (name === "speed") speed = Number(value || 1)
        else if (name === "chapter-list") {}
    }
    // The last frame is still on screen, because keep-open holds it: the replay button sits
    // on it, unless the countdown is already taking us to the next episode.
    function onEnded() { close("Ended"); if (!nextCounting) replayVisible = true }

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
    function delayLine(v) { return "Subtitle delay " + (v >= 0 ? "+" : "") + v.toFixed(1) + " s" }
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
    // The resume the viewer asked for ends the step here rather than waiting on the pause
    // observation, which the step guard may still be swallowing.
    function togglePause() { if (paused) endStepping(); video.setProperty("pause", !paused); showChrome() }
    // Every seek announces itself: MPRIS's Seeked is what tells a widget the position moved
    // for a reason other than playing, and a key, the bar, a skip chip and a D-Bus call are
    // all that same reason. The clamped target is what is announced, not what was asked for.
    function seekTo(secs) { var t = Math.max(0, Math.min(duration > 0 ? duration : secs, secs)); video.command(["seek", String(t), "absolute"]); Player.mprisSeeked(t); showChrome() }
    // The volume the page plays at. mpv gets every step at once; Player.setVolume writes
    // player.toml, so it gets the value only once the steps stop (a held Up is about 30
    // steps a second) and again when the page goes. A change from outside the page is
    // adopted unless a step of ours is still waiting to be saved.
    property real volume: Player.volume
    Timer { id: volumeSave; interval: 500; onTriggered: Player.setVolume(page.volume) }
    function flushVolume() { if (volumeSave.running) { volumeSave.stop(); Player.setVolume(volume) } }
    function setVolume(v) { v = Math.max(0, Math.min(100, v)); volume = v; video.setProperty("volume", v); if (v > 0 && Player.mute) setMute(false); volumeSave.restart(); showChrome() }
    function setMute(m) { video.setProperty("mute", m); Player.setMute(m); showChrome() }
    // Playback speed, [ and ] in quarter steps between 0.25 and 3, Backspace back to 1. The
    // page's value moves first so presses inside the round trip add up; the observation then
    // settles it on what mpv took. The speed shows in the controls tile's time row, which the
    // key brings up, never as a line over the picture, where top-placed signs would be. Each page is its own mpv core, so the next
    // file starts at the configured speed again rather than inheriting this one.
    property real speed: 1
    function speedLine(v) { return "Speed " + Number(v.toFixed(2)) + "\u00d7" }
    function setSpeed(v) {
        v = Math.max(0.25, Math.min(3, Math.round(v * 4) / 4))
        speed = v
        video.setProperty("speed", v)
        showChrome()
    }
    function stepSpeed(d) { setSpeed(speed + d * 0.25) }
    function toggleFullscreen() { frame.hostWindow.visibility = frame.hostWindow.visibility === Window.FullScreen ? Window.Windowed : Window.FullScreen }

    // ---- Skip windows and auto-skip
    // The session opens with whatever the file's chapters gave; SkipWindowsReady replaces
    // the list when AniSkip answers for this episode, which breaks the binding on purpose.
    property var windows: session ? session.skip_windows : []
    Connections { target: Door; function onSkipWindowsReady(session, ws) { if (page.session && session === page.session.session) page.windows = page.asArray(ws) } }
    function windowOf(kind) { return windows.find(function(w) { return w.kind === kind }) || null }
    readonly property var intro: windowOf("Intro")
    readonly property var outro: windowOf("Outro")
    function inside(w, t) { return !!w && t >= w.start && t < w.end }
    function skipWindow(w) { if (w) seekTo(w.end + 1) }
    function skipForward() {
        if (inside(intro, timePos)) skipWindow(intro)
        else if (inside(outro, timePos)) skipWindow(outro)
        else seekTo(timePos + 90)
    }

    // Armed per kind for the length of the session; Undo disarms its kind for good, and a
    // new session on the same episode is armed again. `landed` says a seek put the position
    // inside the window: the user asked to be there, so the frames that follow must not
    // skip back out. Keyed by SkipKind so the two kinds share one rule.
    property var armed: ({ Intro: true, Outro: true })
    property var landed: ({ Intro: false, Outro: false })
    property real lastPos: -1
    // Read once per settings change rather than once per observed frame: Door.settings is a
    // QJsonObject and every read of it rebuilds the JavaScript object behind it.
    readonly property var autoSkip: Door.settings.auto_skip || ({})
    property var noticeUndo: null
    // Entering a window by playback or by the session's opening resume point fires the
    // auto-skip; a seek into it does not, and does not fire on the frame after either.
    function autoSkipWindow(w, on, jumped) {
        if (!inside(w, timePos)) { if (w) landed[w.kind] = false; return }
        if (jumped) { landed[w.kind] = true; return }
        if (!on || !armed[w.kind] || landed[w.kind]) return
        armed[w.kind] = false
        skipWindow(w)
        noticeUndo = function() { seekTo(w.start); armed[w.kind] = false }
        notice.show(w.kind === "Intro" ? "Skipped intro" : "Skipped outro", "Undo")
    }
    onTimePosChanged: {
        var jumped = lastPos >= 0 && Math.abs(timePos - lastPos) > 2
        autoSkipWindow(intro, !!autoSkip.intro, jumped)
        autoSkipWindow(outro, !!autoSkip.outro, jumped)
        lastPos = timePos
        // A step has landed: the line is rewritten with where mpv actually went. While
        // playing this branch is never taken, since stepping only holds on a paused core.
        if (stepping) { readFrame(); hud.flash(frameLine(), "frame") }
        updateNext()
        // Not a signal: MPRIS's Position emits no PropertiesChanged, so this only keeps the
        // value a Get answers with. Seeked is emitted where a seek happens.
        Player.mprisPosition(timePos)
    }

    // ---- Auto-next and the replay. The pill appears when the outro starts, or eight
    // seconds from the end when there is no outro, and the countdown starts at the outro's
    // end when the outro runs to the end of the file, else three seconds from the end.
    property bool nextVisible: false
    property bool nextCounting: false
    property bool nextDismissed: false
    property bool replayVisible: false
    readonly property int nextCountMs: 5000
    Timer { id: nextTimer; interval: page.nextCountMs; onTriggered: page.openNeighbour(page.session.next) }
    function updateNext() {
        if (!session || !session.next || session.is_extra || nextDismissed || !loaded || duration <= 0) { cancelNext(); nextVisible = false; return }
        var remaining = duration - timePos
        var count = (outro && (duration - outro.end) < 20 && timePos >= outro.end) || remaining <= 3
        // A seek back out of the counting zone stops the countdown: someone who rewinds off
        // the end of the episode is not done with it, and it starts again if they play back
        // into the zone. The pill also stays up for as long as the countdown runs, so the
        // shell can never switch episodes with no Stay on screen.
        if (count && !nextCounting) { nextCounting = true; nextTimer.restart() }
        else if (!count && nextCounting) cancelNext()
        nextVisible = nextCounting || (outro ? timePos >= outro.start : remaining <= 8)
    }
    function cancelNext() { nextCounting = false; nextTimer.stop() }
    function stay() { nextDismissed = true; nextVisible = false; cancelNext() }

    // ---- The tracker: Mark watched, the outcomes, and the rating prompt on the last
    // episode. The mark the core fires by itself at the outro or 85 percent arrives on the
    // same signal, so the notice and the prompt read the same either way.
    readonly property var trackerNames: ({ Anilist: "AniList", Mal: "MyAnimeList" })
    function trackerName(t) { return trackerNames[t] || t }
    // A refusal is a rule the core applied, not something that went wrong, so it reads as a
    // sentence and the line it lands in does not say error. The bare enum name is what the
    // core sends and is not for anyone to read.
    readonly property var refusalWords: ({
        Hidden: "the show is hidden",
        NoMatch: "the show has no tracker match",
        NotNewer: "the tracker is already past this episode",
        Extra: "this is an extra",
        Unmatched: "the file is not matched to an episode",
        OnDisk: "the file is only on disk"
    })
    function refusalWord(r) { return refusalWords[r] || String(r) }
    function errorLine(e) { return e.kind === "Refused" ? "Not tracked: " + refusalWord(e.reason) + "." : e.message }
    function markWatched() {
        if (!canMark) return
        var r = Door.markEpisode(session.series, episodeNumber)
        if (r.error) notice.show(errorLine(r.error))
    }
    function outcomeText(o) {
        if (o.reason) return trackerName(o.tracker) + ": " + refusalWord(o.reason)
        return o.message ? trackerName(o.tracker) + ": " + o.message : trackerName(o.tracker) + " could not be updated"
    }
    function outcomeLine(outcomes) {
        var ok = outcomes.filter(function(o) { return o.ok }).map(function(o) { return trackerName(o.tracker) + " " + (o.progress === null || o.progress === undefined ? "ok" : "at " + o.progress) })
        var bad = outcomes.filter(function(o) { return !o.ok })
        var head = ok.length ? "Tracked" : (bad.every(function(o) { return !!o.reason }) ? "Not tracked" : "Could not update your list")
        var all = ok.concat(bad.map(outcomeText))
        // "Tracked: AniList at 12; MyAnimeList at 12", not a list joined by double spaces.
        return all.length ? head + ": " + all.join("; ") : head
    }
    function markOutcome(outcomes) {
        var list = asArray(outcomes)
        notice.show(outcomeLine(list))
        if (session && session.is_last_episode && list.some(function(o) { return o.ok })) rating.visible = true
    }
    Connections {
        target: Door
        function onMarked(series, episode, outcomes) { if (page.session && series === page.session.series) page.markOutcome(outcomes) }
        function onScored(series, score, outcomes) {
            if (!page.session || series !== page.session.series) return
            notice.show(page.asArray(outcomes).every(function(o) { return o.ok }) ? "Rated " + Fmt.score(score) : "Could not save the score")
        }
    }

    // ---- Frame step: mpv's own frame-step and frame-back-step on the paused core. Nothing
    // is anchored or predicted; the timestamp is the observed time-pos and the number is
    // mpv's own estimated-frame-number, read at the step (see readFrame).
    property bool stepping: false
    // mpv's own frame-step unpauses the core for one frame, so the observed `pause` dips to
    // false about a millisecond after the command and back about 40 ms later. Measured on
    // this box: frame-step dips, frame-back-step does not touch pause at all, so waiting for
    // pause to come back would hang on a backward step. A short guard after each step is what
    // separates that dip from the play the viewer asked for.
    // The guard's own expiry re-decides what it deferred: a play that landed inside the
    // 300 ms would otherwise leave `stepping` true for the rest of playback, and every
    // observed time-pos after it would read a property and rewrite a HUD line that never
    // cleared.
    Timer { id: stepGuard; interval: 300; onTriggered: if (!page.paused) page.endStepping() }
    function endStepping() { stepping = false; if (hud.kind === "frame") hud.clear() }
    function frameLine() { return "Frame " + frameNumber + " at " + Fmt.clockMs(timePos) }
    // One read per key press and one per landed step, both on an event, never on a timer:
    // mpv does not notify this property, so there is nothing to observe after the first value.
    function readFrame() { frameNumber = Number(video.getProperty("estimated-frame-number") || 0) }
    function step(dir) {
        if (!paused) video.setProperty("pause", true)
        stepping = true
        stepGuard.restart()
        readFrame()
        hud.flash(frameLine(), "frame")               // the frame still on screen
        video.command([dir > 0 ? "frame-step" : "frame-back-step"])
    }
    // Play clears the frame line at once, and only the frame line: a subtitle delay put on
    // screen while paused is a different message and keeps its own 1.2 s. A step's own dip
    // through unpaused is not a play, so it clears nothing. A pause also stops the auto-next
    // countdown: the pill keeps Next and Stay, and playing back into the counting zone
    // starts it again through updateNext().
    onPausedChanged: {
        if (paused) cancelNext()
        else if (!stepGuard.running) endStepping()
        publishNowPlaying()
    }

    // ---- Display sleep: mpv's own rule, a hold while playing and none while paused. The
    // render API has no window to take it, so the page asks. keep-open pauses at the end of
    // the file, so the end needs no term of its own, and a rewind and play after it holds
    // again. Only this page's player counts; the seek preview in the chrome is another core
    // and never reaches this. A frame step dips through unpaused, so it releases like the
    // pause it is. Leaving the page releases in onDestruction.
    readonly property bool keepAwake: loaded && !paused && !stepping
    onKeepAwakeChanged: Player.setKeepAwake(keepAwake)

    // ---- MPRIS: the two now-playing lines out, one command signal in. The show name is the
    // untranslated romaji first, because that is how the library and the trackers name a
    // series; the language switch is a shell preference and says nothing about D-Bus.
    function episodeNumberOf() { return session.is_extra || episodeNumber < 0 ? -1 : Math.round(episodeNumber) }
    function publishNowPlaying() {
        if (!session) return
        var show = seriesTitles.romaji || seriesTitles.english || seriesTitles.folder || session.series_title
        var lines = Player.nowPlaying(show, episodeNumberOf(), session.episode_title || "", session.is_extra ? session.episode_title || session.code : "")
        Player.mprisUpdate(paused ? "Paused" : "Playing", lines[0], lines[1], session.artwork ? Player.artUrl(session.artwork) : "",
                           duration, !!session.next && !session.is_extra, !!session.prev && !session.is_extra)
    }
    onDurationChanged: publishNowPlaying()
    onSessionChanged: publishNowPlaying()
    Connections {
        target: Player
        // A volume change from anywhere, the keys, the slider or the desktop, republishes,
        // so the desktop's own slider follows rather than snapping back at the next publish.
        function onVolumeChanged() { if (!volumeSave.running) page.volume = Player.volume; page.publishNowPlaying() }
        // Every command goes through the page's own transport, never at the mpv property:
        // that is what clears the frame-step guard on a resume and what persists a volume.
        function onMprisCommand(name, value) {
            if (name === "next") page.openNeighbour(page.session.next)
            else if (name === "previous") page.openNeighbour(page.session.prev)
            else if (name === "play") { if (page.paused) page.togglePause() }
            else if (name === "pause") { if (!page.paused) page.togglePause() }
            else if (name === "playPause") page.togglePause()
            else if (name === "stop") page.leave()
            else if (name === "seek") page.seekTo(page.timePos + value)     // seekTo announces
            else if (name === "setPosition") page.seekTo(value)
            else if (name === "setVolume") page.setVolume(value)   // the save republishes
        }
    }

    // ---- The chrome: header, preview, island, the auto-next pill and the replay button
    PlayerChrome { id: chrome }

    // ---- The pickers. They fill the page and hold the chrome while they are up. The chrome
    // and the key map open them through these three functions rather than by id, so what the
    // page offers its own parts is a contract rather than a reach up the context chain.
    function openAudioPicker(anchor) { audioPicker.openAt(anchor) }
    function openSubPicker(anchor) { subPicker.openAt(anchor) }
    function showHelp() { help.show() }
    function toggleHelp() { if (help.open) help.close(); else help.show() }
    TrackPicker { id: audioPicker; title: "Audio"; tracks: page.audioTracks; selected: page.aid; onPicked: function(id) { page.pickAudio(id) } }
    TrackPicker { id: subPicker; title: "Subtitles"; tracks: page.subTracks; selected: page.sid; offRow: true; onPicked: function(id) { page.pickSubtitle(id) } }

    // ---- The HUD line: one message over the picture, gone after 1.2 s. The subtitle delay
    // and the frame step share it, each owning the line through its own kind. A
    // pane of the video glass like every other island on the player; its text is figures,
    // so ledger.
    Corner {
        id: hud
        objectName: "player-hud"
        visible: false
        anchors.top: parent.top; anchors.topMargin: theme.space(20); anchors.horizontalCenter: parent.horizontalCenter
        width: hudText.implicitWidth + theme.space(6); height: theme.controlHeight
        color: theme.glassVideo; borderColor: Qt.rgba(1, 1, 1, 0.1); borderWidth: 1; highlight: theme.glassHighlight
        // Which key owns the line on screen, so an observation that lands afterwards can
        // rewrite its own line and no other. An omitted kind belongs to nobody.
        property string kind: ""
        readonly property alias text: hudText.text
        Label { id: hudText; anchors.centerIn: parent; role: "ledger"; font.pointSize: theme.typeNormal }
        Timer { id: hudTimer; interval: 1200; onTriggered: hud.clear() }
        function flash(text, kind) { hud.kind = kind || ""; hudText.text = text; visible = true; hudTimer.restart() }
        // The text alone, with the countdown left alone: a confirmation is not a new message.
        function update(text, kind) { if (visible && hud.kind === (kind || "")) hudText.text = text }
        function clear() { visible = false; hud.kind = ""; hudTimer.stop() }
    }

    // ---- The passing notice: Skipped intro and its Undo, and the tracker outcomes
    Notice { id: notice; onActed: if (page.noticeUndo) page.noticeUndo() }

    // ---- The rating prompt, on the last episode of a series once a mark has landed. A
    // raised tile at the top centre; its buttons never take the keyboard, so Space after a
    // click still pauses rather than opening the picker again.
    Tile {
        id: rating
        objectName: "player-rating"
        visible: false
        raised: true
        color: theme.glassVideo; borderColor: Qt.rgba(1, 1, 1, 0.1)
        padding: theme.space(3)
        anchors.horizontalCenter: parent.horizontalCenter; anchors.top: parent.top; anchors.topMargin: theme.space(24)
        width: ratingRow.implicitWidth + padding * 2; height: ratingRow.implicitHeight + padding * 2
        // The picture's own MouseArea is under the tile; a click on its padding must not pause.
        MouseArea { enabled: page.chromeVisible; anchors.fill: parent; anchors.margins: -rating.padding }
        Row {
            id: ratingRow
            anchors.centerIn: parent; spacing: theme.space(3)
            Icon { glyph: "check-check"; size: theme.space(4); anchors.verticalCenter: parent.verticalCenter }
            Label { anchors.verticalCenter: parent.verticalCenter; role: "ui"; text: "Tracked the final episode. Rate this show?" }
            Button { id: rateBtn; objectName: "player-rate"; anchors.verticalCenter: parent.verticalCenter; text: "Rate"; primary: true; small: true; takesFocus: false; onClicked: ratingPicker.openAt(rateBtn, -1) }
            Button { anchors.verticalCenter: parent.verticalCenter; text: "Not now"; flat: true; small: true; takesFocus: false; onClicked: rating.visible = false }
        }
    }
    // ScorePicker pushes itself on the frame's escape stack and pops on destruction, but it
    // knows nothing about the player's chrome, so the count that holds the chrome up while
    // it is open is kept here.
    ScorePicker {
        id: ratingPicker
        onOpenChanged: page.openMenus += open ? 1 : -1
        onSaved: function(v) { rating.visible = false; var r = Door.setScore(page.session.series, v); if (r.error) notice.show(page.errorLine(r.error)) }
    }

    // ---- The key list
    KeyHelp { id: help }

    // ---- Keys. The attached property stays here, on the item that holds focus; the map
    // itself is PlayerKeys.qml, beside the list KeyHelp.qml draws from it.
    PlayerKeys { id: keys }
    Keys.onPressed: function(e) { keys.handle(e) }
}
