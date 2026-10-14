// One torrent as the torrent client reports it: the name, its state in words, the facts
// in the ledger cut (done of size, speed, time left, peers), a 2 px progress line along
// the foot, and Stop or Start and Remove at the right. Remove turns the row's controls
// into an inline question with two red answers, torrent only or files as well, and Keep.
import QtQuick
import com.marcusrosado.AniBeam

Item {
    id: root
    property var torrent: ({})
    signal stop()
    signal start()
    signal remove(bool files)

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
    property bool asking: false

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
    // The controls, or the question in their place
    Item {
        id: controls
        anchors.right: parent.right
        anchors.rightMargin: theme.space(1.5)
        anchors.verticalCenter: parent.verticalCenter
        width: root.asking ? ask.width : buttons.width
        height: theme.controlHeight
        Row {
            id: buttons
            visible: !root.asking
            anchors.right: parent.right
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
                text: "Remove"
                icon: "trash-2"
                flat: true
                small: true
                onClicked: root.asking = true
            }
        }
        Row {
            id: ask
            visible: root.asking
            anchors.right: parent.right
            anchors.verticalCenter: parent.verticalCenter
            spacing: theme.space(2)
            function sync() { if (visible) frame.escapeStack.push("confirm", ask); else frame.escapeStack.pop(ask) }
            function close() { root.asking = false }
            onVisibleChanged: sync()
            Component.onDestruction: frame.escapeStack.pop(ask)
            Label { anchors.verticalCenter: parent.verticalCenter; text: "Remove this torrent?"; role: "ui" }
            Button { text: "Keep files"; icon: "trash-2"; danger: true; small: true; onClicked: { root.asking = false; root.remove(false) } }
            Button { text: "Delete files"; icon: "trash"; danger: true; small: true; onClicked: { root.asking = false; root.remove(true) } }
            Button { text: "Keep"; flat: true; small: true; onClicked: root.asking = false }
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
    Rectangle {
        anchors.bottom: parent.bottom
        anchors.bottomMargin: 1
        x: theme.space(2)
        width: (parent.width - theme.space(4)) * Math.max(0, Math.min(1, root.t.progress || 0))
        height: 2
        color: root.erred ? theme.red : root.done ? theme.caughtUp : root.stopped ? theme.textFaint : theme.accent
        opacity: root.done ? 0.5 : 1
        Behavior on width { enabled: !theme.reducedMotion; NumberAnimation { duration: theme.motionNormal } }
    }
}
