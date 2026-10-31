// Settings: the head row (the title in the display cut, the settings search at the right),
// then the index at the left and the open section beside it. The index is a status board
// as much as a menu: each entry is a glyph, the section's name and one live line on how
// it stands now (who is signed in, which theme, whether the torrent client answered), so
// the page answers "is everything set up" before anything is clicked. The entries sit in
// four runs with a gap between them: the library and its accounts; how things look and
// play; downloads and the phone; the app itself. Typing in the search swaps the section
// for the settings whose names or words match; picking one opens its section, scrolls the
// row into view and lights it for a moment. The section and each one's scroll are session
// state kept on the frame, by section id.
import QtQuick
import QtQuick.Controls.Basic as QC
import com.marcusrosado.AniBeam

FocusScope {
    id: page
    objectName: "settings-page"
    property var props: ({})
    property string title: "Settings"
    property real scrollY: 0

    // ---- The sections, in index order; `run` starts a new run of the index
    readonly property var sections: [
        { id: "library", name: "Library", icon: "folder", run: 0 },
        { id: "accounts", name: "Accounts", icon: "user-check", run: 0 },
        { id: "appearance", name: "Appearance", icon: "palette", run: 1 },
        { id: "playback", name: "Playback", icon: "play", run: 1 },
        { id: "subtitles", name: "Subtitles", icon: "captions", run: 1 },
        { id: "keys", name: "Keyboard", icon: "keyboard", run: 1 },
        { id: "downloads", name: "Downloads", icon: "download", run: 2 },
        { id: "phone", name: "Phone app", icon: "radio", run: 2 },
        { id: "storage", name: "Storage and backup", icon: "hard-drive", run: 3 },
        { id: "about", name: "About", icon: "info", run: 3 }
    ].concat(frame.devOn ? [{ id: "developer", name: "Developer", icon: "bug", run: 3 }] : [])
    // Old tab names still open the section that took their place
    readonly property var aliases: ({ data: "storage", trackers: "accounts" })
    function sectionIndex(id) { for (var i = 0; i < sections.length; i++) if (sections[i].id === id) return i; return -1 }
    property string current: sectionIndex(frame.settingsTab) >= 0 ? frame.settingsTab : "library"
    onCurrentChanged: frame.settingsTab = current
    // Opens a section, and with `focus` the row or group of that name in it. The reveal waits
    // a beat in every case: a section just loaded, or one coming back from behind the search
    // results, has not laid out yet. Unsaved Downloads edits keep the section open: the bar
    // with Save stays where it is and says so.
    function open(id, focus) {
        if (id !== current && content.item && content.item.dirty === true) {
            frame.toast("Save or discard the download settings first")
            return
        }
        clearSearch()
        pendingFocus = focus || ""
        if (current === id) { if (pendingFocus !== "") focusLater.restart(); return }
        current = id
    }
    property string pendingFocus: ""
    // A row shown only in some states (a theme picker while colours come from the desktop)
    // falls back to the group that turns it on
    property string pendingFallback: ""
    function openResult(r) { open(r.section, r.label); pendingFallback = r.fallback || "" }
    function clearSearch() { query = ""; search.text = "" }

    Component.onCompleted: {
        var want = String(props.tab || "").toLowerCase()
        want = aliases[want] || want
        // The focus first: when `tab` names the section already loaded, nothing reloads and
        // the reveal has to be started here
        if (props.focus) pendingFocus = props.focus === "not-interested" ? "Hidden from Airing now" : props.focus
        if (sectionIndex(want) >= 0) current = want
        if (pendingFocus !== "") focusLater.restart()
        forceActiveFocus()
    }
    function focusSearch() { search.focusInput() }
    // Named escapePressed, not escape: QML's compiler reserves "escape" and refuses it as a
    // property, method or signal name on any object.
    function escapePressed() { if (query !== "") { clearSearch(); return true } return false }

    // ---- What each index entry says about its section, read live
    property var srcs: []
    property var downloads: null
    property var storage: null
    function readStatus() {
        var s = Door.listSources(); if (!s.error) srcs = s.reply.sources
        var d = Door.getDownloads(); if (!d.error) downloads = d.reply.status
        var g = Door.getStorage(); if (!g.error) storage = g.reply
    }
    Timer { id: statusDebounce; interval: 300; onTriggered: page.readStatus() }
    Connections {
        target: Door
        function onSourceChanged(s) { statusDebounce.restart() }
        function onSourceRemoved(s) { statusDebounce.restart() }
        function onScanFinished(s, a, c, r) { statusDebounce.restart() }
        function onDownloadsStatusChanged(status) { page.downloads = status }
        function onImagesCleared(removed) { statusDebounce.restart() }
    }
    Timer { interval: 0; running: true; onTriggered: page.readStatus() }

    function tracked(k) { return !!(Door.trackers[k] && Door.trackers[k].connected) }
    function statusOf(id) {
        if (id === "library") {
            var n = 0, gone = 0
            srcs.forEach(function(s) { n += s.series_count || 0; if (s.available === false) gone++ })
            if (!srcs.length) return "No folders yet"
            return (gone ? Fmt.plural(gone, "folder", "folders") + " not found, " : "")
                   + Fmt.plural(n, "series", "series") + " in " + Fmt.plural(srcs.length, "folder", "folders")
        }
        if (id === "accounts") {
            var on = []
            if (tracked("anilist")) on.push("AniList")
            if (tracked("mal")) on.push("MyAnimeList")
            return on.length ? "Syncing to " + on.join(" and ") : "No tracker connected"
        }
        if (id === "appearance") {
            var mode = Theme.mode === "system" ? "Follows the system" : Theme.mode === "light" ? "Light" : "Dark"
            return mode + ", " + (Theme.source === "theme" ? "theme colours" : "desktop colours")
        }
        if (id === "playback") {
            var sk = Door.settings.auto_skip || {}
            var langs = (Door.settings.subtitle_defaults || {}).audio_languages || []
            return (sk.intro && sk.outro ? "Skips intros and outros" : sk.intro ? "Skips intros" : sk.outro ? "Skips outros" : "Plays everything through")
                   + (langs.length ? ", audio " + langs.join(", ") : "")
        }
        if (id === "subtitles") {
            var sd = Door.settings.subtitle_defaults || {}
            var mode = ({ AsScripted: "styled files as made", ScaleOnly: "styled files keep their look", Force: "your style on every file" })[sd.ass_override || "ScaleOnly"]
            return "Size " + Math.round((sd.scale || 1) * 100) + " percent, " + mode
        }
        if (id === "keys") return "Every key the player answers to"
        if (id === "downloads") {
            if (!downloads) return "Not read yet"
            var t = downloads.torrent || {}
            if (t.connected) return "qBittorrent answered"
            var a = downloads.anirss || {}
            if (a.state === "Missing") return "anirss is not installed"
            return "Not connected yet"
        }
        if (id === "phone") return Door.preferences.share_over_tailscale ? "Shared on your tailnet" : "Not shared"
        if (id === "storage") return storage ? Fmt.bytes(storage.image_bytes || 0) + " of images cached" : "Images, export and import"
        if (id === "about") return frame.updatePanel.phase === "offer" && frame.updatePanel.shown ? "Version " + frame.updatePanel.version + " is out" : Shell.build
        if (id === "developer") return frame.devRecorder.last ? frame.devRecorder.last.label + " built in " + Math.round(frame.devRecorder.last.built) + " ms" : "Timing every page"
        return ""
    }
    // An entry that wants attention: a folder gone, downloads not set up
    function warns(id) {
        if (id === "library") return srcs.some(function(s) { return s.available === false })
        if (id === "about") return frame.updatePanel.phase === "offer" && frame.updatePanel.shown
        return false
    }

    // ---- The search: every row and group, by its name and the words a person might type
    // for it. `label` is the row's own label (or the group's title), so reveal finds it.
    readonly property var searchIndex: [
        ["library", "Folders", "add folder scan rescan remove source path collection"],
        ["library", "Matching", "metadata provider anilist tvmaze order match"],
        ["library", "Show hidden shows", "hidden reveal"],
        ["library", "Hidden from Airing now", "not interested airing show again"],
        ["accounts", "Trackers", "anilist myanimelist mal sign in connect login account disconnect"],
        ["accounts", "Main tracker", "progress cards primary", "Trackers"],
        ["accounts", "Sync now", "refresh progress lists tracker", "Trackers"],
        ["accounts", "Move to On hold", "on hold paused inactive untouched days", "Trackers"],
        ["appearance", "Mode", "dark light system theme night"],
        ["appearance", "Colours from", "terminal desktop system theme source kitty"],
        ["appearance", "Dark theme", "colour scheme base16", "Colours from"],
        ["appearance", "Light theme", "colour scheme base16", "Colours from"],
        ["appearance", "Accent", "colour highlight"],
        ["appearance", "Density", "compact comfortable spacing size"],
        ["appearance", "Poster size", "cards small medium large"],
        ["appearance", "Reduce motion", "animation accessibility"],
        ["playback", "Skip intros", "opening op auto skip aniskip chapters"],
        ["playback", "Skip outros", "ending ed credits auto skip aniskip"],
        ["playback", "Use my mpv settings", "mpv.conf config"],
        ["playback", "Subtitle languages", "language track subs"],
        ["playback", "Audio languages", "language track dub japanese"],
        ["subtitles", "Size", "scale subtitle"],
        ["subtitles", "Position", "height margin subtitle"],
        ["subtitles", "Styled subtitles", "ass override force"],
        ["subtitles", "Font", "typeface"],
        ["subtitles", "Text colour", "color"],
        ["subtitles", "Outline", "border stroke"],
        ["subtitles", "Shadow", ""],
        ["subtitles", "Background box", "box opacity"],
        ["subtitles", "Bold", "weight"],
        ["keys", "In the player", "keyboard shortcuts keys hotkeys"],
        ["keys", "Anywhere", "keyboard shortcuts keys hotkeys settings quit"],
        ["downloads", "Torrent client", "qbittorrent password username address webui seedbox"],
        ["downloads", "Search with anirss", "anirss rss feeds token command server"],
        ["downloads", "Rolling downloads", "roll episodes ahead delete watched keep disk space"],
        ["downloads", "Unsubscribe from finished shows", "prune feeds finished"],
        ["downloads", "Search sources", "endpoints nyaa 1337x config"],
        ["downloads", "Endpoint repos", "repo signed endpoints jackett prowlarr"],
        ["phone", "Share over Tailscale", "phone android tailnet remote host"],
        ["storage", "Back up the library", "export backup file json"],
        ["storage", "Restore from a backup", "import restore file json"],
        ["storage", "Image cache", "posters clear images disk cache"],
        ["storage", "Where AniBeam keeps its files", "database data config cache folder path"],
        ["about", "Version", "build what's new changelog"],
        ["about", "Updates", "stable nightly channel check"],
        ["about", "Developer options", "debug timing", "", "release"],
        ["about", "Ratings", "imdb credit"]
    ]
    property string query: ""
    readonly property var results: {
        var q = query.trim().toLowerCase()
        if (q === "") return []
        var words = q.split(/\s+/)
        var out = []
        searchIndex.forEach(function(e) {
            if (sectionIndex(e[0]) < 0) return
            // A switch only a release build shows
            if (e[4] === "release" && Dev.forced) return
            var hay = (e[1] + " " + e[2] + " " + sections[sectionIndex(e[0])].name).toLowerCase()
            var named = e[1].toLowerCase().indexOf(q) >= 0
            if (words.every(function(w) { return hay.indexOf(w) >= 0 })) out.push({ section: e[0], label: e[1], named: named, fallback: e[3] || "" })
        })
        out.sort(function(a, b) { return (b.named ? 1 : 0) - (a.named ? 1 : 0) })
        return out
    }

    // ---- Geometry: the page's centred column, the index at its left
    readonly property real sideInset: Math.round((page.width - Math.min(page.width, theme.contentMax)) / 2) + theme.space(8)
    readonly property real topInset: theme.space(7)
    readonly property real indexWidth: page.width < theme.space(270) ? theme.space(60) : theme.space(74)
    readonly property real columnGap: theme.space(14)

    // ---- The head: the title, the search at the right of the index's column
    Item {
        id: head
        x: page.sideInset
        y: page.topInset
        width: page.width - page.sideInset * 2
        height: theme.space(20)
        Label {
            anchors.left: parent.left
            anchors.verticalCenter: parent.verticalCenter
            text: "Settings"
            role: "display"
            font.pointSize: theme.typePage
        }
    }
    // ---- The index
    Flickable {
        id: index
        objectName: "settings-index"
        x: page.sideInset
        y: head.y + head.height + theme.space(4)
        width: page.indexWidth
        height: page.height - y
        contentHeight: indexCol.height + theme.space(10)
        clip: true
        boundsBehavior: Flickable.StopAtBounds
        WheelStep { flick: index }
        Column {
            id: indexCol
            width: index.width
            SearchField {
                id: search
                objectName: "settings-search"
                width: parent.width
                placeholder: "Search settings"
                hint: "/"
                onTextChanged: page.query = text
                onAccepted: if (page.results.length) page.openResult(page.results[0])
            }
            Item { width: 1; height: theme.space(5) }
            Repeater {
                model: page.sections
                Item {
                    id: entry
                    required property var modelData
                    required property int index
                    readonly property bool active: page.query === "" && page.current === modelData.id
                    readonly property bool runStart: index > 0 && page.sections[index - 1].run !== modelData.run
                    objectName: "settings-tabs-" + modelData.id
                    width: indexCol.width
                    height: theme.space(15) + (runStart ? theme.space(4) : 0)
                    Rectangle {
                        id: plate
                        y: entry.runStart ? theme.space(4) : 0
                        width: parent.width
                        height: theme.space(15)
                        radius: theme.radiusTile
                        color: entry.active ? theme.accentSoft : hover.containsMouse ? theme.tile : "transparent"
                        border.width: entry.active ? 1 : 0
                        border.color: theme.accentEdge
                        Icon {
                            id: glyph
                            x: theme.space(4)
                            anchors.verticalCenter: parent.verticalCenter
                            glyph: entry.modelData.icon
                            size: theme.space(5)
                            color: entry.active ? theme.accent : theme.text
                        }
                        Column {
                            anchors.left: glyph.right
                            anchors.leftMargin: theme.space(3.5)
                            anchors.right: parent.right
                            anchors.rightMargin: theme.space(3)
                            anchors.verticalCenter: parent.verticalCenter
                            spacing: theme.space(0.5)
                            Label {
                                width: parent.width
                                text: entry.modelData.name
                                role: "title"
                                font.pointSize: theme.typeNormal
                                color: entry.active ? theme.accent : theme.text
                                elide: Text.ElideRight
                            }
                            Row {
                                width: parent.width
                                spacing: theme.space(1.5)
                                Rectangle {
                                    visible: page.warns(entry.modelData.id)
                                    anchors.verticalCenter: parent.verticalCenter
                                    width: theme.space(1.5); height: width; radius: width / 2
                                    color: theme.yellow
                                }
                                Label {
                                    width: parent.width - (page.warns(entry.modelData.id) ? theme.space(3) : 0)
                                    text: page.statusOf(entry.modelData.id)
                                    role: "ui"
                                    font.pointSize: theme.typeSmall
                                    color: theme.textDim
                                    elide: Text.ElideRight
                                }
                            }
                        }
                        MouseArea {
                            id: hover
                            anchors.fill: parent
                            hoverEnabled: true
                            cursorShape: Qt.PointingHandCursor
                            onClicked: page.open(entry.modelData.id)
                        }
                    }
                }
            }
        }
    }
    // ---- The section, or the search's results while there is a query
    Item {
        id: stage
        x: index.x + index.width + page.columnGap
        y: index.y
        width: page.width - x - page.sideInset + theme.space(6)
        height: page.height - y
        Loader {
            id: content
            anchors.fill: parent
            // Hidden by opacity, not visibility, so the section keeps its layout and its scroll
            // under the results
            opacity: page.query === "" ? 1 : 0
            enabled: page.query === ""
            source: "Settings" + ({ library: "Library", accounts: "Accounts", appearance: "Appearance", playback: "Playback", subtitles: "Subtitles",
                                     keys: "Keys", downloads: "Downloads", phone: "Phone", storage: "Data", about: "About", developer: "Developer" })[page.current] + "Tab.qml"
            onLoaded: {
                item.contentY = frame.settingsScroll[page.current] || 0
                if (page.pendingFocus !== "") focusLater.restart()
            }
            Timer {
                id: focusLater; interval: 150
                onTriggered: {
                    if (content.item && !content.item.reveal(page.pendingFocus) && page.pendingFallback !== "") content.item.reveal(page.pendingFallback)
                    page.pendingFocus = ""; page.pendingFallback = ""
                }
            }
            Connections {
                target: content.item
                function onContentYChanged() { var s = Object.assign({}, frame.settingsScroll); s[page.current] = content.item.contentY; frame.settingsScroll = s }
            }
        }
        // The results: one row per match, the section it lives in under its name
        Flickable {
            id: found
            anchors.fill: parent
            visible: page.query !== ""
            contentHeight: foundCol.height + theme.space(10)
            clip: true
            boundsBehavior: Flickable.StopAtBounds
            WheelStep { flick: found }
            Column {
                id: foundCol
                width: Math.min(found.width - theme.space(6), theme.space(210))
                spacing: theme.space(2)
                Label {
                    text: page.results.length ? Fmt.plural(page.results.length, "setting", "settings") + " match" + (page.results.length === 1 ? "es" : "") : "Nothing matches"
                    role: "display"
                    font.pointSize: theme.typeHeading
                }
                Label {
                    width: parent.width
                    text: page.results.length ? "Return opens the first one." : "Try a plainer word: theme, subtitles, tracker, password, folder."
                    color: theme.textDim
                    role: "ui"
                    bottomPadding: theme.space(6)
                }
                Rectangle {
                    visible: page.results.length > 0
                    width: parent.width
                    height: resultsCol.height + theme.space(3)
                    radius: theme.radiusTile
                    color: theme.tile
                    Column {
                        id: resultsCol
                        x: theme.space(1.5); y: theme.space(1.5)
                        width: parent.width - theme.space(3)
                        Repeater {
                            model: page.results
                            Rectangle {
                                id: hit
                                required property var modelData
                                required property int index
                                objectName: "settings-result-" + index
                                width: resultsCol.width
                                height: theme.space(15)
                                radius: theme.radiusInner(theme.space(1.5))
                                color: hitMouse.containsMouse ? theme.raised : "transparent"
                                Icon {
                                    id: hitGlyph
                                    x: theme.space(3.5)
                                    anchors.verticalCenter: parent.verticalCenter
                                    glyph: page.sections[page.sectionIndex(hit.modelData.section)].icon
                                    color: theme.textDim
                                }
                                Column {
                                    anchors.left: hitGlyph.right; anchors.leftMargin: theme.space(3.5)
                                    anchors.verticalCenter: parent.verticalCenter
                                    spacing: theme.space(0.5)
                                    Label { text: hit.modelData.label; role: "ui" }
                                    Label { text: page.sections[page.sectionIndex(hit.modelData.section)].name; role: "ui"; font.pointSize: theme.typeSmall; color: theme.textDim }
                                }
                                Icon {
                                    anchors.right: parent.right; anchors.rightMargin: theme.space(4)
                                    anchors.verticalCenter: parent.verticalCenter
                                    glyph: "chevron-right"
                                    size: theme.space(4)
                                    color: theme.textFaint
                                }
                                MouseArea { id: hitMouse; anchors.fill: parent; hoverEnabled: true; cursorShape: Qt.PointingHandCursor; onClicked: page.openResult(hit.modelData) }
                            }
                        }
                    }
                }
            }
        }
    }
}
