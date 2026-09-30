// Spec 4.1 unit 2: the grid of every series, the search, the tabs, the sort, the Airing
// section and the count chip. The core searches and sorts; the page keeps the list in a
// RecordModel and reloads it, debounced, when the core says a series changed.
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
    property string query: props.q || ""
    property bool hiddenExist: false
    property real nowMs: Date.now()
    property int airingPage: 0
    readonly property int airingPageSize: 10
    property bool airingMore: false
    property bool libraryEmpty: false

    RecordModel { id: cards; roles: ["id", "title", "titles", "poster", "code", "watched", "watched_state", "total_episodes", "total_is_estimate", "strip", "community_score", "my_score", "hidden", "next_airing", "last_viewed_at", "episodes_on_disk", "kind"] }
    RecordModel { id: airing; roles: cards.roles }

    function persist() {
        var p = JSON.parse(JSON.stringify(Door.preferences))
        p.library_tab = tab === "Hidden" ? p.library_tab : tab
        p.library_sort = sort; p.library_direction = direction
        Door.setPreferences(p)
    }
    function pickTab(i) { tab = tabNames[i]; persist(); reload() }
    function pickSort(key) { sort = key; direction = key === "Alpha" ? "Asc" : "Desc"; persist(); reload() }
    function flipDirection() { direction = direction === "Asc" ? "Desc" : "Asc"; persist(); reload() }
    readonly property bool showHidden: Door.revealHidden && hiddenExist
    readonly property var tabNames: tabs.concat(showHidden ? ["Hidden"] : [])

    function reload() {
        var keep = grid.contentY
        var r = Door.listSeries(tab, query, sort, direction, Door.revealHidden)
        if (r.error) { frame.toast(r.error.message); return }
        cards.reset(r.reply.series)
        // The lower bound keeps a scroll position that used to sit inside the Airing
        // header sane once that header shrinks or disappears (a search hides it, a tab
        // switch loses it): without it, contentY stays at the old, now out-of-range
        // negative offset and the grid opens on an empty gap instead of its own top.
        var floor = -(grid.headerItem && grid.headerItem.visible ? grid.headerItem.height : 0)
        grid.contentY = Math.max(floor, Math.min(keep, Math.max(0, grid.contentHeight - grid.height)))
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
        reloadAiring()
    }
    function reloadAiring() {
        var r = Door.listAiring(airingPage * airingPageSize, airingPageSize + 1)
        if (r.error) return
        var rows = r.reply.series
        airingMore = rows.length > airingPageSize
        airing.reset(rows.slice(0, airingPageSize))
        if (rows.length === 0 && airingPage > 0) { airingPage = 0; reloadAiring() }
    }
    Timer { id: debounce; interval: 250; onTriggered: page.reload() }
    Timer { id: queryDebounce; interval: 150; onTriggered: { page.query = search.text; frame.nav.current.props = { q: page.query }; page.reload() } }
    Timer { interval: 30000; running: cards.count > 0 || airing.count > 0; repeat: true; onTriggered: page.nowMs = Date.now() }
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
            page.tab = newTab
            page.sort = p.library_sort
            page.direction = p.library_direction
            page.titleLanguage = p.title_language
            if (changed) debounce.restart()
        }
        function onRevealHiddenChanged() { if (page.tab === "Hidden" && !Door.revealHidden) page.tab = "All"; debounce.restart() }
    }
    // Frame now sets props before this fires (a Loader.setSource initial property, not a
    // later Loader.onLoaded assignment), so query already reads the trail's real value and
    // one reload here is the only core round trip a fresh navigation costs.
    Component.onCompleted: {
        var p = Door.preferences
        tab = p.library_tab || "All"
        sort = p.library_sort || "Alpha"
        direction = p.library_direction || "Asc"
        titleLanguage = p.title_language || "Romaji"
        // search.text starts "" (SearchField's own default), so setting it to a real
        // trail query is a genuine text change and queryDebounce.restart() fires from
        // SearchField's own onTextChanged; stop it right back so this one reload is the
        // only core round trip a fresh navigation costs, not a second one 150 ms later.
        search.text = query
        queryDebounce.stop()
        reload()
    }
    // GridView positions itself at its own top once, at construction, using whatever
    // headerItem.height reads at that instant. The Airing header's real height comes from
    // a Flow of Repeater-built Cards, which settles a few ticks late, so that one-time
    // position is taken against a too-small height and never corrected on its own; a search
    // hiding or a tab switch losing the section collapses it again later, and it needs to
    // be chased back too when the section returns. Rather than latch a single "has the
    // position been taken" flag (which round 1 did, and which then refused to chase the
    // section back once a search that briefly hid it had forced one real contentY write),
    // this compares the current position against where the last known header edge put it:
    // sitting exactly there means nothing has moved contentY on its own since, so following
    // the header's new height is still the right call; sitting anywhere else, a drag or a
    // deliberate restore (the frame's Back restore, reload()'s own keep-the-scroll clamp)
    // has taken over and this never touches it again. userScrolled alone is permanent,
    // for "never once the visitor has actually dragged", per the same rule.
    property bool userScrolled: false
    property real priorHeaderHeight: 0
    Connections { target: grid; function onMovementStarted() { page.userScrolled = true } }
    Connections {
        target: grid.headerItem
        function onHeightChanged() {
            var wasAtTop = !page.userScrolled && Math.abs(grid.contentY + page.priorHeaderHeight) < 1
            page.priorHeaderHeight = grid.headerItem.height
            if (wasAtTop) grid.positionViewAtBeginning()
        }
    }

    Column {
        id: header
        anchors.top: parent.top; anchors.topMargin: theme.space(7)
        anchors.left: parent.left; anchors.leftMargin: theme.space(8)
        anchors.right: parent.right; anchors.rightMargin: theme.space(8)
        spacing: theme.space(4)
        visible: !page.libraryEmpty
        Row {
            spacing: theme.space(3)
            Text { text: "Library"; color: theme.text; font.family: theme.fontSans; font.pointSize: theme.typeLarge; font.weight: Font.Bold; anchors.verticalCenter: parent.verticalCenter }
            Chip { anchors.verticalCenter: parent.verticalCenter; text: cards.count + (page.tab === "Movies" ? " films" : " series"); small: true; color: theme.surface; textColor: theme.textDim }
        }
        SearchField { id: search; onTextChanged: queryDebounce.restart(); onCleared: { queryDebounce.stop(); page.query = ""; frame.nav.current.props = {}; page.reload() } }
        Row {
            width: parent.width
            spacing: theme.space(3)
            Seg { options: page.tabNames; index: Math.max(0, page.tabNames.indexOf(page.tab)); onPicked: function(i) { page.pickTab(i) } }
            Item { width: theme.space(2); height: 1 }
            Repeater {
                model: page.sorts
                Chip {
                    required property var modelData
                    anchors.verticalCenter: parent.verticalCenter
                    text: modelData[1]; mono: false; clickable: true
                    selected: page.sort === modelData[0]
                    color: selected ? theme.accentSoft : theme.surface; textColor: theme.textDim
                    onClicked: page.pickSort(modelData[0])
                }
            }
            Chip { anchors.verticalCenter: parent.verticalCenter; text: page.direction === "Desc" ? "Desc" : "Asc"; clickable: true; color: theme.surface; textColor: theme.textDim; onClicked: page.flipDirection() }
        }
    }

    GridView {
        id: grid
        anchors.top: header.visible ? header.bottom : parent.top
        anchors.topMargin: theme.space(6)
        anchors.left: parent.left; anchors.leftMargin: theme.space(8)
        anchors.right: parent.right; anchors.rightMargin: theme.space(8)
        anchors.bottom: parent.bottom
        visible: !page.libraryEmpty
        clip: true
        readonly property real gapX: theme.space(5)
        readonly property real gapY: theme.space(6)
        readonly property int columns: Math.max(1, Math.floor((width + gapX) / (theme.posterWidth + gapX)))
        cellWidth: Math.floor((width + gapX) / columns)
        readonly property real cardWidth: cellWidth - gapX
        cellHeight: Math.ceil(cardWidth * 1.5 + theme.space(2) + theme.typeNormal * 2 * 1.5 + theme.typeSmall * 1.5 + theme.space(1)) + gapY
        model: cards
        cacheBuffer: 1200
        // header is a Component-typed property, so Pager, the one child here with its own
        // "page" property, would shadow the page id with itself inside it (the same trap
        // Frame.qml's hostWindow comment names for "window: window"). grid owns no such
        // property, so these proxies reach page safely from inside that boundary.
        readonly property int airingPage: page.airingPage
        readonly property bool airingMore: page.airingMore
        function pagerPrev() { page.airingPage--; page.reloadAiring() }
        function pagerNext() { page.airingPage++; page.reloadAiring() }
        QC.ScrollBar.vertical: ThinScrollBar {}
        header: Column {
            width: grid.width
            spacing: theme.space(4)
            visible: airing.count > 0 && page.query === ""
            height: visible ? implicitHeight + theme.space(6) : 0
            SectionHeader {
                title: "Airing"; count: airing.count
                Pager {
                    page: grid.airingPage
                    hasMore: grid.airingMore
                    onPrev: grid.pagerPrev()
                    onNext: grid.pagerNext()
                }
            }
            Flow {
                width: parent.width
                spacing: grid.gapX
                Repeater {
                    model: airing
                    Card { required property int index; item: airing.at(index); posterWidth: grid.cardWidth; nowMs: page.nowMs; onOpened: frame.go("series", { id: item.id }, item.title) }
                }
            }
            Rectangle { width: parent.width; height: 1; color: theme.line }
        }
        delegate: Item {
            required property int index
            width: grid.cellWidth; height: grid.cellHeight
            Card { item: cards.at(index); posterWidth: grid.cardWidth; nowMs: page.nowMs; onOpened: frame.go("series", { id: item.id }, item.title) }
        }
        footer: Item { width: 1; height: theme.space(10) }
        EmptyState {
            visible: cards.count === 0 && !page.libraryEmpty
            icon: "search"
            title: page.query !== "" ? "No matches for \"" + page.query + "\"." : "Nothing here"
            body: page.query !== "" ? "" : "No " + (page.tab === "Series" ? "series" : page.tab === "Movies" ? "films" : "items") + " in your library yet."
        }
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
