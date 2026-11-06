// Accounts: the trackers. AniList and MyAnimeList each as a row that connects, waits on
// the browser or disconnects (the flow itself is the frame's), which one the cards read
// progress from, Sync now, and On hold: a Watching series left untouched moves there on
// its own, with the days it waits. On hold and the main tracker only mean something with
// a tracker connected, so with none they stay out and the page says what connecting buys.
import QtQuick
import com.marcusrosado.AniBeam

SettingsTab {
    id: tab
    title: "Accounts"
    lead: "Connect AniList or MyAnimeList and every episode you finish is marked there for you. Counts only ever go up."
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
    function sync() { var r = Door.refreshProgress(""); r.error ? frame.fail(r.error, tab.sync) : frame.toast("Syncing your lists") }
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
        function onProgressRefreshed() { frame.toast("Lists synced"); tab.readWaiting() }
        function onListsMoved() { tab.readWaiting() }
        function onJobFailed(job, kind, error) { if (kind === "RefreshProgress") frame.fail(error, tab.sync) }
    }

    Panel {
        title: "Your anime lists"
        helper: tab.tracked ? "" : "Without one, AniBeam still remembers what you watched on this computer. A list on AniList or MyAnimeList carries it to your phone and your profile."
        TrackerRow { tracker: "Anilist"; account: Door.trackers.anilist || ({})
            onDisconnect: tab.disconnect("Anilist") }
        TrackerRow { tracker: "Mal"; account: Door.trackers.mal || ({})
            onDisconnect: tab.disconnect("Mal") }
    }
    Panel {
        visible: tab.tracked
        title: "Progress"
        SettingRow { visible: tab.anilist && tab.mal; label: "Main list"; helper: "The cards show progress from this one. The other still gets every update."
            Seg { options: ["AniList", "MyAnimeList"]; index: Door.trackers.main === "Mal" ? 1 : 0; onPicked: function(i) { tab.setMain(i === 1 ? "Mal" : "Anilist") } } }
        SettingRow {
            id: moveRow
            readonly property string to: tab.waiting.Anilist > 0 ? "Anilist" : "Mal"
            readonly property int count: tab.waiting[to]
            visible: count > 0
            label: "Lists on this computer"
            helper: (count === 1 ? "One show has" : count + " shows have") + " a status or a score only here. Move them to " + frame.trackerName(to) + " to see them on your phone and your profile."
            Button { objectName: "settings-move-lists"; text: "Review and move"; icon: "upload"; flat: true
                onClicked: if (!frame.openMoveLists(moveRow.to)) frame.toast("Nothing left to move") } }
        SettingRow { label: "Sync now"; helper: "Reads every connected list again. AniBeam also does this when it starts."
            Button { objectName: "settings-sync-trackers"; text: "Sync"; icon: "refresh-cw"; flat: true
                onClicked: tab.sync() } }
    }
    Panel {
        visible: tab.tracked
        title: "On hold"
        SettingRow { label: "Move to On hold"
            helper: "A Watching series you have not touched for a while moves to On hold on your lists. Playing it moves it back. A show waiting for its next episode stays put."
            Switch { objectName: "settings-on-hold"; checked: Door.preferences.on_hold_auto !== false; onToggled: function(on) { tab.setOnHold(on) } } }
        SettingRow { visible: Door.preferences.on_hold_auto !== false; label: "After"
            line: "Days without playing an episode"
            SliderRow { objectName: "settings-on-hold-days"; unit: "days"; from: 7; to: 365; stepSize: 1; value: Door.preferences.on_hold_days || 60
                onMoved: function(v) { tab.pendingDays = Math.round(v); daysDebounce.restart() } } }
    }
}
