// Accounts: the trackers. AniList and MyAnimeList each as a row that connects, waits on
// the browser or disconnects (the flow itself is the frame's), which one the cards read
// progress from, Sync now, and On hold: a Watching series left untouched moves there on
// its own, with the days it waits. On hold and the main tracker only mean something with
// a tracker connected, so with none they stay out and the page says what connecting buys.
import QtQuick
import com.marcusrosado.AniBeam

SettingsTab {
    id: tab
    title: words.t("settings-accounts-title")
    lead: words.t("settings-accounts-lead")
    readonly property bool anilist: !!(Door.trackers.anilist && Door.trackers.anilist.connected)
    readonly property bool mal: !!(Door.trackers.mal && Door.trackers.mal.connected)
    readonly property bool tracked: anilist || mal
    function setOnHold(on) {
        var p = JSON.parse(JSON.stringify(Door.preferences))
        p.on_hold_auto = on
        var r = Door.setPreferences(p)
        if (r.error) frame.fail(r.error, function() { tab.setOnHold(on) })
    }
    // The days are written once the slider rests, so a drag is one write; a section left
    // before then writes them on its way out
    property int pendingDays: 60
    function saveDays() {
        var p = JSON.parse(JSON.stringify(Door.preferences))
        p.on_hold_days = tab.pendingDays
        var r = Door.setPreferences(p)
        if (r.error) frame.fail(r.error, tab.saveDays)
    }
    // Each call that can fail offers Try again on the toast, which runs it again in full
    function disconnect(t) { var r = Door.disconnectTracker(t); if (r.error) frame.fail(r.error, function() { tab.disconnect(t) }) }
    function setMain(t) { var r = Door.setMainTracker(t); if (r.error) frame.fail(r.error, function() { tab.setMain(t) }) }
    function sync() { var r = Door.refreshProgress(""); r.error ? frame.fail(r.error, tab.sync) : frame.toast(words.t("settings-accounts-syncing")) }
    Timer { id: daysDebounce; interval: 400; onTriggered: tab.saveDays() }
    Component.onDestruction: if (daysDebounce.running) tab.saveDays()
    // The list kept on this computer that a connected tracker could take (#580): the quiet
    // pointer a Not now on the move sheet leaves behind
    property var waiting: ({ Anilist: 0, Mal: 0 })
    function readWaiting() {
        var out = { Anilist: 0, Mal: 0 }
        var on = { Anilist: tab.anilist, Mal: tab.mal }
        for (var t in out) {
            if (!on[t]) continue
            var r = Door.call("PreviewListMove", { tracker: t })
            if (!r.error && r.reply && r.reply.rows) out[t] = r.reply.rows.length
        }
        tab.waiting = out
    }
    Component.onCompleted: readWaiting()
    onAnilistChanged: readWaiting()
    onMalChanged: readWaiting()
    Connections {
        target: Door
        function onProgressRefreshed() { frame.toast(words.t("settings-accounts-synced")); tab.readWaiting() }
        function onListsMoved() { tab.readWaiting() }
        function onJobFailed(job, kind, error) { if (kind === "RefreshProgress") frame.fail(error, tab.sync) }
    }

    Panel {
        title: words.t("settings-accounts-lists")
        helper: tab.tracked ? "" : words.t("settings-accounts-lists-helper")
        TrackerRow { tracker: "Anilist"; account: Door.trackers.anilist || ({})
            onDisconnect: tab.disconnect("Anilist") }
        TrackerRow { tracker: "Mal"; account: Door.trackers.mal || ({})
            onDisconnect: tab.disconnect("Mal") }
    }
    Panel {
        visible: tab.tracked
        title: words.t("settings-accounts-progress")
        SettingRow { visible: tab.anilist && tab.mal; label: words.t("settings-accounts-main-list"); helper: words.t("settings-accounts-main-list-helper")
            Seg { options: ["AniList", "MyAnimeList"]; index: Door.trackers.main === "Mal" ? 1 : 0; onPicked: function(i) { tab.setMain(i === 1 ? "Mal" : "Anilist") } } } // i18n: none
        SettingRow {
            id: moveRow
            readonly property string to: tab.waiting.Anilist > 0 ? "Anilist" : "Mal"
            readonly property int count: tab.waiting[to]
            visible: count > 0
            label: words.t("settings-accounts-local-lists")
            helper: words.t("settings-accounts-local-lists-helper", { count: count, tracker: frame.trackerName(to) })
            Button { objectName: "settings-move-lists"; msg: "settings-accounts-review-move"; icon: "upload"; flat: true
                onClicked: if (!frame.openMoveLists(moveRow.to)) frame.toast(words.t("settings-accounts-nothing-to-move")) } }
        SettingRow { label: words.t("settings-accounts-sync-now"); helper: words.t("settings-accounts-sync-now-helper")
            Button { objectName: "settings-sync-trackers"; msg: "settings-accounts-sync"; icon: "refresh-cw"; flat: true
                onClicked: tab.sync() } }
    }
    Panel {
        visible: tab.tracked
        title: words.t("status-on-hold")
        SettingRow { label: words.t("settings-accounts-move-on-hold")
            helper: words.t("settings-accounts-move-on-hold-helper")
            Switch { objectName: "settings-on-hold"; checked: Door.preferences.on_hold_auto !== false; onToggled: function(on) { tab.setOnHold(on) } } }
        SettingRow { visible: Door.preferences.on_hold_auto !== false; label: words.t("settings-accounts-after")
            line: words.t("settings-accounts-after-line")
            SliderRow { objectName: "settings-on-hold-days"; unit: words.t("settings-accounts-days-unit"); from: 7; to: 365; stepSize: 1; value: Door.preferences.on_hold_days || 60
                onMoved: function(v) { tab.pendingDays = Math.round(v); daysDebounce.restart() } } }
    }
}
