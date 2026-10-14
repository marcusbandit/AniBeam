// One subscription: the feed and the auto-download rule anirss registered. The name
// with its state beside it, the query it matches and where it saves in the ledger cut,
// the torrent count, a link to the feed and Remove. Remove asks anirss for the plan first
// and shows it as the inline question; the feed and the rule go, the torrents and the
// files stay (they have their own Remove in the transfers list).
import QtQuick
import com.marcusrosado.AniBeam

Item {
    id: root
    property var rule: ({})
    // The plan the page fetched for this row, or null while none is up
    property var plan: null
    property bool planning: false
    signal askRemove()
    signal confirmRemove()
    signal keep()

    readonly property var r: rule || ({})
    readonly property bool active: r.rule_enabled !== false
    readonly property string fact: {
        var parts = [Fmt.plural(root.r.torrent_count || 0, "torrent", "torrents")]
        if (root.r.query) parts.push("matches \"" + root.r.query + "\"")
        return parts.join(", ")
    }
    readonly property string question: {
        if (!root.plan) return ""
        var n = root.plan.torrent_count_in_qb || 0
        return "Remove the feed and rule for " + root.r.name + "? " + (n > 0 ? Fmt.plural(n, "torrent stays", "torrents stay") + " in the client." : "")
    }

    width: parent ? parent.width : theme.space(100)
    height: theme.space(14)

    Rectangle { anchors.fill: parent; radius: theme.radiusSm; color: hover.containsMouse ? theme.raised : "transparent" }
    MouseArea { id: hover; anchors.fill: parent; hoverEnabled: true; acceptedButtons: Qt.NoButton }

    Icon {
        id: glyph
        anchors.left: parent.left
        anchors.leftMargin: theme.space(2)
        anchors.verticalCenter: parent.verticalCenter
        glyph: "rss"
        size: theme.space(5)
        color: root.active ? theme.accent : theme.textFaint
    }
    Column {
        anchors.left: glyph.right
        anchors.leftMargin: theme.space(3)
        anchors.right: controls.left
        anchors.rightMargin: theme.space(4)
        anchors.verticalCenter: parent.verticalCenter
        spacing: theme.space(0.75)
        Row {
            width: parent.width
            spacing: theme.space(3)
            Label {
                width: Math.min(implicitWidth, parent.width - stateLabel.width - parent.spacing)
                text: root.r.name || ""
                role: "title"
                font.pointSize: theme.typeNormal
                elide: Text.ElideRight
                anchors.verticalCenter: parent.verticalCenter
            }
            Label {
                id: stateLabel
                anchors.verticalCenter: parent.verticalCenter
                text: root.active ? "Active" : "Paused"
                role: "ledger"
                color: root.active ? theme.caughtUp : theme.textFaint
            }
        }
        Label { width: parent.width; text: root.fact; role: "ledger"; color: theme.textDim; elide: Text.ElideRight }
        Label { width: parent.width; text: root.r.save_path || ""; role: "ledger"; color: theme.textFaint; elide: Text.ElideMiddle }
    }
    Item {
        id: controls
        anchors.right: parent.right
        anchors.rightMargin: theme.space(1.5)
        anchors.verticalCenter: parent.verticalCenter
        width: root.plan ? confirm.width : buttons.width
        height: theme.controlHeight
        Row {
            id: buttons
            visible: !root.plan
            anchors.right: parent.right
            anchors.verticalCenter: parent.verticalCenter
            spacing: theme.space(1.5)
            Button { visible: !!root.r.feed_url; text: "Feed"; icon: "external-link"; flat: true; small: true; onClicked: frame.openLink(root.r.feed_url) }
            Button { text: root.planning ? "Planning" : "Remove"; icon: "trash-2"; flat: true; small: true; enabled: !root.planning; onClicked: root.askRemove() }
        }
        InlineConfirm {
            id: confirm
            visible: !!root.plan
            anchors.right: parent.right
            anchors.verticalCenter: parent.verticalCenter
            question: root.question
            confirmText: "Remove"
            onAccepted: root.confirmRemove()
            onKept: root.keep()
        }
    }
    Rectangle { anchors.bottom: parent.bottom; x: theme.space(2); width: parent.width - theme.space(4); height: 1; color: theme.line; opacity: 0.5 }
}
