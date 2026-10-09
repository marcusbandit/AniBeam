// The subscriptions: what anirss watches for the user, read-only, reached from
// the Library settings tab's Open button. ListSubscriptions starts a job that
// shells out to anirss; SubscriptionsListed carries the result, and its four
// shapes are the page's four states: feeds, not installed, no qBittorrent
// session, or no answer.
import QtQuick
import com.marcusrosado.AniBeam

PageScroll {
    id: page
    property var props: ({})
    property string title: "Subscriptions"
    property real scrollY: 0
    function escapePressed() { return false }

    property var result: null
    property bool waiting: false

    function load() {
        var r = Door.listSubscriptions()
        if (r.error) { page.result = { kind: "Error", message: r.error.message }; page.waiting = false; return }
        page.waiting = true
    }
    Connections {
        target: Door
        function onSubscriptionsListed(res) { page.result = res; page.waiting = false }
    }
    Component.onCompleted: load()

    // The head: the title, the refresh on the right
    Item {
        width: parent.width
        height: Math.max(headCol.height, refreshBtn.height)
        Column {
            id: headCol
            anchors.left: parent.left
            spacing: theme.space(1)
            Text {
                text: "Subscriptions"
                color: theme.text
                font.family: theme.fontDisplay
                font.pointSize: theme.typeLarge
                font.weight: Font.DemiBold
            }
            Text {
                text: "The feeds anirss watches for you, and where each one saves."
                color: theme.textDim
                font.family: theme.fontUi
                font.pointSize: theme.typeSmall
            }
        }
        Button {
            id: refreshBtn
            anchors.right: parent.right; anchors.verticalCenter: parent.verticalCenter
            text: "Refresh"
            icon: "refresh-cw"
            flat: true
            onClicked: page.load()
        }
    }

    // Reading
    Text {
        visible: page.waiting && page.result === null
        text: "Reading anirss…"
        color: theme.textDim
        font.family: theme.fontUi
        font.pointSize: theme.typeNormal
    }

    // The four outcomes. EmptyState centers itself, and an anchored child
    // inside a positioner breaks the positioner's layout, so each one sits in
    // a plain wrapper the Column can position.
    Item {
        width: parent.width
        height: emptyNoneYet.height
        visible: emptyNoneYet.visible
        EmptyState {
            id: emptyNoneYet
            visible: !page.waiting && page.result !== null && page.result.kind === "Ok" && page.result.feeds.length === 0
            icon: "rss"
            title: "No subscriptions yet"
            body: "Run anirss in a terminal to subscribe to a feed."
        }
    }
    Item {
        width: parent.width
        height: emptyMissing.height
        visible: emptyMissing.visible
        EmptyState {
            id: emptyMissing
            visible: !page.waiting && page.result !== null && page.result.kind === "Missing"
            icon: "circle-alert"
            title: "anirss is not installed"
            body: "AniBeam reads subscriptions through anirss, which it did not find on PATH."
        }
    }
    Item {
        width: parent.width
        height: emptyAuth.height
        visible: emptyAuth.visible
        EmptyState {
            id: emptyAuth
            visible: !page.waiting && page.result !== null && page.result.kind === "NeedsAuth"
            icon: "log-in"
            title: "No qBittorrent session"
            body: "Run anirss -Sy in a terminal once to log in; AniBeam picks up the cached session."
        }
    }
    Item {
        width: parent.width
        height: emptyTimeout.height
        visible: emptyTimeout.visible
        EmptyState {
            id: emptyTimeout
            visible: !page.waiting && page.result !== null && (page.result.kind === "Timeout" || page.result.kind === "Error")
            icon: "clock"
            title: "anirss did not answer"
            body: page.result && page.result.message ? page.result.message : "Is qBittorrent reachable?"
        }
    }

    Column {
        width: parent.width
        spacing: 0
        visible: !page.waiting && page.result !== null && page.result.kind === "Ok" && page.result.feeds.length > 0
        Repeater {
            model: page.result && page.result.kind === "Ok" ? page.result.feeds : []
            SubscriptionRow { }
        }
    }

    component SubscriptionRow: Item {
        id: row
        required property var modelData
        readonly property var f: modelData || ({})
        width: parent.width
        height: feedGrid.implicitHeight + theme.space(6)

        Rectangle { anchors.fill: parent; color: m.containsMouse ? theme.surface : "transparent" }

        Column {
            id: feedGrid
            anchors.left: parent.left; anchors.leftMargin: theme.space(2)
            anchors.right: parent.right; anchors.rightMargin: theme.space(2)
            anchors.verticalCenter: parent.verticalCenter
            spacing: theme.space(1.5)

            Row {
                width: parent.width
                spacing: theme.space(3)
                Icon {
                    glyph: "rss"
                    size: theme.space(4)
                    color: row.f.active ? theme.accent : theme.textFaint
                    anchors.verticalCenter: parent.verticalCenter
                }
                Text {
                    text: row.f.name
                    color: theme.text
                    font.family: theme.fontDisplay
                    font.pointSize: theme.typeNormal
                    font.weight: Font.DemiBold
                    elide: Text.ElideRight
                    width: parent.width - activeChip.width - torrentText.width - parent.spacing * 2
                    anchors.verticalCenter: parent.verticalCenter
                }
                Chip {
                    id: activeChip
                    anchors.verticalCenter: parent.verticalCenter
                    text: row.f.active ? "Active" : "Paused"
                    textColor: row.f.active ? theme.caughtUp : theme.textFaint
                }
                Text {
                    id: torrentText
                    anchors.verticalCenter: parent.verticalCenter
                    text: Fmt.plural(row.f.torrents, "torrent", "torrents")
                    color: theme.textDim
                    font.family: theme.fontUi
                    font.pointSize: theme.typeSmall
                }
            }
            Row {
                visible: row.f.query !== ""
                spacing: theme.space(2)
                Text {
                    text: "query"
                    color: theme.textFaint
                    font.family: theme.fontUi
                    font.pointSize: theme.typeSmall
                    anchors.verticalCenter: parent.verticalCenter
                }
                Text {
                    text: row.f.query
                    color: theme.textDim
                    font.family: theme.fontUi
                    font.pointSize: theme.typeSmall
                    anchors.verticalCenter: parent.verticalCenter
                }
            }
            Row {
                visible: row.f.save_path !== ""
                spacing: theme.space(2)
                Icon {
                    glyph: "folder"
                    size: theme.space(3.5)
                    color: theme.textFaint
                    anchors.verticalCenter: parent.verticalCenter
                }
                Text {
                    text: row.f.save_path
                    color: theme.textFaint
                    font.family: theme.fontUi
                    font.pointSize: theme.typeSmall
                    elide: Text.ElideMiddle
                    anchors.verticalCenter: parent.verticalCenter
                }
            }
            Row {
                visible: row.f.url !== ""
                spacing: theme.space(1.5)
                Icon {
                    glyph: "external-link"
                    size: theme.space(3.5)
                    color: theme.accent
                    anchors.verticalCenter: parent.verticalCenter
                }
                Text {
                    text: "Open the feed"
                    color: theme.accent
                    font.family: theme.fontUi
                    font.pointSize: theme.typeSmall
                    anchors.verticalCenter: parent.verticalCenter
                    MouseArea {
                        anchors.fill: parent
                        cursorShape: Qt.PointingHandCursor
                        onClicked: Qt.openUrlExternally(row.f.url)
                    }
                }
            }
        }

        Rectangle {
            anchors.bottom: parent.bottom
            width: parent.width
            height: 1
            color: theme.line
        }
        MouseArea {
            id: m
            anchors.fill: parent
            hoverEnabled: true
        }
    }
}
