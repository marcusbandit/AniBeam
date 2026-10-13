// The subscriptions: what anirss watches for the user, read-only, reached from the Library
// settings tab's Open button, as sections of uniform flat tiles. ListSubscriptions starts
// a job that shells out to anirss; SubscriptionsListed carries the result, and its four
// shapes are the page's four states: feeds, not installed, no qBittorrent session, or no
// answer. A job that fails, is cancelled, or never answers ends the wait as "no answer".
// The feeds sit in two sections, Active and Paused, each a grid of tiles whose column
// count comes from the width.
import QtQuick
import com.marcusrosado.AniBeam

Item {
    id: page
    objectName: "subscriptions-page"
    property var props: ({})
    property string title: "Subscriptions"
    property alias scrollY: scroll.contentY
    function escapePressed() { return false }

    property var result: null
    property bool waiting: false
    property real job: -1
    readonly property var feeds: result && result.kind === "Ok" && result.feeds ? result.feeds : []

    function fail(message) {
        page.result = { kind: "Error", message: message || "" }
        page.waiting = false
        page.job = -1
        giveUp.stop()
    }
    function load() {
        page.result = null
        var r = Door.listSubscriptions()
        if (r.error) { fail(r.error.message); return }
        page.job = r.reply && r.reply.job !== undefined ? r.reply.job : -1
        page.waiting = true
        giveUp.restart()
    }
    // anirss has its own timeout; this is the backstop for a job that never reports
    Timer { id: giveUp; interval: 60000; onTriggered: if (page.waiting) page.fail("No answer from anirss after a minute. Check that qBittorrent is running.") }
    Connections {
        target: Door
        function onSubscriptionsListed(res) { page.result = res; page.waiting = false; page.job = -1; giveUp.stop() }
        function onJobFailed(job, kind, error) { if (page.waiting && job === page.job) page.fail(error && error.message ? error.message : "") }
        function onJobCancelled(job, kind) { if (page.waiting && job === page.job) page.fail("The read was cancelled.") }
        function onJobFinished(job, kind, ok) { if (page.waiting && job === page.job && !ok) page.fail("") }
    }
    Component.onCompleted: load()

    // The sections: the active feeds, then the paused ones
    readonly property var sections: {
        var active = feeds.filter(function(f) { return f.active })
        var paused = feeds.filter(function(f) { return !f.active })
        var out = []
        if (active.length) out.push({ title: "Active", cards: active, accent: true, fact: Fmt.plural(active.reduce(function(n, f) { return n + (f.torrents || 0) }, 0), "torrent", "torrents") })
        if (paused.length) out.push({ title: "Paused", cards: paused, accent: false, fact: "" })
        return out
    }
    // The head's one line
    readonly property string summary: {
        if (page.waiting || page.result === null) return "Reading your subscriptions."
        if (feeds.length === 0) return "The feeds you subscribe to, and where each one saves."
        var active = feeds.filter(function(f) { return f.active }).length
        return active === feeds.length ? "All active." : active + " of " + feeds.length + " active."
    }
    readonly property string kind: page.result ? page.result.kind : ""

    // ---- The grid's geometry: as many columns as the width holds at the tile width
    readonly property real gap: theme.space(4)
    readonly property real tileMin: theme.space(90)
    readonly property int cols: Math.max(1, Math.floor((scroll.innerWidth + gap) / (tileMin + gap)))
    readonly property real tileWidth: Math.floor((scroll.innerWidth - gap * (cols - 1)) / cols)

    PageScroll {
        id: scroll
        anchors.fill: parent
        readonly property real innerWidth: width - theme.space(16)

        // ---- The head: the title, the count and the one line beside it, Refresh at the right
        Item {
            width: parent.width
            height: theme.space(14)
            Label {
                id: titleLabel
                anchors.left: parent.left
                anchors.verticalCenter: parent.verticalCenter
                text: "Subscriptions"
                role: "display"
            }
            Label {
                id: countLabel
                visible: page.feeds.length > 0
                anchors.left: titleLabel.right
                anchors.leftMargin: theme.space(3)
                anchors.baseline: titleLabel.baseline
                text: Fmt.plural(page.feeds.length, "feed", "feeds")
                role: "ui"
                color: theme.textDim
            }
            Label {
                objectName: "subscriptions-summary"
                anchors.left: countLabel.visible ? countLabel.right : titleLabel.right
                anchors.leftMargin: theme.space(3)
                anchors.right: refreshBtn.left
                anchors.rightMargin: theme.space(4)
                anchors.baseline: titleLabel.baseline
                text: page.summary
                role: "ui"
                color: theme.textDim
                elide: Text.ElideRight
            }
            Button {
                id: refreshBtn
                objectName: "subscriptions-refresh"
                anchors.right: parent.right
                anchors.verticalCenter: parent.verticalCenter
                text: page.waiting ? "Reading" : "Refresh"
                icon: "refresh-cw"
                flat: true
                enabled: !page.waiting
                onClicked: page.load()
            }
        }

        // ---- The sections: a heading in the display cut with its count and one fact,
        // over a grid of uniform tiles
        Repeater {
            model: page.sections
            Column {
                id: section
                required property var modelData
                required property int index
                objectName: "subscriptions-section-" + String(modelData.title).toLowerCase()
                width: parent.width
                spacing: theme.space(3)
                topPadding: index > 0 ? theme.space(2) : 0
                Item {
                    width: parent.width
                    height: theme.space(12)
                    Row {
                        anchors.left: parent.left
                        anchors.bottom: parent.bottom
                        anchors.bottomMargin: theme.space(2)
                        spacing: theme.space(3)
                        Label {
                            id: sectionTitle
                            text: section.modelData.title
                            role: "display"
                            font.pointSize: theme.typeLarge
                            font.variableAxes: ({ "wdth": 74, "wght": 500 })
                            color: section.modelData.accent ? theme.accent : theme.text
                        }
                        Label { anchors.baseline: sectionTitle.baseline; text: section.modelData.cards.length; role: "ledger"; color: theme.textFaint }
                        Label { anchors.baseline: sectionTitle.baseline; text: section.modelData.fact; role: "ui"; color: theme.textDim }
                    }
                }
                Flow {
                    width: parent.width
                    spacing: page.gap
                    Repeater {
                        model: section.modelData.cards
                        // One feed: its name with the state beside it, the torrent count,
                        // the match, the save path, and a link to the feed
                        Tile {
                            id: feedTile
                            required property var modelData
                            readonly property var f: modelData || ({})
                            width: page.tileWidth
                            height: body.implicitHeight + padding * 2
                            lit: false
                            Column {
                                id: body
                                width: parent.width
                                spacing: theme.space(1.5)
                                Row {
                                    width: parent.width
                                    height: theme.controlHeight
                                    spacing: theme.space(3)
                                    Icon {
                                        glyph: "rss"
                                        size: theme.space(5)
                                        color: feedTile.f.active ? theme.accent : theme.textFaint
                                        anchors.verticalCenter: parent.verticalCenter
                                    }
                                    Label {
                                        text: feedTile.f.name || ""
                                        role: "title"
                                        elide: Text.ElideRight
                                        width: parent.width - theme.space(5) - stateLabel.width - parent.spacing * 2
                                        anchors.verticalCenter: parent.verticalCenter
                                    }
                                    Label {
                                        id: stateLabel
                                        anchors.verticalCenter: parent.verticalCenter
                                        text: feedTile.f.active ? "Active" : "Paused"
                                        role: "ledger"
                                        color: feedTile.f.active ? theme.caughtUp : theme.textFaint
                                    }
                                }
                                Label {
                                    text: Fmt.plural(feedTile.f.torrents || 0, "torrent", "torrents")
                                    role: "ledger"
                                    color: theme.textDim
                                }
                                Label {
                                    visible: !!feedTile.f.query
                                    width: parent.width
                                    text: "Matches \"" + (feedTile.f.query || "") + "\""
                                    role: "ledger"
                                    color: theme.textDim
                                    elide: Text.ElideRight
                                }
                                Row {
                                    visible: !!feedTile.f.save_path
                                    width: parent.width
                                    spacing: theme.space(2)
                                    Icon { glyph: "folder"; size: theme.space(3.5); color: theme.textFaint; anchors.verticalCenter: parent.verticalCenter }
                                    Label {
                                        width: parent.width - theme.space(5.5)
                                        text: feedTile.f.save_path || ""
                                        role: "ledger"
                                        color: theme.textFaint
                                        elide: Text.ElideMiddle
                                        anchors.verticalCenter: parent.verticalCenter
                                    }
                                }
                                Item { width: 1; height: theme.space(1) }
                                Button {
                                    visible: !!feedTile.f.url
                                    text: "Open the feed"
                                    flat: true
                                    small: true
                                    onClicked: Qt.openUrlExternally(feedTile.f.url)
                                }
                            }
                        }
                    }
                }
            }
        }

        // ---- The wait and the four outcomes other than feeds, in a plain box in the column
        Item {
            visible: page.feeds.length === 0
            width: parent.width
            height: visible ? theme.space(60) : 0
            EmptyState {
                visible: page.waiting || page.result === null
                icon: "rss"
                title: "Reading your subscriptions"
                body: "This takes a few seconds."
            }
            EmptyState {
                visible: !page.waiting && page.kind === "Ok"
                icon: "rss"
                title: "No subscriptions yet"
                body: "Add a feed in anirss, then come back here."
            }
            EmptyState {
                visible: !page.waiting && page.kind === "Missing"
                icon: "circle-alert"
                title: "Subscriptions need anirss"
                body: "anirss was not found on your system. Install it to see your subscriptions."
            }
            EmptyState {
                visible: !page.waiting && page.kind === "NeedsAuth"
                icon: "log-in"
                title: "Not logged in to qBittorrent"
                body: "Run anirss once in a terminal and log in to qBittorrent. AniBeam will pick up that login."
            }
            EmptyState {
                visible: !page.waiting && (page.kind === "Timeout" || page.kind === "Error")
                icon: "clock"
                title: "Subscriptions did not load"
                body: page.result && page.result.message ? page.result.message : "Check that qBittorrent is running."
                Button { text: "Try again"; icon: "refresh-cw"; onClicked: page.load() }
            }
        }
    }
}
