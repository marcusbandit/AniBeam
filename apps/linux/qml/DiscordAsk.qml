// Someone pressed Ask to Join on this person's Discord profile (#966). One island at the
// top centre per ask, oldest first, clear of the room panel at the right. Start room (or Let in, in a room) lets them in through
// the frame's room, which starts one when there is none; No closes the ask, and the core
// never brings that person here again. The x only puts the island away: unanswered is not
// a no, and Discord lets the ask go after 30 s anyway, which takes the island with it.
import QtQuick
import com.marcusrosado.Anibeam

Frost {
    id: root
    objectName: "discord-ask"
    visible: asks.length > 0
    radius: theme.radiusTile
    readonly property real pad: theme.space(4)
    width: Math.min(parent ? parent.width - theme.space(8) : theme.space(120),
                    Math.max(theme.space(96), who.implicitWidth + face.width + pad * 2 + theme.space(14)))
    height: body.implicitHeight + pad * 2

    // Each: { user_id, name, avatar, in_room }
    property var asks: []
    readonly property var ask: asks.length > 0 ? asks[0] : null

    function add(a) {
        var keep = asks.filter(function(x) { return x.user_id !== a.user_id })
        keep.push(a)
        asks = keep
    }
    function drop(id) { asks = asks.filter(function(x) { return x.user_id !== id }) }
    function answer(allow) {
        var a = ask
        if (!a) return
        if (allow) { if (frame.room.letIn(a.user_id)) drop(a.user_id) }
        else {
            drop(a.user_id)
            var r = Door.call("DiscordAnswer", { user_id: a.user_id, allow: false })
            if (r.error) frame.fail(r.error)
        }
    }

    Column {
        id: body
        x: root.pad; y: root.pad
        width: root.width - root.pad * 2
        spacing: theme.space(3)
        Row {
            spacing: theme.space(3)
            Corner {
                id: face
                width: theme.space(10); height: width
                radius: width / 2
                smoothing: 0
                color: theme.tone(theme.raised, theme.text, 0.12)
                fillItem: art.status === Image.Ready ? art : null
                Image {
                    id: art; visible: false
                    width: parent.width; height: parent.height
                    source: root.ask && root.ask.avatar ? root.ask.avatar : ""
                    sourceSize.width: 128; sourceSize.height: 128
                    fillMode: Image.PreserveAspectCrop
                    asynchronous: true
                }
                Label {
                    anchors.centerIn: parent
                    visible: art.status !== Image.Ready
                    text: root.ask ? String.fromCodePoint(String(root.ask.name || "?").codePointAt(0)).toUpperCase() : ""
                    role: "ui"; font.weight: 600
                }
            }
            Label {
                id: who
                objectName: "discord-ask-text"
                anchors.verticalCenter: face.verticalCenter
                width: Math.min(implicitWidth, body.width - face.width - theme.space(3))
                text: root.ask ? words.t("discord-ask-wants", { name: root.ask.name }) : ""
                role: "ui"
                wrapMode: Text.Wrap
            }
        }
        Flow {
            width: body.width
            spacing: theme.space(2)
            Button {
                objectName: "discord-ask-let-in"
                text: root.ask && root.ask.in_room ? words.t("discord-ask-let-in") : words.t("discord-ask-start-room")
                icon: root.ask && root.ask.in_room ? "user-check" : "users"
                onClicked: root.answer(true)
            }
            Button {
                objectName: "discord-ask-no"
                text: words.t("discord-ask-no")
                flat: true
                onClicked: root.answer(false)
            }
            Button {
                objectName: "discord-ask-later"
                flat: true
                icon: "x"
                onClicked: if (root.ask) root.drop(root.ask.user_id)
                Tooltip { msg: "action-close" }
            }
        }
    }
}
