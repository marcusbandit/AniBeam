// One tracker: a round avatar with the provider's initials, lit in the accent while
// connected, the name and connection line, and a control area that walks through the
// connect flow: idle with credentials already known, waiting on the browser, connected, or
// the register-and-paste-credentials form when none are known yet.
import QtQuick
import com.marcusrosado.AniBeam

Item {
    id: root
    property string tracker: ""
    property var account: ({})
    property bool waiting: false
    property bool confirming: false
    property string redirectUrl: "http://127.0.0.1:53682/callback"
    signal login(string clientId, string clientSecret)
    signal disconnect()
    signal cancel()

    readonly property string name: tracker === "Anilist" ? "AniList" : "MyAnimeList"
    readonly property string initials: tracker === "Anilist" ? "AL" : "MAL"
    readonly property bool connected: account.connected === true
    readonly property string line: connected
        ? "Connected as " + (account.username || "?") + " · synced " + (account.last_sync ? Fmt.relative(account.last_sync, Date.now() / 1000) : "never")
        : "Not connected"
    readonly property bool hasCredentials: account.bundled_credentials === true || (account.client_id || "") !== ""
    readonly property bool showLogin: !waiting && !connected && hasCredentials
    readonly property bool showCreds: !waiting && !connected && !hasCredentials

    width: parent ? parent.width : theme.space(100)
    implicitHeight: top.height + (showCreds ? theme.space(3) + creds.implicitHeight : 0)

    Item {
        id: top
        width: parent.width
        height: Math.max(theme.space(12), Math.max(avatar.height, words.implicitHeight, slot.height) + theme.space(2) * 2)

        Corner {
            id: avatar
            anchors.left: parent.left
            anchors.verticalCenter: parent.verticalCenter
            width: theme.space(9); height: width
            radius: width / 2
            smoothing: theme.cornerSmoothing
            color: root.connected ? theme.accentSoft : theme.surfaceSunken
            borderColor: root.connected ? "transparent" : theme.line
            borderWidth: root.connected ? 0 : 1
            Text {
                anchors.centerIn: parent
                text: root.initials
                color: root.connected ? theme.accent : theme.textDim
                font.family: theme.fontMono
                font.pointSize: theme.typeSmall
                font.weight: Font.Bold
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
            Text {
                width: parent.width
                text: root.name
                color: theme.text
                wrapMode: Text.Wrap
                font.family: theme.fontSans
                font.pointSize: theme.typeNormal
            }
            Text {
                width: parent.width
                text: root.line
                color: theme.textDim
                wrapMode: Text.Wrap
                font.family: theme.fontMono
                font.pointSize: theme.typeSmall
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
            Note { text: "Waiting for browser authorization…" }
            Button { text: "Cancel"; flat: true; onClicked: root.cancel() }
        }
    }
    Component {
        id: disconnectButton
        Button { text: "Disconnect"; icon: "log-out"; onClicked: root.confirming = true }
    }
    Component {
        id: disconnectConfirm
        InlineConfirm {
            question: "Disconnect " + root.name + "? Your access token will be removed."
            confirmText: "Disconnect"
            confirmIcon: "log-out"
            onAccepted: root.disconnect()
            onKept: root.confirming = false
        }
    }
    Component {
        id: loginButton
        Button { text: "Log in to " + root.name; icon: "log-in"; onClicked: root.login("", "") }
    }

    // Nobody has run this tracker's OAuth app yet: the register help, the redirect URL to
    // paste into it, and the client id (and, for MAL, the secret) to paste back here.
    Column {
        id: creds
        visible: root.showCreds
        anchors.top: top.bottom
        anchors.topMargin: theme.space(3)
        anchors.left: parent.left
        anchors.right: parent.right
        spacing: theme.space(2)
        Note {
            text: root.tracker === "Anilist"
                ? "Create a new client. Paste the redirect URL below into AniList's \"Redirect URL\" field exactly, port and trailing /callback included."
                : "Create an app (App Type: \"Web\"). Paste the redirect URL below into MAL's \"App Redirect URL\" field."
        }
        Row {
            spacing: theme.space(2)
            Text {
                anchors.verticalCenter: parent.verticalCenter
                text: "Redirect URL"
                color: theme.textDim
                font.family: theme.fontSans
                font.pointSize: theme.typeSmall
            }
            Chip {
                anchors.verticalCenter: parent.verticalCenter
                text: root.redirectUrl
                clickable: true
                icon: copied.running ? "check" : "copy"
                onClicked: { clipboard.text = root.redirectUrl; clipboard.selectAll(); clipboard.copy(); copied.restart() }
            }
        }
        TextEdit { id: clipboard; visible: false }
        Timer { id: copied; interval: 1200 }
        Field { id: clientId; width: parent.width; placeholder: "Client ID"; mono: true }
        Field { id: secret; visible: root.tracker === "Mal"; width: parent.width; placeholder: "Client Secret"; mono: true }
        Button { text: "Connect"; icon: "log-in"; onClicked: root.login(clientId.text, secret.visible ? secret.text : "") }
    }
}
