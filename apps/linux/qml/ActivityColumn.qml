// The Downloads page's right column: what the torrent client is doing now and what happened
// today, so a click anywhere on the page has somewhere to show its result. The rate in the
// figures face, the torrents still moving (a poster, the show and episode, bytes as step
// cubes, time left), then the Downloads lines of the log, newest first. The torrents come
// from the page's own poll; the log is backfilled once and then follows Door.event.
import QtQuick
import com.marcusrosado.AniBeam

Item {
    id: root
    property var torrents: []
    property var transfer: null
    // Series id to ShowLink, from the page's torrent read
    property var shows: ({})
    property bool connected: false
    property string clientName: "qBittorrent"
    signal openShow(var torrent)
    signal cancelTorrent(Item anchor, var torrent)
    signal control(string hash, string action)
    // What a torrent's buttons wait on ("Stop", "Start", "Cancel", or ""), from the page
    property var busyOf: function(hash) { return "" }

    // Moving or waiting to: not finished, not stopped; broken ones stay so they get seen
    function moving(t) {
        var s = String(t.state || "")
        if (s === "error" || s === "missingFiles") return true
        if (s.indexOf("paused") === 0 || s.indexOf("stopped") === 0) return false
        return (t.progress || 0) < 1
    }
    function stopped(t) { var s = String(t.state || ""); return s.indexOf("paused") === 0 || s.indexOf("stopped") === 0 }
    // Every unfinished torrent: the moving ones first, the stopped ones after, so a stopped
    // download can still be seen and cancelled
    readonly property var active: torrents.filter(function(t) { return root.moving(t) || (root.stopped(t) && (t.progress || 0) < 1) })
        .sort(function(a, b) { return (root.moving(a) ? 0 : 1) - (root.moving(b) ? 0 : 1) || (b.dlspeed || 0) - (a.dlspeed || 0) || (b.added_on || 0) - (a.added_on || 0) })
    readonly property var finishedToday: {
        var since = new Date(); since.setHours(0, 0, 0, 0)
        var cut = since.getTime() / 1000
        return torrents.filter(function(t) { return (t.progress || 0) >= 1 && (t.completion_on || 0) >= cut })
                       .sort(function(a, b) { return b.completion_on - a.completion_on })
    }

    // ---- Today's Downloads lines from the log
    property var lines: []
    function toLine(e) { return { at: e.at, time: Qt.formatTime(new Date(e.at * 1000), "hh:mm"), level: e.level, msg: e.message } }
    function keep(e) {
        if (e.stage !== "Downloads" || e.level === "Debug") return false
        var since = new Date(); since.setHours(0, 0, 0, 0)
        return e.at * 1000 >= since.getTime()
    }
    Component.onCompleted: {
        var r = Door.recentEvents(400)
        if (r.error) return
        lines = r.reply.events.filter(keep).reverse().slice(0, 30).map(toLine)
    }
    Connections {
        target: Door
        function onEvent(envelope) { if (root.keep(envelope)) root.lines = [root.toLine(envelope)].concat(root.lines).slice(0, 30) }
    }

    function showOf(t) { return t.series_id !== null && t.series_id !== undefined ? root.shows[t.series_id] || null : null }
    // A batch read file by file: one cube per episode, done ones on disk, the rest
    // filling as they come; files with no episode number are left out
    function episodesOf(t) {
        var out = []
        var files = t.files || []
        for (var i = 0; i < files.length; i++) {
            var n = files[i].episode
            if (n === null || n === undefined || n !== Math.floor(n) || n < 1) continue
            out.push({ n: n, p: files[i].progress || 0, skip: files[i].priority === 0 })
        }
        return out.sort(function(a, b) { return a.n - b.n })
    }
    function nameOf(t) {
        var s = showOf(t)
        if (!s) return t.name
        if (t.batch) return s.title + ", a batch"
        if (t.episode !== null && t.episode !== undefined) return s.title + ", episode " + t.episode
        return s.title
    }

    PageScroll {
        id: scroll
        anchors.fill: parent
        footInset: theme.space(6)
        Column {
            width: parent.width
            spacing: theme.space(5)
            Item { width: 1; height: theme.space(5) }
            Label { text: "Activity"; role: "display"; font.pointSize: theme.typeHeading * 1.2; font.variableAxes: ({ "wdth": 68, "wght": 380 }) }
            Row {
                spacing: theme.space(4)
                visible: root.connected
                Column {
                    Label { text: Fmt.bytes(root.transfer ? root.transfer.dl_speed || 0 : 0) + "/s"; role: "figure"; font.pointSize: theme.typeLarge }
                    Label { text: "down"; role: "ui"; font.pointSize: theme.typeSmall; color: theme.textFaint }
                }
                Column {
                    Label { text: Fmt.bytes(root.transfer ? root.transfer.up_speed || 0 : 0) + "/s"; role: "figure"; font.pointSize: theme.typeLarge; color: theme.textDim }
                    Label { text: "up"; role: "ui"; font.pointSize: theme.typeSmall; color: theme.textFaint }
                }
            }
            Label { visible: !root.connected; width: parent.width; text: root.clientName + " is not connected, so nothing moves here."; role: "ui"; color: theme.textDim; wrapMode: Text.Wrap }

            // ---- Downloading now
            Column {
                visible: root.active.length > 0
                width: parent.width
                spacing: theme.space(1)
                Label { text: "Downloading now"; role: "title"; font.pointSize: theme.typeNormal; color: theme.textDim; bottomPadding: theme.space(2) }
                Repeater {
                    model: root.active.slice(0, 12)
                    TorrentLine { required property var modelData; t: modelData }
                }
                Label { visible: root.active.length > 12; text: "And " + (root.active.length - 12) + " more"; role: "ui"; color: theme.textFaint }
            }
            Label {
                visible: root.connected && root.active.length === 0
                width: parent.width
                text: "Nothing is downloading."
                role: "ui"; color: theme.textDim; wrapMode: Text.Wrap
            }

            // ---- Finished today
            Column {
                visible: root.finishedToday.length > 0
                width: parent.width
                spacing: theme.space(1)
                Label { text: "Finished today"; role: "title"; font.pointSize: theme.typeNormal; color: theme.textDim; bottomPadding: theme.space(2) }
                Repeater {
                    model: root.finishedToday.slice(0, 8)
                    TorrentLine { required property var modelData; t: modelData }
                }
            }

            // ---- The log
            Column {
                width: parent.width
                spacing: 0
                Label { text: "Today"; role: "title"; font.pointSize: theme.typeNormal; color: theme.textDim; bottomPadding: theme.space(3) }
                Label { visible: root.lines.length === 0; width: parent.width; text: "Searches, downloads and new subscriptions show up here as they happen."; role: "ui"; color: theme.textFaint; wrapMode: Text.Wrap }
                Repeater {
                    model: root.lines
                    Item {
                        required property var modelData
                        required property int index
                        width: parent.width
                        height: logText.implicitHeight + theme.space(4)
                        Rectangle { x: theme.space(0.5); y: theme.space(1.5); width: theme.space(1.75); height: width; radius: width * 0.3; color: modelData.level === "Error" ? theme.red : modelData.level === "Warn" ? theme.yellow : index === 0 ? theme.text : theme.textFaint }
                        Rectangle { visible: index < root.lines.length - 1; x: theme.space(1.25); y: theme.space(4.5); width: 1; height: parent.height - theme.space(4); color: theme.line }
                        Column {
                            id: logText
                            x: theme.space(5)
                            width: parent.width - x
                            Label { text: modelData.time; role: "figure"; font.pointSize: theme.typeSmall * 0.95; color: theme.textFaint }
                            Label { width: parent.width; text: modelData.msg; role: "ui"; color: theme.textDim; wrapMode: Text.Wrap; maximumLineCount: 4; elide: Text.ElideRight }
                        }
                    }
                }
            }
        }
    }

    // One torrent: the show's poster, what it is, bytes as step cubes, time left
    component TorrentLine: Item {
        id: line
        property var t: ({})
        readonly property var show: root.showOf(t)
        readonly property bool broken: t.state === "error" || t.state === "missingFiles"
        readonly property bool done: (t.progress || 0) >= 1
        readonly property var eps: root.episodesOf(t)
        readonly property int epsDone: eps.filter(function(e) { return e.p >= 1 }).length
        width: parent ? parent.width : 0
        height: Math.max(thumb.height, info.implicitHeight) + theme.space(4)
        Rectangle { anchors.fill: parent; radius: theme.radiusMd; color: lineMouse.containsMouse ? theme.glassFloat : "transparent" }
        MouseArea { id: lineMouse; anchors.fill: parent; hoverEnabled: true; cursorShape: Qt.PointingHandCursor; onClicked: root.openShow(line.t) }
        HoverHandler { id: lineHover }
        // Stop or start, and cancel (the torrent goes, and what it wrote so far, asked
        // first); each spins until the client has done it
        readonly property string waiting: root.busyOf(t.hash)
        Column {
            id: actions
            visible: !line.done
            anchors.right: parent.right
            anchors.rightMargin: theme.space(1)
            anchors.verticalCenter: parent.verticalCenter
            spacing: theme.space(1.5)
            Button {
                id: toggle
                objectName: "activity-toggle"
                readonly property bool stopped: root.stopped(line.t)
                icon: stopped ? "play" : "pause"
                primary: stopped && !busy
                busy: line.waiting === "Stop" || line.waiting === "Start"
                onClicked: root.control(line.t.hash, stopped ? "Start" : "Stop")
                Tooltip { text: toggle.busy ? (line.waiting === "Stop" ? "Stopping" : "Starting") : toggle.stopped ? "Start downloading" : "Stop downloading; what it has stays" }
            }
            Button {
                id: cancel
                objectName: "activity-cancel"
                icon: "x"
                danger: true
                busy: line.waiting === "Cancel"
                onClicked: root.cancelTorrent(cancel, line.t)
                Tooltip { text: cancel.busy ? "Cancelling" : "Cancel this download" }
            }
        }
        Corner {
            id: thumb
            x: theme.space(2)
            anchors.verticalCenter: parent.verticalCenter
            width: theme.space(10); height: width * 1.45
            radius: theme.radiusSm
            smoothing: theme.cornerSmoothing
            color: theme.tileSunken
            Image {
                id: thumbImage
                visible: false
                anchors.fill: parent
                source: line.show && line.show.poster ? "file://" + line.show.poster : ""
                sourceSize: Qt.size(80, 116)
                fillMode: Image.PreserveAspectCrop
                asynchronous: true
            }
            fillItem: thumbImage.status === Image.Ready ? thumbImage : null
        }
        Column {
            id: info
            anchors.left: thumb.right
            anchors.leftMargin: theme.space(3)
            anchors.right: actions.visible ? actions.left : parent.right
            anchors.rightMargin: theme.space(2)
            anchors.verticalCenter: parent.verticalCenter
            spacing: theme.space(1.25)
            Label { width: parent.width; text: root.nameOf(line.t); role: "ui"; elide: Text.ElideRight }
            // Bytes, then time left; the second goes under the first when the row is narrow
            Flow {
                width: parent.width
                spacing: theme.space(2)
                Label { id: sizeText; text: line.done ? Fmt.bytes(line.t.size || 0) : Fmt.bytes(Math.round((line.t.size || 0) * (line.t.progress || 0))) + " of " + Fmt.bytes(line.t.size || 0); role: "figure"; font.pointSize: theme.typeSmall * 0.95; color: theme.textDim }
                Label {
                    id: leftText
                    text: line.broken ? "Files missing" : root.stopped(line.t) && !line.done ? "Stopped" : line.done ? Qt.formatTime(new Date(line.t.completion_on * 1000), "hh:mm")
                        : line.t.eta > 0 && line.t.eta < 8640000 ? Fmt.countdown(line.t.eta) + " left" : line.t.dlspeed > 0 ? Fmt.bytes(line.t.dlspeed) + "/s" : "Waiting for peers"
                    role: line.broken ? "ui" : "figure"
                    font.pointSize: theme.typeSmall * 0.95
                    color: line.broken ? theme.red : theme.textDim
                }
            }
            // A batch: each episode as its own cube, filling as its file comes in
            Label {
                visible: !line.done && line.eps.length > 1
                text: line.epsDone + " of " + line.eps.length + " episodes done"
                role: "ui"; font.pointSize: theme.typeSmall * 0.95; color: theme.textDim
            }
            EpisodeCubes {
                visible: !line.done && line.eps.length > 1
                width: parent.width
                height: implicitHeight
                codes: line.eps.map(function(e) { return e.skip ? "o" : e.p >= 1 ? "d" : "t" })
                fills: line.eps.map(function(e) { return e.p })
                cubeSize: theme.space(3.5)
                minSize: theme.space(1.5)
                maxRows: 3
                tipFor: function(i, code) { var e = line.eps[i]; return "Episode " + e.n + ": " + (e.skip ? "skipped" : e.p >= 1 ? "done" : Math.floor(e.p * 100) + "%") }
            }
            EpisodeCubes {
                visible: !line.done && line.eps.length <= 1
                width: parent.width
                height: implicitHeight
                codes: theme.cubeSteps(line.t.progress || 0, 24, line.broken ? "e" : "t", "g")
                cubeSize: theme.space(1.5)
                minSize: 2
                maxRows: 1
                tipLimit: 0
            }
        }
    }
}
