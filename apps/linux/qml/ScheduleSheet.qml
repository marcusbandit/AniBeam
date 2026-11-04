// The schedule popover (#418): tell a show's schedule what its provider does not say. Two
// ways in, one draft between them. Pick: click episodes (shift-click for a range), then give
// them a day, a month, a year or TBC with the buttons or a typed date. Paste: a release list
// read by the core as it is typed. A paste that reads wrong is fixed in Pick, since both edit
// the same draft; Save keeps the draft for the season the page is about. A raised tile over a
// Blocker the owning page puts under it, on the frame's escape stack like the match panel.
import QtQuick
import com.marcusrosado.AniBeam

Corner {
    id: root
    objectName: "schedule-sheet"
    property real seriesId: 0
    property string seriesTitle: ""
    // The season the notes are for: the card's own for a TVmaze show, else none
    property var season: null
    property var saved: []
    // The draft both modes edit, in the wire shape: {first, last, at, year, month}
    property var draft: []
    property var readSeason: null
    property int mode: 0            // 0 pick, 1 paste
    // Pick: how many episode cells, which are out already, which are picked
    property int upTo: 12
    property var aired: []
    property var picked: []
    property int anchorEp: 0
    property bool weekly: false
    // The typed date as the core reads it, or null
    property var typed: null
    readonly property int nextYear: new Date().getFullYear() + 1

    function has(v) { return v !== null && v !== undefined }
    function epsText(n) { return n.first === n.last ? "Episode " + n.first : "Episodes " + n.first + " to " + n.last }
    function whenOf(n) {
        if (has(n.at)) return Qt.formatDate(new Date(n.at * 1000), "ddd d MMM yyyy")
        if (has(n.year) && n.month) return Qt.formatDate(new Date(n.year, n.month - 1, 1), "MMMM yyyy")
        if (has(n.year)) return "Some time in " + n.year
        return "No date yet"
    }
    function kindOf(n) { return !n ? "" : has(n.at) ? "day" : has(n.year) && n.month ? "month" : has(n.year) ? "year" : "tbc" }
    function kindColor(k) { return k === "day" ? theme.green : k === "month" ? theme.orange : k === "year" ? theme.purple : theme.textDim }
    function keyOf(w) { return [has(w.at) ? w.at : "", has(w.year) ? w.year : "", w.month || ""].join("|") }

    // The draft as one entry an episode, and back into ranges of the same date
    function expand(notes) {
        var by = {}
        for (var i = 0; i < notes.length; i++)
            for (var e = notes[i].first; e <= notes[i].last; e++)
                by[e] = { at: has(notes[i].at) ? notes[i].at : null, year: has(notes[i].year) ? notes[i].year : null, month: notes[i].month || null }
        return by
    }
    function collapse(by) {
        var eps = Object.keys(by).map(Number).sort(function(a, b) { return a - b })
        var out = []
        for (var i = 0; i < eps.length; i++) {
            var w = by[eps[i]], last = out[out.length - 1]
            if (last && last.last === eps[i] - 1 && root.keyOf(last) === root.keyOf(w)) last.last = eps[i]
            else out.push({ first: eps[i], last: eps[i], at: w.at, year: w.year, month: w.month })
        }
        return out
    }
    function noteFor(ep) {
        for (var i = 0; i < draft.length; i++) if (ep >= draft[i].first && ep <= draft[i].last) return draft[i]
        return null
    }

    // Pick: a click toggles one, a shift-click picks the run from the last click
    function clickEp(ep, shift) {
        var p = root.picked.slice()
        if (shift && root.anchorEp > 0) {
            var a = Math.min(root.anchorEp, ep), b = Math.max(root.anchorEp, ep)
            for (var e = a; e <= b; e++) if (p.indexOf(e) < 0) p.push(e)
        } else {
            var at = p.indexOf(ep)
            if (at >= 0) p.splice(at, 1); else p.push(ep)
        }
        root.anchorEp = ep
        root.picked = p.sort(function(x, y) { return x - y })
    }
    // Picks every episode from the first one not out yet to the last cell
    function pickRest() {
        var p = []
        for (var e = 1; e <= root.upTo; e++) if (root.aired.indexOf(e) < 0) p.push(e)
        root.picked = p
    }
    // Gives the picked episodes a date; weekly spreads a day one week apart
    function apply(w) {
        if (!root.picked.length || !w) return
        var by = root.expand(root.draft)
        for (var i = 0; i < root.picked.length; i++) {
            var at = has(w.at) && root.weekly ? w.at + i * 7 * 86400 : (has(w.at) ? w.at : null)
            by[root.picked[i]] = { at: at, year: has(at) ? null : (has(w.year) ? w.year : null), month: has(at) ? null : (w.month || null) }
        }
        root.draft = root.collapse(by)
        root.picked = []
    }
    function unset() {
        var by = root.expand(root.draft)
        for (var i = 0; i < root.picked.length; i++) delete by[root.picked[i]]
        root.draft = root.collapse(by)
        root.picked = []
    }
    function removeNote(n) {
        var by = root.expand(root.draft)
        for (var e = n.first; e <= n.last; e++) delete by[e]
        root.draft = root.collapse(by)
    }
    // A typed date read by the same parser as a paste: "7 Jan", "March 2027", "2027", "TBC"
    function readTyped(text) {
        var t = String(text).trim()
        if (!t) { root.typed = null; return }
        var r = Door.call("ParseSchedule", { text: "Episode 1 - " + t })
        var n = !r.error && r.reply.notes && r.reply.notes.length ? r.reply.notes[0] : null
        root.typed = n ? { at: has(n.at) ? n.at : null, year: has(n.year) ? n.year : null, month: n.month || null } : null
    }

    function openFor(id, title, season, notes, upTo, aired) {
        root.seriesId = id
        root.seriesTitle = title
        root.season = season === undefined ? null : season
        root.saved = notes || []
        root.draft = root.collapse(root.expand(root.saved))
        root.readSeason = null
        root.aired = aired || []
        var top = 0
        for (var i = 0; i < root.saved.length; i++) top = Math.max(top, root.saved[i].last)
        root.upTo = Math.min(Math.max(upTo || 0, top, 12), 200)
        root.picked = []
        root.anchorEp = 0
        root.weekly = false
        root.typed = null
        dateField.text = ""
        input.text = ""
        root.mode = 0
        visible = true
        frame.escapeStack.push("popover", root)
    }
    function close() {
        if (!visible) return
        visible = false
        frame.escapeStack.pop(root)
    }
    Component.onDestruction: if (visible) frame.escapeStack.pop(root)
    // A paste replaces the draft with what it reads; Pick then fixes what it missed
    function parse() {
        if (!input.text.trim()) return
        var r = Door.call("ParseSchedule", { text: input.text })
        if (r.error) return
        root.draft = root.collapse(root.expand(r.reply.notes || []))
        root.readSeason = has(r.reply.season) ? r.reply.season : null
    }
    function save(notes) {
        var wire = notes.map(function(n) {
            return { first: n.first, last: n.last, year: has(n.year) ? n.year : null, month: n.month || null,
                     at: has(n.at) ? { secs_since_epoch: Math.round(n.at), nanos_since_epoch: 0 } : null }
        })
        // A season the text names wins over the page's own, for a show numbered in seasons
        var season = root.season !== null && root.readSeason !== null ? root.readSeason : root.season
        var r = Door.call("SetScheduleNotes", { series: root.seriesId, season: season, notes: wire })
        if (r.error) { frame.toast(frame.errorText(r.error)); return }
        frame.toast(wire.length ? "Schedule saved" : "Schedule notes cleared")
        root.close()
    }

    readonly property real pad: theme.space(5)
    width: Math.min(theme.space(150), (parent ? parent.width : theme.space(150)) - theme.space(16))
    height: Math.min(body.implicitHeight + pad * 2, (parent ? parent.height : theme.space(160)) - theme.space(16))
    Timer { id: typedDebounce; interval: 120; onTriggered: root.readTyped(dateField.text) }
    visible: false
    color: theme.glassFloat
    borderColor: theme.lineStrong
    borderWidth: 1
    highlight: theme.glassHighlight

    Timer { id: debounce; interval: 120; onTriggered: root.parse() }

    // The panel's own press sink: a press on its padding never reaches the shield under it
    MouseArea { anchors.fill: parent; acceptedButtons: Qt.AllButtons; hoverEnabled: true; onWheel: function(w) { w.accepted = true } }

    Flickable {
        id: bodyFlick
        x: root.pad; y: root.pad
        width: root.width - root.pad * 2
        height: root.height - root.pad * 2
        contentWidth: width
        contentHeight: body.implicitHeight
        clip: true
        boundsBehavior: Flickable.StopAtBounds
        WheelStep { flick: bodyFlick }
    Column {
        id: body
        width: bodyFlick.width
        spacing: theme.space(3)

        Item {
            width: parent.width
            height: Math.max(heading.height, closeBtn.height)
            Column {
                id: heading
                width: parent.width - closeBtn.width - theme.space(3)
                Label { text: "Schedule"; role: "title" }
                Label {
                    width: parent.width
                    text: root.seriesTitle + (root.season !== null ? ", season " + (root.readSeason !== null ? root.readSeason : root.season) : "")
                    role: "ui"; color: theme.textDim; elide: Text.ElideRight
                }
            }
            Button {
                id: closeBtn
                objectName: "schedule-close"
                anchors.right: parent.right; anchors.top: parent.top
                small: true; icon: "x"; text: "Close"
                onClicked: root.close()
            }
        }

        Seg {
            id: modeSeg
            objectName: "schedule-mode"
            options: ["Pick episodes", "Paste a list"]
            index: root.mode
            onPicked: function(i) { root.mode = i; if (i === 1) input.forceActiveFocus() }
        }

        Label {
            width: parent.width
            wrapMode: Text.WordWrap
            text: root.mode === 0
                  ? "Click episodes, shift-click for a run, then give them a date. AniBeam stops guessing at a TBC, and a date the provider lists still wins."
                  : "Paste what a site or a post says. What AniBeam reads lands below; anything it got wrong, fix under Pick episodes."
            role: "ui"; color: theme.textDim
        }

        // Pick: the episode cells
        Flow {
            id: cells
            visible: root.mode === 0
            objectName: "schedule-cells"
            width: parent.width
            spacing: theme.space(1.5)
            Repeater {
                model: root.upTo
                delegate: Corner {
                    id: cell
                    required property int index
                    readonly property int ep: index + 1
                    readonly property bool isPicked: root.picked.indexOf(ep) >= 0
                    readonly property bool isOut: root.aired.indexOf(ep) >= 0
                    readonly property string kind: root.kindOf(root.noteFor(ep))
                    objectName: "schedule-ep-" + ep
                    width: theme.space(10); height: theme.space(10)
                    radius: theme.space(2.5)
                    color: isPicked ? theme.accent : cellMouse.containsMouse ? theme.raisedHover : theme.raised
                    borderColor: isPicked ? theme.accent : kind ? root.kindColor(kind) : theme.line
                    borderWidth: kind && !isPicked ? 2 : 1
                    Label {
                        anchors.centerIn: parent
                        text: String(cell.ep)
                        role: "ledger"
                        font.weight: 600
                        color: cell.isPicked ? theme.accentText : cell.isOut ? theme.textFaint : theme.text
                    }
                    MouseArea {
                        id: cellMouse
                        anchors.fill: parent
                        hoverEnabled: true
                        cursorShape: Qt.PointingHandCursor
                        onClicked: function(m) { root.clickEp(cell.ep, (m.modifiers & Qt.ShiftModifier) !== 0) }
                    }
                }
            }
            // One more row of cells, for a season longer than the page knows
            Corner {
                width: theme.space(10); height: theme.space(10)
                radius: theme.space(2.5)
                color: moreMouse.containsMouse ? theme.raisedHover : "transparent"
                borderColor: theme.line; borderWidth: 1; dashed: 1
                Icon { anchors.centerIn: parent; glyph: "plus"; size: theme.space(4); color: theme.textDim }
                MouseArea { id: moreMouse; anchors.fill: parent; hoverEnabled: true; cursorShape: Qt.PointingHandCursor; onClicked: root.upTo = Math.min(root.upTo + 12, 200) }
            }
        }
        Row {
            visible: root.mode === 0
            spacing: theme.space(2)
            Button { small: true; flat: true; text: "Pick the rest"; onClicked: root.pickRest() }
            Button { small: true; flat: true; text: "Pick none"; enabled: root.picked.length > 0; onClicked: root.picked = [] }
            Label {
                anchors.verticalCenter: parent.verticalCenter
                text: root.picked.length ? Fmt.plural(root.picked.length, "episode", "episodes") + " picked" : "Nothing picked"
                role: "ledger"; color: theme.textFaint
            }
        }

        // Pick: the date for what is picked
        Row {
            visible: root.mode === 0
            width: parent.width
            spacing: theme.space(2)
            Field {
                id: dateField
                objectName: "schedule-date"
                width: theme.space(56)
                placeholder: "7 Jan 2027, March 2027, 2027"
                onTextChanged: typedDebounce.restart()
                Keys.onReturnPressed: root.apply(root.typed)
                Keys.onEnterPressed: root.apply(root.typed)
            }
            Button {
                objectName: "schedule-set"
                primary: true
                text: root.typed ? "Set " + root.whenOf(root.typed) : "Set date"
                enabled: root.picked.length > 0 && root.typed !== null
                onClicked: root.apply(root.typed)
            }
            ToggleChip {
                anchors.verticalCenter: parent.verticalCenter
                text: "Weekly"
                checked: root.weekly
                onToggled: root.weekly = !root.weekly
            }
        }
        Row {
            visible: root.mode === 0
            spacing: theme.space(2)
            Button { objectName: "schedule-tbc"; small: true; text: "No date yet"; enabled: root.picked.length > 0; onClicked: root.apply({ at: null, year: null, month: null }) }
            Button { small: true; text: String(root.nextYear - 1); enabled: root.picked.length > 0; onClicked: root.apply({ at: null, year: root.nextYear - 1, month: null }) }
            Button { objectName: "schedule-next-year"; small: true; text: String(root.nextYear); enabled: root.picked.length > 0; onClicked: root.apply({ at: null, year: root.nextYear, month: null }) }
            Button { small: true; flat: true; text: "Clear their dates"; enabled: root.picked.length > 0; onClicked: root.unset() }
        }

        // The text, a pane like a Field that grows to a few lines and then scrolls
        Corner {
            visible: root.mode === 1
            width: parent.width
            height: Math.min(Math.max(input.contentHeight + theme.space(4), theme.space(28)), theme.space(64))
            radius: theme.radiusTile
            smoothing: theme.cornerSmoothing
            color: input.activeFocus ? theme.raised : theme.tile
            borderColor: input.activeFocus ? theme.focusRing : theme.line
            borderWidth: 1
            Flickable {
                id: textFlick
                anchors.fill: parent
                anchors.margins: theme.space(2)
                anchors.leftMargin: theme.space(3)
                contentWidth: width
                contentHeight: input.contentHeight
                clip: true
                boundsBehavior: Flickable.StopAtBounds
                WheelStep { flick: textFlick }
                function ensureVisible(r) {
                    if (contentY >= r.y) contentY = r.y
                    else if (contentY + height <= r.y + r.height) contentY = r.y + r.height - height
                }
                TextEdit {
                    id: input
                    objectName: "schedule-input"
                    width: textFlick.width
                    textFormat: TextEdit.PlainText
                    wrapMode: TextEdit.Wrap
                    color: theme.text
                    font.family: theme.fontFamily
                    font.pointSize: theme.typeSmall
                    font.weight: theme.roleWeight("ledger")
                    font.variableAxes: theme.roleAxes("ledger")
                    font.features: theme.roleFeatures("ledger")
                    selectionColor: theme.accentSoft
                    selectedTextColor: theme.text
                    selectByMouse: true
                    onTextChanged: if (root.visible) debounce.restart()
                    onCursorRectangleChanged: textFlick.ensureVisible(cursorRectangle)
                    Keys.onEscapePressed: root.close()
                    Text {
                        visible: !input.text
                        text: "Paste a release list, a table, a post. For example:\nEpisode 7 - Wednesday 28 October 2026\nEpisodes 8 to 15 - TBC in 2027"
                        color: theme.textFaint
                        font: input.font
                    }
                }
            }
        }


        // The draft, one line a note, each with its own remove
        Label {
            text: root.draft.length ? "The schedule you are saving" : (root.mode === 1 && input.text ? "Nothing here names an episode and a date yet" : "Nothing set yet")
            role: "ledger"; color: theme.textFaint
        }
        Column {
            objectName: "schedule-preview"
            width: parent.width
            spacing: theme.space(1)
            Repeater {
                model: root.draft
                delegate: Item {
                    id: noteRow
                    required property var modelData
                    width: parent ? parent.width : 0
                    height: Math.max(eps.height, rm.height)
                    Rectangle {
                        id: dot
                        anchors.verticalCenter: parent.verticalCenter
                        width: theme.space(2); height: width; radius: width / 2
                        color: root.kindColor(root.kindOf(noteRow.modelData))
                    }
                    Label {
                        id: eps
                        anchors.left: dot.right; anchors.leftMargin: theme.space(2)
                        anchors.verticalCenter: parent.verticalCenter
                        text: root.epsText(noteRow.modelData)
                        role: "ui"
                    }
                    Label {
                        anchors.right: rm.left; anchors.rightMargin: theme.space(2)
                        anchors.verticalCenter: parent.verticalCenter
                        text: root.whenOf(noteRow.modelData)
                        role: "ledger"
                        color: root.kindColor(root.kindOf(noteRow.modelData))
                    }
                    Button {
                        id: rm
                        anchors.right: parent.right
                        anchors.verticalCenter: parent.verticalCenter
                        small: true; flat: true; icon: "x"; text: ""
                        onClicked: root.removeNote(noteRow.modelData)
                    }
                }
            }
        }

        Row {
            spacing: theme.space(2)
            Button {
                objectName: "schedule-save"
                primary: true
                text: "Save"
                enabled: root.draft.length > 0 || root.saved.length > 0
                onClicked: root.save(root.draft)
            }
            Button {
                visible: root.saved.length > 0
                objectName: "schedule-clear"
                flat: true
                text: "Clear all"
                onClicked: root.save([])
            }
            Button {
                flat: true
                text: "Cancel"
                onClicked: root.close()
            }
        }
    }
    }
}
