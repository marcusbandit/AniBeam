// The activity drawer: rises from the status strip with exponential smoothing. A header
// with its filters and the Copy, Clear and Close actions, then the core's event stream at
// Info and above, newest first, with runs of identical lines folded into one row that
// expands on click. A person sees the plain lines only, the ones the core worded for them,
// filtered to everything or just problems (#287). With developer options on (`technical`)
// it shows every line in the developer's words, with the stage and level filters. Signal-only, like the log it shows: the drawer adds no
// chatter of its own, it only renders what the core already reported.
import QtQuick
import QtQuick.Controls.Basic as QC
import com.marcusrosado.AniBeam

Item {
    id: root
    property bool open: false
    property real maxHeight: theme.space(100)
    property var entries: []
    property bool backfilled: false
    property bool technical: false
    property bool problemsOnly: false
    signal closed()

    readonly property var stages: ["Library", "Metadata", "Trackers", "Franchise", "Playback", "Store", "System", "Downloads"]
    readonly property var levels: ["Info", "Warn", "Error"]
    property var stageOn: ({})
    property var levelOn: ({})
    property var expanded: ({})

    property real openness: 0
    readonly property real target: open ? 1 : 0
    FrameAnimation {
        running: Math.abs(root.openness - root.target) > 0.001
        onTriggered: root.openness += (root.target - root.openness) * (1 - Math.exp(-14 * frameTime))
    }
    height: Math.round(maxHeight * openness)
    visible: openness > 0.001
    clip: true

    function backfill() {
        var r = Door.recentEvents(2000)
        if (r.error) return
        entries = r.reply.events.slice().reverse().map(toEntry)
        backfilled = true
    }
    function toEntry(e) { return { time: Qt.formatTime(new Date(e.at * 1000), "hh:mm:ss"), stage: e.stage, level: e.level, msg: e.message, plain: e.plain || "", seq: e.seq } }
    Connections {
        target: Door
        function onEvent(envelope) {
            if (envelope.level === "Debug") return
            if (!root.backfilled) return
            root.entries = [root.toEntry(envelope)].concat(root.entries).slice(0, 2000)
        }
    }

    function toggle() { open ? close() : show() }
    function show() { if (!backfilled) backfill(); open = true; forceActiveFocus(); Door.markLogSeen(); frame.escapeStack.push("drawer", root) }
    function close() { if (!open) return; open = false; frame.escapeStack.pop(root); closed() }
    Component.onDestruction: root.close()

    // The drawer is not rebuilt on navigation, unlike a page's own popovers, so it survives
    // a page switch on its own; Frame's reclaimFrameOwned() keeps its escape-stack entry
    // through the clear on every navigation, so Escape still reaches it after a page
    // change. Nothing here needs to react to navigation. On close, Frame hands key focus
    // back to the page (onClosed), since show() took it.

    function anyOn(o) { for (var k in o) if (o[k]) return true; return false }
    function flip(o, k) { var n = {}; for (var j in o) n[j] = o[j]; n[k] = !n[k]; return n }
    function levelColor(l) { return l === "Error" ? theme.red : l === "Warn" ? theme.yellow : theme.text }

    // Filter, then fold runs of identical consecutive lines; an expanded run is laid out again.
    // A run's key is the seq of its OLDEST member, not its position in entries and not the seq
    // it was first created with. entries is newest first, so the loop meets a run's newest
    // member first and its oldest member last; every live event is prepended (see onEvent
    // above), which only ever adds a new newest member in front of an existing run, so keying
    // on the newest member (or on the array index, the first attempt at this fix) still changes
    // under growth and re-collapses an already-expanded row. Keying on the oldest member, kept
    // up to date on every merge below, never changes as the run grows at the top: seq is the
    // core's own monotonic id for the event and is never reused or altered under it.
    readonly property var rows: {
        var sAny = anyOn(stageOn), lAny = anyOn(levelOn)
        var out = []
        var run = null
        for (var i = 0; i < entries.length; i++) {
            var e = entries[i]
            if (!technical) {
                if (e.plain === "") continue
                if (problemsOnly && e.level === "Info") continue
                e = { time: e.time, stage: "", level: e.level, msg: e.plain, seq: e.seq }
            } else {
                if (sAny && !stageOn[e.stage]) continue
                if (lAny && !levelOn[e.level]) continue
            }
            if (run && run.stage === e.stage && run.level === e.level && run.msg === e.msg) {
                run.count += 1
                run.items.push(e)
                run.key = e.seq
                continue
            }
            run = { time: e.time, stage: e.stage, level: e.level, msg: e.msg, count: 1, key: e.seq, items: [e] }
            out.push(run)
        }
        var flat = []
        out.forEach(function(r) {
            if (r.count > 1 && expanded[r.key]) r.items.forEach(function(e) { flat.push({ time: e.time, stage: e.stage, level: e.level, msg: e.msg, count: 1, key: -1 }) })
            else flat.push(r)
        })
        return flat
    }

    // The rows on screen, one line each, a folded run carrying its count.
    function copyAll() {
        var lines = rows.map(function(r) { return r.time + "  " + (r.stage ? r.stage + "  " + r.level + "  " : "") + r.msg + (r.count > 1 ? "  ×" + r.count : "") })
        clipboard.text = lines.join("\n")
        clipboard.selectAll()
        clipboard.copy()
    }
    TextEdit { id: clipboard; visible: false }

    Item {
        id: body
        width: parent.width
        height: root.maxHeight
        anchors.bottom: parent.bottom

        Frost { anchors.fill: parent; radius: theme.radiusTile }
        // Under the header and the list (declared before them), so they keep their buttons
        // and WheelStep; the gaps between them no longer pass a right-click, a wheel or a
        // hover to the page beneath.
        Blocker { anchors.fill: parent; scrim: false }
        Rectangle { anchors.top: parent.top; width: parent.width; height: 1; color: theme.line }

        Item {
            id: header
            anchors.top: parent.top
            anchors.left: parent.left
            anchors.right: parent.right
            anchors.leftMargin: theme.space(4)
            anchors.rightMargin: theme.space(4)
            height: theme.space(11)

            Row {
                id: left
                anchors.left: parent.left
                anchors.right: actions.left
                anchors.rightMargin: theme.space(4)
                anchors.verticalCenter: parent.verticalCenter
                spacing: theme.space(2)
                Icon {
                    anchors.verticalCenter: parent.verticalCenter
                    glyph: "activity"
                }
                Text {
                    anchors.verticalCenter: parent.verticalCenter
                    text: "Activity"
                    color: theme.text
                    font.family: theme.fontSans
                    font.pointSize: theme.typeNormal
                    font.weight: Font.Bold
                }
                Item { width: theme.space(2); height: 1 }
                Seg {
                    visible: !root.technical
                    anchors.verticalCenter: parent.verticalCenter
                    small: true
                    options: [{ text: "Everything" }, { text: "Problems" }]
                    index: root.problemsOnly ? 1 : 0
                    onPicked: function(i) { root.problemsOnly = i === 1 }
                }
                Repeater {
                    model: root.technical ? root.stages : []
                    Chip {
                        required property string modelData
                        anchors.verticalCenter: parent.verticalCenter
                        text: modelData
                        small: true
                        clickable: true
                        selected: root.stageOn[modelData] === true
                        color: selected ? theme.accentSoft : theme.tileSunken
                        textColor: theme.textDim
                        onClicked: root.stageOn = root.flip(root.stageOn, modelData)
                    }
                }
                Item { visible: root.technical; width: theme.space(2); height: 1 }
                Repeater {
                    model: root.technical ? root.levels : []
                    Chip {
                        required property string modelData
                        anchors.verticalCenter: parent.verticalCenter
                        text: modelData === "Warn" ? "Warning" : modelData
                        small: true
                        clickable: true
                        selected: root.levelOn[modelData] === true
                        color: selected ? theme.accentSoft : theme.tileSunken
                        textColor: modelData === "Info" ? theme.textDim : root.levelColor(modelData)
                        onClicked: root.levelOn = root.flip(root.levelOn, modelData)
                    }
                }
            }
            Row {
                id: actions
                anchors.right: parent.right
                anchors.verticalCenter: parent.verticalCenter
                spacing: theme.space(1)
                Button { text: "Copy"; icon: "copy"; flat: true; small: true; onClicked: root.copyAll() }
                Button { text: "Clear"; icon: "trash-2"; flat: true; small: true; onClicked: { Door.clearEvents(); root.entries = []; root.expanded = ({}) } }
                Button { text: "Close"; icon: "x"; flat: true; small: true; onClicked: root.close() }
            }
        }
        Rectangle { anchors.top: header.bottom; width: parent.width; height: 1; color: theme.line }

        ListView {
            id: list
            anchors.top: header.bottom
            anchors.topMargin: 1
            anchors.bottom: parent.bottom
            anchors.left: parent.left
            anchors.right: parent.right
            anchors.leftMargin: theme.space(4)
            anchors.rightMargin: theme.space(4)
            clip: true
            boundsBehavior: Flickable.StopAtBounds
            model: root.rows
            WheelStep { flick: list }
            delegate: Item {
                required property var modelData
                width: list.width
                height: theme.space(7)
                Row {
                    id: line
                    anchors.fill: parent
                    spacing: theme.space(3)
                    Text {
                        id: stamp
                        anchors.verticalCenter: parent.verticalCenter
                        text: modelData.time
                        color: theme.textFaint
                        font.family: theme.fontUi
                        font.pointSize: theme.typeSmall
                    }
                    Chip {
                        id: stageChip
                        visible: modelData.stage !== ""
                        anchors.verticalCenter: parent.verticalCenter
                        text: modelData.stage
                        small: true
                        color: theme.tileSunken
                        textColor: theme.textDim
                        width: theme.space(16)
                    }
                    Text {
                        anchors.verticalCenter: parent.verticalCenter
                        width: Math.min(implicitWidth, line.width - stamp.width - (stageChip.visible ? stageChip.width + line.spacing : 0) - line.spacing - (fold.visible ? fold.width + line.spacing : 0))
                        text: modelData.msg
                        color: root.levelColor(modelData.level)
                        elide: Text.ElideRight
                        font.family: theme.fontSans
                        font.pointSize: theme.typeSmall
                    }
                    Chip {
                        id: fold
                        anchors.verticalCenter: parent.verticalCenter
                        visible: modelData.count > 1
                        text: "×" + modelData.count
                        small: true
                        clickable: true
                        color: theme.tileSunken
                        textColor: theme.textDim
                        onClicked: { var n = {}; for (var k in root.expanded) n[k] = root.expanded[k]; n[modelData.key] = true; root.expanded = n }
                    }
                }
                Rectangle { anchors.bottom: parent.bottom; width: parent.width; height: 1; color: theme.tileSunken }
            }
            QC.ScrollBar.vertical: ThinScrollBar {}
        }
        Label {
            visible: root.rows.length === 0
            anchors.top: header.bottom; anchors.topMargin: theme.space(4)
            anchors.left: parent.left; anchors.leftMargin: theme.space(4)
            text: root.technical ? "Nothing logged yet." : root.problemsOnly ? "No problems. Everything worked." : "Nothing to report yet. What AniBeam does for you shows up here."
            color: theme.textDim
        }
    }
}
