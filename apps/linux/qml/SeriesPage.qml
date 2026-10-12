// A series as a bento (DESIGN.md, Layout): every fact cut into its own tile on the library's
// module, set in the one joint (theme.grout) that is also the page's margin. The top band:
// a back tile over the poster and an actions tile in the leading columns; the banner, the
// title, Continue and a row of fact tiles beside them, the band's two sides landing on the
// same unit row. Under it two columns: the episode ledger (with the extras and stray files)
// and a side column of synopsis, tags, relations, recommendations and characters. The
// shorter column's last tile runs down to the taller one's foot, and both to the viewport's,
// so the page never ends in bare grout. One GetSeries draws it all; the relations come from
// GetFranchiseGraph and follow the crawl through graphChanged.
import QtQuick
import QtQuick.Controls.Basic as QC
import com.marcusrosado.AniBeam
import "mosaic.js" as Mosaic

Item {
    id: page
    objectName: "series-page"
    property var props: ({})
    property alias scrollY: flick.contentY
    property var detail: null
    property var graph: null
    readonly property var card: detail ? detail.card : ({})
    property string title: card.title || frame.nav.current.label
    // Guarded, not a bare relabel call: title falls back to nav.current.label itself
    // before the first GetSeries reply lands, so an unguarded write here relabels the
    // trail entry to the very value it already reads, which changes nav.current, which
    // is what this binding depends on for that same fallback branch, which QML sees as
    // the binding writing back into its own dependency mid-evaluation ("Binding loop
    // detected for property title"). Skipping the write when nothing would actually
    // change breaks that cycle; once the real title arrives, card.title short-circuits
    // the fallback branch, the dependency on nav.current drops, and this relabel is the
    // only write left standing.
    onTitleChanged: if (frame.nav.current.label !== title) frame.nav.relabel(title)
    function escapePressed() { return false }
    property bool spoilers: false
    property bool synopsisOpen: false
    property bool tagsOpen: false
    property real nowMs: Date.now()
    property real optimisticProgress: -1
    property var optimisticFile: -1
    property bool optimisticWatched: false
    readonly property bool isMovie: card.kind === "Movie"
    readonly property bool hasTracker: !!(card.match_info && (card.match_info.anilist_id || card.match_info.mal_id))
    readonly property var titles: card.titles || ({})
    // The other language's title under the display one, then the native script
    readonly property string altTitle: {
        var t = titles
        if (t.romaji && t.english && t.romaji !== t.english) return card.title === t.english ? t.romaji : t.english
        return ""
    }
    readonly property string nativeTitle: titles.native && titles.native !== card.title ? titles.native : ""
    readonly property string tone: card.poster ? Fmt.posterTone(card.poster) : ""

    function contextItems() {
        return [
            { text: "Rescan show", icon: "refresh-cw", action: page.rescan },
            { text: "Open in Metadata", icon: "database", action: function() { frame.go("metadata", { q: page.title }) } }
        ]
    }
    function rescan() { var r = Door.rescanSeries(page.props.id); frame.toast(r.error ? r.error.message : "Rescan started") }
    function load() {
        var r = Door.getSeries(props.id)
        if (r.error) { frame.toast(r.error.message); return }
        detail = r.reply.detail
        optimisticProgress = -1
        optimisticFile = -1
        loadGraph()
    }
    function loadGraph() {
        if (!detail || !detail.has_graph) { graph = null; return }
        var g = Door.getFranchiseGraph(props.id)
        graph = g.error ? null : g.reply.layout
    }
    Component.onCompleted: { load(); Door.refreshAiring(props.id) }
    Timer { interval: 1000; running: !!card.next_airing && card.next_airing.at * 1000 > page.nowMs; repeat: true; onTriggered: page.nowMs = Date.now() }
    Timer { id: reloadDebounce; interval: 200; onTriggered: page.load() }
    Timer { id: graphDebounce; interval: 400; onTriggered: page.loadGraph() }
    Connections {
        target: Door
        function onSeriesChanged(cards) { for (var i = 0; i < cards.length; i++) if (cards[i].id === page.props.id) { reloadDebounce.restart(); return } }
        function onAiringRefreshed(series, updated) { if (series === page.props.id && updated) reloadDebounce.restart() }
        function onProgressSet(series, progress, outcomes) { if (series === page.props.id) reloadDebounce.restart() }
        function onScored(series, score, outcomes) { if (series === page.props.id) { reloadDebounce.restart(); frame.toast(outcomes.every(function(o) { return o.ok }) ? "Rated " + (score < 0 ? "cleared" : Fmt.score(score)) : "Score failed") } }
        function onMarked(series, episode, outcomes) { if (series === page.props.id) reloadDebounce.restart() }
        // The crawl fills the franchise in behind the first read; its root is an AniList id
        function onGraphChanged(root) { if (page.graph === null || page.graph.root === root) graphDebounce.restart() }
        function onCrawlFinished(fetched, deferred) { if (page.detail && page.detail.has_graph) graphDebounce.restart() }
        // The core resolves card.title (and every other localised field) at query time,
        // so a title-language flip from the rail's JP/EN switch needs a fresh GetSeries;
        // Door reports every preference write here regardless of what changed, exactly
        // like LibraryPage.qml's own onPreferencesChanged, so there is nothing to diff.
        function onPreferencesChanged() { reloadDebounce.restart() }
    }
    function openFile(file) { frame.go("player", { file: file }, page.title) }
    function status(s) { return { Releasing: "Airing", Finished: "Finished", NotYetReleased: "Upcoming", Cancelled: "Cancelled", Hiatus: "On hiatus" }[s] || s }
    function listStatus(s) { return { Watching: "Watching", Planning: "Planning", Completed: "Completed", Paused: "Paused", Dropped: "Dropped", Repeating: "Rewatching" }[s] || s }
    function formatLabel(f) { return isMovie ? "Film" : !f ? "Series" : ({ TV: "TV", TV_SHORT: "TV short", OVA: "OVA", ONA: "ONA", SPECIAL: "Special", MOVIE: "Film" })[f] || f.replace(/_/g, " ") }
    function relationLabel(r) { if (!r) return ""; var s = String(r).replace(/_/g, " ").toLowerCase(); return s.charAt(0).toUpperCase() + s.slice(1) }
    // An episode's name: Fmt.episodeName, the one rule the player and the feed share (its
    // own title, else "Episode 13"; the file name is never a title). A lone film is the film.
    function episodeName(ep) {
        if (!ep.title && isMovie && detail && detail.episodes.length === 1) return page.title
        return Fmt.episodeName(ep.title || "", ep.number)
    }

    // Track to here / untrack to here, optimistic; the core confirms through progressSet.
    // The progress sent to the core is always the floor of ep.number (progress counts
    // whole episodes), but a half-numbered episode (a split cour, a Part 2) then sits on
    // the wrong side of that integral cutoff for its own row: marking 12.5 floors to a
    // target of 12, and 12.5 <= 12 is false, so the row just clicked would not flip.
    // optimisticFile pins the clicked row to the intended state regardless of the number
    // comparison; every other row still reads the integral target normally.
    function marker(ep) {
        var watched = ep.watched
        var target = watched ? Math.max(0, Math.floor(ep.number) - 1) : Math.floor(ep.number)
        optimisticProgress = target
        optimisticFile = ep.file
        optimisticWatched = !watched
        var r = Door.setProgress(props.id, target)
        if (r.error) { optimisticProgress = -1; optimisticFile = -1; frame.toast(r.error.message) }
    }
    function watchedWithOptimism(ep) {
        if (optimisticProgress < 0) return ep.watched
        if (ep.file === optimisticFile) return optimisticWatched
        return ep.number <= optimisticProgress
    }

    // ---- What Continue plays: the next episode, else (a film, or caught up) the first
    readonly property var episodes: detail ? detail.episodes : []
    readonly property var nextEpisode: {
        if (!detail || !detail.next_up) return null
        for (var i = 0; i < episodes.length; i++) if (episodes[i].file === detail.next_up) return episodes[i]
        return null
    }
    readonly property var playEpisode: nextEpisode ? nextEpisode : (episodes.length ? episodes[0] : null)
    readonly property bool tracked: !!detail && detail.progress.watched !== null && detail.progress.watched !== undefined
    readonly property int watchedCount: tracked ? detail.progress.watched : 0
    readonly property bool caughtUp: tracked && !nextEpisode && watchedCount > 0
    readonly property string playVerb: nextEpisode && watchedCount > 0 ? "Continue" : caughtUp ? "Watch again" : "Play"
    readonly property string progressText: {
        if (!tracked) return ""
        var p = detail.progress
        return p.total ? watchedCount + " of " + p.total + (p.estimate ? "+" : "") + " watched" : watchedCount + " watched"
    }
    readonly property real progressFraction: tracked && detail.progress.total ? Math.min(1, watchedCount / detail.progress.total) : 0
    readonly property bool airsAhead: !!card.next_airing && card.next_airing.at * 1000 > nowMs

    // ---- The module: the library's own columns and unit, from this page's width
    readonly property real grout: theme.grout
    readonly property var geo: Mosaic.columns(Math.max(1, width - grout), theme.posterWidth, grout)
    readonly property int cols: geo.cols
    readonly property real unit: geo.u
    readonly property real step: unit + grout
    function px(units) { return Mosaic.unitsToPx(units, unit, grout) }
    function unitsFor(h) { return Mosaic.unitsFor(h, unit, grout) }
    function spanX(col) { return geo.xs[Math.max(0, Math.min(col, cols - 1))] }
    function spanW(col, span) { col = Math.max(0, Math.min(col, cols - 1)); var last = Math.max(col, Math.min(cols - 1, col + span - 1)); return geo.xs[last] + geo.ws[last] - geo.xs[col] }
    readonly property real pad: theme.space(4)

    // The top band. P leading columns carry back, poster and actions; the R beside them
    // the banner, the title (with Continue at its right when there is room) and the facts.
    readonly property int lead: cols >= 4 ? 2 : 1
    readonly property int rest: Math.max(1, cols - lead)
    readonly property bool continueBeside: rest >= 7
    readonly property int continueSpan: continueBeside ? Math.max(3, Math.round(rest * 0.3)) : 0
    readonly property int posterUnits: unitsFor(spanW(0, lead) * 1.5)
    readonly property int titleUnits: Math.max(2, unitsFor(titleBlock.implicitHeight + pad * 2), continueBeside ? unitsFor(continueBlock.implicitHeight + pad * 2) : 0)
    function factSpan(w) { return Math.max(1, Math.ceil((w + pad * 2 + grout) / (geo.w + grout))) }
    readonly property var factLayout: Mosaic.header(facts.map(function(f) { return { key: f.key, span: factSpan(f.width), flex: f.flex } }), rest)
    function factSlot(key) { var t = factLayout.tiles; for (var i = 0; i < t.length; i++) if (t[i].key === key) return t[i]; return { col: 0, row: 0, colSpan: 1 } }
    readonly property int factRows: facts.length ? factLayout.rows : 0
    readonly property int bannerMin: cols >= 10 ? 4 : 3
    readonly property int bandRows: Math.max(1 + posterUnits + 1, bannerMin + titleUnits + factRows)
    readonly property int bannerUnits: bandRows - titleUnits - factRows
    readonly property real bandHeight: px(bandRows)

    // ---- The facts: one tile each, spanned to its words
    FontMetrics { id: captionMetrics; font.family: theme.fontFamily; font.pointSize: theme.typeSmall; font.weight: theme.roleWeight("ui"); font.variableAxes: theme.roleAxes("ui") }
    FontMetrics { id: valueMetrics; font.family: theme.fontFamily; font.pointSize: theme.typeLarge; font.weight: theme.roleWeight("title"); font.variableAxes: theme.roleAxes("title") }
    FontMetrics { id: pillMetrics; font.family: theme.fontFamily; font.pointSize: theme.typeNormal; font.weight: 600; font.variableAxes: ({ "wdth": 88, "wght": 600 }) }
    readonly property var facts: {
        if (!detail) return []
        var out = []
        function add(key, caption, value, extra) {
            var w = Math.max(captionMetrics.advanceWidth(caption), valueMetrics.advanceWidth(value) + (extra || 0))
            out.push({ key: key, caption: caption, value: value, width: w, flex: false })
        }
        if (card.next_airing && card.next_airing.at * 1000 > Date.now())
            out.push({ key: "airing", caption: "Episode " + card.next_airing.episode + " airs in", value: "",
                       width: Math.max(captionMetrics.advanceWidth("Episode 00 airs in"), pillMetrics.advanceWidth("0d 00h 00m 00s") + theme.space(6)), flex: true })
        if (card.community_score !== null && card.community_score !== undefined) add("score", "Average score", Fmt.score(card.community_score))
        add("mine", "Your score", card.my_score !== null && card.my_score !== undefined ? Fmt.score(card.my_score) : "Rate", 0)
        if (!isMovie) add("episodes", "Episodes", card.total_episodes ? String(card.total_episodes) + (card.total_is_estimate ? "+" : "") : "?")
        add("disk", isMovie ? "On disk" : "On disk", isMovie ? "Film" : Fmt.plural(detail.progress.on_disk, "file", "files"))
        if (card.status) add("status", "Status", status(card.status))
        add("format", "Format", formatLabel(card.format))
        if (detail.year) add("year", "Year", String(detail.year))
        if (detail.studio) add("studio", "Studio", detail.studio)
        if (card.list_status) add("list", "On your list", listStatus(card.list_status), theme.space(4))
        if (detail.rewatch_count > 0) add("rewatch", "Rewatched", detail.rewatch_count + (detail.rewatch_count === 1 ? " time" : " times"))
        if (out.length && !out.some(function(f) { return f.flex })) out[out.length - 1].flex = true
        return out
    }

    // ---- The body: the ledger column and the side column, stacked when narrow
    readonly property bool twoCol: cols >= 6
    readonly property int ledgerSpan: twoCol ? Math.ceil(cols * 0.6) : cols
    readonly property real ledgerW: spanW(0, ledgerSpan)
    readonly property real sideX: twoCol ? spanX(ledgerSpan) : 0
    readonly property real sideW: twoCol ? spanW(ledgerSpan, cols - ledgerSpan) : spanW(0, cols)
    // Every body tile's natural height snapped to whole units, and the stretch that brings
    // a column's last tile down to the taller column's foot and to the viewport's
    function colNatural(tiles) { var h = 0, n = 0; for (var i = 0; i < tiles.length; i++) if (tiles[i].visible && tiles[i].inColumn) { h += tiles[i].snapped; n++ } return n ? h + (n - 1) * grout : 0 }
    function lastShown(tiles) { for (var i = tiles.length - 1; i >= 0; i--) if (tiles[i].visible && tiles[i].inColumn) return tiles[i]; return null }
    readonly property var ledgerTiles: [continueTile, episodesTile, extrasTile, strayTile]
    readonly property var sideTiles: [synopsisTile, tagsTile, relationsTile, recommendationsTile, charactersTile]
    readonly property real ledgerNatural: colNatural(ledgerTiles)
    readonly property real sideNatural: colNatural(sideTiles)
    readonly property real bodyTop: bandHeight + grout
    readonly property real floorHeight: Math.max(0, flick.height - bodyTop)
    readonly property real bodyHeight: twoCol ? Math.max(ledgerNatural, sideNatural, floorHeight)
                                              : Math.max(ledgerNatural + (sideNatural > 0 ? grout + sideNatural : 0), floorHeight)
    readonly property var ledgerLast: lastShown(ledgerTiles)
    readonly property var sideLast: lastShown(sideTiles)
    readonly property real ledgerStretch: twoCol ? bodyHeight - ledgerNatural : (sideNatural > 0 ? 0 : bodyHeight - ledgerNatural)
    readonly property real sideStretch: twoCol ? bodyHeight - sideNatural : bodyHeight - ledgerNatural - (sideNatural > 0 ? grout + sideNatural : 0)

    component Stone: Tile {
        property real natural: 0
        property real stretch: 0
        property bool inColumn: true
        readonly property real snapped: page.px(page.unitsFor(natural + padding * 2))
        padding: page.pad
        width: parent ? parent.width : 0
        height: snapped + stretch
    }
    component Caption: Label {
        role: "ui"
        font.pointSize: theme.typeSmall
        color: theme.textDim
    }
    component Heading: Row {
        property string text: ""
        property int count: -1
        spacing: theme.space(2)
        Label { id: headingText; text: parent.text; role: "title"; font.pointSize: theme.typeNormal }
        Label { visible: parent.count >= 0; text: String(parent.count); role: "ledger"; color: theme.textDim; anchors.baseline: headingText.baseline }
    }

    Flickable {
        id: flick
        y: page.grout
        width: page.width - page.grout
        height: page.height - page.grout * 2
        clip: true
        boundsBehavior: Flickable.StopAtBounds
        contentWidth: width
        contentHeight: page.bodyTop + page.bodyHeight
        QC.ScrollBar.vertical: scrollBar
        WheelStep { flick: flick }
        MouseArea { anchors.fill: parent; onPressed: function(m) { flick.forceActiveFocus(); m.accepted = false } }

        // ---- Back: its own tile at the top of the leading columns
        Tile {
            id: backTile
            objectName: "series-back"
            x: page.spanX(0); y: 0
            width: page.spanW(0, page.lead); height: page.unit
            padding: page.pad
            raised: backMouse.containsMouse
            Row {
                anchors.verticalCenter: parent.verticalCenter
                spacing: theme.space(2)
                Icon { anchors.verticalCenter: parent.verticalCenter; glyph: "chevron-left"; size: theme.space(5); color: theme.text }
                Label {
                    anchors.verticalCenter: parent.verticalCenter
                    width: Math.min(implicitWidth, backTile.width - backTile.padding * 2 - theme.space(7))
                    text: frame.nav.backLabel
                    role: "title"; font.pointSize: theme.typeNormal
                    elide: Text.ElideRight
                }
            }
            MouseArea { id: backMouse; anchors.fill: parent; anchors.margins: -backTile.padding; hoverEnabled: true; cursorShape: Qt.PointingHandCursor; onClicked: frame.nav.back() }
        }

        // ---- The poster, cut to the leading columns
        Corner {
            id: posterTile
            x: page.spanX(0); y: page.step
            width: page.spanW(0, page.lead); height: page.px(page.posterUnits)
            radius: theme.radiusTile; smoothing: theme.cornerSmoothing
            color: page.tone !== "" ? theme.plate(page.tone) : theme.tile
            fillItem: poster.status === Image.Ready ? poster : null
            Image {
                id: poster; visible: false
                width: parent.width; height: parent.height
                source: card.poster ? "file://" + card.poster : ""
                sourceSize: Qt.size(Math.ceil(parent.width * theme.devicePixelRatio), Math.ceil(parent.height * theme.devicePixelRatio))
                fillMode: Image.PreserveAspectCrop; asynchronous: true
            }
            Label { visible: !card.poster; anchors.centerIn: parent; text: "No poster"; color: theme.textDim; role: "ui"; font.pointSize: theme.typeSmall }
        }

        // ---- Actions, under the poster down to the band's foot
        Tile {
            id: actionsTile
            x: page.spanX(0); y: page.step * (1 + page.posterUnits)
            width: page.spanW(0, page.lead); height: page.px(page.bandRows - 1 - page.posterUnits)
            padding: page.pad
            Flow {
                width: parent.width
                anchors.verticalCenter: parent.verticalCenter
                spacing: theme.space(2)
                Button { objectName: "series-hide"; small: true; text: card.hidden ? "Unhide" : "Hide"; icon: card.hidden ? "eye" : "eye-off"
                    onClicked: { var r = Door.setHidden(props.id, !card.hidden); if (r.error) frame.toast(r.error.message) }
                    Tooltip { text: "Hidden from every list, and tracker sync stops" } }
                Button { visible: !!(detail && detail.site_url); small: true; text: "AniList"; icon: "external-link"; onClicked: Qt.openUrlExternally(detail.site_url) }
                Button { small: true; text: "Rescan"; icon: "refresh-cw"; onClicked: page.rescan() }
            }
        }

        // ---- The banner: the art across the band, or a plate in the poster's tone
        Corner {
            id: bannerTile
            readonly property bool hasBanner: !!(detail && detail.banner)
            x: page.spanX(page.lead); y: 0
            width: page.spanW(page.lead, page.rest); height: page.px(page.bannerUnits)
            radius: theme.radiusTile; smoothing: theme.cornerSmoothing
            color: page.tone !== "" ? theme.plate(page.tone) : theme.tile
            fillItem: hasBanner && art.status === Image.Ready ? art : null
            Image {
                id: art; visible: false
                width: parent.width; height: parent.height
                source: bannerTile.hasBanner ? "file://" + detail.banner : ""
                fillMode: Image.PreserveAspectCrop; asynchronous: true
            }
        }

        // ---- The title: display cut, then the other language and the native script
        Tile {
            id: titleTile
            x: page.spanX(page.lead); y: page.step * page.bannerUnits
            width: page.spanW(page.lead, page.rest - page.continueSpan)
            height: page.px(page.titleUnits)
            padding: page.pad
            tone: page.tone !== "" ? page.tone : "transparent"
            Column {
                id: titleBlock
                width: parent.width
                anchors.bottom: parent.bottom
                spacing: theme.space(1)
                Label {
                    width: parent.width
                    text: page.title
                    role: "display"
                    wrapMode: Text.Wrap; maximumLineCount: 2; elide: Text.ElideRight
                    lineHeight: 0.95
                }
                Label { visible: page.altTitle !== ""; width: parent.width; text: page.altTitle; role: "ui"; elide: Text.ElideRight; color: theme.contrast(theme.textDim, titleTile.fill) >= 4.5 ? theme.textDim : theme.text }
                Label { visible: page.nativeTitle !== ""; width: parent.width; text: page.nativeTitle; role: "ui"; font.pointSize: theme.typeSmall; elide: Text.ElideRight; color: theme.contrast(theme.textDim, titleTile.fill) >= 4.5 ? theme.textDim : theme.text }
            }
        }

        // ---- The facts row(s) under the title
        Repeater {
            model: page.facts
            delegate: Tile {
                id: fact
                required property var modelData
                readonly property var place: modelData ? page.factSlot(modelData.key) : ({ col: 0, row: 0, colSpan: 1 })
                readonly property bool clickable: modelData.key === "mine"
                objectName: "fact-" + modelData.key
                x: page.spanX(page.lead + place.col)
                y: page.step * (page.bannerUnits + page.titleUnits + place.row)
                width: page.spanW(page.lead + place.col, place.colSpan)
                height: page.unit
                padding: theme.space(3)
                raised: clickable && factMouse.containsMouse
                Caption { text: fact.modelData.caption; width: parent.width; elide: Text.ElideRight }
                Row {
                    anchors.bottom: parent.bottom
                    spacing: theme.space(1.5)
                    StatusDot { visible: fact.modelData.key === "list"; anchors.verticalCenter: parent.verticalCenter; status: card.list_status || "" }
                    Icon { visible: fact.modelData.key === "score" || fact.modelData.key === "mine"; anchors.verticalCenter: parent.verticalCenter; glyph: "star"; size: theme.space(4.5)
                        color: fact.modelData.key === "mine" && card.my_score !== null && card.my_score !== undefined ? theme.accent : theme.textDim }
                    Label {
                        visible: fact.modelData.key !== "airing"
                        text: fact.modelData.value
                        role: "title"
                        width: Math.min(implicitWidth, fact.width - fact.padding * 2 - theme.space(7))
                        elide: Text.ElideRight
                    }
                    // The countdown: a pill in the accent under ground-coloured text
                    Corner {
                        visible: fact.modelData.key === "airing"
                        width: countdown.implicitWidth + theme.space(6); height: countdown.implicitHeight + theme.space(2)
                        radius: height / 2; smoothing: theme.cornerSmoothing
                        color: theme.accent
                        Label {
                            id: countdown
                            anchors.centerIn: parent
                            text: card.next_airing ? Fmt.countdownSeconds(card.next_airing.at - page.nowMs / 1000) : ""
                            role: "ledger"; font.pointSize: theme.typeNormal
                            font.weight: 600; font.variableAxes: ({ "wdth": 88, "wght": 600 })
                            color: theme.contrast(theme.ground, theme.accent) >= 4.5 ? theme.ground : theme.text
                        }
                    }
                }
                MouseArea {
                    id: factMouse
                    enabled: fact.clickable
                    anchors.fill: parent; anchors.margins: -fact.padding
                    hoverEnabled: true; cursorShape: Qt.PointingHandCursor
                    onClicked: scorePicker.openAt(fact, card.my_score === null || card.my_score === undefined ? -1 : card.my_score)
                }
            }
        }

        // ---- The ledger column
        Column {
            id: ledgerCol
            x: 0; y: page.bodyTop
            width: page.ledgerW
            spacing: page.grout

            // Continue: what plays next and how far along the series is
            Stone {
                id: continueTile
                visible: !!page.detail && page.playEpisode !== null
                natural: continueBlock.implicitHeight
                stretch: page.ledgerLast === continueTile ? page.ledgerStretch : 0
                // Beside the title on a wide window; this tile is then its parent's
                parent: page.continueBeside ? continueSlot : ledgerCol
                inColumn: !page.continueBeside
                width: parent ? parent.width : 0
                height: page.continueBeside ? continueSlot.height : snapped + stretch
                tone: page.tone !== "" && page.continueBeside ? page.tone : "transparent"
                Column {
                    id: continueBlock
                    width: parent.width
                    anchors.verticalCenter: parent.verticalCenter
                    spacing: theme.space(2)
                    Row {
                        width: parent.width
                        spacing: theme.space(3)
                        Button {
                            id: continueButton
                            objectName: "series-continue"
                            anchors.verticalCenter: parent.verticalCenter
                            primary: true; icon: "play"
                            text: page.playVerb
                            onClicked: if (page.playEpisode) page.openFile(page.playEpisode.file)
                        }
                        Column {
                            anchors.verticalCenter: parent.verticalCenter
                            width: parent.width - continueButton.width - parent.spacing
                            Label {
                                width: parent.width
                                text: page.playEpisode ? (page.isMovie && page.episodes.length === 1 ? page.title : page.playEpisode.code) : ""
                                role: "ledger"; color: theme.textDim; elide: Text.ElideRight
                            }
                            Label {
                                visible: !(page.isMovie && page.episodes.length === 1)
                                width: parent.width
                                text: page.playEpisode ? page.episodeName(page.playEpisode) : ""
                                role: "title"; font.pointSize: theme.typeNormal; elide: Text.ElideRight
                            }
                        }
                    }
                    // Tracked: "3 of 12 watched" over the bar. Untracked says so, no empty bar.
                    Item {
                        visible: !page.isMovie || page.tracked
                        width: parent.width
                        height: progressLine.implicitHeight + (page.tracked && page.detail.progress.total ? theme.space(2.5) : 0)
                        Label {
                            id: progressLine
                            text: page.tracked ? page.progressText : "Not tracked, " + Fmt.plural(page.detail ? page.detail.progress.on_disk : 0, "episode", "episodes") + " on disk"
                            role: "ledger"; color: theme.textDim
                        }
                        Corner {
                            visible: page.tracked && !!page.detail.progress.total
                            anchors.bottom: parent.bottom
                            width: parent.width; height: theme.space(1)
                            radius: height / 2; smoothing: theme.cornerSmoothing
                            color: theme.tileSunken
                            Corner {
                                width: Math.max(page.progressFraction > 0 ? height : 0, parent.width * page.progressFraction); height: parent.height
                                radius: height / 2; smoothing: theme.cornerSmoothing
                                color: theme.accent
                            }
                        }
                    }
                }
            }

            // The episodes: the ledger proper
            Stone {
                id: episodesTile
                objectName: "series-episodes"
                visible: page.episodes.length > 0
                natural: episodeCol.implicitHeight
                stretch: page.ledgerLast === episodesTile ? page.ledgerStretch : 0
                Column {
                    id: episodeCol
                    width: parent.width
                    spacing: theme.space(0.5)
                    Heading { text: page.isMovie ? "Film" : "Episodes"; count: page.episodes.length; bottomPadding: theme.space(2) }
                    Repeater {
                        model: page.episodes
                        EpisodeRow {
                            required property var modelData
                            required property int index
                            objectName: "episode-row-" + index
                            episode: modelData
                            hasTracker: page.hasTracker
                            title: page.episodeName(modelData)
                            watched: page.watchedWithOptimism(modelData)
                            onPlay: page.openFile(modelData.file)
                            onMarker: page.marker(modelData)
                        }
                    }
                }
            }

            // Openings, endings and the rest, grouped
            Stone {
                id: extrasTile
                visible: !!page.detail && page.detail.extras.length > 0
                natural: extrasCol.implicitHeight
                stretch: page.ledgerLast === extrasTile ? page.ledgerStretch : 0
                Column {
                    id: extrasCol
                    width: parent.width
                    spacing: theme.space(2)
                    Heading { text: "Openings, endings and more"; count: page.detail ? page.detail.extras.length : 0 }
                    Repeater {
                        model: [["Op", "Openings"], ["Ed", "Endings"], ["Pv", "Previews and trailers"], ["Sp", "Specials"], ["Other", "Other"]]
                        Column {
                            required property var modelData
                            readonly property var group: page.detail ? page.detail.extras.filter(function(x) { return x.kind === modelData[0] }) : []
                            visible: group.length > 0
                            width: parent.width
                            spacing: theme.space(0.5)
                            Caption { text: modelData[1] + ", " + group.length }
                            Repeater { model: group; EpisodeRow { required property var modelData; episode: ({ code: modelData.code, title: modelData.label, resume: modelData.resume, watched: false, next_up: false }); title: modelData.label; onPlay: page.openFile(modelData.file) } }
                        }
                    }
                }
            }

            // Files numbered past the matched count: named by their file, since that is
            // what needs reviewing
            Stone {
                id: strayTile
                visible: !!page.detail && page.detail.unmatched_files.length > 0
                natural: strayCol.implicitHeight
                stretch: page.ledgerLast === strayTile ? page.ledgerStretch : 0
                Column {
                    id: strayCol
                    width: parent.width
                    spacing: theme.space(1)
                    Heading { text: "Extra files"; count: page.detail ? page.detail.unmatched_files.length : 0 }
                    Row {
                        width: parent.width
                        spacing: theme.space(2)
                        Icon { glyph: "triangle-alert"; size: theme.space(4); color: theme.yellow }
                        Label {
                            width: parent.width - theme.space(6)
                            wrapMode: Text.Wrap; role: "ui"; font.pointSize: theme.typeSmall; color: theme.textDim
                            text: page.detail ? (page.detail.unmatched_files.length === 1 ? "One file is" : page.detail.unmatched_files.length + " files are") + " more than the " + (card.total_episodes || 0) + " expected episodes: misnamed, duplicates or specials. Rename or remove " + (page.detail.unmatched_files.length === 1 ? "it." : "them.") : ""
                        }
                    }
                    Repeater { model: page.detail ? page.detail.unmatched_files : []
                        EpisodeRow { required property var modelData; episode: modelData; extra: true; title: Fmt.episodeName(modelData.title || "", modelData.number); detailText: modelData.path.split("/").pop(); onPlay: page.openFile(modelData.file) } }
                }
            }
        }

        // Continue's slot beside the title on a wide window
        Item {
            id: continueSlot
            x: page.spanX(page.lead + page.rest - page.continueSpan)
            y: page.step * page.bannerUnits
            width: page.continueBeside ? page.spanW(page.lead + page.rest - page.continueSpan, page.continueSpan) : 0
            height: page.px(page.titleUnits)
            visible: page.continueBeside
        }

        // ---- The side column
        Column {
            id: sideCol
            x: page.sideX
            y: page.twoCol ? page.bodyTop : page.bodyTop + page.ledgerNatural + (page.ledgerNatural > 0 ? page.grout : 0)
            width: page.sideW
            spacing: page.grout

            Stone {
                id: synopsisTile
                visible: !!page.detail && synopsis.text !== ""
                natural: synopsisCol.implicitHeight
                stretch: page.sideLast === synopsisTile ? page.sideStretch : 0
                Column {
                    id: synopsisCol
                    width: parent.width
                    spacing: theme.space(2)
                    Heading { text: "Synopsis" }
                    Label {
                        id: synopsis
                        width: parent.width
                        text: page.detail ? page.detail.synopsis.replace(/<br\s*\/?>/gi, "\n").replace(/<[^>]*>/g, "").replace(/[ \t]+/g, " ").replace(/\n\s*\n+/g, "\n").trim() : ""
                        role: "ui"; color: theme.textDim
                        wrapMode: Text.Wrap; elide: Text.ElideRight
                        maximumLineCount: page.synopsisOpen ? 1000 : 8
                        lineHeight: 1.15
                    }
                    Button { visible: synopsis.truncated || page.synopsisOpen; small: true; text: page.synopsisOpen ? "Less" : "More"; icon: page.synopsisOpen ? "chevron-up" : "chevron-down"; onClicked: page.synopsisOpen = !page.synopsisOpen }
                }
            }

            // Tags by rank; spoiler and adult tags behind the toggle
            Stone {
                id: tagsTile
                readonly property var tags: page.detail ? page.detail.tags.filter(function(t) { return page.spoilers || (!t.spoiler && !t.adult) }).sort(function(a, b) { return b.rank - a.rank }) : []
                readonly property bool anySpoiler: page.detail ? page.detail.tags.some(function(t) { return t.spoiler || t.adult }) : false
                visible: !!page.detail && page.detail.tags.length > 0
                natural: tagsCol.implicitHeight
                stretch: page.sideLast === tagsTile ? page.sideStretch : 0
                Column {
                    id: tagsCol
                    width: parent.width
                    spacing: theme.space(2)
                    Item {
                        width: parent.width; height: Math.max(tagsHeading.height, tagButtons.height)
                        Heading { id: tagsHeading; anchors.verticalCenter: parent.verticalCenter; text: "Tags"; count: tagsTile.tags.length }
                        Row {
                            id: tagButtons
                            anchors.right: parent.right; anchors.verticalCenter: parent.verticalCenter
                            spacing: theme.space(2)
                            Button { visible: tagsTile.anySpoiler; small: true; text: page.spoilers ? "Hide spoilers" : "Spoilers"; icon: page.spoilers ? "eye-off" : "eye"; onClicked: page.spoilers = !page.spoilers }
                            Button { visible: tagFlow.implicitHeight > tagClip.height || page.tagsOpen; small: true; text: page.tagsOpen ? "Less" : "All tags"; onClicked: page.tagsOpen = !page.tagsOpen }
                        }
                    }
                    // A hidden reference chip: chip height depends on font metrics, so the
                    // collapsed clip lands on a whole number of rows by measuring one
                    Chip { id: rowRuler; visible: false; text: "0"; mono: false }
                    Item {
                        id: tagClip
                        width: parent.width
                        readonly property real collapsed: 3 * rowRuler.height + 2 * tagFlow.spacing
                        height: page.tagsOpen ? tagFlow.implicitHeight : Math.min(tagFlow.implicitHeight, collapsed)
                        clip: true
                        Flow {
                            id: tagFlow
                            width: parent.width; spacing: theme.space(1.5)
                            Repeater {
                                model: tagsTile.tags
                                Chip { required property var modelData; text: modelData.name + ", " + modelData.rank + "%"; mono: false; color: theme.raised; textColor: modelData.spoiler || modelData.adult ? theme.yellow : theme.textDim }
                            }
                        }
                    }
                }
            }

            // Relations: the franchise as small stones, in the crawl's own order
            Stone {
                id: relationsTile
                objectName: "series-relations"
                readonly property var nodes: page.graph ? page.graph.nodes.filter(function(n) { return !n.current }).sort(function(a, b) { return a.y - b.y || a.x - b.x }) : []
                visible: nodes.length > 0
                natural: relationsCol.implicitHeight
                stretch: page.sideLast === relationsTile ? page.sideStretch : 0
                Column {
                    id: relationsCol
                    width: parent.width
                    spacing: theme.space(2)
                    Heading { text: "Related"; count: relationsTile.nodes.length }
                    Flow {
                        id: relationFlow
                        width: parent.width
                        spacing: theme.space(2)
                        // As many stones a row as fit at theme.space(64), the row filled
                        readonly property int per: Math.max(1, Math.floor((width + spacing) / (theme.space(64) + spacing)))
                        readonly property real stoneW: (width - (per - 1) * spacing) / per
                        Repeater {
                            model: relationsTile.nodes
                            RelationStone { required property var modelData; node: modelData; width: relationFlow.stoneW }
                        }
                    }
                    Caption { visible: !!page.graph && !page.graph.complete; text: "Still filling in from AniList."; color: theme.textDim }
                }
            }

            Stone {
                id: recommendationsTile
                visible: !!page.detail && page.detail.recommendations.length > 0
                natural: recCol.implicitHeight
                stretch: page.sideLast === recommendationsTile ? page.sideStretch : 0
                Column {
                    id: recCol
                    width: parent.width
                    spacing: theme.space(3)
                    Heading { text: "Recommendations"; count: page.detail ? page.detail.recommendations.length : 0 }
                    Flow { width: parent.width; spacing: theme.space(3)
                        Repeater { model: page.detail ? page.detail.recommendations : []
                            RecommendationCard { required property var modelData; rec: modelData
                                onOpened: modelData.owned ? frame.go("series", { id: modelData.owned }, modelData.title) : Qt.openUrlExternally("https://anilist.co/anime/" + modelData.anilist_id) } } }
                }
            }

            Stone {
                id: charactersTile
                visible: !!page.detail && page.detail.characters.length > 0
                natural: charCol.implicitHeight
                stretch: page.sideLast === charactersTile ? page.sideStretch : 0
                Column {
                    id: charCol
                    width: parent.width
                    spacing: theme.space(3)
                    Heading { text: "Characters"; count: page.detail ? page.detail.characters.length : 0 }
                    Flow { width: parent.width; spacing: theme.space(3); Repeater { model: page.detail ? page.detail.characters : []; PersonCard { required property var modelData; person: modelData } } }
                }
            }
        }
    }
    // The scroll bar rides the right-hand joint, not the tiles' edge
    ThinScrollBar {
        id: scrollBar
        x: page.width - page.grout + Math.max(0, (page.grout - width) / 2)
        y: flick.y
        height: flick.height
        width: Math.min(page.grout, theme.space(1))
    }

    // One related entry: a poster sliver, the title, the relation and year. Owned opens its
    // page; the rest open AniList.
    component RelationStone: Corner {
        id: rel
        property var node: ({})
        readonly property bool owned: node.owned !== null && node.owned !== undefined
        readonly property string relTone: node.poster ? Fmt.posterTone(node.poster) : ""
        height: theme.space(24)
        radius: theme.radiusInner(page.pad > theme.space(2) ? theme.space(2) : page.pad)
        smoothing: theme.cornerSmoothing
        color: relMouse.containsMouse ? theme.raisedHover : theme.raised
        objectName: "relation-" + node.anilist_id
        Corner {
            id: relThumb
            width: height / 1.5; height: parent.height
            radius: parent.radius; smoothing: theme.cornerSmoothing
            color: rel.relTone !== "" ? theme.plate(rel.relTone) : theme.tileSunken
            fillItem: relArt.status === Image.Ready ? relArt : null
            Image { id: relArt; visible: false; width: parent.width; height: parent.height
                source: rel.node.poster ? "file://" + rel.node.poster : ""
                sourceSize: Qt.size(Math.ceil(parent.width * theme.devicePixelRatio), Math.ceil(parent.height * theme.devicePixelRatio))
                fillMode: Image.PreserveAspectCrop; asynchronous: true }
        }
        Column {
            x: relThumb.width + theme.space(3)
            width: parent.width - x - theme.space(3)
            anchors.verticalCenter: parent.verticalCenter
            spacing: theme.space(1)
            Label { width: parent.width; text: rel.node.title || ""; role: "title"; font.pointSize: theme.typeNormal; wrapMode: Text.Wrap; maximumLineCount: 2; elide: Text.ElideRight; color: rel.node.pending ? theme.textDim : theme.text }
            Label { width: parent.width; role: "ledger"; color: theme.textDim; elide: Text.ElideRight
                text: [page.relationLabel(rel.node.relation), page.formatLabel(rel.node.format), rel.node.year ? String(rel.node.year) : ""].filter(function(s) { return s !== "" && s !== "Series" }).join(", ") }
            Row {
                spacing: theme.space(1.5)
                StatusDot { anchors.verticalCenter: parent.verticalCenter; status: rel.node.list_status || "" }
                Label { role: "ledger"; text: rel.owned ? "In your library" : !rel.node.released ? "Not yet released" : "On AniList"; color: rel.owned ? theme.accent : theme.textDim }
            }
        }
        MouseArea {
            id: relMouse
            anchors.fill: parent; hoverEnabled: true; cursorShape: Qt.PointingHandCursor
            onClicked: rel.owned ? frame.go("series", { id: rel.node.owned }, rel.node.title) : (rel.node.site_url ? Qt.openUrlExternally(rel.node.site_url) : Qt.openUrlExternally("https://anilist.co/anime/" + rel.node.anilist_id))
        }
    }

    ScorePicker { id: scorePicker; parent: frame.overlay
        onSaved: function(v) { var r = Door.setScore(page.props.id, v); if (r.error) frame.toast(r.error.message) }
        onCleared: { var r = Door.setScore(page.props.id, -1); if (r.error) frame.toast(r.error.message) } }
}
