// Spec 4.1 unit 2, in the demo's language: a breadcrumb bar, a controls band, and the
// poster grid as numbered sections — Airing first, then everything, grouped by genre or
// viewed state on demand. The core searches and sorts; the page partitions the sorted
// list into sections client-side and keeps it in a RecordModel, reloaded, debounced,
// when the core says a series changed.
import QtQuick
import QtQuick.Controls.Basic as QC
import QtQuick.Dialogs
import com.marcusrosado.AniBeam

Item {
    id: page
    property var props: ({})
    property alias scrollY: grid.contentY
    function focusSearch() { search.focusInput() }
    // "escape" is reserved by the QML compiler; the frame's hook is named escapePressed.
    function escapePressed() { return false }

    readonly property var tabs: ["All", "Series", "Movies"]
    readonly property var sorts: [["Alpha", "A to Z"], ["LastViewed", "Last viewed"], ["Progress", "Progress"], ["CommunityScore", "Score"], ["MyScore", "My score"]]
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
    property bool libraryEmpty: false
    property string groupBy: "None"   // None | Genre | Viewed

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
    readonly property bool showHidden: Door.revealHidden && hiddenExist
    readonly property var tabNames: tabs.concat(watchedExist ? ["Watched"] : []).concat(showHidden ? ["Hidden"] : [])

    // ---- The sections: one entry per section, carrying its cards. The section label is
    // the header — "Airing" with its count beside it — grouped views by size descending.
    // The airing rail is capped high; its own pager lived on the old grid and the stream
    // replaces it.
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
    readonly property bool airingShows: page.query === "" && !page.libraryEmpty
    readonly property var groups: {
        var airingRows = []
        if (airingShows) {
            var r = Door.listAiring(0, 60)
            if (!r.error) airingRows = r.reply.series
        }
        var main = page.tab === "All" ? "Everything" : page.tab
        var buckets = {}
        var order = []
        for (var i = 0; i < cards.count; i++) {
            var c = cards.at(i)
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
        if (airingShows) out.push({ label: "Airing", count: airingRows.length, cards: airingRows })
        for (var s = 0; s < order.length; s++) out.push({ label: order[s], count: buckets[order[s]].length, cards: buckets[order[s]] })
        return out
    }

    function reload() {
        var keep = grid.contentY
        var r = Door.listSeries(tab, query, sort, direction, Door.revealHidden)
        if (r.error) { frame.toast(r.error.message); return }
        cards.reset(r.reply.series)
        if (query === "" && tab === "All") {
            libraryEmpty = r.reply.series.length === 0 && !Door.revealHidden
        } else {
            var all = Door.listSeries("All", "", "Alpha", "Asc", false)
            if (!all.error) libraryEmpty = all.reply.series.length === 0 && !Door.revealHidden
        }
        if (Door.revealHidden) {
            var hidden = Door.listSeries("Hidden", "", "Alpha", "Asc", true)
            if (!hidden.error) hiddenExist = hidden.reply.series.length > 0
        } else {
            hiddenExist = false
        }
        // The Watched tab exists while watched series do, whatever the
        // toggle says: the section is where they live, the toggle is only
        // about joining them back into the others.
        var watched = Door.listSeries("Watched", "", "Alpha", "Asc", false)
        if (!watched.error) watchedExist = watched.reply.series.length > 0
        grid.contentY = Math.max(0, Math.min(keep, Math.max(0, grid.contentHeight - grid.height)))
    }
    Timer { id: debounce; interval: 250; onTriggered: page.reload() }
    Timer { id: queryDebounce; interval: 150; onTriggered: { page.query = search.text; frame.nav.current.props = { q: page.query }; page.reload() } }
    Timer { interval: 30000; running: cards.count > 0; repeat: true; onTriggered: page.nowMs = Date.now() }
    Connections {
        target: Door
        function onSeriesChanged() { debounce.restart() }
        function onSeriesRemoved() { debounce.restart() }
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
            if (changed) debounce.restart()
        }
        function onRevealHiddenChanged() { if (page.tab === "Hidden" && !Door.revealHidden) page.tab = "All"; debounce.restart() }
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
    }

    // ---- The page header: the title in the voice, the live count on the right
    Item {
        id: crumbBar
        anchors.left: parent.left; anchors.leftMargin: theme.space(8)
        // The right margin carries the scroll bar: the column pulls in two extra units so
        // the bar floats in the page's margin instead of over the cards' edge.
        anchors.right: parent.right; anchors.rightMargin: theme.space(10)
        anchors.top: parent.top; anchors.topMargin: theme.space(7)
        height: crumb.height
        visible: !page.libraryEmpty
        Text {
            id: crumb
            anchors.left: parent.left; anchors.verticalCenter: parent.verticalCenter
            text: "Library"
            color: theme.text
            font.family: theme.fontDisplay
            font.pointSize: theme.typeLarge
            font.weight: Font.DemiBold
        }
        Text {
            anchors.right: parent.right; anchors.verticalCenter: parent.verticalCenter
            text: cards.count + " series"
            color: theme.textDim
            font.family: theme.fontUi
            font.pointSize: theme.typeSmall
        }
    }

    // ---- Controls band: the tabs and the search on one edge-aligned row, the sorts and
    // the group switch under them, a hairline closing the band.
    Item {
        id: controls
        anchors.left: crumbBar.left; anchors.right: crumbBar.right
        anchors.top: crumbBar.bottom; anchors.topMargin: theme.space(5)
        height: tabsRow.height + sortsRow.height + theme.space(5)
        visible: !page.libraryEmpty
        Seg {
            id: tabsRow
            anchors.left: parent.left; anchors.top: parent.top
            options: page.tabNames
            index: Math.max(0, page.tabNames.indexOf(page.tab))
            onPicked: function(i) { page.pickTab(i) }
        }
        SearchField {
            id: search; width: theme.space(100)
            anchors.right: parent.right; anchors.top: parent.top
            onTextChanged: queryDebounce.restart()
            onCleared: { queryDebounce.stop(); page.query = ""; frame.nav.current.props = {}; page.reload() }
        }
        Row {
            id: sortsRow
            anchors.left: parent.left; anchors.bottom: parent.bottom
            spacing: theme.space(4)
            Repeater {
                model: page.sorts
                Text {
                    required property var modelData
                    readonly property bool on: page.sort === modelData[0]
                    text: modelData[1] + (on && page.direction === "Desc" && modelData[0] !== "Alpha" ? " ↓" : on ? " ↑" : "")
                    color: on ? theme.text : theme.textFaint
                    font.family: theme.fontUi
                    font.pointSize: theme.typeSmall
                    font.weight: on ? Font.DemiBold : Font.Normal
                    Rectangle { visible: parent.on; anchors.bottom: parent.bottom; width: parent.width; height: 2; color: theme.accent }
                    MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: page.pickSort(parent.modelData[0]) }
                }
            }
        }
        Row {
            anchors.right: parent.right; anchors.bottom: parent.bottom
            spacing: theme.space(3)
            Text {
                text: "Group"
                color: theme.textFaint
                font.family: theme.fontUi
                font.pointSize: theme.typeSmall
                anchors.verticalCenter: parent.verticalCenter
            }
            Seg {
                small: true
                options: ["None", "Genre", "Viewed"]
                index: ["None", "Genre", "Viewed"].indexOf(page.groupBy)
                onPicked: function(i) { page.groupBy = ["None", "Genre", "Viewed"][i] }
            }
        }
    }
    Rectangle {
        anchors.left: crumbBar.left; anchors.right: crumbBar.right
        anchors.top: controls.bottom; anchors.topMargin: theme.space(4)
        height: 1; color: theme.line
        visible: !page.libraryEmpty
    }

    // ---- The sectioned grid: one block per section, a numbered caps header over a Flow
    // of cards. The ListView builds a block only when it scrolls near, so a big library
    // never decodes its whole shelf at open.
    ListView {
        id: grid
        anchors.left: crumbBar.left; anchors.right: crumbBar.right
        anchors.top: controls.bottom; anchors.topMargin: theme.space(6)
        anchors.bottom: parent.bottom
        visible: !page.libraryEmpty
        clip: true
        boundsBehavior: Flickable.StopAtBounds
        readonly property real gapX: theme.space(5)
        readonly property real cardWidth: {
            var columns = Math.max(1, Math.floor((width + gapX) / (theme.posterWidth + gapX)))
            return Math.floor((width + gapX) / columns) - gapX
        }
        model: page.groups
        cacheBuffer: 600
        QC.ScrollBar.vertical: ThinScrollBar {}
        WheelStep { flick: grid }
        spacing: theme.space(7)
        delegate: Column {
            required property var modelData
            width: grid.width
            spacing: theme.space(4)
            Row {
                spacing: theme.space(2)
                Text {
                    text: modelData.label
                    color: theme.text
                    font.family: theme.fontDisplay
                    font.pointSize: theme.typeNormal * 1.15
                    font.weight: Font.DemiBold
                }
                Text {
                    text: modelData.count
                    color: theme.textFaint
                    font.family: theme.fontUi
                    font.pointSize: theme.typeSmall
                    anchors.verticalCenter: parent.verticalCenter
                }
            }
            Flow {
                width: parent.width
                spacing: grid.gapX
                Repeater {
                    model: modelData.cards
                    Card {
                        required property var modelData
                        item: modelData
                        posterWidth: grid.cardWidth; nowMs: page.nowMs
                        onOpened: frame.go("series", { id: modelData.id }, modelData.title)
                    }
                }
            }
        }
        EmptyState {
            visible: cards.count === 0 && !page.libraryEmpty
            icon: "search"
            title: page.query !== "" ? "No matches for \"" + page.query + "\"." : "Nothing here"
            body: page.query !== "" ? "" : "No " + (page.tab === "Series" ? "series" : page.tab === "Movies" ? "films" : page.tab === "Watched" ? "watched items" : "items") + " in your library yet."
        }
        footer: Item { width: 1; height: theme.space(10) }
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
