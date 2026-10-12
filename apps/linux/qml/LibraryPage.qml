// The library as a mosaic (DESIGN.md, Layout): one module, every element a tile on it, set
// in one constant joint (theme.grout) that is also the page's margin. The header is a row of
// tiles (title, tabs, sorts, group, show watched, search, count) that scrolls with the wall
// as its first unit rows; under it, the sections packed by mosaic.js: each opens with a
// label stone, then its hero tiles (2x2: in progress and watched this fortnight, or airing
// this week), then the small tiles filling in around them. The core searches and sorts; the
// page partitions the sorted list into sections client-side, keeps it in a RecordModel and
// reloads, debounced, when the core says a series changed.
import QtQuick
import QtQuick.Controls.Basic as QC
import QtQuick.Dialogs
import com.marcusrosado.AniBeam
import "mosaic.js" as Mosaic

Item {
    id: page
    objectName: "library-page"   // a --drive target, and the evals below
    property var props: ({})
    property alias scrollY: flick.contentY
    function focusSearch() { flick.contentY = 0; search.focusInput() }
    // "escape" is reserved by the QML compiler; the frame's hook is named escapePressed.
    function escapePressed() { return false }

    readonly property var tabs: ["All", "Series", "Movies"]
    readonly property var sorts: [["Alpha", "A to Z"], ["LastViewed", "Last viewed"], ["Progress", "Progress"], ["CommunityScore", "Average score"], ["MyScore", "My score"]]
    // Loaded-with copies, not bindings on Door.preferences: a plain property bound to
    // prefs.xxx keeps tracking Door.preferences on its own, ahead of the Connections
    // handler below that means to compare "what changed" against it, so the very first
    // preferences-changed event of a fresh instance always sees no difference (the binding
    // already applied the new value before the handler's own body runs). Seeded once in
    // Component.onCompleted instead, they are ordinary state from that point on.
    property string tab: "All"
    property string sort: "Alpha"
    property string direction: "Asc"
    property string titleLanguage: "Romaji"
    property bool showWatched: false
    property string query: props.q || ""
    property bool hiddenExist: false
    property bool watchedExist: false
    property real nowMs: Date.now()
    // The clock the heroes are picked against: set on reload, not ticking with nowMs, so
    // the wall is not repacked (every tile rebuilt) twice a minute
    property real packedAt: Date.now()
    property bool libraryEmpty: false
    property string groupBy: "None"   // None | Genre | Viewed
    // Bumped on every reload: the sections depend on it, so a reload that returns the same
    // number of series (a rename, a progress change) still repaints the wall
    property int revision: 0
    // The airing rows, read once per reload (never inside a binding)
    property var airingRows: []

    RecordModel { id: cards; roles: ["id", "title", "titles", "poster", "code", "watched", "watched_state", "total_episodes", "total_is_estimate", "strip", "community_score", "my_score", "hidden", "next_airing", "last_viewed_at", "episodes_on_disk", "kind", "format", "genres"] }

    function persist() {
        var p = JSON.parse(JSON.stringify(Door.preferences))
        p.library_tab = tab === "Hidden" ? p.library_tab : tab
        p.library_sort = sort; p.library_direction = direction
        Door.setPreferences(p)
    }
    function pickTab(i) { tab = tabNames[i]; persist(); reload() }
    function pickSort(key) { sort = key; direction = key === "Alpha" ? "Asc" : "Desc"; persist(); reload() }
    function flipDirection() { direction = direction === "Asc" ? "Desc" : "Asc"; persist(); reload() }
    // The toggle is a preference, not session state: a non-rewatcher's
    // finished series stays out of the grid across launches. The reload
    // itself comes from the preferences-changed event's handler below.
    function pickShowWatched(on) {
        var p = JSON.parse(JSON.stringify(Door.preferences))
        p.library_show_watched = on
        Door.setPreferences(p)
    }
    // A hero's Play: the series' next episode, read only now (never during layout); a
    // series with nothing to play next opens its page instead
    function playSeries(c) {
        var r = Door.getSeries(c.id)
        var d = !r.error ? r.reply.detail : null
        var file = d ? (d.next_up ? d.next_up : (c.kind === "Movie" && d.episodes && d.episodes.length ? d.episodes[0].file : null)) : null
        if (file) frame.go("player", { file: file }, c.title)
        else frame.go("series", { id: c.id }, c.title)
    }
    readonly property bool showHidden: Door.revealHidden && hiddenExist
    readonly property var tabNames: tabs.concat(watchedExist ? ["Watched"] : []).concat(showHidden ? ["Hidden"] : [])

    // ---- The sections: one entry per section, carrying its cards. Airing first (capped
    // high), then everything, or the grouped shelves by size descending. A series shows
    // once: while Airing stands, its series are left out of the sections after it, and
    // each section's count is the tiles it holds.
    function groupLabel(c) {
        if (groupBy === "Genre") {
            var g = c.genres
            return g && g.length ? g[0] : "Unmatched"
        }
        if (groupBy === "Viewed") {
            var w = c.watched
            if (c.watched_state === "CaughtUp") return "Caught up"
            if (c.watched_state === "Behind") return "Watching"
            return w !== null && w !== undefined && w > 0 ? "Watching" : "Not started"
        }
        return "Everything"
    }
    // A label stone's one line under its name. Airing names the next episode out, with the
    // countdown pill ({ at, text }); the rest say the first of these that tells the section
    // something its name and count do not: how many are in progress, not started, or air
    // this week, that every one is watched, else how many episodes are on disk.
    function soonest(list) {
        var best = null, now = page.packedAt / 1000
        for (var i = 0; i < list.length; i++) {
            var n = list[i].next_airing
            if (n && n.at > now && (!best || n.at < best.at)) best = { at: n.at, text: list[i].title }
        }
        return best
    }
    function factOf(list) {
        var now = page.packedAt / 1000, going = 0, fresh = 0, airs = 0, done = 0, files = 0
        for (var i = 0; i < list.length; i++) {
            var c = list[i], w = c.watched !== null && c.watched !== undefined ? c.watched : 0
            files += c.episodes_on_disk || 0
            if (c.watched_state === "CaughtUp") done++
            else if (w > 0 || c.watched_state === "Behind") going++
            else fresh++
            if (c.next_airing && c.next_airing.at > now && c.next_airing.at - now <= 7 * 86400) airs++
        }
        var n = list.length
        if (going > 0 && going < n) return { text: going + " in progress" }
        if (fresh > 0 && fresh < n) return { text: fresh + " not started" }
        if (airs > 0) return { text: airs + " airing this week" }
        if (n > 0 && done === n) return { text: "All watched" }
        if (files > 0) return { text: Fmt.plural(files, "episode", "episodes") + " on disk" }
        return null
    }
    readonly property bool airingShows: page.query === "" && !page.libraryEmpty
    readonly property var groups: {
        page.revision   // a dependency: see revision
        var airingRows = airingShows ? page.airingRows : []
        var main = page.tab === "All" ? "Everything" : page.tab
        var airingIds = {}
        for (var a = 0; a < airingRows.length; a++) airingIds[airingRows[a].id] = true
        var buckets = {}
        var order = []
        for (var i = 0; i < cards.count; i++) {
            var c = cards.at(i)
            if (airingIds[c.id]) continue
            var label = groupBy === "None" ? main : groupLabel(c)
            if (!buckets[label]) { buckets[label] = []; order.push(label) }
            buckets[label].push(c)
        }
        // Section order: first appearance, but grouped views put the biggest shelf
        // first; ties A to Z.
        if (groupBy !== "None") order.sort(function(a, b) {
            var d = buckets[b].length - buckets[a].length
            return d !== 0 ? d : (a < b ? -1 : a > b ? 1 : 0)
        })
        var out = []
        if (airingShows && airingRows.length) out.push({ label: "Airing", count: airingRows.length, cards: airingRows, fact: soonest(airingRows) })
        for (var s = 0; s < order.length; s++) out.push({ label: order[s], count: buckets[order[s]].length, cards: buckets[order[s]], fact: factOf(buckets[order[s]]) })
        return out
    }

    // ---- The module. Content runs from the rail's own joint on the left to one grout
    // short of the page's right, top and bottom edges.
    readonly property real grout: theme.grout
    readonly property var geo: Mosaic.columns(Math.max(1, width - grout), theme.posterWidth, grout)
    readonly property real unit: geo.u
    readonly property real step: unit + grout
    function px(units) { return Mosaic.unitsToPx(units, unit, grout) }
    function spanX(col) { return geo.xs[Math.min(col, geo.cols - 1)] }
    function spanW(col, span) { var last = Math.min(geo.cols - 1, col + span - 1); return geo.xs[last] + geo.ws[last] - geo.xs[col] }
    // A small tile is the poster plus the smallest plate that holds its lines, rounded up
    // to whole units; a hero is two of them and the joint between, both ways.
    Card { id: probe; visible: false; live: false; posterWidth: page.geo.w }
    readonly property int unitsSmall: Mosaic.unitsFor(probe.posterHeight + probe.minPlate, unit, grout)
    readonly property int unitsHero: unitsSmall * 2

    readonly property var sections: {
        // Candidates only: the packer keeps a series large once on the wall and demotes
        // heroes that would leave a hole, so each section offers every qualifying series
        var now = page.packedAt / 1000
        var out = []
        for (var i = 0; i < groups.length; i++) {
            var g = groups[i]
            out.push({ label: g.label, count: g.count, cards: g.cards, fact: g.fact, heroes: Mosaic.pickHeroes(g.cards, now, g.cards.length, null) })
        }
        return out
    }
    readonly property var packed: Mosaic.pack(sections, geo.cols, unitsSmall, unitsHero)
    readonly property var tiles: packed.tiles

    // For a --drive script's evals
    function packedOverlaps() { return Mosaic.overlaps(tiles) }
    function tileCount() { return tiles.filter(function(t) { return t.kind !== "label" }).length }
    function heroCount() { return tiles.filter(function(t) { return t.kind === "hero" }).length }

    function reload() {
        var keep = flick.contentY
        // One snapshot per reload: the shelf, the airing rail and the per-tab counts
        var r = Door.listLibrary(tab, query, sort, direction, Door.revealHidden)
        if (r.error) { frame.toast(r.error.message); return }
        cards.reset(r.reply.series)
        var n = r.reply.counts || {}
        // The library is empty only when nothing is anywhere: All leaves watched series out
        // unless they are shown, and hidden ones out unless revealed. The Watched tab
        // exists while watched series do, whatever the toggle says.
        watchedExist = (n.watched || 0) > 0
        hiddenExist = Door.revealHidden && (n.hidden || 0) > 0
        libraryEmpty = (n.all || 0) === 0 && !watchedExist && (n.hidden || 0) === 0
        airingRows = query === "" && !libraryEmpty && r.reply.airing ? r.reply.airing : []
        packedAt = Date.now()
        revision++
        flick.contentY = Math.max(0, Math.min(keep, Math.max(0, flick.contentHeight - flick.height)))
    }
    // One reload per burst: a scan reports several changes a second, and restarting the
    // timer on each would hold the wall stale for the whole scan, so the first event of a
    // burst arms it and the rest ride along
    Timer { id: debounce; interval: 400; onTriggered: page.reload() }
    function soon() { if (!debounce.running) debounce.start() }
    Timer { id: queryDebounce; interval: 150; onTriggered: { page.query = search.text; frame.nav.current.props = { q: page.query }; page.reload() } }
    Timer { interval: 30000; running: cards.count > 0; repeat: true; onTriggered: page.nowMs = Date.now() }
    Connections {
        target: Door
        function onSeriesChanged() { page.soon() }
        function onSeriesRemoved() { page.soon() }
        // Door always reports a preference write, even one that changes nothing (the
        // core's own tab/sort/direction clicks already reload once, directly, for instant
        // feedback), so only restart the debounce when something this page cares about
        // actually moved: reloading twice on every click would otherwise double every
        // core round trip for no visible benefit. A title language switch does not touch
        // tab/sort/direction, so it still falls through and reloads, which is what
        // repaints every card's title in the new language.
        function onPreferencesChanged(p) {
            var newTab = page.tab === "Hidden" ? page.tab : p.library_tab
            var changed = newTab !== page.tab || p.library_sort !== page.sort
                || p.library_direction !== page.direction || p.title_language !== page.titleLanguage
                || p.library_show_watched !== page.showWatched
            page.tab = newTab
            page.sort = p.library_sort
            page.direction = p.library_direction
            page.titleLanguage = p.title_language
            page.showWatched = p.library_show_watched === true
            if (changed) page.soon()
        }
        function onRevealHiddenChanged() { if (page.tab === "Hidden" && !Door.revealHidden) page.tab = "All"; page.soon() }
    }
    Component.onCompleted: {
        var p = Door.preferences
        tab = p.library_tab || "All"
        sort = p.library_sort || "Alpha"
        direction = p.library_direction || "Asc"
        titleLanguage = p.title_language || "Romaji"
        showWatched = p.library_show_watched === true
        // search.text starts "" (SearchField's own default), so setting it to a real
        // trail query is a genuine text change and queryDebounce.restart() fires from
        // SearchField's own onTextChanged; stop it right back so this one reload is the
        // only core round trip a fresh navigation costs, not a second one 150 ms later.
        search.text = query
        // The trail can open the page with a grouping already set.
        if (["Genre", "Viewed"].indexOf(props.group) >= 0) groupBy = props.group
        queryDebounce.stop()
        reload()
        if (!theme.reducedMotion && frame.nav.pendingScroll <= 0) sweep.start()
        else { heroIn = 1; shelfIn = 1 }
    }

    // ---- The one orchestrated moment: on open the heroes settle, then the shelf in one
    // sweep. Opacity and a one-grout drop; no per-tile stagger.
    property real heroIn: 0
    property real shelfIn: 0
    SequentialAnimation {
        id: sweep
        NumberAnimation { target: page; property: "heroIn"; to: 1; duration: 220; easing.type: Easing.OutCubic }
        NumberAnimation { target: page; property: "shelfIn"; to: 1; duration: 220; easing.type: Easing.OutCubic }
    }

    // ---- The header: a row of tiles on the module, one unit tall, wrapping to a second
    // row on a narrow window. Each tile's span is the whole columns its content needs; the
    // row's spare columns go to the title (or the search on a wrapped row). A control is
    // centred in its tile both ways, so a whole-column span's slack sits evenly around it.
    // A control says what it is, so no tile carries an eyebrow; only the grouping keeps a
    // word ("Group"), since a bare "None" would not.
    readonly property real hpad: theme.space(3)
    function spanFor(w) { return Math.max(1, Math.ceil((w + hpad * 2 + grout) / (geo.w + grout))) }
    readonly property var headerLayout: Mosaic.header([
        // The title tile carries the count at its right and takes the row's spare columns.
        // tracker-one-click merge: the TrackerNudge tile goes after "search" in this list
        // (shown while no tracker is connected); the title gives up the columns it needs.
        { key: "title", span: Math.max(2, spanFor(titleLabel.implicitWidth + theme.space(6) + countRow.implicitWidth)), flex: true },
        { key: "tabs", span: spanFor(tabsSeg.implicitWidth) },
        { key: "sorts", span: spanFor(sortSeg.implicitWidth) },
        { key: "group", span: spanFor(groupRow.implicitWidth) },
        { key: "watched", span: spanFor(watchedRow.implicitWidth) },
        { key: "search", span: Math.max(2, spanFor(theme.space(56))), flex: true }
    ], geo.cols)
    function headerTile(key) {
        var t = headerLayout.tiles
        for (var i = 0; i < t.length; i++) if (t[i].key === key) return t[i]
        return { col: 0, row: 0, colSpan: 1 }
    }
    readonly property real headerHeight: px(headerLayout.rows)

    component HeaderTile: Corner {
        property string key: ""
        readonly property var slot: page.headerTile(key)
        x: page.spanX(slot.col)
        y: slot.row * page.step
        width: page.spanW(slot.col, slot.colSpan)
        height: page.unit
        radius: theme.radiusTile
        smoothing: theme.cornerSmoothing
        color: theme.tile
    }

    // ---- The wall
    Flickable {
        id: flick
        x: 0
        y: page.grout
        width: page.width - page.grout
        height: page.height - page.grout * 2
        visible: !page.libraryEmpty
        clip: true
        boundsBehavior: Flickable.StopAtBounds
        contentWidth: width
        // The header tiles scroll with the wall: they are its first unit rows
        readonly property real wallTop: page.headerHeight + page.grout
        contentHeight: wallTop + page.px(page.packed.rows)
        QC.ScrollBar.vertical: scrollBar
        WheelStep { flick: flick }

        Item {
            id: header
            width: flick.width
            height: page.headerHeight
            opacity: page.shelfIn

            HeaderTile {
                key: "title"
                Label {
                    id: titleLabel
                    x: page.hpad
                    anchors.verticalCenter: parent.verticalCenter
                    text: "Library"
                    role: "display"
                }
                Row {
                    id: countRow
                    anchors.right: parent.right; anchors.rightMargin: page.hpad
                    anchors.baseline: titleLabel.baseline
                    spacing: theme.space(1.5)
                    Label {
                        id: countFigure
                        text: cards.count
                        role: "ledger"
                        font.pointSize: theme.typeDisplay
                        font.weight: 700
                        font.variableAxes: ({ "wdth": 88, "wght": 700 })
                    }
                    Label {
                        anchors.baseline: countFigure.baseline
                        text: "series"
                        role: "ui"
                        font.pointSize: theme.typeSmall
                        color: theme.textDim
                    }
                }
            }
            HeaderTile {
                key: "tabs"
                Seg {
                    id: tabsSeg
                    objectName: "tab"
                    x: Math.max(page.hpad, (parent.width - width) / 2)
                    anchors.verticalCenter: parent.verticalCenter
                    options: page.tabNames
                    index: Math.max(0, page.tabNames.indexOf(page.tab))
                    onPicked: function(i) { page.pickTab(i) }
                }
            }
            HeaderTile {
                key: "sorts"
                // The active sort carries its direction as a chevron; picking it again flips it
                Seg {
                    id: sortSeg
                    objectName: "sort"
                    x: Math.max(page.hpad, (parent.width - width) / 2)
                    anchors.verticalCenter: parent.verticalCenter
                    options: page.sorts.map(function(s) {
                        return { text: s[1], name: s[0], icon: s[0] === page.sort ? (page.direction === "Asc" ? "chevron-up" : "chevron-down") : "" }
                    })
                    index: Math.max(0, page.sorts.findIndex(function(s) { return s[0] === page.sort }))
                    onPicked: function(i) { if (i === index) page.flipDirection(); else page.pickSort(page.sorts[i][0]) }
                }
            }
            HeaderTile {
                key: "group"
                Row {
                    id: groupRow
                    x: Math.max(page.hpad, (parent.width - width) / 2)
                    anchors.verticalCenter: parent.verticalCenter
                    spacing: theme.space(2.5)
                    Label {
                        anchors.verticalCenter: parent.verticalCenter
                        text: "Group"
                        role: "ui"
                        color: theme.textDim
                    }
                    Seg {
                        id: groupSeg
                        objectName: "group"
                        options: ["None", "Genre", "Watch state"]
                        index: ["None", "Genre", "Viewed"].indexOf(page.groupBy)
                        onPicked: function(i) { page.groupBy = ["None", "Genre", "Viewed"][i] }
                    }
                }
            }
            HeaderTile {
                key: "watched"
                id: watchedTile
                color: watchedArea.containsMouse ? theme.tileHover : theme.tile
                // The whole tile toggles; the switch on it takes its own clicks the same way
                MouseArea {
                    id: watchedArea
                    anchors.fill: parent
                    hoverEnabled: true
                    cursorShape: Qt.PointingHandCursor
                    onClicked: page.pickShowWatched(!page.showWatched)
                }
                Row {
                    id: watchedRow
                    x: Math.max(page.hpad, (parent.width - width) / 2)
                    anchors.verticalCenter: parent.verticalCenter
                    spacing: theme.space(2.5)
                    Label {
                        anchors.verticalCenter: parent.verticalCenter
                        text: "Show watched"
                        role: "ui"
                    }
                    Switch {
                        id: watchedSwitch
                        objectName: "show-watched"
                        anchors.verticalCenter: parent.verticalCenter
                        checked: page.showWatched
                        onToggled: function(on) { page.pickShowWatched(on); checked = Qt.binding(function() { return page.showWatched }) }
                    }
                }
            }
            HeaderTile {
                key: "search"
                SearchField {
                    id: search
                    objectName: "library-search"
                    x: page.hpad
                    anchors.verticalCenter: parent.verticalCenter
                    width: parent.width - page.hpad * 2
                    placeholder: "Search titles or folders"
                    onTextChanged: queryDebounce.restart()
                    onCleared: { queryDebounce.stop(); page.query = ""; frame.nav.current.props = {}; page.reload() }
                }
            }
        }

        // The live window: tiles within a screen above or below the viewport carry their
        // posters. The tiles are sorted by row, so the window is two binary searches here
        // and an integer compare per tile.
        readonly property int firstLive: {
            var t = page.tiles, lo = 0, hi = t.length
            var lim = (contentY - wallTop - height - page.px(page.unitsHero)) / page.step
            while (lo < hi) { var m = (lo + hi) >> 1; if (t[m].row < lim) lo = m + 1; else hi = m }
            return lo
        }
        readonly property int lastLive: {
            var t = page.tiles, lo = 0, hi = t.length
            var lim = (contentY - wallTop + height * 2) / page.step
            while (lo < hi) { var m = (lo + hi) >> 1; if (t[m].row <= lim) lo = m + 1; else hi = m }
            return lo - 1
        }

        Repeater {
            model: page.tiles
            delegate: Item {
                id: cell
                required property var modelData
                required property int index
                readonly property bool stone: modelData.kind === "label"
                readonly property bool hero: modelData.kind === "hero"
                readonly property var section: page.sections[modelData.section]
                readonly property real settle: hero ? page.heroIn : page.shelfIn
                // Built only within a screen of the viewport: the rest of the wall is
                // empty positioned Items
                readonly property bool live: index >= flick.firstLive && index <= flick.lastLive
                // Wholly inside the viewport: only such a tile answers to the generic
                // "small-tile" / "hero-tile" names a --drive script clicks
                readonly property bool inView: y >= flick.contentY && y + height <= flick.contentY + flick.height
                objectName: stone ? "stone-" + section.label.toLowerCase() : "tile-" + modelData.card.id
                x: page.spanX(modelData.col)
                y: flick.wallTop + modelData.row * page.step + (1 - settle) * page.grout
                width: page.spanW(modelData.col, modelData.colSpan)
                height: page.px(modelData.rowSpan)
                opacity: settle
                Loader {
                    anchors.fill: parent
                    active: cell.live
                    sourceComponent: cell.stone ? labelStone : seriesTile
                }
                Component {
                    id: seriesTile
                    Card {
                        objectName: cell.inView ? (cell.hero ? "hero-tile" : "small-tile") : ""
                        item: cell.modelData.card
                        hero: cell.hero
                        posterWidth: cell.width
                        tileHeight: cell.height
                        live: true   // the cell itself is built only inside the live window
                        nowMs: page.nowMs
                        onOpened: frame.go("series", { id: cell.modelData.card.id }, cell.modelData.card.title)
                        onPlayed: page.playSeries(cell.modelData.card)
                    }
                }
                // A section's first cell: its name and count, cut as a stone. Airing is set
                // in the accent's own plate, the rest in raised slate.
                Component {
                    id: labelStone
                    Corner {
                        readonly property bool airing: cell.section.label === "Airing"
                        radius: theme.radiusTile
                        smoothing: theme.cornerSmoothing
                        color: airing ? theme.plate(theme.accent) : theme.raised
                        Column {
                            id: stoneCol
                            x: theme.space(4); y: theme.space(4)
                            width: parent.width - theme.space(8)
                            spacing: theme.space(2)
                            Label {
                                id: stoneName
                                width: parent.width
                                text: cell.section.label
                                role: "title"
                                wrapMode: Text.Wrap
                                maximumLineCount: 3
                                elide: Text.ElideRight
                            }
                            StoneFact {
                                width: parent.width
                                // gives way rather than run into the count on a short stone
                                shown: stoneCol.y + stoneName.height + stoneCol.spacing + implicitHeight <= stoneCount.y - theme.space(1)
                                fact: cell.section.fact || null
                                nowMs: page.nowMs
                            }
                        }
                        Label {
                            id: stoneCount
                            anchors.right: parent.right; anchors.rightMargin: theme.space(4)
                            anchors.bottom: parent.bottom; anchors.bottomMargin: theme.space(3)
                            text: cell.section.count
                            role: "ledger"
                            font.pointSize: theme.typeDisplay * 1.6
                            font.weight: 700
                            font.variableAxes: ({ "wdth": 88, "wght": 700 })
                            color: parent.airing ? theme.text : theme.textDim
                        }
                    }
                }
            }
        }

        // Nothing in this shelf: under whatever the wall did place (the Airing section can
        // still stand over an empty tab)
        Item {
            y: flick.contentHeight
            width: flick.width
            height: Math.max(theme.space(60), flick.height - y)
            visible: cards.count === 0 && !page.libraryEmpty
            EmptyState {
                icon: "search"
                title: page.query !== "" ? "No matches for \"" + page.query + "\"" : "Nothing here"
                body: page.query !== "" ? "" : "No " + (page.tab === "Series" ? "series" : page.tab === "Movies" ? "films" : page.tab === "Watched" ? "watched items" : "items") + " in your library yet."
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

    // The empty home: Import, and a pointer at Settings
    EmptyState {
        visible: page.libraryEmpty
        icon: "tv"
        title: "Your library is empty"
        body: "Add a folder in Settings, or import an AniBeam export."
        Button { text: "Import"; icon: "download"; onClicked: importDialog.open() }
        Button { text: "Settings"; icon: "settings"; flat: true; onClicked: frame.go("settings") }
    }
    FileDialog {
        id: importDialog
        title: "Import an AniBeam export"
        nameFilters: ["AniBeam export (*.json)", "All files (*)"]
        onAccepted: { var r = Door.importLibrary(decodeURIComponent(String(selectedFile).replace("file://", ""))); if (r.error) frame.toast(r.error.message); else frame.toast("Import started") }
    }
}
