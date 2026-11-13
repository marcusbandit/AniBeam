// A newer desktop build is out: one quiet island over the page's bottom right corner. The
// frame asks once as the app starts (check()), Settings asks again when the track changes,
// and Shell answers through updateAvailable with how this copy takes the update:
// - install: Update downloads it here, with the progress along the island's foot and a
//   Cancel, and the app restarts into it once it is ready. A download that finishes while the player, a sheet
//   or What's new has the window waits for a Restart click instead.
// - command: a package manager owns this copy (the AUR), so the island shows the command
//   that updates it and a Copy button. Shell only says so once the AUR has the version.
// - source: a build from a checkout, which only a pull and a rebuild moves on.
// - page: the release has no file for this copy, so Release page opens it.
// The track is Stable or Nightly, and until someone picks one it is the running build's
// own; a nightly's sticks. Later hides it until a newer version than this one comes out;
// the version passed over and the track live in shell.conf. It steps aside while the player
// has the window or a sheet is open (`covered`), and comes back once they close.
// Put away or not, an offer stays `pending` until it is installed or the track changes:
// the rail's Settings icon and the About row badge it, and About keeps the way to take it.
import QtQuick
import QtCore
import com.marcusrosado.AniBeam

Frost {
    id: root
    objectName: "update-notice"
    visible: shown && !covered
    radius: theme.radiusTile
    // One width for every state, wider only when the words need it, so the island does not
    // jump about as the update moves along
    readonly property real pad: theme.space(4)
    width: Math.max(theme.space(116), words.implicitWidth + actions.implicitWidth + pad * 2 + theme.space(6))
    height: Math.max(words.implicitHeight, actions.implicitHeight) + theme.space(6)

    property bool shown: false
    // An update is offered for this track, whether or not the island shows it
    property bool pending: false
    property bool covered: false
    property string version: ""
    // The release's own name, "Quality of life"; empty for one known by its number
    property string title: ""
    property string url: ""
    // install, command, source or page, as Shell offered it
    property string how: ""
    property string command: ""
    // offer, downloading, ready or failed
    property string phase: "offer"
    property real got: 0
    property real total: 0
    readonly property real progress: total > 0 ? Math.min(1, got / total) : 0
    property string failure: ""
    property bool copied: false

    Settings {
        id: kept
        location: Shell.settingsUrl
        category: "update"
        // The newest version the notice was put away for
        property string skipped: ""
        // stable or nightly, empty to follow the running build
        property string track: ""
    }

    // The track updates come from, as Settings shows it
    readonly property string track: kept.track !== "" ? kept.track : Shell.ownTrack
    readonly property bool busy: phase === "downloading" || phase === "ready"
    function check() {
        // A nightly keeps its track once picked by default, so the stable release it leads
        // to does not quietly move it to Stable when installed
        if (kept.track === "" && Shell.ownTrack === "nightly") kept.track = "nightly"
        Shell.checkForUpdate(track)
    }
    function setTrack(t) {
        kept.track = t
        // An update already on its way is not dropped for a change of track; if it stops
        // short, its offer goes and the new track is asked then
        if (busy) { trackMoved = true; return }
        drop()
    }
    // The track changed under a download that has not landed
    property bool trackMoved: false
    // The offer goes, notice and badges with it, and the track is asked again
    function drop() {
        trackMoved = false
        shown = false
        pending = false
        phase = "offer"
        check()
    }

    Connections {
        target: Shell
        // An answer for a track since switched away from is dropped
        function onUpdateAvailable(version, title, url, track, how, command) {
            if (track === root.track && !root.busy) root.show(version, title, url, how, command)
        }
        function onUpdateCurrent(track, error) {
            if (track !== root.track) return
            // Nothing newer on this track: an offer from an earlier check is gone, the
            // notice with it
            if (!error && !root.busy) { root.pending = false; root.shown = false; root.phase = "offer" }
            if (!root.asked) return
            root.asked = false
            if (error) frame.toast("AniBeam could not check for updates. Check your internet connection, then try again.", 8, "Try again", root.checkNow, String(error))
            else frame.toast("AniBeam is up to date")
        }
        function onUpdateProgress(got, total) {
            if (root.phase !== "downloading") return
            root.got = got
            root.total = total
        }
        // Restarts at once unless the player, a sheet or What's new has the window; then
        // Restart waits for a click
        function onUpdateReady() {
            root.phase = "ready"
            if (!root.covered) root.restart()
        }
        function onUpdateFailed(message) {
            if (root.trackMoved) { root.drop(); return }
            root.failure = message; root.phase = "failed"; root.shown = true
        }
    }

    // Check for updates in About: like the launch check, but it shows the offer even for a
    // version put off with Later, and says so when there is nothing newer
    property bool asked: false
    function checkNow() { asked = true; laterFor = ""; check() }
    // Later puts the offer away for this run only: the next start offers it again
    property string laterFor: ""
    function show(version, title, url, how, command) {
        asked = false
        root.version = version
        root.title = title
        root.url = url
        root.how = how
        root.command = command
        root.phase = "offer"
        root.copied = false
        root.pending = true
        // Put off with Later: About and the badges still have it, the island stays away
        if (version !== laterFor) shown = true
    }
    function later() {
        // A readied update still lands when the app quits; only an offer is passed over
        if (phase === "offer") laterFor = root.version
        shown = false
    }
    function install() {
        root.got = 0
        root.total = 0
        root.phase = "downloading"
        Shell.installUpdate(root.version)
    }
    function cancel() {
        Shell.cancelUpdate()
        root.phase = "offer"
        if (trackMoved) drop()
    }
    function restart() {
        Shell.restartForUpdate()
        Qt.quit()
    }
    function copyCommand() {
        frame.copyQuiet(root.command)
        root.copied = true
    }

    // The words on the island's buttons, which About's own buttons use too
    readonly property string copyText: copied ? "Copied" : "Copy"
    readonly property string restartText: "Restart"
    readonly property string retryText: "Retry"
    readonly property string cancelText: "Cancel"
    readonly property string pageText: "Release page"

    function mb(bytes) { return Math.round(bytes / 1048576) }
    readonly property string headline: {
        if (phase === "downloading") return "Downloading AniBeam " + version
        if (phase === "ready") return "AniBeam " + version + " is ready"
        if (phase === "failed") return "The update failed"
        if (how === "command") return "AniBeam " + version + " is on the AUR"
        return "AniBeam " + version + (title ? " \u201c" + title + "\u201d" : "") + " is out"
    }
    readonly property string detail: {
        if (phase === "downloading") return Math.floor(progress * 100) + "%  ·  " + mb(got) + " of " + mb(total) + " MB"
        if (phase === "ready") return "Restart now, or it opens next time"
        if (phase === "failed") return failure
        if (how === "command") return command
        if (how === "source") return "This copy is built from source: pull and rebuild"
        if (how === "page") return "No download here for this computer"
        return "Update installs it and restarts AniBeam"
    }

    Column {
        id: words
        anchors.left: parent.left; anchors.leftMargin: root.pad
        anchors.verticalCenter: parent.verticalCenter
        spacing: theme.space(1)
        Label {
            objectName: "update-line"
            role: "ui"
            text: root.headline
        }
        Label {
            objectName: "update-detail"
            // The command is what gets typed, so it reads at full strength in a fixed face;
            // the progress line keeps its digits still
            role: root.phase === "downloading" ? "ledger" : "ui"
            font.family: root.phase === "offer" && root.how === "command" ? "monospace" : theme.roleFamily(role)
            font.pointSize: theme.typeSmall
            color: root.phase === "offer" && root.how === "command" ? theme.text : theme.textDim
            text: root.detail
        }
    }

    Row {
        id: actions
        anchors.right: parent.right; anchors.rightMargin: root.pad
        anchors.verticalCenter: parent.verticalCenter
        spacing: theme.space(2)
        Button {
            objectName: "update-install"
            visible: (root.phase === "offer" && root.how === "install") || root.phase === "failed"
            text: root.phase === "failed" ? root.retryText : "Update"
            icon: root.phase === "failed" ? "rotate-cw" : "download"
            primary: true; small: true
            onClicked: root.install()
        }
        Button {
            objectName: "update-restart"
            visible: root.phase === "ready"
            text: root.restartText; icon: "refresh-cw"; primary: true; small: true
            onClicked: root.restart()
        }
        Button {
            objectName: "update-copy"
            visible: root.phase === "offer" && root.how === "command"
            text: root.copyText; icon: root.copied ? "check" : "copy"
            primary: true; small: true
            onClicked: root.copyCommand()
        }
        Button {
            objectName: "update-page"
            visible: root.phase === "offer" && root.how === "page"
            text: root.pageText; icon: "external-link"; primary: true; small: true
            onClicked: { root.shown = false; frame.openLink(root.url) }
        }
        Button {
            objectName: "update-cancel"
            visible: root.phase === "downloading"
            text: root.cancelText; icon: "x"; small: true
            onClicked: root.cancel()
        }
        Button {
            objectName: "update-later"
            visible: root.phase !== "downloading"
            text: "Later"; icon: "clock"; small: true
            onClicked: root.later()
        }
    }

    // The download along the island's foot, inset clear of its rounded corners
    Rectangle {
        visible: root.phase === "downloading"
        anchors.left: parent.left; anchors.right: parent.right; anchors.bottom: parent.bottom
        anchors.leftMargin: theme.space(3); anchors.rightMargin: theme.space(3); anchors.bottomMargin: 3
        height: 3; radius: 1.5
        color: Qt.rgba(theme.accent.r, theme.accent.g, theme.accent.b, 0.22)
        Rectangle {
            objectName: "update-progress"
            width: parent.width * root.progress
            height: parent.height; radius: 1.5
            color: theme.accent
        }
    }
}
