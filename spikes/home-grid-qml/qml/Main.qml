// Ticket #17: the home grid inside the spike window with the theme knobs live, and the
// second round's sketch: page switching, the settings page, the status strip and the
// activity drawer. One attempt to react to; the owner steers from here. Not the shell.
import QtQuick
import QtQuick.Controls.Basic
import dev.anibeam.proto

ApplicationWindow {
    id: window
    width: 1600
    height: 1000
    visible: true
    title: "AniBeam prototype: home grid"
    color: theme.bg

    Theme {
        id: theme
        palettes: JSON.parse(Proto.palettesJson)
    }

    property var library: JSON.parse(Proto.libraryJson)
    property string titleLang: "jp"
    property int tab: 0                // All, Series, Movies
    property int sortKey: 0            // A to Z, Last viewed, Progress, Score, My score
    property bool descending: false
    property string query: ""
    property real nowMs: Date.now()
    Timer { interval: 30000; running: true; repeat: true; onTriggered: window.nowMs = Date.now() }

    readonly property var pageNames: ["Library", "Feed", "Watching", "Metadata", "Settings"]
    readonly property int page: rail.active

    // Fake activity: the strip shows the newest line, the drawer the whole log
    property int unseenErrors: 2
    property bool jobRunning: false
    readonly property var activity: [
        { time: "12:04:11", stage: "scan", level: "info", msg: "3 new episodes in Frieren" },
        { time: "12:04:09", stage: "scan", level: "info", msg: "Scan of Anime finished, 412 series" },
        { time: "12:03:58", stage: "match", level: "info", msg: "Matched Sousou no Frieren to AniList 154587" },
        { time: "12:03:57", stage: "image", level: "info", msg: "Poster fetched for Sousou no Frieren" },
        { time: "12:03:41", stage: "tracker", level: "error", msg: "MyAnimeList rejected the mark for Dungeon Meshi 12: token expired" },
        { time: "12:03:40", stage: "tracker", level: "warn", msg: "Retrying MyAnimeList after 429" },
        { time: "12:03:35", stage: "tracker", level: "warn", msg: "Retrying MyAnimeList after 429" },
        { time: "12:03:30", stage: "tracker", level: "warn", msg: "Retrying MyAnimeList after 429" },
        { time: "12:02:12", stage: "tracker", level: "info", msg: "Marked Dungeon Meshi 12 on AniList" },
        { time: "12:02:12", stage: "play", level: "info", msg: "Reached the outro of Dungeon Meshi 12" },
        { time: "11:41:03", stage: "play", level: "info", msg: "Playing Dungeon Meshi 12, ja audio, en subtitles" },
        { time: "11:12:30", stage: "image", level: "error", msg: "Poster download failed for Kusuriya no Hitorigoto: 404" },
        { time: "11:12:28", stage: "match", level: "warn", msg: "No confident match for Films/Perfect Blue, best was 0.41" },
        { time: "11:12:20", stage: "match", level: "info", msg: "Matched 2 folders from the scan" },
        { time: "11:12:04", stage: "scan", level: "info", msg: "2 new series in Anime" },
        { time: "11:12:01", stage: "scan", level: "warn", msg: "Films is unavailable, skipped" },
        { time: "11:11:58", stage: "scan", level: "info", msg: "Scanning Anime" },
        { time: "11:11:57", stage: "system", level: "info", msg: "Config reloaded, theme.toml changed" },
        { time: "09:30:12", stage: "export", level: "info", msg: "Wrote anibeam-export-2026-09-04.json, 1.2 MB" },
        { time: "09:30:10", stage: "export", level: "info", msg: "Export started, library only" },
        { time: "09:02:41", stage: "system", level: "info", msg: "AniBeam 2.0.0-proto started" },
        { time: "09:02:40", stage: "system", level: "info", msg: "Database opened, 16 tables" }
    ]

    function displayTitle(i) {
        if (titleLang === "en") return i.titleEnglish || i.titleRomaji || i.folderName || ""
        return i.titleRomaji || i.titleEnglish || i.folderName || ""
    }
    function progressOf(i) {
        if (i.watched === null || i.watched === undefined || !i.total) return null
        var p = i.watched / i.total
        return (p <= 0 || p >= 1) ? null : p
    }
    readonly property var visibleItems: {
        var q = query.trim().toLowerCase()
        var out = library.filter(function(i) {
            if (tab === 1 && i.isMovie) return false
            if (tab === 2 && !i.isMovie) return false
            if (q === "") return true
            return [i.titleRomaji, i.titleEnglish, i.matchedTitle, i.folderName].some(function(t) { return t && t.toLowerCase().indexOf(q) >= 0 })
        })
        var key = sortKey
        function val(i) {
            if (key === 1) return i.lastViewedAt || null
            if (key === 2) return progressOf(i)
            if (key === 3) return (i.score === null || i.score === undefined) ? null : i.score
            if (key === 4) return (i.myScore === null || i.myScore === undefined) ? null : i.myScore
            return null
        }
        out.sort(function(a, b) {
            var ta = displayTitle(a).toLowerCase(), tb = displayTitle(b).toLowerCase()
            if (key === 0) return descending ? tb.localeCompare(ta) : ta.localeCompare(tb)
            var va = val(a), vb = val(b)
            if (va === null && vb === null) return ta.localeCompare(tb)
            if (va === null) return 1
            if (vb === null) return -1
            if (va !== vb) return descending ? vb - va : va - vb
            return ta.localeCompare(tb)
        })
        return out
    }

    font.family: theme.fontSans
    font.pointSize: theme.typeNormal

    Rail {
        id: rail
        anchors.left: parent.left
        anchors.top: parent.top
        anchors.bottom: parent.bottom
        titleLang: window.titleLang
        onLangPicked: function(l) { window.titleLang = l }
    }

    // Everything right of the rail: the page, the drawer rising over it, the strip at the foot
    Item {
        id: content
        anchors.left: rail.right
        anchors.right: parent.right
        anchors.top: parent.top
        anchors.bottom: parent.bottom

        Item {
            id: pages
            anchors.left: parent.left
            anchors.right: parent.right
            anchors.top: parent.top
            anchors.bottom: strip.top
            anchors.leftMargin: theme.space(8)
            anchors.rightMargin: theme.space(8)

            Item {
                id: libraryPage
                anchors.fill: parent
                visible: window.page === 0

                Column {
                    id: header
                    anchors.top: parent.top
                    anchors.topMargin: theme.space(7)
                    width: parent.width
                    spacing: theme.space(4)

                    Row {
                        spacing: theme.space(3)
                        Text {
                            text: "Library"
                            color: theme.text
                            font.family: theme.fontSans
                            font.pointSize: theme.typeLarge
                            font.weight: Font.Bold
                            anchors.verticalCenter: parent.verticalCenter
                        }
                        Chip {
                            anchors.verticalCenter: parent.verticalCenter
                            text: window.visibleItems.length + (window.tab === 2 ? " films" : " series")
                            small: true
                            color: theme.surface
                            textColor: theme.textDim
                        }
                    }

                    // Search pill
                    Corner {
                        width: Math.min(parent.width, theme.space(120))
                        height: theme.controlHeight
                        radius: height / 2
                        smoothing: theme.cornerSmoothing
                        color: theme.surfaceSunken
                        borderColor: search.activeFocus ? theme.focusRing : theme.line
                        borderWidth: 1
                        TextInput {
                            id: search
                            anchors.fill: parent
                            anchors.leftMargin: theme.space(4)
                            anchors.rightMargin: theme.space(4)
                            verticalAlignment: TextInput.AlignVCenter
                            color: theme.text
                            font.family: theme.fontSans
                            font.pointSize: theme.typeNormal
                            selectionColor: theme.accentSoft
                            selectedTextColor: theme.text
                            clip: true
                            onTextChanged: window.query = text
                            Keys.onEscapePressed: { text = ""; focus = false }
                            Text {
                                anchors.fill: parent
                                verticalAlignment: Text.AlignVCenter
                                visible: !search.text
                                text: "Search romaji, english or folder"
                                color: theme.textFaint
                                font: search.font
                            }
                        }
                        Text {
                            anchors.right: parent.right
                            anchors.rightMargin: theme.space(4)
                            anchors.verticalCenter: parent.verticalCenter
                            visible: !search.activeFocus
                            text: "/  Ctrl K"
                            color: theme.textFaint
                            font.family: theme.fontMono
                            font.pointSize: theme.typeSmall
                        }
                    }

                    Row {
                        width: parent.width
                        spacing: theme.space(3)
                        Seg {
                            options: ["All", "Series", "Movies"]
                            index: window.tab
                            onPicked: function(i) { window.tab = i }
                        }
                        Item { width: theme.space(2); height: 1 }
                        Repeater {
                            model: ["A to Z", "Last viewed", "Progress", "Score", "My score"]
                            Chip {
                                anchors.verticalCenter: parent.verticalCenter
                                text: modelData
                                mono: false
                                clickable: true
                                selected: window.sortKey === index
                                color: selected ? theme.accentSoft : theme.surface
                                textColor: theme.textDim
                                onClicked: { window.sortKey = index; window.descending = index !== 0 }
                            }
                        }
                        Chip {
                            anchors.verticalCenter: parent.verticalCenter
                            text: window.descending ? "Desc" : "Asc"
                            clickable: true
                            color: theme.surface
                            textColor: theme.textDim
                            onClicked: window.descending = !window.descending
                        }
                    }
                }

                GridView {
                    id: grid
                    anchors.top: header.bottom
                    anchors.topMargin: theme.space(6)
                    anchors.left: parent.left
                    anchors.right: parent.right
                    anchors.bottom: parent.bottom
                    clip: true
                    readonly property real gapX: theme.space(5)
                    readonly property real gapY: theme.space(6)
                    readonly property int columns: Math.max(1, Math.floor((width + gapX) / (theme.posterWidth + gapX)))
                    cellWidth: Math.floor((width + gapX) / columns)
                    readonly property real cardWidth: cellWidth - gapX
                    cellHeight: Math.ceil(cardWidth * 1.5 + theme.space(2) + theme.typeNormal * 2 * 1.5 + theme.typeSmall * 1.5 + theme.space(1)) + gapY
                    model: window.visibleItems
                    cacheBuffer: 1200
                    delegate: Item {
                        width: grid.cellWidth
                        height: grid.cellHeight
                        Card {
                            item: modelData
                            posterWidth: grid.cardWidth
                            titleLang: window.titleLang
                            nowMs: window.nowMs
                        }
                    }
                    footer: Item { width: 1; height: window.footInset }
                    ScrollBar.vertical: ScrollBar {
                        policy: ScrollBar.AsNeeded
                        contentItem: Rectangle { implicitWidth: 4; radius: 2; color: theme.lineStrong; opacity: parent.active ? 1 : 0.4 }
                    }
                }
            }

            SettingsPage {
                id: settingsPage
                anchors.fill: parent
                visible: window.page === 4
                footInset: window.footInset
                library: window.library
                titleLang: window.titleLang
            }

            // Feed, Watching and Metadata: a title and a line, nothing else in this round
            Column {
                anchors.top: parent.top
                anchors.topMargin: theme.space(7)
                width: parent.width
                spacing: theme.space(2)
                visible: window.page > 0 && window.page < 4
                Text {
                    text: window.pageNames[window.page]
                    color: theme.text
                    font.family: theme.fontSans
                    font.pointSize: theme.typeLarge
                    font.weight: Font.Bold
                }
                Text {
                    text: "Not in this prototype"
                    color: theme.textDim
                    font.family: theme.fontSans
                    font.pointSize: theme.typeNormal
                }
            }
        }

        ActivityDrawer {
            id: drawer
            anchors.left: parent.left
            anchors.right: parent.right
            anchors.bottom: strip.top
            maxHeight: Math.round(pages.height * 0.6)
            entries: window.activity
            onOpenChanged: if (open) window.unseenErrors = 0
        }

        StatusStrip {
            id: strip
            anchors.left: parent.left
            anchors.right: parent.right
            anchors.bottom: parent.bottom
            stage: window.jobRunning ? "scan" : (window.activity.length ? window.activity[0].stage : "system")
            message: window.jobRunning ? "Scanning Anime · 43 of 120" : (window.activity.length ? window.activity[0].msg : "")
            time: window.jobRunning ? "" : (window.activity.length ? window.activity[0].time.slice(0, 5) : "")
            running: window.jobRunning
            fraction: 43 / 120
            unseenErrors: window.unseenErrors
            onClicked: drawer.toggle()
        }
    }

    // Room a scrolling page leaves at its foot so the knob bar never covers the last row
    readonly property real footInset: knobBar.visible ? knobBar.height + theme.space(10) : theme.space(10)

    KnobBar {
        id: knobBar
        maxWidth: window.width - 32
        anchors.horizontalCenter: parent.horizontalCenter
        anchors.bottom: parent.bottom
        anchors.bottomMargin: 16
    }

    // --preset mode=light,source=theme,dark=gruvbox-dark-medium,light=catppuccin-latte,density=compact,
    //          poster=240,smoothing=0,base=10,accent=6,lang=en,knobs=0,sort=1,tab=1,page=settings:look,
    //          drawer=open,job=1,scroll=1400,confirm=1
    // page=settings opens the Library tab; settings:look, settings:playback and settings:data the
    // others. scroll and confirm apply last, to whichever settings tab the preset chose.
    Component.onCompleted: {
        var args = Qt.application.arguments
        var at = args.indexOf("--preset")
        if (at < 0 || at + 1 >= args.length) return
        var late = {}
        args[at + 1].split(",").forEach(function(kv) {
            var k = kv.split("=")[0], v = kv.split("=")[1]
            if (k === "mode") theme.mode = v
            else if (k === "source") theme.colourSource = v
            else if (k === "dark") theme.themeDark = v
            else if (k === "light") theme.themeLight = v
            else if (k === "density") theme.density = v
            else if (k === "poster") theme.posterWidth = parseInt(v)
            else if (k === "smoothing") theme.cornerSmoothing = parseFloat(v)
            else if (k === "base") theme.cornerBase = parseFloat(v)
            else if (k === "accent") theme.accentSlot = parseInt(v)
            else if (k === "lang") window.titleLang = v
            else if (k === "knobs") knobBar.visible = v !== "0"
            else if (k === "sort") { window.sortKey = parseInt(v); window.descending = parseInt(v) !== 0 }
            else if (k === "tab") window.tab = parseInt(v)
            else if (k === "page") {
                var name = v.split(":")[0], sub = v.split(":")[1]
                var p = window.pageNames.map(function(n) { return n.toLowerCase() }).indexOf(name)
                if (p >= 0) rail.active = p
                if (p === 4 && sub) { var t = settingsPage.tabNames.map(function(n) { return n.toLowerCase() }).indexOf(sub); if (t >= 0) settingsPage.tab = t }
            }
            else if (k === "drawer") { drawer.open = v === "open" }
            else if (k === "job") window.jobRunning = v !== "0"
            else if (k === "scroll" || k === "confirm") late[k] = v
        })
        if (late.scroll !== undefined) settingsPage.scrollY = parseInt(late.scroll)
        if (late.confirm !== undefined) settingsPage.demoConfirm = late.confirm !== "0"
    }

    Shortcut { sequence: "H"; onActivated: knobBar.visible = !knobBar.visible }
    Shortcut { sequence: "Ctrl+K"; onActivated: { rail.active = 0; search.forceActiveFocus() } }
    Shortcut { sequence: "/"; enabled: window.page === 0; onActivated: search.forceActiveFocus() }
    Shortcut { sequence: "Ctrl+,"; onActivated: rail.active = 4 }
    Shortcut { sequence: "Ctrl+L"; onActivated: drawer.toggle() }
    Shortcut { sequence: "Ctrl+R"; onActivated: { Proto.reload(); window.library = JSON.parse(Proto.libraryJson); theme.palettes = JSON.parse(Proto.palettesJson) } }
    Shortcut { sequence: "Ctrl+Q"; onActivated: Qt.quit() }
}
