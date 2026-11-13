// Phone app: the core serves the library, the subscriptions, the downloads and the episodes
// themselves on this computer's Tailscale addresses while sharing is on. The page says what
// the phone gets before the switch, then whether the headless host service keeps it
// reachable with this window closed, then the addresses the phone app should be given, one
// row each, with Copy. Below them, the people whose devices it serves, the computer's own
// person first, each keeping their own progress here (#633), then every device on the
// tailnet with a switch and whose it is: all are trusted until the person switches one
// off, and one switched off gets nothing.
import QtQuick
import com.marcusrosado.AniBeam

SettingsTab {
    id: tab
    title: "Phone app"
    lead: "AniBeam on your phone can play this library, see what is airing and follow your downloads, wherever you are."
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
    readonly property string enableCommand: "systemctl --user enable --now anibeam-host"
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
        var names = { linux: "Linux", windows: "Windows", android: "Android", ios: "iOS", macos: "macOS", freebsd: "FreeBSD", openbsd: "OpenBSD" }
        var os = names[d.os.toLowerCase()] || d.os
        var parts = []
        if (os !== "") parts.push(os)
        parts.push(d.online ? "Online" : "Offline")
        if (d.person) parts.push(d.person)
        if (d.nobody) parts.push("Gets nothing: Tailscale names no person for it")
        else if (!d.trusted) parts.push("Gets nothing from this computer")
        return parts.join(" · ")
    }
    function personLine(p) {
        var parts = []
        if (p.login && p.login !== p.name) parts.push(p.login)
        if (p.owner) parts.push(p.devices === 0 ? "This computer" : p.devices === 1 ? "This computer and 1 device" : "This computer and " + p.devices + " devices")
        else parts.push(p.devices === 1 ? "1 device" : p.devices + " devices")
        return parts.join(" · ")
    }
    Component.onCompleted: { reloadHost(); reloadDevices(); service = Shell.hostService() }
    Connections { target: Door; function onTailnetDevicesChanged(devices, people) { tab.devices = devices; tab.people = people } }
    Connections { target: Door; function onHostChanged(status) { tab.host = status } }
    readonly property bool on: !!Door.preferences.share_over_tailscale

    Panel {
        objectName: "settings-share"
        title: "Sharing"
        helper: "Only your own devices on Tailscale can reach it. Episodes watched on the phone keep their place and progress here."
        SettingRow {
            label: "Share over Tailscale"
            line: tab.host.message
            // The service shares whatever the switch says, so with one running it reads on
            Switch {
                objectName: "settings-share-switch"
                checked: tab.on || tab.service !== ""
                enabled: tab.service === ""
                opacity: enabled ? 1 : theme.disabledOpacity
                onToggled: function(on) { tab.setSharing(on) }
            }
        }
        SettingRow {
            label: "Stream episodes"
            helper: Door.preferences.stream_over_tailscale !== false
                ? "Your devices can play and cast the episodes on this computer."
                : "Your devices see the library and what is airing, but cannot play from here."
            Switch {
                objectName: "settings-stream-switch"
                checked: Door.preferences.stream_over_tailscale !== false
                onToggled: function(on) { tab.setStreaming(on) }
            }
        }
        SettingRow {
            objectName: "settings-host-service"
            label: "Host service"
            line: tab.service === "" ? "Not running" : "Running"
            helper: tab.service === "" ? "Run the command once and your devices reach AniBeam from login on, with no window open."
                : "Your devices reach AniBeam with no window open. While this window is open, it shares in the service's place."
            Button {
                visible: tab.service === ""
                text: "Copy command"; icon: "copy"; flat: true; small: true
                onClicked: frame.copyText(tab.enableCommand, "Command copied")
            }
        }
    }
    Panel {
        objectName: "settings-tailnet-people"
        visible: tab.people.length > 0
        title: "People"
        Repeater {
            model: tab.people
            SettingRow {
                required property var modelData
                label: modelData.name
                line: tab.personLine(modelData)
                Chip {
                    visible: modelData.owner
                    text: "This computer's profile"
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
        title: "Devices on your tailnet"
        helper: "Every device on your tailnet can reach this computer while sharing is on. Switch one off and it gets nothing from here."
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
    Panel {
        visible: tab.host.enabled && tab.host.urls.length > 0
        title: "Addresses"
        helper: "Type one of these into the phone app the first time it asks."
        Repeater {
            model: tab.host.urls
            SettingRow {
                required property var modelData
                label: modelData
                Button { text: "Copy"; icon: "copy"; flat: true; small: true; onClicked: frame.copyText(modelData, "Address copied") }
            }
        }
    }
}
