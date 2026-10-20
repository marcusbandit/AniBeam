// The episodes popover: what a Metadata tile actually holds. A compact pane in the match
// panel's idiom (a Corner in frame.overlay over a Blocker the owning page puts under it,
// onto the escape stack, closed through close()). openFor(id, line) reads GetSeries and
// shows the title with the tile's state line under it, then the episodes as a wrap of
// square number chips, one per expected episode number: filled in the soft accent when the
// file is on disk, a quiet outline when it is missing, so the gaps read at a glance; one a
// torrent is still writing takes the downloading purple, since it is not whole yet. A
// series whose files carry seasons gets a row per season with a small "S2" before it. Under
// the chips one quiet line counts the extras and the unmatched files when there are any. A
// film has no numbers to show, so it says On disk or Not on disk. At the foot three flat
// pills: Open (the series page), Change match (closes this and opens the match panel for
// the same series), Refresh.
import QtQuick
import QtQuick.Controls.Basic as QC
import com.marcusrosado.AniBeam

Corner {
    id: root
    objectName: "episodes-pane"
    property real seriesId: 0
    property var detail: null
    property string stateLine: ""
    signal changeMatch(real id, string title, var matchInfo)

    readonly property var card: detail ? detail.card : ({})
    readonly property string title: card.title || (card.titles ? card.titles.folder : "") || ""
    readonly property bool film: card.kind === "Movie" || card.format === "MOVIE"
    readonly property var episodes: detail ? detail.episodes : []

    function openFor(id, line) {
        root.seriesId = id
        root.stateLine = line || ""
        root.load()
        visible = true
        frame.escapeStack.push("popover", root)
        root.forceActiveFocus()
    }
    function close() {
        if (!visible) return
        visible = false
        frame.escapeStack.pop(root)
    }
    Component.onDestruction: if (visible) frame.escapeStack.pop(root)
    function load() {
        var r = Door.getSeries(root.seriesId)
        if (r.error) { frame.toast(r.error.message); return }
        root.detail = r.reply.detail
    }
    onVisibleChanged: if (!visible) detail = null
    Timer { id: reloadDebounce; interval: 200; onTriggered: root.load() }
    Connections {
        target: Door
        function onSeriesChanged(cards) { if (root.visible) for (var i = 0; i < cards.length; i++) if (cards[i].id === root.seriesId) { reloadDebounce.restart(); return } }
    }

    // ---- The numbers: what the series is measured against, the way the tile measures it.
    // The next episode while it is still ahead, else null
    readonly property var next: card.next_airing && card.next_airing.at * 1000 > Date.now() ? card.next_airing : null
    readonly property bool airing: card.status === "Releasing" || next !== null
    readonly property int highestOnDisk: {
        var n = 0
        for (var i = 0; i < episodes.length; i++) n = Math.max(n, Math.floor(episodes[i].number))
        return n
    }
    // The expected count: the record's total, or the episodes aired so far on an airing
    // series (one short of the next), or the highest number on disk when there is neither
    readonly property int expected: {
        if (airing) return next ? Math.max(0, next.episode - 1) : Math.max(highestOnDisk, episodes.length)
        if (card.total_episodes) return card.total_episodes
        return highestOnDisk
    }
    // The seasons the files carry, ascending; one unnamed row when they carry none. A row
    // is { season, numbers }: the integer numbers on disk in it, as a set.
    readonly property var seasonRows: {
        var bySeason = {}, loading = {}, keys = []
        for (var i = 0; i < episodes.length; i++) {
            var ep = episodes[i]
            var k = ep.season !== null && ep.season !== undefined ? String(ep.season) : ""
            if (!bySeason[k]) { bySeason[k] = {}; loading[k] = {}; keys.push(k) }
            if (Math.floor(ep.number) === ep.number) {
                bySeason[k][ep.number] = true
                if (ep.downloading !== undefined && ep.downloading !== null) loading[k][ep.number] = true
            }
        }
        if (keys.length === 0) keys.push("")
        keys.sort(function(a, b) { return (a === "" ? -1 : Number(a)) - (b === "" ? -1 : Number(b)) })
        var rows = []
        for (var s = 0; s < keys.length; s++) {
            var numbers = bySeason[keys[s]] || {}
            var highest = 0
            for (var n in numbers) highest = Math.max(highest, Number(n))
            // One season is measured against the expected count; several each against
            // their own highest number, since the record counts one of them at most
            var count = keys.length === 1 ? Math.max(root.expected, highest) : highest
            rows.push({ season: keys[s], count: count, numbers: numbers, loading: loading[keys[s]] || {} })
        }
        return rows
    }
    readonly property string extrasLine: {
        if (!detail) return ""
        var bits = []
        if (detail.extras.length > 0) bits.push(Fmt.plural(detail.extras.length, "extra", "extras"))
        if (detail.unmatched_files.length > 0) bits.push(Fmt.plural(detail.unmatched_files.length, "unmatched file", "unmatched files"))
        return bits.join(", ")
    }

    readonly property real pad: theme.space(5)
    readonly property real chip: theme.space(7)
    readonly property real chipGap: theme.space(1.5)
    width: Math.min(theme.space(120), (parent ? parent.width : theme.space(120)) - theme.space(16))
    // As tall as its content, never past the window
    height: Math.min(body.height + pad * 2, (parent ? parent.height : theme.space(160)) - theme.space(16))
    visible: false
    color: theme.glassFloat
    borderColor: theme.lineStrong
    borderWidth: 1
    highlight: theme.glassHighlight

    // The pane's own press sink: a press on its padding never reaches the shield under it
    MouseArea { anchors.fill: parent; acceptedButtons: Qt.AllButtons; hoverEnabled: true; onWheel: function(w) { w.accepted = true } }

    Flickable {
        id: flick
        x: root.pad; y: root.pad
        width: root.width - root.pad * 2
        height: root.height - root.pad * 2
        contentHeight: body.height
        clip: true
        boundsBehavior: Flickable.StopAtBounds
        QC.ScrollBar.vertical: ThinScrollBar {}
        WheelStep { flick: flick }

        Column {
            id: body
            width: flick.width
            spacing: theme.space(4)

            // The title and the tile's state line under it, the close at the right
            Item {
                width: parent.width
                height: Math.max(heading.height, closeBtn.height)
                Column {
                    id: heading
                    width: parent.width - closeBtn.width - theme.space(3)
                    spacing: theme.space(0.5)
                    Label { objectName: "episodes-title"; width: parent.width; text: root.title; role: "title"; font.pointSize: theme.typeNormal; elide: Text.ElideRight }
                    Label { objectName: "episodes-state"; width: parent.width; text: root.stateLine; role: "ledger"; color: theme.textDim; elide: Text.ElideRight }
                }
                Button {
                    id: closeBtn
                    objectName: "episodes-close"
                    anchors.right: parent.right; anchors.top: parent.top
                    small: true; flat: true; icon: "x"
                    onClicked: root.close()
                }
            }

            // A film: on disk or not
            Label {
                visible: root.film
                text: root.episodes.length > 0 ? "On disk" : "Not on disk"
                role: "ui"
                color: root.episodes.length > 0 ? theme.text : theme.textDim
            }

            // The chips, a row per season
            Column {
                visible: !root.film
                width: parent.width
                spacing: theme.space(2)
                Repeater {
                    model: root.seasonRows
                    Row {
                        id: seasonRow
                        required property var modelData
                        required property int index
                        objectName: "episodes-season-" + index
                        width: parent.width
                        spacing: theme.space(2)
                        Label {
                            id: seasonLabel
                            visible: seasonRow.modelData.season !== ""
                            width: visible ? theme.space(7) : 0
                            height: root.chip
                            verticalAlignment: Text.AlignVCenter
                            text: visible ? "S" + seasonRow.modelData.season : ""
                            role: "ledger"
                            color: theme.textDim
                        }
                        Flow {
                            width: parent.width - (seasonLabel.visible ? seasonLabel.width + parent.spacing : 0)
                            spacing: root.chipGap
                            Repeater {
                                model: seasonRow.modelData.count
                                Corner {
                                    required property int index
                                    readonly property int number: index + 1
                                    readonly property bool present: seasonRow.modelData.numbers[number] === true
                                    objectName: "episodes-chip-" + seasonRow.index + "-" + number
                                    width: root.chip; height: root.chip
                                    radius: theme.radiusSm; smoothing: theme.cornerSmoothing
                                    readonly property bool loading: seasonRow.modelData.loading[number] === true
                                    color: loading ? theme.cubeState("t").fill : present ? theme.cubeState("d").fill : "transparent"
                                    borderColor: present ? "transparent" : theme.cubeState("f").edge
                                    borderWidth: 1
                                    Label {
                                        anchors.centerIn: parent
                                        text: parent.number
                                        role: "ledger"
                                        color: parent.present ? theme.bg : theme.textFaint
                                    }
                                }
                            }
                        }
                    }
                }
                // No expected count and nothing on disk: say so rather than draw nothing
                Label {
                    visible: root.detail !== null && root.seasonRows.every(function(r) { return r.count === 0 })
                    text: "No episodes on disk"
                    role: "ui"; color: theme.textDim
                }
            }

            Label {
                objectName: "episodes-extras"
                visible: root.extrasLine !== ""
                width: parent.width
                text: root.extrasLine
                role: "ledger"; color: theme.textDim
                elide: Text.ElideRight
            }

            // The foot
            Flow {
                width: parent.width
                spacing: theme.space(2)
                Button {
                    objectName: "episodes-open"
                    flat: true; small: true; icon: "arrow-up-right"; text: "Open"
                    onClicked: { var id = root.seriesId, t = root.title; root.close(); frame.go("series", { id: id }, t) }
                }
                Button {
                    objectName: "episodes-change-match"
                    flat: true; small: true; icon: "link"; text: "Change match"
                    onClicked: { var id = root.seriesId, t = root.title, m = root.card.match_info || null; root.close(); root.changeMatch(id, t, m) }
                }
                Button {
                    objectName: "episodes-refresh"
                    flat: true; small: true; icon: "refresh-cw"; text: "Refresh"
                    onClicked: { var r = Door.refreshSeries(root.seriesId); frame.toast(r.error ? r.error.message : "Refreshing " + root.title) }
                }
            }
        }
    }
}
