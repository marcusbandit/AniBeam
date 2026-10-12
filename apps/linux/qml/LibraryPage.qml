// The library as a mosaic (DESIGN.md, Layout): one module, every element a tile on it, set
// in one constant joint (theme.grout) that is also the page's margin. The header is a row of
// tiles (title, tabs, sorts, arrange, show watched, search, count) that scrolls with the
// wall as its first unit rows; under it, the arranged wall packed by mosaic.js: the Airing
// this week box, then the groups of the chosen arrangement (Related, Type, Genre, Watch
// state) as framed boxes, each opened by its label stone with its members packed inside,
// and lone series as plain tiles filling in around them. Heroes (2x2: in progress and
// watched this fortnight, or airing this week) stand one per box. The core searches and
// sorts; the page arranges the sorted list client-side, keeps it in a RecordModel and
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
    // Related | Type | Genre | WatchState | None (Door.preferences.library_arrange)
    property string arrange: "Related"
    readonly property var arrangements: [["Related", "Related"], ["Type", "Type"], ["Genre", "Genre"], ["WatchState", "Watch state"], ["None", "None"]]
    // Bumped on every reload: the groups depend on it, so a reload that returns the same
    // number of series (a rename, a progress change) still repaints the wall
    property int revision: 0
    // The airing rows, read once per reload (never inside a binding)
    property var airingRows: []

    RecordModel { id: cards; roles: ["id", "title", "titles", "poster", "code", "watched", "watched_state", "total_episodes", "total_is_estimate", "strip", "community_score", "my_score", "hidden", "next_airing", "last_viewed_at", "episodes_on_disk", "kind", "format", "genres", "franchise"] }

    function persist() {
        var p = JSON.parse(JSON.stringify(Door.preferences))
        p.library_tab = tab === "Hidden" ? p.library_tab : tab
        p.library_sort = sort; p.library_direction = direction
        p.library_arrange = arrange
        Door.setPreferences(p)
    }
    // Arranging is the page's own work on the list it has: no reload, only a repack
    function pickArrange(key) { arrange = key; persist() }
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

    // ---- The arrangement: one entry per box or lone tile, in reading order. Airing this
    // week first (a box in every arrangement; its series also stand in their own groups, the
    // one exception to a series showing once, like a "now" shelf), then the groups. Related
    // groups by the core's franchise tag (a group of one is a plain tile); Type, Genre and
    // Watch state box every group, since the label is the point. Members keep the sort's
    // order (Related: release order when the cards carry a year), and the boxes go in the
    // order of their best member.
    function typeOf(c) {
        var f = c.format || ""
        if (f === "TV" || f === "TV_SHORT") return "TV"
        if (f === "MOVIE" || (f === "" && c.kind === "Movie")) return "Movie"
        if (f === "OVA" || f === "ONA" || f === "SPECIAL") return "OVA, ONA and specials"
        if (f === "MUSIC") return "Music"
        return "Unknown"
    }
    function stateOf(c) {
        var w = c.watched
        if (c.watched_state === "CaughtUp") return "Caught up"
        if (c.watched_state === "Behind") return "Watching"
        return w !== null && w !== undefined && w > 0 ? "Watching" : "Not started"
    }
    // A group's key and title, or null for a series that stands alone
    function groupOf(c) {
        if (arrange === "Related") return c.franchise ? { key: "f" + c.franchise.id, title: c.franchise.title } : null
        if (arrange === "Type") { var t = typeOf(c); return { key: t, title: t } }
        if (arrange === "Genre") { var g = c.genres && c.genres.length ? c.genres[0] : "Unmatched"; return { key: g, title: g } }
        if (arrange === "WatchState") { var st = stateOf(c); return { key: st, title: st } }
        return null
    }
    // Release order, when the card carries one (a year or a start date); else null
    function releaseOf(c) {
        if (c.start_date) return String(c.start_date)
        if (c.year) return String(c.year)
        if (c.season_year) return String(c.season_year)
        return null
    }
    // A label stone's one line under its name. Airing names the next episode out, with the
    // countdown pill ({ at, text }); the rest say the first of these that tells the group
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
        var buckets = {}, order = [], loose = 0
        for (var i = 0; i < cards.count; i++) {
            var c = cards.at(i)
            var g = groupOf(c)
            var key = g ? g.key : "\u0000" + (loose++)
            if (!buckets[key]) { buckets[key] = { title: g ? g.title : "", boxed: !!g && arrange !== "Related", cards: [] }; order.push(key) }
            buckets[key].cards.push(c)
        }
        var out = []
        if (airingRows.length) out.push({ title: "Airing this week", airing: true, box: true, cards: airingRows, fact: soonest(airingRows) })
        for (var s = 0; s < order.length; s++) {
            var b = buckets[order[s]]
            var list = b.cards
            if (arrange === "Related" && list.length > 1 && list.every(function(x) { return page.releaseOf(x) !== null })) {
                var ranked = list.map(function(x, k) { return { c: x, k: k, r: page.releaseOf(x) } })
                ranked.sort(function(p, q) { return p.r < q.r ? -1 : p.r > q.r ? 1 : p.k - q.k })
                list = ranked.map(function(x) { return x.c })
            }
            out.push({ title: b.title, airing: false, box: b.boxed || list.length > 1, cards: list, fact: list.length > 1 || b.boxed ? factOf(list) : null })
        }
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

    readonly property var items: {
        // Hero candidates: one per box (none in Airing, whose tiles stay small), and every
        // qualifying lone series, ranked; the packer caps the lone ones and keeps a series
        // large once on the wall
        var now = page.packedAt / 1000
        var taken = {}, out = []
        for (var i = 0; i < groups.length; i++) {
            var g = groups[i]
            var hero = null, rank = 0
            if (!g.airing) {
                var picked = Mosaic.pickHeroes(g.cards, now, 1, taken)
                if (picked.length) { hero = picked[0]; rank = Mosaic.heroRank(hero, now) }
            }
            out.push({ title: g.title, airing: g.airing, box: g.box, cards: g.cards, fact: g.fact, hero: hero, heroRank: rank })
        }
        return out
    }
    readonly property var packed: Mosaic.packGroups(items, geo.cols, unitsSmall, { slotW: geo.w, slotH: px(unitsSmall), grout: grout })
    readonly property var tiles: packed.tiles

    // For a --drive script's evals
    function packedOverlaps() { return Mosaic.overlaps(tiles.filter(function(t) { return t.kind !== "frame" })) || Mosaic.overlaps(tiles.filter(function(t) { return t.kind === "frame" })) }
    function packedHoles() { return Mosaic.holes(tiles.filter(function(t) { return t.kind !== "frame" }), geo.cols, 0, unitsSmall) }
    // series tiles, the Airing box's repeats left out
    function tileCount() { return tiles.filter(function(t) { return t.card && !t.airing }).length }
    function heroCount() { return tiles.filter(function(t) { return t.kind === "hero" }).length }
    // the boxes on the wall: [{ title, airing, ids }]
    function boxes() {
        var out = []
        for (var i = 0; i < tiles.length; i++) {
            var t = tiles[i]
            if (t.kind !== "frame") continue
            var it = items[t.item]
            out.push({ title: it.title, airing: it.airing, ids: it.cards.map(function(c) { return c.id }) })
        }
        return out
    }
    // the box a series stands in outside Airing (its title), or "" when it stands alone
    function boxOf(id) {
        var b = boxes()
        for (var i = 0; i < b.length; i++) if (!b[i].airing && b[i].ids.indexOf(id) >= 0) return b[i].title
        return ""
    }
    // ids in the order the core sorted them, for the evals
    function cardIds() { var out = []; for (var i = 0; i < cards.count; i++) out.push(cards.at(i).id); return out }
    function seriesWhere(text) { var out = []; for (var i = 0; i < cards.count; i++) { var c = cards.at(i); if (String(c.title).toLowerCase().indexOf(text) >= 0) out.push({ id: c.id, title: c.title, format: c.format, franchise: c.franchise === undefined ? "undefined" : c.franchise }) } return out }

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
            // arranging repacks on its own, no reload
            if (p.library_arrange !== undefined) page.arrange = p.library_arrange
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
        arrange = p.library_arrange || "Related"
        // search.text starts "" (SearchField's own default), so setting it to a real
        // trail query is a genuine text change and queryDebounce.restart() fires from
        // SearchField's own onTextChanged; stop it right back so this one reload is the
        // only core round trip a fresh navigation costs, not a second one 150 ms later.
        search.text = query
        // The trail can open the page with an arrangement already set (and the old
        // grouping's names).
        var asked = props.arrange || (props.group === "Viewed" ? "WatchState" : props.group)
        if (arrangements.some(function(a) { return a[0] === asked })) arrange = asked
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
    // A control says what it is, so no tile carries an eyebrow; only the arrangement keeps
    // a word ("Arrange"), since a bare "None" would not.
    readonly property real hpad: theme.space(3)
    function spanFor(w) { return Math.max(1, Math.ceil((w + hpad * 2 + grout) / (geo.w + grout))) }
    readonly property var headerLayout: Mosaic.header([
        // The title tile carries the count at its right and takes the row's spare columns.
        { key: "title", span: Math.max(2, spanFor(titleLabel.implicitWidth + theme.space(6) + countRow.implicitWidth)), flex: true },
        { key: "tabs", span: spanFor(tabsSeg.implicitWidth) },
        { key: "sorts", span: spanFor(sortSeg.implicitWidth) },
        { key: "arrange", span: spanFor(arrangeRow.implicitWidth) },
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
        // The header tiles scroll with the wall: they are its first unit rows, and the
        // tracker nudge, while it stands, the full-width row under them
        readonly property real wallTop: page.headerHeight + page.grout + (nudge.visible ? nudge.height + page.grout : 0)
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
                key: "arrange"
                Row {
                    id: arrangeRow
                    x: Math.max(page.hpad, (parent.width - width) / 2)
                    anchors.verticalCenter: parent.verticalCenter
                    spacing: theme.space(2.5)
                    Label {
                        anchors.verticalCenter: parent.verticalCenter
                        text: "Arrange"
                        role: "ui"
                        color: theme.textDim
                    }
                    Seg {
                        id: arrangeSeg
                        objectName: "arrange"
                        options: page.arrangements.map(function(a) { return { text: a[1], name: a[0] } })
                        index: Math.max(0, page.arrangements.findIndex(function(a) { return a[0] === page.arrange }))
                        onPicked: function(i) { page.pickArrange(page.arrangements[i][0]) }
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
                readonly property bool frameKind: modelData.kind === "frame"
                readonly property bool stone: modelData.kind === "label"
                readonly property bool hero: modelData.kind === "hero"
                readonly property var group: page.items[modelData.item]
                readonly property real settle: hero ? page.heroIn : page.shelfIn
                // Built only within a screen of the viewport: the rest of the wall is
                // empty positioned Items. A frame is a bare outline, always drawn, since a
                // tall box can start well above the window.
                readonly property bool live: index >= flick.firstLive && index <= flick.lastLive
                // Wholly inside the viewport: only such a tile answers to the generic
                // "small-tile" / "hero-tile" names a --drive script clicks
                readonly property bool inView: y >= flick.contentY && y + height <= flick.contentY + flick.height
                readonly property string slug: group && group.title ? String(group.title).toLowerCase().replace(/[^a-z0-9]+/g, "-") : ""
                objectName: frameKind ? "box-" + slug : stone ? "stone-" + slug : (modelData.airing ? "airing-" : "tile-") + modelData.card.id
                x: page.spanX(modelData.col)
                y: flick.wallTop + modelData.row * page.step + (1 - settle) * page.grout
                width: page.spanW(modelData.col, modelData.colSpan)
                height: page.px(modelData.rowSpan)
                // a frame's edge lies over its members: the box is drawn last
                z: frameKind ? 1 : 0
                opacity: settle
                // A box's frame: a line straddling the box's edge, one pixel in the joint
                // and one over its members' outer pixel, so it reads against posters and
                // against the ground alike; the members sit inside on the module and the
                // label stone opens it. (A glass-alpha hairline vanished over posters.)
                Rectangle {
                    visible: cell.frameKind
                    x: -1; y: -1
                    width: parent.width + 2; height: parent.height + 2
                    radius: theme.radiusTile
                    color: "transparent"
                    border.width: 2
                    border.color: theme.textFaint
                }
                Loader {
                    anchors.fill: parent
                    active: cell.live && !cell.frameKind
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
                // A box's first cell, its header: the group's name, its one fact line and
                // its count, on the box's glass. It spans the box's slack columns, so a wide
                // one is the box's title strip.
                Component {
                    id: labelStone
                    Tile {
                        raised: true
                        padding: 0
                        Column {
                            id: stoneCol
                            x: theme.space(4); y: theme.space(4)
                            width: parent.width - theme.space(8)
                            spacing: theme.space(2)
                            Label {
                                id: stoneName
                                width: parent.width
                                text: cell.group.title
                                role: "title"
                                wrapMode: Text.Wrap
                                maximumLineCount: 3
                                elide: Text.ElideRight
                            }
                            StoneFact {
                                width: parent.width
                                // gives way rather than run into the count on a short stone
                                shown: stoneCol.y + stoneName.height + stoneCol.spacing + implicitHeight <= stoneCount.y - theme.space(1)
                                fact: cell.group.fact || null
                                nowMs: page.nowMs
                            }
                        }
                        Label {
                            id: stoneCount
                            anchors.right: parent.right; anchors.rightMargin: theme.space(4)
                            anchors.bottom: parent.bottom; anchors.bottomMargin: theme.space(3)
                            text: cell.group.cards.length
                            role: "ledger"
                            font.pointSize: theme.typeDisplay * 1.6
                            font.weight: 700
                            font.variableAxes: ({ "wdth": 88, "wght": 700 })
                            color: theme.textDim
                        }
                    }
                }
            }
        }

        // Nothing in this shelf: under whatever the wall did place (the Airing box can
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
    // ---- The tracker nudge: first run only, until a tracker is connected or it is put off.
    // On the wall it is a full-width stone under the header, cut to whole unit rows and
    // scrolling with it; over the empty library, where there is no wall, it is the same
    // stone at the top of the page, since that is exactly when a new user is here. One
    // instance either way, reparented, so its dismissal is one setting read once.
    TrackerNudge {
        id: nudge
        parent: page.libraryEmpty ? page : flick.contentItem
        x: page.spanX(0)
        y: page.libraryEmpty ? page.grout : page.headerHeight + page.grout
        width: page.spanW(0, page.geo.cols)
        height: page.px(Math.max(1, Math.ceil((implicitHeight + page.grout) / page.step)))
        opacity: page.libraryEmpty ? 1 : page.shelfIn
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
