// A newer desktop build is out: one quiet island over the page's bottom right corner, the
// version and a Download button that opens the release page in the browser. The frame asks
// once as the app starts (check()), Settings asks again when the track changes, and Shell
// answers through updateAvailable. The track is Stable or Nightly, and until someone picks
// one it is the running build's own; a nightly's sticks. Later hides it until a newer
// version than this one comes out; the version passed over and the track live in shell.conf.
// It steps aside while the player has the window or a sheet is open (`covered`), and comes
// back once they close.
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
    function check() {
        // A nightly keeps its track once picked by default, so the stable release it leads
        // to does not quietly move it to Stable when installed
        if (kept.track === "" && Shell.ownTrack === "nightly") kept.track = "nightly"
        Shell.checkForUpdate(track)
    }
    function setTrack(t) {
        kept.track = t
        shown = false
        check()
    }

    Connections {
        target: Shell
        // An answer for a track since switched away from is dropped
        function onUpdateAvailable(version, url, track) { if (track === root.track) root.show(version, url) }
    }

    function show(version, url) {
        if (version === kept.skipped) return
        root.version = version
        root.url = url
        shown = true
    }
    function later() { kept.skipped = root.version; shown = false }

    Row {
        id: row
        anchors.centerIn: parent
        spacing: theme.space(3)
        Label {
            anchors.verticalCenter: parent.verticalCenter
            role: "ui"
            text: "AniBeam " + root.version + " is available for download"
        }
        Button {
            objectName: "update-download"
            anchors.verticalCenter: parent.verticalCenter
            text: "Download"; icon: "download"; primary: true; small: true
            onClicked: { root.shown = false; frame.openLink(root.url) }
        }
        Button {
            objectName: "update-later"
            anchors.verticalCenter: parent.verticalCenter
            text: "Later"; icon: "clock"; small: true
            onClicked: root.later()
        }
    }
}
