// Metadata, flat (DESIGN.md, the flat home pass): every series in the library, the record
// it is matched to, and the files on disk against the expected count. The head is the
// library's: the title in the display cut with the count beside it, the filter, the two
// bulk actions and the search at the right. Under it the library's figures (matched,
// unmatched, missing folders, incomplete, files against expected), then the rows cut into
// the sections a librarian works in order: Unmatched, Missing on disk, Incomplete,
// Complete, each a heading in the display cut with its count and one fact, over rows with
// a hairline between them, two to a line on a wide window. The core owns the filtering and
// the four counts (ListMetadata { filter, query, reveal_hidden }); the figures and the
// sections are cut client-side, the figures off the whole library, the sections off the
// filtered rows. The page runs the two bulk jobs (Match unmatched is the AutoMatch job,
// Refresh all the RefreshAll job) and opens the match panel over the overlay for a row
// that needs a different record.
import QtQuick
import com.marcusrosado.AniBeam

Item {
    id: page
    objectName: "metadata-page"
    property var props: ({})
    property string title: "Metadata"
    property alias scrollY: scroll.contentY
    function escapePressed() { return false }
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
        page.loaded = true
        page.revision++
        scroll.contentY = Math.max(0, Math.min(keep, Math.max(0, scroll.contentHeight - scroll.height)))
    }

    // ---- What a row is, read once for the figures and the sections
    function isMatched(card) {
        return !!card.match_info && ["anilist_id", "mal_id", "tvmaze_id", "tmdb_id"].some(function(k) { return card.match_info[k] !== null && card.match_info[k] !== undefined })
    }
    function expectedOf(d) {
        var card = d.series || {}
        var movie = card.kind === "Movie" || card.format === "MOVIE"
        return d.expected !== null && d.expected !== undefined ? d.expected : (movie ? 1 : 0)
    }
    // The bucket a row works in, in the order a person would take them: a gone folder is
    // past matching, so Missing wins over Unmatched; an unmatched folder has no expected
    // count to be short of, so Unmatched wins over Incomplete
    function bucketOf(d) {
        var card = d.series || {}
        if (card.missing === true) return "missing"
        if (!isMatched(card)) return "unmatched"
        var expected = expectedOf(d)
        if (expected > 0 && (d.have || 0) < expected) return "incomplete"
        return "complete"
    }
    readonly property var sectionOrder: [
        { key: "unmatched", title: "Unmatched" },
        { key: "missing", title: "Missing on disk" },
        { key: "incomplete", title: "Incomplete" },
        { key: "complete", title: "Complete" }
    ]
    // A section's one fact beside its count: what its name does not already say
    function factOf(key, list) {
        var i, n
        if (key === "incomplete") {
            var short = 0
            for (i = 0; i < list.length; i++) short += Math.max(0, expectedOf(list[i]) - (list[i].have || 0))
            return short > 0 ? Fmt.plural(short, "episode", "episodes") + " short" : ""
        }
        if (key === "complete") {
            var files = 0
            for (i = 0; i < list.length; i++) files += list[i].have || 0
            return files > 0 ? Fmt.plural(files, "file", "files") + " on disk" : ""
        }
        if (key === "unmatched") {
            var films = 0
            for (i = 0; i < list.length; i++) { var c = list[i].series || {}; if (c.kind === "Movie" || c.format === "MOVIE") films++ }
            n = list.length
            return films > 0 && films < n ? (n - films) + " series, " + Fmt.plural(films, "film", "films") : ""
        }
        return ""
    }
    // The sections, cut from the filtered rows; an empty one is not shown
    readonly property var sections: {
        page.revision
        var buckets = {}
        for (var i = 0; i < page.rows.length; i++) {
            var k = page.bucketOf(page.rows[i])
            if (!buckets[k]) buckets[k] = []
            buckets[k].push(page.rows[i])
        }
        var out = []
        for (var s = 0; s < page.sectionOrder.length; s++) {
            var o = page.sectionOrder[s]
            if (buckets[o.key] && buckets[o.key].length > 0) out.push({ key: o.key, title: o.title, rows: buckets[o.key], fact: page.factOf(o.key, buckets[o.key]) })
        }
        return out
    }
    // The first row of the page carries the row-match and row-refresh names the drive clicks
    readonly property string firstId: sections.length > 0 ? String(sections[0].rows[0].series.id) : ""
    // The figures: the library's, whatever the filter and the search
    readonly property var figures: {
        page.revision
        var matched = 0, unmatched = 0, missing = 0, incomplete = 0, have = 0, expected = 0
        for (var i = 0; i < page.allRows.length; i++) {
            var d = page.allRows[i], card = d.series || {}
            if (card.missing === true) missing++
            if (page.isMatched(card)) matched++; else unmatched++
            var b = page.bucketOf(d)
            if (b === "incomplete") incomplete++
            have += d.have || 0
            expected += page.expectedOf(d)
        }
        return [
            { value: String(matched), caption: "Matched" },
            { value: String(unmatched), caption: "Unmatched" },
            { value: String(missing), caption: "Missing folders" },
            { value: String(incomplete), caption: "Incomplete" },
            { value: have.toLocaleString(Qt.locale(), "f", 0) + " of " + expected.toLocaleString(Qt.locale(), "f", 0), caption: "Files on disk" }
        ]
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

    // ---- The rows' geometry: two to a line while each still has theme.space(150)
    readonly property real gap: theme.space(6)
    readonly property int rowCols: Math.max(1, Math.min(2, Math.floor((scroll.innerWidth + gap) / (theme.space(150) + gap))))
    readonly property real rowWidth: Math.floor((scroll.innerWidth - gap * (rowCols - 1)) / rowCols)

    PageScroll {
        id: scroll
        anchors.fill: parent
        readonly property real innerWidth: width - theme.space(16)

        // ---- The head: the title and its count, the tools beside them while they fit, the
        // search at the right
        Item {
            id: head
            width: parent.width
            height: theme.space(14)
            readonly property bool toolsInline: titleRow.implicitWidth + theme.space(6) + tools.implicitWidth + theme.space(6) + search.width <= width
            Row {
                id: titleRow
                anchors.left: parent.left
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
            Item {
                id: inlineSlot
                anchors.left: titleRow.right; anchors.leftMargin: theme.space(6)
                anchors.verticalCenter: parent.verticalCenter
                width: head.toolsInline ? tools.implicitWidth : 0
                height: tools.implicitHeight
            }
            SearchField {
                id: search
                objectName: "metadata-search"
                anchors.right: parent.right
                anchors.verticalCenter: parent.verticalCenter
                width: Math.min(theme.space(100), Math.max(theme.space(40), parent.width * 0.3))
                placeholder: "Filter titles"
                hint: ""
                onTextChanged: queryDebounce.restart()
                onCleared: { queryDebounce.stop(); page.query = ""; page.reload() }
            }
        }
        // The tools on their own line when the head is too narrow to hold them
        Item {
            id: wrapSlot
            width: parent.width
            height: head.toolsInline ? 0 : tools.implicitHeight
            visible: !head.toolsInline
        }
        // The tools are declared once and sit in whichever slot the width allows
        Row {
            id: tools
            parent: head.toolsInline ? inlineSlot : wrapSlot
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

        // ---- The figures: the library's state at a glance, on the ground
        Tiles {
            objectName: "metadata-figures"
            visible: page.loaded && page.counts.all > 0
            tiles: page.figures
        }

        // ---- The sections: a heading in the display cut with its count and one fact, over
        // the rows as a flat list, a hairline under each, two to a line when wide
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
                            color: section.modelData.key === "missing" ? theme.red : theme.text
                        }
                        Label { anchors.baseline: sectionTitle.baseline; text: section.modelData.rows.length; role: "ledger"; color: theme.textFaint }
                        Label { anchors.baseline: sectionTitle.baseline; text: section.modelData.fact; role: "ui"; color: theme.textDim }
                    }
                    // The hairline the rows continue
                    Rectangle { anchors.bottom: parent.bottom; width: parent.width; height: 1; color: theme.line }
                }
                Flow {
                    width: parent.width
                    spacing: page.gap
                    Repeater {
                        model: section.modelData.rows
                        MetaRow {
                            width: page.rowWidth
                            first: String(card.id) === page.firstId
                        }
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

    // One series: the poster thumbnail, the titles, the kind and the record as pills, the
    // files against the expected count with a thin line, and the row's actions, on the
    // ground with a hairline under it. A click anywhere else on it opens the match panel.
    component MetaRow: Item {
        id: metaRow
        required property var modelData
        property bool first: false
        readonly property var d: modelData || ({})
        readonly property var card: d.series || ({})
        // Matched on any provider the record names an id for: AniList, MyAnimeList, TVmaze or TMDB
        readonly property bool matched: page.isMatched(card)
        // The record's season when it has one, else its year, as a dim note after the pills
        readonly property string when: card.season ? String(card.season) : (card.year !== null && card.year !== undefined ? String(card.year) : "")
        readonly property bool isMovie: card.kind === "Movie" || card.format === "MOVIE"
        readonly property bool missing: card.missing === true
        readonly property real expected: d.expected !== null && d.expected !== undefined ? d.expected : (isMovie ? 1 : 0)
        readonly property real have: d.have || 0
        readonly property real pct: expected > 0 ? Math.min(1, have / expected) : 0
        readonly property real extra: expected > 0 ? Math.max(0, have - expected) : 0
        readonly property real pad: theme.space(3)
        objectName: "meta-row-" + card.id
        height: theme.space(19)

        // The hover: one step of the surface, inset so the hairlines stay a list
        Rectangle {
            anchors.fill: parent
            anchors.topMargin: theme.space(1)
            anchors.bottomMargin: theme.space(1)
            radius: theme.radiusSm
            color: theme.tile
            opacity: m.containsMouse ? 1 : 0
        }
        // Under everything else, so the buttons take their own clicks
        MouseArea {
            id: m
            anchors.fill: parent
            hoverEnabled: true
            cursorShape: Qt.PointingHandCursor
            onClicked: matchPanel.openFor(metaRow.card.id, metaRow.card.title, metaRow.card.match_info)
        }

        // The poster, decoded at its displayed size and shown only once it is Ready
        Corner {
            id: thumb
            x: metaRow.pad
            anchors.verticalCenter: parent.verticalCenter
            height: parent.height - theme.space(4)
            width: Math.round(height / 1.5)
            radius: theme.radiusSm; smoothing: theme.cornerSmoothing
            color: theme.tile
            fillItem: art.status === Image.Ready ? art : null
            Image {
                id: art
                visible: false
                width: thumb.width; height: thumb.height
                source: metaRow.card.poster ? "file://" + metaRow.card.poster : ""
                sourceSize: Qt.size(Math.ceil(thumb.width * theme.devicePixelRatio), Math.ceil(thumb.height * theme.devicePixelRatio))
                fillMode: Image.PreserveAspectCrop
                smooth: true
                asynchronous: true
            }
            Icon {
                visible: art.status !== Image.Ready
                anchors.centerIn: parent
                glyph: metaRow.isMovie ? "film" : "tv"
                size: theme.space(5)
                color: theme.textDim
            }
        }

        // The titles
        Column {
            id: titles
            anchors.left: thumb.right; anchors.leftMargin: metaRow.pad
            anchors.right: readout.left; anchors.rightMargin: metaRow.pad
            anchors.verticalCenter: parent.verticalCenter
            spacing: theme.space(0.5)
            Label {
                width: parent.width
                text: metaRow.card.title || (metaRow.card.titles ? metaRow.card.titles.folder : "")
                role: "title"; font.pointSize: theme.typeNormal
                wrapMode: Text.Wrap; maximumLineCount: 2; elide: Text.ElideRight
                color: m.containsMouse ? theme.accent : theme.text
            }
            Label {
                visible: !!metaRow.d.alt_title
                width: parent.width
                text: metaRow.d.alt_title || ""
                role: "ui"; font.pointSize: theme.typeSmall
                color: theme.textDim
                elide: Text.ElideRight
            }
        }

        // The readouts: the kind and the record as pills over the files line and its count
        Column {
            id: readout
            anchors.right: actions.left; anchors.rightMargin: metaRow.pad
            anchors.verticalCenter: parent.verticalCenter
            width: theme.space(56)
            spacing: theme.space(2)
            Row {
                spacing: theme.space(1.5)
                Chip { small: true; mono: false; color: theme.tile; text: metaRow.isMovie ? "Film" : "Series"; textColor: theme.textDim }
                Chip { small: true; mono: false; color: theme.tile
                    text: !metaRow.matched ? "No match" : page.providerLabel(metaRow.card.match_info.provider)
                    textColor: metaRow.matched ? theme.text : theme.textDim }
                Chip { visible: metaRow.missing; small: true; mono: false; color: theme.redSoft; text: "Missing"; textColor: theme.red }
                Label {
                    visible: metaRow.when !== ""
                    anchors.verticalCenter: parent.verticalCenter
                    text: metaRow.when
                    role: "ui"; font.pointSize: theme.typeSmall
                    color: theme.textDim
                }
            }
            Item {
                width: parent.width
                height: filesCount.implicitHeight
                Label {
                    id: filesCount
                    text: metaRow.have + " of " + (metaRow.expected > 0 ? metaRow.expected : "?") + (metaRow.extra > 0 ? ", " + metaRow.extra + " extra" : "")
                    role: "ledger"
                    color: theme.textDim
                    Tooltip { text: metaRow.extra > 0 ? "Files beyond the expected count, likely misnamed or stray" : "" }
                }
                Rectangle {
                    anchors.left: filesCount.right; anchors.leftMargin: theme.space(3)
                    anchors.right: parent.right
                    anchors.verticalCenter: filesCount.verticalCenter
                    height: theme.space(0.75)
                    radius: height / 2
                    color: theme.behind
                    Rectangle {
                        width: Math.max(metaRow.pct > 0 ? height : 0, parent.width * metaRow.pct)
                        height: parent.height
                        radius: height / 2
                        color: theme.caughtUp
                    }
                }
            }
        }

        // The row's actions: small flat buttons
        Row {
            id: actions
            anchors.right: parent.right; anchors.rightMargin: metaRow.pad
            anchors.verticalCenter: parent.verticalCenter
            spacing: theme.space(1.5)
            Button {
                id: matchBtn
                objectName: metaRow.first ? "row-match" : ""
                text: "Match"; icon: "link"; small: true; flat: true
                onClicked: matchPanel.openFor(metaRow.card.id, metaRow.card.title, metaRow.card.match_info)
            }
            Button {
                id: refreshBtn
                visible: !metaRow.missing
                objectName: metaRow.first ? "row-refresh" : ""
                text: "Refresh"; icon: "refresh-cw"; small: true; flat: true
                onClicked: { var r = Door.refreshSeries(metaRow.card.id); frame.toast(r.error ? r.error.message : "Refreshing " + (metaRow.card.title || "")) }
            }
            Button {
                visible: metaRow.missing
                objectName: "row-forget"
                text: "Forget"; icon: "trash-2"; small: true; danger: true
                onClicked: page.forget(metaRow.card)
                Tooltip { text: "The folder is gone: drop this series and its history" }
            }
        }

        // The hairline under the row
        Rectangle { anchors.bottom: parent.bottom; width: parent.width; height: 1; color: theme.line }
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
}
