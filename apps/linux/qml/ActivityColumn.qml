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
    signal reconnect()
    // A show's torrents shown as one line: stop, start or cancel them together
    signal controlAll(var hashes, string action)
    signal cancelAll(Item anchor, var torrents)
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
    // The lines of Downloading now: a show with more than one torrent moving reads as one
    // line, the show and its episodes as cubes, the way a batch does; a lone torrent keeps
    // its own line. A group sits where its first torrent sorted.
    readonly property var entries: {
        var out = [], at = {}
        for (var i = 0; i < active.length; i++) {
            var t = active[i]
            var key = showOf(t) ? "s" + t.series_id : "h" + t.hash
            if (at[key] === undefined) { at[key] = out.length; out.push([t]) }
            else out[at[key]].push(t)
        }
        return out.map(function(m) { return m.length === 1 ? m[0] : root.groupOf(m) })
    }
    // The lowest episode a torrent carries, from the page, so the show view orders the same way
    property var firstEpisode: function(t) { return 1e9 }
    // Several torrents of one show as one: summed bytes and rate, the slowest one's time
    // left, and their episodes as files so the line draws them as cubes
    function groupOf(members) {
        members = members.slice().sort(function(a, b) { return root.firstEpisode(a) - root.firstEpisode(b) })
        var size = 0, got = 0, dl = 0, eta = 0, files = [], doneAt = 0
        var allStopped = true, broken = false
        members.forEach(function(t) {
            size += t.size || 0
            got += (t.size || 0) * (t.progress || 0)
            dl += t.dlspeed || 0
            doneAt = Math.max(doneAt, t.completion_on || 0)
            if (root.moving(t) && t.eta > 0 && t.eta < 8640000) eta = Math.max(eta, t.eta)
            if (!root.stopped(t)) allStopped = false
            if (t.state === "error" || t.state === "missingFiles") broken = true
            var eps = root.episodesOf(t)
            if (eps.length) eps.forEach(function(e) { files.push({ episode: e.n, progress: e.p, priority: e.skip ? 0 : 1 }) })
            else if (t.episode !== null && t.episode !== undefined) files.push({ episode: t.episode, progress: t.progress || 0, priority: 1 })
        })
        return { grouped: true, members: members, hash: members[0].hash, name: members[0].name, series_id: members[0].series_id,
                 size: size, progress: size > 0 ? got / size : members.every(function(t) { return (t.progress || 0) >= 1 }) ? 1 : 0, dlspeed: dl, eta: eta, files: files, batch: false, episode: null, completion_on: doneAt,
                 state: broken ? "error" : allStopped ? "stoppedDL" : "downloading" }
    }
    readonly property var finishedToday: {
        var since = new Date(); since.setHours(0, 0, 0, 0)
        var cut = since.getTime() / 1000
        return torrents.filter(function(t) { return (t.progress || 0) >= 1 && (t.completion_on || 0) >= cut })
                       .sort(function(a, b) { return b.completion_on - a.completion_on })
    }
    // Finished today as lines: torrents of one show from one release group that finished
    // within ten minutes of each other read as one, unless another finish came in between
    function groupTag(t) { var m = /^\s*\[([^\]]+)\]/.exec(t.name || ""); return m ? m[1].toLowerCase() : "" }
    readonly property var finishedEntries: {
        var runs = []
        finishedToday.forEach(function(t) {
            var run = runs.length ? runs[runs.length - 1] : null
            var last = run ? run[run.length - 1] : null
            if (last && root.showOf(t) && t.series_id === last.series_id && root.groupTag(t) === root.groupTag(last)
                    && Math.abs((last.completion_on || 0) - (t.completion_on || 0)) <= 600) run.push(t)
            else runs.push([t])
        })
        return runs.map(function(m) { return m.length === 1 ? m[0] : root.groupOf(m) })
    }

    // ---- Today's Downloads lines from the log
    property var lines: []
    function toLine(e) { return { at: e.at, time: Qt.formatTime(new Date(e.at * 1000), words.t("activity-column-time-format")), level: e.level, msg: e.message } }
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
    // The row's two lines: the show (else the torrent's own name), then what of it this is
    function titleOf(t) { var s = showOf(t); return s ? s.title : t.name }
    function kindOf(t) {
        if (t.grouped) {
            // Each episode once, and none the client was told to skip
            var ns = t.files.filter(function(f) { return f.priority !== 0 }).map(function(f) { return f.episode })
                .filter(function(n, i, all) { return all.indexOf(n) === i }).sort(function(a, b) { return a - b })
            var run = ns.length > 1 && ns.every(function(n, i) { return i === 0 || n === ns[i - 1] + 1 })
            return run ? words.t("activity-column-episode-run", { first: ns[0], last: ns[ns.length - 1] }) : ns.length ? words.t("activity-column-episode-list", { list: ns.join(", ") })
                : words.t("activity-column-downloads", { count: t.members.length })
        }
        if (t.batch) return words.t("activity-column-batch")
        if (t.episode !== null && t.episode !== undefined) return words.t("episode-number", { number: t.episode })
        return showOf(t) ? words.t("activity-column-download") : ""
    }
    function nameOf(t) {
        var s = showOf(t)
        if (!s) return t.name
        if (t.batch) return words.t("activity-column-name-batch", { title: s.title })
        if (t.episode !== null && t.episode !== undefined) return words.t("activity-column-name-episode", { title: s.title, number: t.episode })
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
            Label { msg: "activity-column-title"; role: "display"; font.pointSize: theme.typeHeading * 1.2; font.variableAxes: ({ "wdth": 68, "wght": 380 }) }
            Row {
                spacing: theme.space(4)
                visible: root.connected
                Column {
                    Label { text: Fmt.bytes(root.transfer ? root.transfer.dl_speed || 0 : 0) + "/s"; role: "figure"; font.pointSize: theme.typeLarge }
                    Label { msg: "activity-column-down"; role: "ui"; font.pointSize: theme.typeSmall; color: theme.textFaint }
                }
                Column {
                    Label { text: Fmt.bytes(root.transfer ? root.transfer.up_speed || 0 : 0) + "/s"; role: "figure"; font.pointSize: theme.typeLarge; color: theme.textDim }
                    Label { msg: "activity-column-up"; role: "ui"; font.pointSize: theme.typeSmall; color: theme.textFaint }
                }
            }
            Label { visible: !root.connected; width: parent.width; text: words.t("activity-column-not-connected", { client: root.clientName }); role: "ui"; color: theme.textDim; wrapMode: Text.Wrap }
            Button { objectName: "activity-reconnect"; visible: !root.connected; msg: "activity-column-connect-again"; icon: "refresh-cw"; small: true; onClicked: root.reconnect() }

            // ---- Downloading now
            Column {
                visible: root.active.length > 0
                width: parent.width
                spacing: theme.space(1)
                Label { msg: "activity-column-downloading"; role: "title"; font.pointSize: theme.typeNormal; color: theme.textDim; bottomPadding: theme.space(2) }
                // Twelve, then the rest on a click
                property bool all: false
                Repeater {
                    model: parent.all ? root.entries : root.entries.slice(0, 12)
                    TorrentLine { required property var modelData; t: modelData }
                }
                Button { visible: root.entries.length > 12; small: true; flat: true
                    text: parent.all ? words.t("activity-column-fewer") : words.t("activity-column-more", { count: root.entries.length - 12 })
                    onClicked: parent.all = !parent.all }
            }
            Label {
                visible: root.connected && root.active.length === 0
                width: parent.width
                msg: "activity-column-nothing-downloading"
                role: "ui"; color: theme.textDim; wrapMode: Text.Wrap
            }

            // ---- Finished today
            Column {
                visible: root.finishedEntries.length > 0
                width: parent.width
                spacing: theme.space(1)
                Label { msg: "activity-column-finished-today"; role: "title"; font.pointSize: theme.typeNormal; color: theme.textDim; bottomPadding: theme.space(2) }
                property bool all: false
                Repeater {
                    model: parent.all ? root.finishedEntries : root.finishedEntries.slice(0, 8)
                    TorrentLine { required property var modelData; t: modelData }
                }
                Button { visible: root.finishedEntries.length > 8; small: true; flat: true
                    text: parent.all ? words.t("activity-column-fewer") : words.t("activity-column-more", { count: root.finishedEntries.length - 8 })
                    onClicked: parent.all = !parent.all }
            }

            // ---- The log
            Column {
                width: parent.width
                spacing: 0
                Label { msg: "activity-column-today"; role: "title"; font.pointSize: theme.typeNormal; color: theme.textDim; bottomPadding: theme.space(3) }
                Label { visible: root.lines.length === 0; width: parent.width; msg: "activity-column-today-empty"; role: "ui"; color: theme.textFaint; wrapMode: Text.Wrap }
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
        height: Math.max(thumb.height, info.implicitHeight, actions.implicitHeight) + theme.space(4)
        Rectangle { anchors.fill: parent; radius: theme.radiusMd; color: lineMouse.containsMouse ? theme.glassFloat : "transparent" }
        MouseArea { id: lineMouse; anchors.fill: parent; hoverEnabled: true; cursorShape: Qt.PointingHandCursor; onClicked: root.openShow(line.t.grouped ? line.t.members[0] : line.t); Accessible.role: Accessible.Button; Accessible.name: theme.nameNear(parent)
            Accessible.onPressAction: root.openShow(line.t.grouped ? line.t.members[0] : line.t) }
        HoverHandler { id: lineHover }
        // Stop or start, and cancel (the torrent goes, and what it wrote so far, asked
        // first); each spins until the client has done it. A show's group does all of its
        // torrents: Stop takes the ones still moving, Start the stopped ones.
        readonly property var members: t.grouped ? t.members : [t]
        readonly property string waiting: {
            for (var i = 0; i < members.length; i++) { var w = root.busyOf(members[i].hash); if (w !== "") return w }
            return ""
        }
        Column {
            id: actions
            visible: !line.done
            anchors.right: parent.right
            anchors.rightMargin: theme.space(1)
            anchors.top: parent.top
            anchors.topMargin: theme.space(2)
            spacing: theme.space(1.5)
            Button {
                id: toggle
                width: theme.space(12.5)
                objectName: "activity-toggle"
                readonly property bool stopped: root.stopped(line.t)
                icon: stopped ? "play" : "pause"
                primary: stopped && !busy
                busy: line.waiting === "Stop" || line.waiting === "Start"
                onClicked: {
                    if (!line.t.grouped) { root.control(line.t.hash, stopped ? "Start" : "Stop"); return }
                    var want = stopped ? "Start" : "Stop"
                    root.controlAll(line.members.filter(function(m) { return root.stopped(m) === stopped }).map(function(m) { return m.hash }), want)
                }
                Tooltip { text: toggle.busy ? (line.waiting === "Stop" ? words.t("activity-column-stopping") : words.t("activity-column-starting")) : toggle.stopped ? words.t("activity-column-start-tip") : words.t("activity-column-stop-tip") }
            }
            Button {
                id: cancel
                objectName: "activity-cancel"
                width: theme.space(12.5)
                icon: "x"
                danger: true
                busy: line.waiting === "Cancel"
                onClicked: line.t.grouped ? root.cancelAll(cancel, line.members) : root.cancelTorrent(cancel, line.t)
                Tooltip { text: cancel.busy ? words.t("activity-column-cancelling") : line.t.grouped ? words.t("activity-column-cancel-all-tip") : words.t("activity-column-cancel-tip") }
            }
        }
        Corner {
            id: thumb
            x: theme.space(1)
            anchors.top: parent.top
            anchors.topMargin: theme.space(2)
            width: theme.space(11.5); height: width * 1.5
            radius: theme.radiusSm
            smoothing: theme.cornerSmoothing
            color: theme.tileSunken
            Image {
                id: thumbImage
                visible: false
                anchors.fill: parent
                source: line.show && line.show.poster ? Fmt.fileUrl(line.show.poster) : ""
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
            anchors.top: parent.top
            anchors.topMargin: theme.space(1)
            spacing: theme.space(0.75)
            Label { width: parent.width; text: root.titleOf(line.t); role: "ui"; font.pointSize: theme.typeNormal * 1.25; elide: Text.ElideRight }
            Label { visible: text !== ""; width: parent.width; text: root.kindOf(line.t); role: "ui"; font.pointSize: theme.typeSmall * 1.08; elide: Text.ElideRight }
            // Bytes, then time left; the second goes under the first when the row is narrow
            Flow {
                width: parent.width
                spacing: theme.space(2)
                Label { id: sizeText; text: line.done ? Fmt.bytes(line.t.size || 0) : words.t("activity-column-size-of", { got: Fmt.bytes(Math.round((line.t.size || 0) * (line.t.progress || 0))), total: Fmt.bytes(line.t.size || 0) }); role: "figure"; font.pointSize: theme.typeSmall * 0.95; color: theme.textDim }
                // Stopped needs no word: the play button says it
                Label {
                    id: leftText
                    visible: text !== ""
                    text: line.broken ? words.t("activity-column-files-missing") : root.stopped(line.t) && !line.done ? "" : line.done ? Qt.formatTime(new Date(line.t.completion_on * 1000), words.t("activity-column-time-format"))
                        : line.t.eta > 0 && line.t.eta < 8640000 ? words.t("activity-column-left", { time: words.countdown(line.t.eta) }) : line.t.dlspeed > 0 ? Fmt.bytes(line.t.dlspeed) + "/s" : words.t("activity-column-waiting-peers")
                    role: line.broken ? "ui" : "figure"
                    font.pointSize: theme.typeSmall * 0.95
                    color: line.broken ? theme.red : theme.textDim
                }
            }
            // A batch: each episode as its own cube, filling as its file comes in
            Label {
                visible: !line.done && line.eps.length > 1
                text: words.t("activity-column-episodes-done", { done: line.epsDone, count: line.eps.length })
                role: "ui"; font.pointSize: theme.typeSmall * 0.95; color: theme.textDim
            }
            // Six to a row, so a season reads as two short rows
            EpisodeCubes {
                visible: !line.done && line.eps.length > 1
                width: Math.min(parent.width, 6 * (cubeSize + gap) - gap)
                height: implicitHeight
                codes: line.eps.map(function(e) { return e.skip ? "o" : e.p >= 1 ? "d" : "t" })
                fills: line.eps.map(function(e) { return e.p })
                cubeSize: theme.space(5.25)
                minSize: theme.space(2)
                maxRows: 4
                tipFor: function(i, code) { var e = line.eps[i]; return e.skip ? words.t("activity-column-cube-skipped", { number: e.n }) : e.p >= 1 ? words.t("activity-column-cube-done", { number: e.n })
                    : words.t("activity-column-cube-progress", { number: e.n, percent: Math.floor(e.p * 100) }) }
            }
            RollControls {
                width: parent.width
                // A group shows the roll of whichever member belongs to one
                roll: (line.t.grouped ? (line.t.members.filter(function(m) { return !!m.roll })[0] || {}).roll : line.t.roll) || null
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
