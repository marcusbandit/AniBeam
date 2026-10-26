// The first run: a fresh install opens here instead of on an empty library. Five steps, one
// idea each: what AniBeam is, the folder the anime lives in, the tracker, how Downloads gets
// its torrent client, and a closing screen that says what got set up and where each skipped
// step lives. It takes the whole window, the rail included, as two panes of the mosaic: the
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
    // Steps left with Skip, by key: the closing screen points those back at Settings
    property var skipped: ({})

    property var sources: []
    // The series found so far, a title and a poster path (empty until one is fetched) each
    property var shelfCards: []
    property int seriesCount: 0
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
    readonly property string scanLine: {
        var j = scanJob
        if (!j) return seriesCount > 0 ? "Found " + Fmt.plural(seriesCount, "series", "series") + ". You can go on; the rest fills in by itself."
                                       : "No video files turned up. Check that it is the right folder."
        var of = j.total > 0 ? ", " + j.done + " of " + j.total : ""
        if (j.kind === "Scan") return "Reading the folder" + of
        if (j.kind === "AutoMatch") return "Looking up " + (j.label || "your shows") + of
        if (j.kind === "FillImages") return "Fetching posters" + of
        return "Finding related seasons" + of
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
        if (r.error) frame.toast(r.error.message)
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
            id: version
            anchors.top: mark.bottom; anchors.topMargin: theme.space(1)
            text: "Setup, about a minute"
            role: "ui"; color: theme.textDim
        }
        Column {
            anchors.top: version.bottom; anchors.topMargin: theme.space(10)
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
                        color: row.current ? theme.accentSoft : rowMouse.containsMouse && row.passed ? theme.raised : "transparent"
                        border.width: row.current ? 1 : 0
                        border.color: theme.accentEdge
                    }
                    Rectangle {
                        id: dot
                        x: theme.space(3)
                        anchors.verticalCenter: parent.verticalCenter
                        width: theme.space(7); height: width; radius: width / 2
                        color: row.passed && !row.wasSkipped ? theme.accent : "transparent"
                        border.width: row.passed && !row.wasSkipped ? 0 : 1
                        border.color: row.current ? theme.accent : theme.lineStrong
                        Icon {
                            anchors.centerIn: parent
                            size: theme.space(3.5)
                            glyph: row.passed && !row.wasSkipped ? "check" : row.modelData.icon
                            color: row.passed && !row.wasSkipped ? theme.accentText : row.current ? theme.text : theme.textFaint
                        }
                    }
                    Label {
                        anchors.left: dot.right; anchors.leftMargin: theme.space(3)
                        anchors.verticalCenter: parent.verticalCenter
                        text: row.modelData.label
                        role: "ui"
                        font.weight: row.current ? Font.DemiBold : Font.Normal
                        color: row.current ? theme.text : row.passed ? theme.textDim : theme.textFaint
                    }
                    Label {
                        anchors.right: parent.right; anchors.rightMargin: theme.space(3)
                        anchors.verticalCenter: parent.verticalCenter
                        visible: row.wasSkipped
                        text: "Skipped"
                        role: "ui"; font.pointSize: theme.typeSmall; color: theme.textFaint
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
            icon: "chevrons-right"
            flat: true; small: true
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

            Column {
                id: head
                width: Math.min(parent.width, theme.space(190))
                spacing: theme.space(3)
                Label {
                    objectName: "onboarding-eyebrow"
                    text: "Step " + (root.step + 1) + " of " + root.steps.length
                    role: "ledger"; color: theme.textFaint
                }
                Label {
                    objectName: "onboarding-title"
                    width: parent.width
                    text: [
                        "Welcome to AniBeam",
                        "Where is your anime?",
                        "Connect your anime list",
                        "How downloads work",
                        root.anythingSkipped ? "Ready, with a few things for later" : "You're all set"
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
                        "Follow a show on the Downloads page and new episodes arrive in your library on their own as they air. AniBeam needs a torrent client for that, and it can bring its own.",
                        "Everything below can be changed later in Settings. You can run this setup again from Settings, Data."
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
                anchors.top: head.bottom; anchors.topMargin: theme.space(10)
                anchors.bottom: foot.top; anchors.bottomMargin: theme.space(8)
                width: parent.width
                sourceComponent: [welcomePane, libraryPane, listsPane, downloadsPane, readyPane][root.step]
            }

            Item {
                id: foot
                anchors.bottom: parent.bottom
                width: parent.width
                height: cont.height
                Button {
                    objectName: "onboarding-back"
                    visible: root.step > 0
                    text: "Back"; icon: "arrow-left"; flat: true
                    onClicked: root.step--
                }
                Row {
                    anchors.right: parent.right
                    spacing: theme.space(3)
                    Button {
                        objectName: "onboarding-skip"
                        visible: root.skippable
                        text: "Skip for now"; flat: true
                        onClicked: root.skip()
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
    }
    // Skip stands beside Continue only where the step asks for something not yet given
    readonly property bool skippable: (step === 1 && sources.length === 0) || (step === 2 && !tracked)
    readonly property bool anythingSkipped: sources.length === 0 || !tracked || torrentMode === "Existing"

    // ---- Welcome: what the app does, before it asks for anything
    Component {
        id: welcomePane
        Grid {
            id: grid
            columns: width > theme.space(200) ? 4 : 2
            spacing: theme.grout
            readonly property real cell: (width - spacing * (columns - 1)) / columns
            Repeater {
                model: [
                    { icon: "folder-open", title: "Your library", line: "Posters, episodes and what comes next, for every show in your folders." },
                    { icon: "circle-play", title: "Plays in the window", line: "Subtitles, intro skips, and it remembers where you stopped." },
                    { icon: "user-check", title: "Your list, kept current", line: "Finish an episode and it counts on AniList or MyAnimeList." },
                    { icon: "download", title: "New episodes, by themselves", line: "Follow a show and each new episode lands in your library." }
                ]
                Tile {
                    required property var modelData
                    raised: true
                    width: grid.cell
                    // The four fill the pane, the words at the foot of each, the way the
                    // home wall sets its tiles
                    height: Math.max(theme.space(52), grid.columns === 4 ? grid.height : (grid.height - grid.spacing) / 2)
                    padding: theme.space(7)
                    Corner {
                        id: badge
                        width: theme.space(12); height: width; radius: width / 2
                        smoothing: theme.cornerSmoothing
                        color: theme.accentSoft
                        borderColor: theme.accentEdge; borderWidth: 1
                        Icon { anchors.centerIn: parent; glyph: modelData.icon; color: theme.accent; size: theme.space(6) }
                    }
                    Label { id: t; anchors.bottom: tl.top; anchors.bottomMargin: theme.space(2); width: parent.width; text: modelData.title; role: "title"; font.pointSize: theme.typeLarge * 1.2; wrapMode: Text.Wrap }
                    Label { id: tl; anchors.bottom: parent.bottom; width: parent.width; text: modelData.line; color: theme.textDim; wrapMode: Text.Wrap; lineHeight: 1.1 }
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
                    Icon { anchors.horizontalCenter: parent.horizontalCenter; glyph: "folder-plus"; size: theme.space(14); color: theme.accent }
                    Label { anchors.horizontalCenter: parent.horizontalCenter; text: "Choose your anime folder"; role: "title"; font.pointSize: theme.typeLarge }
                    Label { anchors.horizontalCenter: parent.horizontalCenter; text: "A folder of films works too: call it Movies."; color: theme.textDim }
                    Button { anchors.horizontalCenter: parent.horizontalCenter; objectName: "onboarding-add-folder"; text: "Choose folder"; icon: "folder-open"; primary: true; onClicked: folderDialog.open() }
                }
                MouseArea { id: chooseMouse; anchors.fill: parent; z: -1; hoverEnabled: true; cursorShape: Qt.PointingHandCursor; onClicked: folderDialog.open() }
            }
            // Chosen: the folders, the scan as it runs, and the posters it found
            Column {
                id: libraryRoot
                height: parent.height
                visible: root.sources.length > 0
                width: parent.width
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
                                Label { text: modelData.path; role: "ledger"; anchors.verticalCenter: parent.verticalCenter }
                            }
                        }
                        Item {
                            width: parent.width
                            height: addMore.height
                            Row {
                                anchors.verticalCenter: parent.verticalCenter
                                spacing: theme.space(3)
                                Icon {
                                    glyph: root.scanJob ? "loader-circle" : "circle-check"
                                    color: root.scanJob ? theme.textDim : theme.accent
                                    anchors.verticalCenter: parent.verticalCenter
                                    RotationAnimator on rotation { running: root.scanJob !== null; from: 0; to: 360; duration: 1000; loops: Animation.Infinite }
                                }
                                Label {
                                    objectName: "onboarding-scan-line"
                                    anchors.verticalCenter: parent.verticalCenter
                                    text: root.scanLine
                                }
                            }
                            Button { id: addMore; anchors.right: parent.right; text: "Add another folder"; icon: "folder-plus"; small: true; onClicked: folderDialog.open() }
                        }
                    }
                }
                // The payoff: the library as it starts to look, a tile per series found,
                // the title until its poster arrives
                Item {
                    id: shelf
                    width: parent.width
                    height: Math.max(0, libraryRoot.height - folderTile.height - theme.grout)
                    readonly property int count: Math.max(1, Math.floor((width + theme.grout) / (theme.space(36) + theme.grout)))
                    readonly property real cardW: (width - theme.grout * (count - 1)) / count
                    readonly property real cardH: Math.round(cardW * 1.45)
                    readonly property int rows: Math.max(1, Math.floor((height + theme.grout) / (cardH + theme.grout)))
                    clip: true
                    Grid {
                        columns: shelf.count
                        spacing: theme.grout
                        Repeater {
                            model: root.shelfCards.slice(0, shelf.count * shelf.rows)
                            Corner {
                                id: card
                                required property var modelData
                                readonly property string path: modelData.poster || ""
                                width: shelf.cardW; height: shelf.cardH
                                radius: theme.radiusTile
                                smoothing: theme.cornerSmoothing
                                color: theme.raised
                                borderColor: theme.line; borderWidth: 1
                                fillItem: path !== "" && art.status === Image.Ready ? art : null
                                Image {
                                    id: art
                                    visible: false
                                    anchors.fill: parent
                                    source: card.path !== "" ? "file://" + card.path : ""
                                    sourceSize: theme.posterDecodeSize
                                    fillMode: Image.PreserveAspectCrop
                                    asynchronous: true
                                }
                                Label {
                                    visible: card.fillItem === null
                                    anchors.fill: parent; anchors.margins: theme.space(4)
                                    text: card.modelData.title
                                    role: "title"
                                    color: theme.textDim
                                    wrapMode: Text.Wrap
                                    horizontalAlignment: Text.AlignHCenter
                                    verticalAlignment: Text.AlignVCenter
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
            spacing: theme.grout
            Tile {
                width: parent.width
                height: rows.implicitHeight + padding * 2
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
                    : "No account yet? Either service is free. One is plenty; you can add the other later in Settings, Library."
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
                      line: "AniBeam talks to yours through its Web UI. Setup ends on Settings, Downloads, where you give its address and login." }
                ]
                Tile {
                    id: choice
                    required property var modelData
                    readonly property bool on: root.torrentMode === modelData.mode
                    objectName: "onboarding-mode-" + modelData.mode
                    width: choices.cell
                    height: theme.space(64)
                    raised: on || choiceMouse.containsMouse
                    borderColor: on ? theme.accent : theme.line
                    borderWidth: on ? 2 : 1
                    padding: theme.space(7)
                    Icon { id: ci; glyph: modelData.icon; size: theme.space(8); color: choice.on ? theme.accent : theme.textDim }
                    Rectangle {
                        visible: modelData.tag !== ""
                        anchors.right: parent.right
                        width: tagText.implicitWidth + theme.space(4); height: theme.space(6.5); radius: height / 2
                        color: theme.accentSoft; border.color: theme.accentEdge; border.width: 1
                        Label { id: tagText; anchors.centerIn: parent; text: modelData.tag; font.pointSize: theme.typeSmall; color: theme.accent }
                    }
                    Label { id: ct; anchors.top: ci.bottom; anchors.topMargin: theme.space(5); width: parent.width; text: modelData.title; role: "title"; font.pointSize: theme.typeLarge; wrapMode: Text.Wrap }
                    Label { anchors.top: ct.bottom; anchors.topMargin: theme.space(2); width: parent.width; text: modelData.line; color: theme.textDim; wrapMode: Text.Wrap; lineHeight: 1.1 }
                    Rectangle {
                        anchors.bottom: parent.bottom
                        width: theme.space(5.5); height: width; radius: width / 2
                        color: choice.on ? theme.accent : "transparent"
                        border.width: choice.on ? 0 : 1; border.color: theme.lineStrong
                        Icon { visible: choice.on; anchors.centerIn: parent; glyph: "check"; size: theme.space(3.5); color: theme.accentText }
                    }
                    MouseArea { id: choiceMouse; anchors.fill: parent; anchors.margins: -choice.padding; hoverEnabled: true; cursorShape: Qt.PointingHandCursor; onClicked: { root.torrentMode = modelData.mode; root.saveTorrentMode() } }
                }
            }
        }
    }

    // ---- Ready: what got set up, and where each skipped part lives
    Component {
        id: readyPane
        Row {
            id: summary
            spacing: theme.grout
            readonly property real cell: (width - spacing * 2) / 3
            Repeater {
                model: [
                    { ok: root.sources.length > 0, icon: "folder-open", title: "Library",
                      line: root.sources.length > 0 ? Fmt.plural(root.sources.length, "folder", "folders") + (root.seriesCount > 0 ? ", " + Fmt.plural(root.seriesCount, "series", "series") + " so far" : ", scanning") : "No folder yet. Add one in Settings, Library." },
                    { ok: root.tracked, icon: "user-check", title: "Your list",
                      line: root.anilist.connected === true ? "AniList, as " + (root.anilist.username || "?") : root.mal.connected === true ? "MyAnimeList, as " + (root.mal.username || "?") : "Not connected. Connect in Settings, Library, or from the tile on your library." },
                    { ok: root.torrentMode !== "Existing", icon: "download", title: "Downloads",
                      line: root.torrentMode === "Existing" ? "Your own qBittorrent. Settings, Downloads opens next for its address and login." : "AniBeam's own qBittorrent. Nothing else to do." }
                ]
                Tile {
                    required property var modelData
                    width: summary.cell
                    height: theme.space(64)
                    padding: theme.space(7)
                    raised: modelData.ok
                    Rectangle {
                        id: mark
                        width: theme.space(11); height: width; radius: width / 2
                        color: modelData.ok ? theme.accent : "transparent"
                        border.width: modelData.ok ? 0 : 1; border.color: theme.lineStrong
                        Icon { anchors.centerIn: parent; glyph: modelData.ok ? "check" : modelData.icon; size: theme.space(5.5); color: modelData.ok ? theme.accentText : theme.textDim }
                    }
                    Label { id: rt; anchors.bottom: rl.top; anchors.bottomMargin: theme.space(2); width: parent.width; text: modelData.title; role: "title"; font.pointSize: theme.typeLarge * 1.2 }
                    Label { id: rl; anchors.bottom: parent.bottom; width: parent.width; text: modelData.line; color: modelData.ok ? theme.textDim : theme.text; wrapMode: Text.Wrap; lineHeight: 1.1 }
                }
            }
        }
    }
}
