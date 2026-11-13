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
    // Named stateText, not words, so the global `words` stays reachable in this file
    readonly property string stateText: {
        var s = root.state
        if (s === "downloading" || s === "forcedDL") return words.t("torrent-row-state-downloading")
        if (s === "stalledDL") return words.t("torrent-row-state-stalled")
        if (s === "metaDL" || s === "forcedMetaDL") return words.t("torrent-row-state-metadata")
        if (s === "queuedDL" || s === "queuedUP") return words.t("torrent-row-state-queued")
        if (s === "pausedDL" || s === "stoppedDL") return words.t("torrent-row-state-stopped")
        if (s === "pausedUP" || s === "stoppedUP") return words.t("torrent-row-state-done")
        if (s === "uploading" || s === "stalledUP" || s === "forcedUP") return words.t("torrent-row-state-seeding")
        if (s.indexOf("checking") === 0) return words.t("torrent-row-state-checking")
        if (s === "moving") return words.t("torrent-row-state-moving")
        if (s === "error") return words.t("torrent-row-state-error")
        if (s === "missingFiles") return words.t("torrent-row-state-files-missing")
        if (s === "allocating") return words.t("torrent-row-state-making-room")
        if (s === "metaDL" || s === "forcedMetaDL") return words.t("torrent-row-state-finding-peers")
        if (s === "forcedDL") return words.t("torrent-row-state-downloading")
        // A state this list does not know yet still reads as a word, never the client's code
        return words.t("torrent-row-state-waiting")
    }
    readonly property string facts: {
        var parts = []
        var size = root.t.size || 0
        if (root.done) parts.push(Fmt.bytes(size))
        else parts.push(words.t("torrent-row-done-of-size", { done: Fmt.bytes(Math.round(size * (root.t.progress || 0))), size: Fmt.bytes(size) }))
        if ((root.t.dlspeed || 0) > 0) parts.push(words.t("torrent-row-speed-down", { speed: Fmt.bytes(root.t.dlspeed) }))
        if ((root.t.upspeed || 0) > 0) parts.push(words.t("torrent-row-speed-up", { speed: Fmt.bytes(root.t.upspeed) }))
        if (!root.done && (root.t.dlspeed || 0) > 0 && root.t.eta > 0 && root.t.eta < 8640000) parts.push(words.t("torrent-row-time-left", { time: words.countdown(root.t.eta) }))
        if (!root.done) parts.push(words.t("torrent-row-seeders", { count: root.t.num_seeds || 0 }))
        if (root.t.roll) parts.push(words.t("torrent-row-rolling", { next: root.t.roll.next, through: root.t.roll.through, total: root.t.roll.total }))
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
                text: root.stateText
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
            text: root.stopped ? words.t("action-start") : words.t("torrent-row-stop")
            icon: root.stopped ? "play" : "pause"
            flat: true
            small: true
            onClicked: root.stopped ? root.start() : root.stop()
        }
        Button {
            id: removeButton
            objectName: "torrent-remove"
            msg: "action-remove"
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
