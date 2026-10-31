// Library: the folders AniBeam watches (each with its counts and its own Open, Rescan and
// Remove, then Add a folder and the last scan with Scan all), Matching (the providers in
// the order matching tries them, and the way to the Metadata page where each match is
// reviewed), the session's Show hidden shows, and the shows hidden from Airing now, each
// with a way back. The figures and the per-series ledger the old tab carried are the
// Metadata page's job and live there.
import QtQuick
import QtQuick.Dialogs
import com.marcusrosado.AniBeam

SettingsTab {
    id: tab
    title: "Library"
    lead: "The folders AniBeam watches, and how it works out what is in them."
    property var sources: []
    property var lastScan: null              // the newest ScanFinished event
    function sourcePath(id) { for (var i = 0; i < sources.length; i++) if (sources[i].id === id) return sources[i].path; return "" }
    function reload() {
        var r = Door.listSources()
        if (!r.error) sources = r.reply.sources
        // Source carries no last-scan time of its own, so this reads it off the event
        // history: recentEvents serialises each body externally tagged, { ScanFinished: {...} }
        var ev = Door.recentEvents(2000)
        var newest = null
        if (!ev.error) ev.reply.events.forEach(function(e) {
            if (e.body && e.body.ScanFinished !== undefined && (!newest || e.at > newest.at)) newest = { at: e.at, scan: e.body.ScanFinished }
        })
        lastScan = newest
    }
    readonly property string scanLine: {
        if (!lastScan) return "Not scanned yet"
        var s = lastScan.scan
        var what = s.added + s.changed + s.removed === 0 ? "nothing changed"
                 : [s.added ? s.added + " added" : "", s.changed ? s.changed + " changed" : "", s.removed ? s.removed + " removed" : ""].filter(Boolean).join(", ")
        var where = s.source !== null && s.source !== undefined && sourcePath(s.source) !== "" ? sourcePath(s.source).split(/[\\/]/).pop() + ": " : ""
        return where + what.charAt(0).toUpperCase() + what.slice(1)
    }
    // Airing now's not interested marks (#202), newest first, each with a way back
    property var notInterested: []
    function reloadNotInterested() { var r = Door.listNotInterested(); if (!r.error) notInterested = r.reply.shows }
    function showAgain(id) {
        var r = Door.setNotInterested(id, false)
        if (r.error) { frame.toast(r.error.message); return }
        reloadNotInterested()
    }
    Component.onCompleted: { reload(); reloadNotInterested() }
    // The match providers, first tried first; the core keeps the order and says what it is
    readonly property var providerOrder: Door.settings && Door.settings.provider_order && Door.settings.provider_order.length ? Door.settings.provider_order : ["Anilist", "Tvmaze"]
    readonly property var providerAbout: ({ Anilist: "Anime: series, films and how they relate. What you watch is marked on your trackers.",
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
    }

    Panel {
        objectName: "settings-folders"
        title: "Folders"
        helper: "Each folder inside one of these is a series. A file at the top of a Movies folder is a film."
        Repeater {
            model: tab.sources
            SourceRow {
                required property var modelData
                source: modelData
                onOpen: frame.openLink(Fmt.fileUrl(modelData.path))
                onRescan: { var r = Door.scan(modelData.id); frame.toast(r.error ? r.error.message : "Rescan started") }
                onRemove: confirming = true
                onRemoveAccepted: { var r = Door.removeSource(modelData.id); if (r.error) frame.toast(r.error.message) }
            }
        }
        SettingRow {
            visible: tab.sources.length === 0
            label: "No folders yet"
            helper: "Point AniBeam at the folder your anime lives in. It finds the series and films by itself."
        }
        SettingRow {
            label: tab.sources.length ? (tab.lastScan ? "Scanned " + Fmt.relative(tab.lastScan.at, Date.now() / 1000) : "Not scanned yet") : "Add a folder"
            line: tab.sources.length && tab.lastScan ? tab.scanLine : ""
            Row {
                spacing: theme.space(2)
                Button { visible: tab.sources.length > 0; text: "Scan all"; icon: "refresh-cw"; flat: true; onClicked: { var r = Door.scan(-1); frame.toast(r.error ? r.error.message : "Scan started") } }
                Button { objectName: "settings-add-folder"; text: "Add folder"; icon: "folder-plus"; primary: tab.sources.length === 0; onClicked: folderDialog.open() }
            }
        }
    }
    Panel {
        objectName: "settings-providers"
        title: "Matching"
        helper: "A new folder is looked up in this order and keeps the first good match. A match you set by hand is never replaced."
        Repeater {
            model: tab.providerOrder
            SettingRow {
                required property var modelData
                required property int index
                objectName: "provider-row-" + String(modelData).toLowerCase()
                label: frame.providerName(modelData)
                line: tab.ordinal(index)
                helper: tab.providerAbout[modelData] || ""
                Button {
                    objectName: "provider-" + (index === 0 ? "down-" : "up-") + String(modelData).toLowerCase()
                    text: index === 0 ? "Try later" : "Try first"
                    icon: index === 0 ? "arrow-down" : "arrow-up"
                    flat: true; small: true
                    onClicked: tab.moveProvider(index, index === 0 ? 1 : index - 1)
                }
            }
        }
        SettingRow {
            label: "Check the matches"
            helper: "The Metadata page lists every series with the record it matched, and fixes a wrong one."
            Button { text: "Open Metadata"; icon: "database"; flat: true; onClicked: frame.go("metadata") }
        }
    }
    Panel {
        objectName: "settings-not-interested"
        title: "Hidden shows"
        SettingRow {
            label: "Show hidden shows"
            helper: "Brings the series you hid back onto every page until AniBeam closes."
            Switch { objectName: "settings-reveal-hidden"; checked: Door.revealHidden; onToggled: function(on) { Door.revealHidden = on } }
        }
        SettingRow {
            label: "Hidden from Airing now"
            helper: tab.notInterested.length ? "Shows you marked not interested. Each stays off Airing now until you bring it back."
                                             : "Nothing yet. Shows you mark not interested on Airing now collect here, so any of them can come back."
        }
        Repeater {
            model: tab.notInterested
            SettingRow {
                id: niRow
                required property var modelData
                objectName: "not-interested-" + modelData.anilist_id
                label: modelData.title
                Button {
                    objectName: "not-interested-again-" + niRow.modelData.anilist_id
                    text: "Show again"; icon: "eye"; flat: true; small: true
                    onClicked: tab.showAgain(niRow.modelData.anilist_id)
                }
            }
        }
    }
    FolderDialog {
        id: folderDialog
        title: "Add a folder"
        onAccepted: { var r = Door.addSource(Fmt.localPath(String(selectedFolder))); frame.toast(r.error ? r.error.message : "Folder added, scanning") }
    }
}
