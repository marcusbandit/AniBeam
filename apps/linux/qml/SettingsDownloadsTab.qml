// The Downloads tab: anirss at the left (how the core reaches it, its state, the search
// endpoints it was configured with), the torrent client at the right (which one, where,
// the login, a test). Edits sit in a draft until Save: the two secrets only cross once,
// and saving stops a running sidecar (its environment carries the password), so a save
// is one deliberate act followed by a reconnect.
import QtQuick
import QtCore
import QtQuick.Dialogs
import com.marcusrosado.AniBeam

SettingsTab {
    id: tab
    property var status: null
    property var draft: null
    property string tokenText: ""
    property bool tokenTouched: false
    property string passwordText: ""
    property bool passwordTouched: false
    property bool dirty: false
    property bool saving: false
    property real testJob: -1
    property bool testing: false
    property string confirmingRemove: ""
    property real endpointJob: -1
    readonly property var settings: status ? status.settings : null
    readonly property var anirss: status ? status.anirss : null
    readonly property var torrent: status ? status.torrent : null
    readonly property bool sidecar: !!draft && draft.anirss_mode === "Sidecar"
    readonly property var clients: [{ name: "qBittorrent", id: "Qbittorrent" }]
    readonly property var endpoints: status && status.endpoints ? status.endpoints : []
    readonly property string clientName: {
        for (var i = 0; i < clients.length; i++) if (draft && clients[i].id === draft.torrent_client) return clients[i].name
        return "qBittorrent"
    }

    function reload() {
        var r = Door.getDownloads()
        if (r.error) { frame.toast(r.error.message); return }
        tab.status = r.reply.status
        if (!tab.dirty) tab.draft = JSON.parse(JSON.stringify(r.reply.status.settings))
    }
    function edit(f) { var d = JSON.parse(JSON.stringify(draft)); f(d); draft = d; dirty = true }
    function save() {
        if (!draft) return
        var secrets = {}
        if (tokenTouched) secrets.anirss_token = tokenText
        if (passwordTouched) secrets.torrent_password = passwordText
        var r = Door.setDownloadSettings(draft, secrets)
        if (r.error) { frame.toast(r.error.message); return }
        dirty = false
        tokenText = ""; tokenTouched = false
        passwordText = ""; passwordTouched = false
        // The sidecar was stopped by the save; bring both halves back up on the new settings
        var c = Door.connectDownloads()
        if (c.error) frame.toast(c.error.message)
    }
    function discard() { dirty = false; tokenText = ""; tokenTouched = false; passwordText = ""; passwordTouched = false; reload() }
    function test() {
        if (dirty) { save(); return }
        var r = Door.testTorrentClient()
        if (r.error) { frame.toast(r.error.message); return }
        tab.testJob = r.reply.job
        tab.testing = true
    }
    function removeEndpoint(name) {
        tab.confirmingRemove = ""
        var r = Door.call("RemoveEndpoint", { name: name })
        if (r.error) { frame.toast(r.error.message); return }
        tab.endpointJob = r.reply.job
    }
    function exportPack(path) {
        var r = Door.call("ExportEndpointPack", {})
        if (r.error) { frame.toast(r.error.message); return }
        var w = Door.writeTextFile(path, JSON.stringify(r.reply.pack, null, 2) + "\n")
        if (w.error) { frame.toast(w.error.message); return }
        frame.toast("Endpoint pack saved")
    }
    function connect() {
        var r = Door.connectDownloads()
        if (r.error) frame.toast(r.error.message)
    }
    Connections {
        target: Door
        function onDownloadsStatusChanged(status) { tab.status = status; if (!tab.dirty) tab.draft = JSON.parse(JSON.stringify(status.settings)); tab.testing = false }
        function onJobFailed(job, kind, error) {
            if (job === tab.testJob) { tab.testing = false; frame.toast(error.message) }
            if (job === tab.endpointJob) { tab.endpointJob = -1; frame.toast(error.message) }
        }
        function onJobFinished(job, kind, ok) { if (job === tab.endpointJob) tab.endpointJob = -1 }
        function onJobCancelled(job, kind) { if (job === tab.testJob) tab.testing = false; if (job === tab.endpointJob) tab.endpointJob = -1 }
    }
    Component.onCompleted: reload()

    readonly property string anirssLine: {
        if (!anirss) return ""
        if (anirss.state === "Ready") return "anirss " + anirss.version + " is up at " + anirss.url + "."
        if (anirss.state === "Starting") return "Starting anirss."
        if (anirss.state === "Missing") return anirss.message
        if (anirss.state === "Failed") return "anirss is not running: " + anirss.message
        return "Not started yet. It starts the first time the Downloads page asks for it."
    }
    readonly property string torrentLine: {
        if (!torrent) return ""
        if (tab.testing) return "Trying " + clientName + "."
        if (torrent.connected) return clientName + " " + torrent.version + " answered."
        return torrent.message ? torrent.message : "Not tried yet."
    }
    readonly property string passwordLine: {
        if (!settings) return ""
        if (settings.has_torrent_password) return "A password is saved here."
        if (torrent && torrent.password_source === "anirss") return "Using the password anirss saved; save one here to stop depending on it."
        return "No password saved. A client that bypasses its login for this machine needs none."
    }
    // anirss and this app each have their own idea of where the client is; say so when they differ
    readonly property string mismatch: {
        if (!anirss || !settings || anirss.state !== "Ready" || !anirss.qbt_url) return ""
        var a = String(anirss.qbt_url).replace(/\/+$/, ""), b = String(settings.torrent_url).replace(/\/+$/, "")
        if (a === b) return ""
        return "anirss talks to " + a + " while AniBeam talks to " + b + ". Point both at the same client: anirss reads its address from its own config.toml."
    }

    SettingsPair {
        split: 1 / 2
        leftPanels: [ anirssPanel, endpointsPanel ]
        rightPanels: [ clientPanel, savePanel ]
    }
    Component {
        id: anirssPanel
        Panel {
            title: "anirss"; icon: "rss"
            helper: "anirss searches the release feeds and registers subscriptions and downloads in the torrent client. AniBeam can run it for you, or talk to one you already run."
            SettingRow { label: "How it runs"; helper: tab.sidecar ? "AniBeam starts anirss --serve on a port and token of its own and stops it when it quits." : "AniBeam talks to a server you run with anirss --serve, using its address and token."
                Seg { options: [{ text: "AniBeam runs it", icon: "play" }, { text: "Already running", icon: "radio" }]; index: tab.sidecar ? 0 : 1
                      onPicked: function(i) { tab.edit(function(d) { d.anirss_mode = i === 0 ? "Sidecar" : "Remote" }) } } }
            SettingRow { visible: tab.sidecar; label: "Command"; helper: "A name on PATH or a full path. ~/.local/bin is searched too."
                Field { objectName: "downloads-anirss-command"; text: tab.draft ? tab.draft.anirss_command : ""; mono: true; width: theme.space(40); onEdited: function(t) { tab.edit(function(d) { d.anirss_command = t }) } } }
            SettingRow { visible: !tab.sidecar; label: "Address"; helper: "The server's base, without /api/v1."
                Field { text: tab.draft ? tab.draft.anirss_url : ""; mono: true; width: theme.space(50); onEdited: function(t) { tab.edit(function(d) { d.anirss_url = t }) } } }
            SettingRow { visible: !tab.sidecar; label: "Token"; helper: tab.settings && tab.settings.has_anirss_token ? "A token is saved. Type a new one to replace it, or clear the field to remove it." : "The bearer token anirss printed when it started, or ANIRSS_API_TOKEN."
                Field { text: tab.tokenText; secret: true; mono: true; width: theme.space(50); placeholder: tab.settings && tab.settings.has_anirss_token ? "Saved" : "Not set"
                        onEdited: function(t) { tab.tokenText = t; tab.tokenTouched = true; tab.dirty = true } } }
            SettingRow { label: "Unsubscribe from finished shows"; helper: "16 days after a show's last episode airs, its RSS feed is removed. The show and its files stay."
                Switch { objectName: "downloads-prune-finished"; checked: !!tab.draft && tab.draft.prune_finished_feeds !== false; onToggled: function(on) { tab.edit(function(d) { d.prune_finished_feeds = on }) } } }
            SettingRow { label: "State"; line: tab.anirssLine; hairline: false
                Button { text: tab.anirss && tab.anirss.state === "Starting" ? "Starting" : tab.anirss && tab.anirss.state === "Ready" ? "Reconnect" : "Connect"; icon: "refresh-cw"; flat: true; enabled: !tab.dirty && !(tab.anirss && tab.anirss.state === "Starting"); onClicked: tab.connect() } }
        }
    }
    Component {
        id: endpointsPanel
        Panel {
            title: "Search endpoints"; icon: "globe"; grows: true
            helper: "Where searches go, in the order anirss tries them; a search that finds nothing on the first falls through to the next. They live in anirss's own config.toml."
            Repeater {
                model: tab.endpoints
                SettingRow {
                    id: epRow
                    required property var modelData
                    required property int index
                    width: parent ? parent.width : 0
                    label: (index + 1) + ". " + modelData.name
                    line: modelData.url
                    helper: modelData.on_demand !== null && modelData.on_demand !== undefined ? "Started on demand. Edit it in anirss's config." : ""
                    Item {
                        width: tab.confirmingRemove === epRow.modelData.name ? confirmRemove.implicitWidth : removeBtn.implicitWidth
                        height: theme.controlHeight
                        Button { id: removeBtn; objectName: "endpoint-remove-" + epRow.modelData.name; visible: tab.confirmingRemove !== epRow.modelData.name; text: "Remove"; icon: "trash-2"; flat: true; enabled: tab.endpointJob === -1; onClicked: tab.confirmingRemove = epRow.modelData.name }
                        InlineConfirm {
                            id: confirmRemove
                            visible: tab.confirmingRemove === epRow.modelData.name
                            question: "Remove " + epRow.modelData.name + "?"
                            onAccepted: tab.removeEndpoint(epRow.modelData.name)
                            onKept: tab.confirmingRemove = ""
                        }
                    }
                }
            }
            Note { visible: tab.endpoints.length === 0; text: tab.anirss && tab.anirss.state === "Ready" ? "No endpoints yet. Add one to search from the Downloads page." : "Connect to anirss to read them." }
            SettingRow { label: "Add or share"; helper: "Paste a site's search or RSS address, or import a pack. Export pack writes these endpoints to a file, tokens in their addresses included, with the best-fit ranking."
                Row { spacing: theme.space(2)
                    Button { objectName: "endpoints-add"; text: "Add"; icon: "plus"; enabled: !!(tab.anirss && tab.anirss.state === "Ready"); onClicked: frame.addEndpoints("url") }
                    Button { objectName: "endpoints-export"; text: "Export pack"; icon: "file-down"; flat: true; enabled: tab.endpoints.length > 0; onClicked: exportDialog.open() } } }
            SettingRow { label: "anirss's config"; helper: (tab.anirss && tab.anirss.config_path ? tab.anirss.config_path : "~/.config/anirss/config.toml") + " holds the endpoints, the save folders, the tags and the best-fit rules."; hairline: false
                Button { text: "Open"; icon: "external-link"; flat: true; enabled: !!(tab.anirss && tab.anirss.config_path); onClicked: frame.openLink("file://" + tab.anirss.config_path) } }
        }
    }
    Component {
        id: clientPanel
        Panel {
            title: "Torrent client"; icon: "download"
            helper: "The client anirss hands torrents to, and the one this page reads transfers from. The password is kept in the system password store and passed to anirss when AniBeam starts it."
            SettingRow { label: "Client"; helper: "qBittorrent's WebUI is the one client for now."
                Dropdown { objectName: "downloads-client"; options: tab.clients.map(function(c) { return c.name }); minWidth: theme.space(36)
                           index: { var i = 0; for (var k = 0; k < tab.clients.length; k++) if (tab.draft && tab.clients[k].id === tab.draft.torrent_client) i = k; return i }
                           onPicked: function(i) { tab.edit(function(d) { d.torrent_client = tab.clients[i].id }) } } }
            SettingRow { label: "Address"; helper: "The WebUI, as a URL. anirss's own config fills this in on a fresh install."
                Field { objectName: "downloads-client-url"; text: tab.draft ? tab.draft.torrent_url : ""; mono: true; width: theme.space(50); placeholder: "http://localhost:8080"; onEdited: function(t) { tab.edit(function(d) { d.torrent_url = t }) } } }
            SettingRow { label: "Username"
                Field { text: tab.draft ? tab.draft.torrent_username : ""; mono: true; width: theme.space(36); onEdited: function(t) { tab.edit(function(d) { d.torrent_username = t }) } } }
            SettingRow { label: "Password"; helper: tab.passwordLine
                Field { objectName: "downloads-client-password"; text: tab.passwordText; secret: true; mono: true; width: theme.space(36); placeholder: tab.settings && tab.settings.has_torrent_password ? "Saved" : "Not set"
                        onEdited: function(t) { tab.passwordText = t; tab.passwordTouched = true; tab.dirty = true } } }
            SettingRow { label: "State"; line: tab.torrentLine; status: tab.mismatch; hairline: false
                Button { text: tab.testing ? "Trying" : "Test"; icon: "check-check"; flat: true; enabled: !tab.testing; onClicked: tab.test() } }
        }
    }
    Component {
        id: savePanel
        Panel {
            title: "Apply"; icon: "check"
            helper: tab.dirty ? "Unsaved changes. Saving stops a running anirss and starts it again on the new settings." : "Everything above is saved."
            SettingRow { label: tab.dirty ? "Save the changes" : "Saved"; hairline: false
                Row { spacing: theme.space(2)
                    Button { text: "Discard"; flat: true; enabled: tab.dirty; onClicked: tab.discard() }
                    Button { objectName: "downloads-save"; text: "Save"; icon: "check"; primary: true; enabled: tab.dirty; onClicked: tab.save() } } }
        }
    }
    FileDialog {
        id: exportDialog
        fileMode: FileDialog.SaveFile
        defaultSuffix: "json"
        nameFilters: ["Endpoint pack (*.json)"]
        currentFolder: StandardPaths.writableLocation(StandardPaths.HomeLocation)
        selectedFile: currentFolder + "/anibeam-endpoints.json"
        onAccepted: tab.exportPack(decodeURIComponent(String(selectedFile).replace("file://", "")))
    }
}
