// Watch together (#846): the room panel. In the player it drops from the Watch together tile
// at the top right and holds the chrome while it is up; anywhere else it stands in the middle
// of the window over a scrim (Ctrl+J, the rail). One panel, two states:
//
// - Not in a room: Start a room (in the player, for the episode playing), and a code to join
//   one. A name comes first when there is none, asked here, once.
// - In a room: the code large, the QR code and the link with Copy, who is in it and whether
//   each has the episode open, the offset with its two steps and the openings' suggestion, the
//   connection, and Leave.
//
// The room itself is the frame's (Room.qml); this only shows it and calls into it.
import QtQuick
import com.marcusrosado.AniBeam

Item {
    id: root
    objectName: "room-panel"
    // The player page this opens over, or null for the frame's own
    property Item player: null
    readonly property bool inPlayer: player !== null
    readonly property var room: frame.room
    readonly property var info: room.info
    property bool open: false
    anchors.fill: parent
    visible: open
    z: 900

    property var anchorItem: null
    function place() {
        if (!inPlayer || !anchorItem) {
            panel.x = Math.round((root.width - panel.width) / 2)
            panel.y = Math.max(theme.space(4), Math.round((root.height - panel.height) / 2))
            return
        }
        var p = anchorItem.mapToItem(root, anchorItem.width, anchorItem.height)
        panel.x = Math.max(theme.space(2), Math.min(p.x - panel.width, root.width - panel.width - theme.space(2)))
        panel.y = Math.min(p.y + theme.space(2), Math.max(theme.space(2), root.height - panel.height - theme.space(2)))
    }
    onWidthChanged: if (open) Qt.callLater(root.place)
    onHeightChanged: if (open) Qt.callLater(root.place)
    // The panel changes size as the room does (a name asked, people joining): it stays put
    // against its anchor rather than drifting
    Connections { target: panel; function onWidthChanged() { if (root.open) root.place() } function onHeightChanged() { if (root.open) root.place() } }

    function openAt(anchor) {
        if (open) return
        anchorItem = anchor || null
        if (room.waiting === null) room.error = ""
        editingName = room.needsName && room.waiting !== null
        nameField.text = room.name !== "" ? room.name : room.suggestedName
        open = true
        place()
        if (inPlayer) player.openMenus++
        frame.escapeStack.push(inPlayer ? "popover" : "sheet", root)
        // Outside the player the code is what this was opened for; inside it Space and the
        // arrows stay the player's until a field is clicked
        if (!inPlayer && !room.inRoom) Qt.callLater(function() { if (root.asking) nameField.focusInput(); else codeField.focusInput() })
    }
    function close() {
        if (!open) return
        open = false
        room.cancelWaiting()
        editingName = false
        frame.escapeStack.pop(root)
        if (inPlayer) { player.openMenus--; player.showChrome() }
        else frame.focusPage(root, null)
    }
    Component.onDestruction: root.close()

    // The name: asked when an action waits on it, or when the person asks to change it
    property bool editingName: false
    readonly property bool asking: !room.inRoom && (editingName || room.waiting !== null)
    function saveName() {
        if (room.setName(nameField.text)) editingName = false
    }
    function joinTyped() { room.join(codeField.text) }

    readonly property string offsetText: info ? signed(info.offset) : ""
    function signed(v) {
        var r = Math.round(v * 10) / 10
        return words.t("room-panel-seconds", { value: (r > 0 ? "+" : "") + r.toFixed(1) })
    }
    readonly property string episodeLine: {
        var e = info ? info.episode : null
        if (!e) return ""
        var ep = e.episode_title ? e.episode_title : (e.number !== null && e.number !== undefined ? words.episodeName("", e.number) : e.episode_key)
        return words.t("room-panel-watching", { series: e.series_title, episode: ep })
    }
    // How each member's state shows on the right of their row
    readonly property var stateLook: ({
        Ready: { icon: "circle-check", color: theme.green, msg: "room-panel-ready", spin: false },
        Loading: { icon: "loader-circle", color: theme.textDim, msg: "room-panel-opening", spin: true },
        Missing: { icon: "circle-alert", color: theme.yellow, msg: "room-panel-missing", spin: false },
        Downloading: { icon: "download", color: theme.accent, msg: "room-panel-downloading", spin: false },
        Asking: { icon: "loader-circle", color: theme.textDim, msg: "room-panel-asking", spin: true },
        Searching: { icon: "loader-circle", color: theme.textDim, msg: "room-panel-searching", spin: true },
        Failed: { icon: "circle-x", color: theme.yellow, msg: "room-panel-failed", spin: false }
    })
    // The first letter of a name, for its round avatar
    function initial(name) {
        var s = String(name || "").trim()
        if (s === "") return "?" // i18n: none
        var c = s.codePointAt(0)
        return String.fromCodePoint(c).toUpperCase()
    }

    Blocker { anchors.fill: parent; scrim: !root.inPlayer; dismissOnPress: true; onDismissed: root.close() }
    Corner {
        id: panel
        objectName: "room-panel-box"
        readonly property real pad: theme.space(5)
        width: Math.min(root.width - theme.space(4), content.implicitWidth + pad * 2)
        height: content.implicitHeight + pad * 2
        color: theme.glassFloat; borderColor: theme.lineStrong; borderWidth: 1; highlight: theme.glassHighlight
        MouseArea { anchors.fill: parent; acceptedButtons: Qt.AllButtons }

        Column {
            id: content
            x: panel.pad; y: panel.pad
            spacing: theme.space(4)
            // The two states share the head; the widest state sets the panel's width
            readonly property real wide: root.info ? inside.implicitWidth : theme.space(100)

            // ---- Head: the feature's name, the connection once in a room, and a way out
            Item {
                width: content.wide
                height: theme.space(9)
                Row {
                    anchors.verticalCenter: parent.verticalCenter
                    spacing: theme.space(2.5)
                    Icon { anchors.verticalCenter: parent.verticalCenter; glyph: "users"; size: theme.space(5); color: theme.accent }
                    Label { anchors.verticalCenter: parent.verticalCenter; msg: "room-panel-title"; role: "title" }
                }
                Row {
                    anchors.right: parent.right; anchors.verticalCenter: parent.verticalCenter
                    spacing: theme.space(2)
                    Chip {
                        objectName: "room-panel-connection"
                        anchors.verticalCenter: parent.verticalCenter
                        visible: root.info !== null
                        small: true; mono: false
                        icon: root.info && root.info.connected ? "circle-check" : "loader-circle"
                        textColor: root.info && root.info.connected ? theme.green : theme.yellow
                        msg: root.info && root.info.connected ? "room-panel-connected" : "room-panel-reconnecting"
                    }
                    PlayerButton { objectName: "room-panel-close"; anchors.verticalCenter: parent.verticalCenter; glyph: "x"; tip: words.t("room-panel-close"); onClicked: root.close() }
                }
            }

            // ---- The name, asked once
            Column {
                id: nameAsk
                objectName: "room-panel-name"
                visible: root.asking
                width: content.wide
                spacing: theme.space(2)
                Label { msg: "room-panel-name-label"; role: "ui"; color: theme.textDim; font.pointSize: theme.typeSmall }
                Row {
                    spacing: theme.space(2)
                    Field {
                        id: nameField
                        objectName: "room-panel-name-field"
                        width: content.wide - nameSave.width - parent.spacing
                        placeholder: words.t("room-panel-name-placeholder")
                        accessibleName: words.t("room-panel-name-label")
                        Keys.onReturnPressed: root.saveName()
                        Keys.onEnterPressed: root.saveName()
                    }
                    Button {
                        id: nameSave
                        objectName: "room-panel-name-save"
                        primary: true
                        icon: "arrow-right"
                        msg: room.waiting !== null && room.waiting.kind === "create" ? "room-panel-name-start"
                           : room.waiting !== null ? "room-panel-name-join" : "action-save"
                        takesFocus: !root.inPlayer
                        onClicked: root.saveName()
                    }
                }
            }

            // ---- Not in a room: start one, or join one by its code
            Column {
                id: outside
                visible: !root.info && !root.asking
                width: content.wide
                spacing: theme.space(4)
                Button {
                    objectName: "room-panel-start"
                    visible: root.inPlayer
                    width: parent.width
                    primary: true
                    icon: "users"
                    msg: "room-panel-start"
                    busy: room.busy === "create"
                    enabled: room.busy !== "join" && !!(root.player && root.player.session && !root.player.closed)
                    takesFocus: false
                    onClicked: room.create(root.player.session.session)
                }
                Column {
                    width: parent.width
                    spacing: theme.space(2)
                    Label { msg: root.inPlayer ? "room-panel-or-join" : "room-panel-join-label"; role: "ui"; color: theme.textDim; font.pointSize: theme.typeSmall }
                    Row {
                        spacing: theme.space(2)
                        CodeInput {
                            id: codeField
                            objectName: "room-panel-code"
                            width: parent.parent.width - joinButton.width - parent.spacing
                            accessibleName: words.t("room-panel-join-label")
                            invalid: room.error !== "" && room.busy === ""
                            onFilled: root.joinTyped()
                            Keys.onReturnPressed: root.joinTyped()
                            Keys.onEnterPressed: root.joinTyped()
                        }
                        Button {
                            id: joinButton
                            anchors.verticalCenter: parent.verticalCenter
                            objectName: "room-panel-join"
                            primary: !root.inPlayer
                            icon: "log-in"
                            msg: "room-panel-join"
                            busy: room.busy === "join"
                            enabled: room.busy !== "create"
                            takesFocus: !root.inPlayer
                            onClicked: root.joinTyped()
                        }
                    }
                }
                // Who the room will say is here, and the way to change it
                Row {
                    visible: !room.needsName
                    spacing: theme.space(2)
                    Label { anchors.verticalCenter: parent.verticalCenter; text: words.t("room-panel-you-are", { name: room.name }); role: "ui"; color: theme.textDim; font.pointSize: theme.typeSmall }
                    Button { objectName: "room-panel-change-name"; anchors.verticalCenter: parent.verticalCenter; msg: "room-panel-change-name"; flat: true; small: true; takesFocus: !root.inPlayer
                        onClicked: { root.editingName = true; nameField.text = room.name; nameField.focusInput() } }
                }
            }

            ErrorLine {
                objectName: "room-panel-error"
                width: content.wide
                text: room.error
            }

            // ---- In a room
            Row {
                id: inside
                visible: root.info !== null
                spacing: theme.space(6)
                // The code, the QR code and the link: what someone else needs to come in
                Column {
                    id: share
                    width: theme.space(60)
                    spacing: theme.space(3)
                    Column {
                        spacing: theme.space(0.5)
                        Label { msg: "room-panel-code-label"; role: "ui"; color: theme.textDim; font.pointSize: theme.typeSmall }
                        Label {
                            objectName: "room-panel-code-text"
                            text: root.info ? root.info.code : ""
                            role: "ledger"
                            font.pointSize: theme.typeHeading
                            font.weight: 600
                            font.letterSpacing: theme.space(1)
                        }
                    }
                    // The QR code on its own white card, so a phone camera reads it on any theme
                    Rectangle {
                        objectName: "room-panel-qr"
                        width: share.width; height: width
                        radius: theme.radiusMd
                        color: "#ffffff"
                        Image {
                            anchors.fill: parent
                            anchors.margins: theme.space(1)
                            source: root.info ? Shell.qrCode(root.info.link) : ""
                            sourceSize: Qt.size(Math.round(width * 2), Math.round(height * 2))
                            fillMode: Image.PreserveAspectFit
                            smooth: false
                        }
                    }
                    // The link in full, since it is what gets pasted to a friend
                    Label {
                        objectName: "room-panel-link"
                        width: share.width
                        text: root.info ? root.info.link : ""
                        role: "ledger"; color: theme.textDim
                        wrapMode: Text.WrapAnywhere
                        maximumLineCount: 2
                        elide: Text.ElideRight
                    }
                    Button {
                        objectName: "room-panel-copy"
                        msg: "room-panel-copy"; icon: "copy"; small: true; takesFocus: false
                        onClicked: frame.copyText(root.info.link, words.t("room-panel-link-copied"))
                    }
                }
                // Who is here, the offset, and Leave
                Item {
                    id: peopleBox
                    width: people.width
                    // As tall as the code and the QR code beside it, Leave at the foot
                    implicitHeight: Math.max(share.implicitHeight, people.implicitHeight + theme.space(4) + leaveButton.height)
                    height: implicitHeight
                    Column {
                        id: people
                        width: theme.space(76)
                        spacing: theme.space(3)
                        Label {
                            objectName: "room-panel-episode"
                            width: parent.width
                            text: root.episodeLine
                            role: "title"; font.pointSize: theme.typeNormal
                            wrapMode: Text.Wrap; maximumLineCount: 2; elide: Text.ElideRight
                        }
                        Button {
                            objectName: "room-panel-open-episode"
                            visible: !root.inPlayer && !!room.episode && room.episode.file >= 0
                            msg: "room-panel-open-episode"; icon: "circle-play"; small: true
                            onClicked: { root.close(); room.openEpisode() }
                        }
                        Label { msg: "room-panel-people"; msgArgs: ({ count: root.info ? root.info.members.length : 0 }); role: "ui"; color: theme.textDim; font.pointSize: theme.typeSmall }
                        // Who is here: a round initial, the name with the host's crown, and
                        // where each one is on the right. Someone fetching the episode gets a
                        // thin line under their name with the time left; your own row carries
                        // Get media while you lack it.
                        Column {
                            objectName: "room-panel-members"
                            width: parent.width
                            spacing: theme.space(1)
                            Repeater {
                                model: root.info ? root.info.members : []
                                Item {
                                    id: member
                                    required property var modelData
                                    readonly property var m: modelData
                                    readonly property bool me: root.info !== null && m.id === root.info.you
                                    readonly property string fetch: me && root.info ? String(root.info.fetch || "Idle") : "Idle"
                                    readonly property bool downloading: m.ready === "Downloading"
                                    readonly property bool offer: me && root.info.can_fetch === true
                                    readonly property real percent: downloading ? Math.max(0, Math.min(100, Number(m.progress || 0))) : 0
                                    // Your own fetch says more than the relay's echo of it
                                    readonly property string shown: me && (fetch === "Asking" || fetch === "Searching") ? fetch
                                                                  : me && fetch === "Failed" && !offer ? "Failed"
                                                                  : String(m.ready)
                                    readonly property var look: root.stateLook[shown] || root.stateLook.Loading
                                    objectName: "room-panel-member-" + m.id
                                    width: people.width
                                    height: theme.space(10) + (downloading ? theme.space(4) : 0)

                                    Rectangle {
                                        id: avatar
                                        y: theme.space(1)
                                        width: theme.space(8); height: width; radius: width / 2
                                        color: member.m.host ? theme.tone(theme.raised, theme.accent, 0.28) : theme.tone(theme.raised, theme.text, 0.12)
                                        border.width: member.me ? 1 : 0
                                        border.color: theme.lineStrong
                                        Label {
                                            anchors.centerIn: parent
                                            text: root.initial(member.m.name)
                                            role: "ui"; font.weight: 600
                                            color: member.m.host ? theme.accent : theme.text
                                        }
                                    }
                                    Row {
                                        id: nameRow
                                        anchors.left: avatar.right; anchors.leftMargin: theme.space(3)
                                        anchors.verticalCenter: avatar.verticalCenter
                                        spacing: theme.space(1.5)
                                        readonly property real budget: (member.offer ? getButton.x : status.x) - x - theme.space(3)
                                        Label {
                                            id: nameText
                                            anchors.verticalCenter: parent.verticalCenter
                                            width: Math.min(implicitWidth, Math.max(theme.space(10), nameRow.budget - (crown.visible ? crown.width + nameRow.spacing : 0) - (youText.visible ? youText.implicitWidth + nameRow.spacing : 0)))
                                            text: member.m.name
                                            role: "ui"
                                            elide: Text.ElideRight
                                        }
                                        Icon {
                                            id: crown
                                            objectName: "room-panel-crown"
                                            anchors.verticalCenter: parent.verticalCenter
                                            visible: member.m.host
                                            glyph: "crown"; size: theme.space(3.5); color: theme.accent
                                        }
                                        Label {
                                            id: youText
                                            anchors.verticalCenter: parent.verticalCenter
                                            visible: member.me
                                            msg: "room-panel-you"
                                            color: theme.textFaint; font.pointSize: theme.typeSmall
                                        }
                                    }
                                    Row {
                                        id: status
                                        objectName: "room-panel-status"
                                        anchors.right: parent.right
                                        anchors.verticalCenter: avatar.verticalCenter
                                        visible: !member.offer
                                        spacing: theme.space(1.5)
                                        Icon {
                                            id: statusIcon
                                            anchors.verticalCenter: parent.verticalCenter
                                            glyph: member.look.icon; size: theme.space(4); color: member.look.color
                                            RotationAnimation on rotation {
                                                running: member.look.spin && status.visible && !theme.reducedMotion
                                                from: 0; to: 360; duration: 1200; loops: Animation.Infinite
                                                onRunningChanged: if (!running) statusIcon.rotation = 0
                                            }
                                        }
                                        Label {
                                            anchors.verticalCenter: parent.verticalCenter
                                            text: member.downloading ? words.t("room-panel-percent", { percent: Math.floor(member.percent) }) : words.t(member.look.msg)
                                            role: member.downloading ? "ledger" : "ui"
                                            color: member.look.color
                                            font.pointSize: theme.typeSmall
                                        }
                                    }
                                    Button {
                                        id: getButton
                                        objectName: "room-panel-get-media"
                                        anchors.right: parent.right
                                        anchors.verticalCenter: avatar.verticalCenter
                                        visible: member.offer
                                        primary: true; small: true
                                        icon: member.fetch === "Failed" ? "rotate-cw" : "download"
                                        msg: member.fetch === "Failed" ? "action-try-again" : "room-panel-get-media"
                                        takesFocus: false
                                        onClicked: room.getMedia()
                                    }
                                    // The download: a thin line from the name to the edge, the time left at its end
                                    Item {
                                        objectName: "room-panel-progress"
                                        visible: member.downloading
                                        anchors.left: nameRow.left; anchors.right: parent.right
                                        anchors.top: avatar.bottom; anchors.topMargin: theme.space(1)
                                        height: theme.space(3)
                                        Rectangle {
                                            id: track
                                            anchors.left: parent.left; anchors.verticalCenter: parent.verticalCenter
                                            width: parent.width - (etaText.visible ? etaText.implicitWidth + theme.space(3) : 0)
                                            height: 3; radius: 1.5
                                            color: theme.line
                                            Rectangle {
                                                width: Math.max(height, parent.width * member.percent / 100)
                                                height: parent.height; radius: parent.radius
                                                color: theme.accent
                                                Behavior on width { enabled: !theme.reducedMotion; NumberAnimation { duration: theme.motionNormal; easing.type: Easing.OutCubic } }
                                            }
                                        }
                                        Label {
                                            id: etaText
                                            anchors.right: parent.right; anchors.verticalCenter: parent.verticalCenter
                                            visible: member.m.eta !== null && member.m.eta !== undefined && member.m.eta > 0
                                            text: visible ? words.t("room-panel-left", { time: member.m.eta < 60 ? words.countdownSeconds(member.m.eta) : words.countdown(member.m.eta) }) : ""
                                            role: "ledger"; color: theme.textDim; font.pointSize: theme.typeSmall
                                        }
                                    }
                                }
                            }
                        }
                        // The offset: how far this copy is from the host's, set by ear or by the openings
                        Column {
                            objectName: "room-panel-offset"
                            visible: root.inPlayer && room.drivesPlayer
                            width: parent.width
                            spacing: theme.space(1)
                            Rectangle { width: parent.width; height: 1; color: theme.line }
                            Item { width: 1; height: theme.space(1) }
                            Item {
                                width: parent.width
                                height: theme.space(9)
                                Label { anchors.verticalCenter: parent.verticalCenter; msg: "room-panel-offset"; role: "ui" }
                                Row {
                                    anchors.right: parent.right; anchors.verticalCenter: parent.verticalCenter
                                    spacing: theme.space(1)
                                    PlayerButton { objectName: "room-panel-offset-down"; glyph: "minus"; tip: words.t("room-panel-offset-earlier"); onClicked: room.nudgeOffset(-0.5) }
                                    Label { objectName: "room-panel-offset-value"; anchors.verticalCenter: parent.verticalCenter; width: theme.space(16); horizontalAlignment: Text.AlignHCenter; role: "ledger"; text: root.offsetText }
                                    PlayerButton { objectName: "room-panel-offset-up"; glyph: "plus"; tip: words.t("room-panel-offset-later"); onClicked: room.nudgeOffset(0.5) }
                                }
                            }
                            Item {
                                readonly property var suggested: root.info ? root.info.suggested_offset : null
                                visible: suggested !== null && suggested !== undefined
                                width: parent.width
                                height: visible ? theme.space(8) : 0
                                Label {
                                    anchors.verticalCenter: parent.verticalCenter
                                    width: parent.width - useSuggested.width - theme.space(2)
                                    text: parent.visible ? words.t("room-panel-suggested", { offset: root.signed(parent.suggested) }) : ""
                                    role: "ui"; color: theme.textDim; font.pointSize: theme.typeSmall
                                    elide: Text.ElideRight
                                }
                                Button {
                                    id: useSuggested
                                    objectName: "room-panel-use-suggested"
                                    anchors.right: parent.right; anchors.verticalCenter: parent.verticalCenter
                                    visible: parent.visible && Math.abs(parent.suggested - root.info.offset) > 0.05
                                    msg: "room-panel-use-suggested"; small: true; flat: true; takesFocus: false
                                    onClicked: room.setOffset(parent.suggested)
                                }
                            }
                        }
                    }
                    Button {
                        id: leaveButton
                        anchors.bottom: parent.bottom
                        objectName: "room-panel-leave"
                        danger: true
                        icon: "log-out"
                        msg: "room-panel-leave"
                        takesFocus: false
                        onClicked: room.leave()
                    }
                }
            }
        }
    }
}
