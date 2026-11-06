// Notes on a series' list entry (#211): one text box, written to every connected tracker as
// AniList's notes and MAL's comments. A raised tile over a Blocker the owning page puts under
// it, on the frame's escape stack like the schedule sheet.
import QtQuick
import com.marcusrosado.AniBeam

Corner {
    id: root
    objectName: "notes-sheet"
    property real seriesId: 0
    property string seriesTitle: ""
    property string saved: ""
    // Where the notes go, as a person names the sites
    property string sites: ""
    // The core's one limit for both trackers, so a note never fits one and fails on the other
    readonly property int maxChars: 2000
    // Characters as the core counts them: a pair of UTF-16 halves, an emoji say, is one
    readonly property int count: input.text.trim().replace(/[\uD800-\uDBFF][\uDC00-\uDFFF]/g, "_").length

    function openFor(id, title, notes, sites) {
        root.seriesId = id
        root.seriesTitle = title
        root.saved = notes || ""
        root.sites = sites || "your tracker"
        input.text = root.saved
        visible = true
        frame.escapeStack.push("popover", root)
        input.forceActiveFocus()
        input.cursorPosition = input.text.length
    }
    function close() {
        if (!visible) return
        visible = false
        frame.escapeStack.pop(root)
    }
    Component.onDestruction: if (visible) frame.escapeStack.pop(root)
    function save() {
        var r = Door.call("SetSeriesNotes", { series: root.seriesId, notes: input.text })
        if (r.error) { frame.fail(r.error); return }
        root.close()
    }

    readonly property real pad: theme.space(6)
    width: Math.min(theme.space(150), (parent ? parent.width : theme.space(150)) - theme.space(16))
    height: Math.min(body.implicitHeight + pad * 2, (parent ? parent.height : theme.space(120)) - theme.space(16))
    visible: false
    color: theme.glassFloat
    borderColor: theme.lineStrong
    borderWidth: 1
    highlight: theme.glassHighlight

    // The panel's own press sink: a press on its padding never reaches the shield under it
    MouseArea { anchors.fill: parent; acceptedButtons: Qt.AllButtons; hoverEnabled: true; onWheel: function(w) { w.accepted = true } }

    Column {
        id: body
        x: root.pad; y: root.pad
        width: root.width - root.pad * 2
        spacing: theme.space(4)

        Item {
            width: parent.width
            height: Math.max(heading.height, closeButton.height)
            Column {
                id: heading
                width: parent.width - closeButton.width - theme.space(4)
                Label { text: "Notes"; role: "title" }
                Label {
                    width: parent.width
                    text: root.seriesTitle
                    role: "ui"; color: theme.textDim; elide: Text.ElideRight
                }
            }
            Button {
                id: closeButton
                objectName: "notes-close"
                anchors.right: parent.right
                anchors.verticalCenter: heading.verticalCenter
                flat: true
                icon: "x"
                text: ""
                onClicked: root.close()
            }
        }

        Corner {
            width: parent.width
            height: theme.space(48)
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
                    objectName: "notes-input"
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
                    onCursorRectangleChanged: textFlick.ensureVisible(cursorRectangle)
                    Keys.onEscapePressed: root.close()
                    Text {
                        visible: !input.text
                        text: "Where you left off, who recommended it, which release to get"
                        color: theme.textFaint
                        font: input.font
                    }
                }
            }
        }

        Rectangle { width: parent.width; height: 1; color: theme.line }
        Item {
            width: parent.width
            height: theme.controlHeight
            Label {
                anchors.verticalCenter: parent.verticalCenter
                width: parent.width - actions.width - theme.space(4)
                elide: Text.ElideRight
                text: root.count > root.maxChars
                      ? (root.count - root.maxChars) + " characters over the limit"
                      : "Saved to " + root.sites
                role: "ui"
                color: root.count > root.maxChars ? theme.red : theme.textDim
            }
            Row {
                id: actions
                anchors.right: parent.right
                spacing: theme.space(2)
                Button { flat: true; text: "Cancel"; onClicked: root.close() }
                Button {
                    objectName: "notes-save"
                    primary: true
                    text: "Save"
                    enabled: input.text.trim() !== root.saved.trim() && root.count <= root.maxChars
                    onClicked: root.save()
                }
            }
        }
    }
}
