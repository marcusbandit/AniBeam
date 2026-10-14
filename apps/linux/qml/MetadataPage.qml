// Metadata, flat (DESIGN.md, the flat home pass): every series in the library as one small
// poster tile, cut into the sections a librarian works in order. The head is the library's:
// the title in the display cut with the count beside it, the search at the right, then one
// line of pills (the filter with its counts, Match unmatched, Refresh all) that wraps on a
// narrow window. Under it the figures (Tiles.qml), which are the sections' own counts off
// the whole library, so they agree with what is below. Then the sections: Needs a match,
// Folder gone, Missing episodes, Airing, Complete, each a heading in the display cut with
// its count, over a grid of tiles in the library's column math (columns from the width,
// tiles widened to fill the line, so a narrow window has fewer columns). A tile is the
// poster at 2:3, the title on one line, a state line that says the section's fact for this
// series ("9 of 12", "5 of 7 aired", "12 of 12 +2", "No record yet"), and a dim line (the
// season, or the countdown to the next episode on an airing series), each on one line that
// elides, never wraps. A click on a matched tile opens the episodes pane (EpisodesPane.qml:
// which numbers are on disk and which are missing, as chips), on an unmatched one the match
// panel; hover shows one glyph on the poster's foot, Refresh, or Forget on a gone folder.
// Complete has a sort of its own, kept in shell.conf: Added (the folder first seen, newest
// first), Season, A to Z; the other sections keep their order. An airing series is measured against what has aired so far, never the
// record's total: a show two episodes in is not ten episodes short. The core owns the
// filtering and the four filter counts (ListMetadata { filter, query, reveal_hidden }); the
// page cuts the sections off the filtered rows and the figures off the whole library.
import QtQuick
import QtCore
import com.marcusrosado.AniBeam

Item {
    id: page
    objectName: "metadata-page"
    property var props: ({})
    property string title: "Metadata"
    property alias scrollY: scroll.contentY
    function escapePressed() { return false }
    function focusSearch() { scroll.contentY = 0; search.focusInput() }
    Keys.onPressed: function(event) { scroll.keyPressed(event) }

    readonly property var filters: ["All", "Series", "Movies", "MissingFiles"]
    readonly property var filterLabels: ["All", "Series", "Films", "Missing"]
    property string filter: "All"
    property string query: props.q || ""
    property var counts: ({ all: 0, series: 0, movies: 0, missing_files: 0 })
    property var rows: []            // the filtered list, flat: the drive reads its length
    property var allRows: []         // the whole library, no filter and no query: the figures
    property bool loaded: false
    property int revision: 0
    property real nowMs: Date.now()
    // The Complete section's sort, kept across runs
    Settings { id: kept; location: Shell.settingsUrl; category: "metadata"; property string completeSort: "Added" }
    readonly property var completeSorts: ["Added", "Season", "A to Z"]
    property alias completeSort: kept.completeSort

    function reload() {
        var keep = scroll.contentY
        var r = Door.listMetadata(page.filter, page.query, Door.revealHidden)
        if (r.error) { frame.toast(r.error.message); return }
        page.counts = r.reply.counts
        page.rows = r.reply.rows
        // The figures are the library's, whatever the filter: one more read only when the
        // filtered list is not already the whole library
        if (page.filter === "All" && page.query === "") page.allRows = r.reply.rows
        else { var a = Door.listMetadata("All", "", Door.revealHidden); page.allRows = a.error ? r.reply.rows : a.reply.rows }
        page.nowMs = Date.now()
        page.loaded = true
        page.revision++
        scroll.contentY = Math.max(0, Math.min(keep, Math.max(0, scroll.contentHeight - scroll.height)))
    }

    // ---- What a row is, read once for the figures, the sections and the tiles
    function isMatched(card) {
        return !!card.match_info && ["anilist_id", "mal_id", "tvmaze_id", "tmdb_id"].some(function(k) { return card.match_info[k] !== null && card.match_info[k] !== undefined })
    }
    function isMovie(card) { return card.kind === "Movie" || card.format === "MOVIE" }
    function expectedOf(d) {
        var card = d.series || {}
        return d.expected !== null && d.expected !== undefined ? d.expected : (isMovie(card) ? 1 : 0)
    }
    // The next episode while it is still ahead, else null
    function nextOf(card) {
        var n = card.next_airing
        return n && n.at * 1000 > page.nowMs ? n : null
    }
    function isAiring(card) { return card.status === "Releasing" || !!nextOf(card) }
    // What an airing series can be measured against: the episodes out so far, which is one
    // short of the next one when the record says when that lands, else what is on disk
    function airedOf(d) {
        var n = nextOf(d.series || {})
        return n ? Math.max(0, n.episode - 1) : (d.have || 0)
    }
    // The section a row works in, in the order a person would take them: a gone folder is
    // past matching, so Folder gone wins over Needs a match; an unmatched folder has no
    // expected count to be short of; an airing series is never short of a total that has
    // not aired yet
    function bucketOf(d) {
        var card = d.series || {}
        if (card.missing === true) return "missing"
        if (!isMatched(card)) return "unmatched"
        if (isAiring(card)) return "airing"
        var expected = expectedOf(d)
        if (expected > 0 && (d.have || 0) < expected) return "incomplete"
        return "complete"
    }
    readonly property var sectionOrder: [
        { key: "unmatched", title: "Needs a match" },
        { key: "missing", title: "Folder gone" },
        { key: "incomplete", title: "Missing episodes" },
        { key: "airing", title: "Airing" },
        { key: "complete", title: "Complete" }
    ]
    // The tile's state line: the section's fact for this one series, short enough for a
    // tile's width (the section says the episodes are missing; the pane says which)
    function stateOf(d) {
        var card = d.series || {}, have = d.have || 0, k = bucketOf(d)
        if (k === "missing") return "Folder not found"
        if (k === "unmatched") return "No record yet"
        if (k === "airing") { var aired = airedOf(d); return Math.min(have, aired) + " of " + aired + " aired" }
        var expected = expectedOf(d)
        if (k === "incomplete") return have + " of " + expected
        var extra = expected > 0 ? Math.max(0, have - expected) : 0
        return have + " of " + (expected > 0 ? expected : "?") + (extra > 0 ? " +" + extra : "")
    }
    // The same fact with the shortfall spelled out, for the pane's head
    function detailOf(d) {
        var have = d.have || 0, k = bucketOf(d)
        if (k === "airing") {
            var aired = airedOf(d), shortBy = Math.max(0, aired - have)
            return Math.min(have, aired) + " of " + aired + " aired on disk" + (shortBy > 0 ? ", " + shortBy + " missing" : "")
        }
        if (k === "incomplete") { var expected = expectedOf(d); return have + " of " + expected + " on disk, " + (expected - have) + " missing" }
        return stateOf(d)
    }
    // A tile's click: the episodes for a matched series, the match for one without a record
    function openTile(d) {
        var card = d.series || {}
        if (isMatched(card)) episodesPane.openFor(card.id, detailOf(d))
        else matchPanel.openFor(card.id, card.title, card.match_info)
    }
    function shortCountdown(secs) {
        var t = Math.max(0, Math.floor(secs))
        var d = Math.floor(t / 86400), h = Math.floor(t % 86400 / 3600), m = Math.floor(t % 3600 / 60)
        return d > 0 ? d + "d " + h + "h" : h > 0 ? h + "h " + m + "m" : Math.max(1, m) + "m"
    }
    // The tile's dim line: the countdown on an airing series, else the season or the year
    function noteOf(d) {
        var card = d.series || {}
        var n = nextOf(card)
        if (n) return "Next in " + shortCountdown(n.at - page.nowMs / 1000)
        return card.season ? String(card.season) : (card.year !== null && card.year !== undefined ? String(card.year) : "")
    }
    // When a record aired, as one number that sorts: the year four times plus the season's
    // place in it ("Fall 2025" is 2025 * 4 + 3), a bare year counts as its winter, and a
    // record with neither sorts last
    readonly property var seasonWords: ["Winter", "Spring", "Summer", "Fall"]
    function whenKey(card) {
        var year = card.year !== null && card.year !== undefined ? Number(card.year) : NaN
        var place = 0
        if (card.season) {
            var parts = String(card.season).split(" ")
            var w = page.seasonWords.indexOf(parts[0])
            var y = parseInt(parts[parts.length - 1])
            if (!isNaN(y)) year = y
            if (w >= 0) place = w
        }
        return isNaN(year) ? -1 : year * 4 + place
    }
    function titleOf(d) { var c = d.series || {}; return c.title || (c.titles ? c.titles.folder : "") || "" }
    // The provider word under a tile only when the library mixes providers: one provider
    // everywhere is texture, not a fact
    readonly property bool mixedProviders: {
        var seen = {}, n = 0
        for (var i = 0; i < rows.length; i++) {
            var mi = rows[i].series && rows[i].series.match_info
            var k = mi && mi.provider ? String(mi.provider) : ""
            if (k !== "" && !seen[k]) { seen[k] = true; n++ }
        }
        return n > 1
    }
    // The sections, cut from the filtered rows; an empty one is not shown. Airing is
    // ordered by the next episode, soonest first; Complete by its own sort (Added: the
    // folder first seen, newest first; Season; A to Z); every other section by season,
    // newest first, then by title
    function addedOf(d) { var c = d.series || {}; return typeof c.added_at === "number" ? c.added_at : 0 }
    readonly property var sections: {
        page.revision
        var completeSort = page.completeSort
        var buckets = {}
        for (var i = 0; i < page.rows.length; i++) {
            var k = page.bucketOf(page.rows[i])
            if (!buckets[k]) buckets[k] = []
            buckets[k].push(page.rows[i])
        }
        var out = []
        for (var s = 0; s < page.sectionOrder.length; s++) {
            var o = page.sectionOrder[s]
            var list = buckets[o.key]
            if (!list || list.length === 0) continue
            list.sort(function(a, b) {
                if (o.key === "airing") {
                    var na = page.nextOf(a.series || {}), nb = page.nextOf(b.series || {})
                    var ta = na ? na.at : Infinity, tb = nb ? nb.at : Infinity
                    if (ta !== tb) return ta - tb
                }
                if (o.key === "complete" && completeSort === "A to Z") return page.titleOf(a).localeCompare(page.titleOf(b))
                if (o.key === "complete" && completeSort === "Added") {
                    var aa = page.addedOf(a), ab = page.addedOf(b)
                    if (aa !== ab) return ab - aa
                }
                var ka = page.whenKey(a.series || {}), kb = page.whenKey(b.series || {})
                if (ka !== kb) return kb - ka
                return page.titleOf(a).localeCompare(page.titleOf(b))
            })
            out.push({ key: o.key, title: o.title, rows: list })
        }
        return out
    }
    // The names the drives click: row-match on the first Needs a match tile when that
    // section is there (its click opens the match panel), else on the page's first tile;
    // row-refresh on the first tile that has a Refresh glyph, row-forget on the first gone
    // folder
    readonly property string firstId: {
        for (var s = 0; s < sections.length; s++) if (sections[s].key === "unmatched") return String(sections[s].rows[0].series.id)
        return sections.length > 0 ? String(sections[0].rows[0].series.id) : ""
    }
    readonly property string firstRefreshId: {
        for (var s = 0; s < sections.length; s++) if (sections[s].key !== "missing") return String(sections[s].rows[0].series.id)
        return ""
    }
    readonly property string firstGoneId: {
        for (var s = 0; s < sections.length; s++) if (sections[s].key === "missing") return String(sections[s].rows[0].series.id)
        return ""
    }
    // The figures: the sections' own counts off the whole library, whatever the filter and
    // the search, and the files on disk against the expected total. A zero is a fact here
    readonly property var figures: {
        page.revision
        var n = { unmatched: 0, missing: 0, incomplete: 0, airing: 0, complete: 0 }, have = 0, expected = 0
        for (var i = 0; i < page.allRows.length; i++) {
            var d = page.allRows[i]
            n[page.bucketOf(d)]++
            have += d.have || 0
            expected += page.isAiring(d.series || {}) ? page.airedOf(d) : page.expectedOf(d)
        }
        return page.sectionOrder.map(function(o) { return { value: String(n[o.key]), caption: o.title } }).concat([
            { value: have.toLocaleString(Qt.locale(), "f", 0) + " of " + expected.toLocaleString(Qt.locale(), "f", 0), caption: "Files on disk" }
        ])
    }
    function filterCount(f) {
        return f === "All" ? page.counts.all : f === "Series" ? page.counts.series : f === "Movies" ? page.counts.movies : page.counts.missing_files
    }
    function providerLabel(p) { return frame.providerName(p) }
    function crawlJob() {
        for (var i = 0; i < Door.runningJobs.length; i++) {
            var j = Door.runningJobs[i]
            if (String(j.kind).toLowerCase() === "crawl") return j
        }
        return null
    }
    function forget(card) {
        var r = Door.forgetSeries(card.id)
        frame.toast(r.error ? r.error.message : "Forgot " + (card.title || card.titles.folder))
    }
    Timer { id: debounce; interval: 250; onTriggered: page.reload() }
    Timer { id: queryDebounce; interval: 250; onTriggered: { page.query = search.text; page.reload() } }
    Timer { interval: 30000; running: page.rows.length > 0; repeat: true; onTriggered: page.nowMs = Date.now() }
    Connections {
        target: Door
        function onSeriesChanged(changed) { debounce.restart() }
        function onSeriesRemoved(ids) { debounce.restart() }
        function onMatchApplied(series) { debounce.restart() }
        function onRevealHiddenChanged() { debounce.restart() }
        function onRefreshFinished(refreshed, failed) {
            debounce.restart()
            frame.toast(failed > 0 ? "Refreshed " + refreshed + ", " + failed + " could not be refreshed" : "Refreshed " + Fmt.plural(refreshed, "series", "series"))
        }
        function onAutoMatchFinished(backfilled, matched, unmatched) {
            debounce.restart()
            frame.toast(matched > 0 ? "Matched " + Fmt.plural(matched, "series", "series") + (unmatched > 0 ? ", " + unmatched + " still unmatched" : "")
                                    : unmatched > 0 ? "No new matches, " + unmatched + " still unmatched" : "Everything is matched")
        }
        // A cleared match emits nothing of its own, and the job end is the one signal
        // every bulk action shares
        function onJobFinished(job, kind, ok) { if (kind === "ClearMatch" || kind === "Refresh") debounce.restart() }
    }
    Component.onCompleted: {
        search.text = query
        queryDebounce.stop()
        reload()
    }

    // ---- The crawl, while it runs: a label and a thin line in the tools
    readonly property bool crawling: crawlJob() !== null

    // ---- The grid's geometry: small poster tiles, as many to a line as the width holds at
    // theme.space(24) each, widened to fill the line (the library's column math)
    readonly property real tileGap: theme.space(4)
    readonly property real tileMin: theme.space(24)
    readonly property int tileCols: Math.max(1, Math.floor((scroll.innerWidth + tileGap) / (tileMin + tileGap)))
    readonly property real tileWidth: Math.floor((scroll.innerWidth - tileGap * (tileCols - 1)) / tileCols)

    PageScroll {
        id: scroll
        anchors.fill: parent
        readonly property real innerWidth: width - theme.space(16)

        // ---- The head: the title and its count, the search at the right
        Item {
            id: head
            width: parent.width
            height: theme.space(14)
            Row {
                id: titleRow
                anchors.left: parent.left
                anchors.right: search.left; anchors.rightMargin: theme.space(6)
                anchors.verticalCenter: parent.verticalCenter
                spacing: theme.space(3)
                Label { id: titleLabel; anchors.verticalCenter: parent.verticalCenter; text: "Metadata"; role: "display" }
                Label {
                    objectName: "metadata-count"
                    anchors.baseline: titleLabel.baseline
                    text: Fmt.plural(page.counts.all, "series", "series")
                    role: "ui"
                    color: theme.textDim
                }
            }
            SearchField {
                id: search
                objectName: "metadata-search"
                anchors.right: parent.right
                anchors.verticalCenter: parent.verticalCenter
                width: Math.min(theme.space(100), Math.max(theme.space(40), parent.width * 0.3))
                placeholder: "Filter titles"
                onTextChanged: queryDebounce.restart()
                onCleared: { queryDebounce.stop(); page.query = ""; page.reload() }
            }
        }

        // ---- The tools: one line of pills, wrapping on a narrow window
        Flow {
            width: parent.width
            spacing: theme.space(3)
            Seg {
                id: filterSeg
                objectName: "metadata-filter"
                options: page.filters.map(function(f, i) { return { text: page.filterLabels[i] + " " + page.filterCount(f), name: f.toLowerCase() } })
                index: Math.max(0, page.filters.indexOf(page.filter))
                onPicked: function(i) { page.filter = page.filters[i]; page.reload() }
            }
            Button {
                objectName: "metadata-match-all"
                flat: true
                text: "Match unmatched"
                icon: "link"
                onClicked: { var r = Door.autoMatch(); frame.toast(r.error ? r.error.message : "Looking up a match for unmatched folders") }
                Tooltip { text: "Looks up every folder that has no match yet, on each metadata provider in the order Settings gives them" }
            }
            Button {
                objectName: "metadata-refresh-all"
                flat: true
                text: "Refresh all"
                icon: "refresh-cw"
                onClicked: { var r = Door.refreshAll(); frame.toast(r.error ? r.error.message : "Refreshing every matched series") }
                Tooltip { text: "Fetches every matched series' record again" }
            }
            // The franchise crawl's own line, only while it runs
            Column {
                visible: page.crawling
                readonly property var job: page.crawlJob()
                width: theme.space(50)
                height: theme.controlHeight
                spacing: theme.space(1.5)
                topPadding: theme.space(1)
                Row {
                    spacing: theme.space(2)
                    Label { text: "Finding franchise links"; role: "ui"; font.pointSize: theme.typeSmall; color: theme.textDim }
                    Label { text: parent.parent.job && parent.parent.job.total > 0 ? parent.parent.job.done + " of " + parent.parent.job.total : ""; role: "ledger"; color: theme.textDim }
                }
                Rectangle {
                    width: parent.width; height: theme.space(0.75); radius: height / 2
                    color: theme.behind
                    Rectangle {
                        width: parent.width * (parent.parent.job && parent.parent.job.total > 0 ? parent.parent.job.done / parent.parent.job.total : 0)
                        height: parent.height; radius: height / 2
                        color: theme.accent
                    }
                }
            }
        }

        // ---- The figures: the sections' counts, on the ground
        Tiles {
            objectName: "metadata-figures"
            visible: page.loaded && page.counts.all > 0
            tiles: page.figures
        }

        // ---- The sections: a heading in the display cut with its count, over the tiles
        Repeater {
            model: page.sections
            Column {
                id: section
                required property var modelData
                objectName: "meta-section-" + modelData.key
                width: parent.width
                spacing: theme.space(2)
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
                            color: section.modelData.key === "missing" ? theme.red : section.modelData.key === "airing" ? theme.accent : theme.text
                        }
                        Label { anchors.baseline: sectionTitle.baseline; text: section.modelData.rows.length; role: "ledger"; color: theme.textFaint }
                    }
                    // Complete's sort, at the heading's right
                    Seg {
                        visible: section.modelData.key === "complete"
                        objectName: visible ? "complete-sort" : ""
                        anchors.right: parent.right
                        anchors.bottom: parent.bottom
                        anchors.bottomMargin: theme.space(2)
                        small: true
                        options: page.completeSorts.map(function(s) { return { text: s, name: s.replace(/ /g, "").toLowerCase() } })
                        index: Math.max(0, page.completeSorts.indexOf(page.completeSort))
                        onPicked: function(i) { page.completeSort = page.completeSorts[i] }
                    }
                    Rectangle { anchors.bottom: parent.bottom; width: parent.width; height: 1; color: theme.line }
                }
                Flow {
                    width: parent.width
                    spacing: page.tileGap
                    // Air under the heading's hairline
                    topPadding: theme.space(3)
                    Repeater {
                        model: section.modelData.rows
                        MetaTile { width: page.tileWidth }
                    }
                }
            }
        }

        // EmptyState centres itself in its parent, so it gets a plain box in the column
        Item {
            visible: page.loaded && page.rows.length === 0
            width: parent.width
            height: visible ? theme.space(60) : 0
            EmptyState {
                icon: page.counts.all === 0 ? "database" : "search"
                title: page.counts.all === 0 ? "No metadata yet" : page.query !== "" ? "No matches for \"" + page.query + "\"" : "Nothing under this filter"
                body: page.counts.all === 0 ? "Your library is empty. Add a folder in Settings and scan it to get started." : page.query !== "" ? "" : "No series here right now."
            }
        }
    }

    // One series as a tile of the grid: the poster at 2:3, under it the title on one line,
    // the state line in the ledger with the provider as a faint word at its right when the
    // library mixes providers, and the dim line (the season, or the countdown on an airing
    // series), each on one line. At rest nothing else; hover lifts the tile one step of
    // the surface, turns the title to the accent and shows one glyph on the poster's foot:
    // Refresh, or Forget on a gone folder. A click on the tile opens the episodes pane, or
    // the match panel for a series with no record. The glyph is always present (a drive
    // click lands on it at rest); only its opacity follows the hover.
    component MetaTile: Item {
        id: tile
        required property var modelData
        readonly property var d: modelData || ({})
        readonly property var card: d.series || ({})
        readonly property bool gone: card.missing === true
        readonly property string provider: card.match_info && card.match_info.provider ? page.providerLabel(card.match_info.provider) : ""
        readonly property string note: page.noteOf(d)
        readonly property real pad: theme.space(2)
        FontMetrics { id: tileTitleMetrics; font.family: theme.fontFamily; font.pointSize: theme.typeSmall; font.weight: theme.roleWeight("title"); font.variableAxes: theme.roleAxes("title") }
        objectName: "meta-row-" + card.id
        height: poster.height + theme.space(1.5) + writing.height + pad

        // 0 rest, 1 hover, followed with exponential smoothing like every tracking value
        property real lift: 0
        readonly property real liftTarget: m.containsMouse || refreshGlyph.hovered || forgetGlyph.hovered ? 1 : 0
        FrameAnimation {
            running: Math.abs(tile.lift - tile.liftTarget) > 0.002
            onTriggered: tile.lift = theme.follow(tile.lift, tile.liftTarget, frameTime)
        }
        z: lift > 0.01 ? 1 : 0

        // The hover: one step of the surface, reaching into the gap so the tile reads lifted
        // rather than boxed
        Rectangle {
            anchors.fill: parent
            anchors.margins: -tile.pad
            radius: theme.radiusSm + tile.pad
            color: theme.tile
            opacity: tile.lift
        }
        // Under the glyph, so it takes its own click
        MouseArea {
            id: m
            anchors.fill: parent
            anchors.margins: -tile.pad
            hoverEnabled: true
            cursorShape: Qt.PointingHandCursor
            onClicked: page.openTile(tile.d)
        }

        // The poster, decoded at its displayed size and shown only once it is Ready
        Corner {
            id: poster
            width: parent.width
            height: Math.round(width * 1.5)
            radius: theme.radiusSm; smoothing: theme.cornerSmoothing
            color: theme.tile
            fillItem: art.status === Image.Ready ? art : null
            Image {
                id: art
                visible: false
                width: poster.width; height: poster.height
                source: tile.card.poster ? "file://" + tile.card.poster : ""
                sourceSize: Qt.size(Math.ceil(poster.width * theme.devicePixelRatio), Math.ceil(poster.height * theme.devicePixelRatio))
                fillMode: Image.PreserveAspectCrop
                smooth: true
                asynchronous: true
            }
            Icon {
                visible: art.status !== Image.Ready
                anchors.centerIn: parent
                glyph: page.isMovie(tile.card) ? "film" : "tv"
                size: theme.space(5)
                color: theme.textDim
            }
        }
        // The drive's row-match: a plain Item over the first tile's poster that takes no
        // input, so a click at its centre falls through to the tile's MouseArea
        Item {
            objectName: String(tile.card.id) === page.firstId ? "row-match" : ""
            anchors.fill: poster
        }
        // The one glyph on the poster's foot, seen on hover. A hair above zero at rest: the
        // drive's findItem treats opacity 0 as hidden
        Item {
            anchors.horizontalCenter: poster.horizontalCenter
            anchors.bottom: poster.bottom
            anchors.bottomMargin: tile.pad
            width: tile.gone ? forgetGlyph.width : refreshGlyph.width
            height: tile.gone ? forgetGlyph.height : refreshGlyph.height
            opacity: Math.max(0.001, tile.lift)
            Button {
                id: refreshGlyph
                visible: !tile.gone
                readonly property bool hovered: visible && lift > 0.5
                objectName: String(tile.card.id) === page.firstRefreshId ? "row-refresh" : ""
                icon: "refresh-cw"; small: true
                onClicked: { var r = Door.refreshSeries(tile.card.id); frame.toast(r.error ? r.error.message : "Refreshing " + (tile.card.title || "")) }
                Tooltip { text: "Fetch the record again" }
            }
            Button {
                id: forgetGlyph
                visible: tile.gone
                readonly property bool hovered: visible && lift > 0.5
                objectName: String(tile.card.id) === page.firstGoneId ? "row-forget" : ""
                icon: "trash-2"; small: true; danger: true
                onClicked: page.forget(tile.card)
                Tooltip { text: "The folder is gone: drop this series and its history" }
            }
        }

        // The writing under the poster
        Column {
            id: writing
            anchors.top: poster.bottom; anchors.topMargin: theme.space(1.5)
            width: parent.width
            spacing: theme.space(0.25)
            // One line at the face's own height, whatever glyph the title falls back for
            // (a △ in a title took a taller fallback and pushed the lines under it down)
            Label {
                width: parent.width
                height: Math.ceil(tileTitleMetrics.height)
                clip: true
                text: page.titleOf(tile.d)
                role: "title"; font.pointSize: theme.typeSmall
                elide: Text.ElideRight
                verticalAlignment: Text.AlignVCenter
                color: theme.tone(theme.text, theme.accent, tile.lift)
            }
            // The state line on one line, eliding before it reaches the provider word at
            // its right
            Item {
                width: parent.width
                height: stateLine.implicitHeight
                Label {
                    id: stateLine
                    anchors.left: parent.left
                    anchors.right: providerWord.left; anchors.rightMargin: theme.space(1)
                    text: page.stateOf(tile.d)
                    role: "ledger"
                    color: tile.gone ? theme.red : theme.textDim
                    maximumLineCount: 1
                    elide: Text.ElideRight
                }
                Label {
                    id: providerWord
                    anchors.right: parent.right
                    anchors.top: parent.top
                    text: page.mixedProviders ? tile.provider : ""
                    role: "ledger"
                    color: theme.textFaint
                }
            }
            Label {
                visible: tile.note !== ""
                width: parent.width
                text: tile.note
                role: "ledger"
                color: theme.textFaint
                maximumLineCount: 1
                elide: Text.ElideRight
            }
        }
    }

    MatchPanel {
        id: matchPanel
        parent: frame.overlay
        anchors.centerIn: parent
    }
    Blocker {
        objectName: "match-blocker"
        parent: frame.overlay
        anchors.fill: parent
        z: -1
        visible: matchPanel.visible
        dismissOnPress: true
        onDismissed: matchPanel.close()
    }
    EpisodesPane {
        id: episodesPane
        parent: frame.overlay
        anchors.centerIn: parent
        onChangeMatch: function(id, title, matchInfo) { matchPanel.openFor(id, title, matchInfo) }
    }
    Blocker {
        objectName: "episodes-blocker"
        parent: frame.overlay
        anchors.fill: parent
        z: -1
        visible: episodesPane.visible
        dismissOnPress: true
        onDismissed: episodesPane.close()
    }
}
