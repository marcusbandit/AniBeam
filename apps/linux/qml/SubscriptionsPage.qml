// The subscriptions: what anirss watches for the user, read-only, reached from the Library
// settings tab's Open button, as tiles on the library's module. ListSubscriptions starts a
// job that shells out to anirss; SubscriptionsListed carries the result, and its four
// shapes are the page's four states: feeds, not installed, no qBittorrent session, or no
// answer. A job that fails, is cancelled, or never answers ends the wait as "no answer".
import QtQuick
import com.marcusrosado.AniBeam
import "mosaic.js" as Mosaic

Item {
    id: page
    objectName: "subscriptions-page"
    property var props: ({})
    property string title: "Subscriptions"
    property alias scrollY: wall.scrollY
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

    FontMetrics { id: smallMetrics; font.family: theme.fontFamily; font.pointSize: theme.typeSmall }

    MosaicWall {
        id: wall
        anchors.fill: parent

        MosaicWall.HeaderTile {
            need: titleCol.implicitWidth
            minSpan: 2
            flex: true
            Column {
                id: titleCol
                x: parent.pad
                anchors.verticalCenter: parent.verticalCenter
                spacing: theme.space(1)
                Label { text: "Subscriptions"; role: "display" }
                Label { text: "The feeds you subscribe to, and where each one saves."; role: "ui"; color: theme.textDim }
            }
        }
        MosaicWall.HeaderTile {
            visible: page.feeds.length > 0
            need: countRow.implicitWidth
            Row {
                id: countRow
                anchors.right: parent.right; anchors.rightMargin: parent.pad
                anchors.verticalCenter: parent.verticalCenter
                spacing: theme.space(1.5)
                Label {
                    id: countFigure
                    text: page.feeds.length
                    role: "ledger"
                    font.pointSize: theme.typeDisplay
                    font.weight: 700
                    font.variableAxes: ({ "wdth": 88, "wght": 700 })
                }
                Label {
                    anchors.baseline: countFigure.baseline
                    text: page.feeds.length === 1 ? "feed" : "feeds"
                    role: "ui"
                    font.pointSize: theme.typeSmall
                    color: theme.textDim
                }
            }
        }
        MosaicWall.HeaderTile {
            need: refreshBtn.implicitWidth
            Button {
                id: refreshBtn
                objectName: "subscriptions-refresh"
                x: parent.pad
                anchors.verticalCenter: parent.verticalCenter
                text: "Refresh"
                icon: "refresh-cw"
                enabled: !page.waiting
                onClicked: page.load()
            }
        }

        body: page.feeds.length > 0 ? feedTiles : stateTile
    }

    // The wait and the four outcomes other than feeds: one tile filling the page
    Component {
        id: stateTile
        Tile {
            id: st
            implicitHeight: theme.space(60)
            readonly property string kind: page.result ? page.result.kind : ""
            EmptyState {
                visible: page.waiting || page.result === null
                icon: "rss"
                title: "Reading your subscriptions"
                body: "This takes a few seconds."
            }
            EmptyState {
                visible: !page.waiting && st.kind === "Ok"
                icon: "rss"
                title: "No subscriptions yet"
                body: "Add a feed in anirss, then come back here."
            }
            EmptyState {
                visible: !page.waiting && st.kind === "Missing"
                icon: "circle-alert"
                title: "Subscriptions need anirss"
                body: "anirss was not found on your system. Install it to see your subscriptions."
            }
            EmptyState {
                visible: !page.waiting && st.kind === "NeedsAuth"
                icon: "log-in"
                title: "Not logged in to qBittorrent"
                body: "Run anirss once in a terminal and log in to qBittorrent. AniBeam will pick up that login."
            }
            EmptyState {
                visible: !page.waiting && (st.kind === "Timeout" || st.kind === "Error")
                icon: "clock"
                title: "Subscriptions did not load"
                body: page.result && page.result.message ? page.result.message : "Check that qBittorrent is running."
                Button { text: "Try again"; icon: "refresh-cw"; onClicked: page.load() }
            }
        }
    }

    // The feeds: one tile each, as many across as fit at three columns or more, the rows
    // stretched on whole units to fill the viewport when they would leave it half empty
    Component {
        id: feedTiles
        Item {
            id: grid
            readonly property int span: Math.min(wall.geo.cols, Math.max(3, wall.spanFor(theme.space(110))))
            readonly property int across: Math.max(1, Math.floor(wall.geo.cols / span))
            readonly property int rows: Math.ceil(page.feeds.length / across)
            readonly property real lineH: smallMetrics.height + theme.space(1.5)
            readonly property int needUnits: Mosaic.unitsFor(theme.space(10) + theme.controlHeight * 2 + lineH * 3, wall.unit, wall.grout)
            readonly property int fitUnits: Math.floor((height + wall.grout) / wall.step / Math.max(1, rows))
            readonly property int rowUnits: Math.max(needUnits, fitUnits)
            implicitHeight: rows * (wall.px(needUnits) + wall.grout) - wall.grout
            Repeater {
                model: page.feeds
                Tile {
                    id: feedTile
                    required property var modelData
                    required property int index
                    readonly property int col: index % grid.across
                    readonly property int row: Math.floor(index / grid.across)
                    // the row's columns shared out whole, the last tile taking the remainder
                    readonly property int c0: Math.floor(col * wall.geo.cols / grid.across)
                    readonly property int c1: Math.floor((col + 1) * wall.geo.cols / grid.across)
                    readonly property var f: modelData || ({})
                    x: wall.spanX(c0)
                    y: row * (wall.px(grid.rowUnits) + wall.grout)
                    width: wall.spanW(c0, c1 - c0)
                    height: wall.px(grid.rowUnits)
                    Column {
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
                                width: parent.width - theme.space(5) - activeChip.width - parent.spacing * 2
                                anchors.verticalCenter: parent.verticalCenter
                            }
                            Chip {
                                id: activeChip
                                anchors.verticalCenter: parent.verticalCenter
                                text: feedTile.f.active ? "Active" : "Paused"
                                textColor: feedTile.f.active ? theme.caughtUp : theme.textFaint
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
                    }
                    Button {
                        visible: !!feedTile.f.url
                        anchors.left: parent.left
                        anchors.bottom: parent.bottom
                        text: "Open the feed"
                        icon: "external-link"
                        flat: true
                        small: true
                        onClicked: Qt.openUrlExternally(feedTile.f.url)
                    }
                }
            }
        }
    }
}
