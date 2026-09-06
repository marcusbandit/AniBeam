// One episode: the marker (track to here / untrack to here), the code, the title, the
// pills, and a resume bar under it. Clicking the row opens the player.
import QtQuick

Corner {
    id: root
    property var episode: ({})
    property bool extra: false
    property bool hasTracker: false
    property string title: episode.title || ""
    signal play()
    signal marker()
    width: parent ? parent.width : implicitWidth
    height: theme.space(11)
    radius: theme.radiusSm; smoothing: theme.cornerSmoothing
    color: hover.containsMouse ? theme.surface : "transparent"

    property bool watched: !!episode.watched
    readonly property bool nextUp: !!episode.next_up
    readonly property real resumeFraction: episode.resume && episode.resume.duration > 0 ? episode.resume.position / episode.resume.duration : 0

    Row {
        anchors.left: parent.left; anchors.leftMargin: theme.space(2)
        anchors.right: pills.left; anchors.rightMargin: theme.space(3)
        anchors.verticalCenter: parent.verticalCenter
        spacing: theme.space(3)
        Corner {
            id: mark
            anchors.verticalCenter: parent.verticalCenter
            width: theme.space(6); height: width
            radius: width / 2; smoothing: theme.cornerSmoothing
            color: root.watched ? theme.accentSoft : theme.surfaceSunken
            borderColor: markHover.containsMouse ? theme.accent : theme.line; borderWidth: 1
            Icon { anchors.centerIn: parent; glyph: root.watched ? "check" : "play"; size: theme.space(3.5); color: root.watched ? theme.accent : theme.textDim }
            MouseArea { id: markHover; anchors.fill: parent; hoverEnabled: true; enabled: root.hasTracker && !root.extra; cursorShape: Qt.PointingHandCursor; onClicked: root.marker() }
            Tooltip { text: root.hasTracker && !root.extra ? (root.watched ? "untrack to here" : "track to here") : "" }
        }
        Text { anchors.verticalCenter: parent.verticalCenter; text: episode.code || ""; color: theme.textDim; font.family: theme.fontMono; font.pointSize: theme.typeSmall; width: theme.space(16) }
        Text { anchors.verticalCenter: parent.verticalCenter; text: root.title; color: root.nextUp ? theme.text : (root.watched ? theme.textDim : theme.text); font.family: theme.fontSans; font.pointSize: theme.typeNormal; elide: Text.ElideRight; width: parent.width - mark.width - theme.space(16) - theme.space(6) }
    }
    Row {
        id: pills
        anchors.right: parent.right; anchors.rightMargin: theme.space(3)
        anchors.verticalCenter: parent.verticalCenter
        spacing: theme.space(1.5)
        Chip { visible: root.extra; text: "Extra"; small: true; mono: false; color: theme.tone(theme.bg, theme.yellow, 0.2); textColor: theme.yellow }
        Chip { visible: root.nextUp; text: "Next up"; small: true; mono: false; color: theme.accentSoft; textColor: theme.accent }
        Chip { visible: root.watched && !root.nextUp; text: "Watched"; small: true; mono: false; color: theme.surface; textColor: theme.textDim }
    }
    Corner {
        visible: root.resumeFraction > 0 && hover.containsMouse
        x: theme.space(11); width: (parent.width - theme.space(14)) * root.resumeFraction; height: theme.space(0.5)
        anchors.bottom: parent.bottom; anchors.bottomMargin: theme.space(1)
        radius: height / 2; smoothing: theme.cornerSmoothing; color: theme.accent
    }
    MouseArea { id: hover; anchors.fill: parent; hoverEnabled: true; cursorShape: Qt.PointingHandCursor; z: -1; onClicked: root.play() }
}
