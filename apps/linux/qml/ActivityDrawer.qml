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
    // What each stage and level chip reads, by message ID
    readonly property var stageWords: ({ Library: "activity-drawer-stage-library", Metadata: "activity-drawer-stage-metadata", Trackers: "activity-drawer-stage-trackers",
                                         Franchise: "activity-drawer-stage-franchise", Playback: "activity-drawer-stage-playback", Store: "activity-drawer-stage-store",
                                         System: "activity-drawer-stage-system", Downloads: "activity-drawer-stage-downloads" })
    readonly property var levelWords: ({ Info: "activity-drawer-level-info", Warn: "activity-drawer-level-warn", Error: "activity-drawer-level-error" })
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
    // Today's lines say the time; older ones say the day too, so yesterday's error does not
    // read as a fresh one
    function stampOf(at) {
        var d = new Date(at * 1000), now = new Date()
        return d.toDateString() === now.toDateString() ? Qt.formatTime(d, words.t("activity-drawer-time-format")) : Qt.formatDateTime(d, words.t("activity-drawer-stamp-format"))
    }
    // `plainWords` is the plain line as a message ID and its arguments (#792), rendered when
    // the row is drawn so a language switch reaches lines already in the log; `plain` is the
    // line as the core first said it, for lines from before the core kept their IDs
    function toEntry(e) { return { time: stampOf(e.at), stage: e.stage, level: e.level, msg: e.message, plain: e.plain || "", plainWords: e.plain_words || null, seq: e.seq } }
    function plainOf(e) { return e.plainWords ? words.t(e.plainWords.id, e.plainWords.args) : e.plain }
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
                var said = plainOf(e)
                if (said === "") continue
                if (problemsOnly && e.level === "Info") continue
                e = { time: e.time, stage: "", level: e.level, msg: said, seq: e.seq }
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
            // An unfolded run keeps a fold chip on its first line, to fold it back up
            if (r.count > 1 && expanded[r.key]) r.items.forEach(function(e, n) { flat.push({ time: e.time, stage: e.stage, level: e.level, msg: e.msg, count: 1, key: -1, foldKey: n === 0 ? r.key : -1 }) })
            else flat.push(r)
        })
        return flat
    }

    // The rows on screen, one line each, a folded run carrying its count.
    function lineOf(r) { return r.time + "  " + (r.stage ? r.stage + "  " + r.level + "  " : "") + r.msg + (r.count > 1 ? "  ×" + r.count : "") }
    property bool copied: false
    Timer { id: copiedTimer; interval: 2000; onTriggered: root.copied = false }
    function copyAll() {
        frame.copyQuiet(rows.map(lineOf).join("\n"))
        copied = true
        copiedTimer.restart()
    }
    // One line, by a click on it
    function copyLine(r) { frame.copyText(lineOf(r), words.t("activity-drawer-copied-line")) }
    // Clear empties the log for good: the first click asks, the second clears
    property bool clearArmed: false
    Timer { id: clearDisarm; interval: 4000; onTriggered: root.clearArmed = false }
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
                    text: words.t("activity-drawer-title")
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
                    options: [{ text: words.t("activity-drawer-everything") }, { text: words.t("activity-drawer-problems") }]
                    index: root.problemsOnly ? 1 : 0
                    onPicked: function(i) { root.problemsOnly = i === 1 }
                }
                Repeater {
                    model: root.technical ? root.stages : []
                    Chip {
                        required property string modelData
                        anchors.verticalCenter: parent.verticalCenter
                        text: words.t(root.stageWords[modelData])
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
                        text: words.t(root.levelWords[modelData])
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
                Button { text: root.copied ? words.t("activity-drawer-copied") : words.t("activity-drawer-copy"); icon: root.copied ? "check" : "copy"; flat: true; small: true; onClicked: root.copyAll() }
                Button {
                    text: root.clearArmed ? words.t("activity-drawer-clear-every") : words.t("action-clear"); icon: "trash-2"; flat: true; small: true; danger: root.clearArmed
                    onClicked: {
                        if (!root.clearArmed) { root.clearArmed = true; clearDisarm.restart(); return }
                        root.clearArmed = false
                        Door.clearEvents(); root.entries = []; root.expanded = ({})
                    }
                }
                Button { msg: "action-close"; icon: "x"; flat: true; small: true; onClicked: root.close() }
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
                id: logRow
                required property var modelData
                width: list.width
                height: theme.textSpace(7)
                Rectangle { anchors.fill: parent; color: rowMouse.containsMouse ? theme.tileHover : "transparent"; radius: theme.radiusSm }
                MouseArea { id: rowMouse; anchors.fill: parent; hoverEnabled: true; cursorShape: Qt.PointingHandCursor; onClicked: root.copyLine(logRow.modelData); Accessible.role: Accessible.Button; Accessible.name: theme.nameNear(parent)
                    Accessible.onPressAction: root.copyLine(logRow.modelData) }
                Tooltip { msg: "activity-drawer-copy-tip" }
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
                        anchors.verticalCenter: parent.verticalCenter
                        visible: modelData.foldKey !== undefined && modelData.foldKey >= 0
                        msg: "activity-drawer-fold"
                        small: true
                        clickable: true
                        color: theme.tileSunken
                        textColor: theme.textDim
                        onClicked: { var n = {}; for (var k in root.expanded) if (Number(k) !== modelData.foldKey) n[k] = root.expanded[k]; root.expanded = n }
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
            text: root.technical ? words.t("activity-drawer-empty-technical") : root.problemsOnly ? words.t("activity-drawer-empty-problems") : words.t("activity-drawer-empty")
            color: theme.textDim
        }
    }
}
