// Episodes and dates (#418, #432): tell a season what its provider does not say. The grid is
// the one view: every episode is a tile with its date under its number, the last episode is
// marked, and nothing past it is predicted. Pick tiles (shift-click for a run), then one row
// of actions dates them, ends the season there or clears them. Paste a list opens a box above
// the grid whose reading lands on the tiles, to fix by picking. One control height, one
// button style and two text sizes throughout. A raised tile over a Blocker the owning page
// puts under it, on the frame's escape stack like the match panel.
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
    // The draft, in the wire shape: {first, last, at, year, month}
    property var draft: []
    property var readSeason: null
    property bool pasting: false
    property int upTo: 12
    property var aired: []
    property var picked: []
    property int anchorEp: 0
    property bool weekly: false
    // The typed date as the core reads it, or null
    property var typed: null
    // The season's last episode, which is the episode count set by hand; 0 for none
    property int endAt: 0
    property int savedEnd: 0
    // What the record already says, by episode: {at, guess}; guesses are dropped here and
    // made again live from the record's own dates, so the grid shows what Save would give
    property var known: ({})
    property string siteName: ""
    // The record's real dates, without its guesses
    readonly property var recordDates: {
        var out = {}
        for (var k in known) if (!known[k].guess) out[k] = known[k].at
        return out
    }
    // A preview of the weekly guesses cards::plan makes from what is set here: each episode
    // with no date guesses one cadence on from the dated one before it, up to the last
    // episode (or one past the last date when none is set); an episode with no date, a year
    // or a month only stops the run until the next date. Close to what Save gives, not the
    // core's word: plan also moves an overdue guess a week on.
    readonly property var guesses: {
        var out = {}
        var by = expand(draft)
        var dated = {}
        for (var k in recordDates) dated[k] = recordDates[k]
        for (var e in by) if (has(by[e].at)) dated[e] = by[e].at
        var nums = Object.keys(dated).map(Number).sort(function(a, b) { return a - b })
        if (!nums.length) return out
        // The cadence: the first gap between two neighbours, in whole weeks, at least one
        var week = 7 * 86400, step = week
        for (var i = 1; i < nums.length; i++) if (nums[i] === nums[i - 1] + 1) {
            step = Math.max(1, Math.round((dated[nums[i]] - dated[nums[i - 1]]) / week)) * week
            break
        }
        var last = nums[nums.length - 1]
        var to = endAt > 0 ? endAt : last + 1
        var anchor = null
        for (var n = nums[0]; n <= to; n++) {
            if (dated[n] !== undefined) { anchor = { n: n, at: dated[n] }; continue }
            if (by[n]) { anchor = null; continue }
            if (anchor && aired.indexOf(n) < 0) out[n] = anchor.at + (n - anchor.n) * step
        }
        return out
    }

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
    // What a tile says under its number: the date, short
    function tileText(ep) {
        if (root.endAt > 0 && ep > root.endAt) return ""
        var n = root.noteFor(ep)
        if (!n && root.recordDates[ep] !== undefined && root.aired.indexOf(ep) < 0)
            return Qt.formatDate(new Date(root.recordDates[ep] * 1000), "d MMM")
        if (!n && root.guesses[ep] !== undefined)
            return "≈ " + Qt.formatDate(new Date(root.guesses[ep] * 1000), "d MMM")
        if (n) {
            if (has(n.at)) return Qt.formatDate(new Date(n.at * 1000), "d MMM")
            if (has(n.year) && n.month) return Qt.formatDate(new Date(n.year, n.month - 1, 1), "MMM yyyy")
            if (has(n.year)) return String(n.year)
            return "No date"
        }
        if (root.endAt === ep) return "Last"
        return root.aired.indexOf(ep) >= 0 ? "Out" : ""
    }
    // The footer's one line: where the season ends and how much is dated
    readonly property string summary: {
        var dated = 0
        for (var i = 0; i < draft.length; i++) dated += draft[i].last - draft[i].first + 1
        var bits = []
        if (endAt > 0) bits.push("Ends at episode " + endAt)
        var fromRecord = 0
        for (var k in recordDates) if (aired.indexOf(Number(k)) < 0 && !noteFor(Number(k))) fromRecord++
        if (fromRecord > 0) bits.push(fromRecord + " dated by " + (siteName || "the record"))
        if (dated > 0) bits.push(dated + " set by you")
        var guessed = Object.keys(guesses).length
        if (guessed > 0) bits.push(guessed + " guessed weekly")
        return bits.length ? bits.join(" · ") : "Nothing set yet"
    }
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
        dateField.text = ""
        root.typed = null
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

    // Marks the one picked episode as the season's last, or clears the mark when it already is
    function markLast() {
        if (root.picked.length !== 1) return
        root.endAt = root.endAt === root.picked[0] ? 0 : root.picked[0]
        // Nothing past the end keeps a date
        var by = root.expand(root.draft)
        if (root.endAt > 0) for (var e in by) if (Number(e) > root.endAt) delete by[e]
        root.draft = root.collapse(by)
        root.picked = []
    }
    function openFor(id, title, season, notes, upTo, aired, endAt, known, siteName) {
        root.known = known || {}
        root.siteName = siteName || ""
        root.endAt = endAt || 0
        root.savedEnd = root.endAt
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
        root.pasting = false
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
        // A list naming episodes past the marked end moves the end out of their way
        var top = 0
        for (var i = 0; i < root.draft.length; i++) top = Math.max(top, root.draft[i].last)
        if (root.endAt > 0 && top > root.endAt) root.endAt = 0
        root.upTo = Math.min(Math.max(root.upTo, top + 1), 200)
    }
    function save(notes) {
        var wire = notes.map(function(n) {
            return { first: n.first, last: n.last, year: has(n.year) ? n.year : null, month: n.month || null,
                     at: has(n.at) ? { secs_since_epoch: Math.round(n.at), nanos_since_epoch: 0 } : null }
        })
        // Always the season the page is about, the one its count and its tiles belong to
        var r = Door.call("SetScheduleNotes", { series: root.seriesId, season: root.season, notes: wire })
        if (r.error) { frame.fail(r.error); return }
        if (root.endAt !== root.savedEnd) {
            var c = Door.call("SetEpisodeCount", { series: root.seriesId, count: root.endAt > 0 ? root.endAt : null })
            if (c.error) { frame.fail(c.error); return }
        }
        frame.toast(wire.length || root.endAt ? "Saved" : "Schedule notes cleared")
        root.close()
    }


    readonly property real pad: theme.space(6)
    readonly property real gap: theme.space(4)
    readonly property real tileSize: theme.space(16)
    width: Math.min(theme.space(200), (parent ? parent.width : theme.space(200)) - theme.space(16))
    height: Math.min(body.implicitHeight + pad * 2, (parent ? parent.height : theme.space(160)) - theme.space(16))
    visible: false
    color: theme.glassFloat
    borderColor: theme.lineStrong
    borderWidth: 1
    highlight: theme.glassHighlight

    Timer { id: debounce; interval: 150; onTriggered: root.parse() }
    Timer { id: typedDebounce; interval: 120; onTriggered: root.readTyped(dateField.text) }

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
            spacing: root.gap

            // Head: what and whose, with paste and close on the right
            Item {
                width: parent.width
                height: Math.max(heading.height, headActions.height)
                Column {
                    id: heading
                    width: parent.width - headActions.width - theme.space(4)
                    Label { text: "Episodes and dates"; role: "title" }
                    Label {
                        width: parent.width
                        text: root.seriesTitle + (root.season !== null ? " · Season " + root.season : "")
                        role: "ui"; color: theme.textDim; elide: Text.ElideRight
                    }
                }
                Row {
                    id: headActions
                    anchors.right: parent.right
                    anchors.verticalCenter: heading.verticalCenter
                    spacing: theme.space(2)
                    Button {
                        objectName: "schedule-paste-toggle"
                        flat: true
                        icon: root.pasting ? "check" : "copy"
                        text: root.pasting ? "Done pasting" : "Paste a list"
                        onClicked: { root.pasting = !root.pasting; if (root.pasting) input.forceActiveFocus() }
                    }
                    Button {
                        objectName: "schedule-close"
                        flat: true
                        icon: "x"
                        text: ""
                        onClicked: root.close()
                    }
                }
            }

            // Paste: the box, and the one line saying where its reading goes
            Column {
                visible: root.pasting
                width: parent.width
                spacing: theme.space(2)
                Corner {
                    width: parent.width
                    height: theme.space(32)
                    radius: theme.radiusTile
                    smoothing: theme.cornerSmoothing
                    color: input.activeFocus ? theme.raised : theme.tile
                    borderColor: input.activeFocus ? theme.focusRing : theme.line
                    borderWidth: 1
                    Flickable {
                        id: textFlick
                        anchors.fill: parent
                        anchors.margins: theme.space(3)
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
                            font.pointSize: theme.typeNormal
                            font.weight: theme.roleWeight("ui")
                            font.variableAxes: theme.roleAxes("ui")
                            selectionColor: theme.accentSoft
                            selectedTextColor: theme.text
                            selectByMouse: true
                            onTextChanged: if (root.visible) debounce.restart()
                            onCursorRectangleChanged: textFlick.ensureVisible(cursorRectangle)
                            Keys.onEscapePressed: root.pasting = false
                            Text {
                                visible: !input.text
                                text: "Paste a release list, a table or a post"
                                color: theme.textFaint
                                font: input.font
                            }
                        }
                    }
                }
                Label {
                    text: "What AniBeam reads shows on the episodes below."
                    role: "ui"; color: theme.textDim
                }
            }

            // The episodes: one tile each, its date under its number
            Flow {
                id: cells
                objectName: "schedule-cells"
                width: parent.width
                spacing: theme.space(2)
                Repeater {
                    model: root.upTo
                    delegate: Corner {
                        id: cell
                        required property int index
                        readonly property int ep: index + 1
                        readonly property bool isPicked: root.picked.indexOf(ep) >= 0
                        readonly property bool isOut: root.aired.indexOf(ep) >= 0
                        readonly property string kind: root.kindOf(root.noteFor(ep))
                        readonly property bool isEnd: root.endAt === ep
                        readonly property bool past: root.endAt > 0 && ep > root.endAt
                        objectName: "schedule-ep-" + ep
                        width: root.tileSize; height: root.tileSize
                        radius: theme.radiusInner(theme.space(1))
                        color: isPicked ? theme.accent : cellMouse.containsMouse ? theme.raisedHover : theme.raised
                        borderColor: isPicked ? theme.accent : kind && !past ? root.kindColor(kind) : theme.line
                        borderWidth: kind && !isPicked && !past ? 2 : 1
                        opacity: past && !isPicked ? 0.4 : 1
                        Column {
                            anchors.centerIn: parent
                            spacing: theme.space(0.5)
                            Label {
                                anchors.horizontalCenter: parent.horizontalCenter
                                text: String(cell.ep)
                                role: "ui"
                                font.weight: 650
                                color: cell.isPicked ? theme.accentText : cell.isOut && !cell.kind ? theme.textFaint : theme.text
                            }
                            Label {
                                anchors.horizontalCenter: parent.horizontalCenter
                                text: root.tileText(cell.ep) || " "
                                role: "ledger"
                                // A date the user set takes its kind's colour, the record's own date the
                                // text colour, a guess and Out stay faint
                                color: cell.isPicked ? theme.accentText
                                     : cell.kind && !cell.past ? root.kindColor(cell.kind)
                                     : root.recordDates[cell.ep] !== undefined && !cell.isOut ? theme.text
                                     : theme.textFaint
                            }
                        }
                        // The last episode: a bar along its foot
                        Rectangle {
                            visible: cell.isEnd
                            anchors.bottom: parent.bottom; anchors.bottomMargin: theme.space(1.5)
                            anchors.horizontalCenter: parent.horizontalCenter
                            width: parent.width * 0.5; height: theme.space(0.75); radius: height / 2
                            color: cell.isPicked ? theme.accentText : theme.text
                        }
                        MouseArea {
                            Accessible.role: Accessible.Button
                            Accessible.name: theme.nameNear(parent)
                            Accessible.onPressAction: root.clickEp(cell.ep, false)
                            id: cellMouse
                            anchors.fill: parent
                            hoverEnabled: true
                            cursorShape: Qt.PointingHandCursor
                            onClicked: function(m) { root.clickEp(cell.ep, (m.modifiers & Qt.ShiftModifier) !== 0) }
                        }
                    }
                }
                // One more row of tiles, for a season longer than the page knows
                Corner {
                    objectName: "schedule-more"
                    width: root.tileSize; height: root.tileSize
                    radius: theme.radiusInner(theme.space(1))
                    color: moreMouse.containsMouse ? theme.raisedHover : "transparent"
                    borderColor: theme.line; borderWidth: 1; dashed: 1
                    Icon { anchors.centerIn: parent; glyph: "plus"; size: theme.space(5); color: theme.textDim }
                    MouseArea { id: moreMouse; anchors.fill: parent; hoverEnabled: true; cursorShape: Qt.PointingHandCursor; onClicked: root.upTo = Math.min(root.upTo + 12, 200); Accessible.role: Accessible.Button; Accessible.name: "Show more episodes"
                        Accessible.onPressAction: root.upTo = Math.min(root.upTo + 12, 200) }
                }
            }

            // Nothing picked: one line on how to start, at the row's own height
            Label {
                visible: root.picked.length === 0
                height: theme.controlHeight
                verticalAlignment: Text.AlignVCenter
                text: "Click an episode to date it. Shift-click picks a run."
                role: "ui"; color: theme.textDim
            }
            // Something picked: the one row of actions for it
            Flow {
                visible: root.picked.length > 0
                width: parent.width
                spacing: theme.space(2)
                Label {
                    height: theme.controlHeight
                    verticalAlignment: Text.AlignVCenter
                    text: (root.picked.length === 1 ? "Episode " + root.picked[0] : Fmt.plural(root.picked.length, "episode", "episodes")) + ":"
                    role: "ui"
                }
                Field {
                    id: dateField
                    objectName: "schedule-date"
                    width: theme.space(44)
                    placeholder: "7 Jan, March 2027, 2027"
                    onTextChanged: typedDebounce.restart()
                    Keys.onReturnPressed: root.apply(root.typed)
                    Keys.onEnterPressed: root.apply(root.typed)
                }
                Button {
                    objectName: "schedule-set"
                    text: "Set date"
                    enabled: root.picked.length > 0 && root.typed !== null
                    onClicked: root.apply(root.typed)
                }
                Button {
                    objectName: "schedule-tbc"
                    text: "No date yet"
                    enabled: root.picked.length > 0
                    onClicked: root.apply({ at: null, year: null, month: null })
                }
                Button {
                    objectName: "schedule-weekly"
                    visible: root.picked.length > 1
                    text: "Weekly"
                    icon: root.weekly ? "check" : ""
                    flat: !root.weekly
                    onClicked: root.weekly = !root.weekly
                }
                Button {
                    objectName: "schedule-last"
                    text: root.picked.length === 1 && root.endAt === root.picked[0] ? "Not the last" : "Last episode"
                    visible: root.picked.length === 1
                    onClicked: root.markLast()
                }
                Button {
                    objectName: "schedule-clear-picked"
                    flat: true
                    text: "Clear date"
                    visible: root.picked.some(function(e) { return root.noteFor(e) !== null })
                    onClicked: root.unset()
                }
            }

            // A hairline, then the summary and the way out
            Rectangle { width: parent.width; height: 1; color: theme.line }
            Item {
                width: parent.width
                height: theme.controlHeight
                Label {
                    anchors.verticalCenter: parent.verticalCenter
                    text: root.summary
                    role: "ui"; color: theme.textDim
                }
                Row {
                    anchors.right: parent.right
                    spacing: theme.space(2)
                    Button { flat: true; text: "Cancel"; onClicked: root.close() }
                    Button {
                        objectName: "schedule-save"
                        primary: true
                        text: "Save"
                        enabled: JSON.stringify(root.draft) !== JSON.stringify(root.collapse(root.expand(root.saved))) || root.endAt !== root.savedEnd
                        onClicked: root.save(root.draft)
                    }
                }
            }
        }
    }
}
