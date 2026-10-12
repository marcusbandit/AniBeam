// The library's first-run nudge toward a tracker: one flat tile across the top of the page
// while neither tracker is connected, with a Connect for each and Not now. Connect goes
// through frame.openConnect, so with the build's own credentials it is the whole sign-in in
// one click. Not now is kept in the shell's own settings file under the config directory
// (a --root run's sandbox included), and connecting either tracker retires it as well.
import QtQuick
import QtCore
import com.marcusrosado.AniBeam

Corner {
    id: root
    Settings {
        id: kept
        location: "file://" + Shell.configDir + "/shell.conf"
        category: "onboarding"
        property bool trackerNudgeDismissed: false
    }
    readonly property bool anyConnected: (Door.trackers.anilist && Door.trackers.anilist.connected === true)
        || (Door.trackers.mal && Door.trackers.mal.connected === true)
    readonly property bool wanted: !kept.trackerNudgeDismissed && !anyConnected

    visible: wanted
    height: visible ? Math.max(words.implicitHeight, actions.height) + pad * 2 : 0
    radius: theme.radiusTile
    smoothing: theme.cornerSmoothing
    color: theme.tile
    readonly property real pad: theme.space(5)

    Corner {
        id: mark
        x: root.pad
        anchors.verticalCenter: parent.verticalCenter
        width: theme.space(10); height: width
        radius: width / 2
        smoothing: theme.cornerSmoothing
        color: theme.accentSoft
        Icon { anchors.centerIn: parent; glyph: "user-check"; color: theme.accent; size: theme.space(5) }
    }
    Column {
        id: words
        anchors.left: mark.right; anchors.leftMargin: theme.space(4)
        anchors.right: actions.left; anchors.rightMargin: theme.space(6)
        anchors.verticalCenter: parent.verticalCenter
        spacing: theme.space(1)
        Label { width: parent.width; text: "Connect your anime list"; role: "title"; font.pointSize: theme.typeNormal; elide: Text.ElideRight }
        Label { width: parent.width; text: "AniBeam marks episodes on AniList or MyAnimeList as you watch."; color: theme.textDim; font.pointSize: theme.typeSmall; wrapMode: Text.Wrap }
    }
    Row {
        id: actions
        anchors.right: parent.right; anchors.rightMargin: root.pad
        anchors.verticalCenter: parent.verticalCenter
        spacing: theme.space(2)
        Button { objectName: "nudge-anilist"; text: "Connect AniList"; icon: "log-in"; primary: true; onClicked: frame.openConnect("Anilist") }
        Button { objectName: "nudge-mal"; text: "Connect MyAnimeList"; icon: "log-in"; onClicked: frame.openConnect("Mal") }
        Button { objectName: "nudge-dismiss"; text: "Not now"; flat: true; onClicked: kept.trackerNudgeDismissed = true }
    }
}
