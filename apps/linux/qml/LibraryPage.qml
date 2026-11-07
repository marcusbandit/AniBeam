// The library: a title with its count, the search, one row of controls (the tabs at the
// left; at the right a Sort menu, its direction in words, a Group menu and two toggle chips
// for franchises and watched shows, #325), then the series in sections. A section is a
// heading in the display cut with one fact and a count, over a grid of uniform cards whose
// column count comes from the width. Group franchises (a switch, on by default) pulls every
// franchise with two or more series in the library out as a section of its own, ahead of
// the cut; the cut is Season (newest first), Type, Genre, Watch state, or None for one
// flat grid. Airing is its own section before all of that: a series airing now is there
// and also under its franchise or its cut, except in the current season's section, which
// holds only what is done airing. The core searches and orders; the page cuts the sections client-side and
// reloads, debounced, when the core says a series changed. The visible words are Sort by
// (the core's ordering, `sort` here) and Group by (the cut, `arrange` here); the names stay
// for the preference keys and the drive scripts.
//
// Several at once (#208): Ctrl+click or a held press picks a card, Shift+click picks every card
// from the last pick to this one, Ctrl+Space and Shift+Space do the same from the keyboard and
// Ctrl+A picks every card shown. While anything is picked a bar at the foot counts the picks
// and acts on all of them in one call (ActOnSeries): Hide, Rate, a list, Rescan and Find a
// match. The core writes one line to the activity log for the lot; Escape or Cancel stops.
import QtQuick
import QtQuick.Controls.Basic as QC
import QtQuick.Dialogs
import com.marcusrosado.AniBeam

Item {
    id: page
    objectName: "library-page"   // a --drive target, and the evals below
    property var props: ({})
    property alias scrollY: scroll.contentY
    function focusSearch() { scroll.contentY = 0; search.focusInput() }
    // "escape" is reserved by the QML compiler; the frame's hook is named escapePressed.
    function escapePressed() {
        if (page.selecting) { page.clearPicks(); return true }
        return false
    }
    // The arrows with nothing focused go to the scroll's grid keys (Down lands on the first card).
    // Ctrl+A picks every card shown; the search field keeps it for its own text.
    Keys.onPressed: function(event) {
        if (event.key === Qt.Key_A && (event.modifiers & Qt.ControlModifier) && !search.editing && page.view.count > 0) {
            page.pickAll(); event.accepted = true; return
        }
        scroll.keyPressed(event)
    }

    // ---- Picking several (#208): the picks by series id, and the one a Shift range runs from
    property var picked: ({})
    property int anchorId: -1
    readonly property int pickedCount: Object.keys(picked).length
    readonly property bool selecting: pickedCount > 0
    // Every card in the order the page shows them, each once: a series airing now sits in
    // the Airing section and again under its franchise or cut, and a range counts it once
    function shownIds() {
        var seen = {}, out = []
        for (var i = 0; i < sections.length; i++) {
            var ids = sections[i].ids
            for (var j = 0; j < ids.length; j++) if (!seen[ids[j]]) { seen[ids[j]] = true; out.push(ids[j]) }
        }
        return out
    }
    function togglePick(id) {
        var next = Object.assign({}, page.picked)
        if (next[id]) delete next[id]
        else next[id] = true
        page.picked = next
        page.anchorId = id
    }
    function pickRange(id) {
        var ids = page.shownIds()
        var from = page.selecting ? ids.indexOf(page.anchorId) : -1, to = ids.indexOf(id)
        if (from < 0 || to < 0) { page.togglePick(id); return }
        var next = Object.assign({}, page.picked)
        for (var i = Math.min(from, to); i <= Math.max(from, to); i++) next[ids[i]] = true
        page.picked = next
    }
    function pickAll() {
        var next = {}
        page.shownIds().forEach(function(id) { next[id] = true })
        page.picked = next
    }
    function clearPicks() { page.picked = ({}); page.anchorId = -1 }
    function pickedIds() { return Object.keys(page.picked).map(Number) }
    // A pick that left the view (a search, a tab, a hide) is no longer picked
    function keepShownPicks() {
        if (!page.selecting) return
        var shown = {}
        page.shownIds().forEach(function(id) { shown[id] = true })
        var next = {}, moved = false
        for (var k in page.picked) { if (shown[k]) next[k] = true; else moved = true }
        if (moved) page.picked = next
    }
    readonly property var pickedCards: { var out = []; for (var k in picked) out.push(Door.card(Number(k))); return out }
    // Rate and a list both need a pick a connected tracker carries
    readonly property bool pickedTracked: pickedCards.some(function(c) { return frame.canSetListStatus(c) })
    readonly property bool pickedUnmatched: pickedCards.some(function(c) { return !c.match_info })
    function pickedNoun(n) { return Fmt.plural(n, "show", "shows") }
    // One call for every pick. A refusal (nothing reachable, a folder that is away) is the toast
    function act(action, said) {
        var ids = page.pickedIds()
        var r = Door.call("ActOnSeries", { series: ids, action: action })
        if (r.error) { frame.fail(r.error); return null }
        page.clearPicks()
        if (said) frame.toast(said)
        return ids
    }
    function hidePicked() {
        var hide = page.tab !== "Hidden"
        var n = page.pickedCount
        var tracked = page.pickedCards.some(function(c) { return !!(c.match_info && (c.match_info.anilist_id || c.match_info.mal_id)) })
        var ids = page.act({ Hide: { hidden: hide } }, "")
        if (!ids) return
        frame.toast(hide ? (page.pickedNoun(n) + " hidden" + (tracked ? ", and their tracker sync stops" : ""))
                         : (page.pickedNoun(n) + " back, and syncing again"),
                    6, "Undo", function() {
                        var u = Door.call("ActOnSeries", { series: ids, action: { Hide: { hidden: !hide } } })
                        if (u.error) frame.fail(u.error)
                    })
    }
    function ratePicked(score) {
        var n = page.pickedCount
        page.act({ Score: { score: score } }, score === null ? "Clearing the score of " + page.pickedNoun(n)
                                                             : "Rating " + page.pickedNoun(n) + " " + Fmt.score(score))
    }
    function listPicked(anchor) {
        var lists = frame.listStatusItems(-1, "")
        // Above the bar, clear of it: the menu's rows plus its own padding
        var p = anchor.mapToItem(frame, 0, 0)
        frame.openMenu(p.x, p.y - lists.length * theme.controlHeight - theme.space(8), lists.map(function(l) {
            return { text: l.text, icon: l.icon, action: function() {
                var n = page.pickedCount
                page.act({ ListStatus: { status: l.status } }, "Moving " + page.pickedNoun(n) + " to " + l.text)
            } }
        }))
    }
    function rescanPicked() { var n = page.pickedCount; page.act("Rescan", "Rescanning " + page.pickedNoun(n)) }
    function matchPicked() {
        var n = page.pickedCards.filter(function(c) { return !c.match_info }).length
        page.act("FindMatch", "Looking for a match for " + page.pickedNoun(n))
    }

    readonly property var tabs: ["All", "Series", "Movies"]
    readonly property var sorts: [["LastViewed", "Recently watched"], ["Downloaded", "Recently downloaded"], ["Alpha", "Title"], ["Progress", "Progress"], ["CommunityScore", "Average score"], ["MyScore", "My score"]]
    // The direction in the words of the order it flips: a title runs A to Z, a date newest
    // first, a score highest first
    function directionWords(key, dir) {
        var asc = dir === "Asc"
        if (key === "Alpha") return asc ? "A to Z" : "Z to A"
        if (key === "LastViewed" || key === "Downloaded") return asc ? "Oldest first" : "Newest first"
        if (key === "Progress") return asc ? "Least watched first" : "Most watched first"
        return asc ? "Lowest first" : "Highest first"
    }
    // The cuts. A stored "Related" (the cut before the franchise toggle) reads as Season
    // with the toggle on, which is what it was.
    readonly property var arrangements: [["None", "Nothing"], ["Season", "Season"], ["Type", "Type"], ["Genre", "Genre"], ["WatchState", "Watch state"]]
    function cutOf(key) { return key === "Related" ? "Season" : key }
    // Loaded-with copies, not bindings on Door.preferences: seeded once in
    // Component.onCompleted, ordinary state from then on, so the preferences-changed
    // handler below can compare what moved.
    property string tab: "All"
    property string sort: "Alpha"
    property string direction: "Asc"
    property string titleLanguage: "Romaji"
    property bool showWatched: false
    property string arrange: "None"
    // Franchises as sections of their own ahead of the cut, or every series under its own
    // season, type or genre: a person who wants to know when things came out turns it off
    property bool groupFranchises: true
    property string query: props.q || ""
    // A genre picked on Home: the grid holds that genre's series only, until cleared
    property string genre: props.genre || ""
    property bool hiddenExist: false
    property bool watchedExist: false
    property bool libraryEmpty: false
    // An empty library with folders is a scan that found nothing, not a library to start
    property bool hasFolders: false
    // A read that failed says so with Try again, never as an empty library
    property bool loadFailed: false
    function showAll() { tab = "All"; persist(); reload() }
    function scanAll() { var r = Door.scan(-1); r.error ? frame.fail(r.error, page.scanAll) : frame.toast("Scan started") }
    property real nowMs: Date.now()

    function clearGenre() { genre = ""; frame.nav.current.props = { q: page.query }; frame.nav.relabel("Library"); reload() }
    function persist() {
        var p = JSON.parse(JSON.stringify(Door.preferences))
        p.library_tab = tab === "Hidden" ? p.library_tab : tab
        p.library_sort = sort; p.library_direction = direction
        p.library_arrange = arrange
        Door.setPreferences(p)
    }
    function pickArrange(key) { arrange = key; persist() }
    // Everything in one list, A to Z: no airing, franchise or dropped sections. No button
    // since #325 (Sort Title, Group Nothing says it); the drive scripts call it.
    function plain() { sort = "Alpha"; direction = "Asc"; arrange = "None"; persist(); reload() }
    function pickGroupFranchises(on) {
        var p = JSON.parse(JSON.stringify(Door.preferences))
        p.library_group_franchises = on
        Door.setPreferences(p)
    }
    function pickTab(i) { tab = tabNames[i]; clearPicks(); persist(); reload() }
    function pickSort(key) { sort = key; direction = key === "Alpha" ? "Asc" : "Desc"; persist(); reload() }
    function flipDirection() { direction = direction === "Asc" ? "Desc" : "Asc"; persist(); reload() }
    function pickShowWatched(on) {
        var p = JSON.parse(JSON.stringify(Door.preferences))
        p.library_show_watched = on
        Door.setPreferences(p)
    }
    function playSeries(c) {
        var r = Door.getSeries(c.id)
        var d = !r.error ? r.reply.detail : null
        var file = d ? (d.next_up ? d.next_up : (c.kind === "Movie" && d.episodes && d.episodes.length && (d.episodes[0].downloading === undefined || d.episodes[0].downloading === null) ? d.episodes[0].file : null)) : null
        if (file) frame.go("player", { file: file }, c.title)
        else frame.go("series", { id: c.id }, c.title)
    }
    readonly property bool showHidden: Door.revealHidden && hiddenExist
    readonly property var tabNames: tabs.concat(watchedExist ? ["Watched"] : []).concat(showHidden ? ["Hidden"] : [])
    // How many each tab holds, from the last read
    property var tabCounts: ({})
    readonly property var tabOptions: tabNames.map(function(t) {
        var k = { All: "all", Series: "series", Movies: "movies", Watched: "watched", Hidden: "hidden" }[t]
        return { text: t, name: t.toLowerCase(), count: page.tabCounts[k] }
    })
    // What the count under the title counts, by tab
    function countLine(n) {
        if (tab === "Movies") return Fmt.plural(n, "movie", "movies")
        if (tab === "Watched") return n + " watched"
        if (tab === "Hidden") return n + " hidden"
        return Fmt.plural(n, "series", "series")
    }

    // The season now, which the Season cut (library_view.rs) keeps airing series out of
    readonly property var seasonWords: ["Winter", "Spring", "Summer", "Fall"]
    // As a card's season label reads it ("Fall 2026")
    readonly property string currentSeason: {
        var d = new Date(nowMs)
        return seasonWords[Math.floor(d.getMonth() / 3)] + " " + d.getFullYear()
    }
    // The sections, cut in Rust from the lean library read (Door.libraryView, #289): titles,
    // facts and series ids, nothing per card. At 300 series, cutting them here over every
    // card object, and Qt converting those objects for the Repeaters, was most of a visit.
    // A card reads its own record from Door.card once it is built.
    // `sections` changes only when the structure does (the view's key): a re-cut that only
    // moved the facts (the clock) or the counts leaves every section and card standing
    property var view: ({ ids: [], count: 0 })
    property var sections: []
    property var facts: []
    property var counts: ({})
    property string viewKey: ""
    function cut() {
        var r = Door.libraryView({ tab: tab, query: query, sort: sort, direction: direction, reveal: Door.revealHidden,
                                   arrange: arrange, group: groupFranchises, genre: genre, now: nowMs, season: currentSeason })
        loadFailed = !!r.error
        // A failed read is not an empty library, whatever the last read said
        if (r.error) { libraryEmpty = false; frame.fail(r.error, page.reload); return false }
        var v = r.reply
        counts = v.counts || {}
        if (v.key !== viewKey) {
            viewKey = v.key
            view = { ids: v.ids, count: v.count }
            sections = v.sections
        }
        facts = v.facts
        keepShownPicks()
        return true
    }
    // A change of cut re-cuts at once; the first reload and reload itself cut once, so a
    // visit asks the Door one time, not once per setting it seeds
    property bool settled: false
    onArrangeChanged: if (settled) cut()
    onGroupFranchisesChanged: if (settled) cut()
    onNowMsChanged: if (settled && !reloading) cut()
    property bool reloading: false
    readonly property int shownCount: view.count

    // For a --drive script's evals
    function cardIds() { return view.ids }
    function tileCount() { return view.count }
    function heroCount() { return 0 }
    function boxes() { return sections.filter(function(s) { return s.title !== "" }).map(function(s) { return { title: s.title, airing: s.airing, ids: s.ids } }) }
    function boxOf(id) { var b = boxes(); for (var i = 0; i < b.length; i++) if (b[i].ids.indexOf(id) >= 0) return b[i].title; return "" }
    function seriesWhere(text) { return view.ids.map(function(id) { return Door.card(id) }).filter(function(c) { return String(c.title).toLowerCase().indexOf(text) >= 0 }).map(function(c) { return { id: c.id, title: c.title, format: c.format, franchise: c.franchise === undefined ? "undefined" : c.franchise } }) }

    // The core's library read is served at once from the last answer while a fresh one
    // runs elsewhere; when that one differs, the page reads again (#289)
    Connections { target: Door; function onReadRefreshed() { Qt.callLater(page.reload) } }
    function reload() {
        var keep = scroll.contentY
        reloading = true
        nowMs = Date.now()
        reloading = false
        settled = true
        if (!cut()) return
        var n = counts
        tabCounts = n
        watchedExist = (n.watched || 0) > 0
        hiddenExist = Door.revealHidden && (n.hidden || 0) > 0
        libraryEmpty = (n.all || 0) === 0 && !watchedExist && (n.hidden || 0) === 0
        if (libraryEmpty) { var src = Door.listSources(); hasFolders = !src.error && src.reply.sources.length > 0 }
        scroll.contentY = Math.max(0, Math.min(keep, Math.max(0, scroll.contentHeight - scroll.height)))
    }
    Timer { id: debounce; interval: 400; onTriggered: page.reload() }
    function soon() { if (!debounce.running) debounce.start() }
    Timer { id: queryDebounce; interval: 150; onTriggered: { page.query = search.text; frame.nav.current.props = { q: page.query, genre: page.genre }; page.reload() } }
    Timer { interval: 30000; running: page.view.count > 0; repeat: true; onTriggered: page.nowMs = Date.now() }
    Connections {
        target: Door
        function onSeriesChanged() { page.soon() }
        function onSeriesRemoved() { page.soon() }
        function onPreferencesChanged(p) {
            var newTab = page.tab === "Hidden" ? page.tab : p.library_tab
            var changed = newTab !== page.tab || p.library_sort !== page.sort
                || p.library_direction !== page.direction || p.title_language !== page.titleLanguage
                || p.library_show_watched !== page.showWatched
            page.tab = newTab
            if (p.library_arrange !== undefined) page.arrange = page.cutOf(p.library_arrange)
            if (p.library_group_franchises !== undefined) page.groupFranchises = p.library_group_franchises === true
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
        arrange = cutOf(p.library_arrange || "None")
        groupFranchises = p.library_group_franchises !== false
        search.text = query
        var asked = props.arrange || (props.group === "Viewed" ? "WatchState" : props.group)
        if (arrangements.some(function(a) { return a[0] === cutOf(asked) })) arrange = cutOf(asked)
        if (props.sort && sorts.some(function(x) { return x[0] === props.sort })) { sort = props.sort; direction = props.sort === "Alpha" ? "Asc" : "Desc" }
        queryDebounce.stop()
        reload()
        // Home's See all names the box it stands for: the page opens on it
        if (props.box) Qt.callLater(page.revealBox, props.box)
    }
    function revealBox(title) {
        for (var i = 0; i < boxRepeater.count; i++) {
            var it = boxRepeater.itemAt(i)
            if (it && it.modelData.title === title) { scroll.contentY = Math.max(0, it.mapToItem(scroll.contentItem, 0, 0).y - theme.space(4)); return }
        }
    }

    // ---- The grid's geometry: as many columns as the width holds at the poster width
    readonly property real gap: theme.space(4)
    readonly property int cols: Math.max(1, Math.floor((scroll.innerWidth + gap) / (theme.posterWidth + gap)))
    // The columns stretch a little so the row ends on the head's right edge, the search
    // and the chips: every card is the same width, a fraction of a pixel included (#694)
    readonly property real cardWidth: (scroll.innerWidth - gap * (cols - 1)) / cols

    PageScroll {
        id: scroll
        anchors.fill: parent
        visible: !page.libraryEmpty
        // The column PageScroll centres the content in, less its side margins
        readonly property real innerWidth: column - theme.space(16)

        // ---- The head: the title and its count, the search at the right
        Item {
            width: parent.width
            height: theme.space(20)
            Label {
                id: titleLabel
                anchors.left: parent.left
                anchors.verticalCenter: parent.verticalCenter
                text: page.genre !== "" ? page.genre : "Library"
                role: "display"
                font.pointSize: theme.typePage
            }
            Button {
                id: clearGenreButton
                objectName: "clear-genre"
                visible: page.genre !== ""
                anchors.left: titleLabel.right; anchors.leftMargin: theme.space(3)
                anchors.verticalCenter: titleLabel.verticalCenter
                small: true
                flat: true
                icon: "x"
                text: "All genres"
                onClicked: page.clearGenre()
            }
            Label {
                objectName: "library-count"
                anchors.left: clearGenreButton.visible ? clearGenreButton.right : titleLabel.right
                anchors.leftMargin: theme.space(3)
                anchors.baseline: titleLabel.baseline
                text: page.countLine(page.shownCount)
                role: "ui"
                color: theme.textDim
            }
            SearchField {
                id: search
                objectName: "library-search"
                anchors.right: parent.right
                anchors.verticalCenter: parent.verticalCenter
                width: Math.min(theme.space(100), Math.max(theme.space(40), parent.width * 0.38))
                placeholder: "Search titles or folders"
                onTextChanged: queryDebounce.restart()
                onCleared: { queryDebounce.stop(); page.query = ""; frame.nav.current.props = page.genre !== "" ? { genre: page.genre } : {}; page.reload() }
            }
        }

        // ---- The controls: the tabs at the left; at the right what orders the grid and what
        // cuts it, each one pill that names itself ("Sort  Recently downloaded"), the order's
        // direction in words beside it, then the two switches as toggle chips. One line, or two
        // when the window is too narrow for both halves.
        Item {
            id: controls
            width: parent.width
            // Placed by x, not by anchors that flip: an anchor set and then cleared leaves the
            // row stretched to the width it was given
            readonly property bool oneLine: tabSeg.implicitWidth + theme.space(8) + orderRow.fullWidth <= width
            height: oneLine ? Math.max(tabSeg.height, orderRow.height) : tabSeg.height + theme.space(3) + orderRow.height
            Seg {
                id: tabSeg
                objectName: "tab"
                anchors.left: parent.left
                options: page.tabOptions
                index: Math.max(0, page.tabNames.indexOf(page.tab))
                onPicked: function(i) { page.pickTab(i) }
            }
            // The order pair and the cut trio. Fixed widths for the pills and the direction, so
            // a pick never moves the controls under the pointer or flips the line
            Flow {
                id: orderRow
                readonly property real fullWidth: sortPair.width + spacing + cutGroup.width
                width: controls.oneLine ? fullWidth : controls.width
                x: controls.oneLine ? controls.width - fullWidth : 0
                y: controls.oneLine ? (controls.height - height) / 2 : tabSeg.height + theme.space(3)
                spacing: theme.space(4)
                Row {
                    id: sortPair
                    spacing: theme.space(2)
                    Dropdown {
                        objectName: "sort-menu"
                        prefix: "Sort"
                        minWidth: theme.space(58)
                        options: page.sorts.map(function(s) { return s[1] })
                        index: Math.max(0, page.sorts.findIndex(function(s) { return s[0] === page.sort }))
                        onPicked: function(i) { page.pickSort(page.sorts[i][0]) }
                    }
                    Button {
                        objectName: "sort-direction"
                        anchors.verticalCenter: parent.verticalCenter
                        width: Math.max(implicitWidth, theme.space(46))
                        flat: true
                        icon: page.direction === "Asc" ? "arrow-up-narrow-wide" : "arrow-down-wide-narrow"
                        text: page.directionWords(page.sort, page.direction)
                        onClicked: page.flipDirection()
                    }
                }
                Row {
                    id: cutGroup
                    spacing: theme.space(2)
                    Dropdown {
                        objectName: "arrange-menu"
                        prefix: "Group"
                        minWidth: theme.space(42)
                        dividerAfter: 0
                        options: page.arrangements.map(function(a) { return a[1] })
                        index: Math.max(0, page.arrangements.findIndex(function(a) { return a[0] === page.arrange }))
                        onPicked: function(i) { page.pickArrange(page.arrangements[i][0]) }
                    }
                    // Kept in place under Nothing, faint and out of play, so the row never shifts
                    ToggleChip {
                        objectName: "group-franchises"
                        anchors.verticalCenter: parent.verticalCenter
                        interactive: page.arrange !== "None"
                        checked: page.groupFranchises
                        text: "Franchises together"
                        onToggled: function(on) { page.pickGroupFranchises(on) }
                    }
                    ToggleChip {
                        objectName: "show-watched"
                        anchors.verticalCenter: parent.verticalCenter
                        interactive: page.tab !== "Watched" && page.tab !== "Hidden"
                        checked: page.showWatched
                        text: "Watched shows"
                        onToggled: function(on) { page.pickShowWatched(on) }
                    }
                }
            }
        }

        TrackerNudge { id: nudge; width: parent.width; height: implicitHeight }

        // ---- The sections
        // Each section is a LazyTile of its own known height (header, then whole rows of
        // cards), so a page of 300 series builds only the sections near the view; a section
        // spends none of the first frame's budget itself, only its cards do (#289)
        Repeater {
            id: boxRepeater
            model: page.sections
            LazyTile {
                id: box
                required property var modelData
                required property int index
                spends: false
                width: parent.width
                readonly property int rows: Math.ceil(modelData.count / page.cols)
                height: (modelData.title !== "" ? theme.space(12) + theme.space(3) : 0)
                        + rows * Math.round(page.cardWidth * 1.5) + Math.max(0, rows - 1) * page.gap
                sourceComponent: Column {
                        id: section
                        readonly property var modelData: box.modelData
                        readonly property int index: box.index
                        readonly property string slug: String(modelData.title).toLowerCase().replace(/[^a-z0-9]+/g, "-")
                        objectName: modelData.title !== "" ? "box-" + slug : "box-all"
                        width: parent.width
                        spacing: theme.space(3)
                        Item {
                            visible: section.modelData.title !== ""
                            width: parent.width
                            height: visible ? theme.space(12) : 0
                            Row {
                                anchors.left: parent.left
                                anchors.bottom: parent.bottom
                                anchors.bottomMargin: theme.space(2)
                                spacing: theme.space(3)
                                Label {
                                    id: sectionTitle
                                    text: section.modelData.title
                                    role: "display"
                                    font.pointSize: theme.typeHeading
                                    font.variableAxes: ({ "wdth": 70, "wght": 420 })
                                    color: section.modelData.airing ? theme.accent : section.modelData.dropped ? theme.textDim : theme.text
                                }
                                Label { anchors.baseline: sectionTitle.baseline; text: section.modelData.count; role: "ledger"; color: theme.textFaint }
                                Label { anchors.baseline: sectionTitle.baseline; text: page.facts[section.index] || ""; role: "ui"; color: theme.textDim }
                            }
                        }
                        Flow {
                            // Half a pixel wider than the column so a row of exactly-fitting
                            // fractional cards never wraps its last card on rounding
                            width: parent.width + 0.5
                            spacing: page.gap
                            Repeater {
                                model: section.modelData.ids
                                LazyTile {
                                    id: tile
                                    required property int modelData
                                    width: page.cardWidth
                                    height: Math.round(page.cardWidth * 1.5)
                                    sourceComponent: Card {
                                        readonly property var series: Door.card(tile.modelData)
                                        objectName: "tile-" + series.id
                                        item: series
                                        posterWidth: page.cardWidth
                                        nowMs: page.nowMs
                                        selectable: true
                                        ranges: true
                                        selecting: page.selecting
                                        selected: !!page.picked[series.id]
                                        onSelectToggled: page.togglePick(series.id)
                                        onSelectRange: page.pickRange(series.id)
                                        onOpened: frame.go("series", { id: series.id }, series.title)
                                        onPlayed: page.playSeries(series)
                                    }
                                }
                            }
                        }
                    }
            }
        }

        // EmptyState centres itself in its parent, so it gets a plain box in the column
        Item {
            visible: page.view.count === 0 && !page.libraryEmpty
            width: parent.width
            height: visible ? theme.space(60) : 0
            readonly property string what: page.tab === "Series" ? "series" : page.tab === "Movies" ? "films" : page.tab === "Watched" ? "watched shows" : page.tab === "Hidden" ? "hidden shows" : "shows"
            EmptyState {
                objectName: "library-nothing-here"
                icon: page.loadFailed ? "circle-alert" : "search"
                title: page.loadFailed ? "The library did not load"
                     : page.query !== "" ? "No matches for \"" + page.query + "\""
                     : page.genre !== "" ? "No " + page.genre + " " + parent.what + " here"
                     : "No " + parent.what + " here"
                body: page.loadFailed ? "AniBeam could not read the library just now. Try again in a moment."
                    : page.query !== "" ? "Check the spelling, or search the whole library from All."
                    : page.genre !== "" ? "Nothing in this view has that genre. All genres shows the rest."
                    : page.tab !== "All" ? "Your library has none of these yet. All shows everything in it."
                    : "Everything in your library is filtered out of this view."
                Button { visible: page.loadFailed; text: "Try again"; icon: "refresh-cw"; primary: true; onClicked: page.reload() }
                Button { visible: !page.loadFailed && page.query !== ""; text: "Clear search"; icon: "x"; onClicked: { search.text = ""; queryDebounce.stop(); page.query = ""; frame.nav.current.props = page.genre !== "" ? { genre: page.genre } : {}; page.reload() } }
                Button { visible: !page.loadFailed && page.query === "" && page.genre !== ""; text: "All genres"; icon: "x"; onClicked: page.clearGenre() }
                Button { visible: !page.loadFailed && page.query === "" && page.genre === "" && page.tab !== "All"; text: "Show all"; icon: "layout-grid"; onClicked: page.showAll() }
            }
        }
    }

    // ---- Picking: how many, and what to do with all of them. Each action is greyed out when
    // no pick can take it: Rate and a list need a tracker, Find a match a pick without one.
    Rectangle {
        objectName: "library-picked"
        visible: page.selecting
        anchors.horizontalCenter: parent.horizontalCenter
        anchors.bottom: parent.bottom
        anchors.bottomMargin: theme.space(6)
        width: Math.min(parent.width - theme.space(8), pickRow.implicitWidth + theme.space(6))
        height: theme.controlHeight + theme.space(3)
        radius: theme.radiusTile
        color: theme.raised
        border.color: theme.accent
        border.width: 1
        // A click on the bar's own ground stays on the bar, not on the card under it
        MouseArea { anchors.fill: parent }
        Row {
            id: pickRow
            anchors.centerIn: parent
            spacing: theme.space(3)
            Label {
                objectName: "library-picked-count"
                anchors.verticalCenter: parent.verticalCenter
                text: page.pickedCount + " picked"
                role: "ui"
            }
            Button {
                objectName: "library-picked-all"
                small: true; flat: true
                visible: page.pickedCount < page.shownIds().length
                text: "All"; icon: "check-check"
                onClicked: page.pickAll()
                Tooltip { text: "Pick every card shown (Ctrl+A)" }
            }
            Button {
                objectName: "library-picked-hide"
                small: true
                text: page.tab === "Hidden" ? "Unhide" : "Hide"
                icon: page.tab === "Hidden" ? "eye" : "eye-off"
                onClicked: page.hidePicked()
            }
            Button {
                id: rateButton
                objectName: "library-picked-rate"
                small: true
                enabled: page.pickedTracked
                text: "Rate"; icon: "star"
                onClicked: pickScore.openAt(rateButton, -1)
                Tooltip { text: rateButton.enabled ? "One score for every pick, on every connected tracker" : "Needs a connected tracker and a matched show" }
            }
            Button {
                id: listButton
                objectName: "library-picked-list"
                small: true
                enabled: page.pickedTracked
                text: "Move to list"; icon: "bookmark"
                onClicked: page.listPicked(listButton)
                Tooltip { text: listButton.enabled ? "" : "Needs a connected tracker and a matched show" }
            }
            Button {
                objectName: "library-picked-rescan"
                small: true
                text: "Rescan"; icon: "refresh-cw"
                onClicked: page.rescanPicked()
            }
            Button {
                id: matchButton
                objectName: "library-picked-match"
                small: true
                enabled: page.pickedUnmatched
                text: "Find a match"; icon: "link"
                onClicked: page.matchPicked()
                Tooltip { text: matchButton.enabled ? "Search for each pick without a match" : "Every pick has a match" }
            }
            Button { objectName: "library-picked-cancel"; small: true; flat: true; text: "Cancel"; onClicked: page.clearPicks() }
        }
    }
    ScorePicker {
        id: pickScore
        parent: frame.overlay
        onSaved: function(v) { page.ratePicked(v) }
        onCleared: page.ratePicked(null)
    }

    // The empty library: a folder to add, or, with folders, a scan that found nothing
    EmptyState {
        objectName: "library-empty"
        visible: page.libraryEmpty
        icon: page.hasFolders ? "folder-search" : "tv"
        title: page.hasFolders ? "Nothing found in your folders yet" : "Your library is empty"
        body: page.hasFolders ? "AniBeam found no series or films in the folders it watches. Scan them again, or check the folders in Settings."
            : "Add the folder your anime lives in and the library fills itself, or import an AniBeam export."
        Button { visible: page.hasFolders; text: "Scan all"; icon: "refresh-cw"; primary: true; onClicked: page.scanAll() }
        Button { visible: !page.hasFolders; text: "Add a folder"; icon: "folder-plus"; primary: true; onClicked: frame.go("settings", { tab: "library" }) }
        Button { visible: page.hasFolders; text: "Library settings"; icon: "settings"; flat: true; onClicked: frame.go("settings", { tab: "library" }) }
        Button { visible: !page.hasFolders; text: "Import"; icon: "download"; flat: true; onClicked: importDialog.open() }
    }
    FileDialog {
        id: importDialog
        title: "Import an AniBeam export"
        nameFilters: ["AniBeam export (*.json)", "All files (*)"]
        onAccepted: { var r = Door.importLibrary(Fmt.localPath(String(selectedFile))); if (r.error) frame.fail(r.error); else frame.toast("Import started") }
    }
}
