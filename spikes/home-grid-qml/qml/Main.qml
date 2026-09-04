// Ticket #17: the home grid inside the spike window with the theme knobs live.
// One attempt to react to; the owner steers from here. Not the shell.
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

    Item {
        id: page
        anchors.left: rail.right
        anchors.right: parent.right
        anchors.top: parent.top
        anchors.bottom: parent.bottom
        anchors.leftMargin: theme.space(8)
        anchors.rightMargin: theme.space(8)

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
            footer: Item { width: 1; height: knobBar.visible ? knobBar.height + theme.space(10) : theme.space(10) }
            ScrollBar.vertical: ScrollBar {
                policy: ScrollBar.AsNeeded
                contentItem: Rectangle { implicitWidth: 4; radius: 2; color: theme.lineStrong; opacity: parent.active ? 1 : 0.4 }
            }
        }
    }

    KnobBar {
        id: knobBar
        maxWidth: window.width - 32
        anchors.horizontalCenter: parent.horizontalCenter
        anchors.bottom: parent.bottom
        anchors.bottomMargin: 16
    }

    // --preset mode=light,source=theme,dark=gruvbox-dark-medium,light=catppuccin-latte,density=compact,
    //          poster=240,smoothing=0,base=10,accent=6,lang=en,knobs=0,sort=1,tab=1
    Component.onCompleted: {
        var args = Qt.application.arguments
        var at = args.indexOf("--preset")
        if (at < 0 || at + 1 >= args.length) return
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
        })
    }

    Shortcut { sequence: "H"; onActivated: knobBar.visible = !knobBar.visible }
    Shortcut { sequence: "Ctrl+K"; onActivated: search.forceActiveFocus() }
    Shortcut { sequence: "/"; onActivated: search.forceActiveFocus() }
    Shortcut { sequence: "Ctrl+R"; onActivated: { Proto.reload(); window.library = JSON.parse(Proto.libraryJson); theme.palettes = JSON.parse(Proto.palettesJson) } }
    Shortcut { sequence: "Ctrl+Q"; onActivated: Qt.quit() }
}
