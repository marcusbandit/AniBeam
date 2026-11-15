// Library: the folders Anibeam watches (each with its counts and its own Open, Rescan and
// Remove, then Add a folder and the last scan with Scan all), Matching (the providers in
// the order matching tries them, and the way to the Metadata page where each match is
// reviewed), the session's Show hidden shows, and the shows hidden from Airing now, each
// with a way back, then Calendar (the iCal file of airing times the core keeps while
// its switch is on, with its path to copy) and Notifications (a desktop notification when a
// followed show's episode airs, #212). The figures and the per-series ledger the old tab
// carried are the Metadata page's job and live there.
import QtQuick
import QtQuick.Dialogs
import com.marcusrosado.AniBeam

SettingsTab {
    id: tab
    title: words.t("settings-library-title")
    lead: words.t("settings-library-lead")
    property var sources: []
    property var lastScan: null              // the newest ScanFinished event
    function sourcePath(id) { for (var i = 0; i < sources.length; i++) if (sources[i].id === id) return sources[i].path; return "" }
    // A folder list that did not load says so, with Try again, rather than reading as none
    property bool loadFailed: false
    function reload() {
        var r = Door.listSources()
        loadFailed = !!r.error
        if (r.error) frame.fail(r.error, tab.reload)
        else sources = r.reply.sources
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
        if (!lastScan) return words.t("settings-library-not-scanned")
        var s = lastScan.scan
        var what = s.added + s.changed + s.removed === 0 ? words.t("settings-library-scan-nothing-changed")
                 : [s.added ? words.t("settings-library-scan-added", { count: s.added }) : "", s.changed ? words.t("settings-library-scan-changed", { count: s.changed }) : "", s.removed ? words.t("settings-library-scan-removed", { count: s.removed }) : ""].filter(Boolean).join(", ")
        return s.source !== null && s.source !== undefined && sourcePath(s.source) !== "" ? words.t("settings-library-scan-in-folder", { folder: Fmt.baseName(sourcePath(s.source)), what: what }) : what
    }
    // Airing now's not interested marks (#202), newest first, each with a way back
    property var notInterested: []
    function reloadNotInterested() { var r = Door.listNotInterested(); if (!r.error) notInterested = r.reply.shows }
    function showAgain(id) {
        var r = Door.setNotInterested(id, false)
        if (r.error) { frame.fail(r.error, function() { tab.showAgain(id) }); return }
        reloadNotInterested()
    }
    Component.onCompleted: { reload(); reloadNotInterested() }
    // The match providers, first tried first; the core keeps the order and says what it is
    readonly property var providerOrder: Door.settings && Door.settings.provider_order && Door.settings.provider_order.length ? Door.settings.provider_order : ["Anilist", "Tvmaze"]
    readonly property var providerAbout: ({ Anilist: "settings-library-anilist-about", Tvmaze: "settings-library-tvmaze-about" })
    function ordinal(i) {
        var named = (["settings-library-tried-first", "settings-library-tried-second", "settings-library-tried-third", "settings-library-tried-fourth"])[i]
        return named ? words.t(named) : words.t("settings-library-tried-nth", { number: i + 1 })
    }
    function moveProvider(from, to) {
        if (to < 0 || to >= providerOrder.length) return
        var o = providerOrder.slice()
        var p = o.splice(from, 1)[0]
        o.splice(to, 0, p)
        var r = Door.setProviderOrder(o)
        if (r.error) frame.fail(r.error, function() { tab.moveProvider(from, to) })
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
        title: words.t("settings-library-folders")
        helper: words.t("settings-library-folders-helper")
        Repeater {
            model: tab.sources
            SourceRow {
                required property var modelData
                source: modelData
                onOpen: frame.openLink(Fmt.fileUrl(modelData.path))
                onRescan: { var id = modelData.id, again = function() { var r = Door.scan(id); r.error ? frame.fail(r.error, again) : frame.toast(words.t("settings-library-rescan-started")) }; again() }
                onRemove: confirming = true
                onRemoveAccepted: { var r = Door.removeSource(modelData.id); if (r.error) frame.fail(r.error) }
            }
        }
        SettingRow {
            objectName: "settings-folders-failed"
            visible: tab.loadFailed
            label: words.t("settings-library-load-failed")
            helper: words.t("settings-library-load-failed-helper")
            Button { msg: "action-try-again"; icon: "refresh-cw"; onClicked: tab.reload() }
        }
        SettingRow {
            visible: tab.sources.length === 0 && !tab.loadFailed
            label: words.t("settings-library-no-folders")
            helper: words.t("settings-library-no-folders-helper")
        }
        SettingRow {
            label: tab.sources.length ? (tab.lastScan ? words.t("settings-library-scanned", { when: words.relative(tab.lastScan.at, Date.now() / 1000) }) : words.t("settings-library-not-scanned")) : words.t("settings-library-add-a-folder")
            line: tab.sources.length && tab.lastScan ? tab.scanLine : ""
            Row {
                spacing: theme.space(2)
                Button { visible: tab.sources.length > 0; msg: "settings-library-scan-all"; icon: "refresh-cw"; flat: true; onClicked: { var r = Door.scan(-1); r.error ? frame.fail(r.error, clicked) : frame.toast(words.t("settings-library-scan-started")) } }
                Button { objectName: "settings-add-folder"; msg: "settings-library-add-folder"; icon: "folder-plus"; primary: tab.sources.length === 0; onClicked: folderDialog.open() }
            }
        }
    }
    Panel {
        objectName: "settings-providers"
        title: words.t("settings-library-matching")
        helper: words.t("settings-library-matching-helper")
        Repeater {
            model: tab.providerOrder
            SettingRow {
                required property var modelData
                required property int index
                objectName: "provider-row-" + String(modelData).toLowerCase()
                label: frame.providerName(modelData)
                line: tab.ordinal(index)
                helper: tab.providerAbout[modelData] ? words.t(tab.providerAbout[modelData]) : ""
                Button {
                    objectName: "provider-" + (index === 0 ? "down-" : "up-") + String(modelData).toLowerCase()
                    text: index === 0 ? words.t("settings-library-try-later") : words.t("settings-library-try-first")
                    icon: index === 0 ? "arrow-down" : "arrow-up"
                    flat: true; small: true
                    onClicked: tab.moveProvider(index, index === 0 ? 1 : index - 1)
                }
            }
        }
        SettingRow {
            label: words.t("settings-library-check-matches")
            helper: words.t("settings-library-check-matches-helper")
            Button { msg: "settings-library-open-show-info"; icon: "database"; flat: true; onClicked: frame.go("metadata") }
        }
    }
    Panel {
        objectName: "settings-not-interested"
        title: words.t("settings-library-hidden-shows")
        SettingRow {
            label: words.t("settings-library-show-hidden")
            helper: words.t("settings-library-show-hidden-helper")
            Switch { objectName: "settings-reveal-hidden"; checked: Door.revealHidden; onToggled: function(on) { Door.revealHidden = on } }
        }
        SettingRow {
            label: words.t("settings-library-hidden-from-airing")
            helper: tab.notInterested.length ? words.t("settings-library-hidden-from-airing-helper")
                                             : words.t("settings-library-hidden-from-airing-empty")
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
                    msg: "settings-library-show-again"; icon: "eye"; flat: true; small: true
                    onClicked: tab.showAgain(niRow.modelData.anilist_id)
                }
            }
        }
    }
    // The calendar feed (#217): the core keeps the file current while the switch is on
    readonly property string calendarPath: Door.about && Door.about.data_dir ? Door.about.data_dir + "/anibeam.ics" : ""
    function setCalendar(on) {
        var p = JSON.parse(JSON.stringify(Door.preferences))
        p.calendar_feed = on
        var r = Door.setPreferences(p)
        if (r.error) frame.fail(r.error, function() { tab.setCalendar(on) })
    }
    Panel {
        objectName: "settings-calendar"
        title: words.t("settings-library-calendar")
        SettingRow {
            label: words.t("settings-library-calendar-feed")
            helper: words.t("settings-library-calendar-feed-helper")
            Switch { objectName: "settings-calendar-switch"; checked: !!Door.preferences.calendar_feed; onToggled: function(on) { tab.setCalendar(on) } }
        }
        SettingRow {
            objectName: "settings-calendar-file"
            visible: !!Door.preferences.calendar_feed && tab.calendarPath !== ""
            label: words.t("settings-library-calendar-file")
            helper: words.t("settings-library-calendar-file-helper")
            status: tab.calendarPath
            Row { spacing: theme.space(2)
                Button { objectName: "settings-calendar-copy"; msg: "settings-library-copy"; icon: "copy"; flat: true; small: true; onClicked: frame.copyText(tab.calendarPath, words.t("settings-library-path-copied")) }
                Button { objectName: "settings-calendar-open"; msg: "settings-library-open-folder"; icon: "folder-open"; flat: true; small: true; onClicked: frame.openLink(Fmt.fileUrl(Door.about.data_dir)) } }
        }
    }
    // Episode out (#212): a desktop notification when a followed show's episode airs
    function setEpisodeNotifications(on) {
        var p = JSON.parse(JSON.stringify(Door.preferences))
        p.episode_notifications = on
        var r = Door.setPreferences(p)
        if (r.error) frame.fail(r.error, function() { tab.setEpisodeNotifications(on) })
    }
    Panel {
        objectName: "settings-notifications"
        title: words.t("settings-library-notifications")
        SettingRow {
            label: words.t("settings-library-new-episodes")
            helper: words.t("settings-library-new-episodes-helper")
            Switch { objectName: "settings-episode-notifications"; checked: Door.preferences.episode_notifications !== false; onToggled: function(on) { tab.setEpisodeNotifications(on) } }
        }
    }
    FolderDialog {
        id: folderDialog
        title: words.t("settings-library-add-a-folder")
        onAccepted: { var r = Door.addSource(Fmt.localPath(String(selectedFolder))); r.error ? frame.fail(r.error, folderDialog.open) : frame.toast(words.t("settings-library-folder-added")) }
    }
}
