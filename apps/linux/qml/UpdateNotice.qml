// A newer desktop build is out: one quiet island over the page's bottom right corner. The
// frame asks once as the app starts (check()), Settings asks again when the track changes,
// and Shell answers through updateAvailable with how this copy takes the update:
// - install: Update downloads it here, with the progress along the island's foot, and the
//   app restarts into it once it is ready. A download that finishes while the player has
//   the window waits for a Restart click instead, so nothing stops mid-episode.
// - command: a package manager owns this copy (the AUR), so the island shows the command
//   that updates it and a Copy button. Shell only says so once the AUR has the version.
// - source: a build from a checkout, which only a pull and a rebuild moves on.
// - page: the release has no file for this copy, so Release page opens it.
// The track is Stable or Nightly, and until someone picks one it is the running build's
// own; a nightly's sticks. Later hides it until a newer version than this one comes out;
// the version passed over and the track live in shell.conf. It steps aside while the player
// has the window or a sheet is open (`covered`), and comes back once they close.
import QtQuick
import QtCore
import com.marcusrosado.AniBeam

Frost {
    id: root
    objectName: "update-notice"
    visible: shown && !covered
    radius: theme.radiusTile
    width: row.implicitWidth + theme.space(6); height: theme.controlHeight + theme.space(2)

    property bool shown: false
    property bool covered: false
    property string version: ""
    property string url: ""
    // install, command, source or page, as Shell offered it
    property string how: ""
    property string command: ""
    // offer, downloading, ready or failed
    property string phase: "offer"
    property real progress: 0
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
        // An update already on its way is not dropped for a change of track
        if (busy) return
        shown = false
        check()
    }

    Connections {
        target: Shell
        // An answer for a track since switched away from is dropped
        function onUpdateAvailable(version, url, track, how, command) {
            if (track === root.track && !root.busy) root.show(version, url, how, command)
        }
        function onUpdateProgress(fraction) { root.progress = fraction }
        function onUpdateReady() {
            root.phase = "ready"
            if (!root.covered) root.restart()
        }
        function onUpdateFailed(message) { root.failure = message; root.phase = "failed"; root.shown = true }
    }

    function show(version, url, how, command) {
        if (version === kept.skipped) return
        root.version = version
        root.url = url
        root.how = how
        root.command = command
        root.phase = "offer"
        root.copied = false
        shown = true
    }
    function later() {
        // A readied update still lands when the app quits; only an offer is passed over
        if (phase === "offer") kept.skipped = root.version
        shown = false
    }
    function install() {
        root.progress = 0
        root.phase = "downloading"
        Shell.installUpdate()
    }
    function restart() {
        Shell.restartForUpdate()
        Qt.quit()
    }
    function copyCommand() {
        clipboard.text = root.command
        clipboard.selectAll()
        clipboard.copy()
        root.copied = true
    }
    TextEdit { id: clipboard; visible: false }

    readonly property string line: {
        if (phase === "downloading") return "Downloading AniBeam " + version + "  " + Math.floor(progress * 100) + "%"
        if (phase === "ready") return "AniBeam " + version + " is ready"
        if (phase === "failed") return "The update did not work: " + failure
        if (how === "command") return "AniBeam " + version + " is on the AUR"
        if (how === "source") return "AniBeam " + version + " is out. This copy is built from source, so pull and rebuild"
        return "AniBeam " + version + " is out"
    }

    Row {
        id: row
        anchors.centerIn: parent
        spacing: theme.space(3)
        Label {
            objectName: "update-line"
            anchors.verticalCenter: parent.verticalCenter
            role: "ui"
            text: root.line
        }
        Label {
            objectName: "update-command"
            visible: root.phase === "offer" && root.how === "command"
            anchors.verticalCenter: parent.verticalCenter
            role: "ledger"
            color: theme.textDim
            text: root.command
        }
        Button {
            objectName: "update-install"
            visible: (root.phase === "offer" && root.how === "install") || root.phase === "failed"
            anchors.verticalCenter: parent.verticalCenter
            text: root.phase === "failed" ? "Retry" : "Update"
            icon: root.phase === "failed" ? "rotate-cw" : "download"
            primary: true; small: true
            onClicked: root.install()
        }
        Button {
            objectName: "update-restart"
            visible: root.phase === "ready"
            anchors.verticalCenter: parent.verticalCenter
            text: "Restart"; icon: "refresh-cw"; primary: true; small: true
            onClicked: root.restart()
        }
        Button {
            objectName: "update-copy"
            visible: root.phase === "offer" && root.how === "command"
            anchors.verticalCenter: parent.verticalCenter
            text: root.copied ? "Copied" : "Copy"; icon: root.copied ? "check" : "copy"
            primary: true; small: true
            onClicked: root.copyCommand()
        }
        Button {
            objectName: "update-page"
            visible: root.phase === "offer" && root.how === "page"
            anchors.verticalCenter: parent.verticalCenter
            text: "Release page"; icon: "external-link"; primary: true; small: true
            onClicked: { root.shown = false; frame.openLink(root.url) }
        }
        Button {
            objectName: "update-later"
            visible: root.phase !== "downloading"
            anchors.verticalCenter: parent.verticalCenter
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
