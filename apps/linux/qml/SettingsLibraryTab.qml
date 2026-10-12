// The Library tab: sources with their counts and the Movies folders under them, Scan all,
// Add folder through the native picker, Rescan and Remove per source, Show hidden shows,
// the Subscriptions row; and the Trackers panel.
import QtQuick
import QtQuick.Dialogs
import com.marcusrosado.AniBeam

SettingsTab {
    id: tab
    property var sources: []
    property var stats: ({ series: 0, films: 0, episodes: 0, lastScan: "never" })
    function reload() {
        var r = Door.listSources()
        if (!r.error) sources = r.reply.sources
        var all = Door.listSeries("All", "", "Alpha", "Asc", Door.revealHidden)
        // The stats describe the library, not what the watched toggle
        // currently leaves visible, so the Watched tab rides along.
        var watched = Door.listSeries("Watched", "", "Alpha", "Asc", Door.revealHidden)
        if (!all.error) {
            var s = all.reply.series.concat(watched.error ? [] : watched.reply.series)
            var eps = 0; s.forEach(function(c) { eps += c.episodes_on_disk || 0 })
            // Source carries no last-scan time of its own (core/src/contract/records.rs), so
            // this reads it off the event history instead: recentEvents' Reply::Events serialises
            // each Event.body through plain serde, externally tagged as { ScanFinished: {...} },
            // not through the door.rs event_json helper that flattens a live-pushed envelope to
            // a top-level "kind"; there is no e.kind here.
            var ev = Door.recentEvents(2000)
            var lastAt = null
            if (!ev.error) ev.reply.events.forEach(function(e) { if (e.body && e.body.ScanFinished !== undefined && (lastAt === null || e.at > lastAt)) lastAt = e.at })
            var last = lastAt === null ? "never" : Fmt.relative(lastAt, Date.now() / 1000)
            stats = { series: s.filter(function(c) { return c.kind === "Show" }).length, films: s.filter(function(c) { return c.kind === "Movie" }).length, episodes: eps, lastScan: last }
        }
    }
    Component.onCompleted: reload()
    Timer { id: debounce; interval: 250; onTriggered: tab.reload() }
    Connections {
        target: Door
        function onSourceChanged(s) { debounce.restart() }
        function onSourceRemoved(s) { debounce.restart() }
        function onScanFinished(s, a, c, r) { debounce.restart() }
        function onSeriesChanged(c) { debounce.restart() }
    }

    SettingsPair {
        split: 3 / 5
        leftPanels: [ libraryPanel ]
        rightPanels: [ trackersPanel ]
    }
    Component {
        id: libraryPanel
        Panel {
            title: "Library"; icon: "folder-open"; grows: true
            Tiles { tiles: [{ value: String(tab.stats.series), caption: "Series" }, { value: String(tab.stats.films), caption: "Films" }, { value: tab.stats.episodes.toLocaleString(), caption: "Episodes" }, { value: tab.stats.lastScan, caption: "Last scan" }] }
            stretch: Corner {
                anchors.fill: parent
                implicitHeight: list.implicitHeight + theme.space(4)
                radius: theme.radiusMd; smoothing: theme.cornerSmoothing; color: theme.surfaceSunken; borderColor: theme.line; borderWidth: 1
                Column {
                    id: list
                    x: theme.space(2); y: theme.space(2); width: parent.width - theme.space(4)
                    Repeater {
                        model: tab.sources
                        SourceRow {
                            required property var modelData
                            source: modelData
                            onOpen: Qt.openUrlExternally("file://" + modelData.path)
                            onRescan: { var r = Door.scan(modelData.id); frame.toast(r.error ? r.error.message : "Rescan started") }
                            onRemove: confirming = true
                            onRemoveAccepted: { var r = Door.removeSource(modelData.id); if (r.error) frame.toast(r.error.message) }
                        }
                    }
                    Text { visible: tab.sources.length === 0; text: "No folders yet. Click Add folder to point AniBeam at your collection."; color: theme.textDim; font.family: theme.fontSans; font.pointSize: theme.typeNormal; padding: theme.space(3) }
                }
            }
            foot: [
                Row { spacing: theme.space(2)
                    Button { text: "Add folder"; icon: "folder-plus"; onClicked: folderDialog.open() }
                    Button { text: "Scan all"; icon: "refresh-cw"; enabled: tab.sources.length > 0; onClicked: { var r = Door.scan(-1); frame.toast(r.error ? r.error.message : "Scan started") } } },
                Note { text: "AniBeam scans these folders for video files. A folder is a series; a file at the top level of a Movies folder is a film." },
                SettingRow { label: "Show hidden shows"; helper: "Shows hidden series on every page until AniBeam closes."
                    Switch { checked: Door.revealHidden; onToggled: function(on) { Door.revealHidden = on } } },
                SettingRow { label: "Subscriptions"; helper: "The feeds anirss watches for you."
                    Button { text: "Open"; icon: "arrow-up-right"; flat: true; onClicked: frame.go("subscriptions") } }
            ]
        }
    }
    Component {
        id: trackersPanel
        Panel {
            title: "Trackers"; icon: "user-check"; grows: true
            helper: "Episodes are marked on every connected tracker when you reach the outro or mark them by hand. Counts only go up."
            // The connect flow itself is the frame's: the rows start it, show it and stop it
            // through frame.openConnect and the state beside it.
            TrackerRow { tracker: "Anilist"; account: Door.trackers.anilist || ({})
                onDisconnect: { var r = Door.disconnectTracker("Anilist"); if (r.error) frame.toast(r.error.message) } }
            TrackerRow { tracker: "Mal"; account: Door.trackers.mal || ({})
                onDisconnect: { var r = Door.disconnectTracker("Mal"); if (r.error) frame.toast(r.error.message) } }
            SettingRow { label: "Main tracker"; helper: "Whose count the cards show. The other tracker still receives every mark."
                Seg { options: ["AniList", "MyAnimeList"]; index: Door.trackers.main === "Mal" ? 1 : 0; onPicked: function(i) { var r = Door.setMainTracker(i === 1 ? "Mal" : "Anilist"); if (r.error) frame.toast(r.error.message) } } }
        }
    }
    FolderDialog {
        id: folderDialog
        title: "Add a folder"
        onAccepted: { var r = Door.addSource(decodeURIComponent(String(selectedFolder).replace("file://", ""))); frame.toast(r.error ? r.error.message : "Folder added, scanning") }
    }
}
