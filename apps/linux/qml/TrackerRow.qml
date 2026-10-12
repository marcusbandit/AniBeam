// One tracker: a pill naming the service in full (no logo is bundled, and a caps monogram
// is a killed idiom), lit in the accent while connected, the connection line, and a control area that walks through the
// connect flow: idle with credentials already known, waiting on the browser, connected, or
// the register-and-paste-credentials form when none are known yet.
import QtQuick
import com.marcusrosado.AniBeam

Item {
    id: root
    objectName: tracker !== "" ? "tracker-" + tracker : ""
    property string tracker: ""
    property var account: ({})
    property bool waiting: false
    property bool confirming: false
    // Guided: nobody has run this tracker's OAuth app yet, so the row hands
    // the flow to the connect sheet instead of showing the paste-a-form.
    property bool guided: false
    property string redirectUrl: "http://127.0.0.1:53682/callback"
    signal login(string clientId, string clientSecret)
    signal setup()
    signal disconnect()
    signal cancel()

    readonly property string name: tracker === "Anilist" ? "AniList" : "MyAnimeList"
    readonly property bool connected: account.connected === true
    // A confirm left up must not greet the next connection: any way the row stops being
    // connected (the confirm's own Disconnect, a revoked token) puts the question away
    onConnectedChanged: if (!connected) confirming = false
    readonly property string line: connected
        ? "Connected as " + (account.username || "?") + ", synced " + (account.last_sync ? Fmt.relative(account.last_sync, Date.now() / 1000) : "never")
        : "Not connected"
    readonly property bool hasCredentials: account.bundled_credentials === true || (account.client_id || "") !== ""
    readonly property bool showLogin: !waiting && !connected && hasCredentials
    readonly property bool showCreds: !waiting && !connected && !hasCredentials
    readonly property bool showSetup: showCreds && guided

    width: parent ? parent.width : theme.space(100)
    implicitHeight: top.height + (showSetup ? theme.space(3) + setupBox.implicitHeight : showCreds ? theme.space(3) + creds.implicitHeight : 0)

    Item {
        id: top
        width: parent.width
        height: Math.max(theme.space(12), Math.max(avatar.height, words.implicitHeight, slot.height) + theme.space(2) * 2)

        Corner {
            id: avatar
            anchors.left: parent.left
            anchors.verticalCenter: parent.verticalCenter
            // One width for both services, so the two rows' connection lines start level
            width: Math.max(theme.space(30), badge.implicitWidth + theme.space(3) * 2)
            height: theme.controlHeight
            radius: height / 2
            smoothing: theme.cornerSmoothing
            color: root.connected ? theme.accentSoft : theme.surfaceSunken
            borderColor: root.connected ? "transparent" : theme.line
            borderWidth: root.connected ? 0 : 1
            Label {
                id: badge
                anchors.centerIn: parent
                text: root.name
                role: "ledger"
                color: root.connected ? theme.accent : theme.textDim
            }
        }
        Column {
            id: words
            anchors.left: avatar.right
            anchors.leftMargin: theme.space(3)
            anchors.right: slot.left
            anchors.rightMargin: theme.space(6)
            anchors.verticalCenter: parent.verticalCenter
            spacing: theme.space(0.5)
            Label {
                width: parent.width
                text: root.line
                role: "ui"
                font.pointSize: theme.typeSmall
                color: theme.textDim
                wrapMode: Text.Wrap
            }
        }
        // Loaders, not four plain children behind `visible`: `childrenRect`, which sizes
        // this slot, unions every child's geometry whether or not it is visible, so an
        // inactive branch (the wide waiting note, the long "Log in to MyAnimeList") would
        // still widen the slot and crush the name column beside it. An inactive Loader
        // holds no item and truly measures zero.
        Item {
            id: slot
            anchors.right: parent.right
            anchors.verticalCenter: parent.verticalCenter
            width: childrenRect.width
            height: childrenRect.height

            Loader { active: root.waiting; visible: active; sourceComponent: waitingBox }
            Loader { active: !root.waiting && root.connected && !root.confirming; visible: active; sourceComponent: disconnectButton }
            Loader { active: !root.waiting && root.connected && root.confirming; visible: active; sourceComponent: disconnectConfirm }
            Loader { active: root.showLogin; visible: active; sourceComponent: loginButton }
        }
    }
    Component {
        id: waitingBox
        Column {
            width: theme.space(64)
            spacing: theme.space(1.5)
            Note { text: "Waiting for you to approve in the browser…" }
            Button { text: "Cancel"; flat: true; onClicked: root.cancel() }
        }
    }
    Component {
        id: disconnectButton
        Button { objectName: root.tracker + "-disconnect"; text: "Disconnect"; icon: "log-out"; onClicked: root.confirming = true }
    }
    Component {
        id: disconnectConfirm
        InlineConfirm {
            question: "Disconnect " + root.name + "? Your sign-in will be removed."
            confirmText: "Disconnect"
            confirmIcon: "log-out"
            onAccepted: { root.confirming = false; root.disconnect() }
            onKept: root.confirming = false
        }
    }
    Component {
        id: loginButton
        Button { text: "Log in to " + root.name; icon: "log-in"; onClicked: root.login("", "") }
    }

    // The guided path: one line and the button that opens the sheet, which
    // walks the create-client, paste-id, connect sequence with the state
    // each step leaves behind.
    Column {
        id: setupBox
        visible: root.showSetup
        anchors.top: top.bottom
        anchors.topMargin: theme.space(3)
        anchors.left: parent.left
        anchors.right: parent.right
        spacing: theme.space(2)
        Label {
            width: parent.width
            role: "ui"
            text: root.tracker === "Anilist"
                ? "AniList needs an app of your own to sign AniBeam in. Setup walks you through creating it, pasting its Client ID and connecting."
                : "Create an app (App Type: \"Web\") on MyAnimeList with the redirect URL below, then paste its Client ID and secret here."
            color: theme.textDim
            wrapMode: Text.Wrap
            font.pointSize: theme.typeSmall
        }
        Button { text: "Set up " + root.name; icon: "log-in"; onClicked: root.setup() }
    }

    // Nobody has run this tracker's OAuth app yet: the register help, the redirect URL to
    // paste into it, and the client id (and, for MAL, the secret) to paste back here.
    Column {
        id: creds
        visible: root.showCreds && !root.guided
        anchors.top: top.bottom
        anchors.topMargin: theme.space(3)
        anchors.left: parent.left
        anchors.right: parent.right
        spacing: theme.space(2)
        Note {
            text: root.tracker === "Anilist"
                ? "Create a new client. Paste the redirect URL below into AniList's \"Redirect URL\" field exactly as shown, including the port and the /callback path."
                : "Create an app (App Type: \"Web\"). Paste the redirect URL below into MAL's \"App Redirect URL\" field."
        }
        // A Flow, not a Row: at a narrow window (W=1000 is the brief's own probe) the label
        // and the full callback URL do not fit on one line, and an unbound Row just runs
        // the chip past the panel's edge instead of wrapping. Bound to the column's width
        // so Flow knows when to wrap the chip to its own line, and the chip's own
        // maxLabelWidth keeps it from overflowing even alone on that line, eliding the
        // middle rather than clipping mid-character; the full URL still goes to the
        // clipboard regardless of what is shown.
        Flow {
            width: parent.width
            spacing: theme.space(2)
            Label {
                text: "Redirect URL"
                role: "ui"
                color: theme.textDim
                height: redirectChip.height
                verticalAlignment: Text.AlignVCenter
                font.pointSize: theme.typeSmall
            }
            Chip {
                id: redirectChip
                text: root.redirectUrl
                clickable: true
                icon: copied.running ? "check" : "copy"
                maxLabelWidth: parent.width - theme.space(14)
                onClicked: { clipboard.text = root.redirectUrl; clipboard.selectAll(); clipboard.copy(); copied.restart() }
            }
        }
        TextEdit { id: clipboard; visible: false }
        Timer { id: copied; interval: 1200 }
        Field { id: clientId; width: parent.width; placeholder: "Client ID"; mono: true }
        Field { id: secret; visible: root.tracker === "Mal"; width: parent.width; placeholder: "Client secret"; mono: true }
        Button { text: "Connect"; icon: "log-in"; onClicked: root.login(clientId.text, secret.visible ? secret.text : "") }
    }
}
