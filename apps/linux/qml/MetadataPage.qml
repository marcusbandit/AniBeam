// The metadata wall: every series in the library as a row tile on the library's module, the
// record it is matched to, and the files on disk against the expected count. The header is
// a row of tiles (title, the filter with its counts, the search, the bulk actions); under it
// the row tiles, as many to a line as hold their words, each a whole number of columns. The
// core owns the filtering and the counts (ListMetadata { filter, query, reveal_hidden });
// the page runs the two bulk jobs (Match unmatched is the AutoMatch job, Refresh all the
// RefreshAll job) and opens the match panel over the overlay for a row that needs a
// different record.
import QtQuick
import QtQuick.Controls.Basic as QC
import com.marcusrosado.AniBeam
import "mosaic.js" as Mosaic

Item {
    id: page
    objectName: "metadata-page"
    property var props: ({})
    property string title: "Metadata"
    property alias scrollY: flick.contentY
    function escapePressed() { return false }

    readonly property var filters: ["All", "Series", "Movies", "MissingFiles"]
    readonly property var filterLabels: ["All", "Series", "Films", "Missing"]
    property string filter: "All"
    property string query: props.q || ""
    property var counts: ({ all: 0, series: 0, movies: 0, missing_files: 0 })
    property var rows: []
    property bool loaded: false

    function reload() {
        var keep = flick.contentY
        var r = Door.listMetadata(page.filter, page.query, Door.revealHidden)
        if (r.error) { frame.toast(r.error.message); return }
        page.counts = r.reply.counts
        page.rows = r.reply.rows
        page.loaded = true
        flick.contentY = Math.max(0, Math.min(keep, Math.max(0, flick.contentHeight - flick.height)))
    }
    function filterCount(f) {
        return f === "All" ? page.counts.all : f === "Series" ? page.counts.series : f === "Movies" ? page.counts.movies : page.counts.missing_files
    }
    function providerLabel(p) { return ({ Anilist: "AniList", Mal: "MyAnimeList", Tmdb: "TMDB" })[p] || p || "" }
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

    // ---- The module: the library's columns and unit
    readonly property real grout: theme.grout
    readonly property var geo: Mosaic.columns(Math.max(1, width - grout), theme.posterWidth, grout)
    readonly property int cols: geo.cols
    readonly property real unit: geo.u
    readonly property real step: unit + grout
    function px(units) { return Mosaic.unitsToPx(units, unit, grout) }
    function spanX(col) { return geo.xs[Math.max(0, Math.min(col, cols - 1))] }
    function spanW(col, span) { col = Math.max(0, Math.min(col, cols - 1)); var last = Math.max(col, Math.min(cols - 1, col + span - 1)); return geo.xs[last] + geo.ws[last] - geo.xs[col] }
    readonly property real hpad: theme.space(2.5)
    function spanFor(w) { return Math.max(1, Math.ceil((w + hpad * 2 + grout) / (geo.w + grout))) }

    // The header: one unit tall, wrapping on a narrow window; spare columns to the title
    readonly property bool crawling: crawlJob() !== null
    readonly property var headerLayout: Mosaic.header([
        { key: "title", span: Math.max(2, spanFor(titleLabel.implicitWidth)), flex: true },
        { key: "filter", span: spanFor(filterSeg.implicitWidth) },
        { key: "search", span: Math.max(2, spanFor(theme.space(56))), flex: false },
        { key: "actions", span: spanFor(actionsRow.implicitWidth) }
    ].concat(crawling ? [{ key: "crawl", span: spanFor(theme.space(60)) }] : []), cols)
    function headerTile(key) {
        var t = headerLayout.tiles
        for (var i = 0; i < t.length; i++) if (t[i].key === key) return t[i]
        return { col: 0, row: 0, colSpan: 1 }
    }
    readonly property real headerHeight: px(headerLayout.rows)

    // The rows: as many row tiles to a line as hold at least five columns each, the line's
    // columns shared out from the left
    readonly property int perLine: Math.max(1, Math.floor(cols / 6))
    function rowCol(i) { var k = i % perLine; return Math.floor(k * cols / perLine) }
    function rowSpan(i) { var k = i % perLine; return Math.floor((k + 1) * cols / perLine) - Math.floor(k * cols / perLine) }
    readonly property int rowLines: Math.ceil(rows.length / perLine)
    readonly property real wallTop: headerHeight + grout

    component HeaderTile: Corner {
        property string key: ""
        readonly property var place: page.headerTile(key)
        x: page.spanX(place.col)
        y: place.row * page.step
        width: page.spanW(place.col, place.colSpan)
        height: page.unit
        radius: theme.radiusTile
        smoothing: theme.cornerSmoothing
        color: theme.tile
    }
    component Caption: Label {
        x: page.hpad; y: page.hpad
        role: "ui"
        font.pointSize: theme.typeSmall
        color: theme.textDim
    }

    Flickable {
        id: flick
        y: page.grout
        width: page.width - page.grout
        height: page.height - page.grout * 2
        clip: true
        boundsBehavior: Flickable.StopAtBounds
        contentWidth: width
        contentHeight: Math.max(height, page.wallTop + (page.rows.length ? page.px(page.rowLines) : 0) + (page.rows.length ? 0 : page.px(4)))
        QC.ScrollBar.vertical: scrollBar
        WheelStep { flick: flick }
        MouseArea { anchors.fill: parent; onPressed: function(m) { flick.forceActiveFocus(); m.accepted = false } }

        HeaderTile {
            key: "title"
            Label {
                id: titleLabel
                x: page.hpad
                y: parent.height - page.hpad - height
                text: "Metadata"
                role: "display"
            }
        }
        HeaderTile {
            key: "filter"
            Caption { text: "Show" }
            Seg {
                id: filterSeg
                objectName: "metadata-filter"
                x: page.hpad
                y: parent.height - page.hpad - height
                options: page.filters.map(function(f, i) { return { text: page.filterLabels[i] + " " + page.filterCount(f), name: f.toLowerCase() } })
                index: Math.max(0, page.filters.indexOf(page.filter))
                onPicked: function(i) { page.filter = page.filters[i]; page.reload() }
            }
        }
        HeaderTile {
            key: "search"
            Caption { text: "Search" }
            SearchField {
                id: search
                objectName: "metadata-search"
                x: page.hpad
                y: parent.height - page.hpad - height
                width: parent.width - page.hpad * 2
                placeholder: "Filter titles"
                hint: ""
                onTextChanged: queryDebounce.restart()
            }
        }
        HeaderTile {
            key: "actions"
            Caption { text: "Every series" }
            Row {
                id: actionsRow
                x: page.hpad
                y: parent.height - page.hpad - height
                spacing: theme.space(2)
                Button {
                    objectName: "metadata-match-all"
                    text: "Match unmatched"
                    icon: "link"
                    onClicked: { var r = Door.autoMatch(); frame.toast(r.error ? r.error.message : "Looking up AniList for unmatched folders") }
                    Tooltip { text: "Looks up AniList for every folder that has no match yet" }
                }
                Button {
                    objectName: "metadata-refresh-all"
                    text: "Refresh all"
                    icon: "refresh-cw"
                    onClicked: { var r = Door.refreshAll(); frame.toast(r.error ? r.error.message : "Refreshing every matched series") }
                    Tooltip { text: "Fetches every matched series' record again" }
                }
            }
        }
        // The franchise crawl's own bar, only while it runs
        HeaderTile {
            key: "crawl"
            visible: page.crawling
            readonly property var job: page.crawlJob()
            Caption { text: "Finding franchise links" }
            Label {
                anchors.right: parent.right; anchors.rightMargin: page.hpad; y: page.hpad
                text: parent.job && parent.job.total > 0 ? parent.job.done + " of " + parent.job.total : ""
                role: "ledger"; color: theme.textDim
            }
            Corner {
                x: page.hpad; width: parent.width - page.hpad * 2; height: theme.space(1)
                y: parent.height - page.hpad - height * 2
                radius: height / 2; smoothing: theme.cornerSmoothing
                color: theme.tileSunken
                Corner {
                    width: parent.width * (parent.parent.job && parent.parent.job.total > 0 ? parent.parent.job.done / parent.parent.job.total : 0)
                    height: parent.height
                    radius: height / 2; smoothing: theme.cornerSmoothing
                    color: theme.accent
                }
            }
        }

        // Nothing to show: a tile, not text on the grout
        Tile {
            visible: page.loaded && page.rows.length === 0
            x: 0; y: page.wallTop
            width: flick.width
            height: flick.height - page.wallTop
            EmptyState {
                icon: page.counts.all === 0 ? "database" : "search"
                title: page.counts.all === 0 ? "No metadata yet" : page.query !== "" ? "No matches for \"" + page.query + "\"" : "Nothing under this filter"
                body: page.counts.all === 0 ? "Your library is empty. Add a folder in Settings and scan it to get started." : page.query !== "" ? "" : "No series here right now."
            }
        }

        Repeater {
            model: page.rows
            MetaRow {
                required property int index
                x: page.spanX(page.rowCol(index))
                y: page.wallTop + Math.floor(index / page.perLine) * page.step
                width: page.spanW(page.rowCol(index), page.rowSpan(index))
                height: page.unit
                first: index === 0
            }
        }
    }
    ThinScrollBar {
        id: scrollBar
        x: page.width - page.grout + Math.max(0, (page.grout - width) / 2)
        y: flick.y
        height: flick.height
        width: Math.min(page.grout, theme.space(1))
    }

    // One series: the poster sliver flush in the tile, the titles, the kind and the record,
    // the files against the expected count, and the row's actions, all in one stone. A click
    // anywhere else on it opens the match panel.
    component MetaRow: Corner {
        id: metaRow
        required property var modelData
        property bool first: false
        readonly property var d: modelData || ({})
        readonly property var card: d.series || ({})
        readonly property bool matched: !!(card.match_info && (card.match_info.anilist_id !== null || card.match_info.mal_id !== null))
        readonly property bool isMovie: card.kind === "Movie" || card.format === "MOVIE"
        readonly property bool missing: card.missing === true
        readonly property real expected: d.expected !== null && d.expected !== undefined ? d.expected : (isMovie ? 1 : 0)
        readonly property real have: d.have || 0
        readonly property real pct: expected > 0 ? Math.min(1, have / expected) : 0
        readonly property real extra: expected > 0 ? Math.max(0, have - expected) : 0
        readonly property real inPad: theme.space(3)
        objectName: "meta-row-" + card.id
        radius: theme.radiusTile
        smoothing: theme.cornerSmoothing
        color: m.containsMouse ? theme.tileHover : theme.tile

        // Under everything else, so the buttons take their own clicks
        MouseArea {
            id: m
            anchors.fill: parent
            hoverEnabled: true
            cursorShape: Qt.PointingHandCursor
            onClicked: matchPanel.openFor(metaRow.card.id, metaRow.card.title, metaRow.card.match_info)
        }

        // The poster: flush with the tile's left edge, its own corners the tile's.
        // Decoded at its own displayed size and shown only once it is Ready.
        Corner {
            id: thumb
            width: Math.round(height / 1.5); height: parent.height
            radius: theme.radiusTile; smoothing: theme.cornerSmoothing
            color: theme.tileSunken
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

        // The titles, between the poster and the readouts
        Column {
            anchors.left: thumb.right; anchors.leftMargin: metaRow.inPad
            anchors.right: readout.left; anchors.rightMargin: metaRow.inPad
            anchors.verticalCenter: parent.verticalCenter
            spacing: theme.space(1)
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

        // The readouts: kind and record over the files bar and its count
        Column {
            id: readout
            anchors.right: actions.left; anchors.rightMargin: metaRow.inPad
            anchors.verticalCenter: parent.verticalCenter
            width: theme.space(56)
            spacing: theme.space(2)
            Row {
                spacing: theme.space(1.5)
                Chip { small: true; mono: false; color: theme.raised; text: metaRow.isMovie ? "Film" : "Series"; textColor: theme.hue(Theme.formatHue(metaRow.isMovie ? "MOVIE" : "TV")) }
                Chip { small: true; mono: false; color: theme.raised
                    text: !metaRow.matched ? "No match" : page.providerLabel(metaRow.card.match_info.provider)
                    textColor: metaRow.matched ? theme.text : theme.textDim }
                Chip { visible: metaRow.missing; small: true; mono: false; color: theme.redSoft; text: "Missing"; textColor: theme.red }
            }
            Item {
                width: parent.width
                height: filesCount.implicitHeight
                Label {
                    id: filesCount
                    text: metaRow.have + " of " + (metaRow.expected > 0 ? metaRow.expected : "?") + (metaRow.extra > 0 ? ", " + metaRow.extra + " extra" : "")
                    role: "ledger"
                    color: metaRow.extra > 0 ? theme.behind : theme.textDim
                    Tooltip { text: metaRow.extra > 0 ? "Files beyond the expected count, likely misnamed or stray" : "" }
                }
                Corner {
                    anchors.left: filesCount.right; anchors.leftMargin: theme.space(3)
                    anchors.right: parent.right
                    anchors.verticalCenter: filesCount.verticalCenter
                    height: theme.space(1)
                    radius: height / 2; smoothing: theme.cornerSmoothing
                    color: theme.tileSunken
                    Corner {
                        width: Math.max(metaRow.pct > 0 ? height : 0, parent.width * metaRow.pct)
                        height: parent.height
                        radius: height / 2; smoothing: theme.cornerSmoothing
                        color: metaRow.extra > 0 ? theme.behind : theme.caughtUp
                    }
                }
            }
        }

        // The row's actions
        Column {
            id: actions
            anchors.right: parent.right; anchors.rightMargin: metaRow.inPad
            anchors.verticalCenter: parent.verticalCenter
            spacing: theme.space(1.5)
            Button {
                id: matchBtn
                objectName: metaRow.first ? "row-match" : ""
                width: Math.max(implicitWidth, refreshBtn.implicitWidth)
                text: "Match"; icon: "link"; small: true
                onClicked: matchPanel.openFor(metaRow.card.id, metaRow.card.title, metaRow.card.match_info)
            }
            Button {
                id: refreshBtn
                visible: !metaRow.missing
                objectName: metaRow.first ? "row-refresh" : ""
                width: Math.max(implicitWidth, matchBtn.implicitWidth)
                text: "Refresh"; icon: "refresh-cw"; small: true
                onClicked: { var r = Door.refreshSeries(metaRow.card.id); frame.toast(r.error ? r.error.message : "Refreshing " + (metaRow.card.title || "")) }
            }
            Button {
                visible: metaRow.missing
                objectName: "row-forget"
                width: Math.max(implicitWidth, matchBtn.implicitWidth)
                text: "Forget"; icon: "trash-2"; small: true; danger: true
                onClicked: page.forget(metaRow.card)
                Tooltip { text: "The folder is gone: drop this series and its history" }
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
}
