// The Downloads tab: anirss at the left (how the core reaches it, its state, the search
// endpoints it was configured with), the torrent client at the right (whose, which of the
// user's saved clients, where, the login, a test). Edits sit in a draft until Save: the
// secrets only cross once, and saving stops a running sidecar (its environment carries the
// password), so a save is one deliberate act followed by a reconnect. The address, user and
// password fields edit the saved client picked above them.
import QtQuick
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
    readonly property bool mine: !!draft && draft.torrent_mode === "Existing"
    readonly property var savedClients: draft && draft.saved_clients ? draft.saved_clients : []
    readonly property int activeIndex: {
        for (var i = 0; i < savedClients.length; i++) if (savedClients[i].id === draft.active_client) return i
        return -1
    }
    readonly property var activeEntry: activeIndex >= 0 ? savedClients[activeIndex] : null
    readonly property bool hasPassword: activeEntry ? !!activeEntry.has_password : !!(settings && settings.has_torrent_password)

    function reload() {
        var r = Door.getDownloads()
        if (r.error) { frame.toast(r.error.message); return }
        tab.status = r.reply.status
        if (!tab.dirty) tab.draft = JSON.parse(JSON.stringify(r.reply.status.settings))
    }
    function edit(f) { var d = JSON.parse(JSON.stringify(draft)); f(d); draft = d; dirty = true }
    function entryOf(d, id) { for (var i = 0; i < d.saved_clients.length; i++) if (d.saved_clients[i].id === id) return d.saved_clients[i]; return null }
    // A field edit lands in the client in use too, so picking another keeps what was typed
    function setActive(d, key, value) { var c = d.torrent_mode === "Existing" ? entryOf(d, d.active_client) : null; if (c) c[key] = value }
    function use(d, c) { d.active_client = c.id; d.torrent_url = c.url; d.torrent_username = c.username }
    // A password typed for one client is never saved as another's
    function forgetPassword() { passwordText = ""; passwordTouched = false }
    function pickClient(i) { tab.edit(function(d) { tab.use(d, d.saved_clients[i]) }); forgetPassword() }
    function addClient() {
        tab.edit(function(d) {
            // Never the id of one saved before this edit, even one removed in it: the id names
            // a password, and a removed client's must not pass to the new one
            var n = 2
            while (tab.entryOf(d, "c" + n) || (tab.settings && tab.settings.saved_clients && tab.entryOf(tab.settings, "c" + n))) n++
            var c = { id: "c" + n, name: "", url: "http://localhost:8080", username: "admin", has_password: false }
            d.saved_clients.push(c)
            d.torrent_mode = "Existing"
            tab.use(d, c)
        })
        forgetPassword()
    }
    function removeClient() {
        tab.edit(function(d) {
            d.saved_clients = d.saved_clients.filter(function(c) { return c.id !== d.active_client })
            tab.use(d, d.saved_clients[0])
        })
        forgetPassword()
    }
    function save() {
        if (!draft) return
        var secrets = {}
        if (tokenTouched) secrets.anirss_token = tokenText
        if (passwordTouched) secrets.torrent_password = passwordText
        var switched = !settings || settings.torrent_url !== draft.torrent_url || settings.torrent_mode !== draft.torrent_mode
        var r = Door.setDownloadSettings(draft, secrets)
        if (r.error) { frame.toast(r.error.message); return }
        if (switched) frame.forgetDownloadsRules()
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
    function connect() {
        var r = Door.connectDownloads()
        if (r.error) frame.toast(r.error.message)
    }
    Connections {
        target: Door
        function onDownloadsStatusChanged(status) { tab.status = status; if (!tab.dirty) tab.draft = JSON.parse(JSON.stringify(status.settings)); tab.testing = false }
        function onJobFailed(job, kind, error) { if (job === tab.testJob) { tab.testing = false; frame.toast(error.message) } }
        function onJobCancelled(job, kind) { if (job === tab.testJob) tab.testing = false }
    }
    Component.onCompleted: reload()

    readonly property string anirssLine: {
        if (!anirss) return ""
        if (anirss.state === "Ready") return "anirss " + anirss.version + " is up at " + anirss.url + "."
        if (anirss.state === "Starting") return "Starting anirss."
        if (anirss.state === "Missing") return "anirss is not installed. " + anirss.message
        if (anirss.state === "Failed") return "anirss is not running: " + anirss.message
        return "Not started yet. It starts the first time the Downloads page asks for it."
    }
    readonly property string torrentLine: {
        if (!torrent) return ""
        if (tab.testing) return "Trying " + clientName + "."
        if (torrent.connected) return clientName + " " + torrent.version + " answered."
        return torrent.message ? torrent.message : "Not tried yet."
    }
    readonly property bool managed: !!draft && draft.torrent_mode === "Managed"
    readonly property string modeLine: {
        if (!draft) return ""
        if (draft.torrent_mode === "Existing") return "Your own qBittorrent, with its Web UI turned on."
        if (draft.torrent_mode === "Managed") return "AniBeam's own qbittorrent-nox. It keeps running after AniBeam quits so subscriptions keep downloading, into ~/Downloads/Anime."
        return "On first use: a qBittorrent that answers below, else AniBeam's own."
    }
    readonly property string passwordLine: {
        if (!settings) return ""
        if (tab.hasPassword) return "A password is saved here for this client."
        if (torrent && torrent.password_source === "anirss") return "Using the password anirss saved; save one here to stop depending on it."
        return "No password saved. A client that bypasses its login for this machine needs none."
    }
    // anirss and this app each have their own idea of where the client is; say so when they differ
    readonly property string mismatch: {
        if (!anirss || !settings || anirss.state !== "Ready" || !anirss.qbt_url) return ""
        var a = String(anirss.qbt_url).replace(/\/+$/, ""), b = String(settings.torrent_url).replace(/\/+$/, "")
        if (a === b) return ""
        return "anirss talks to " + a + " while AniBeam talks to " + b + ". A remote anirss reads the address from its own config.toml, so switching clients here does not move it."
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
            SettingRow { label: "State"; line: tab.anirssLine; hairline: false
                Button { text: tab.anirss && tab.anirss.state === "Starting" ? "Starting" : tab.anirss && tab.anirss.state === "Ready" ? "Reconnect" : "Connect"; icon: "refresh-cw"; flat: true; enabled: !tab.dirty && !(tab.anirss && tab.anirss.state === "Starting"); onClicked: tab.connect() } }
        }
    }
    Component {
        id: endpointsPanel
        Panel {
            title: "Search endpoints"; icon: "globe"; grows: true
            helper: "Where searches go, in the order anirss tries them. They live in anirss's own config.toml; a search that finds nothing on the first falls through to the next."
            Flow {
                visible: tab.endpoints.length > 0
                width: parent ? parent.width : theme.space(100)
                spacing: theme.space(1.5)
                Repeater {
                    model: tab.endpoints
                    Chip {
                        required property var modelData
                        required property int index
                        text: (index + 1) + ". " + modelData.name + " (" + modelData.kind + ")"
                        small: true
                        Tooltip { text: modelData.url }
                    }
                }
            }
            Note { visible: tab.endpoints.length === 0; text: tab.anirss && tab.anirss.state === "Ready" ? "anirss has no endpoints configured." : "Connect to anirss to read them." }
            SettingRow { label: "anirss's config"; helper: (tab.anirss && tab.anirss.config_path ? tab.anirss.config_path : "~/.config/anirss/config.toml") + " holds the endpoints, the save folders, the tags and the best-fit rules."; hairline: false
                Button { text: "Open"; icon: "external-link"; flat: true; enabled: !!(tab.anirss && tab.anirss.config_path); onClicked: frame.openLink("file://" + tab.anirss.config_path) } }
        }
    }
    Component {
        id: clientPanel
        Panel {
            title: "Torrent client"; icon: "download"
            helper: "The client anirss hands torrents to, and the one this page reads transfers from. AniBeam uses a qBittorrent you already run, or runs its own when you have none. A password is kept in the system password store and passed to anirss when AniBeam starts it."
            SettingRow { label: "Whose"; helper: tab.modeLine
                Seg { objectName: "downloads-client-mode"; options: [{ text: "Pick for me", icon: "sparkles" }, { text: "Mine", icon: "user-check" }, { text: "AniBeam's own", icon: "play" }]
                      index: !tab.draft ? 0 : tab.draft.torrent_mode === "Existing" ? 1 : tab.draft.torrent_mode === "Managed" ? 2 : 0
                      onPicked: function(i) {
                          tab.edit(function(d) {
                              d.torrent_mode = ["Auto", "Existing", "Managed"][i]
                              // Mine is the saved client used last, else the first
                              var c = i === 1 ? (tab.entryOf(d, d.active_client) || d.saved_clients[0]) : null
                              if (c) tab.use(d, c)
                          })
                          tab.forgetPassword()
                      } } }
            SettingRow { visible: tab.mine; label: "Client"; helper: "Your qBittorrents, a seedbox and the one on this machine say. The one picked here is the one Downloads use; the client chip on the Downloads page switches between them too."
                Row { spacing: theme.space(2)
                    Dropdown { objectName: "downloads-client"; options: tab.savedClients.map(function(c) { return c.name || c.url }); minWidth: theme.space(36)
                               visible: tab.savedClients.length > 0; index: Math.max(0, tab.activeIndex)
                               onPicked: function(i) { tab.pickClient(i) } }
                    Button { objectName: "downloads-client-add"; text: "Add"; icon: "plus"; flat: true; onClicked: tab.addClient() }
                    Button { objectName: "downloads-client-remove"; text: "Remove"; icon: "trash-2"; flat: true; enabled: tab.savedClients.length > 1; onClicked: tab.removeClient() } } }
            SettingRow { visible: tab.mine; label: "Name"; helper: "What the client chip and this list call it."
                Field { objectName: "downloads-client-name"; text: tab.activeEntry ? tab.activeEntry.name : ""; width: theme.space(36); placeholder: "Named after its address"
                        onEdited: function(t) { tab.edit(function(d) { tab.setActive(d, "name", t) }) } } }
            SettingRow { label: "Address"; helper: tab.managed ? "Where AniBeam's own qBittorrent listens. Only this machine can reach it, and it needs no password." : "The WebUI, as a URL. anirss's own config fills this in on a fresh install."
                Field { objectName: "downloads-client-url"; text: tab.draft ? tab.draft.torrent_url : ""; mono: true; width: theme.space(50); placeholder: "http://localhost:8080"; onEdited: function(t) { tab.edit(function(d) { d.torrent_url = t; tab.setActive(d, "url", t) }) } } }
            SettingRow { visible: !tab.managed; label: "Username"
                Field { text: tab.draft ? tab.draft.torrent_username : ""; mono: true; width: theme.space(36); onEdited: function(t) { tab.edit(function(d) { d.torrent_username = t; tab.setActive(d, "username", t) }) } } }
            SettingRow { visible: !tab.managed; label: "Password"; helper: tab.passwordLine
                Field { objectName: "downloads-client-password"; text: tab.passwordText; secret: true; mono: true; width: theme.space(36); placeholder: tab.hasPassword ? "Saved" : "Not set"
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
}
