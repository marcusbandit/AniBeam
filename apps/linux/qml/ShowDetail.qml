// One show inside the downloads page, opened from its subscription or its downloads: what
// the Feed button used to send the person to a browser for. The poster at the left with
// Open in library under it; at the right the title, one sentence on where the show stands,
// every episode as a cell (on disk, or not), and the ways out (Unsubscribe, Remove
// torrents), each opening the removal sheet. Under that, side by side when the
// width holds both: what the subscription's feed holds right now, each release marked
// with what the client has of it (a missing one can be fetched from here), and the
// torrents the client holds for the show's folder.
import QtQuick
import com.marcusrosado.AniBeam

Column {
    id: root
    property var rule: null
    property var show: null
    property string title: ""
    property string folder: ""
    property var torrents: []
    // The feed as last read: { url, entries, loading, error }
    property var feed: null
    // Every torrent in the client, by lower case hash and by name, to mark feed entries
    property var clientIndex: ({ byHash: {}, byName: {} })
    property real nowMs: Date.now()
    property bool removing: false
    signal back()
    signal unsubscribe(Item anchor)
    signal removeAll(Item anchor)
    signal removeTorrent(var torrent, Item anchor)
    signal control(string hash, string action)
    signal downloadEntry(var entry)
    signal refreshFeed()

    readonly property int total: show && show.total_episodes && !show.total_is_estimate ? show.total_episodes : 0
    readonly property var missing: show && show.missing ? show.missing : []
    readonly property bool complete: !!show && show.complete === true
    readonly property bool finished: !!show && (show.status === "Finished" || show.status === "Cancelled")
    readonly property bool airing: !!show && !!show.next_airing && show.next_airing.at * 1000 > nowMs
    readonly property bool wide: width >= theme.space(220)
    readonly property real gap: theme.space(6)
    readonly property var entries: feed && feed.entries ? feed.entries : []

    readonly property string standing: {
        if (!show) return "This folder is not in the library yet. Once a scan finds it, its poster and episodes show here."
        if (show.kind === "Movie") return complete ? "The film is on disk." : "The film is not on disk yet."
        var have = show.episodes_on_disk || 0
        if (complete) return "Finished airing, and all " + total + " episodes are here. Nothing more will come, so the subscription has nothing left to do."
        var parts = []
        if (finished) parts.push("Finished airing.")
        else if (airing) parts.push("Airing.")
        if (total > 0) parts.push(have + " of " + total + " episodes here.")
        else parts.push(Fmt.plural(have, "episode", "episodes") + " here.")
        if (airing) parts.push("Episode " + show.next_airing.episode + " airs in " + Fmt.countdown(show.next_airing.at - nowMs / 1000) + ".")
        if (finished && missing.length) parts.push((missing.length === 1 ? "Episode " : "Episodes ") + missing.slice(0, 8).join(", ") + (missing.length > 8 ? " and " + (missing.length - 8) + " more" : "") + (missing.length === 1 ? " is" : " are") + " not here.")
        return parts.join(" ")
    }
    function entryState(e) {
        var t = (e.info_hash && root.clientIndex.byHash[e.info_hash]) || root.clientIndex.byName[e.title]
        if (!t) return { word: "", torrent: null }
        if ((t.progress || 0) >= 1) return { word: "Downloaded", torrent: t }
        return { word: "Downloading " + Math.round((t.progress || 0) * 100) + "%", torrent: t }
    }

    spacing: theme.space(5)
    opacity: removing ? 0.6 : 1

    // ---- The head: Back, the title
    Row {
        width: parent.width
        spacing: theme.space(3)
        Button { objectName: "show-back"; text: "Back"; icon: "arrow-left"; flat: true; small: true; anchors.verticalCenter: parent.verticalCenter; onClicked: root.back() }
        Label {
            anchors.verticalCenter: parent.verticalCenter
            width: Math.min(implicitWidth, root.width - theme.space(40))
            text: root.title
            role: "display"
            font.pointSize: theme.typeLarge
            elide: Text.ElideRight
        }
        Rectangle {
            visible: root.complete
            anchors.verticalCenter: parent.verticalCenter
            radius: theme.radiusSm
            width: completeRow.implicitWidth + theme.space(4); height: completeRow.implicitHeight + theme.space(1.5)
            color: theme.raised
            border.color: theme.line
            border.width: 1
            Row {
                id: completeRow
                anchors.centerIn: parent
                spacing: theme.space(1.25)
                Rectangle { anchors.verticalCenter: parent.verticalCenter; width: theme.space(2); height: width; radius: width / 2; color: theme.red }
                Label { text: "Complete"; role: "ledger"; color: theme.text }
            }
        }
    }

    // ---- The poster at the left, the standing and the actions at the right
    Item {
        width: parent.width
        height: Math.max(posterCol.height, info.implicitHeight)
        Column {
            id: posterCol
            width: theme.space(52)
            spacing: theme.space(3)
            Corner {
                id: posterTile
                width: parent.width
                height: width * 1.5
                radius: theme.radiusTile
                color: theme.tile
                borderColor: theme.line
                borderWidth: 1
                Image {
                    id: posterImage
                    visible: false
                    anchors.fill: parent
                    source: root.show && root.show.poster ? "file://" + root.show.poster : ""
                    sourceSize: Qt.size(theme.posterDecodeSize.width * 2, theme.posterDecodeSize.height * 2)
                    fillMode: Image.PreserveAspectCrop
                    smooth: true
                    asynchronous: true
                }
                fillItem: posterImage.status === Image.Ready ? posterImage : null
                Icon { visible: posterImage.status !== Image.Ready; anchors.centerIn: parent; glyph: root.rule ? "rss" : "folder"; size: theme.space(12); color: theme.textFaint }
            }
            Button {
                visible: !!root.show
                width: parent.width
                text: "Open in library"
                icon: "layers"
                onClicked: frame.go("series", { id: root.show.series_id }, root.show.title)
            }
        }
        Column {
            id: info
            x: posterCol.width + root.gap
            width: parent.width - x
            spacing: theme.space(4)
            Label {
                width: Math.min(parent.width, theme.space(200))
                text: root.standing
                role: "ui"
                font.pointSize: theme.typeNormal * 1.08
                color: root.complete ? theme.text : theme.textDim
                wrapMode: Text.Wrap
            }
            // Every episode as a cell: on disk in the accent, not here as an outline, red once
            // the show has finished and it is still missing
            Flow {
                visible: root.total > 0 && root.show.kind !== "Movie"
                width: parent.width
                spacing: theme.space(1)
                Repeater {
                    model: Math.min(root.total, 400)
                    Rectangle {
                        required property int index
                        readonly property int number: index + 1
                        readonly property bool here: root.missing.indexOf(number) < 0
                        width: theme.space(7.5); height: theme.space(6)
                        radius: theme.radiusSm
                        color: here ? theme.accentSoft : "transparent"
                        border.color: here ? theme.accentEdge : root.finished ? Qt.rgba(theme.red.r, theme.red.g, theme.red.b, 0.6) : theme.line
                        border.width: 1
                        Label {
                            anchors.centerIn: parent
                            text: parent.number
                            role: "ledger"
                            font.pointSize: theme.typeSmall
                            color: parent.here ? theme.accent : root.finished ? theme.red : theme.textFaint
                        }
                    }
                }
            }
            // The ways out, each through the removal sheet
            Row {
                spacing: theme.space(2)
                Button {
                    id: unsubscribeButton
                    objectName: "show-unsubscribe"
                    visible: !!root.rule
                    enabled: !root.removing
                    text: "Unsubscribe"
                    icon: "rss"
                    onClicked: root.unsubscribe(unsubscribeButton)
                }
                Button {
                    id: removeAllButton
                    objectName: "show-remove-torrents"
                    visible: root.torrents.length > 0
                    enabled: !root.removing
                    text: root.torrents.length === 1 ? "Remove the torrent" : "Remove " + root.torrents.length + " torrents"
                    icon: "x"
                    flat: true
                    onClicked: root.removeAll(removeAllButton)
                }
            }
            // The subscription's facts
            Grid {
                visible: !!root.rule
                columns: 2
                columnSpacing: theme.space(5)
                rowSpacing: theme.space(1.5)
                Label { text: "Searches"; role: "ledger"; color: theme.textFaint }
                Label { text: root.rule && root.rule.query ? "“" + root.rule.query + "”" : "the whole feed"; role: "ui"; color: theme.text; elide: Text.ElideRight; width: Math.min(implicitWidth, info.width - theme.space(40)) }
                Label { text: "From"; role: "ledger"; color: theme.textFaint }
                Label { text: root.rule ? root.feedHost(root.rule.feed_url) : ""; role: "ui"; color: theme.textDim }
                Label { text: "Rule"; role: "ledger"; color: theme.textFaint }
                Label { text: root.rule && root.rule.rule_enabled === false ? "Paused" : "Active"; role: "ui"; color: root.rule && root.rule.rule_enabled === false ? theme.textFaint : theme.text }
                Label { text: "Saves to"; role: "ledger"; color: theme.textFaint }
                Label { text: root.folder; role: "ledger"; color: theme.textDim; elide: Text.ElideMiddle; width: Math.min(implicitWidth, info.width - theme.space(40)) }
            }
            Grid {
                visible: !root.rule
                columns: 2
                columnSpacing: theme.space(5)
                Label { text: "Folder"; role: "ledger"; color: theme.textFaint }
                Label { text: root.folder; role: "ledger"; color: theme.textDim; elide: Text.ElideMiddle; width: Math.min(implicitWidth, info.width - theme.space(40)) }
            }
        }
    }
    function feedHost(url) {
        var u = String(url || "")
        var m = u.match(/^[a-z]+:\/\/([^\/?#]+)/i)
        return m ? m[1] : u
    }

    // ---- The feed and the torrents
    Item {
        width: parent.width
        height: root.wide ? Math.max(feedCol.height, torrentCol.height) : feedCol.height + torrentCol.height + root.gap
        Column {
            id: feedCol
            visible: !!root.rule
            width: !root.rule ? 0 : root.wide ? Math.floor((parent.width - root.gap) * 0.55) : parent.width
            height: visible ? implicitHeight : 0
            spacing: theme.space(1)
            Item {
                width: parent.width
                height: theme.space(11)
                Row {
                    anchors.left: parent.left
                    anchors.bottom: parent.bottom
                    anchors.bottomMargin: theme.space(2)
                    spacing: theme.space(3)
                    Label { id: feedTitle; text: "In the feed"; role: "display"; font.pointSize: theme.typeLarge * 0.85; font.variableAxes: ({ "wdth": 74, "wght": 500 }) }
                    Label { anchors.baseline: feedTitle.baseline; visible: root.entries.length > 0; text: root.entries.length; role: "ledger"; color: theme.textFaint }
                    Label {
                        anchors.baseline: feedTitle.baseline
                        text: !root.feed || root.feed.loading ? "Reading the feed." : root.feed.error ? root.feed.error : root.entries.length ? "What the feed offers now, newest first." : "The feed is empty."
                        role: "ui"
                        color: root.feed && root.feed.error ? theme.red : theme.textDim
                    }
                }
                Button { anchors.right: parent.right; anchors.bottom: parent.bottom; anchors.bottomMargin: theme.space(1); text: "Read again"; icon: "refresh-cw"; flat: true; small: true; enabled: !!root.feed && !root.feed.loading; onClicked: root.refreshFeed() }
            }
            Repeater {
                model: root.entries
                Item {
                    id: entryRow
                    required property var modelData
                    readonly property var st: root.entryState(modelData)
                    width: feedCol.width
                    height: theme.space(13)
                    Rectangle { anchors.fill: parent; radius: theme.radiusSm; color: eh.hovered ? theme.raised : "transparent" }
                    HoverHandler { id: eh }
                    Column {
                        anchors.left: parent.left
                        anchors.leftMargin: theme.space(2)
                        anchors.right: entryEnd.left
                        anchors.rightMargin: theme.space(3)
                        anchors.verticalCenter: parent.verticalCenter
                        spacing: theme.space(0.75)
                        Label { width: parent.width; text: entryRow.modelData.title; role: "ui"; elide: Text.ElideMiddle }
                        Label {
                            width: parent.width
                            text: {
                                var e = entryRow.modelData, parts = []
                                if (e.published) parts.push(Fmt.relative(e.published, root.nowMs / 1000))
                                if (e.size) parts.push(e.size)
                                if (e.seeders !== null && e.seeders !== undefined) parts.push(Fmt.plural(e.seeders, "seeder", "seeders"))
                                return parts.join(", ")
                            }
                            role: "ledger"
                            color: theme.textDim
                            elide: Text.ElideRight
                        }
                    }
                    Item {
                        id: entryEnd
                        anchors.right: parent.right
                        anchors.rightMargin: theme.space(2)
                        anchors.verticalCenter: parent.verticalCenter
                        width: Math.max(stateWord.implicitWidth, getButton.implicitWidth)
                        height: theme.controlHeight
                        Label { id: stateWord; visible: entryRow.st.word !== ""; anchors.right: parent.right; anchors.verticalCenter: parent.verticalCenter; text: entryRow.st.word; role: "ledger"; color: theme.accent }
                        Button { id: getButton; visible: entryRow.st.word === "" && !!entryRow.modelData.link; anchors.right: parent.right; anchors.verticalCenter: parent.verticalCenter; text: "Download"; icon: "download"; flat: true; small: true; onClicked: root.downloadEntry(entryRow.modelData) }
                    }
                    Rectangle { anchors.bottom: parent.bottom; x: theme.space(2); width: parent.width - theme.space(4); height: 1; color: theme.line; opacity: 0.5 }
                }
            }
        }
        Column {
            id: torrentCol
            x: root.wide && root.rule ? feedCol.width + root.gap : 0
            y: root.wide || !root.rule ? 0 : feedCol.height + root.gap
            width: root.wide && root.rule ? parent.width - feedCol.width - root.gap : parent.width
            spacing: theme.space(1)
            Item {
                width: parent.width
                height: theme.space(11)
                Row {
                    anchors.left: parent.left
                    anchors.bottom: parent.bottom
                    anchors.bottomMargin: theme.space(2)
                    spacing: theme.space(3)
                    Label { id: trTitle; text: "Torrents"; role: "display"; font.pointSize: theme.typeLarge * 0.85; font.variableAxes: ({ "wdth": 74, "wght": 500 }) }
                    Label { anchors.baseline: trTitle.baseline; visible: root.torrents.length > 0; text: root.torrents.length; role: "ledger"; color: theme.textFaint }
                    Label { anchors.baseline: trTitle.baseline; text: root.torrents.length ? "What the client holds for this folder." : "The client holds none for this folder."; role: "ui"; color: theme.textDim }
                }
            }
            Repeater {
                model: root.torrents
                TorrentRow {
                    required property var modelData
                    width: torrentCol.width
                    torrent: modelData
                    onStop: root.control(modelData.hash, "Stop")
                    onStart: root.control(modelData.hash, "Start")
                    onRemoveAsked: function(anchor) { root.removeTorrent(modelData, anchor) }
                }
            }
        }
    }
}
