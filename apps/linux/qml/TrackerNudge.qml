// The library's first-run nudge toward a tracker: one tile across the full width of the
// wall, set under the header tiles as a stone of its own, while neither tracker is connected, with a Connect for each and Not now. Connect goes
// through frame.openConnect, so with the build's own credentials it is the whole sign-in in
// one click. Not now is kept in the shell's own settings file under the config directory
// (a --root run's sandbox included), and connecting either tracker retires it as well.
import QtQuick
import QtCore
import com.marcusrosado.AniBeam

Tile {
    id: root
    Settings {
        id: kept
        location: Shell.settingsUrl
        category: "onboarding"
        property bool trackerNudgeDismissed: false
    }
    readonly property bool anyConnected: (Door.trackers.anilist && Door.trackers.anilist.connected === true)
        || (Door.trackers.mal && Door.trackers.mal.connected === true)
    readonly property bool wanted: !kept.trackerNudgeDismissed && !anyConnected

    visible: wanted
    // What the content needs; the page cuts the tile to the module from this.
    implicitHeight: Math.max(words.implicitHeight, actions.height, mark.height) + padding * 2

    Corner {
        id: mark
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
        anchors.right: parent.right
        anchors.verticalCenter: parent.verticalCenter
        spacing: theme.space(2)
        ConnectButton { objectName: "nudge-anilist"; tracker: "Anilist"; text: "Connect AniList"; icon: "log-in"; primary: true; onClicked: frame.openConnect("Anilist") }
        ConnectButton { objectName: "nudge-mal"; tracker: "Mal"; text: "Connect MyAnimeList"; icon: "log-in"; onClicked: frame.openConnect("Mal") }
        Button { objectName: "nudge-dismiss"; text: "Not now"; flat: true; onClicked: kept.trackerNudgeDismissed = true }
    }
}
