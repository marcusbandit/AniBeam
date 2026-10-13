// The library: a title with its count, the search, one row of pill controls (the tab, the
// sort, the arrangement, the watched switch), then the series in sections. A section is a
// heading in the display cut with one fact and a count, over a grid of uniform cards whose
// column count comes from the width. Related names the franchises with two or more series
// in the library as sections and gathers the rest under one; Type, Genre and Watch state
// section every group; None is one flat grid. The core searches and sorts; the page cuts
// the sections client-side and reloads, debounced, when the core says a series changed.
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
    function escapePressed() { return false }
    // The arrows with nothing focused go to the scroll's grid keys (Down lands on the first card)
    Keys.onPressed: function(event) { scroll.keyPressed(event) }

    readonly property var tabs: ["All", "Series", "Movies"]
    readonly property var sorts: [["Alpha", "A to Z"], ["LastViewed", "Last viewed"], ["Progress", "Progress"], ["CommunityScore", "Average score"], ["MyScore", "My score"]]
    readonly property var arrangements: [["Related", "Related"], ["Type", "Type"], ["Genre", "Genre"], ["WatchState", "Watch state"], ["None", "None"]]
    // Loaded-with copies, not bindings on Door.preferences: seeded once in
    // Component.onCompleted, ordinary state from then on, so the preferences-changed
    // handler below can compare what moved.
    property string tab: "All"
    property string sort: "Alpha"
    property string direction: "Asc"
    property string titleLanguage: "Romaji"
    property bool showWatched: false
    property string arrange: "Related"
    property string query: props.q || ""
    // A genre picked on Home: the grid holds that genre's series only, until cleared
    property string genre: props.genre || ""
    property bool hiddenExist: false
    property bool watchedExist: false
    property bool libraryEmpty: false
    property real nowMs: Date.now()
    property var airingRows: []
    property var cardList: []
    property int revision: 0

    function clearGenre() { genre = ""; frame.nav.current.props = { q: page.query }; frame.nav.relabel("Library"); reload() }
    function inGenre(c) { return genre === "" || (c.genres && c.genres.indexOf(genre) >= 0) }
    function persist() {
        var p = JSON.parse(JSON.stringify(Door.preferences))
        p.library_tab = tab === "Hidden" ? p.library_tab : tab
        p.library_sort = sort; p.library_direction = direction
        p.library_arrange = arrange
        Door.setPreferences(p)
    }
    function pickArrange(key) { arrange = key; persist() }
    function pickTab(i) { tab = tabNames[i]; persist(); reload() }
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
        var file = d ? (d.next_up ? d.next_up : (c.kind === "Movie" && d.episodes && d.episodes.length ? d.episodes[0].file : null)) : null
        if (file) frame.go("player", { file: file }, c.title)
        else frame.go("series", { id: c.id }, c.title)
    }
    readonly property bool showHidden: Door.revealHidden && hiddenExist
    readonly property var tabNames: tabs.concat(watchedExist ? ["Watched"] : []).concat(showHidden ? ["Hidden"] : [])

    // ---- The sections
    function typeOf(c) {
        var f = c.format || ""
        if (f === "TV" || f === "TV_SHORT") return "TV"
        if (f === "MOVIE" || (f === "" && c.kind === "Movie")) return "Films"
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
    function groupOf(c) {
        if (arrange === "Related") return c.franchise ? { key: "f" + c.franchise.id, title: c.franchise.title } : null
        if (arrange === "Type") { var t = typeOf(c); return { key: t, title: t } }
        if (arrange === "Genre") { var g = c.genres && c.genres.length ? c.genres[0] : "Unmatched"; return { key: g, title: g } }
        if (arrange === "WatchState") { var st = stateOf(c); return { key: st, title: st } }
        return null
    }
    function releaseOf(c) {
        if (c.start_date) return String(c.start_date)
        if (c.year) return String(c.year)
        return null
    }
    function shortCountdown(secs) {
        var t = Math.max(0, Math.floor(secs))
        var d = Math.floor(t / 86400), h = Math.floor(t % 86400 / 3600), m = Math.floor(t % 3600 / 60)
        return d > 0 ? d + "d " + h + "h" : h > 0 ? h + "h " + m + "m" : Math.max(1, m) + "m"
    }
    // A section's one fact: what its name and count do not already say
    function factOf(list) {
        var now = nowMs / 1000, going = 0, fresh = 0, done = 0, files = 0, soon = null
        for (var i = 0; i < list.length; i++) {
            var c = list[i], w = c.watched !== null && c.watched !== undefined ? c.watched : 0
            files += c.episodes_on_disk || 0
            if (c.watched_state === "CaughtUp" && !c.next_airing) done++
            else if (w > 0 || c.watched_state === "Behind") going++
            else fresh++
            var n = c.next_airing
            if (n && n.at > now && (!soon || n.at < soon.at)) soon = n
        }
        var n = list.length
        if (soon) return "Next episode in " + shortCountdown(soon.at - now)
        if (going > 0 && going < n) return going + " in progress"
        if (fresh > 0 && fresh < n) return fresh + " not started"
        if (n > 0 && done === n) return "All watched"
        if (files > 0) return Fmt.plural(files, "episode", "episodes") + " on disk"
        return ""
    }
    readonly property var sections: {
        page.revision
        var airingIds = {}
        for (var a = 0; a < airingRows.length; a++) airingIds[airingRows[a].id] = true
        var buckets = {}, order = [], lone = []
        for (var i = 0; i < cardList.length; i++) {
            var c = cardList[i]
            var g = groupOf(c)
            if (!g) { lone.push(c); continue }
            if (!buckets[g.key]) { buckets[g.key] = { title: g.title, cards: [] }; order.push(g.key) }
            buckets[g.key].cards.push(c)
        }
        var out = []
        // Related: a franchise with one series in the library is a lone one; the lone
        // ones airing now lead, the rest follow the franchises
        if (arrange === "Related") {
            var airing = [], rest = []
            for (var k = 0; k < order.length; k++) {
                var b = buckets[order[k]]
                if (b.cards.length < 2) { lone = lone.concat(b.cards); continue }
                if (b.cards.every(function(x) { return page.releaseOf(x) !== null }))
                    b.cards.sort(function(p, q) { var rp = page.releaseOf(p), rq = page.releaseOf(q); return rp < rq ? -1 : rp > rq ? 1 : 0 })
                out.push({ title: b.title, cards: b.cards, airing: b.cards.some(function(x) { return airingIds[x.id] === true }) })
            }
            for (var l = 0; l < lone.length; l++) (airingIds[lone[l].id] ? airing : rest).push(lone[l])
            if (query === "" && airing.length) out.unshift({ title: "Airing", cards: airing, airing: true })
            if (rest.length) out.push({ title: out.length ? "Everything else" : "", cards: rest, airing: false })
            return out
        }
        if (arrange === "None") return cardList.length ? [{ title: "", cards: cardList, airing: false }] : []
        for (var s = 0; s < order.length; s++) {
            var bb = buckets[order[s]]
            out.push({ title: bb.title, cards: bb.cards, airing: bb.cards.some(function(x) { return airingIds[x.id] === true }) })
        }
        return out
    }
    readonly property int shownCount: cardList.length

    // For a --drive script's evals
    function cardIds() { return cardList.map(function(c) { return c.id }) }
    function tileCount() { return cardList.length }
    function heroCount() { return 0 }
    function boxes() { return sections.filter(function(s) { return s.title !== "" }).map(function(s) { return { title: s.title, airing: s.airing, ids: s.cards.map(function(c) { return c.id }) } }) }
    function boxOf(id) { var b = boxes(); for (var i = 0; i < b.length; i++) if (b[i].ids.indexOf(id) >= 0) return b[i].title; return "" }
    function seriesWhere(text) { return cardList.filter(function(c) { return String(c.title).toLowerCase().indexOf(text) >= 0 }).map(function(c) { return { id: c.id, title: c.title, format: c.format, franchise: c.franchise === undefined ? "undefined" : c.franchise } }) }

    function reload() {
        var keep = scroll.contentY
        var r = Door.listLibrary(tab, query, sort, direction, Door.revealHidden)
        if (r.error) { frame.toast(r.error.message); return }
        cardList = r.reply.series.filter(page.inGenre)
        var n = r.reply.counts || {}
        watchedExist = (n.watched || 0) > 0
        hiddenExist = Door.revealHidden && (n.hidden || 0) > 0
        libraryEmpty = (n.all || 0) === 0 && !watchedExist && (n.hidden || 0) === 0
        airingRows = !libraryEmpty && r.reply.airing ? r.reply.airing.filter(page.inGenre) : []
        nowMs = Date.now()
        revision++
        scroll.contentY = Math.max(0, Math.min(keep, Math.max(0, scroll.contentHeight - scroll.height)))
    }
    Timer { id: debounce; interval: 400; onTriggered: page.reload() }
    function soon() { if (!debounce.running) debounce.start() }
    Timer { id: queryDebounce; interval: 150; onTriggered: { page.query = search.text; frame.nav.current.props = { q: page.query, genre: page.genre }; page.reload() } }
    Timer { interval: 30000; running: cardList.length > 0; repeat: true; onTriggered: page.nowMs = Date.now() }
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
        search.text = query
        var asked = props.arrange || (props.group === "Viewed" ? "WatchState" : props.group)
        if (arrangements.some(function(a) { return a[0] === asked })) arrange = asked
        queryDebounce.stop()
        reload()
    }

    // ---- The grid's geometry: as many columns as the width holds at the poster width
    readonly property real gap: theme.space(4)
    readonly property int cols: Math.max(1, Math.floor((scroll.innerWidth + gap) / (theme.posterWidth + gap)))
    readonly property real cardWidth: Math.floor((scroll.innerWidth - gap * (cols - 1)) / cols)
    readonly property bool compact: cols < 6

    PageScroll {
        id: scroll
        anchors.fill: parent
        visible: !page.libraryEmpty
        readonly property real innerWidth: width - theme.space(16)

        // ---- The head: the title and its count, the search at the right
        Item {
            width: parent.width
            height: theme.space(14)
            Label {
                id: titleLabel
                anchors.left: parent.left
                anchors.verticalCenter: parent.verticalCenter
                text: page.genre !== "" ? page.genre : "Library"
                role: "display"
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
                text: Fmt.plural(page.shownCount, "series", "series")
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

        // ---- The controls: one line of pills, wrapping on a narrow window
        Flow {
            width: parent.width
            spacing: theme.space(4)
            Seg {
                objectName: "tab"
                options: page.tabNames
                index: Math.max(0, page.tabNames.indexOf(page.tab))
                onPicked: function(i) { page.pickTab(i) }
            }
            Seg {
                objectName: "sort"
                visible: !page.compact
                options: page.sorts.map(function(s) {
                    return { text: s[1], name: s[0], icon: s[0] === page.sort ? (page.direction === "Asc" ? "chevron-up" : "chevron-down") : "" }
                })
                index: Math.max(0, page.sorts.findIndex(function(s) { return s[0] === page.sort }))
                onPicked: function(i) { if (i === index) page.flipDirection(); else page.pickSort(page.sorts[i][0]) }
            }
            Row {
                visible: page.compact
                spacing: theme.space(1)
                Dropdown {
                    objectName: "sort-menu"
                    minWidth: theme.space(28)
                    options: page.sorts.map(function(s) { return s[1] })
                    index: Math.max(0, page.sorts.findIndex(function(s) { return s[0] === page.sort }))
                    onPicked: function(i) { page.pickSort(page.sorts[i][0]) }
                }
                Button {
                    objectName: "sort-direction"
                    anchors.verticalCenter: parent.verticalCenter
                    small: true; flat: true
                    icon: page.direction === "Asc" ? "chevron-up" : "chevron-down"
                    onClicked: page.flipDirection()
                }
            }
            Row {
                spacing: theme.space(2.5)
                Label { anchors.verticalCenter: parent.verticalCenter; text: "Arrange"; role: "ui"; color: theme.textDim }
                Seg {
                    objectName: "arrange"
                    visible: !page.compact
                    options: page.arrangements.map(function(a) { return { text: a[1], name: a[0] } })
                    index: Math.max(0, page.arrangements.findIndex(function(a) { return a[0] === page.arrange }))
                    onPicked: function(i) { page.pickArrange(page.arrangements[i][0]) }
                }
                Dropdown {
                    objectName: "arrange-menu"
                    visible: page.compact
                    minWidth: theme.space(28)
                    options: page.arrangements.map(function(a) { return a[1] })
                    index: Math.max(0, page.arrangements.findIndex(function(a) { return a[0] === page.arrange }))
                    onPicked: function(i) { page.pickArrange(page.arrangements[i][0]) }
                }
            }
            Row {
                spacing: theme.space(2.5)
                height: theme.controlHeight
                Label { anchors.verticalCenter: parent.verticalCenter; text: "Show watched"; role: "ui"; color: theme.textDim }
                Switch {
                    objectName: "show-watched"
                    anchors.verticalCenter: parent.verticalCenter
                    checked: page.showWatched
                    onToggled: function(on) { page.pickShowWatched(on); checked = Qt.binding(function() { return page.showWatched }) }
                }
            }
        }

        TrackerNudge { id: nudge; width: parent.width; height: implicitHeight }

        // ---- The sections
        Repeater {
            model: page.sections
            Column {
                id: section
                required property var modelData
                required property int index
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
                            font.pointSize: theme.typeLarge
                            font.variableAxes: ({ "wdth": 74, "wght": 500 })
                            color: section.modelData.airing ? theme.accent : theme.text
                        }
                        Label { anchors.baseline: sectionTitle.baseline; text: section.modelData.cards.length; role: "ledger"; color: theme.textFaint }
                        Label { anchors.baseline: sectionTitle.baseline; text: page.factOf(section.modelData.cards); role: "ui"; color: theme.textDim }
                    }
                }
                Flow {
                    width: parent.width
                    spacing: page.gap
                    Repeater {
                        model: section.modelData.cards
                        Card {
                            required property var modelData
                            objectName: "tile-" + modelData.id
                            item: modelData
                            posterWidth: page.cardWidth
                            nowMs: page.nowMs
                            onOpened: frame.go("series", { id: modelData.id }, modelData.title)
                            onPlayed: page.playSeries(modelData)
                        }
                    }
                }
            }
        }

        // EmptyState centres itself in its parent, so it gets a plain box in the column
        Item {
            visible: page.cardList.length === 0 && !page.libraryEmpty
            width: parent.width
            height: visible ? theme.space(60) : 0
            EmptyState {
                icon: "search"
                title: page.query !== "" ? "No matches for \"" + page.query + "\"" : "Nothing here"
                body: page.query !== "" ? "" : "No " + (page.tab === "Series" ? "series" : page.tab === "Movies" ? "films" : page.tab === "Watched" ? "watched items" : "items") + " in your library yet."
            }
        }
    }

    // The empty library: Import, and a pointer at Settings
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
