// The quick fix (#170): what each file of a series really is, against how the scan read its
// name. A floating pane over a Blocker, like the match panel: one row per file with its name,
// how it reads now and what it should be (a kind and a number), the page's own suggestion one
// click away, and Apply with a small, unticked check beside it that files the fix as a GitHub
// issue. Applying writes the fixes at once (FixFiles) and every later scan keeps them; a fixed
// row can go back to reading its name. With the check ticked the frame drafts the issue
// (frame.reportFix) and opens it in the browser when it is ready, a coding agent's triage
// included when one is installed. Escape, the X, Cancel and a press on the shield all close
// it through close().
import QtQuick
import QtQuick.Controls.Basic as QC
import com.marcusrosado.AniBeam

Corner {
    id: root
    objectName: "fix-sheet"
    property real seriesId: 0
    property string seriesTitle: ""
    // { file, name, reading: { Episode: { number, season } } or { Extra: { kind, index } }, fixed }
    property var rows: []
    // file -> { kind, number, undo }: what the user has set so far, by row
    property var edits: ({})
    property var suggestion: null
    property bool report: false
    // The coding agent that would triage the issue, by name, or ""
    property string agent: ""
    property real applyToken: -1
    readonly property bool applying: applyToken >= 0
    readonly property var kinds: ["Episode", "Opening", "Ending", "Preview", "Special", "Other"]
    readonly property var extraKinds: ["", "Op", "Ed", "Pv", "Sp", "Other"]

    function kindOf(reading) {
        if (reading.Episode) return 0
        var i = extraKinds.indexOf(reading.Extra.kind)
        return i > 0 ? i : 5
    }
    function numberOf(reading) {
        if (reading.Episode) return String(reading.Episode.number)
        var i = reading.Extra.index
        return i === null || i === undefined ? "" : String(i)
    }
    function describe(reading) {
        if (reading.Episode) {
            var s = reading.Episode.season
            return (s !== null && s !== undefined && s !== 1 ? "Season " + s + " episode " : "Episode ") + reading.Episode.number
        }
        var i = reading.Extra.index
        return kinds[kindOf(reading)] + (i === null || i === undefined ? "" : " " + i)
    }
    function stateOf(row) {
        var e = edits[row.file]
        return e ? e : { kind: kindOf(row.reading), number: numberOf(row.reading), undo: false }
    }
    function setEdit(row, patch) {
        var next = Object.assign({}, edits)
        next[row.file] = Object.assign({}, stateOf(row), patch)
        edits = next
    }
    // The reading a row's controls say, or null when they cannot say one yet
    function readingFrom(row, e) {
        var text = String(e.number).trim()
        if (e.kind === 0) {
            if (!/^\d{1,5}(\.\d+)?$/.test(text) || Number(text) > 10000) return null
            var season = row.reading.Episode ? row.reading.Episode.season : null
            return { Episode: { number: Number(text), season: season === undefined ? null : season } }
        }
        if (text !== "" && (!/^\d{1,5}$/.test(text) || Number(text) > 10000)) return null
        return { Extra: { kind: extraKinds[e.kind], index: text === "" ? null : Number(text) } }
    }
    function same(a, b) {
        if (!a || !b || (!a.Episode && !a.Extra) || (!b.Episode && !b.Extra)) return false
        if (!!a.Episode !== !!b.Episode) return false
        if (a.Episode) return a.Episode.number === b.Episode.number && (a.Episode.season || null) === (b.Episode.season || null)
        return a.Extra.kind === b.Extra.kind && (a.Extra.index || null) === (b.Extra.index || null)
    }
    function invalid(row) { var e = edits[row.file]; return !!e && !e.undo && readingFrom(row, e) === null }
    function changed(row) {
        var e = edits[row.file]
        if (!e) return false
        if (e.undo) return row.fixed
        var r = readingFrom(row, e)
        return r !== null && !same(r, row.reading)
    }
    readonly property var changes: rows.filter(changed).map(function(row) {
        var e = edits[row.file]
        return { file: row.file, reading: e.undo ? null : readingFrom(row, e) }
    })
    readonly property bool anyInvalid: rows.some(invalid)
    readonly property bool suggestionUsed: !!suggestion && suggestion.fixes.every(function(f) {
        var row = rows.filter(function(r) { return r.file === f.file })[0]
        return !row || (!!edits[f.file] && !edits[f.file].undo && same(readingFrom(row, edits[f.file]), f.reading))
    })

    function openFor(id, title, files, suggested) {
        root.seriesId = id
        root.seriesTitle = title
        root.rows = files
        root.edits = ({})
        root.report = false
        root.applyToken = -1
        root.suggestion = suggested || null
        var r = Door.call("TriageAgent", {})
        root.agent = !r.error && r.reply && r.reply.name ? r.reply.name : ""
        if (root.suggestion) root.useSuggestion()
        list.contentY = 0
        visible = true
        frame.escapeStack.push("popover", root)
        panelFocus.forceActiveFocus()
    }
    function close() {
        if (!visible) return
        visible = false
        frame.escapeStack.pop(root)
    }
    Component.onDestruction: if (visible) frame.escapeStack.pop(root)
    function useSuggestion() {
        if (!root.suggestion) return
        var next = Object.assign({}, root.edits)
        root.suggestion.fixes.forEach(function(f) {
            if (!f.reading) return
            next[f.file] = { kind: root.kindOf(f.reading), number: root.numberOf(f.reading), undo: false }
        })
        root.edits = next
    }
    function apply() {
        if (root.applying || root.changes.length === 0 || root.anyInvalid) return
        root.applyToken = Door.callAsync("FixFiles", { series: root.seriesId, fixes: root.changes })
        if (root.applyToken < 0) frame.toast("Could not apply the fixes")
    }
    Connections {
        target: Door
        function onCallDone(token, reply) {
            if (token !== root.applyToken) return
            root.applyToken = -1
            if (reply.error) { frame.toast(frame.errorText(reply.error)); return }
            var fixed = root.changes.filter(function(c) { return c.reading !== null }).map(function(c) { return c.file })
            var n = root.changes.length
            var file = n === 1 ? "1 file" : n + " files"
            var wantReport = root.report && fixed.length > 0
            root.close()
            if (wantReport) frame.reportFix(root.seriesId, fixed, root.agent)
            else frame.toast("Fixed how " + file + " read")
        }
    }

    readonly property real pad: theme.space(5)
    readonly property real readWidth: theme.space(34)
    readonly property real kindWidth: theme.space(34)
    readonly property real numberWidth: theme.space(16)
    readonly property real undoWidth: theme.space(9)
    width: Math.min(theme.space(210), (parent ? parent.width : theme.space(210)) - theme.space(16))
    height: Math.min(head.height + theme.space(3) + listHeight + foot.height + root.pad * 2 + theme.space(3),
                     (parent ? parent.height : theme.space(170)) - theme.space(16))
    readonly property real rowHeight: theme.controlHeight + theme.space(2)
    readonly property real listHeight: Math.max(rowHeight, rows.length * (rowHeight + list.spacing) + theme.space(7))
    visible: false
    color: theme.glassFloat
    borderColor: theme.lineStrong
    borderWidth: 1
    highlight: theme.glassHighlight

    // The panel's own press sink: a press on its padding never reaches the shield under it
    MouseArea { anchors.fill: parent; acceptedButtons: Qt.AllButtons; hoverEnabled: true; onWheel: function(w) { w.accepted = true } }
    Item {
        id: panelFocus
        focus: true
        Keys.onEscapePressed: root.close()
        Keys.onReturnPressed: root.apply()
        Keys.onEnterPressed: root.apply()
    }

    Column {
        id: head
        x: root.pad; y: root.pad
        width: root.width - root.pad * 2
        spacing: theme.space(3)
        Item {
            width: parent.width
            height: Math.max(heading.height, closeBtn.height)
            Row {
                id: heading
                width: parent.width - closeBtn.width - theme.space(3)
                spacing: theme.space(3)
                Icon { anchors.verticalCenter: parent.verticalCenter; glyph: "wrench"; size: theme.space(5); color: theme.accent }
                Column {
                    width: parent.width - theme.space(8)
                    Label { text: "Fix how files are read"; role: "title" }
                    Label { width: parent.width; text: root.seriesTitle + ". Say what each file really is; every scan after this keeps it."; role: "ui"; color: theme.textDim; elide: Text.ElideRight }
                }
            }
            Button {
                id: closeBtn
                objectName: "fix-close"
                anchors.right: parent.right; anchors.top: parent.top
                small: true; icon: "x"; text: "Close"
                onClicked: root.close()
            }
        }
        // The page's own suggestion, one click from the rows
        Corner {
            objectName: "fix-suggestion"
            visible: !!root.suggestion
            width: parent.width
            height: suggestionRow.implicitHeight + theme.space(3) * 2
            radius: theme.radiusTile
            smoothing: theme.cornerSmoothing
            color: theme.accentSoft
            Row {
                id: suggestionRow
                x: theme.space(4)
                width: parent.width - theme.space(8)
                anchors.verticalCenter: parent.verticalCenter
                spacing: theme.space(3)
                Icon { anchors.verticalCenter: parent.verticalCenter; glyph: "sparkles"; size: theme.space(4.5); color: theme.accent }
                Label {
                    anchors.verticalCenter: parent.verticalCenter
                    width: parent.width - theme.space(7.5) - useBtn.width - parent.spacing * 2
                    text: root.suggestion ? root.suggestion.text : ""
                    role: "ui"; color: theme.text; wrapMode: Text.Wrap
                }
                Button {
                    id: useBtn
                    anchors.verticalCenter: parent.verticalCenter
                    small: true
                    text: root.suggestionUsed ? "Filled in" : "Fill in"
                    icon: root.suggestionUsed ? "check" : "sparkles"
                    flat: root.suggestionUsed
                    onClicked: root.useSuggestion()
                }
            }
        }
    }

    // The files, a row each, scrolling inside the panel
    ListView {
        id: list
        objectName: "fix-rows"
        x: root.pad
        y: head.y + head.height + theme.space(3)
        width: root.width - root.pad * 2
        height: foot.y - y - theme.space(3)
        clip: true
        spacing: theme.space(1)
        model: root.rows
        boundsBehavior: Flickable.StopAtBounds
        QC.ScrollBar.vertical: ThinScrollBar {}
        WheelStep { flick: list }
        header: Item {
            id: header
            width: list.width
            height: theme.space(7)
            Row {
                anchors.fill: parent
                anchors.leftMargin: theme.space(3)
                spacing: theme.space(3)
                Caption { width: list.width - root.readWidth - root.kindWidth - root.numberWidth - root.undoWidth - theme.space(3) * 5; text: "File" }
                Caption { width: root.readWidth; text: "Reads as" }
                Caption { width: root.kindWidth + root.numberWidth + theme.space(3); text: "Should be" }
            }
        }
        delegate: Corner {
            id: line
            required property var modelData
            required property int index
            objectName: "fix-row-" + index
            readonly property var st: root.stateOf(modelData)
            readonly property bool isChanged: root.changed(modelData)
            readonly property bool isInvalid: root.invalid(modelData)
            width: list.width
            height: root.rowHeight
            radius: theme.radiusMd
            smoothing: theme.cornerSmoothing
            color: isChanged ? theme.accentSoft : rowMouse.containsMouse ? theme.floatHover : "transparent"
            MouseArea { id: rowMouse; anchors.fill: parent; hoverEnabled: true }
            Row {
                x: theme.space(3)
                width: parent.width - theme.space(3)
                anchors.verticalCenter: parent.verticalCenter
                spacing: theme.space(3)
                Label {
                    anchors.verticalCenter: parent.verticalCenter
                    width: list.width - root.readWidth - root.kindWidth - root.numberWidth - root.undoWidth - theme.space(3) * 5
                    text: line.modelData.name
                    role: "ledger"; color: theme.text
                    elide: Text.ElideMiddle
                    Tooltip { text: line.modelData.name }
                }
                Row {
                    anchors.verticalCenter: parent.verticalCenter
                    width: root.readWidth
                    spacing: theme.space(2)
                    Label { anchors.verticalCenter: parent.verticalCenter; text: root.describe(line.modelData.reading); role: "ui"; color: theme.textDim; elide: Text.ElideRight }
                    Label { visible: line.modelData.fixed; anchors.verticalCenter: parent.verticalCenter; text: "Fixed"; role: "ledger"; color: theme.accent }
                }
                Dropdown {
                    objectName: "fix-kind-" + line.index
                    anchors.verticalCenter: parent.verticalCenter
                    width: root.kindWidth
                    minWidth: root.kindWidth
                    options: root.kinds
                    index: line.st.kind
                    interactive: !line.st.undo
                    opacity: line.st.undo ? 0.4 : 1
                    onPicked: function(i) { root.setEdit(line.modelData, { kind: i, undo: false }) }
                }
                Field {
                    id: numberField
                    objectName: "fix-number-" + line.index
                    anchors.verticalCenter: parent.verticalCenter
                    width: root.numberWidth
                    mono: true
                    placeholder: line.st.kind === 0 ? "No." : "None"
                    text: line.st.number
                    enabled: !line.st.undo
                    opacity: line.st.undo ? 0.4 : 1
                    onTextChanged: if (text !== String(line.st.number)) root.setEdit(line.modelData, { number: text, undo: false })
                    // A number this kind cannot take: the field's edge in red
                    Rectangle {
                        anchors.fill: parent
                        visible: line.isInvalid
                        radius: numberField.radius
                        color: "transparent"
                        border.color: theme.red
                        border.width: 1.5
                    }
                }
                Item {
                    anchors.verticalCenter: parent.verticalCenter
                    width: root.undoWidth; height: theme.controlHeight
                    Button {
                        anchors.centerIn: parent
                        visible: line.modelData.fixed
                        small: true; flat: true
                        icon: line.st.undo ? "x" : "rotate-ccw"
                        text: ""
                        onClicked: root.setEdit(line.modelData, { undo: !line.st.undo })
                        Tooltip { text: line.st.undo ? "Keep the fix" : "Read this file by its name again" }
                    }
                }
            }
        }
    }

    // Apply, with the small check that also files it as an issue
    Item {
        id: foot
        x: root.pad
        y: root.height - root.pad - height
        width: root.width - root.pad * 2
        height: Math.max(reportCheck.height, buttons.height)
        Item {
            id: reportCheck
            objectName: "fix-report"
            width: parent.width - buttons.width - theme.space(6)
            height: Math.max(box.height, words.implicitHeight)
            Rectangle {
                id: box
                y: theme.space(0.25)
                width: theme.space(4); height: width
                radius: theme.radiusSm
                color: root.report ? theme.accent : "transparent"
                border.color: root.report ? theme.accent : (checkMouse.containsMouse ? theme.textDim : theme.lineStrong)
                border.width: 1.5
                Icon { visible: root.report; anchors.centerIn: parent; glyph: "check"; size: theme.space(3); color: theme.contrast(theme.accent, Qt.rgba(1, 1, 1, 1)) >= 4.5 ? "#ffffff" : "#0d0e11" }
            }
            Column {
                id: words
                anchors.left: box.right
                anchors.leftMargin: theme.space(2)
                anchors.right: parent.right
                spacing: theme.space(0.5)
                Label { width: parent.width; text: "File an issue on GitHub"; role: "ui"; font.pointSize: theme.typeSmall; color: theme.text }
                Label {
                    width: parent.width
                    visible: root.report
                    text: "Opens a filled in issue for you to send. It is public and names these files." + (root.agent !== "" ? " " + root.agent + " looks into it first." : "")
                    role: "ui"; font.pointSize: theme.typeSmall; color: theme.textDim; wrapMode: Text.Wrap
                }
            }
            MouseArea { id: checkMouse; anchors.fill: parent; hoverEnabled: true; cursorShape: Qt.PointingHandCursor; onClicked: root.report = !root.report }
        }
        Row {
            id: buttons
            anchors.right: parent.right
            anchors.bottom: parent.bottom
            spacing: theme.space(2)
            Button { objectName: "fix-cancel"; text: "Cancel"; flat: true; onClicked: root.close() }
            Button {
                objectName: "fix-apply"
                primary: true
                icon: "wrench"
                busy: root.applying
                enabled: root.changes.length > 0 && !root.anyInvalid && !root.applying
                text: root.changes.length === 0 ? "Apply" : root.changes.length === 1 ? "Apply 1 fix" : "Apply " + root.changes.length + " fixes"
                onClicked: root.apply()
            }
        }
    }

    component Caption: Label {
        role: "ui"
        font.pointSize: theme.typeSmall
        color: theme.textFaint
        elide: Text.ElideRight
    }
}
