// A newer desktop release is out: one quiet island over the page's bottom right corner, the
// version and a Download button that opens the release page in the browser. The frame asks
// Shell once as the app starts and Shell answers through updateAvailable. Later hides it
// until a newer version than this one comes out; the version passed over lives in shell.conf.
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
    }

    Connections {
        target: Shell
        function onUpdateAvailable(version, url) { root.show(version, url) }
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
            text: "Download"; icon: "download"; small: true
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
