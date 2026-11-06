// Phone app: the core serves the library, the subscriptions, the downloads and the episodes
// themselves on this computer's Tailscale addresses while sharing is on. The page says what
// the phone gets before the switch, then whether the headless host service keeps it
// reachable with this window closed, then the addresses the phone app should be given, one
// row each, with Copy. Below them, every device on the tailnet with a switch: all are
// trusted until the person switches one off, and one switched off gets nothing.
import QtQuick
import com.marcusrosado.AniBeam

SettingsTab {
    id: tab
    title: "Phone app"
    lead: "AniBeam on your phone can play this library, see what is airing and follow your downloads, wherever you are."
    property var host: ({ enabled: false, urls: [], message: "" })
    function reloadHost() { var r = Door.getHost(); if (!r.error) host = r.reply.status }
    function setSharing(on) {
        var p = JSON.parse(JSON.stringify(Door.preferences))
        p.share_over_tailscale = on
        var r = Door.setPreferences(p)
        if (r.error) frame.toast(r.error.message)
    }
    readonly property string enableCommand: "systemctl --user enable --now anibeam-host"
    // "serving", "waiting" while this window has the library, or empty with no service.
    property string service: ""
    property var devices: []
    function reloadDevices() { var r = Door.listTailnetDevices(); if (!r.error) devices = r.reply.devices }
    function setTrusted(device, on) {
        var r = Door.setTailnetDeviceTrust(device, on)
        if (r.error) frame.toast(r.error.message)
        reloadDevices()
    }
    function deviceLine(d) {
        // Tailscale's own words for the system, as people write them
        var names = { linux: "Linux", windows: "Windows", android: "Android", ios: "iOS", macos: "macOS", freebsd: "FreeBSD", openbsd: "OpenBSD" }
        var os = names[d.os.toLowerCase()] || d.os
        var parts = []
        if (os !== "") parts.push(os)
        parts.push(d.online ? "Online" : "Offline")
        if (!d.trusted) parts.push("Gets nothing from this computer")
        return parts.join(" · ")
    }
    Component.onCompleted: { reloadHost(); reloadDevices(); service = Shell.hostService() }
    Connections { target: Door; function onTailnetDevicesChanged(devices) { tab.devices = devices } }
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
