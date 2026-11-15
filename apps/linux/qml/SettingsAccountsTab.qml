// Accounts: the trackers, then the phone app. AniList and MyAnimeList each as a row that
// connects, waits on the browser or disconnects (the flow itself is the frame's), which one
// the cards read progress from, Sync now, and On hold: a Watching series left untouched
// moves there on its own, with the days it waits. On hold and the main tracker only mean
// something with a tracker connected, so with none they stay out.
// The phone app follows: the core serves the library, the subscriptions, the downloads and
// the episodes themselves on this computer's Tailscale addresses while sharing is on. The
// share switch, the stream switch and whether the headless host service keeps it reachable
// with this window closed, then the people whose devices it serves, the computer's own
// person first, each keeping their own progress here (#633), then every device on the
// tailnet with a switch and whose it is: all are trusted until the person switches one
// off, and one switched off gets nothing. The addresses the phone app should be given sit
// under the host service, one row each, with Copy.
import QtQuick
import com.marcusrosado.Anibeam

SettingsTab {
    id: tab
    title: words.t("settings-accounts-title")
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
    // The phone app's reads (the host, the tailnet's devices, the host service) wait a turn,
    // so the lists at the top paint before they run
    Component.onCompleted: { readWaiting(); Qt.callLater(tab.readPhone) }
    function readPhone() { reloadHost(); reloadDevices(); service = Shell.hostService() }
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
        TrackerRow { tracker: "Anilist"; account: Door.trackers.anilist || ({})
            onDisconnect: tab.disconnect("Anilist") }
        TrackerRow { tracker: "Mal"; account: Door.trackers.mal || ({})
            onDisconnect: tab.disconnect("Mal") }
    }
    Panel {
        visible: tab.tracked
        title: words.t("settings-accounts-progress")
        SettingRow { visible: tab.anilist && tab.mal; label: words.t("settings-accounts-main-list")
            Seg { options: ["AniList", "MyAnimeList"]; index: Door.trackers.main === "Mal" ? 1 : 0; onPicked: function(i) { tab.setMain(i === 1 ? "Mal" : "Anilist") } } } // i18n: none
        SettingRow {
            id: moveRow
            readonly property string to: tab.waiting.Anilist > 0 ? "Anilist" : "Mal"
            readonly property int count: tab.waiting[to]
            visible: count > 0
            label: words.t("settings-accounts-local-lists")
            status: words.t("settings-accounts-local-lists-helper", { count: count, tracker: frame.trackerName(to) })
            Button { objectName: "settings-move-lists"; msg: "settings-accounts-review-move"; icon: "upload"; flat: true
                onClicked: if (!frame.openMoveLists(moveRow.to)) frame.toast(words.t("settings-accounts-nothing-to-move")) } }
        SettingRow { label: words.t("settings-accounts-sync-now")
            Button { objectName: "settings-sync-trackers"; msg: "settings-accounts-sync"; icon: "refresh-cw"; flat: true
                onClicked: tab.sync() } }
    }
    Panel {
        visible: tab.tracked
        title: words.t("status-on-hold")
        SettingRow { label: words.t("settings-accounts-move-on-hold")
            Switch { objectName: "settings-on-hold"; checked: Door.preferences.on_hold_auto !== false; onToggled: function(on) { tab.setOnHold(on) } } }
        SettingRow { visible: Door.preferences.on_hold_auto !== false; label: words.t("settings-accounts-after")
            line: words.t("settings-accounts-after-line")
            SliderRow { objectName: "settings-on-hold-days"; unit: words.t("settings-accounts-days-unit"); from: 7; to: 365; stepSize: 1; value: Door.preferences.on_hold_days || 60
                onMoved: function(v) { tab.pendingDays = Math.round(v); daysDebounce.restart() } } }
    }

    // ---- The phone app
    property var host: ({ enabled: false, urls: [], message: "" })
    // Read once on open; a failed read says so with Try again, not as sharing switched off
    function reloadHost() { var r = Door.getHost(); if (r.error) frame.fail(r.error, tab.reloadHost); else host = r.reply.status }
    function setSharing(on) {
        var p = JSON.parse(JSON.stringify(Door.preferences))
        p.share_over_tailscale = on
        var r = Door.setPreferences(p)
        if (r.error) frame.fail(r.error, function() { tab.setSharing(on) })
    }
    function setStreaming(on) {
        var p = JSON.parse(JSON.stringify(Door.preferences))
        p.stream_over_tailscale = on
        var r = Door.setPreferences(p)
        if (r.error) frame.toast(r.error.message)
    }
    readonly property string enableCommand: "systemctl --user enable --now anibeam-host" // i18n: none
    // "serving", "waiting" while this window has the library, or empty with no service.
    property string service: ""
    property var devices: []
    property var people: []
    function reloadDevices() {
        var r = Door.call("ListTailnetDevices", {})
        if (r.error) return
        devices = r.reply.devices
        people = r.reply.people || []
    }
    function setTrusted(device, on) {
        var r = Door.call("SetTailnetDeviceTrust", { device: device, trusted: on })
        if (r.error) frame.fail(r.error, function() { tab.setTrusted(device, on) })
        reloadDevices()
    }
    function deviceLine(d) {
        // Tailscale's own words for the system, as people write them
        var names = { linux: "Linux", windows: "Windows", android: "Android", ios: "iOS", macos: "macOS", freebsd: "FreeBSD", openbsd: "OpenBSD" } // i18n: none
        var os = names[d.os.toLowerCase()] || d.os
        var parts = []
        if (os !== "") parts.push(os)
        parts.push(d.online ? words.t("settings-phone-online") : words.t("settings-phone-offline"))
        if (d.person) parts.push(d.person)
        if (d.nobody) parts.push(words.t("settings-phone-nobody"))
        else if (!d.trusted) parts.push(words.t("settings-phone-untrusted"))
        return parts.join(" · ")
    }
    function personLine(p) {
        var parts = []
        if (p.login && p.login !== p.name) parts.push(p.login)
        parts.push(words.t(p.owner ? "settings-phone-person-own-devices" : "settings-phone-person-devices", { count: p.devices }))
        return parts.join(" · ")
    }
    Connections { target: Door; function onTailnetDevicesChanged(devices, people) { tab.devices = devices; tab.people = people } }
    Connections { target: Door; function onHostChanged(status) { tab.host = status } }
    readonly property bool sharing: !!Door.preferences.share_over_tailscale

    Panel {
        objectName: "settings-share"
        title: words.t("settings-phone-title")
        SettingRow {
            label: words.t("settings-phone-share")
            line: tab.host.message
            // The service shares whatever the switch says, so with one running it reads on
            Switch {
                objectName: "settings-share-switch"
                checked: tab.sharing || tab.service !== ""
                enabled: tab.service === ""
                opacity: enabled ? 1 : theme.disabledOpacity
                onToggled: function(on) { tab.setSharing(on) }
            }
        }
        SettingRow {
            label: words.t("settings-phone-stream")
            Switch {
                objectName: "settings-stream-switch"
                checked: Door.preferences.stream_over_tailscale !== false
                onToggled: function(on) { tab.setStreaming(on) }
            }
        }
        SettingRow {
            objectName: "settings-host-service"
            label: words.t("settings-phone-host-service")
            line: tab.service === "" ? words.t("settings-phone-not-running") : words.t("settings-phone-running")
            Button {
                visible: tab.service === ""
                msg: "settings-phone-copy-command"; icon: "copy"; flat: true; small: true
                onClicked: frame.copyText(tab.enableCommand, words.t("settings-phone-command-copied"))
            }
        }
        // The addresses the phone app should be given, while sharing is on
        Repeater {
            model: tab.host.enabled ? tab.host.urls : []
            SettingRow {
                required property var modelData
                label: modelData
                Button { msg: "settings-phone-copy"; icon: "copy"; flat: true; small: true; onClicked: frame.copyText(modelData, words.t("settings-phone-address-copied")) }
            }
        }
    }
    Panel {
        objectName: "settings-tailnet-people"
        visible: tab.people.length > 0
        title: words.t("settings-phone-people")
        Repeater {
            model: tab.people
            SettingRow {
                required property var modelData
                label: modelData.name
                line: tab.personLine(modelData)
                Chip {
                    visible: modelData.owner
                    text: words.t("settings-phone-own-profile")
                    mono: false
                    small: true
                    color: theme.raised
                }
            }
        }
    }
    Panel {
        objectName: "settings-tailnet-devices"
        visible: tab.devices.length > 0
        title: words.t("settings-phone-devices")
        Repeater {
            model: tab.devices
            SettingRow {
                required property var modelData
                label: modelData.name
                line: tab.deviceLine(modelData)
                Switch {
                    checked: modelData.trusted
                    onToggled: function(on) { tab.setTrusted(modelData.id, on) }
                }
            }
        }
    }
}
