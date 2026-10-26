// The first run: a fresh install opens here instead of on an empty library. Five steps, one
// idea each: what AniBeam is, the folder the anime lives in, the tracker, how Downloads gets
// its torrent client, and a closing screen that says what got set up and offers each skipped
// step again. It takes the whole window, the rail included, as two panes of the mosaic: the
// steps at the left, the step itself at the right.
//
// Every step can be skipped, and so can the whole thing. It shows once: `done` lives in
// shell.conf. An install that already has a library, a tracker or a version of read notes is
// not a fresh one, so an update never lands here. Settings, Data, About runs it again.
//
// The folder step adds the source and shows the scan's real result as it comes in, posters
// included; the tracker step is the same TrackerRow the Library tab uses, so the sign-in is
// the frame's one flow and the connect sheet rises over this when the build has no
// credentials of its own.
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
        { key: "welcome", label: "Welcome", icon: "sparkles" },
        { key: "library", label: "Your library", icon: "folder-open" },
        { key: "lists", label: "Your list", icon: "user-check" },
        { key: "downloads", label: "Downloads", icon: "download" },
        { key: "ready", label: "Ready", icon: "circle-check" }
    ]
    property int step: 0
    // Steps left with Skip, by key
    property var skipped: ({})

    property var sources: []
    // The series found so far, a title and a poster path (empty until one is fetched) each
    property var shelfCards: []
    property int seriesCount: 0
    readonly property int posterCount: shelfCards.filter(function(c) { return c.poster !== "" }).length
    // "Auto" lets AniBeam pick (a qBittorrent that answers, else its own); "Existing" is the
    // user's own, finished in Settings, Downloads once this closes
    property string torrentMode: "Auto"

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
        var tally = seriesCount > 0 ? ", " + posterCount + " of " + seriesCount + " found" : ""
        if (!j && settling.running) return "Reading the folder"
        if (!j) return seriesCount > 0 ? "Found " + Fmt.plural(seriesCount, "series", "series") + ". Go on whenever you like; the rest fills in by itself."
                                       : "No video files turned up. Check that it is the right folder."
        if (j.kind === "Scan") return "Reading the folder"
        if (j.kind === "AutoMatch") return "Looking up " + (j.label || "your shows") + tally
        if (j.kind === "FillImages") return "Fetching posters" + tally
        return "Finding related seasons" + tally
    }

    // As the app starts. A capture or a drive never opens it on its own; a drive calls open().
    function start() {
        if (kept.done) return
        if (notes.version !== "" || hasLibrary() || tracked) { kept.done = true; return }
        if (Shell.shoot === "" && Shell.driveOut === "") open()
    }
    function hasLibrary() { var r = Door.listSources(); return !r.error && r.reply.sources.length > 0 }
    function open(at) {
        step = at || 0
        skipped = ({})
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
        if (torrentMode === "Existing") { frame.settingsTab = 3; frame.go("settings") }
        else if (sources.length > 0) frame.go("library")
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
    function addFolder(url) {
        var r = Door.addSource(decodeURIComponent(String(url).replace("file://", "")))
        if (r.error) { frame.toast(r.error.message); return }
        settling.restart()
        reload()
    }
    function saveTorrentMode() {
        var d = Door.getDownloads()
        if (d.error) return
        var s = JSON.parse(JSON.stringify(d.reply.status.settings))
        if ((s.torrent_mode === "Existing") === (torrentMode === "Existing")) return
        s.torrent_mode = torrentMode
        var r = Door.setDownloadSettings(s, {})
        if (r.error) frame.toast(r.error.message)
    }
    FolderDialog {
        id: folderDialog
        title: "Choose your anime folder"
        currentFolder: StandardPaths.writableLocation(StandardPaths.MoviesLocation)
        onAccepted: root.addFolder(selectedFolder)
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
            text: "A minute of setup. Your library finishes on its own."
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
                        text: "Later"
                        role: "ui"; font.pointSize: theme.typeSmall; color: theme.textDim
                    }
                    // A step already passed can be opened again
                    MouseArea {
                        id: rowMouse
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
            text: "Skip setup"
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
                                "Welcome to AniBeam",
                                "Where is your anime?",
                                "Connect your anime list",
                                "How downloads work",
                                root.laterCount === 0 ? "You're all set" : root.laterCount === 1 ? "Ready, one thing for later" : "Ready, a few things for later"
                            ][root.step]
                            role: "display"
                            font.pointSize: theme.typeHeading * 1.6
                            wrapMode: Text.Wrap
                        }
                        Label {
                            width: parent.width
                            text: [
                                "AniBeam keeps your anime in one place. It finds the shows on your disk, plays them right here, and keeps AniList or MyAnimeList up to date as you watch.",
                                "Pick the folder you keep your shows in. Every folder inside it becomes a series, with its poster, its episodes and where you left off. AniBeam only reads the folder. It never moves or renames a file.",
                                "Sign in once, and every episode you finish here is marked on your list, with nothing to type. The sign-in happens in your browser, so AniBeam never sees your password.",
                                "Follow a show on the Downloads page and new episodes land in your library as they air.",
                                "Everything here can be changed later in Settings, and Settings, Data runs this setup again."
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
                    text: "Skip for now"; flat: true
                    onClicked: root.skip()
                }
                Button {
                    objectName: "onboarding-back"
                    visible: root.step > 0
                    text: "Back"; icon: "arrow-left"
                    onClicked: root.step--
                }
                Button {
                    id: cont
                    objectName: "onboarding-continue"
                    primary: true
                    icon: root.step === root.steps.length - 1 ? "circle-play" : "arrow-right"
                    text: root.step === 0 ? "Get started" : root.step === root.steps.length - 1 ? "Open AniBeam" : "Continue"
                    enabled: root.step !== 1 || root.sources.length > 0
                    onClicked: root.next()
                }
            }
        }
    }
    // Skip stands beside Continue only where the step asks for something not yet given
    readonly property bool skippable: (step === 1 && sources.length === 0) || (step === 2 && !tracked)
    readonly property int laterCount: (sources.length === 0 ? 1 : 0) + (tracked ? 0 : 1) + (torrentMode === "Existing" ? 1 : 0)

    // ---- Welcome: what the app does, before it asks for anything
    Component {
        id: welcomePane
        Grid {
            id: grid
            columns: width > theme.space(200) ? 4 : 2
            spacing: theme.grout
            readonly property real cell: (width - spacing * (columns - 1)) / columns
            // One height for the four, the tallest one's
            property real tallest: 0
            Repeater {
                model: [
                    { icon: "folder-open", title: "Your library", line: "Posters, episodes and what comes next, for every show in your folders." },
                    { icon: "circle-play", title: "Plays right here", line: "Subtitles, intro skips, and it remembers where you stopped." },
                    { icon: "user-check", title: "Your list, synced", line: "Finish an episode and it counts on AniList or MyAnimeList." },
                    { icon: "download", title: "Episodes arrive", line: "Follow a show and each new episode lands in your library." }
                ]
                Tile {
                    required property var modelData
                    width: grid.cell
                    readonly property real need: words.y + words.height + padding * 2
                    onNeedChanged: grid.tallest = Math.max(grid.tallest, need)
                    height: Math.max(need, grid.tallest)
                    padding: theme.space(7)
                    Badge { id: badge; glyph: modelData.icon }
                    Column {
                        id: words
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
                    Label { anchors.horizontalCenter: parent.horizontalCenter; text: "Choose your anime folder"; role: "title"; font.pointSize: theme.typeLarge * 1.2 }
                    Label { anchors.horizontalCenter: parent.horizontalCenter; text: "A folder of films works too: call it Movies."; color: theme.textDim }
                    Item { width: 1; height: theme.space(2) }
                    Button { anchors.horizontalCenter: parent.horizontalCenter; objectName: "onboarding-add-folder"; text: "Choose folder"; icon: "folder-open"; primary: true; onClicked: folderDialog.open() }
                }
                MouseArea { id: chooseMouse; anchors.fill: parent; z: -1; hoverEnabled: true; cursorShape: Qt.PointingHandCursor; onClicked: folderDialog.open() }
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
                            Button { id: addMore; anchors.right: parent.right; text: "Add another folder"; icon: "folder-plus"; small: true; onClicked: folderDialog.open() }
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
                        var g = theme.grout, minW = theme.space(30), maxW = theme.space(48), best = null
                        for (var r = 1; r <= 8 && n > 0; r++) {
                            var c = Math.ceil(n / r)
                            var w = Math.min(maxW, (width - g * (c - 1)) / c)
                            if (w < minW) continue
                            if (r * Math.round(w * 1.45) + (r - 1) * g > height) break
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
                                    Label { anchors.horizontalCenter: parent.horizontalCenter; text: "more"; color: theme.textDim }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // ---- The list: the Library tab's own rows, so the flow and its states are the frame's
    Component {
        id: listsPane
        Column {
            spacing: theme.space(4)
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
                text: root.tracked ? "Connected. What you watch here now counts on your list."
                    : "No account yet? Both are free, and one is plenty."
                color: root.tracked ? theme.accent : theme.textDim
                wrapMode: Text.Wrap
            }
        }
    }

    // ---- Downloads: whose torrent client
    Component {
        id: downloadsPane
        Row {
            id: choices
            spacing: theme.grout
            readonly property real cell: (width - spacing) / 2
            Repeater {
                model: [
                    { mode: "Auto", icon: "sparkles", title: "Let AniBeam handle it", tag: "Recommended",
                      line: "AniBeam brings its own qBittorrent and runs it quietly in the background. Nothing to install, nothing to fill in." },
                    { mode: "Existing", icon: "hard-drive", title: "I already use qBittorrent", tag: "",
                      line: "AniBeam connects to the qBittorrent you already run. You enter its address and login in Settings next." }
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
                    MouseArea { id: choiceMouse; anchors.fill: parent; anchors.margins: -choice.padding; hoverEnabled: true; cursorShape: Qt.PointingHandCursor; onClicked: { root.torrentMode = modelData.mode; root.saveTorrentMode() } }
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
                    { ok: root.sources.length > 0, icon: "folder-open", title: "Library", step: 1,
                      line: root.sources.length > 0 ? Fmt.plural(root.sources.length, "folder", "folders") + (root.seriesCount > 0 ? ", " + Fmt.plural(root.seriesCount, "series", "series") + " so far" : ", scanning") : "No folder yet." },
                    { ok: root.tracked, icon: "user-check", title: "Your list", step: 2,
                      line: root.anilist.connected === true ? "AniList, as " + (root.anilist.username || "?") : root.mal.connected === true ? "MyAnimeList, as " + (root.mal.username || "?") : "Not connected yet." },
                    { ok: root.torrentMode !== "Existing", icon: "download", title: "Downloads", step: 3,
                      line: root.torrentMode === "Existing" ? "Your own qBittorrent. Settings, Downloads opens next for its address and login." : "AniBeam's own qBittorrent. All set." }
                ]
                Tile {
                    id: tile
                    required property var modelData
                    required property int index
                    readonly property real contentH: rc.y + rc.height
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
                        Item { visible: !modelData.ok && modelData.step !== 3; width: 1; height: theme.space(2) }
                        Button {
                            visible: !modelData.ok && modelData.step !== 3
                            objectName: "onboarding-later-" + modelData.step
                            text: "Set up now"; icon: "arrow-right"; small: true
                            onClicked: root.step = modelData.step
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
