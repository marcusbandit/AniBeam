// Moving the list kept on this device up to a tracker (#580). It opens by itself once a
// tracker has connected and its list has been read, when there is anything to move, and
// from Settings, Accounts after that. It shows every show it would change, what is kept
// here beside what the tracker has now, and moves nothing until Move is pressed. Not now
// leaves the list here and the pointer in Settings.
import QtQuick
import com.marcusrosado.AniBeam

Item {
    id: root
    objectName: "move-lists-sheet"
    anchors.fill: parent
    visible: false
    property string tracker: "Anilist"
    property var rows: []
    readonly property string site: frame.trackerName(tracker)
    readonly property real pad: theme.space(6)

    // The preview for a tracker, and the sheet open when it has anything to move. Answers
    // whether it opened.
    function openFor(t) {
        var r = Door.call("PreviewListMove", { tracker: t })
        if (r.error || !r.reply || !r.reply.rows || !r.reply.rows.length) return false
        root.tracker = t
        root.rows = r.reply.rows
        root.visible = true
        frame.escapeStack.push("sheet", root)
        return true
    }
    function close() {
        if (!root.visible) return
        root.visible = false
        frame.escapeStack.pop(root)
    }
    Component.onDestruction: if (visible) frame.escapeStack.pop(root)
    function move() {
        var ids = root.rows.map(function(row) { return row.series })
        var r = Door.call("MoveLists", { tracker: root.tracker, series: ids })
        if (r.error) { frame.toast(frame.errorText(r.error)); return }
        frame.toast("Moving " + (ids.length === 1 ? "1 show" : ids.length + " shows") + " to " + root.site)
        root.close()
    }
    function statusText(s) {
        var names = { Watching: "Watching", Planning: "Planning", Completed: "Completed", Paused: "On hold", Dropped: "Dropped", Repeating: "Rewatching" }
        return s ? (names[s] || s) : ""
    }
    // One side of a row: the status and the score, or a dash of words for nothing
    function sideText(status, score, empty) {
        var bits = []
        if (status) bits.push(root.statusText(status))
        if (score !== null && score !== undefined) bits.push("score " + Fmt.score(score))
        return bits.length ? bits.join(", ") : empty
    }

    // The shield: a press outside closes it, like the other sheets
    Blocker { anchors.fill: parent; dismissOnPress: true; onDismissed: root.close() }

    Corner {
        id: panel
        anchors.centerIn: parent
        width: Math.min(theme.space(170), root.width - theme.space(16))
        height: Math.min(body.implicitHeight + root.pad * 2, root.height - theme.space(16))
        color: theme.glassFloat
        borderColor: theme.lineStrong
        borderWidth: 1
        highlight: theme.glassHighlight
        MouseArea { anchors.fill: parent; acceptedButtons: Qt.AllButtons; hoverEnabled: true; onWheel: function(w) { w.accepted = true } }

        Column {
            id: body
            x: root.pad; y: root.pad
            width: panel.width - root.pad * 2
            spacing: theme.space(4)

            Label { text: "Move your list to " + root.site; role: "title" }
            Label {
                width: parent.width
                wrapMode: Text.Wrap
                role: "ui"; color: theme.textDim
                text: (root.rows.length === 1 ? "One show has" : root.rows.length + " shows have") +
                      " a status or a score only on this computer. Moving them puts them on your " +
                      root.site + " list, where your phone and your profile see them. For these shows, what " +
                      root.site + " has now is replaced. Nothing is sent until you press Move."
            }

            // The shows, with a header row naming the two sides
            Item {
                width: parent.width
                height: Math.min(list.contentHeight, theme.space(90)) + headRow.height
                Row {
                    id: headRow
                    width: parent.width
                    height: theme.space(7)
                    Label { width: parent.width * 0.44; text: "Show"; role: "ui"; color: theme.textFaint }
                    Label { width: parent.width * 0.28; text: "On this computer"; role: "ui"; color: theme.textFaint }
                    Label { width: parent.width * 0.28; text: "On " + root.site + " now"; role: "ui"; color: theme.textFaint }
                }
                ListView {
                    id: list
                    objectName: "move-lists-rows"
                    anchors.top: headRow.bottom
                    width: parent.width
                    height: parent.height - headRow.height
                    clip: true
                    boundsBehavior: Flickable.StopAtBounds
                    model: root.rows
                    WheelStep { flick: list }
                    delegate: Item {
                        width: list.width
                        height: theme.space(9)
                        Rectangle { anchors.bottom: parent.bottom; width: parent.width; height: 1; color: theme.line }
                        Label { width: parent.width * 0.44 - theme.space(2); anchors.verticalCenter: parent.verticalCenter; text: modelData.title; role: "ui"; elide: Text.ElideRight }
                        Label { x: parent.width * 0.44; width: parent.width * 0.28 - theme.space(2); anchors.verticalCenter: parent.verticalCenter
                                text: root.sideText(modelData.status, modelData.score, "Nothing"); role: "ui"; elide: Text.ElideRight }
                        Label { x: parent.width * 0.72; width: parent.width * 0.28; anchors.verticalCenter: parent.verticalCenter
                                text: root.sideText(modelData.tracker_status, modelData.tracker_score, "Not on your list"); role: "ui"; color: theme.textDim; elide: Text.ElideRight }
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
                    text: "Later: Settings, Accounts"
                    role: "ui"; color: theme.textDim
                }
                Row {
                    id: actions
                    anchors.right: parent.right
                    spacing: theme.space(2)
                    Button { objectName: "move-lists-later"; flat: true; text: "Not now"; onClicked: root.close() }
                    Button {
                        objectName: "move-lists-move"
                        primary: true
                        text: root.rows.length === 1 ? "Move 1 show" : "Move " + root.rows.length + " shows"
                        onClicked: root.move()
                    }
                }
            }
        }
    }
}
