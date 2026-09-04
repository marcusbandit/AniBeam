// The activity drawer: rises from the status strip with exponential smoothing. A header
// with the stage and level filters and the Copy, Clear and Close actions, glyphs and all, then the log,
// newest first, with runs of identical lines folded into one row that expands on click.
import QtQuick
import QtQuick.Controls.Basic as QC

Item {
    id: root
    property bool open: false
    property real maxHeight: theme.space(100)
    property var entries: []
    signal closed()

    readonly property var stages: ["scan", "match", "image", "play", "tracker", "export", "system"]
    readonly property var levels: ["info", "warn", "error"]
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

    function toggle() { open = !open; if (open) forceActiveFocus() }
    function close() { open = false; closed() }
    Keys.onEscapePressed: close()

    function anyOn(o) { for (var k in o) if (o[k]) return true; return false }
    function flip(o, k) { var n = {}; for (var j in o) n[j] = o[j]; n[k] = !n[k]; return n }
    function levelColor(l) { return l === "error" ? theme.red : l === "warn" ? theme.yellow : theme.text }

    // Filter, then fold runs of identical consecutive lines; an expanded run is laid out again
    readonly property var rows: {
        var sAny = anyOn(stageOn), lAny = anyOn(levelOn)
        var out = []
        var run = null
        for (var i = 0; i < entries.length; i++) {
            var e = entries[i]
            if (sAny && !stageOn[e.stage]) continue
            if (lAny && !levelOn[e.level]) continue
            if (run && run.stage === e.stage && run.level === e.level && run.msg === e.msg) {
                run.count += 1
                run.items.push(e)
                continue
            }
            run = { time: e.time, stage: e.stage, level: e.level, msg: e.msg, count: 1, key: i, items: [e] }
            out.push(run)
        }
        var flat = []
        out.forEach(function(r) {
            if (r.count > 1 && expanded[r.key]) r.items.forEach(function(e) { flat.push({ time: e.time, stage: e.stage, level: e.level, msg: e.msg, count: 1, key: -1 }) })
            else flat.push(r)
        })
        return flat
    }

    function copyAll() {
        var lines = entries.map(function(e) { return e.time + "  " + e.stage + "  " + e.level + "  " + e.msg })
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

        Rectangle { anchors.fill: parent; color: theme.surface }
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
                Repeater {
                    model: root.stages
                    Chip {
                        required property string modelData
                        anchors.verticalCenter: parent.verticalCenter
                        text: modelData
                        small: true
                        clickable: true
                        selected: root.stageOn[modelData] === true
                        color: selected ? theme.accentSoft : theme.surfaceSunken
                        textColor: theme.textDim
                        onClicked: root.stageOn = root.flip(root.stageOn, modelData)
                    }
                }
                Item { width: theme.space(2); height: 1 }
                Repeater {
                    model: root.levels
                    Chip {
                        required property string modelData
                        anchors.verticalCenter: parent.verticalCenter
                        text: modelData
                        small: true
                        clickable: true
                        selected: root.levelOn[modelData] === true
                        color: selected ? theme.accentSoft : theme.surfaceSunken
                        textColor: modelData === "info" ? theme.textDim : root.levelColor(modelData)
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
                Button { text: "Clear"; icon: "trash-2"; flat: true; small: true; onClicked: { root.entries = []; root.expanded = ({}) } }
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
            model: root.rows
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
                        font.family: theme.fontMono
                        font.pointSize: theme.typeSmall
                    }
                    Chip {
                        id: stageChip
                        anchors.verticalCenter: parent.verticalCenter
                        text: modelData.stage
                        small: true
                        color: theme.surfaceSunken
                        textColor: theme.textDim
                        width: theme.space(16)
                    }
                    Text {
                        anchors.verticalCenter: parent.verticalCenter
                        width: Math.min(implicitWidth, line.width - stamp.width - stageChip.width - line.spacing * 2 - (fold.visible ? fold.width + line.spacing : 0))
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
                        color: theme.surfaceSunken
                        textColor: theme.textDim
                        onClicked: { var n = {}; for (var k in root.expanded) n[k] = root.expanded[k]; n[modelData.key] = true; root.expanded = n }
                    }
                }
                Rectangle { anchors.bottom: parent.bottom; width: parent.width; height: 1; color: theme.surfaceSunken }
            }
            QC.ScrollBar.vertical: QC.ScrollBar {
                policy: QC.ScrollBar.AsNeeded
                contentItem: Corner { implicitWidth: theme.space(1); radius: implicitWidth / 2; smoothing: theme.cornerSmoothing; color: theme.lineStrong; opacity: parent.active ? 1 : 0.4 }
            }
        }
    }
}
