// Phone app: the core serves a read-only copy of the library, the subscriptions and the
// downloads on this computer's Tailscale addresses while sharing is on. The page says what
// the phone gets before the switch, then the addresses the phone app should be given, one
// row each, with Copy.
import QtQuick
import com.marcusrosado.AniBeam

SettingsTab {
    id: tab
    title: "Phone app"
    lead: "AniBeam on your phone can browse this library, see what is airing and follow your downloads, wherever you are."
    property var host: ({ enabled: false, urls: [], message: "" })
    function reloadHost() { var r = Door.getHost(); if (!r.error) host = r.reply.status }
    function setSharing(on) {
        var p = JSON.parse(JSON.stringify(Door.preferences))
        p.share_over_tailscale = on
        var r = Door.setPreferences(p)
        if (r.error) frame.toast(r.error.message)
    }
    Component.onCompleted: reloadHost()
    Connections { target: Door; function onHostChanged(status) { tab.host = status } }
    readonly property bool on: !!Door.preferences.share_over_tailscale

    Panel {
        objectName: "settings-share"
        title: "Sharing"
        helper: "Only your own devices on Tailscale can reach it. The phone reads; it never changes anything here."
        SettingRow {
            label: "Share over Tailscale"
            line: tab.host.message
            Switch { objectName: "settings-share-switch"; checked: tab.on; onToggled: function(on) { tab.setSharing(on) } }
        }
    }
    Panel {
        visible: tab.on && tab.host.urls.length > 0
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
