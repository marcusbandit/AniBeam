// The connect sheet: a guided onboarding for signing a tracker in. AniList's
// flow is a genuine sequence — create a client on their developer page with
// AniBeam's redirect URL, paste the client id back here, connect through the
// browser — so the sheet walks it as three numbered steps with the state each
// leaves behind. Reached from the Watching page's empty list and the Trackers
// panel; Escape or the scrim closes it, and the connect job itself keeps
// running either way, its success announced by the toast.
import QtQuick
import com.marcusrosado.AniBeam

Item {
    id: root
    property string tracker: "Anilist"
    readonly property string name: tracker === "Anilist" ? "AniList" : "MyAnimeList"
    readonly property bool anilist: tracker === "Anilist"
    readonly property var account: tracker === "Anilist" ? (Door.trackers.anilist || ({})) : (Door.trackers.mal || ({}))
    property bool waiting: false
    property bool failed: false
    property string connectedAs: ""
    // The client's existence is the first two steps' fact: once an id stands
    // in the field (or arrived bundled), they are both done.
    readonly property bool hasId: (accountId.text || "") !== "" || root.account.bundled_credentials === true
    signal closed()

    readonly property string redirectUrl: "http://127.0.0.1:53682/callback"

    function openFor(t) {
        root.tracker = t || "Anilist"
        root.waiting = false
        root.failed = false
        root.connectedAs = ""
        accountId.text = root.account.client_id || ""
        visible = true
        frame.escapeStack.push("popover", root)
        if (accountId.text !== "") accountId.focusInput()
    }
    function close() {
        if (!visible) return
        frame.escapeStack.pop(root)
        visible = false
        closed()
        frame.focusPage(root)
    }

    width: parent ? parent.width : 0
    height: parent ? parent.height : 0
    visible: false
    z: 10

    // The scrim and shield: a press of any button off the sheet closes it, and nothing
    // (a right-click, a wheel, a hover) reaches the page under it.
    Blocker { anchors.fill: parent; dismissOnPress: true; onDismissed: root.close() }

    // The step's roundel: the number while pending, the check once the step's
    // fact exists, the clock while the browser is out.
    component StepBadge: Corner {
        property int step: 0
        property bool done: false
        property bool current: false
        property bool working: false
        width: theme.space(7); height: width
        radius: width / 2
        smoothing: theme.cornerSmoothing
        color: done ? theme.accentSoft : theme.surfaceSunken
        borderColor: current && !done ? theme.accent : theme.line
        borderWidth: current && !done ? 1.5 : 1
        anchors.verticalCenter: parent.top
        Icon {
            anchors.centerIn: parent
            glyph: parent.working ? "clock" : parent.done ? "check" : ""
            size: theme.space(3.5)
            color: parent.done ? theme.accent : theme.textDim
        }
        Text {
            anchors.centerIn: parent
            visible: !parent.done && !parent.working
            text: parent.step
            color: parent.current ? theme.accent : theme.textFaint
            font.family: theme.fontUi
            font.pointSize: theme.typeSmall
            font.weight: Font.DemiBold
        }
    }

    Corner {
        id: sheet
        anchors.centerIn: parent
        width: Math.min(parent.width - theme.space(20), theme.space(170))
        height: Math.min(parent.height - theme.space(8), steps.implicitHeight + theme.space(14))
        radius: theme.radiusLg
        smoothing: theme.cornerSmoothing
        color: theme.surfaceRaised
        borderColor: theme.lineStrong
        borderWidth: 1
        objectName: "connect-sheet"

        // Takes every press on the sheet's text and padding, which would otherwise reach
        // the shield and close it; the steps are declared after it, so their controls sit on top.
        MouseArea { anchors.fill: parent; acceptedButtons: Qt.AllButtons }

        Column {
            id: steps
            anchors.left: parent.left; anchors.right: parent.right
            anchors.top: parent.top; anchors.margins: theme.space(7)
            spacing: theme.space(5)

            // The head
            Item {
                width: parent.width
                height: headTitle.height + headSub.height + theme.space(0.5)
                Column {
                    spacing: theme.space(0.5)
                    Text {
                        id: headTitle
                        text: root.connectedAs !== "" ? "Connected" : "Connect " + root.name
                        color: theme.text
                        font.family: theme.fontDisplay
                        font.pointSize: theme.typeNormal * 1.3
                        font.weight: Font.DemiBold
                    }
                    Text {
                        id: headSub
                        text: root.connectedAs !== "" ? "Signed in as " + root.connectedAs + ". Your list is on the Watching page."
                            : "AniList signs AniBeam in through an app of your own. It takes about a minute, once."
                        color: theme.textDim
                        font.family: theme.fontUi
                        font.pointSize: theme.typeSmall
                    }
                }
                Icon {
                    glyph: "x"
                    size: theme.space(4.5)
                    color: theme.textDim
                    anchors.right: parent.right; anchors.verticalCenter: parent.verticalCenter
                    MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: root.close() }
                }
            }

            // Connected: the receipt and nothing else
            Row {
                visible: root.connectedAs !== ""
                spacing: theme.space(3)
                Button { text: "Open Watching"; icon: "eye"; onClicked: { root.close(); frame.go("watching") } }
                Button { text: "Done"; flat: true; onClicked: root.close() }
            }

            // Step 1: the client on AniList's developer page
            Row {
                visible: root.connectedAs === ""
                width: parent.width
                spacing: theme.space(3)
                StepBadge { id: badge1; step: 1; done: root.hasId; current: !root.hasId }
                Column {
                    width: parent.width - badge1.width - parent.spacing * 2
                    spacing: theme.space(2)
                    Text {
                        text: "Create the client on AniList"
                        color: theme.text
                        font.family: theme.fontUi
                        font.pointSize: theme.typeNormal
                        font.weight: Font.DemiBold
                    }
                    Text {
                        width: parent.width
                        text: "Open AniList's developer settings, create a client, and give it this redirect URL exactly, including the port and the /callback path."
                        color: theme.textDim
                        wrapMode: Text.Wrap
                        font.family: theme.fontUi
                        font.pointSize: theme.typeSmall
                    }
                    Flow {
                        width: parent.width
                        spacing: theme.space(2)
                        Button { text: "Open the developer page"; icon: "external-link"; small: true; flat: true
                            onClicked: Qt.openUrlExternally("https://anilist.co/settings/developer") }
                        Chip {
                            text: root.redirectUrl
                            clickable: true
                            icon: urlCopied.running ? "check" : "copy"
                            maxLabelWidth: parent.width - theme.space(30)
                            onClicked: { copyBuf.text = root.redirectUrl; copyBuf.selectAll(); copyBuf.copy(); urlCopied.restart() }
                            Tooltip { text: "Copy the redirect URL" }
                        }
                        TextEdit { id: copyBuf; visible: false }  // not "clip": that id would shadow Item.clip
                        Timer { id: urlCopied; interval: 1200 }
                    }
                }
            }

            // Step 2: the client id back here
            Row {
                visible: root.connectedAs === ""
                width: parent.width
                spacing: theme.space(3)
                StepBadge { id: badge2; step: 2; done: root.hasId; current: !root.hasId }
                Column {
                    width: parent.width - badge2.width - parent.spacing * 2
                    spacing: theme.space(2)
                    Text {
                        text: root.account.bundled_credentials === true ? "Your Client ID is already set" : "Paste the Client ID"
                        color: theme.text
                        font.family: theme.fontUi
                        font.pointSize: theme.typeNormal
                        font.weight: Font.DemiBold
                    }
                    Text {
                        visible: root.account.bundled_credentials !== true
                        width: parent.width
                        text: "It is the number on your new client's page."
                        color: theme.textDim
                        font.family: theme.fontUi
                        font.pointSize: theme.typeSmall
                    }
                    Field {
                        id: accountId
                        visible: root.account.bundled_credentials !== true
                        width: parent.width
                        placeholder: "Client ID"
                        mono: true
                    }
                }
            }

            // Step 3: connect through the browser
            Row {
                visible: root.connectedAs === ""
                width: parent.width
                spacing: theme.space(3)
                StepBadge { id: badge3; step: 3; done: false; current: root.hasId && !root.waiting; working: root.waiting }
                Column {
                    width: parent.width - badge3.width - parent.spacing * 2
                    spacing: theme.space(2)
                    Text {
                        text: root.waiting ? "Waiting for AniList in your browser" : "Connect"
                        color: theme.text
                        font.family: theme.fontUi
                        font.pointSize: theme.typeNormal
                        font.weight: Font.DemiBold
                    }
                    Text {
                        width: parent.width
                        text: root.failed
                            ? root.name + " did not finish signing you in. Check the Client ID and try again."
                            : root.waiting
                                ? "AniList asks you to sign in and approve AniBeam; the reply comes back here on its own."
                                : "The browser opens, AniList asks you to sign in, and AniBeam catches the reply."
                        color: root.failed ? theme.red : theme.textDim
                        wrapMode: Text.Wrap
                        font.family: theme.fontUi
                        font.pointSize: theme.typeSmall
                    }
                    Row {
                        spacing: theme.space(2)
                        Button {
                            text: root.waiting ? "Connecting…" : "Connect"
                            icon: root.waiting ? "clock" : "log-in"
                            enabled: root.hasId && !root.waiting
                            onClicked: {
                                root.failed = false
                                var c = Door.setTrackerCredentials(root.tracker, accountId.text, "")
                                if (c.error) { root.failed = true; frame.toast(c.error.message); return }
                                var r = Door.connectTracker(root.tracker)
                                if (r.error) { root.failed = true; frame.toast(r.error.message); return }
                                root.waiting = true
                            }
                        }
                        Button {
                            visible: root.waiting
                            text: "Cancel"
                            flat: true
                            onClicked: {
                                var jobs = Door.listJobs()
                                if (!jobs.error) jobs.reply.jobs.forEach(function(j) {
                                    if (String(j.kind).toLowerCase() === "connecttracker") Door.cancelJob(j.id)
                                })
                                root.waiting = false
                            }
                        }
                    }
                }
            }
        }
    }

    Connections {
        target: Door
        function onAuthUrlReady(t, openUrl, redirectUrl) { if (t === root.tracker) root.waiting = true }
        function onTrackerConnected(t, username) {
            if (t !== root.tracker) return
            root.waiting = false
            root.failed = false
            root.connectedAs = username || root.name
        }
        function onJobFinished(job, kind, ok) {
            if (kind !== "ConnectTracker" || ok) return
            root.waiting = false
            root.failed = true
        }
    }
}
