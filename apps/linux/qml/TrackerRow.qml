// One tracker: a pill naming the service in full (no logo is bundled, and a caps monogram
// is a killed idiom), lit in the accent while connected, the connection line, and one
// control on the right. Not connected,
// that control is Connect, which hands the tracker to frame.openConnect: with credentials
// the browser opens on that click, without them the sheet walks the set-up. While the
// browser is out it is Cancel and the sign-in page again; connected, Disconnect behind an
// inline confirm. The flow's state is the frame's, so the row only reads it. A MAL app
// that turned out to want its client secret gets a field for it right here.
import QtQuick
import com.marcusrosado.AniBeam

Item {
    id: root
    objectName: tracker !== "" ? "tracker-" + tracker : ""
    property string tracker: ""
    property var account: ({})
    property bool confirming: false
    property bool addingSecret: false
    signal disconnect()

    readonly property string name: frame.trackerName(tracker)
    readonly property bool connected: account.connected === true
    // A confirm left up must not greet the next connection: any way the row stops being
    // connected (the confirm's own Disconnect, a revoked token) puts the question away
    onConnectedChanged: if (!connected) confirming = false
    readonly property bool waiting: frame.connecting[tracker] !== undefined
    readonly property string error: frame.connectErrors[tracker] || ""
    readonly property bool needsSecret: tracker === "Mal" && !connected && !waiting && /client secret/i.test(error)
    readonly property string line: connected
        ? "Connected as " + (account.username || "?") + ", synced " + (account.last_sync ? Fmt.relative(account.last_sync, Date.now() / 1000) : "never")
        : waiting ? "Waiting for you to approve in the browser"
        : error !== "" ? error
        : "Not connected"

    width: parent ? parent.width : theme.space(100)
    // A secret is saved and nothing is asking for one: offered for removal, for the app
    // that turned out to be "other", where MAL refuses a secret it never issued.
    readonly property bool canRemoveSecret: tracker === "Mal" && !connected && !waiting && !needsSecret
        && account.has_client_secret === true
    implicitHeight: top.height + (secretForm.visible ? theme.space(1) + secretForm.implicitHeight : 0)
        + (savedSecret.visible ? theme.space(1) + savedSecret.height : 0)
    onNeedsSecretChanged: if (!needsSecret) addingSecret = false

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
                color: !root.connected && !root.waiting && root.error !== "" ? theme.red : theme.textDim
                wrapMode: Text.Wrap
            }
        }
        // Loaders, not plain children behind `visible`: an inactive branch (the wide
        // confirm, the waiting buttons) would still take room it is not using and crush the
        // name column beside it. An inactive Loader holds no item and measures zero, and the
        // slot is the widest of the four rather than `childrenRect`, which was seen keeping
        // a branch's old width after the branch shrank.
        Item {
            id: slot
            anchors.right: parent.right
            anchors.verticalCenter: parent.verticalCenter
            width: Math.max(waitingSlot.width, disconnectSlot.width, confirmSlot.width, connectSlot.width)
            height: Math.max(waitingSlot.height, disconnectSlot.height, confirmSlot.height, connectSlot.height)

            Loader { id: waitingSlot; anchors.right: parent.right; active: root.waiting; visible: active; sourceComponent: waitingButtons }
            Loader { id: disconnectSlot; anchors.right: parent.right; active: !root.waiting && root.connected && !root.confirming; visible: active; sourceComponent: disconnectButton }
            Loader { id: confirmSlot; anchors.right: parent.right; active: !root.waiting && root.connected && root.confirming; visible: active; sourceComponent: disconnectConfirm }
            Loader { id: connectSlot; anchors.right: parent.right; active: !root.waiting && !root.connected; visible: active; sourceComponent: connectButton }
        }
    }
    Component {
        id: waitingButtons
        Row {
            spacing: theme.space(2)
            Button {
                text: "Open again"; icon: "external-link"; flat: true
                enabled: (frame.signInUrls[root.tracker] || "") !== ""
                onClicked: frame.reopenSignIn(root.tracker)
                Tooltip { text: "Open the sign-in page again" }
            }
            Button { text: "Cancel"; flat: true; onClicked: frame.cancelConnect(root.tracker) }
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
        id: connectButton
        ConnectButton { objectName: "tracker-" + root.tracker + "-connect"; tracker: root.tracker; text: "Connect"; icon: "log-in"; primary: !root.needsSecret; onClicked: frame.openConnect(root.tracker) }
    }

    Row {
        id: savedSecret
        visible: root.canRemoveSecret
        anchors.top: top.bottom
        anchors.topMargin: theme.space(1)
        anchors.left: parent.left
        anchors.leftMargin: avatar.width + theme.space(3)
        spacing: theme.space(2)
        Label {
            anchors.verticalCenter: parent.verticalCenter
            text: "A client secret is saved for this app."
            role: "ui"
            color: theme.textDim
            font.pointSize: theme.typeSmall
        }
        // An empty secret is the contract's "remove"; an absent one would keep it.
        Button {
            text: "Remove it"; icon: "trash-2"; flat: true; small: true
            onClicked: {
                var r = Door.call("SetTrackerCredentials", { tracker: root.tracker, client_id: root.account.client_id || "", client_secret: "" })
                frame.toast(r.error ? frame.errorText(r.error) : "Client secret removed")
            }
        }
    }

    // MAL answered that this app is a "web" one: its secret, saved, and the flow again.
    // Under the row rather than beside Connect, where it would squeeze the line that says
    // why it is needed.
    Column {
        id: secretForm
        visible: root.needsSecret
        anchors.top: top.bottom
        anchors.topMargin: theme.space(1)
        anchors.left: parent.left
        anchors.leftMargin: avatar.width + theme.space(3)
        anchors.right: parent.right
        spacing: theme.space(2)
        Button {
            objectName: "tracker-" + root.tracker + "-add-secret"
            visible: !root.addingSecret
            text: "Add client secret"; icon: "plus"; primary: true
            onClicked: { root.addingSecret = true; Qt.callLater(secret.focusInput) }
        }
        Note { visible: root.addingSecret; text: "It is the long string under \"Client Secret\" on the app's page at myanimelist.net/apiconfig." }
        Field { id: secret; visible: root.addingSecret; width: parent.width; placeholder: "Client secret"; mono: true }
        Row {
            visible: root.addingSecret
            spacing: theme.space(2)
            Button {
                text: "Save and connect"; icon: "log-in"; primary: true
                enabled: secret.text.trim() !== ""
                onClicked: {
                    var r = Door.setTrackerCredentials(root.tracker, root.account.client_id || "", secret.text.trim())
                    if (r.error) { frame.toast(frame.errorText(r.error)); return }
                    secret.text = ""
                    frame.connectTracker(root.tracker)
                }
            }
            Button { text: "Cancel"; flat: true; onClicked: root.addingSecret = false }
        }
    }
}
