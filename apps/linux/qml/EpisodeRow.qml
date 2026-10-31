// One episode as a flat row: its cube (the episode cubes' colour for its state, and a click
// that tracks or untracks to here), the code
// in the ledger cut, the title with "Next up" or the resume point beside it, an optional
// detail line (a stray file's name), the air date at the right, and a thin resume bar along
// the foot of a half-watched row. The next episode up sits on a filled row; the rest are
// bare, lifting to the tile fill on hover. Clicking the row plays. An episode a torrent is
// still writing reads "Downloading 34%" in the downloading purple with a thin bar along its
// foot, and does nothing when clicked: it is on disk but not whole, so it never plays. An
// absent episode (aired, no file) names its state beside the title and never plays either,
// and nor does one still to come, which shows its date, or its predicted date, at the right.
// With `gettable` set, an absent row reads "Get episode" on hover and a click asks for it
// (#169). While the page picks files to delete, the cube gives way to the phone's circle,
// filled red once picked, and a click on the row or the circle picks (#168).
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
    // Picking files to delete: a click picks this one instead of playing it.
    property bool picking: false
    property bool picked: false
    signal pick()
    // An absent episode that a click goes and gets (the Downloads show view)
    property bool gettable: false
    signal download()
    readonly property bool getting: gettable && absent && !picking
    // A Tab stop: Return plays (or picks, or gets), as a click does
    activeFocusOnTab: playable || getting
    function activate() {
        if (root.playable) { if (root.picking) root.pick(); else root.play() }
        else if (root.getting) root.download()
    }
    Keys.onReturnPressed: activate()
    Keys.onEnterPressed: activate()
    borderColor: activeFocus ? theme.focusRing : "transparent"
    borderWidth: activeFocus ? 2 : 0
    width: parent ? parent.width : implicitWidth
    height: Math.max(theme.space(10), words.implicitHeight + theme.space(3))
    radius: theme.radiusMd
    smoothing: theme.cornerSmoothing
    color: picked ? theme.redSoft
         : hover.pressed && (playable || getting) && !picking ? theme.surfacePressed
         : picking ? (hover.containsMouse && playable ? theme.tile : "transparent")
         : getting && hover.containsMouse ? theme.tile
         : nextUp ? (hover.containsMouse ? theme.raised : theme.tile) : hover.containsMouse && playable ? theme.tile : "transparent"
    opacity: picking && !playable ? 0.45 : 1

    property bool watched: !!episode.watched
    // Aired with no file: missing or not downloaded, nothing to play
    readonly property bool absent: !!episode.absent
    // Not aired yet: scheduled, or a predicted date
    readonly property bool upcoming: !!episode.upcoming
    readonly property bool predicted: upcoming && !!episode.predicted
    // The cube's code: the core's, unless a track or untrack is waiting on the core
    readonly property string cube: {
        if (downloading) return "t"
        var base = episode.cube || (episode.watched ? "l" : "d")
        if (watched === !!episode.watched) return base
        if (watched) return hasTracker ? "w" : "l"
        // An untrack leaves a watch made here standing
        return absent || upcoming || base === "l" ? base : "d"
    }
    readonly property var cubeState: theme.cubeState(cube)
    Component.onDestruction: frame.hoverEpisode(root, false)
    readonly property bool playable: !downloading && !absent && !upcoming
    // With a tracker, any aired episode marks there; without one, an episode on disk marks
    // as watched here
    readonly property bool trackable: !extra && !upcoming && (hasTracker || !absent)
    readonly property bool downloading: episode.downloading !== undefined && episode.downloading !== null
    readonly property bool nextUp: !!episode.next_up && !downloading
    readonly property real downloadFraction: downloading ? Math.max(0, Math.min(1, episode.downloading)) : 0
    readonly property color downloadColor: theme.cubeState("t").fill
    readonly property real resumeFraction: episode.resume && episode.resume.duration > 0 ? episode.resume.position / episode.resume.duration : 0
    readonly property string dateText: !episode.air_date ? ""
        : upcoming && !predicted ? Qt.formatDateTime(new Date(episode.air_date * 1000), "ddd d MMM, HH:mm")
        : upcoming ? Qt.formatDate(new Date(episode.air_date * 1000), "ddd d MMM")
        : Qt.formatDate(new Date(episode.air_date * 1000), "d MMM yyyy")

    MouseArea {
        id: hover
        anchors.fill: parent
        hoverEnabled: true
        cursorShape: root.playable || root.getting ? Qt.PointingHandCursor : Qt.ArrowCursor
        // The frame's right-click menu reads the row under the pointer for its episode rows
        onContainsMouseChanged: frame.hoverEpisode(root, containsMouse)
        onClicked: root.activate()
    }

    Row {
        id: line
        x: theme.space(3)
        width: parent.width - theme.space(6)
        anchors.verticalCenter: parent.verticalCenter
        spacing: theme.space(3)
        // The cube: the episode cubes' square in its state's colour, lifting on hover
        Item {
            id: mark
            anchors.verticalCenter: parent.verticalCenter
            width: theme.space(5); height: width
            // While picking, the phone's circle: filled red once picked
            Rectangle {
                objectName: "episode-pick"
                visible: root.picking && root.playable
                anchors.centerIn: parent
                width: theme.space(4.5); height: width
                radius: width / 2
                color: root.picked ? theme.red : "transparent"
                border.width: root.picked ? 0 : 1.5
                border.color: markHover.containsMouse || hover.containsMouse ? theme.textDim : theme.lineStrong
                Icon { visible: root.picked; anchors.centerIn: parent; glyph: "check"; size: theme.space(3); color: theme.contrast(theme.red, Qt.rgba(1, 1, 1, 1)) >= 4.5 ? "#ffffff" : "#0d0e11" }
            }
            Rectangle {
                objectName: "episode-cube"
                visible: !(root.picking && root.playable)
                anchors.centerIn: parent
                width: theme.space(3.5); height: width
                radius: Math.max(1, width * 0.3)
                color: root.cubeState.fill
                border.width: markHover.containsMouse ? 1 : root.cubeState.edge === "transparent" ? 0 : 1
                border.color: markHover.containsMouse ? theme.accent : root.cubeState.edge
                scale: theme.reducedMotion ? 1 : (markHover.containsMouse ? 1.18 : 1)
                Behavior on scale { enabled: !theme.reducedMotion; NumberAnimation { duration: theme.motionFast } }
                Behavior on color { enabled: !theme.reducedMotion; ColorAnimation { duration: theme.motionFast } }
            }
            MouseArea {
                id: markHover
                anchors.fill: parent; anchors.margins: -theme.space(1)
                hoverEnabled: true
                cursorShape: root.trackable || (root.picking && root.playable) ? Qt.PointingHandCursor : Qt.ArrowCursor
                onClicked: {
                    if (root.picking) { if (root.playable) root.pick() }
                    else if (root.trackable) root.marker()
                }
            }
            Tooltip { text: root.picking ? "" : root.cubeState.label + (root.trackable ? (root.watched ? ". Click to mark unwatched from here" : ". Click to mark watched up to here") : "") }
        }
        Label {
            id: code
            anchors.verticalCenter: parent.verticalCenter
            width: theme.space(12)
            text: root.episode.code || ""
            role: "ledger"
            color: root.nextUp ? theme.accent : theme.textDim
            elide: Text.ElideRight
        }
        Column {
            id: words
            anchors.verticalCenter: parent.verticalCenter
            width: parent.width - mark.width - code.width - (date.visible ? date.width + parent.spacing : 0) - parent.spacing * 2
            Row {
                width: parent.width
                spacing: theme.space(2)
                Label {
                    id: name
                    anchors.verticalCenter: parent.verticalCenter
                    width: Math.min(implicitWidth, parent.width - (tags.width > 0 ? tags.width + parent.spacing : 0))
                    text: root.title
                    role: "ui"
                    font.weight: root.nextUp ? 600 : theme.roleWeight("ui")
                    font.variableAxes: ({ "wdth": 100, "wght": root.nextUp ? 600 : theme.roleWeight("ui") })
                    color: root.absent || root.upcoming ? theme.textFaint : root.downloading || (root.watched && !root.nextUp) ? theme.textDim : theme.text
                    elide: Text.ElideRight
                }
                Row {
                    id: tags
                    anchors.verticalCenter: parent.verticalCenter
                    spacing: theme.space(2)
                    Label { visible: root.nextUp; anchors.verticalCenter: parent.verticalCenter; text: "Next up"; role: "ledger"; color: theme.accent }
                    Label {
                        objectName: "episode-downloading"
                        visible: root.downloading
                        anchors.verticalCenter: parent.verticalCenter
                        text: "Downloading " + Math.floor(root.downloadFraction * 100) + "%"
                        role: "ledger"
                        color: root.downloadColor
                    }
                    Label {
                        objectName: "episode-absent"
                        visible: root.absent && (!root.watched || (root.getting && hover.containsMouse))
                        anchors.verticalCenter: parent.verticalCenter
                        text: root.getting && hover.containsMouse ? "Get episode" : root.cubeState.label
                        role: "ledger"
                        color: root.getting && hover.containsMouse ? theme.accent : root.cube === "m" ? root.cubeState.fill : theme.textFaint
                    }
                    Label {
                        objectName: "episode-predicted"
                        visible: root.predicted
                        anchors.verticalCenter: parent.verticalCenter
                        text: "Predicted"
                        role: "ledger"
                        color: theme.textDim
                    }
                    Label { visible: root.extra; anchors.verticalCenter: parent.verticalCenter; text: "Extra"; role: "ledger"; color: theme.textFaint }
                    Label {
                        visible: root.resumeFraction > 0 && root.resumeFraction < 0.97 && !root.downloading
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
        Label {
            id: date
            visible: root.dateText !== ""
            anchors.verticalCenter: parent.verticalCenter
            text: root.dateText
            role: "ledger"
            color: theme.textFaint
        }
    }
    // The download along the foot of the row, in the downloading purple over a faint track
    Rectangle {
        visible: root.downloading
        x: line.x + mark.width + line.spacing
        width: parent.width - x - theme.space(3)
        height: theme.space(0.5)
        anchors.bottom: parent.bottom; anchors.bottomMargin: theme.space(1)
        radius: height / 2
        color: Qt.rgba(root.downloadColor.r, root.downloadColor.g, root.downloadColor.b, 0.22)
        Rectangle {
            objectName: "episode-download-bar"
            width: parent.width * root.downloadFraction
            height: parent.height
            radius: height / 2
            color: root.downloadColor
        }
    }
    // The resume point along the foot of the row
    Rectangle {
        visible: root.resumeFraction > 0 && !root.downloading
        x: line.x + mark.width + line.spacing
        width: (parent.width - x - theme.space(3)) * Math.min(1, root.resumeFraction)
        height: theme.space(0.5)
        anchors.bottom: parent.bottom; anchors.bottomMargin: theme.space(1)
        radius: height / 2
        color: theme.accent
        opacity: 0.7
    }
}
