// Settings: the index at the left with the settings search on top of it, and the open
// section beside it. The index is a plain menu: each entry is a glyph and the section's
// name, with a red dot at its right when the section wants attention (a folder gone, no
// list connected, an update waiting). Typing in the search swaps the section for the
// settings whose names or words match; picking one opens its section, scrolls the row into
// view and lights it for a moment. The section and each one's scroll are session state
// kept on the frame, by section id. Section ids that have since moved into another
// section (subtitles, phone, storage, help) still open the section that took them.
import QtQuick
import com.marcusrosado.AniBeam

FocusScope {
    id: page
    objectName: "settings-page"
    property var props: ({})
    property string title: words.t("settings-title")
    property real scrollY: 0

    // ---- The sections, in index order
    readonly property var sections: [
        { id: "library", name: words.t("settings-section-library"), icon: "folder" },
        { id: "accounts", name: words.t("settings-section-accounts"), icon: "user-check" },
        { id: "appearance", name: words.t("settings-section-appearance"), icon: "palette" },
        { id: "playback", name: words.t("settings-section-playback"), icon: "play" },
        { id: "downloads", name: words.t("settings-section-downloads"), icon: "download" },
        { id: "keys", name: words.t("settings-section-keys"), icon: "keyboard" },
        { id: "about", name: words.t("settings-section-about"), icon: "info" }
    ].concat(frame.devOn ? [{ id: "developer", name: words.t("settings-section-developer"), icon: "bug" }] : [])
    // Old tab names still open the section that took their place
    readonly property var aliases: ({ subtitles: "playback", phone: "accounts", storage: "about", data: "about", help: "about", trackers: "accounts" })
    function sectionIndex(id) { for (var i = 0; i < sections.length; i++) if (sections[i].id === id) return i; return -1 }
    property string current: sectionIndex(aliases[frame.settingsTab] || frame.settingsTab) >= 0 ? (aliases[frame.settingsTab] || frame.settingsTab) : "library"
    onCurrentChanged: frame.settingsTab = current
    // Opens a section, and with `focus` the row or group of that name in it. The reveal waits
    // a beat in every case: a section just loaded, or one coming back from behind the search
    // results, has not laid out yet. Unsaved Downloads edits ride along on the frame
    // (settingsDrafts) and come back with the section.
    function open(id, focus) {
        clearSearch()
        pendingFocus = focus || ""
        if (current === id) { if (pendingFocus !== "") focusLater.restart(); return true }
        current = id
        return true
    }
    property string pendingFocus: ""
    // A row shown only in some states (a theme picker while colours come from the desktop)
    // falls back to the group that turns it on
    property string pendingFallback: ""
    function openResult(r) { if (open(r.section, r.label)) pendingFallback = r.fallback || "" }
    function clearSearch() { query = ""; search.text = "" }

    Component.onCompleted: {
        var want = String(props.tab || "").toLowerCase()
        want = aliases[want] || want
        // The focus first: when `tab` names the section already loaded, nothing reloads and
        // the reveal has to be started here
        if (props.focus) pendingFocus = props.focus === "not-interested" ? words.t("settings-library-hidden-from-airing") : props.focus
        if (sectionIndex(want) >= 0) current = want
        if (pendingFocus !== "") focusLater.restart()
        forceActiveFocus()
    }
    function focusSearch() { search.focusInput() }
    // Named escapePressed, not escape: QML's compiler reserves "escape" and refuses it as a
    // property, method or signal name on any object.
    function escapePressed() { if (query !== "") { clearSearch(); return true } return false }

    // ---- Which index entries want attention, read live
    property var srcs: []
    function readSources() { var s = Door.listSources(); if (!s.error) srcs = s.reply.sources }
    Timer { id: sourcesDebounce; interval: 300; onTriggered: page.readSources() }
    Connections {
        target: Door
        function onSourceChanged(s) { sourcesDebounce.restart() }
        function onSourceRemoved(s) { sourcesDebounce.restart() }
        function onScanFinished(s, a, c, r) { sourcesDebounce.restart() }
    }
    Timer { interval: 0; running: true; onTriggered: page.readSources() }
    function tracked(k) { return !!(Door.trackers[k] && Door.trackers[k].connected) }
    // An entry that wants attention: a folder gone, no list connected, an update waiting
    function warns(id) {
        if (id === "library") return srcs.some(function(s) { return s.available === false })
        if (id === "accounts") return !tracked("anilist") && !tracked("mal")
        if (id === "about") return frame.updatePanel.pending
        return false
    }

    // ---- The search: every row and group, by its name and the words a person might type
    // for it. `label` is the row's own label (or the group's title), so reveal finds it.
    // Built only when a query is typed: it is some 140 catalog lookups nobody else reads.
    function searchIndex() { return [
        ["library", words.t("settings-library-folders"), words.t("settings-find-folders.words")],
        ["library", words.t("settings-library-matching"), words.t("settings-find-matching.words")],
        ["library", words.t("settings-library-show-hidden"), words.t("settings-find-hidden-shows.words")],
        ["library", words.t("settings-library-hidden-from-airing"), words.t("settings-find-hidden-airing.words")],
        ["library", words.t("settings-library-calendar-feed"), words.t("settings-find-calendar.words")],
        ["accounts", words.t("settings-accounts-lists"), words.t("settings-find-anime-lists.words")],
        ["accounts", words.t("settings-accounts-main-list"), words.t("settings-find-main-list.words"), words.t("settings-accounts-lists")],
        ["accounts", words.t("settings-accounts-sync-now"), words.t("settings-find-sync-now.words"), words.t("settings-accounts-lists")],
        ["accounts", words.t("settings-accounts-move-on-hold"), words.t("settings-find-on-hold.words"), words.t("settings-accounts-lists")],
        ["appearance", words.t("settings-appearance-language"), words.t("settings-find-language.words")],
        ["appearance", words.t("settings-appearance-title-language"), words.t("settings-find-title-language.words")],
        ["appearance", words.t("settings-appearance-mode"), words.t("settings-find-mode.words")],
        ["appearance", words.t("settings-appearance-colours-from"), words.t("settings-find-colours-from.words")],
        ["appearance", words.t("settings-appearance-dark-theme"), words.t("settings-find-dark-theme.words"), words.t("settings-appearance-colours-from")],
        ["appearance", words.t("settings-appearance-light-theme"), words.t("settings-find-light-theme.words"), words.t("settings-appearance-colours-from")],
        ["appearance", words.t("settings-appearance-accent"), words.t("settings-find-accent.words")],
        ["appearance", words.t("settings-appearance-text-size"), words.t("settings-find-text-size.words")],
        ["appearance", words.t("settings-appearance-density"), words.t("settings-find-density.words")],
        ["appearance", words.t("settings-appearance-poster-size"), words.t("settings-find-poster-size.words")],
        ["appearance", words.t("settings-appearance-out-now"), words.t("settings-find-out-now.words")],
        ["appearance", words.t("settings-appearance-out-now-grid"), words.t("settings-find-out-now-cards.words"), words.t("settings-appearance-out-now")],
        ["appearance", words.t("settings-appearance-reduce-motion"), words.t("settings-find-reduce-motion.words")],
        ["appearance", words.t("settings-appearance-app-icon"), words.t("settings-find-app-icon.words")],
        ["playback", words.t("settings-playback-skip-intros"), words.t("settings-find-skip-intros.words")],
        ["playback", words.t("settings-playback-skip-outros"), words.t("settings-find-skip-outros.words")],
        ["playback", words.t("settings-playback-my-mpv"), words.t("settings-find-mpv-settings.words")],
        ["playback", words.t("settings-playback-discord"), words.t("settings-find-discord.words")],
        ["playback", words.t("settings-playback-name-next"), words.t("settings-find-next-episode.words")],
        ["playback", words.t("settings-playback-subtitle-languages"), words.t("settings-find-subtitle-languages.words")],
        ["playback", words.t("settings-playback-audio-languages"), words.t("settings-find-audio-languages.words")],
        ["playback", words.t("settings-playback-room-name"), words.t("settings-find-room-name.words")],
        ["playback", words.t("settings-playback-room-relay"), words.t("settings-find-room-relay.words")],
        ["playback", words.t("settings-subtitles-start-over"), words.t("settings-find-start-over.words")],
        ["playback", words.t("settings-subtitles-size"), words.t("settings-find-size.words")],
        ["playback", words.t("settings-subtitles-position"), words.t("settings-find-position.words")],
        ["playback", words.t("settings-subtitles-styled"), words.t("settings-find-styled.words")],
        ["playback", words.t("settings-subtitles-font"), words.t("settings-find-font.words")],
        ["playback", words.t("settings-subtitles-text-colour"), words.t("settings-find-text-colour.words")],
        ["playback", words.t("settings-subtitles-outline"), words.t("settings-find-outline.words")],
        ["playback", words.t("settings-subtitles-shadow"), ""],
        ["playback", words.t("settings-subtitles-box"), words.t("settings-find-background.words")],
        ["playback", words.t("settings-subtitles-bold"), words.t("settings-find-bold.words")],
        ["keys", words.t("settings-keys-player"), words.t("settings-find-keys-player.words")],
        ["keys", words.t("settings-keys-player-around"), words.t("settings-find-keys-player-around.words")],
        ["keys", words.t("settings-keys-around"), words.t("settings-find-keys-around.words")],
        ["keys", words.t("settings-keys-anywhere"), words.t("settings-find-keys-anywhere.words")],
        ["downloads", words.t("settings-downloads-torrent-title"), words.t("settings-find-torrent-client.words")],
        ["downloads", words.t("settings-downloads-search-helper"), words.t("settings-find-search-helper.words")],
        ["downloads", words.t("settings-downloads-roll-title"), words.t("settings-find-next-few.words")],
        ["downloads", words.t("settings-downloads-prune"), words.t("settings-find-unsubscribe.words")],
        ["downloads", words.t("settings-downloads-last-search"), words.t("settings-find-last-search.words")],
        ["downloads", words.t("settings-downloads-sources-title"), words.t("settings-find-search-sources.words")],
        ["downloads", words.t("endpoint-repos-panel-title"), words.t("settings-find-source-lists.words")],
        ["accounts", words.t("settings-phone-share"), words.t("settings-find-share-tailscale.words")],
        ["accounts", words.t("settings-phone-stream"), words.t("settings-find-stream.words")],
        ["accounts", words.t("settings-phone-devices"), words.t("settings-find-devices.words")],
        ["about", words.t("settings-data-back-up-library"), words.t("settings-find-back-up.words")],
        ["about", words.t("settings-data-restore-from"), words.t("settings-find-restore.words")],
        ["about", words.t("settings-data-image-cache"), words.t("settings-find-image-cache.words")],
        ["about", words.t("settings-data-files"), words.t("settings-find-files.words")],
        ["about", words.t("settings-help-folders"), words.t("settings-find-help-folders.words")],
        ["about", words.t("settings-help-more"), words.t("settings-find-help-else.words")],
        ["about", words.t("settings-about-version"), words.t("settings-find-version.words")],
        ["about", words.t("settings-about-setup"), words.t("settings-find-setup.words")],
        ["about", words.t("settings-about-updates"), words.t("settings-find-updates.words")],
        ["about", words.t("settings-about-developer"), words.t("settings-find-developer-options.words"), "", "unforced"],
        ["about", words.t("settings-about-ratings"), words.t("settings-find-ratings.words")]
    ] }
    property string query: ""
    readonly property var results: {
        var q = query.trim().toLowerCase()
        if (q === "") return []
        var terms = q.split(/\s+/)
        var out = []
        searchIndex().forEach(function(e) {
            if (sectionIndex(e[0]) < 0) return
            // The switch ANIBEAM_DEV=1 hides
            if (e[4] === "unforced" && Dev.forced) return
            var hay = (e[1] + " " + e[2] + " " + sections[sectionIndex(e[0])].name).toLowerCase()
            var named = e[1].toLowerCase().indexOf(q) >= 0
            if (terms.every(function(w) { return hay.indexOf(w) >= 0 })) out.push({ section: e[0], label: e[1], named: named, fallback: e[3] || "" })
        })
        out.sort(function(a, b) { return (b.named ? 1 : 0) - (a.named ? 1 : 0) })
        return out
    }

    // ---- Geometry: the page's centred column, the index at its left, the search on top of it
    readonly property real sideInset: Math.round((page.width - Math.min(page.width, theme.contentMax)) / 2) + theme.space(8)
    readonly property real topInset: theme.space(7)
    readonly property real indexWidth: page.width < theme.space(270) ? theme.space(60) : theme.space(74)
    readonly property real columnGap: theme.space(14)

    // ---- The index
    Flickable {
        id: indexPane
        objectName: "settings-index"
        x: page.sideInset
        y: page.topInset
        width: page.indexWidth
        height: page.height - y
        contentHeight: indexCol.height + theme.space(10)
        clip: true
        boundsBehavior: Flickable.StopAtBounds
        WheelStep { flick: indexPane }
        Column {
            id: indexCol
            width: indexPane.width
            SearchField {
                id: search
                objectName: "settings-search"
                width: parent.width
                placeholder: words.t("settings-search-placeholder")
                hint: "/"
                onTextChanged: page.query = text
                onAccepted: if (page.results.length) page.openResult(page.results[0])
            }
            Item { width: 1; height: theme.space(4) }
            Repeater {
                model: page.sections
                Rectangle {
                    id: entry
                    required property var modelData
                    readonly property bool active: page.query === "" && page.current === modelData.id
                    objectName: "settings-tabs-" + modelData.id
                    width: indexCol.width
                    height: theme.space(11)
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
                    Label {
                        anchors.left: glyph.right
                        anchors.leftMargin: theme.space(3.5)
                        anchors.right: dot.left
                        anchors.rightMargin: theme.space(2)
                        anchors.verticalCenter: parent.verticalCenter
                        text: entry.modelData.name
                        role: "title"
                        font.pointSize: theme.typeNormal
                        color: entry.active ? theme.accent : theme.text
                        elide: Text.ElideRight
                    }
                    // The section wants attention
                    Rectangle {
                        id: dot
                        objectName: "settings-tabs-" + entry.modelData.id + "-dot"
                        visible: page.warns(entry.modelData.id)
                        anchors.right: parent.right
                        anchors.rightMargin: theme.space(4)
                        anchors.verticalCenter: parent.verticalCenter
                        width: theme.space(2); height: width; radius: width / 2
                        color: theme.red
                    }
                    MouseArea {
                        Accessible.role: Accessible.Button
                        Accessible.onPressAction: page.open(entry.modelData.id)
                        Accessible.name: theme.nameNear(parent)
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
    // ---- The section, or the search's results while there is a query
    Item {
        id: stage
        x: indexPane.x + indexPane.width + page.columnGap
        y: indexPane.y
        width: page.width - x - page.sideInset + theme.space(6)
        height: page.height - y
        Loader {
            id: content
            anchors.fill: parent
            // Hidden by opacity, not visibility, so the section keeps its layout and its scroll
            // under the results
            opacity: page.query === "" ? 1 : 0
            enabled: page.query === ""
            source: "Settings" + ({ library: "Library", accounts: "Accounts", appearance: "Appearance", playback: "Playback",
                                     downloads: "Downloads", keys: "Keys", about: "About", developer: "Developer" })[page.current] + "Tab.qml"
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
                    text: page.results.length ? words.t("settings-results", { count: page.results.length }) : words.t("settings-results-none")
                    role: "display"
                    font.pointSize: theme.typeHeading
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
                                MouseArea { id: hitMouse; anchors.fill: parent; hoverEnabled: true; cursorShape: Qt.PointingHandCursor; onClicked: page.openResult(hit.modelData); Accessible.role: Accessible.Button; Accessible.name: theme.nameNear(parent)
                                    Accessible.onPressAction: page.openResult(hit.modelData) }
                            }
                        }
                    }
                }
            }
        }
    }
}
