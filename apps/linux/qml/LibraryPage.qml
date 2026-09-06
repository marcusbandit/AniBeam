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
    readonly property var prefs: Door.preferences
    property string tab: prefs.library_tab || "All"
    property string sort: prefs.library_sort || "Alpha"
    property string direction: prefs.library_direction || "Asc"
    property string titleLanguage: prefs.title_language || "Romaji"
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
    Component.onCompleted: reload()
    // Frame assigns props in the Loader's onLoaded, which runs after this page's own
    // Component.onCompleted, so props.q is never here yet at construction; react to the
    // assignment instead. This also carries the trail's search text back in through the
    // same debounce a keystroke uses, so the filtered grid and the empty-state copy both
    // follow it.
    onPropsChanged: search.text = props.q || ""
    // GridView positions itself at its own top once, at construction, using whatever
    // headerItem.height reads at that instant. The Airing header's real height comes from
    // a Flow of Repeater-built Cards, which settles a few ticks late, so that one-time
    // position is taken against a too-small height and never corrected on its own. Keep
    // reasserting it, event-driven rather than timed, until the initial position is taken
    // for real: a genuine drag, or any other write to contentY that is not this very
    // correction (the frame's Back restore through the scrollY alias, reload()'s own
    // keep-the-scroll clamp). After that, a later legitimate resize (a search hiding the
    // section, a tab switch) is left alone rather than fought.
    property bool positionTaken: false
    property bool correctingHeaderPosition: false
    function correctHeaderPosition() {
        correctingHeaderPosition = true
        grid.positionViewAtBeginning()
        correctingHeaderPosition = false
    }
    Connections { target: grid; function onMovementStarted() { page.positionTaken = true } }
    Connections { target: grid; function onContentYChanged() { if (!page.correctingHeaderPosition) page.positionTaken = true } }
    Connections {
        target: grid.headerItem
        function onHeightChanged() { if (!page.positionTaken) page.correctHeaderPosition() }
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
