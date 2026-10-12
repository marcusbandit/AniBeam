// The connect sheet: what a tracker sign-in looks like from AniBeam's side. With
// credentials, bundled with the build or pasted earlier, the frame has already started the
// flow by the time the sheet opens, so the sheet is one step: the sign-in page is open in
// the browser, approve AniBeam there, and the sheet turns into the receipt and closes
// itself. Without credentials it walks the set-up first, as three numbered steps with the
// state each leaves behind: create the app on the provider's developer page with AniBeam's
// redirect URL, paste its Client ID back here, connect. The flow's state lives on the
// frame (frame.connecting and its neighbours), so this only reads it; Escape or the scrim
// closes the sheet and the connect keeps running, its result announced by the toast.
import QtQuick
import com.marcusrosado.AniBeam

Item {
    id: root
    property string tracker: "Anilist"
    readonly property string name: frame.trackerName(tracker)
    readonly property bool anilist: tracker === "Anilist"
    readonly property var account: frame.trackerAccount(tracker)
    readonly property bool waiting: frame.connecting[tracker] !== undefined
    readonly property string signInUrl: frame.signInUrls[tracker] || ""
    readonly property string error: frame.connectErrors[tracker] || ""
    readonly property string connectedName: frame.connectedAs[tracker] || ""
    // MAL refused a secretless exchange: the app is a "web" one and its secret is the fix.
    readonly property bool needsSecret: tracker === "Mal" && /client secret/i.test(error)
    // Whether the set-up steps are the way in. Fixed when the sheet opens, so a Client ID
    // saved halfway through does not swap the steps out from under the user.
    property bool guided: false
    readonly property string mode: connectedName !== "" ? "done"
        : (waiting || error !== "" || !guided) ? "browser" : "setup"
    // The client's existence is the first two steps' fact: once an id stands in the
    // field, they are both done.
    readonly property bool hasId: (accountId.text || "").trim() !== ""
    signal closed()

    readonly property string developerUrl: anilist ? "https://anilist.co/settings/developer" : "https://myanimelist.net/apiconfig"

    function openFor(t) {
        root.tracker = t || "Anilist"
        root.guided = !frame.hasCredentials(root.tracker)
        // A receipt from an earlier connect is not this one's; nor is an old failure when
        // the set-up is what the user came back for.
        frame.connectedAs = frame.withKey(frame.connectedAs, root.tracker)
        if (root.guided && !root.waiting) frame.connectErrors = frame.withKey(frame.connectErrors, root.tracker)
        accountId.text = root.account.client_id || ""
        accountSecret.text = ""
        secretRetry.text = ""
        visible = true
        frame.escapeStack.push("popover", root)
        if (root.guided) accountId.focusInput()
    }
    function close() {
        if (!visible) return
        frame.escapeStack.pop(root)
        visible = false
        closed()
        frame.focusPage(root)
    }
    // The guided step 3, and the retry after a missing secret: whatever was typed is saved
    // first, so a kill mid-flow does not lose it, then the flow starts over.
    function saveAndConnect(clientId, secret) {
        var c = Door.setTrackerCredentials(root.tracker, clientId, secret)
        if (c.error) { frame.connectErrors = frame.withKey(frame.connectErrors, root.tracker, frame.errorText(c.error)); return }
        frame.connectTracker(root.tracker)
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
    // One step's heading and the line under it.
    component StepText: Column {
        property string title: ""
        property string body: ""
        property color bodyColor: theme.textDim
        width: parent.width
        spacing: theme.space(1)
        Text {
            text: parent.title
            color: theme.text
            font.family: theme.fontUi
            font.pointSize: theme.typeNormal
            font.weight: Font.DemiBold
        }
        Text {
            visible: parent.body !== ""
            width: parent.width
            text: parent.body
            color: parent.bodyColor
            wrapMode: Text.Wrap
            font.family: theme.fontUi
            font.pointSize: theme.typeSmall
        }
    }

    Corner {
        id: sheet
        objectName: "connect-sheet-card"   // the frame's instance is "connect-sheet"
        anchors.centerIn: parent
        width: Math.min(parent.width - theme.space(20), theme.space(root.mode === "setup" ? 170 : 130))
        height: Math.min(parent.height - theme.space(8), steps.implicitHeight + theme.space(14))
        radius: theme.radiusLg
        smoothing: theme.cornerSmoothing
        color: theme.surfaceRaised
        borderColor: theme.lineStrong
        borderWidth: 1
        // Takes every press on the sheet's text and padding, which would otherwise reach
        // the shield and close it; the steps are declared after it, so their controls sit on top.
        MouseArea { anchors.fill: parent; acceptedButtons: Qt.AllButtons }
        HoverHandler { id: sheetHover }

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
                    anchors.left: parent.left; anchors.right: closeGlyph.left; anchors.rightMargin: theme.space(3)
                    spacing: theme.space(0.5)
                    Text {
                        id: headTitle
                        text: root.mode === "done" ? "Connected to " + root.name
                            : root.mode === "setup" ? "Connect " + root.name
                            : root.error !== "" ? "The sign-in did not finish"
                            : "Finish in your browser"
                        color: theme.text
                        font.family: theme.fontDisplay
                        font.pointSize: theme.typeNormal * 1.3
                        font.weight: Font.DemiBold
                    }
                    Text {
                        id: headSub
                        width: parent.width
                        wrapMode: Text.Wrap
                        text: root.mode === "done"
                            ? "Signed in as " + root.connectedName + "." + (root.anilist ? " Your list is on the Watching page." : " Episodes are marked there as you watch.")
                            : root.mode === "setup"
                                ? root.name + " signs AniBeam in through an app of your own. It takes about a minute, once."
                                : root.error !== "" ? root.error
                                : root.signInUrl !== ""
                                    ? root.name + "'s sign-in page is open in your browser. Approve AniBeam there and this closes itself."
                                    : "Opening " + root.name + "'s sign-in page in your browser."
                        color: root.mode === "browser" && root.error !== "" ? theme.red : theme.textDim
                        font.family: theme.fontUi
                        font.pointSize: theme.typeSmall
                    }
                }
                Icon {
                    id: closeGlyph
                    glyph: "x"
                    size: theme.space(4.5)
                    color: theme.textDim
                    anchors.right: parent.right; anchors.top: parent.top
                    MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: root.close() }
                }
            }

            // Connected: the receipt and nothing else. It closes itself a moment later,
            // unless the pointer is resting on it.
            Row {
                visible: root.mode === "done"
                spacing: theme.space(3)
                Button { visible: root.anilist; text: "Open Watching"; icon: "eye"; onClicked: { root.close(); frame.go("watching") } }
                Button { text: "Done"; flat: root.anilist; primary: !root.anilist; onClicked: root.close() }
            }
            Timer {
                interval: 2500
                running: root.visible && root.mode === "done" && !sheetHover.hovered
                onTriggered: root.close()
            }

            // The browser is out: open its page again, or stop waiting for it
            Flow {
                visible: root.mode === "browser" && root.error === ""
                width: parent.width
                spacing: theme.space(2)
                Button {
                    visible: root.waiting
                    text: "Open the sign-in page again"; icon: "external-link"; flat: true
                    enabled: root.signInUrl !== ""
                    onClicked: frame.reopenSignIn(root.tracker)
                }
                ConnectButton {
                    visible: !root.waiting
                    tracker: root.tracker
                    text: "Connect"; icon: "log-in"; primary: true
                    onClicked: frame.connectTracker(root.tracker)
                }
                Button { text: "Cancel"; flat: true; onClicked: { frame.cancelConnect(root.tracker); root.close() } }
            }

            // It failed: try again, with the secret when MAL said that was what it wanted
            Column {
                visible: root.mode === "browser" && root.error !== ""
                width: parent.width
                spacing: theme.space(3)
                Column {
                    visible: root.needsSecret
                    width: parent.width
                    spacing: theme.space(2)
                    Text {
                        width: parent.width
                        text: "It is the long string under \"Client Secret\" on the app's page at myanimelist.net/apiconfig."
                        color: theme.textDim
                        wrapMode: Text.Wrap
                        font.family: theme.fontUi
                        font.pointSize: theme.typeSmall
                    }
                    Field { id: secretRetry; width: parent.width; placeholder: "Client secret"; mono: true }
                }
                Flow {
                    width: parent.width
                    spacing: theme.space(2)
                    Button {
                        visible: root.needsSecret
                        text: "Save and try again"; icon: "log-in"; primary: true
                        enabled: secretRetry.text.trim() !== ""
                        onClicked: root.saveAndConnect(root.account.client_id || "", secretRetry.text.trim())
                    }
                    ConnectButton {
                        visible: !root.needsSecret
                        tracker: root.tracker
                        text: "Try again"; icon: "rotate-ccw"; primary: true
                        onClicked: frame.connectTracker(root.tracker)
                    }
                    // A pasted id that is wrong is fixed in the set-up, not by trying again.
                    Button {
                        visible: root.account.bundled_credentials !== true
                        text: "Change the Client ID"; icon: "pencil"; flat: true
                        onClicked: { root.guided = true; frame.connectErrors = frame.withKey(frame.connectErrors, root.tracker); accountId.focusInput() }
                    }
                    Button { text: "Close"; flat: true; onClicked: root.close() }
                }
            }

            // Step 1: the app on the provider's developer page
            Row {
                visible: root.mode === "setup"
                width: parent.width
                spacing: theme.space(3)
                StepBadge { id: badge1; step: 1; done: root.hasId; current: !root.hasId }
                Column {
                    width: parent.width - badge1.width - parent.spacing * 2
                    spacing: theme.space(2)
                    StepText {
                        title: root.anilist ? "Create the client on AniList" : "Create the app on MyAnimeList"
                        body: root.anilist
                            ? "Open AniList's developer settings, create a client, and give it this redirect URL exactly, including the port and the /callback path."
                            : "Open MyAnimeList's API page, create an ID with App Type \"other\", and give it this redirect URL exactly, including the port and the /callback path."
                    }
                    Flow {
                        width: parent.width
                        spacing: theme.space(2)
                        Button { text: "Open the developer page"; icon: "external-link"; small: true; flat: true
                            onClicked: Qt.openUrlExternally(root.developerUrl) }
                        Chip {
                            text: frame.redirectUrl
                            clickable: true
                            icon: urlCopied.running ? "check" : "copy"
                            maxLabelWidth: parent.width - theme.space(30)
                            onClicked: { copyBuf.text = frame.redirectUrl; copyBuf.selectAll(); copyBuf.copy(); urlCopied.restart() }
                            Tooltip { text: "Copy the redirect URL" }
                        }
                        TextEdit { id: copyBuf; visible: false }  // not "clip": that id would shadow Item.clip
                        Timer { id: urlCopied; interval: 1200 }
                    }
                }
            }

            // Step 2: the Client ID back here, and for a MAL "web" app its secret
            Row {
                visible: root.mode === "setup"
                width: parent.width
                spacing: theme.space(3)
                StepBadge { id: badge2; step: 2; done: root.hasId; current: !root.hasId }
                Column {
                    width: parent.width - badge2.width - parent.spacing * 2
                    spacing: theme.space(2)
                    StepText {
                        title: "Paste the Client ID"
                        body: root.anilist ? "It is the number on your new client's page." : "It is the long string under \"Client ID\" on your new app's page."
                    }
                    Field { id: accountId; width: parent.width; placeholder: "Client ID"; mono: true }
                    Field {
                        id: accountSecret
                        visible: !root.anilist
                        width: parent.width
                        placeholder: "Client secret (only for Web apps)"
                        mono: true
                    }
                }
            }

            // Step 3: connect through the browser
            Row {
                visible: root.mode === "setup"
                width: parent.width
                spacing: theme.space(3)
                StepBadge { id: badge3; step: 3; done: false; current: root.hasId }
                Column {
                    width: parent.width - badge3.width - parent.spacing * 2
                    spacing: theme.space(2)
                    StepText {
                        title: "Connect"
                        body: "The browser opens, " + root.name + " asks you to sign in and approve AniBeam, and AniBeam catches the reply."
                    }
                    Button {
                        text: "Connect"; icon: "log-in"; primary: root.hasId
                        enabled: root.hasId
                        onClicked: root.saveAndConnect(accountId.text.trim(), root.anilist ? "" : accountSecret.text.trim())
                    }
                }
            }
        }
    }
}
