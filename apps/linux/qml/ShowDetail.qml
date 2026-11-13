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
    // The subscription's rule as the client holds it ({ must_contain, must_not_contain,
    // save_path, last_match, matching: [titles the rule accepts] }), or null until read
    property var ruleView: null
    // A subscription made a moment ago: the page is reading the client quickly
    property bool watching: false
    property bool justSubscribed: false
    // A repair running, and the step it is on
    // The endpoint the feed belongs to, by name ("proxied"), or empty for the page to fall back
    // on the feed's host
    property string source: ""
    property bool repairing: false
    property string repairLabel: ""
    signal repair()
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
        if (!show) return words.t("show-detail-not-in-library")
        if (show.kind === "Movie") return complete ? words.t("show-detail-film-on-disk") : words.t("show-detail-film-not-on-disk")
        var have = show.episodes_on_disk || 0
        if (complete) return words.t("show-detail-complete-line", { total: total })
        var parts = []
        if (finished) parts.push(words.t("show-detail-finished"))
        else if (airing) parts.push(words.t("show-detail-airing"))
        if (total > 0) parts.push(words.t("show-detail-have-of-total", { have: have, total: total }))
        else parts.push(words.t("show-detail-have", { count: have }))
        if (airing) {
            var next = show.next_airing
            parts.push(words.t(next.predicted ? "show-detail-next-predicted" : "show-detail-next-airs", {
                span: Fmt.episodeSpan(next.episode, next.through || 0),
                count: next.through ? 2 : 1,
                wait: Fmt.countdown(next.at - nowMs / 1000)
            }))
        }
        if (finished && have > 0 && missing.length) {
            var shown = missing.slice(0, 8).join(", ")
            parts.push(missing.length > 8 ? words.t("show-detail-missing-more", { list: shown, more: missing.length - 8 })
                : words.t("show-detail-missing", { count: missing.length, list: shown }))
        }
        return parts.join(" ")
    }
    // Letters and digits, lower case: a torrent's name and its feed title differ in dots,
    // spaces and a trailing group more often than not
    function key(t) { return String(t || "").toLowerCase().replace(/[^a-z0-9]/g, "") }
    function torrentFor(e) {
        var t = (e.info_hash && root.clientIndex.byHash[e.info_hash]) || root.clientIndex.byName[e.title]
        if (t) return t
        var k = root.key(e.title)
        if (k.length < 12) return null
        for (var i = 0; i < root.torrents.length; i++) {
            var n = root.key(root.torrents[i].name)
            if (n.length >= 12 && (k.indexOf(n) === 0 || n.indexOf(k) === 0)) return root.torrents[i]
        }
        return null
    }
    readonly property var matchingKeys: {
        var m = {}
        var list = root.ruleView && root.ruleView.matching ? root.ruleView.matching : []
        for (var i = 0; i < list.length; i++) m[root.key(list[i])] = true
        return m
    }
    function entryState(e) {
        var t = root.torrentFor(e)
        if (t) {
            if ((t.progress || 0) >= 1) return { word: words.t("show-detail-downloaded"), ink: theme.accent, torrent: t }
            return { word: words.t("show-detail-downloading", { percent: Math.round((t.progress || 0) * 100) }), ink: theme.accent, torrent: t }
        }
        if (!root.ruleView) return { word: "", ink: theme.textDim, torrent: null }
        if (root.matchingKeys[root.key(e.title)]) return { word: root.watching ? words.t("show-detail-taken-adding") : words.t("show-detail-taken"), ink: theme.orange, torrent: null }
        return { word: words.t("show-detail-left-out"), ink: theme.textFaint, torrent: null }
    }
    // What the rule takes of the feed as it reads now (qBittorrent can still hold articles
    // an earlier version of the feed found; those are not counted)
    readonly property int takenCount: {
        var n = 0
        for (var i = 0; i < entries.length; i++) if (root.matchingKeys[root.key(entries[i].title)]) n++
        return n
    }
    readonly property bool feedEmpty: !!root.rule && !!root.feed && !root.feed.loading && !root.feed.error && root.entries.length === 0
    readonly property string feedQuery: {
        var u = String(root.rule && root.rule.feed_url ? root.rule.feed_url : (root.feed ? root.feed.url : ""))
        var m = u.match(/[?&](?:q|s)=([^&]*)/)
        if (!m) return ""
        try { return decodeURIComponent(m[1].replace(/\+/g, " ")) } catch (e) { return m[1] }
    }
    readonly property int movingCount: { var n = 0; for (var i = 0; i < torrents.length; i++) if ((torrents[i].progress || 0) < 1) n++; return n }
    // One line on what a fresh subscription is doing
    readonly property string settling: {
        if (!root.justSubscribed || !root.rule && !root.watching) return ""
        if (root.feedEmpty) return ""
        if (!root.ruleView || !root.feed || root.feed.loading) return words.t("show-detail-subscribed-reading")
        if (root.takenCount > 0) {
            var counts = { taken: root.takenCount, releases: root.entries.length }
            if (root.movingCount > 0) return words.t("show-detail-taken-moving", Object.assign({ moving: root.movingCount }, counts))
            if (root.torrents.length > 0) return words.t("show-detail-taken-held", Object.assign({ torrents: root.torrents.length }, counts))
            return words.t("show-detail-taken-adding-line", counts)
        }
        if (root.feed && !root.feed.loading && root.entries.length === 0) return words.t("show-detail-feed-holds-nothing")
        return words.t("show-detail-nothing-fits")
    }

    spacing: theme.space(5)
    opacity: removing ? 0.6 : 1

    // ---- The head: Back, the title
    Row {
        width: parent.width
        spacing: theme.space(3)
        Button { objectName: "show-back"; msg: "action-back"; icon: "arrow-left"; flat: true; small: true; anchors.verticalCenter: parent.verticalCenter; onClicked: root.back() }
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
                Label { msg: "show-detail-complete"; role: "ledger"; color: theme.text }
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
                    source: root.show && root.show.poster ? Fmt.fileUrl(root.show.poster) : ""
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
                msg: "show-detail-open-in-library"
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
            // A feed that finds nothing never feeds the rule: say so, and offer the repair
            Rectangle {
                visible: root.feedEmpty
                width: Math.min(parent.width, theme.space(200))
                height: emptyBody.implicitHeight + theme.space(4) * 2
                radius: theme.radiusSm
                color: Qt.rgba(theme.orange.r, theme.orange.g, theme.orange.b, 0.08)
                border.color: Qt.rgba(theme.orange.r, theme.orange.g, theme.orange.b, 0.45)
                border.width: 1
                Column {
                    id: emptyBody
                    x: theme.space(4)
                    y: theme.space(4)
                    width: parent.width - theme.space(8)
                    spacing: theme.space(2)
                    Label { msg: "show-detail-feed-finds-nothing"; role: "title"; font.pointSize: theme.typeNormal * 1.05; color: theme.text }
                    Label {
                        width: parent.width
                        text: {
                            var asked = { source: root.source !== "" ? root.source : root.feedHost(root.rule ? root.rule.feed_url : ""), query: root.feedQuery }
                            var line = words.t(root.feedQuery ? "show-detail-feed-empty-query" : "show-detail-feed-empty", asked)
                            return root.ruleView && root.ruleView.matching && root.ruleView.matching.length ? [line, words.t("show-detail-feed-empty-earlier")].join(" ") : line
                        }
                        role: "ui"
                        color: theme.textDim
                        wrapMode: Text.Wrap
                    }
                    Row {
                        spacing: theme.space(3)
                        Button {
                            objectName: "show-repair"
                            text: root.repairing ? words.t("show-detail-repairing") : words.t("show-detail-repair")
                            icon: "wrench"
                            primary: true
                            enabled: !root.repairing
                            onClicked: root.repair()
                        }
                        Label { visible: root.repairing && root.repairLabel !== ""; anchors.verticalCenter: parent.verticalCenter; text: root.repairLabel; role: "ui"; color: theme.textDim }
                    }
                }
            }
            Rectangle {
                visible: root.settling !== ""
                width: Math.min(parent.width, theme.space(200))
                height: settleRow.implicitHeight + theme.space(3) * 2
                radius: theme.radiusSm
                color: theme.raised
                border.color: theme.line
                border.width: 1
                Row {
                    id: settleRow
                    x: theme.space(3)
                    anchors.verticalCenter: parent.verticalCenter
                    width: parent.width - theme.space(6)
                    spacing: theme.space(2.5)
                    Rectangle {
                        id: settleDot
                        anchors.verticalCenter: parent.verticalCenter
                        width: theme.space(2.5); height: width; radius: width / 2
                        color: root.movingCount > 0 || root.watching ? theme.accent : root.takenCount > 0 ? theme.green : theme.textFaint
                        SequentialAnimation on opacity {
                            running: root.watching && !theme.reducedMotion
                            loops: Animation.Infinite
                            NumberAnimation { from: 1; to: 0.25; duration: 700 }
                            NumberAnimation { from: 0.25; to: 1; duration: 700 }
                        }
                    }
                    Label { width: parent.width - settleDot.width - parent.spacing; text: root.settling; role: "ui"; color: theme.text; wrapMode: Text.Wrap }
                }
            }
            // Every episode as a cube, coloured by its state (Tokens.cubeStates): the series
            // card's own row when the show is in the library, else on disk or not from the
            // missing list
            EpisodeCubes {
                visible: root.total > 0 && root.show.kind !== "Movie"
                width: parent.width
                height: implicitHeight
                cubeSize: theme.space(4)
                maxRows: 4
                codes: {
                    if (root.show && root.show.cubes) return root.show.cubes
                    // Missing is a hole before an episode that is here; with none here
                    // yet, or none after it, the rest is simply not downloaded (#735)
                    var cap = Math.min(root.total, 400), last = 0, out = []
                    for (var k = 1; k <= cap; k++) if (root.missing.indexOf(k) < 0) last = k
                    for (var n = 1; n <= cap; n++)
                        out.push(root.missing.indexOf(n) < 0 ? "d" : root.finished && n < last ? "m" : "g")
                    return out
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
                    msg: "show-detail-unsubscribe"
                    icon: "rss"
                    onClicked: root.unsubscribe(unsubscribeButton)
                }
                Button {
                    id: removeAllButton
                    objectName: "show-remove-torrents"
                    visible: root.torrents.length > 0
                    enabled: !root.removing
                    text: words.t("show-detail-remove-torrents", { count: root.torrents.length })
                    icon: "x"
                    flat: true
                    danger: true
                    onClicked: root.removeAll(removeAllButton)
                }
            }
            // The subscription's facts
            Grid {
                visible: !!root.rule
                columns: 2
                columnSpacing: theme.space(5)
                rowSpacing: theme.space(1.5)
                Label { msg: "show-detail-searches"; role: "ledger"; color: theme.textFaint }
                Label { text: root.rule && root.rule.query ? words.t("show-detail-quoted", { words: root.rule.query }) : words.t("show-detail-whole-feed"); role: "ui"; color: theme.text; elide: Text.ElideRight; width: Math.min(implicitWidth, info.width - theme.space(40)) }
                Label { msg: "show-detail-from"; role: "ledger"; color: theme.textFaint }
                Label { text: root.rule ? (root.source !== "" ? root.source + " (" + root.feedHost(root.rule.feed_url) + ")" : root.feedHost(root.rule.feed_url)) : ""; role: "ui"; color: theme.textDim }
                Label { msg: "show-detail-rule"; role: "ledger"; color: theme.textFaint }
                Label { text: root.rule && root.rule.followable === false ? words.t("show-detail-rule-unfollowable") : root.rule && root.rule.rule_enabled === false ? words.t("status-paused") : words.t("show-detail-rule-active"); role: "ui"; color: root.rule && root.rule.followable === false ? theme.red : root.rule && root.rule.rule_enabled === false ? theme.textFaint : theme.text; elide: Text.ElideRight; width: Math.min(implicitWidth, info.width - theme.space(40)) }
                Label { visible: !!root.ruleView && root.ruleView.must_contain !== ""; msg: "show-detail-keeps"; role: "ledger"; color: theme.textFaint }
                Label { visible: !!root.ruleView && root.ruleView.must_contain !== ""; text: root.ruleView ? words.t("show-detail-keeps-value", { words: root.ruleView.must_contain }) : ""; role: "ui"; color: theme.text }
                Label { visible: !!root.ruleView && root.ruleView.must_not_contain !== ""; msg: "show-detail-drops"; role: "ledger"; color: theme.textFaint }
                Label { visible: !!root.ruleView && root.ruleView.must_not_contain !== ""; text: root.ruleView ? words.t("show-detail-drops-value", { words: root.ruleView.must_not_contain.split("|").map(function(w) { return words.t("show-detail-quoted", { words: w }) }).reduce(function(a, b) { return words.t("show-detail-or", { first: a, second: b }) }) }) : ""; role: "ui"; color: theme.text }
                Label { visible: !!root.ruleView && root.ruleView.last_match !== ""; msg: "show-detail-last-took"; role: "ledger"; color: theme.textFaint }
                Label { visible: !!root.ruleView && root.ruleView.last_match !== ""; text: root.ruleView ? root.ruleView.last_match : ""; role: "ui"; color: theme.textDim }
                Label { msg: "show-detail-saves-to"; role: "ledger"; color: theme.textFaint }
                Label { text: root.folder; role: "ledger"; color: theme.textDim; elide: Text.ElideMiddle; width: Math.min(implicitWidth, info.width - theme.space(40)) }
            }
            Grid {
                visible: !root.rule
                columns: 2
                columnSpacing: theme.space(5)
                Label { msg: "show-detail-folder"; role: "ledger"; color: theme.textFaint }
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
                    Label { id: feedTitle; msg: "show-detail-in-the-feed"; role: "display"; font.pointSize: theme.typeLarge * 0.85; font.variableAxes: ({ "wdth": 74, "wght": 500 }) }
                    Label { anchors.baseline: feedTitle.baseline; visible: root.entries.length > 0; text: root.entries.length; role: "ledger"; color: theme.textFaint }
                    Label {
                        anchors.baseline: feedTitle.baseline
                        text: !root.feed || root.feed.loading ? words.t("show-detail-feed-reading") : root.feed.error ? root.feed.error : root.entries.length ? words.t("show-detail-feed-offers") : words.t("show-detail-feed-is-empty")
                        role: "ui"
                        color: root.feed && root.feed.error ? theme.red : theme.textDim
                    }
                }
                Button { anchors.right: parent.right; anchors.bottom: parent.bottom; anchors.bottomMargin: theme.space(1); msg: "show-detail-read-again"; icon: "refresh-cw"; flat: true; small: true; enabled: !!root.feed && !root.feed.loading; onClicked: root.refreshFeed() }
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
                                if (e.seeders !== null && e.seeders !== undefined) parts.push(words.t("show-detail-seeders", { count: e.seeders }))
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
                        width: stateWord.implicitWidth + (getButton.visible ? getButton.implicitWidth + theme.space(2) : 0)
                        height: theme.controlHeight
                        Label { id: stateWord; visible: entryRow.st.word !== ""; anchors.right: parent.right; anchors.verticalCenter: parent.verticalCenter; text: entryRow.st.word; role: "ledger"; color: entryRow.st.ink }
                        Button { id: getButton; anchors.rightMargin: stateWord.visible ? stateWord.implicitWidth + theme.space(2) : 0; visible: !entryRow.st.torrent && entryRow.st.ink !== theme.orange && !!entryRow.modelData.link; anchors.right: parent.right; anchors.verticalCenter: parent.verticalCenter; msg: "show-detail-download"; icon: "download"; flat: true; small: true; onClicked: root.downloadEntry(entryRow.modelData) }
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
                    Label { id: trTitle; msg: "show-detail-torrents"; role: "display"; font.pointSize: theme.typeLarge * 0.85; font.variableAxes: ({ "wdth": 74, "wght": 500 }) }
                    Label { anchors.baseline: trTitle.baseline; visible: root.torrents.length > 0; text: root.torrents.length; role: "ledger"; color: theme.textFaint }
                    Label { anchors.baseline: trTitle.baseline; text: root.torrents.length ? (root.movingCount > 0 ? words.t("show-detail-torrents-moving", { count: root.movingCount }) : words.t("show-detail-torrents-held")) : (root.watching ? words.t("show-detail-torrents-waiting") : words.t("show-detail-torrents-none")); role: "ui"; color: theme.textDim }
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
