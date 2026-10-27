// A newer desktop release is out: one quiet island over the page's bottom right corner, the
// version and a Download button that opens the release page in the browser. The frame asks
// Shell once as the app starts and Shell answers through updateAvailable. Later hides it
// until a newer version than this one comes out; the version passed over lives in shell.conf.
import QtQuick
import QtCore
import com.marcusrosado.AniBeam

Frost {
    id: root
    objectName: "update-notice"
    visible: false
    radius: theme.radiusTile
    width: row.implicitWidth + theme.space(6); height: theme.controlHeight + theme.space(2)

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
        visible = true
    }
    function later() { kept.skipped = root.version; visible = false }

    Row {
        id: row
        anchors.centerIn: parent
        spacing: theme.space(3)
        Icon { anchors.verticalCenter: parent.verticalCenter; glyph: "download"; color: theme.accent; size: theme.space(4.5) }
        Label {
            anchors.verticalCenter: parent.verticalCenter
            role: "ui"
            text: "AniBeam " + root.version + " is available for download"
        }
        Button {
            objectName: "update-download"
            anchors.verticalCenter: parent.verticalCenter
            text: "Download"; primary: true; small: true
            onClicked: Shell.openUrl(root.url)
        }
        Button {
            objectName: "update-later"
            anchors.verticalCenter: parent.verticalCenter
            text: "Later"; flat: true; small: true
            onClicked: root.later()
        }
    }
}
