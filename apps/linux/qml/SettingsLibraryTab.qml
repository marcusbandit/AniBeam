// The Library tab, flat sections: the figures (the same counts the Metadata filter
// shows), Folders sized to its sources, and the recent scans growing into the rest of
// the column, as many as it holds; beside them Trackers (with On hold under the accounts
// while one is connected), the metadata providers in the order matching tries them, the
// library options at their own height, and the series on disk, most files first, filling
// the rest.
import QtQuick
import QtQuick.Dialogs
import com.marcusrosado.AniBeam

SettingsTab {
    id: tab
    property var sources: []
    property var stats: ({ series: 0, films: 0, episodes: 0, lastScan: "never" })
    property var scans: []                  // the ScanFinished events, newest first
    property var disk: []                   // the Metadata rows, most episodes on disk first
    function sourcePath(id) { for (var i = 0; i < sources.length; i++) if (sources[i].id === id) return sources[i].path; return "" }
    function reload() {
        var r = Door.listSources()
        if (!r.error) sources = r.reply.sources
        // The counts are the Metadata page's own (one rule for films: a Movie folder or
        // a MOVIE format), and the episodes are its files on disk, so every page agrees
        var m = Door.listMetadata("All", "", false)
        if (m.error) return
        var eps = 0
        m.reply.rows.forEach(function(row) { eps += row.have || 0 })
        // Source carries no last-scan time of its own (core/src/contract/records.rs), so
        // this reads it off the event history instead: recentEvents' Reply::Events serialises
        // each Event.body through plain serde, externally tagged as { ScanFinished: {...} },
        // not through the door.rs event_json helper that flattens a live-pushed envelope to
        // a top-level "kind"; there is no e.kind here.
        var ev = Door.recentEvents(2000)
        var found = []
        if (!ev.error) ev.reply.events.forEach(function(e) { if (e.body && e.body.ScanFinished !== undefined) found.push({ at: e.at, scan: e.body.ScanFinished }) })
        found.sort(function(a, b) { return b.at - a.at })
        scans = found
        disk = m.reply.rows.filter(function(row) { return (row.have || 0) + (row.extra_on_disk || 0) > 0 })
                           .sort(function(a, b) { return (b.have || 0) - (a.have || 0) || (a.series.title < b.series.title ? -1 : 1) })
        var last = found.length ? Fmt.relative(found[0].at, Date.now() / 1000) : "never"
        stats = { series: m.reply.counts.series, films: m.reply.counts.movies, episodes: eps, lastScan: last }
    }
    // The tailnet host: whether it is on, what it says about itself and the URLs it serves
    property var host: ({ enabled: false, urls: [], message: "" })
    function reloadHost() { var r = Door.getHost(); if (!r.error) host = r.reply.status }
    function setSharing(on) {
        var p = JSON.parse(JSON.stringify(Door.preferences))
        p.share_over_tailscale = on
        var r = Door.setPreferences(p)
        if (r.error) frame.toast(r.error.message)
    }
    Component.onCompleted: { reload(); reloadHost() }
    // On hold is a list on a tracker, so its two rows show only while one is connected
    readonly property bool tracked: !!(Door.trackers.anilist && Door.trackers.anilist.connected) || !!(Door.trackers.mal && Door.trackers.mal.connected)
    function setOnHold(on) {
        var p = JSON.parse(JSON.stringify(Door.preferences))
        p.on_hold_auto = on
        var r = Door.setPreferences(p)
        if (r.error) frame.toast(r.error.message)
    }
    // The days are written once the slider rests, so a drag is one write
    property int pendingDays: 60
    Timer {
        id: daysDebounce
        interval: 400
        onTriggered: {
            var p = JSON.parse(JSON.stringify(Door.preferences))
            p.on_hold_days = tab.pendingDays
            var r = Door.setPreferences(p)
            if (r.error) frame.toast(r.error.message)
        }
    }
    // The match providers, first tried first; the core keeps the order and says what it is
    readonly property var providerOrder: Door.settings && Door.settings.provider_order && Door.settings.provider_order.length ? Door.settings.provider_order : ["Anilist", "Tvmaze"]
    readonly property var providerAbout: ({ Anilist: "Anime: series, films and their relations. Episodes you watch are marked on your trackers.",
                                           Tvmaze: "Other television: live action, cartoons, documentaries. Nothing is marked on a tracker." })
    function ordinal(i) { return (["Tried first", "Tried second", "Tried third", "Tried fourth"])[i] || "Tried " + (i + 1) + "th" }
    function moveProvider(from, to) {
        if (to < 0 || to >= providerOrder.length) return
        var o = providerOrder.slice()
        var p = o.splice(from, 1)[0]
        o.splice(to, 0, p)
        var r = Door.setProviderOrder(o)
        if (r.error) frame.toast(r.error.message)
    }
    Timer { id: debounce; interval: 250; onTriggered: tab.reload() }
    Connections {
        target: Door
        function onSourceChanged(s) { debounce.restart() }
        function onSourceRemoved(s) { debounce.restart() }
        function onScanFinished(s, a, c, r) { debounce.restart() }
        function onSeriesChanged(c) { debounce.restart() }
        function onSeriesRemoved(ids) { debounce.restart() }
        function onProgressRefreshed() { frame.toast("Tracker progress synced") }
        function onHostChanged(status) { tab.host = status }
    }

    SettingsPair {
        split: 3 / 5
        leftPanels: [ statStones, foldersPanel, scansPanel ]
        rightPanels: [ trackersPanel, providersPanel, optionsPanel, sharePanel, diskPanel ]
    }
    Component {
        id: statStones
        Tiles { tiles: [{ value: String(tab.stats.series), caption: "Series" }, { value: String(tab.stats.films), caption: "Films" }, { value: tab.stats.episodes.toLocaleString(Qt.locale(), "f", 0), caption: "Episodes on disk" }, { value: tab.stats.lastScan, caption: "Last scan" }] }
    }
    Component {
        id: foldersPanel
        Panel {
            objectName: "settings-folders"
            title: "Folders"; icon: "folder-open"
            helper: "AniBeam scans these folders for video files. A folder is a series; a file at the top level of a Movies folder is a film."
            Column {
                width: parent.width
                Repeater {
                    model: tab.sources
                    SourceRow {
                        required property var modelData
                        source: modelData
                        onOpen: frame.openLink("file://" + modelData.path)
                        onRescan: { var r = Door.scan(modelData.id); frame.toast(r.error ? r.error.message : "Rescan started") }
                        onRemove: confirming = true
                        onRemoveAccepted: { var r = Door.removeSource(modelData.id); if (r.error) frame.toast(r.error.message) }
                    }
                }
                Label { visible: tab.sources.length === 0; width: parent.width; wrapMode: Text.Wrap; text: "No folders yet. Add a folder to point AniBeam at your collection."; color: theme.textDim; role: "ui"; topPadding: theme.space(2); bottomPadding: theme.space(2) }
            }
            Row { spacing: theme.space(2)
                Button { objectName: "settings-add-folder"; text: "Add folder"; icon: "folder-plus"; onClicked: folderDialog.open() }
                Button { text: "Scan all"; icon: "refresh-cw"; enabled: tab.sources.length > 0; onClicked: { var r = Door.scan(-1); frame.toast(r.error ? r.error.message : "Scan started") } } }
        }
    }
    // The scans the folders have had, newest first: what each one found
    Component {
        id: scansPanel
        Panel {
            title: "Recent scans"; icon: "scan"; grows: true
            helper: tab.scans.length ? "" : "No scans yet. Scan all reads every folder again."
            stretch: FillRows {
                objectName: "settings-scans"
                anchors.fill: parent
                model: tab.scans
                delegate: Item {
                    required property var modelData
                    width: parent ? parent.width : 0
                    height: theme.space(10)
                    readonly property var s: modelData.scan
                    Label {
                        anchors.left: parent.left; anchors.right: when.left; anchors.rightMargin: theme.space(4)
                        anchors.verticalCenter: parent.verticalCenter
                        elide: Text.ElideMiddle
                        role: "ui"
                        text: (parent.s.source !== null && parent.s.source !== undefined && tab.sourcePath(parent.s.source) !== "" ? tab.sourcePath(parent.s.source) : "Every folder") + ": "
                              + (parent.s.added + parent.s.changed + parent.s.removed === 0 ? "nothing changed"
                                 : [parent.s.added ? parent.s.added + " added" : "", parent.s.changed ? parent.s.changed + " changed" : "", parent.s.removed ? parent.s.removed + " removed" : ""].filter(Boolean).join(", "))
                    }
                    Label {
                        id: when
                        anchors.right: parent.right; anchors.verticalCenter: parent.verticalCenter
                        text: Fmt.relative(modelData.at, Date.now() / 1000)
                        role: "ledger"; color: theme.textDim
                    }
                    Rectangle { anchors.bottom: parent.bottom; width: parent.width; height: 1; color: theme.line; opacity: 0.5 }
                }
            }
        }
    }
    Component {
        id: trackersPanel
        Panel {
            title: "Trackers"; icon: "user-check"
            helper: "Episodes are marked on every connected tracker when you reach the outro or mark them by hand. Counts only go up."
            // The connect flow itself is the frame's: the rows start it, show it and stop it
            // through frame.openConnect and the state beside it.
            TrackerRow { tracker: "Anilist"; account: Door.trackers.anilist || ({})
                onDisconnect: { var r = Door.disconnectTracker("Anilist"); if (r.error) frame.toast(r.error.message) } }
            TrackerRow { tracker: "Mal"; account: Door.trackers.mal || ({})
                onDisconnect: { var r = Door.disconnectTracker("Mal"); if (r.error) frame.toast(r.error.message) } }
            SettingRow { label: "Main tracker"; helper: "Whose progress the cards show. The other tracker still gets every update."
                Seg { options: ["AniList", "MyAnimeList"]; index: Door.trackers.main === "Mal" ? 1 : 0; onPicked: function(i) { var r = Door.setMainTracker(i === 1 ? "Mal" : "Anilist"); if (r.error) frame.toast(r.error.message) } } }
            SettingRow { visible: tab.tracked; label: "Move to On hold"
                helper: "A Watching series nobody has touched for the days set below moves to On hold on your trackers. One waiting for its next episode stays. Checked when AniBeam starts and on Sync."
                Switch { objectName: "settings-on-hold"; checked: Door.preferences.on_hold_auto !== false; onToggled: function(on) { tab.setOnHold(on) } } }
            SettingRow { visible: tab.tracked && Door.preferences.on_hold_auto !== false; label: "Days untouched"
                SliderRow { objectName: "settings-on-hold-days"; from: 7; to: 365; stepSize: 1; value: Door.preferences.on_hold_days || 60
                    onMoved: function(v) { tab.pendingDays = Math.round(v); daysDebounce.restart() } } }
            SettingRow { label: "Sync progress now"; helper: "Reads your lists from every connected tracker again."
                Button { objectName: "settings-sync-trackers"; text: "Sync"; icon: "refresh-cw"; enabled: !!(Door.trackers.anilist && Door.trackers.anilist.connected) || !!(Door.trackers.mal && Door.trackers.mal.connected)
                    onClicked: { var r = Door.refreshProgress(""); frame.toast(r.error ? r.error.message : "Syncing tracker progress") } } }
        }
    }
    Component {
        id: providersPanel
        Panel {
            objectName: "settings-providers"
            title: "Metadata providers"; icon: "database"
            helper: "Matching a folder tries each provider from the top and keeps the first good match. A match you set by hand is never replaced."
            Repeater {
                model: tab.providerOrder
                SettingRow {
                    required property var modelData
                    required property int index
                    objectName: "provider-row-" + String(modelData).toLowerCase()
                    label: frame.providerName(modelData)
                    line: tab.ordinal(index)
                    helper: tab.providerAbout[modelData] || ""
                    Row {
                        spacing: theme.space(1)
                        Button { objectName: "provider-up-" + String(modelData).toLowerCase(); text: "Move up"; icon: "arrow-up"; flat: true; small: true
                            enabled: index > 0; onClicked: tab.moveProvider(index, index - 1) }
                        Button { objectName: "provider-down-" + String(modelData).toLowerCase(); text: "Move down"; icon: "arrow-down"; flat: true; small: true
                            enabled: index < tab.providerOrder.length - 1; onClicked: tab.moveProvider(index, index + 1) }
                    }
                }
            }
        }
    }
    Component {
        id: optionsPanel
        Panel {
            title: "Library options"; icon: "sliders-horizontal"
            SettingRow { label: "Show hidden shows"; helper: "Shows hidden series on every page until AniBeam closes."
                Switch { objectName: "settings-reveal-hidden"; checked: Door.revealHidden; onToggled: function(on) { Door.revealHidden = on } } }
            SettingRow { label: "Downloads"; helper: "Search the release feeds, subscribe, and watch what the torrent client is doing."
                Button { text: "Open"; icon: "download"; onClicked: frame.go("downloads") } }
            SettingRow { label: "Metadata"; helper: "Every series, the record it is matched to, and its files on disk."
                Button { text: "Open"; icon: "database"; onClicked: frame.go("metadata") } }
        }
    }
    // The phone app's door: the core serves a read-only copy of the library, the
    // subscriptions and the downloads on this computer's Tailscale addresses while it is on
    Component {
        id: sharePanel
        Panel {
            objectName: "settings-share"
            title: "Phone app"; icon: "radio"
            helper: "Your phone's AniBeam app reads your library, subscriptions and downloads from here. Only devices on your tailnet can reach it."
            SettingRow {
                label: "Share over Tailscale"
                line: tab.host.message
                status: tab.host.urls.join("\n")
                hairline: false
                Switch { objectName: "settings-share-switch"; checked: !!Door.preferences.share_over_tailscale; onToggled: function(on) { tab.setSharing(on) } }
            }
        }
    }
    // Every series with files, most episodes first: a ledger of what the folders hold
    Component {
        id: diskPanel
        Panel {
            objectName: "settings-disk"
            title: "On disk"; icon: "hard-drive"; grows: true
            helper: "Episodes on disk per series, most first, against the count AniList gives."
            stretch: FillRows {
                anchors.fill: parent
                model: tab.disk
                rowHeight: theme.space(9)
                delegate: Item {
                    id: diskRow
                    required property var modelData
                    width: parent ? parent.width : 0
                    height: theme.space(9)
                    readonly property var row: modelData
                    Label {
                        anchors.left: parent.left; anchors.right: count.left; anchors.rightMargin: theme.space(4)
                        anchors.verticalCenter: parent.verticalCenter
                        text: diskRow.row.series.title
                        role: "ui"
                        color: rowMouse.containsMouse ? theme.accent : theme.text
                        elide: Text.ElideRight
                    }
                    Label {
                        id: count
                        anchors.right: parent.right; anchors.verticalCenter: parent.verticalCenter
                        text: (diskRow.row.have || 0) + (diskRow.row.expected ? " / " + diskRow.row.expected : "")
                              + (diskRow.row.extra_on_disk ? "  +" + diskRow.row.extra_on_disk + " extra" : "")
                        role: "ledger"; color: theme.textDim
                    }
                    Rectangle { anchors.bottom: parent.bottom; width: parent.width; height: 1; color: theme.line; opacity: 0.5 }
                    MouseArea {
                        id: rowMouse
                        anchors.fill: parent
                        hoverEnabled: true
                        cursorShape: Qt.PointingHandCursor
                        onClicked: frame.go("series", { id: diskRow.row.series.id }, diskRow.row.series.title)
                    }
                }
            }
        }
    }
    FolderDialog {
        id: folderDialog
        title: "Add a folder"
        onAccepted: { var r = Door.addSource(decodeURIComponent(String(selectedFolder).replace("file://", ""))); frame.toast(r.error ? r.error.message : "Folder added, scanning") }
    }
}
