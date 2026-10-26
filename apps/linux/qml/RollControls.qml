// A rolling download's window and its controls, under a torrent that belongs to one: where
// the window stands ("Episodes 13 to 24 of 170"), a button that fetches one more window
// now, a stepper for how many stay ahead, and Get all, which stops rolling and fetches the
// rest after an inline confirm. Each control calls the core directly; the next torrent
// read brings the new window back in `roll`.
import QtQuick
import com.marcusrosado.AniBeam

Column {
    id: root
    property var roll: null
    readonly property bool on: !!roll
    readonly property int remaining: on ? Math.max(0, roll.total - roll.through) : 0
    property bool confirming: false
    visible: on
    spacing: theme.space(1.5)

    function send(name, args) {
        var r = Door.call(name, args)
        if (r.error) frame.toast(r.error.message)
    }
    function setAhead(n) {
        if (n < 1 || n > 500 || n === root.roll.ahead) return
        send("SetRoll", { id: root.roll.id, ahead: n, on: null, delete_watched: null })
    }

    Label {
        objectName: "roll-window"
        width: parent.width
        text: !root.on ? "" : "Rolling " + root.roll.ahead + " ahead: episodes " + root.roll.next + " to " + root.roll.through + " of " + root.roll.total
              + (root.roll.watched > 0 ? ", " + root.roll.watched + " watched" : "")
        role: "ui"
        color: theme.textDim
        wrapMode: Text.Wrap
    }
    // Wraps under itself in a narrow column rather than running off its edge
    Flow {
        visible: !root.confirming
        width: parent.width
        spacing: theme.space(1.5)
        Button {
            objectName: "roll-more"
            text: "Get " + Math.min(root.on ? root.roll.ahead : 0, root.remaining) + " more"
            icon: "download"
            small: true
            enabled: root.remaining > 0
            onClicked: root.send("RollMore", { id: root.roll.id, count: root.roll.ahead })
            Tooltip { text: "Download the next episodes now. Rolling carries on once you catch up." }
        }
        // The stepper wraps as one piece
        Row {
            spacing: theme.space(1)
            Button { objectName: "roll-fewer"; icon: "chevron-down"; small: true; flat: true; enabled: root.on && root.roll.ahead > 1; onClicked: root.setAhead(root.roll.ahead - 1)
                     Tooltip { text: "Keep one fewer ahead" } }
            Label { height: theme.space(6); verticalAlignment: Text.AlignVCenter; text: root.on ? root.roll.ahead + " ahead" : ""; role: "figure"; color: theme.textDim }
            Button { objectName: "roll-further"; icon: "chevron-up"; small: true; flat: true; enabled: root.on; onClicked: root.setAhead(root.roll.ahead + 1)
                     Tooltip { text: "Keep one more ahead" } }
        }
        Button {
            objectName: "roll-all"
            text: "Get all"
            icon: "layers"
            small: true
            flat: true
            enabled: root.remaining > 0
            onClicked: root.confirming = true
            Tooltip { text: "Stop rolling and download every episode left" }
        }
    }
    // The question over its buttons, so it fits a narrow column
    Label {
        visible: root.confirming
        width: parent.width
        text: "Download all " + root.remaining + " episodes left?"
        role: "ui"
        wrapMode: Text.Wrap
    }
    InlineConfirm {
        objectName: "roll-all-confirm"
        visible: root.confirming
        question: ""
        confirmText: "Get all"
        confirmIcon: "download"
        onAccepted: { root.confirming = false; root.send("SetRoll", { id: root.roll.id, ahead: null, on: false, delete_watched: null }) }
        onKept: root.confirming = false
    }
}
