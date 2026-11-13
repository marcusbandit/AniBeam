// The first run: a fresh install opens here instead of on an empty library. Five steps, one
// idea each: what AniBeam is, the folder the anime lives in, the tracker, how Downloads gets
// its torrent client, and a closing screen that says what got set up and offers each skipped
// step again. It takes the whole window, the rail included, as two panes of the mosaic: the
// steps at the left, the step itself at the right.
//
// Every step can be skipped, and so can the whole thing. It shows once: `done` lives in
// shell.conf. Whether a start opens it is the core's rule (first_run): an install that ran
// before, has a library or a tracker, or kept a version of read notes is not a fresh one, so
// an update never lands here. Settings, About runs it again.
//
// The folder step adds the source and shows the scan's real result as it comes in, posters
// included. Someone with no folder yet gets one made: Anime in the platform's videos folder.
// The tracker step is the same TrackerRow the Accounts tab uses, so the sign-in is the
// frame's one flow and the connect sheet rises over this when the build has no credentials
// of its own. Each step that asks for something first shows what it gives.
//
// White is the theme's accent and the primary button's fill, so a done step or a chosen card
// is marked in the soft accent with an accent edge, never a solid white disc: the one solid
// white thing on a screen is the button that moves on.
import QtQuick
import QtQuick.Dialogs
import QtCore
import com.marcusrosado.AniBeam

Item {
    id: root
    objectName: "onboarding"
    anchors.fill: parent
    visible: false

    Settings {
        id: kept
        location: Shell.settingsUrl
        category: "onboarding"
        property bool done: false
    }
    // The What's new record, read only: a version kept there means this install has run before
    Settings {
        id: notes
        location: Shell.settingsUrl
        category: "whatsNew"
        property string version: ""
    }

    readonly property var steps: [
        { key: "welcome", label: words.t("onboarding-step-welcome"), icon: "sparkles" },
        { key: "library", label: words.t("onboarding-step-library"), icon: "folder-open" },
        { key: "lists", label: words.t("onboarding-step-lists"), icon: "user-check" },
        { key: "downloads", label: words.t("onboarding-step-downloads"), icon: "download" },
        { key: "ready", label: words.t("onboarding-step-ready"), icon: "circle-check" }
    ]
    property int step: 0
    // Steps left with Skip, by key
    property var skipped: ({})

    property var sources: []
    // The folder made for someone with none, so its empty scan reads as a new folder
    property string starterPath: ""
    // Search endpoints installed from a repo; Downloads finds nothing without one
    property int endpointCount: 0
    // The series found so far, a title and a poster path (empty until one is fetched) each
    property var shelfCards: []
    property int seriesCount: 0
    readonly property int posterCount: shelfCards.filter(function(c) { return c.poster !== "" }).length
    // "Auto" lets AniBeam pick (a qBittorrent that answers, else its own); "Existing" is the
    // user's own, finished in Settings, Downloads once this closes
    property string torrentMode: "Auto"
    // "Add a repo after setup": finishing opens Settings, Downloads at the repo field
    property bool wantsSources: false

    readonly property var anilist: Door.trackers.anilist || ({})
    readonly property var mal: Door.trackers.mal || ({})
    readonly property bool tracked: anilist.connected === true || mal.connected === true
    readonly property var scanJob: {
        var jobs = Door.runningJobs
        for (var i = 0; i < jobs.length; i++) if (/^(Scan|AutoMatch|FillImages|Crawl)$/.test(String(jobs[i].kind))) return jobs[i]
        return null
    }
    // Between the scan and the lookup there is a beat with no job at all; "Found" there would
    // read as finished and then flip back to working, so the first seconds after a folder is
    // added still read as reading it.
    Timer { id: settling; interval: 8000 }
    readonly property string scanLine: {
        var j = scanJob
        // With series in, each working line counts the posters found so far
        var tally = seriesCount > 0, args = { posters: posterCount, count: seriesCount }
        if (!j && settling.running) return words.t("onboarding-scan-reading")
        if (!j) return seriesCount > 0 ? words.t("onboarding-scan-found", { count: seriesCount })
                     : starterOnly ? words.t("onboarding-scan-starter-empty")
                     : words.t("onboarding-scan-no-video")
        if (j.kind === "Scan") return words.t("onboarding-scan-reading")
        if (j.kind === "AutoMatch") {
            args.what = j.label || words.t("onboarding-scan-your-shows")
            return words.t(tally ? "onboarding-scan-looking-up-tally" : "onboarding-scan-looking-up", args)
        }
        if (j.kind === "FillImages") return words.t(tally ? "onboarding-scan-posters-tally" : "onboarding-scan-posters", args)
        return words.t(tally ? "onboarding-scan-seasons-tally" : "onboarding-scan-seasons", args)
    }

    // As the app starts. A capture or a drive never opens it on its own; a drive calls open().
    function start() {
        var s = Shell.firstRun(kept.done, notes.version, hasLibrary(), tracked)
        if (s === "settle") kept.done = true
        else if (s === "open" && Shell.shoot === "" && Shell.driveOut === "") open()
    }
    readonly property bool starterOnly: starterPath !== "" && sources.length === 1 && sources[0].path === starterPath
    function hasLibrary() { var r = Door.listSources(); return !r.error && r.reply.sources.length > 0 }
    function open(at) {
        step = at || 0
        skipped = ({})
        wantsSources = false
        reload()
        var d = Door.getDownloads()
        if (!d.error) torrentMode = d.reply.status.settings.torrent_mode === "Existing" ? "Existing" : "Auto"
        visible = true
        frame.escapeStack.push("onboarding", root)
        body.forceActiveFocus()
    }
    // Escape asks for this through the escape stack. The steps are all skippable, but a stray
    // Escape must not throw away a first run, so it only steps back.
    function close() { if (step > 0) step-- }
    function finish() {
        kept.done = true
        frame.escapeStack.pop(root)
        visible = false
        if (torrentMode === "Existing" || wantsSources) frame.go("settings", { tab: "downloads" })
        else if (sources.length > 0 && !starterOnly) frame.go("library")
        frame.focusPage(root)
    }
    function next() { if (step < steps.length - 1) step++; else finish() }
    function skip() {
        var s = Object.assign({}, skipped)
        s[steps[step].key] = true
        skipped = s
        next()
    }

    function reload() {
        var r = Door.listSources()
        if (!r.error) sources = r.reply.sources
        var e = Door.call("ListRepoEndpoints", {})
        if (e && e.kind === "EndpointCatalog") endpointCount = e.reply.catalog.installed.length
        var l = Door.listLibrary("All", "", "Alpha", "Asc", false)
        if (l.error) return
        seriesCount = l.reply.series.length
        // Those with a poster first, so the shelf fills with pictures as they arrive
        var cards = l.reply.series.slice()
        cards.sort(function(a, b) { return (b.poster ? 1 : 0) - (a.poster ? 1 : 0) })
        shelfCards = cards.slice(0, 40).map(function(c) { return { title: c.title, poster: c.poster || "" } })
    }
    Timer { id: debounce; interval: 400; onTriggered: root.reload() }
    Connections {
        target: Door
        enabled: root.visible
        function onSourceChanged(s) { debounce.restart() }
        function onScanFinished(s, a, c, r) { debounce.restart() }
        function onSeriesChanged(c) { debounce.restart() }
    }
    function addFolder(path) {
        var r = Door.addSource(path)
        if (r.error) { frame.fail(r.error); return false }
        settling.restart()
        reload()
        return true
    }
    // "I don't have a folder yet": Anime in the videos folder, made if need be, and added
    // Not readonly, so a drive can point it at its sandbox
    property string videosDir: Fmt.localPath(String(StandardPaths.writableLocation(StandardPaths.MoviesLocation)))
    function makeStarter() {
        var r = Shell.makeStarterFolder(videosDir)
        if (r.error) { frame.toast(r.error); return }
        if (addFolder(r.path)) starterPath = r.path
    }
    function saveTorrentMode() {
        var d = Door.getDownloads()
        if (d.error) return
        // Written on every pick: the status read here is the last one the core sent, which
        // still holds the previous pick for a moment after a write, so comparing with it
        // would skip the write that takes that pick back. Managed stays Managed under Auto.
        var s = JSON.parse(JSON.stringify(d.reply.status.settings))
        s.torrent_mode = torrentMode === "Existing" ? "Existing" : s.torrent_mode === "Managed" ? "Managed" : "Auto"
        var r = Door.setDownloadSettings(s, {})
        if (r.error) frame.fail(r.error)
    }
    FolderDialog {
        id: folderDialog
        title: words.t("onboarding-choose-folder")
        currentFolder: StandardPaths.writableLocation(StandardPaths.MoviesLocation)
        onAccepted: root.addFolder(Fmt.localPath(String(selectedFolder)))
    }

    // A step's mark, the same everywhere: a disc, the soft accent with its edge when done or
    // chosen, a plain edge otherwise
    component Badge: Rectangle {
        property string glyph: ""
        property bool on: false
        property real box: theme.space(12)
        width: box; height: box; radius: box / 2
        color: on ? theme.accentSoft : "transparent"
        border.width: 1
        border.color: on ? theme.accentEdge : theme.lineStrong
        Icon { anchors.centerIn: parent; glyph: parent.glyph; size: parent.box / 2; color: parent.on ? theme.accent : theme.textDim }
    }

    // The ground: the window's own colour, so nothing of the empty frame shows behind
    Rectangle { anchors.fill: parent; color: theme.bg }
    Blocker { anchors.fill: parent; scrim: false }

    // ---- The steps
    Tile {
        id: side
        objectName: "onboarding-steps"
        x: theme.grout; y: theme.grout
        width: Math.round(Math.max(theme.space(64), Math.min(parent.width * 0.24, theme.space(84))))
        height: parent.height - theme.grout * 2
        padding: theme.space(7)

        Label { id: mark; text: "AniBeam"; role: "display"; font.pointSize: theme.typeHeading * 1.15 }
        Label {
            id: about
            anchors.top: mark.bottom; anchors.topMargin: theme.space(1)
            width: parent.width
            msg: "onboarding-about"
            role: "ui"; color: theme.textDim
            wrapMode: Text.Wrap
        }
        Column {
            anchors.top: about.bottom; anchors.topMargin: theme.space(10)
            width: parent.width
            spacing: theme.space(1)
            Repeater {
                model: root.steps
                Item {
                    id: row
                    required property var modelData
                    required property int index
                    readonly property bool current: index === root.step
                    readonly property bool passed: index < root.step
                    readonly property bool wasSkipped: root.skipped[modelData.key] === true
                    width: parent.width
                    height: theme.space(12)
                    Rectangle {
                        anchors.fill: parent
                        radius: theme.radiusMd
                        color: row.current ? theme.raised : rowMouse.containsMouse && row.passed ? theme.raised : "transparent"
                        border.width: row.current ? 1 : 0
                        border.color: theme.lineStrong
                    }
                    Badge {
                        id: dot
                        x: theme.space(3)
                        anchors.verticalCenter: parent.verticalCenter
                        box: theme.space(7)
                        on: row.passed && !row.wasSkipped
                        glyph: row.passed && !row.wasSkipped ? "check" : row.modelData.icon
                    }
                    Label {
                        anchors.left: dot.right; anchors.leftMargin: theme.space(3)
                        anchors.verticalCenter: parent.verticalCenter
                        text: row.modelData.label
                        role: "ui"
                        font.weight: row.current ? Font.DemiBold : Font.Normal
                        color: row.current ? theme.text : theme.textDim
                    }
                    Label {
                        anchors.right: parent.right; anchors.rightMargin: theme.space(3)
                        anchors.verticalCenter: parent.verticalCenter
                        visible: row.wasSkipped
                        msg: "onboarding-step-later"
                        role: "ui"; font.pointSize: theme.typeSmall; color: theme.textDim
                    }
                    // A step already passed can be opened again
                    MouseArea {
                        id: rowMouse
                        Accessible.role: Accessible.Button
                        Accessible.name: theme.nameNear(parent)
                        Accessible.onPressAction: rowMouse.clicked(null)
                        anchors.fill: parent
                        enabled: row.passed
                        hoverEnabled: true
                        cursorShape: row.passed ? Qt.PointingHandCursor : Qt.ArrowCursor
                        onClicked: root.step = row.index
                    }
                }
            }
        }
        Button {
            objectName: "onboarding-skip-all"
            anchors.bottom: parent.bottom
            visible: root.step < root.steps.length - 1
            msg: "onboarding-skip-setup"
            flat: true
            onClicked: root.finish()
        }
    }

    // ---- The step
    Tile {
        id: main
        objectName: "onboarding-step"
        anchors.left: side.right; anchors.leftMargin: theme.grout
        anchors.right: parent.right; anchors.rightMargin: theme.grout
        y: theme.grout
        height: parent.height - theme.grout * 2
        padding: theme.space(12)

        FocusScope {
            id: body
            anchors.fill: parent
            Keys.onReturnPressed: if (cont.enabled) root.next()
            Keys.onEnterPressed: if (cont.enabled) root.next()

            // The library step's shelf grows into the whole stage; every other step is one
            // block, head over pane, centred in it, so a short step is not a heading with a
            // hole under it
            readonly property bool fills: root.step === 1
            Item {
                id: stage
                width: parent.width
                anchors.top: parent.top
                anchors.bottom: foot.top; anchors.bottomMargin: theme.space(8)

                Column {
                    id: block
                    width: parent.width
                    y: body.fills ? 0 : Math.max(0, Math.round((stage.height - height) / 2))
                    spacing: theme.space(10)
                    Column {
                        id: head
                        width: Math.min(parent.width, theme.space(190))
                        spacing: theme.space(3)
                        Label {
                            objectName: "onboarding-title"
                            width: parent.width
                            text: [
                                words.t("onboarding-title-welcome"),
                                words.t("onboarding-title-library"),
                                words.t("onboarding-title-lists"),
                                words.t("onboarding-title-downloads"),
                                root.laterCount === 0 ? words.t("onboarding-title-ready") : words.t("onboarding-title-later", { count: root.laterCount })
                            ][root.step]
                            role: "display"
                            font.pointSize: theme.typeHeading * 1.6
                            wrapMode: Text.Wrap
                        }
                        Label {
                            width: parent.width
                            text: [
                                words.t("onboarding-body-welcome"),
                                words.t("onboarding-body-library"),
                                words.t("onboarding-body-lists"),
                                words.t("onboarding-body-downloads"),
                                words.t("onboarding-body-ready")
                            ][root.step]
                            role: "ui"
                            font.pointSize: theme.typeLarge
                            color: theme.textDim
                            wrapMode: Text.Wrap
                            lineHeight: 1.15
                        }
                    }
                    Loader {
                        id: pane
                        width: parent.width
                        height: body.fills ? stage.height - head.height - block.spacing : (item ? item.implicitHeight : 0)
                        sourceComponent: [welcomePane, libraryPane, listsPane, downloadsPane, readyPane][root.step]
                    }
                }
            }

            // Back beside the button it undoes, Skip quieter than both
            Row {
                id: foot
                anchors.bottom: parent.bottom
                anchors.right: parent.right
                spacing: theme.space(3)
                Button {
                    objectName: "onboarding-skip"
                    visible: root.skippable
                    msg: "onboarding-skip-step"; flat: true
                    onClicked: root.skip()
                }
                Button {
                    objectName: "onboarding-back"
                    visible: root.step > 0
                    msg: "action-back"; icon: "arrow-left"
                    onClicked: root.step--
                }
                Button {
                    id: cont
                    objectName: "onboarding-continue"
                    primary: true
                    icon: root.step === root.steps.length - 1 ? "circle-play" : "arrow-right"
                    text: root.step === 0 ? words.t("onboarding-get-started") : root.step === root.steps.length - 1 ? words.t("onboarding-open-app") : words.t("onboarding-continue")
                    enabled: root.step !== 1 || root.sources.length > 0
                    onClicked: root.next()
                }
            }
        }
    }
    // Skip stands beside Continue only where the step asks for something not yet given
    readonly property bool skippable: (step === 1 && sources.length === 0) || (step === 2 && !tracked)
    readonly property bool downloadsReady: torrentMode !== "Existing" && endpointCount > 0
    readonly property int laterCount: (sources.length === 0 ? 1 : 0) + (tracked ? 0 : 1) + (downloadsReady ? 0 : 1)
    // Leaves the walkthrough for a Settings section, for a thing it cannot finish itself
    function finishAt(tab) {
        kept.done = true
        frame.escapeStack.pop(root)
        visible = false
        frame.go("settings", { tab: tab })
        frame.focusPage(root)
    }

    // ---- Welcome: what the app does, before it asks for anything
    Component {
        id: welcomePane
        Column {
        spacing: theme.space(6)
        // The language first: everything after it reads in the one picked. It starts on the
        // system's, and Settings, Appearance changes it later.
        Row {
            objectName: "onboarding-language"
            spacing: theme.space(3)
            Icon { anchors.verticalCenter: parent.verticalCenter; glyph: "languages"; color: theme.textDim }
            Label { anchors.verticalCenter: parent.verticalCenter; msg: "onboarding-language"; color: theme.textDim }
            Dropdown {
                anchors.verticalCenter: parent.verticalCenter
                readonly property var codes: [""].concat(words.languages.map(function(l) { return l.code }))
                options: [words.t("settings-appearance-language-system", { language: root.languageName(words.systemCode) })]
                         .concat(words.languages.map(function(l) { return l.name }))
                index: Math.max(0, codes.indexOf(words.picked))
                onPicked: function(i) { words.pick(codes[i]) }
            }
        }
        Grid {
            id: grid
            width: parent.width
            columns: width > theme.space(200) ? 4 : 2
            spacing: theme.grout
            readonly property real cell: (width - spacing * (columns - 1)) / columns
            // One height for the four, the tallest one's
            property real tallest: 0
            Repeater {
                model: [
                    { icon: "folder-open", title: words.t("onboarding-payoff-library"), line: words.t("onboarding-payoff-library-line") },
                    { icon: "circle-play", title: words.t("onboarding-payoff-plays"), line: words.t("onboarding-payoff-plays-line") },
                    { icon: "user-check", title: words.t("onboarding-payoff-list"), line: words.t("onboarding-payoff-list-line") },
                    { icon: "download", title: words.t("onboarding-payoff-episodes"), line: words.t("onboarding-payoff-episodes-line") }
                ]
                Tile {
                    required property var modelData
                    width: grid.cell
                    readonly property real need: tileWords.y + tileWords.height + padding * 2
                    onNeedChanged: grid.tallest = Math.max(grid.tallest, need)
                    height: Math.max(need, grid.tallest)
                    padding: theme.space(7)
                    Badge { id: badge; glyph: modelData.icon }
                    Column {
                        id: tileWords
                        anchors.top: badge.bottom; anchors.topMargin: theme.space(5)
                        width: parent.width
                        spacing: theme.space(2)
                        Label { width: parent.width; text: modelData.title; role: "title"; font.pointSize: theme.typeLarge * 1.2; elide: Text.ElideRight }
                        Label { width: parent.width; text: modelData.line; wrapMode: Text.Wrap; lineHeight: 1.1; color: theme.textDim }
                    }
                }
            }
        }
        }
    }
    function languageName(code) {
        var hit = words.languages.filter(function(l) { return l.code === code })
        return hit.length ? hit[0].name : code
    }

    // ---- The library: the folder, then what the scan found, posters and all
    Component {
        id: libraryPane
        Item {
            // Nothing chosen yet: one big target
            Tile {
                objectName: "onboarding-choose"
                visible: root.sources.length === 0
                raised: chooseMouse.containsMouse
                dashed: 1
                borderColor: chooseMouse.containsMouse ? theme.accent : theme.lineStrong
                width: parent.width
                height: parent.height
                Column {
                    anchors.centerIn: parent
                    spacing: theme.space(4)
                    Badge { anchors.horizontalCenter: parent.horizontalCenter; glyph: "folder-plus"; box: theme.space(20) }
                    Label { anchors.horizontalCenter: parent.horizontalCenter; msg: "onboarding-choose-folder"; role: "title"; font.pointSize: theme.typeLarge * 1.2 }
                    Label { anchors.horizontalCenter: parent.horizontalCenter; msg: "onboarding-films-too"; color: theme.textDim }
                    Item { width: 1; height: theme.space(2) }
                    Button { anchors.horizontalCenter: parent.horizontalCenter; objectName: "onboarding-add-folder"; msg: "onboarding-choose-folder-button"; icon: "folder-open"; primary: true; onClicked: folderDialog.open() }
                    // Someone starting from nothing gets a folder, so downloads have a home
                    Button {
                        anchors.horizontalCenter: parent.horizontalCenter
                        objectName: "onboarding-make-folder"
                        msg: "onboarding-no-folder"
                        flat: true
                        onClicked: root.makeStarter()
                    }
                    Label {
                        anchors.horizontalCenter: parent.horizontalCenter
                        text: words.t("onboarding-starter-where", { dir: root.videosDir })
                        font.pointSize: theme.typeSmall
                        color: theme.textDim
                    }
                }
                MouseArea { id: chooseMouse; anchors.fill: parent; z: -1; hoverEnabled: true; cursorShape: Qt.PointingHandCursor; onClicked: folderDialog.open() ; Accessible.role: Accessible.Button; Accessible.name: theme.nameNear(parent); Accessible.onPressAction: folderDialog.open() }
            }
            // Chosen: the folders, the scan as it runs, and the posters it found
            Column {
                id: libraryRoot
                visible: root.sources.length > 0
                width: parent.width
                height: parent.height
                spacing: theme.grout
                Tile {
                    id: folderTile
                    width: parent.width
                    height: folders.implicitHeight + padding * 2
                    Column {
                        id: folders
                        width: parent.width
                        spacing: theme.space(3)
                        Repeater {
                            model: root.sources
                            Row {
                                required property var modelData
                                spacing: theme.space(3)
                                Icon { glyph: "folder"; color: theme.textDim; anchors.verticalCenter: parent.verticalCenter }
                                Label { text: modelData.path; anchors.verticalCenter: parent.verticalCenter }
                            }
                        }
                        Item {
                            width: parent.width
                            height: addMore.height
                            Row {
                                anchors.verticalCenter: parent.verticalCenter
                                spacing: theme.space(3)
                                Icon {
                                    id: spin
                                    readonly property bool busy: root.scanJob !== null || settling.running
                                    glyph: busy ? "loader-circle" : "circle-check"
                                    color: busy ? theme.textDim : theme.accent
                                    anchors.verticalCenter: parent.verticalCenter
                                    RotationAnimator on rotation { running: spin.busy; from: 0; to: 360; duration: 1000; loops: Animation.Infinite }
                                }
                                Label {
                                    objectName: "onboarding-scan-line"
                                    anchors.verticalCenter: parent.verticalCenter
                                    text: root.scanLine
                                    color: theme.textDim
                                }
                            }
                            Button { id: addMore; anchors.right: parent.right; msg: "onboarding-add-another"; icon: "folder-plus"; small: true; onClicked: folderDialog.open() }
                        }
                    }
                }
                // The payoff: the library as it starts to look. Whole rows only: when more
                // series came in than fit, the last tile counts the rest instead of one
                // orphan sitting on a row of its own.
                Item {
                    id: shelf
                    width: parent.width
                    height: Math.max(0, libraryRoot.height - folderTile.height - theme.grout)
                    readonly property int n: root.shelfCards.length
                    // The biggest tile that fits every series in whole rows, between a floor a
                    // title still reads at and a ceiling past which a poster is a hero, not a
                    // shelf. A library too big for the floor fills the rows it can and counts
                    // the rest on its last tile.
                    readonly property var fit: {
                        var g = theme.grout, minW = theme.space(30), maxW = theme.space(64), best = null
                        for (var r = 1; r <= 8 && n > 0; r++) {
                            var c = Math.ceil(n / r)
                            // As wide as the row allows and as tall as the rows allow, so a
                            // short library grows into the stage instead of one thin row
                            var w = Math.min(maxW, (width - g * (c - 1)) / c, ((height - (r - 1) * g) / r - 1) / 1.45)
                            if (w < minW) continue
                            if (!best || w > best.w) best = { c: c, w: w, shown: n }
                        }
                        if (best) return best
                        var cc = Math.max(1, Math.floor((width + g) / (minW + g)))
                        var ww = (width - g * (cc - 1)) / cc
                        var rr = Math.max(1, Math.floor((height + g) / (Math.round(ww * 1.45) + g)))
                        return { c: cc, w: ww, shown: Math.min(n, cc * rr) }
                    }
                    readonly property int count: fit.c
                    readonly property real cardW: fit.w
                    readonly property real cardH: Math.round(cardW * 1.45)
                    readonly property int shown: fit.shown
                    readonly property int rest: n - shown
                    clip: true
                    Grid {
                        columns: shelf.count
                        spacing: theme.grout
                        Repeater {
                            model: root.shelfCards.slice(0, shelf.shown)
                            Corner {
                                id: card
                                required property var modelData
                                required property int index
                                readonly property string path: modelData.poster || ""
                                readonly property bool more: shelf.rest > 0 && index === shelf.shown - 1
                                readonly property bool waiting: !more && fillItem === null
                                width: shelf.cardW; height: shelf.cardH
                                radius: theme.radiusTile
                                smoothing: theme.cornerSmoothing
                                color: waiting ? theme.tile : theme.raised
                                borderColor: theme.line; borderWidth: 1
                                fillItem: !more && path !== "" && art.status === Image.Ready ? art : null
                                Image {
                                    id: art
                                    visible: false
                                    anchors.fill: parent
                                    source: card.path !== "" ? "file://" + card.path : ""
                                    sourceSize: theme.posterDecodeSize
                                    fillMode: Image.PreserveAspectCrop
                                    asynchronous: true
                                }
                                // A poster on its way: a slow breath, and the title where a
                                // poster's title sits, not in the middle like an error
                                SequentialAnimation on opacity {
                                    running: card.waiting
                                    loops: Animation.Infinite
                                    NumberAnimation { from: 1; to: 0.55; duration: 900; easing.type: Easing.InOutSine }
                                    NumberAnimation { from: 0.55; to: 1; duration: 900; easing.type: Easing.InOutSine }
                                    onRunningChanged: if (!running) card.opacity = 1
                                }
                                Label {
                                    visible: card.waiting
                                    anchors.left: parent.left; anchors.right: parent.right; anchors.bottom: parent.bottom
                                    anchors.margins: theme.space(4)
                                    text: card.modelData.title
                                    color: theme.textDim
                                    wrapMode: Text.Wrap
                                    maximumLineCount: 3
                                    elide: Text.ElideRight
                                }
                                Column {
                                    visible: card.more
                                    anchors.centerIn: parent
                                    Label { anchors.horizontalCenter: parent.horizontalCenter; text: "+" + (shelf.rest + 1); role: "display"; font.pointSize: theme.typeHeading }
                                    Label { anchors.horizontalCenter: parent.horizontalCenter; msg: "onboarding-shelf-more"; color: theme.textDim }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // ---- The list: first what it does, with a show from their own folder when there is
    // one, then the Accounts tab's own rows, so the flow and its states are the frame's
    Component {
        id: listsPane
        Column {
            spacing: theme.space(4)
            readonly property var show: {
                var cards = root.shelfCards
                for (var i = 0; i < cards.length; i++) if (cards[i].poster !== "") return cards[i]
                return cards.length ? cards[0] : null
            }
            Tile {
                objectName: "onboarding-list-preview"
                width: parent.width
                height: theme.space(36)
                padding: theme.space(5)
                Corner {
                    id: thumb
                    width: Math.round(height / 1.45); height: parent.height
                    radius: theme.radiusInner(theme.space(5))
                    color: theme.tileSunken
                    fillItem: thumbArt.status === Image.Ready ? thumbArt : null
                    Image {
                        id: thumbArt
                        visible: false
                        anchors.fill: parent
                        source: show && show.poster !== "" ? "file://" + show.poster : ""
                        sourceSize: theme.posterDecodeSize
                        fillMode: Image.PreserveAspectCrop
                        asynchronous: true
                    }
                    Icon { visible: thumbArt.status !== Image.Ready; anchors.centerIn: parent; glyph: "film"; color: theme.textDim }
                }
                Column {
                    id: watched
                    anchors.left: thumb.right; anchors.leftMargin: theme.space(5)
                    anchors.verticalCenter: parent.verticalCenter
                    width: (parent.width - thumb.width - arrow.width - theme.space(5) * 4) / 2
                    spacing: theme.space(1)
                    Label { width: parent.width; text: show ? show.title : words.t("onboarding-preview-show"); role: "title"; font.pointSize: theme.typeLarge; elide: Text.ElideRight }
                    Label { width: parent.width; msg: "onboarding-preview-finished"; color: theme.textDim; elide: Text.ElideRight }
                }
                Icon { id: arrow; anchors.left: watched.right; anchors.leftMargin: theme.space(5); anchors.verticalCenter: parent.verticalCenter; glyph: "arrow-right"; color: theme.textDim }
                Column {
                    anchors.left: arrow.right; anchors.leftMargin: theme.space(5)
                    anchors.right: parent.right
                    anchors.verticalCenter: parent.verticalCenter
                    spacing: theme.space(1)
                    Row {
                        spacing: theme.space(2)
                        Icon { glyph: "circle-check"; color: theme.accent; anchors.verticalCenter: parent.verticalCenter }
                        Label { msg: "onboarding-preview-on-list"; role: "title"; font.pointSize: theme.typeLarge; anchors.verticalCenter: parent.verticalCenter }
                    }
                    Label { width: parent.width; msg: "onboarding-preview-set-by"; color: theme.textDim; elide: Text.ElideRight }
                }
            }
            Tile {
                width: parent.width
                height: rows.implicitHeight + padding * 2
                padding: theme.space(7)
                Column {
                    id: rows
                    width: parent.width
                    spacing: theme.space(5)
                    TrackerRow { tracker: "Anilist"; account: root.anilist }
                    Rectangle { width: parent.width; height: 1; color: theme.line }
                    TrackerRow { tracker: "Mal"; account: root.mal }
                }
            }
            Label {
                width: parent.width
                text: root.tracked ? words.t("onboarding-lists-connected")
                    : words.t("onboarding-lists-no-account")
                color: root.tracked ? theme.accent : theme.textDim
                wrapMode: Text.Wrap
            }
        }
    }

    // ---- Downloads: where it searches, then whose torrent client fetches
    Component {
        id: downloadsPane
        Column {
          spacing: theme.space(6)
          // Where to search. The app carries no sources: a repo someone gives the user does
          Tile {
            id: where
            objectName: "onboarding-sources"
            width: parent.width
            height: whereWords.implicitHeight + padding * 2
            padding: theme.space(7)
            Badge { id: whereMark; glyph: root.endpointCount > 0 ? "check" : "search"; on: root.endpointCount > 0 }
            Column {
                id: whereWords
                anchors.left: whereMark.right; anchors.leftMargin: theme.space(5)
                anchors.right: whereButton.left; anchors.rightMargin: theme.space(5)
                spacing: theme.space(2)
                Label { msg: "onboarding-search-title"; role: "title"; font.pointSize: theme.typeLarge * 1.2 }
                Label {
                    width: parent.width
                    text: root.endpointCount > 0
                        ? words.t("onboarding-search-ready", { count: root.endpointCount })
                        : words.t("onboarding-search-none")
                    color: theme.textDim; wrapMode: Text.Wrap; lineHeight: 1.1
                }
            }
            Button {
                id: whereButton
                objectName: "onboarding-add-sources"
                anchors.right: parent.right
                anchors.verticalCenter: whereWords.verticalCenter
                visible: root.endpointCount === 0
                text: root.wantsSources ? words.t("onboarding-sources-after") : words.t("onboarding-sources-add")
                icon: root.wantsSources ? "check" : "plus"
                onClicked: root.wantsSources = !root.wantsSources
            }
          }
          Label { msg: "onboarding-fetch-title"; role: "title"; font.pointSize: theme.typeLarge * 1.2 }
          Row {
            id: choices
            width: parent.width
            spacing: theme.grout
            readonly property real cell: (width - spacing) / 2
            Repeater {
                model: [
                    { mode: "Auto", icon: "sparkles",
                      title: words.t("onboarding-fetch-auto"), tag: words.t("onboarding-fetch-recommended"),
                      line: words.t("onboarding-fetch-auto-line") },
                    { mode: "Existing", icon: "hard-drive",
                      title: words.t("onboarding-fetch-existing"), tag: "",
                      line: words.t("onboarding-fetch-existing-line") }
                ]
                Tile {
                    id: choice
                    required property var modelData
                    readonly property bool on: root.torrentMode === modelData.mode
                    objectName: "onboarding-mode-" + modelData.mode
                    width: choices.cell
                    height: cw.y + cw.height + padding * 2
                    raised: on || choiceMouse.containsMouse
                    borderColor: on ? theme.accentEdge : theme.line
                    borderWidth: on ? 2 : 1
                    padding: theme.space(7)
                    Badge { id: ci; glyph: modelData.icon; on: choice.on }
                    // Chosen or not, at the corner where the eye starts
                    Rectangle {
                        anchors.right: parent.right
                        width: theme.space(6); height: width; radius: width / 2
                        color: choice.on ? theme.accentSoft : "transparent"
                        border.width: 1; border.color: choice.on ? theme.accentEdge : theme.lineStrong
                        Icon { visible: choice.on; anchors.centerIn: parent; glyph: "check"; size: theme.space(3.5); color: theme.accent }
                    }
                    Column {
                        id: cw
                        anchors.top: ci.bottom; anchors.topMargin: theme.space(5)
                        width: parent.width
                        spacing: theme.space(2)
                        Row {
                            spacing: theme.space(3)
                            Label { id: ct; text: modelData.title; role: "title"; font.pointSize: theme.typeLarge * 1.2 }
                            Label { visible: modelData.tag !== ""; anchors.baseline: ct.baseline; text: modelData.tag; color: theme.textDim }
                        }
                        Label { width: parent.width; text: modelData.line; color: theme.textDim; wrapMode: Text.Wrap; lineHeight: 1.1 }
                    }
                    MouseArea { id: choiceMouse; anchors.fill: parent; anchors.margins: -choice.padding; hoverEnabled: true; cursorShape: Qt.PointingHandCursor; onClicked: { root.torrentMode = modelData.mode; root.saveTorrentMode() }
                        Accessible.role: Accessible.RadioButton; Accessible.name: modelData.title; Accessible.checked: choice.on; Accessible.onPressAction: choiceMouse.clicked(null) }
                }
            }
          }
        }
    }

    // ---- Ready: what got set up, and a way straight back to each thing left for later
    Component {
        id: readyPane
        Row {
            id: summary
            spacing: theme.grout
            readonly property real cell: (width - spacing * 2) / 3
            readonly property real tallest: Math.max(t0 ? t0.contentH : 0, t1 ? t1.contentH : 0, t2 ? t2.contentH : 0)
            Repeater {
                id: tiles
                model: [
                    { ok: root.sources.length > 0, icon: "folder-open", title: words.t("onboarding-ready-library"), step: 1,
                      line: root.sources.length === 0 ? words.t("onboarding-ready-library-none")
                          : root.seriesCount > 0 ? words.t("onboarding-ready-library-series", { count: root.sources.length, series: root.seriesCount })
                          : root.starterOnly ? words.t("onboarding-ready-library-empty", { count: root.sources.length })
                          : words.t("onboarding-ready-library-scanning", { count: root.sources.length }) },
                    { ok: root.tracked, icon: "user-check", title: words.t("onboarding-ready-lists"), step: 2,
                      line: root.anilist.connected === true ? words.t("onboarding-ready-lists-as", { tracker: "AniList", user: root.anilist.username || "?" })
                          : root.mal.connected === true ? words.t("onboarding-ready-lists-as", { tracker: "MyAnimeList", user: root.mal.username || "?" })
                          : words.t("onboarding-ready-lists-none") },
                    { ok: root.downloadsReady, icon: "download", title: words.t("onboarding-ready-downloads"), step: 3,
                      line: words.t("onboarding-ready-downloads-line", { sources: root.endpointCount, client: root.torrentMode === "Existing" ? "own" : "bundled",
                          next: root.downloadsReady ? "ready" : root.wantsSources || root.torrentMode === "Existing" ? "opens" : "both" }) }
                ]
                Tile {
                    id: tile
                    required property var modelData
                    required property int index
                    readonly property real contentH: rc.y + rc.height
                    // Downloads opens Settings next by itself when asked to; no button then
                    readonly property bool offers: !modelData.ok && (modelData.step !== 3 || !(root.wantsSources || root.torrentMode === "Existing"))
                    width: summary.cell
                    height: summary.tallest + padding * 2
                    padding: theme.space(7)
                    Component.onCompleted: { if (index === 0) summary.t0 = tile; else if (index === 1) summary.t1 = tile; else summary.t2 = tile }
                    Badge { id: mark; glyph: modelData.ok ? "check" : modelData.icon; on: modelData.ok }
                    Column {
                        id: rc
                        anchors.top: mark.bottom; anchors.topMargin: theme.space(5)
                        width: parent.width
                        spacing: theme.space(2)
                        Label { text: modelData.title; role: "title"; font.pointSize: theme.typeLarge * 1.2 }
                        Label { width: parent.width; text: modelData.line; color: theme.textDim; wrapMode: Text.Wrap; lineHeight: 1.1 }
                        Item { visible: tile.offers; width: 1; height: theme.space(2) }
                        Button {
                            visible: tile.offers
                            objectName: "onboarding-later-" + modelData.step
                            text: modelData.step === 3 ? words.t("onboarding-ready-open-downloads") : words.t("onboarding-ready-set-up"); icon: "arrow-right"; small: true
                            onClicked: if (modelData.step === 3) root.finishAt("downloads"); else root.step = modelData.step
                        }
                    }
                }
            }
            property Item t0: null
            property Item t1: null
            property Item t2: null
        }
    }
}
