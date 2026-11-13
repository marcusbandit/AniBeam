// Stats (#216): what a year of watching adds up to, from the history the core keeps. A head
// with the year's picker, a row of four figures (hours, episodes, series finished, the busiest
// month), a bar per month, then the top genres, studios and seasons as ranked rows, and the
// recap: one card for the year with the shows watched most, which Save as image writes to a
// PNG. GetStats reads only; the page reads again when a series changes (an episode finished).
import QtCore
import QtQuick
import QtQuick.Dialogs
import com.marcusrosado.AniBeam

Item {
    id: page
    objectName: "stats-page"
    property var props: ({})
    property string title: words.t("stats-title")
    property alias scrollY: scroll.contentY
    function escapePressed() { return false }

    property int year: props.year || 0
    property var stats: null
    readonly property var monthIds: ["stats-month-jan", "stats-month-feb", "stats-month-mar", "stats-month-apr", "stats-month-may", "stats-month-jun",
                                     "stats-month-jul", "stats-month-aug", "stats-month-sep", "stats-month-oct", "stats-month-nov", "stats-month-dec"]
    function hours(minutes) { var h = minutes / 60; return h >= 10 ? Math.round(h) : Math.round(h * 10) / 10 }
    readonly property int busiest: {
        if (!stats) return -1
        var best = -1, n = 0
        for (var i = 0; i < 12; i++) if (stats.months[i].minutes > n) { n = stats.months[i].minutes; best = i }
        return best
    }
    readonly property real monthMax: {
        if (!stats) return 1
        var m = 1
        for (var i = 0; i < 12; i++) m = Math.max(m, stats.months[i].minutes)
        return m
    }

    // The years to pick from: this one, and every year with any watching, newest first
    readonly property var yearOptions: {
        if (!stats) return []
        var ys = stats.years.slice()
        var now = new Date().getFullYear()
        if (ys.indexOf(now) < 0) ys.push(now)
        if (ys.indexOf(stats.year) < 0) ys.push(stats.year)
        return ys.sort(function(a, b) { return b - a })
    }

    function reload() {
        stale = false
        var y = page.year > 0 ? page.year : new Date().getFullYear()
        // The local midnights opening each month and the next January, so a month keeps its
        // edges across a clock change; the offset places the other years
        var starts = []
        for (var m = 0; m <= 12; m++) starts.push(Math.floor(new Date(y, m, 1).getTime() / 1000))
        var offset = -new Date(y, 0, 1).getTimezoneOffset() * 60
        var r = Door.call("GetStats", { year: y, utc_offset: offset, month_starts: starts })
        if (r.error) { frame.toast(frame.errorText(r.error)); return }
        page.stats = r.reply.stats
        if (page.year === 0) page.year = page.stats.year
    }
    function pickYear(y) { page.year = y; frame.nav.current.props = { year: y }; page.reload() }
    // A finished episode changes the numbers, but a scan says SeriesChanged in bursts and
    // every read loads the library: read once the burst settles, and only while on screen
    property bool stale: false
    Timer { id: debounce; interval: 1500; onTriggered: page.visible ? page.reload() : (page.stale = true) }
    Connections { target: Door; function onSeriesChanged() { debounce.restart() } }
    onVisibleChanged: if (visible && stale) reload()
    Component.onCompleted: reload()

    // A figure: a big number over a dim word
    component Figure: Tile {
        id: figure
        property string value: ""
        property string label: ""
        height: theme.space(28)
        Column {
            anchors.left: parent.left
            anchors.verticalCenter: parent.verticalCenter
            spacing: theme.space(1)
            Label { text: figure.value; role: "display"; font.pointSize: theme.typePage }
            Label { text: figure.label; role: "ui"; color: theme.textDim }
        }
    }
    // A ranked list: a name, a bar for its share of the leader, the episodes
    component Ranked: Tile {
        id: ranked
        property string heading: ""
        property var rows: []
        height: theme.space(14) + Math.max(1, rows.length) * theme.space(9)
        Column {
            anchors.fill: parent
            spacing: theme.space(2)
            Label { text: ranked.heading; role: "title" }
            Label { visible: ranked.rows.length === 0; msg: "stats-nothing-yet"; role: "ui"; color: theme.textFaint }
            Repeater {
                model: ranked.rows
                Item {
                    required property var modelData
                    required property int index
                    width: parent.width
                    height: theme.space(7)
                    Label { id: rankName; anchors.left: parent.left; anchors.verticalCenter: parent.verticalCenter; width: parent.width * 0.42; elide: Text.ElideRight; text: modelData.name; role: "ui" }
                    Rectangle {
                        anchors.left: rankName.right; anchors.verticalCenter: parent.verticalCenter
                        height: theme.space(2); radius: height / 2
                        width: Math.max(height, (parent.width * 0.4) * modelData.episodes / Math.max(1, ranked.rows[0].episodes))
                        color: index === 0 ? theme.accent : theme.line
                    }
                    Label { anchors.right: parent.right; anchors.verticalCenter: parent.verticalCenter; msg: "stats-ranked-episodes"; msgArgs: ({ count: modelData.episodes }); role: "ledger"; color: theme.textDim }
                }
            }
        }
    }

    PageScroll {
        id: scroll
        anchors.fill: parent

        // ---- The head: the title, the year's words, the years with any watching
        Item {
            width: parent.width
            height: theme.space(20)
            Label { id: titleLabel; anchors.left: parent.left; anchors.verticalCenter: parent.verticalCenter; msg: "stats-title"; role: "display"; font.pointSize: theme.typePage }
            Label {
                anchors.left: titleLabel.right; anchors.leftMargin: theme.space(3); anchors.baseline: titleLabel.baseline
                text: page.stats ? words.t("stats-year-summary", { year: String(page.stats.year) }) : ""
                role: "ui"; color: theme.textDim
            }
            Seg {
                objectName: "stats-year"
                visible: page.yearOptions.length > 1
                anchors.right: parent.right; anchors.verticalCenter: parent.verticalCenter
                options: page.yearOptions.map(function(y) { return { text: String(y), name: String(y) } })
                index: Math.max(0, page.yearOptions.indexOf(page.stats ? page.stats.year : page.year))
                onPicked: function(i) { page.pickYear(page.yearOptions[i]) }
            }
        }

        // ---- Nothing watched yet
        Item {
            visible: !!page.stats && page.stats.years.length === 0
            width: parent.width; height: visible ? theme.space(60) : 0
            EmptyState {
                icon: "chart-pie"
                title: words.t("stats-empty-title")
                body: words.t("stats-empty-body")
                Button { msg: "stats-open-library"; icon: "layers"; onClicked: frame.go("library") }
            }
        }

        // ---- Four figures
        Row {
            visible: !!page.stats && page.stats.years.length > 0
            width: parent.width
            spacing: theme.grout
            readonly property real cell: (width - 3 * spacing) / 4
            Figure { width: parent.cell; objectName: "stats-hours"; value: page.stats ? String(page.hours(page.stats.minutes)) : ""; label: words.t("stats-hours-watched") }
            Figure { width: parent.cell; value: page.stats ? String(page.stats.episodes) : ""; label: words.t("stats-episodes-label", { count: page.stats ? page.stats.episodes : 0 }) }
            Figure { width: parent.cell; value: page.stats ? String(page.stats.series_completed) : ""; label: words.t("stats-series-finished") }
            Figure { width: parent.cell; value: page.busiest >= 0 ? words.t(page.monthIds[page.busiest]) : words.t("stats-none"); label: words.t("stats-busiest-month") }
        }

        // ---- A bar per month, hours above it
        Tile {
            visible: !!page.stats && page.stats.years.length > 0
            width: parent.width
            height: theme.space(64)
            Label { id: monthsTitle; msg: "stats-by-month"; role: "title" }
            Row {
                id: bars
                anchors.top: monthsTitle.bottom; anchors.topMargin: theme.space(4)
                anchors.bottom: parent.bottom
                width: parent.width
                spacing: theme.space(2)
                readonly property real cell: (width - 11 * spacing) / 12
                Repeater {
                    model: page.stats ? page.stats.months : []
                    Item {
                        required property var modelData
                        required property int index
                        width: bars.cell; height: bars.height
                        Label { id: monthLabel; anchors.bottom: parent.bottom; anchors.horizontalCenter: parent.horizontalCenter; text: words.t(page.monthIds[index]); role: "ledger"; color: theme.textDim }
                        Rectangle {
                            id: bar
                            anchors.bottom: monthLabel.top; anchors.bottomMargin: theme.space(2)
                            anchors.horizontalCenter: parent.horizontalCenter
                            width: Math.min(parent.width, theme.space(10))
                            height: Math.max(modelData.minutes > 0 ? 3 : 1, (parent.height - theme.space(16)) * modelData.minutes / page.monthMax)
                            radius: theme.radiusSm
                            color: index === page.busiest ? theme.accent : modelData.minutes > 0 ? theme.textFaint : theme.line
                        }
                        Label { visible: modelData.minutes > 0; anchors.bottom: bar.top; anchors.bottomMargin: theme.space(1); anchors.horizontalCenter: parent.horizontalCenter; msg: "stats-month-hours"; msgArgs: ({ hours: page.hours(modelData.minutes) }); role: "ledger"; color: theme.textDim }
                    }
                }
            }
        }

        // ---- Genres, studios, seasons
        Row {
            visible: !!page.stats && page.stats.years.length > 0
            width: parent.width
            spacing: theme.grout
            readonly property real cell: (width - 2 * spacing) / 3
            Ranked { width: parent.cell; heading: words.t("stats-top-genres"); rows: page.stats ? page.stats.genres : [] }
            Ranked { width: parent.cell; heading: words.t("stats-top-studios"); rows: page.stats ? page.stats.studios : [] }
            Ranked { width: parent.cell; heading: words.t("stats-seasons-watched"); rows: page.stats ? page.stats.seasons : [] }
        }

        // ---- The recap: one card for the year, saved as a PNG
        Item {
            visible: !!page.stats && page.stats.years.length > 0
            width: parent.width
            height: recap.height
            Tile {
                id: recap
                objectName: "stats-recap"
                // Room kept beside it for Save as image, so the button never falls off the edge
                width: Math.min(parent.width - saveRecapButton.width - theme.space(4), theme.space(160))
                height: width * 0.62
                padding: theme.space(8)
                Column {
                    id: recapHead
                    spacing: theme.space(1)
                    Label { text: page.stats ? String(page.stats.year) : ""; role: "display"; font.pointSize: theme.typePage * 1.6; color: theme.accent }
                    Label { msg: "stats-recap-title"; role: "title" }
                }
                Grid {
                    id: recapFigures
                    anchors.top: recapHead.bottom; anchors.topMargin: theme.space(6)
                    columns: 2; columnSpacing: theme.space(10); rowSpacing: theme.space(4)
                    Repeater {
                        model: page.stats ? [[String(page.hours(page.stats.minutes)), words.t("stats-recap-hours")], [String(page.stats.episodes), words.t("stats-recap-episodes")],
                                             [String(page.stats.series_completed), words.t("stats-series-finished")],
                                             [page.stats.genres.length ? page.stats.genres[0].name : words.t("stats-none"), words.t("stats-recap-top-genre")]] : []
                        Column {
                            required property var modelData
                            Label { text: modelData[0]; role: "display"; font.pointSize: theme.typeHeading }
                            Label { text: modelData[1]; role: "ui"; color: theme.textDim }
                        }
                    }
                }
                Row {
                    anchors.right: parent.right; anchors.bottom: parent.bottom
                    spacing: theme.space(3)
                    Repeater {
                        model: page.stats ? page.stats.top_series.slice(0, 3) : []
                        Column {
                            required property var modelData
                            spacing: theme.space(1)
                            readonly property real w: recap.width * 0.15
                            Rectangle {
                                width: parent.w; height: parent.w * 1.5
                                radius: theme.radiusSm; color: theme.raised; clip: true
                                Image {
                                    anchors.fill: parent
                                    source: modelData.poster ? Fmt.fileUrl(modelData.poster) : ""
                                    fillMode: Image.PreserveAspectCrop
                                    asynchronous: false
                                    smooth: true
                                }
                            }
                            Label { width: parent.w; text: modelData.title; role: "ui"; elide: Text.ElideRight }
                            Label { width: parent.w; msg: "count-episodes"; msgArgs: ({ count: modelData.episodes }); role: "ledger"; color: theme.textDim }
                        }
                    }
                }
                Label { anchors.left: parent.left; anchors.bottom: parent.bottom; text: "AniBeam"; role: "ledger"; color: theme.textFaint }
            }
            Button {
                id: saveRecapButton
                objectName: "stats-save-recap"
                anchors.left: recap.right; anchors.leftMargin: theme.space(4)
                anchors.top: recap.top
                msg: "stats-save-image"; icon: "image"
                onClicked: saveDialog.open()
            }
        }
    }
    FileDialog {
        id: saveDialog
        title: words.t("stats-save-dialog-title")
        fileMode: FileDialog.SaveFile
        defaultSuffix: "png"
        nameFilters: [words.t("stats-save-filter-png")]
        currentFolder: StandardPaths.writableLocation(StandardPaths.PicturesLocation)
        currentFile: currentFolder + "/AniBeam " + (page.stats ? page.stats.year : "") + ".png"
        onAccepted: page.saveRecap(Fmt.localPath(String(selectedFile)))
    }
    // The card at twice its size on screen, so the PNG holds up when shared
    function saveRecap(path) {
        recap.grabToImage(function(result) {
            frame.toast(result.saveToFile(path) ? words.t("stats-recap-saved") : words.t("stats-recap-save-failed"))
        }, Qt.size(recap.width * 2, recap.height * 2))
    }
}
