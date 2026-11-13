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
    title: words.t("settings-phone-title")
    lead: words.t("settings-phone-lead")
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
    function reloadDevices() { var r = Door.call("ListTailnetDevices", {}); if (!r.error) devices = r.reply.devices }
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
        if (!d.trusted) parts.push(words.t("settings-phone-untrusted"))
        return parts.join(" · ")
    }
    Component.onCompleted: { reloadHost(); reloadDevices(); service = Shell.hostService() }
    Connections { target: Door; function onTailnetDevicesChanged(devices) { tab.devices = devices } }
    Connections { target: Door; function onHostChanged(status) { tab.host = status } }
    readonly property bool on: !!Door.preferences.share_over_tailscale

    Panel {
        objectName: "settings-share"
        title: words.t("settings-phone-sharing")
        helper: words.t("settings-phone-sharing-helper")
        SettingRow {
            label: words.t("settings-phone-share")
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
            label: words.t("settings-phone-stream")
            helper: Door.preferences.stream_over_tailscale !== false
                ? words.t("settings-phone-stream-on")
                : words.t("settings-phone-stream-off")
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
            helper: tab.service === "" ? words.t("settings-phone-service-off-helper")
                : words.t("settings-phone-service-on-helper")
            Button {
                visible: tab.service === ""
                msg: "settings-phone-copy-command"; icon: "copy"; flat: true; small: true
                onClicked: frame.copyText(tab.enableCommand, words.t("settings-phone-command-copied"))
            }
        }
    }
    Panel {
        objectName: "settings-tailnet-devices"
        visible: tab.devices.length > 0
        title: words.t("settings-phone-devices")
        helper: words.t("settings-phone-devices-helper")
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
        title: words.t("settings-phone-addresses")
        helper: words.t("settings-phone-addresses-helper")
        Repeater {
            model: tab.host.urls
            SettingRow {
                required property var modelData
                label: modelData
                Button { msg: "settings-phone-copy"; icon: "copy"; flat: true; small: true; onClicked: frame.copyText(modelData, words.t("settings-phone-address-copied")) }
            }
        }
    }
}
