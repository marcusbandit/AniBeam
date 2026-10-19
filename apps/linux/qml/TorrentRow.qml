// One torrent as the torrent client reports it: the name, its state in words, the facts
// in the ledger cut (done of size, speed, time left, peers), a 2 px progress line along
// the foot, and Stop or Start and Remove at the right. Remove opens the removal sheet
// (the torrent goes, and with a tick what it downloaded goes to the trash).
import QtQuick
import com.marcusrosado.AniBeam

Item {
    id: root
    property var torrent: ({})
    signal stop()
    signal start()
    signal removeAsked(Item anchor)

    readonly property var t: torrent || ({})
    readonly property string state: String(t.state || "")
    readonly property bool done: (t.progress || 0) >= 1
    readonly property bool stopped: state.indexOf("paused") === 0 || state.indexOf("stopped") === 0
    readonly property bool erred: state === "error" || state === "missingFiles"
    readonly property string words: {
        var s = root.state
        if (s === "downloading" || s === "forcedDL") return "Downloading"
        if (s === "stalledDL") return "Stalled"
        if (s === "metaDL" || s === "forcedMetaDL") return "Fetching metadata"
        if (s === "queuedDL" || s === "queuedUP") return "Queued"
        if (s === "pausedDL" || s === "stoppedDL") return "Stopped"
        if (s === "pausedUP" || s === "stoppedUP") return "Done"
        if (s === "uploading" || s === "stalledUP" || s === "forcedUP") return "Seeding"
        if (s.indexOf("checking") === 0) return "Checking"
        if (s === "moving") return "Moving"
        if (s === "error") return "Error"
        if (s === "missingFiles") return "Files missing"
        return s
    }
    readonly property string facts: {
        var parts = []
        var size = root.t.size || 0
        if (root.done) parts.push(Fmt.bytes(size))
        else parts.push(Fmt.bytes(Math.round(size * (root.t.progress || 0))) + " of " + Fmt.bytes(size))
        if ((root.t.dlspeed || 0) > 0) parts.push(Fmt.bytes(root.t.dlspeed) + "/s down")
        if ((root.t.upspeed || 0) > 0) parts.push(Fmt.bytes(root.t.upspeed) + "/s up")
        if (!root.done && (root.t.dlspeed || 0) > 0 && root.t.eta > 0 && root.t.eta < 8640000) parts.push(Fmt.countdown(root.t.eta) + " left")
        if (!root.done) parts.push(String(root.t.num_seeds || 0) + " seeds")
        return parts.join(", ")
    }

    width: parent ? parent.width : theme.space(100)
    height: theme.space(13)

    Rectangle { anchors.fill: parent; radius: theme.radiusSm; color: hover.containsMouse ? theme.raised : "transparent" }
    MouseArea { id: hover; anchors.fill: parent; hoverEnabled: true; acceptedButtons: Qt.NoButton }

    Column {
        anchors.left: parent.left
        anchors.leftMargin: theme.space(2)
        anchors.right: controls.left
        anchors.rightMargin: theme.space(4)
        anchors.verticalCenter: parent.verticalCenter
        spacing: theme.space(0.75)
        Row {
            width: parent.width
            spacing: theme.space(3)
            Label {
                width: parent.width - stateLabel.width - parent.spacing
                text: root.t.name || ""
                role: "ui"
                elide: Text.ElideMiddle
                anchors.verticalCenter: parent.verticalCenter
            }
            Label {
                id: stateLabel
                anchors.verticalCenter: parent.verticalCenter
                text: root.words
                role: "ledger"
                color: root.erred ? theme.red : root.done ? theme.caughtUp : root.stopped ? theme.textFaint : theme.accent
            }
        }
        Label {
            width: parent.width
            text: root.facts
            role: "ledger"
            color: theme.textDim
            elide: Text.ElideRight
        }
    }
    Row {
        id: controls
        anchors.right: parent.right
        anchors.rightMargin: theme.space(1.5)
        anchors.verticalCenter: parent.verticalCenter
        spacing: theme.space(1.5)
        Button {
            visible: !root.erred
            text: root.stopped ? "Start" : "Stop"
            icon: root.stopped ? "play" : "pause"
            flat: true
            small: true
            onClicked: root.stopped ? root.start() : root.stop()
        }
        Button {
            id: removeButton
            objectName: "torrent-remove"
            text: "Remove"
            icon: "x"
            flat: true
            small: true
            onClicked: root.removeAsked(removeButton)
        }
    }
    // The progress line along the foot, the hairline under it
    Rectangle {
        anchors.bottom: parent.bottom
        x: theme.space(2)
        width: parent.width - theme.space(4)
        height: 1
        color: theme.line
        opacity: 0.5
    }
    // The bytes as step cubes: purple while it downloads, red when it broke
    EpisodeCubes {
        anchors.bottom: parent.bottom
        anchors.bottomMargin: theme.space(1)
        x: theme.space(2)
        width: parent.width - theme.space(4)
        height: implicitHeight
        visible: !root.done
        codes: theme.cubeSteps(root.t.progress || 0, 40, root.erred ? "e" : root.stopped ? "g" : "t", "g")
        cubeSize: theme.space(1.25)
        minSize: 2
        maxRows: 1
        tipLimit: 0
    }
}
