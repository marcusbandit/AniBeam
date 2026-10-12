// The metadata table: every series in the library, the record it is matched
// to, and the files on disk against the expected count. The core owns the
// filtering and the counts (ListMetadata { filter, query, reveal_hidden });
// the page picks the filter, runs the two bulk jobs — Attach sources is the
// AutoMatch job, Refresh all the RefreshAll job — and opens the match panel
// over the overlay for a row that needs a different record.
import QtQuick
import com.marcusrosado.AniBeam

PageScroll {
    id: page
    property var props: ({})
    property string title: "Metadata"
    property real scrollY: 0
    function escapePressed() { return false }

    readonly property var filters: ["All", "Series", "Movies", "MissingFiles"]
    readonly property var filterLabels: ["All", "Series", "Movies", "Missing files"]
    property string filter: "All"
    property string query: props.q || ""
    property var counts: ({ all: 0, series: 0, movies: 0, missing_files: 0 })
    property var rows: []
    property bool loaded: false

    function reload() {
        var r = Door.listMetadata(page.filter, page.query, Door.revealHidden)
        if (r.error) { frame.toast(r.error.message); return }
        page.counts = r.reply.counts
        page.rows = r.reply.rows
        page.loaded = true
    }
    function filterCount(f) {
        return f === "All" ? page.counts.all : f === "Series" ? page.counts.series : f === "Movies" ? page.counts.movies : page.counts.missing_files
    }
    function crawlJob() {
        for (var i = 0; i < Door.runningJobs.length; i++) {
            var j = Door.runningJobs[i]
            if (String(j.kind).toLowerCase() === "crawl") return j
        }
        return null
    }
    Timer { id: debounce; interval: 250; onTriggered: page.reload() }
    Timer { id: queryDebounce; interval: 250; onTriggered: { page.query = search.text; page.reload() } }
    Connections {
        target: Door
        function onSeriesChanged(changed) { debounce.restart() }
        function onMatchApplied(series) { debounce.restart() }
        function onRevealHiddenChanged() { debounce.restart() }
    }
    Component.onCompleted: {
        search.text = query
        queryDebounce.stop()
        reload()
    }

    // The head: title and sub, the two bulk actions on the right
    Item {
        id: head
        width: parent.width
        height: Math.max(headCol.height, headBtns.height)
        Column {
            id: headCol
            anchors.left: parent.left
            spacing: theme.space(1)
            Text {
                text: "Metadata"
                color: theme.text
                font.family: theme.fontDisplay
                font.pointSize: theme.typeLarge
                font.weight: Font.DemiBold
            }
            Text {
                text: "Every series in the library, the record behind it, and the files on disk."
                color: theme.textDim
                font.family: theme.fontUi
                font.pointSize: theme.typeSmall
            }
        }
        Row {
            id: headBtns
            anchors.right: parent.right; anchors.verticalCenter: parent.verticalCenter
            spacing: theme.space(3)
            Button {
                text: "Attach sources"
                icon: "link"
                flat: true
                onClicked: {
                    var r = Door.autoMatch()
                    if (r.error) frame.toast(r.error.message)
                }
            }
            Button {
                text: "Refresh all"
                icon: "refresh-cw"
                flat: true
                onClicked: {
                    var r = Door.refreshAll()
                    if (r.error) frame.toast(r.error.message)
                }
            }
        }
    }

    // The crawl's own bar, only while a Crawl job runs
    Column {
        id: crawlBar
        width: parent.width
        spacing: theme.space(1)
        visible: crawlJob() !== null
        readonly property var job: crawlJob()
        Row {
            width: parent.width
            Text {
                text: "Franchise crawl"
                color: theme.textDim
                font.family: theme.fontUi
                font.pointSize: theme.typeSmall
            }
            Text {
                anchors.right: parent.right
                text: crawlBar.job && crawlBar.job.total > 0 ? crawlBar.job.done + " of " + crawlBar.job.total : ""
                color: theme.textDim
                font.family: theme.fontUi
                font.pointSize: theme.typeSmall
            }
        }
        Corner {
            width: parent.width; height: theme.space(1)
            radius: height / 2; smoothing: theme.cornerSmoothing
            color: theme.surfaceSunken
            Corner {
                width: parent.width * (crawlBar.job && crawlBar.job.total > 0 ? crawlBar.job.done / crawlBar.job.total : 0)
                height: parent.height
                radius: height / 2; smoothing: theme.cornerSmoothing
                color: theme.accent
            }
        }
    }

    // The controls: the filter with its counts, the query beside it
    Item {
        width: parent.width
        height: Math.max(filterSeg.height, search.height)
        Seg {
            id: filterSeg
            anchors.left: parent.left; anchors.verticalCenter: parent.verticalCenter
            options: page.filters.map(function(f, i) { return page.filterLabels[i] + " " + page.filterCount(f) })
            index: Math.max(0, page.filters.indexOf(page.filter))
            onPicked: function(i) { page.filter = page.filters[i]; page.reload() }
        }
        SearchField {
            id: search
            width: theme.space(100)
            anchors.right: parent.right; anchors.verticalCenter: parent.verticalCenter
            placeholder: "Filter titles"
            hint: ""
            onTextChanged: queryDebounce.restart()
        }
    }

    // The empty library, before any row exists. EmptyState centers itself, and
    // an anchored child inside a positioner breaks the positioner's layout, so
    // each one sits in a plain wrapper the Column can position.
    Item {
        width: parent.width
        height: emptyLib.height
        visible: emptyLib.visible
        EmptyState {
            id: emptyLib
            visible: page.loaded && page.counts.all === 0
            icon: "database"
            title: "No metadata yet"
            body: "Your library is empty. Add a folder in Settings and scan it to get started."
        }
    }
    Item {
        width: parent.width
        height: emptyNoRows.height
        visible: emptyNoRows.visible
        EmptyState {
            id: emptyNoRows
            visible: page.loaded && page.counts.all > 0 && page.rows.length === 0
            icon: "search"
            title: page.query !== "" ? "No matches for \"" + page.query + "\"" : "Nothing under this filter"
            body: page.query !== "" ? "" : "No series here right now."
        }
    }

    // The table
    Column {
        width: parent.width
        spacing: 0
        visible: page.rows.length > 0
        Repeater {
            model: page.rows
            MetaRow { }
        }
    }

    component MetaRow: Item {
        id: metaRow
        required property var modelData
        readonly property var row: modelData || ({})
        readonly property var d: row || ({})
        readonly property var card: d.series || ({})
        readonly property bool matched: !!(card.match_info && (card.match_info.anilist_id !== null || card.match_info.mal_id !== null))
        readonly property bool isMovie: card.kind === "Movie"
        readonly property real expected: d.expected !== null && d.expected !== undefined ? d.expected : (isMovie ? 1 : 0)
        readonly property real pct: expected > 0 ? Math.min(1, (d.have || 0) / expected) : 0
        readonly property real extra: expected > 0 ? Math.max(0, (d.have || 0) - expected) : 0
        width: parent.width
        height: theme.space(21)

        Rectangle { anchors.fill: parent; color: m.containsMouse ? theme.surface : "transparent" }

        Corner {
            id: thumb
            x: theme.space(2)
            width: theme.space(11); height: width * 1.5
            anchors.verticalCenter: parent.verticalCenter
            radius: 2; smoothing: 0
            color: theme.surface
            borderColor: theme.line; borderWidth: 1
            Image {
                width: thumb.width; height: thumb.height
                source: metaRow.card.poster ? "file://" + metaRow.card.poster : ""
                sourceSize: theme.posterDecodeSize
                fillMode: Image.PreserveAspectCrop
                smooth: true
                asynchronous: true
                cache: true
            }
            Text {
                visible: !metaRow.card.poster
                anchors.centerIn: parent
                text: metaRow.isMovie ? "Film" : "TV"
                color: theme.textFaint
                font.family: theme.fontUi
                font.pointSize: theme.typeSmall
            }
        }

        // The title block, between the thumb and the fixed right zones
        Column {
            anchors.left: thumb.right; anchors.leftMargin: theme.space(4)
            anchors.right: typeChip.left; anchors.rightMargin: theme.space(4)
            anchors.verticalCenter: parent.verticalCenter
            spacing: theme.space(0.5)
            Text {
                width: parent.width
                text: metaRow.card.title || (metaRow.card.titles ? metaRow.card.titles.folder : "")
                color: m.containsMouse ? theme.accent : theme.text
                font.family: theme.fontDisplay
                font.pointSize: theme.typeNormal
                font.weight: Font.DemiBold
                elide: Text.ElideRight
            }
            Text {
                visible: metaRow.d.alt_title !== null && metaRow.d.alt_title !== undefined
                width: parent.width
                text: metaRow.d.alt_title || ""
                color: theme.textDim
                font.family: theme.fontDisplay
                font.pointSize: theme.typeSmall
                elide: Text.ElideRight
            }
        }
        Chip {
            id: typeChip
            x: parent.width - theme.space(124)
            anchors.verticalCenter: parent.verticalCenter
            text: metaRow.isMovie ? "Movie" : "Series"
            textColor: theme.hue(Theme.formatHue(metaRow.isMovie ? "MOVIE" : "TV"))
        }
        Chip {
            x: parent.width - theme.space(104)
            anchors.verticalCenter: parent.verticalCenter
            text: !metaRow.matched ? "none" : (metaRow.card.match_info.provider || "")
            color: metaRow.matched ? theme.surfaceRaised : theme.surfaceSunken
            textColor: metaRow.matched ? theme.textDim : theme.textFaint
        }

        // The files readout: the bar, the count, the extra flag
        Row {
            anchors.right: parent.right; anchors.rightMargin: theme.space(52)
            anchors.verticalCenter: parent.verticalCenter
            spacing: theme.space(3)
            Corner {
                width: theme.space(30); height: theme.space(1.5)
                anchors.verticalCenter: parent.verticalCenter
                radius: height / 2; smoothing: theme.cornerSmoothing
                color: theme.surfaceSunken
                borderColor: theme.line; borderWidth: 1
                Corner {
                    width: parent.width * metaRow.pct
                    height: parent.height
                    radius: height / 2; smoothing: theme.cornerSmoothing
                    color: metaRow.extra > 0 ? theme.behind : theme.caughtUp
                }
            }
            Text {
                text: metaRow.d.have + " / " + (metaRow.expected > 0 ? metaRow.expected : "?")
                color: theme.textDim
                font.family: theme.fontUi
                font.pointSize: theme.typeSmall
                anchors.verticalCenter: parent.verticalCenter
            }
            Chip {
                visible: metaRow.extra > 0
                text: "+" + metaRow.extra
                small: true
                textColor: theme.behind
                Tooltip { text: "Files beyond the expected count, likely misnamed or stray" }
            }
        }

        // The row's actions: match, refresh
        Row {
            anchors.right: parent.right; anchors.rightMargin: theme.space(3)
            anchors.verticalCenter: parent.verticalCenter
            spacing: theme.space(2)
            Button {
                anchors.verticalCenter: parent.verticalCenter
                text: "Match"
                icon: "link"
                small: true
                flat: true
                onClicked: matchPanel.openFor(metaRow.card.id, metaRow.card.title, metaRow.card.match_info)
            }
            Button {
                anchors.verticalCenter: parent.verticalCenter
                text: "Refresh"
                icon: "refresh-cw"
                small: true
                flat: true
                onClicked: {
                    var r = Door.refreshSeries(metaRow.card.id)
                    if (r.error) frame.toast(r.error.message)
                }
            }
        }

        Rectangle {
            anchors.bottom: parent.bottom
            width: parent.width
            height: 1
            color: theme.line
        }
        MouseArea {
            id: m
            anchors.fill: parent
            hoverEnabled: true
            cursorShape: Qt.PointingHandCursor
            onClicked: matchPanel.openFor(metaRow.card.id, metaRow.card.title, metaRow.card.match_info)
        }
    }

    MatchPanel {
        id: matchPanel
        parent: frame.overlay
        anchors.centerIn: parent
    }
}
