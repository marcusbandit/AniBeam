// The schedule popover (#418): tell a show's schedule what its provider does not say. Paste
// a release list or type one episode a line ("Episode 8 - TBC in 2027", "Episode 4 -
// Wednesday 21 October 2026"); the core reads it by rule as it is typed, the preview shows
// what it read, and Save keeps it for the season the page is about. A raised tile over a
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
    property var read: []
    property var readSeason: null

    // A note as a line the parser reads back the same way, so Change opens on what is saved
    function lineOf(n) {
        var eps = n.first === n.last ? "Episode " + n.first : "Episodes " + n.first + " to " + n.last
        return eps + " - " + root.whenOf(n)
    }
    function whenOf(n) {
        if (n.at !== null && n.at !== undefined) return Qt.formatDate(new Date(n.at * 1000), "dddd d MMMM yyyy")
        if (n.year !== null && n.year !== undefined && n.month) return Qt.formatDate(new Date(n.year, n.month - 1, 1), "MMMM yyyy")
        if (n.year !== null && n.year !== undefined) return "TBC in " + n.year
        return "TBC"
    }
    function openFor(id, title, season, notes) {
        root.seriesId = id
        root.seriesTitle = title
        root.season = season === undefined ? null : season
        root.saved = notes || []
        input.text = root.saved.map(root.lineOf).join("\n")
        root.parse()
        visible = true
        frame.escapeStack.push("popover", root)
        input.forceActiveFocus()
        input.cursorPosition = input.length
    }
    function close() {
        if (!visible) return
        visible = false
        frame.escapeStack.pop(root)
    }
    Component.onDestruction: if (visible) frame.escapeStack.pop(root)
    function parse() {
        var r = Door.call("ParseSchedule", { text: input.text })
        if (r.error) { root.read = []; root.readSeason = null; return }
        root.read = r.reply.notes || []
        root.readSeason = r.reply.season === undefined ? null : r.reply.season
    }
    function save(notes) {
        var wire = notes.map(function(n) {
            return { first: n.first, last: n.last, year: n.year === undefined ? null : n.year, month: n.month === undefined ? null : n.month,
                     at: n.at === null || n.at === undefined ? null : { secs_since_epoch: Math.round(n.at), nanos_since_epoch: 0 } }
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
    visible: false
    color: theme.glassFloat
    borderColor: theme.lineStrong
    borderWidth: 1
    highlight: theme.glassHighlight

    Timer { id: debounce; interval: 120; onTriggered: root.parse() }

    // The panel's own press sink: a press on its padding never reaches the shield under it
    MouseArea { anchors.fill: parent; acceptedButtons: Qt.AllButtons; hoverEnabled: true; onWheel: function(w) { w.accepted = true } }

    Column {
        id: body
        x: root.pad; y: root.pad
        width: root.width - root.pad * 2
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

        Label {
            width: parent.width
            wrapMode: Text.WordWrap
            text: "Paste a release list, or write one episode a line. A date, a year or TBC each work. AniBeam stops guessing at a TBC, and a date the provider lists still wins."
            role: "ui"; color: theme.textDim
        }

        // The text, a pane like a Field that grows to a few lines and then scrolls
        Corner {
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
                        text: "Episode 7 - Wednesday 28 October 2026\nEpisodes 8 to 15 - TBC in 2027"
                        color: theme.textFaint
                        font: input.font
                    }
                }
            }
        }

        // What the core read, one line a note
        Label { text: root.read.length ? "AniBeam read" : (input.text ? "Nothing here names an episode and a date yet" : ""); role: "ledger"; color: theme.textFaint; visible: text !== "" }
        Column {
            objectName: "schedule-preview"
            width: parent.width
            spacing: theme.space(1)
            Repeater {
                model: root.read
                delegate: Item {
                    required property var modelData
                    width: parent ? parent.width : 0
                    height: eps.height
                    Label {
                        id: eps
                        text: modelData.first === modelData.last ? "Episode " + modelData.first : "Episodes " + modelData.first + " to " + modelData.last
                        role: "ui"
                    }
                    Label {
                        anchors.right: parent.right
                        anchors.verticalCenter: eps.verticalCenter
                        text: root.whenOf(modelData)
                        role: "ledger"
                        color: modelData.at !== null && modelData.at !== undefined ? theme.textDim : theme.accent
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
                enabled: root.read.length > 0
                onClicked: root.save(root.read)
            }
            Button {
                visible: root.saved.length > 0
                objectName: "schedule-clear"
                flat: true
                text: "Clear notes"
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
