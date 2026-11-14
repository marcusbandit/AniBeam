// Watch together (#846): the room this window is in, kept by the frame so it outlives the
// player page. Everyone plays their own copy; the core owns the socket, the clock and the
// sync, and this keeps what the panels show and does what the core asks of the shell:
//
// - Create and Join wait on the relay's welcome for up to 20 s, so they run off the Qt
//   thread through Door.callAsync and the panel shows them as busy.
// - A join is answered by RoomEpisode: with a file this library holds, the player opens on it
//   and attaches; without one, the panel offers Get media, which downloads the host's copy.
// - The player calls playerOpened() with each session it opens. The host's always attaches,
//   which is how the host's next episode moves the whole room; anyone else's attaches when it
//   is the room's episode.
// - A name comes first: the core refuses Create and Join without one, so the action waits
//   while the panel asks, and runs once the name is saved.
import QtQuick
import com.marcusrosado.AniBeam

QtObject {
    id: root
    // RoomInfo while in a room, else null
    property var info: null
    readonly property bool inRoom: info !== null
    readonly property bool host: inRoom && info.host === true
    // "create" or "join" while one waits on the relay
    property string busy: ""
    // The last thing that went wrong, as a sentence that says what to do; cleared by the next try
    property string error: ""
    // The last RoomEpisode: { episode, series, file }, file -1 when this library lacks it
    property var episode: null
    readonly property bool missing: inRoom && episode !== null && episode.file < 0
    // The player page showing now, or null
    property Item player: null
    // The session the room drives, when it is the player's
    readonly property bool drivesPlayer: inRoom && player !== null && player.session !== null
                                         && info.session === player.session.session

    // The name the room shows; the core refuses rooms without one
    // The name just saved stands in until the core's PreferencesChanged brings it back, so the
    // action that waited on it runs at once
    property string savedName: ""
    readonly property string name: savedName !== "" ? savedName : String(Door.preferences.room_name || "")
    readonly property bool needsName: name.trim() === ""
    // A name to offer when there is none: the tracker account's, read from what the frame already holds
    readonly property string suggestedName: {
        var t = Door.trackers || {}
        if (t.anilist && t.anilist.connected && t.anilist.username) return String(t.anilist.username)
        if (t.mal && t.mal.connected && t.mal.username) return String(t.mal.username)
        return ""
    }
    // What waits on the name: { kind: "create", session } or { kind: "join", code }
    property var waiting: null

    function setName(n) {
        n = String(n || "").trim()
        if (n === "") { error = words.t("room-name-empty"); return false }
        var p = JSON.parse(JSON.stringify(Door.preferences))
        p.room_name = n
        var r = Door.setPreferences(p)
        if (r.error) { error = r.error.kind === "Invalid" ? words.t("room-name-bad") : errorText(r.error); return false }
        error = ""
        savedName = n
        var w = waiting
        waiting = null
        if (w && w.kind === "create") create(w.session)
        else if (w && w.kind === "join") join(w.code)
        return true
    }
    function setRelay(url) {
        var p = JSON.parse(JSON.stringify(Door.preferences))
        p.room_relay = String(url || "").trim()
        var r = Door.setPreferences(p)
        if (r.error) { frame.fail(r.error); return false }
        return true
    }

    // ---- The calls that wait on the relay, off the Qt thread
    property var pending: ({})   // token -> kind
    function callAsync(kind, name, args) {
        var t = Door.callAsync(name, args)
        if (t < 0) { busy = ""; error = words.t("room-error-unreachable"); return }
        var p = Object.assign({}, pending)
        p[t] = kind
        pending = p
        busy = kind
    }
    function create(session) {
        if (busy !== "") return
        error = ""
        if (needsName) { waiting = { kind: "create", session: session }; return }
        callAsync("create", "RoomCreate", { session: session })
    }
    function join(code) {
        if (busy !== "") return
        code = String(code || "").trim()
        error = ""
        if (code === "") { error = words.t("room-error-bad-code"); return }
        if (needsName) { waiting = { kind: "join", code: code }; return }
        episode = null
        callAsync("join", "RoomJoin", { code: code })
    }
    function cancelWaiting() { waiting = null; error = "" }
    function leave() {
        waiting = null
        error = ""
        var r = Door.call("RoomLeave", {})
        if (r.error) frame.fail(r.error)
    }
    function attach(session) {
        var r = Door.call("RoomAttach", { session: session })
        if (r.error) { error = errorText(r.error); return false }
        if (r.reply && r.reply.room) info = r.reply.room
        return true
    }
    // The person's own play, pause or seek on the player the room drives, at its own position
    function command(action, position) {
        if (!drivesPlayer) return
        var r = Door.call("RoomCommand", { action: action, position: Math.max(0, position) })
        if (r.error) console.warn("anibeam: room command:", r.error.message)
    }
    function setOffset(v) {
        var r = Door.call("RoomSetOffset", { offset: Math.round(v * 100) / 100 })
        if (r.error) { error = errorText(r.error); return }
        if (r.reply && r.reply.room) info = r.reply.room
    }
    function nudgeOffset(d) { if (inRoom) setOffset(info.offset + d) }
    // Get media (#889): the host's app passes this member its torrent or its release, and the
    // core downloads it; the episode opens once it is in the library (RoomEpisode)
    function getMedia() {
        error = ""
        var r = Door.call("RoomGetMedia", {})
        if (r.error) { error = errorText(r.error); return }
        if (r.reply && r.reply.room) info = r.reply.room
    }

    // The player opened a session: the host's always drives the room, anyone else's when it
    // is the episode the room watches
    function playerOpened(page) {
        player = page
        if (!inRoom || !page.session) return
        var file = page.session.file
        if (host || (episode && episode.file === file)) attach(page.session.session)
    }
    function playerClosed(page) { if (player === page) player = null }
    // A joiner's episode can arrive before the room itself does: the player that opened on it
    // attaches once both are here
    function settle() {
        if (!inRoom || host || drivesPlayer || !player || !player.session || !episode) return
        if (episode.file === player.session.file) attach(player.session.session)
    }

    // A RoomEpisode with a file: the player that has it attaches, else the player opens on it
    function openEpisode() {
        if (!episode || episode.file < 0) return
        var p = player
        if (p && p.session && p.session.file === episode.file) {
            if (!drivesPlayer) attach(p.session.session)
            return
        }
        var title = episode.episode.series_title || ""
        // The player takes over from the frame's panel
        if (frame.roomSheetPanel.open) frame.roomSheetPanel.close()
        if (p) frame.nav.replace("player", { file: episode.file }, title)
        else frame.go("player", { file: episode.file }, title)
    }

    // Every room error as a sentence that says what to do
    function errorText(e) {
        if (!e) return ""
        if (e.kind === "Service") {
            var ids = { not_found: "room-error-not-found", full: "room-error-full", unreachable: "room-error-unreachable",
                        refused: "room-error-refused", proto: "room-error-proto", not_in_room: "room-error-not-in-room",
                        cannot_fetch: "room-error-cannot-fetch" }
            if (ids[e.code]) return words.t(ids[e.code])
        }
        if (e.kind === "Invalid" && e.field === "code") return words.t("room-error-bad-code")
        if (e.kind === "Invalid" && e.field === "room_relay") return words.t("room-error-bad-relay")
        if (e.kind === "Invalid" && e.field === "room_name") return words.t("room-name-bad")
        return e.plain || e.message || ""
    }

    // Reads where the core is, once the frame is up: a window opened again finds no room,
    // but a reload of the QML would
    function refresh() {
        var r = Door.call("RoomStatus", {})
        if (!r.error) info = r.reply.room || null
    }

    property Connections events: Connections {
        target: Door
        function onPreferencesChanged() { root.savedName = "" }
        function onRoomChanged(room) {
            var was = root.info ? String(root.info.fetch || "") : ""
            root.info = room && room.code ? room : null
            if (!root.info) { root.episode = null; return }
            root.settle()
            // A Get media that stopped short says why in the line the core logged with it,
            // which the bridge sets before it passes the room on
            if (root.info.fetch === "Failed" && was !== "Failed") {
                var l = Door.latestPlain
                if (l && l.plain_words) root.error = words.t(l.plain_words.id, l.plain_words.args)
                else if (l && l.plain) root.error = String(l.plain)
            }
        }
        function onRoomEpisode(ep, series, file) {
            root.episode = { episode: ep, series: series, file: file }
            if (file >= 0) root.openEpisode()
        }
        function onCallDone(token, reply) {
            if (!(token in root.pending)) return
            var kind = root.pending[token]
            var p = Object.assign({}, root.pending)
            delete p[token]
            root.pending = p
            root.busy = ""
            if (reply.error) {
                // A name the core will not take: ask again rather than fail
                if (reply.error.kind === "Invalid" && reply.error.field === "room_name") { root.error = words.t("room-name-bad"); return }
                root.error = root.errorText(reply.error)
                return
            }
            if (reply.reply && reply.reply.room) { root.info = reply.reply.room; root.settle() }
            // A new room starts paused at nought: it starts where the host's player is instead
            if (kind === "create" && root.drivesPlayer) root.command(root.player.paused ? "Pause" : "Play", root.player.timePos)
        }
    }
}
