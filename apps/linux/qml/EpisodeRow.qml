// One episode in a ledger tile: the marker (track to here / untrack to here), the code in
// the ledger cut, the title with "Next up" right beside it, an optional detail line (a stray
// file's name), and the resume point. Everything sits left, next to what it describes; the
// row lifts one step on hover, its corners nested in the tile's. Clicking it plays.
import QtQuick
import com.marcusrosado.AniBeam

Corner {
    id: root
    property var episode: ({})
    property bool extra: false
    property bool hasTracker: false
    property string title: episode.title || ""
    property string detailText: ""
    signal play()
    signal marker()
    width: parent ? parent.width : implicitWidth
    height: Math.max(theme.space(11), words.implicitHeight + theme.space(2))
    radius: theme.radiusInner(theme.space(4))
    smoothing: theme.cornerSmoothing
    color: hover.containsMouse ? theme.tileHover : "transparent"

    property bool watched: !!episode.watched
    readonly property bool nextUp: !!episode.next_up
    readonly property real resumeFraction: episode.resume && episode.resume.duration > 0 ? episode.resume.position / episode.resume.duration : 0

    MouseArea { id: hover; anchors.fill: parent; hoverEnabled: true; cursorShape: Qt.PointingHandCursor; onClicked: root.play() }

    Row {
        id: line
        x: theme.space(2)
        width: parent.width - theme.space(4)
        anchors.verticalCenter: parent.verticalCenter
        spacing: theme.space(3)
        Corner {
            id: mark
            anchors.verticalCenter: parent.verticalCenter
            width: theme.space(6); height: width
            radius: width / 2; smoothing: theme.cornerSmoothing
            color: root.watched ? theme.accentSoft : theme.tileSunken
            borderColor: markHover.containsMouse ? theme.accent : "transparent"; borderWidth: 1
            Icon { anchors.centerIn: parent; glyph: root.watched ? "check" : "play"; size: theme.space(3.5); color: root.watched ? theme.accent : theme.textDim }
            MouseArea { id: markHover; anchors.fill: parent; hoverEnabled: true; enabled: root.hasTracker && !root.extra; cursorShape: Qt.PointingHandCursor; onClicked: root.marker() }
            Tooltip { text: root.hasTracker && !root.extra ? (root.watched ? "Untrack to here" : "Track to here") : "" }
        }
        Label {
            id: code
            anchors.verticalCenter: parent.verticalCenter
            width: theme.space(14)
            text: root.episode.code || ""
            role: "ledger"
            color: root.nextUp ? theme.accent : theme.textDim
            elide: Text.ElideRight
        }
        Column {
            id: words
            anchors.verticalCenter: parent.verticalCenter
            width: parent.width - mark.width - code.width - parent.spacing * 2
            Row {
                width: parent.width
                spacing: theme.space(2)
                Label {
                    id: name
                    anchors.verticalCenter: parent.verticalCenter
                    width: Math.min(implicitWidth, parent.width - (tags.width > 0 ? tags.width + parent.spacing : 0))
                    text: root.title
                    role: "ui"
                    color: root.watched && !root.nextUp ? theme.textDim : theme.text
                    elide: Text.ElideRight
                }
                Row {
                    id: tags
                    anchors.verticalCenter: parent.verticalCenter
                    spacing: theme.space(1.5)
                    Chip { visible: root.nextUp; text: "Next up"; small: true; mono: false; color: theme.accentSoft; textColor: theme.accent }
                    Chip { visible: root.extra; text: "Extra"; small: true; mono: false; color: theme.tone(theme.tile, theme.yellow, 0.2); textColor: theme.yellow }
                    Label {
                        visible: root.resumeFraction > 0 && root.resumeFraction < 0.97
                        anchors.verticalCenter: parent.verticalCenter
                        text: root.episode.resume ? "Resume at " + Fmt.clock(root.episode.resume.position) : ""
                        role: "ledger"
                        color: theme.textDim
                    }
                }
            }
            Label {
                visible: root.detailText !== ""
                width: parent.width
                text: root.detailText
                role: "ledger"
                color: theme.textDim
                elide: Text.ElideMiddle
            }
        }
    }
    // The resume point along the foot of the row
    Corner {
        visible: root.resumeFraction > 0
        x: line.x + mark.width + line.spacing
        width: (parent.width - x - theme.space(2)) * Math.min(1, root.resumeFraction)
        height: theme.space(0.5)
        anchors.bottom: parent.bottom; anchors.bottomMargin: theme.space(0.5)
        radius: height / 2; smoothing: 0
        color: theme.tone(theme.tile, theme.accent, 0.6)
    }
}
