// Watch together (#922): the host's ready gate. The host pressed play while someone in the
// room lacks the episode or is still opening it, so the room holds paused and the host
// chooses: start anyway, or wait for everyone and start on its own once they are ready. How
// many are ready stands as the heading, with a bar under it. The room panel shows it above
// the people, and the player shows it over the picture while the panel is closed.
import QtQuick
import com.marcusrosado.Anibeam

Column {
    id: root
    objectName: "room-gate"
    readonly property var room: frame.room
    readonly property var info: room.info
    readonly property string gate: info ? String(info.gate || "Off") : "Off"
    readonly property bool shown: room.host && gate !== "Off"
    readonly property int total: info ? info.members.length : 0
    readonly property int ready: {
        if (!info) return 0
        var n = 0
        for (var i = 0; i < info.members.length; i++) if (info.members[i].ready === "Ready") n++
        return n
    }
    readonly property real fraction: total > 0 ? ready / total : 0
    // The player's buttons never take the keyboard: Space and the arrows stay the player's
    property bool takesFocus: true
    // The width the heading and both buttons need side by side, whatever the language: a card
    // sizes to it, and a narrower column puts the second button under the first
    readonly property real needed: Math.max(count.implicitWidth, start.implicitWidth + buttons.spacing + wait.implicitWidth)
    visible: shown
    spacing: theme.space(3)

    Label {
        id: count
        objectName: "room-gate-count"
        width: parent.width
        text: words.t("room-gate-ready", { ready: root.ready, total: root.total, percent: Math.round(root.fraction * 100) })
        role: "title"; font.pointSize: theme.typeNormal
        elide: Text.ElideRight
    }
    Rectangle {
        objectName: "room-gate-bar"
        width: parent.width; height: 4; radius: 2
        color: theme.line
        Rectangle {
            width: Math.max(root.fraction > 0 ? height : 0, parent.width * root.fraction)
            height: parent.height; radius: parent.radius
            color: theme.accent
            Behavior on width { enabled: !theme.reducedMotion; NumberAnimation { duration: theme.motionNormal; easing.type: Easing.OutCubic } }
        }
    }
    Flow {
        id: buttons
        width: parent.width
        spacing: theme.space(2)
        Button {
            id: start
            objectName: "room-gate-start"
            primary: true; small: true
            icon: "play"
            msg: "room-gate-start"
            takesFocus: root.takesFocus
            onClicked: root.room.startAnyway()
        }
        Button {
            id: wait
            objectName: "room-gate-wait"
            small: true
            icon: "clock"
            msg: root.gate === "Waiting" ? "room-gate-waiting" : "room-gate-wait"
            busy: root.gate === "Waiting"
            takesFocus: root.takesFocus
            onClicked: root.room.waitForEveryone()
        }
    }
}
