// Downloads: the torrent client first (which one, which of the person's saved clients,
// where, the login, a test), then anirss (how AniBeam reaches it, its state), Rolling
// downloads, finished shows, the search sources anirss was configured with and the endpoint
// repos. Edits sit in a draft until Save: the secrets only cross once, and saving stops a
// running sidecar (its environment carries the password), so a save is one deliberate act
// followed by a reconnect. While the draft differs, a bar floats at the foot of the section
// with Discard and Save, so the way to keep a change is never a scroll away from it.
import QtQuick
import com.marcusrosado.AniBeam

SettingsTab {
    id: tab
    title: "Downloads"
    lead: "Where AniBeam sends what you download, and how it searches the release feeds."
    footInset: theme.space(36)
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
    readonly property var roll: draft && draft.rolling ? draft.rolling : ({ big: true, ahead: 12, over: 30, delete_watched: true, keep_watched: 1 })
    // "Finishing episode 3 deletes episode 1" for the count kept behind
    readonly property string keepLine: {
        var k = roll.keep_watched === undefined ? 1 : roll.keep_watched
        if (k === 0) return "Finishing an episode deletes the one before it."
        return "Watched episodes kept behind the one you just finished. With " + k + ", finishing episode " + (k + 2) + " deletes episode 1."
    }
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
    // Settings saved before rolling existed carry none; fill the defaults the core reads
    function rolling(d) { if (!d.rolling) d.rolling = { big: true, ahead: 12, over: 30, delete_watched: true, keep_watched: 1 }; return d.rolling }
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
    // Unsaved edits are saved first, then the test runs on them
    function test() {
        if (dirty) { save(); if (dirty) return }
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
    // Edits not saved yet outlive a switch to another section or page: the frame keeps them
    // for this run, and they come back marked unsaved
    Component.onCompleted: {
        reload()
        var kept = frame.settingsDrafts.downloads
        if (!kept) return
        draft = kept.draft
        tokenText = kept.tokenText; tokenTouched = kept.tokenTouched
        passwordText = kept.passwordText; passwordTouched = kept.passwordTouched
        dirty = true
        frame.settingsDrafts = Object.assign({}, frame.settingsDrafts, { downloads: null })
    }
    Component.onDestruction: {
        if (!dirty) return
        frame.settingsDrafts = Object.assign({}, frame.settingsDrafts, { downloads: {
            draft: draft, tokenText: tokenText, tokenTouched: tokenTouched, passwordText: passwordText, passwordTouched: passwordTouched } })
    }

    readonly property string anirssLine: {
        if (!anirss) return ""
        if (anirss.state === "Ready") return "Running, version " + anirss.version + " at " + anirss.url + "."
        if (anirss.state === "Starting") return "Starting anirss."
        if (anirss.state === "Missing") return "anirss is not installed. " + anirss.message
        if (anirss.state === "Failed") return "anirss is not running: " + anirss.message
        return "Not started yet. It starts the first time you search on the Downloads page."
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
        if (draft.torrent_mode === "Existing") return "A qBittorrent you run yourself, with its Web UI turned on."
        if (draft.torrent_mode === "Managed") return "AniBeam runs a qBittorrent of its own into ~/Downloads/Anime. It keeps going after AniBeam quits, so subscriptions keep downloading."
        return "Uses a qBittorrent that answers at the address below, else runs one of its own."
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

    Panel {
        title: "Torrent client"
        helper: "The qBittorrent that does the downloading. Its password is kept in the system password store."
        SettingRow { label: "Which client"; helper: tab.modeLine
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
        SettingRow { visible: tab.mine; label: "Saved clients"; helper: "A seedbox and the one on this machine, say. The client chip on the Downloads page switches between them too."
            Row { spacing: theme.space(2)
                Dropdown { objectName: "downloads-client"; options: tab.savedClients.map(function(c) { return c.name || c.url }); minWidth: theme.space(36)
                           visible: tab.savedClients.length > 0; index: Math.max(0, tab.activeIndex)
                           onPicked: function(i) { tab.pickClient(i) } }
                Button { objectName: "downloads-client-add"; text: "Add"; icon: "plus"; flat: true; onClicked: tab.addClient() }
                Button { objectName: "downloads-client-remove"; text: "Remove"; icon: "trash-2"; flat: true; visible: tab.savedClients.length > 1; onClicked: tab.removeClient() } } }
        SettingRow { visible: tab.mine; label: "Name"
            Field { objectName: "downloads-client-name"; text: tab.activeEntry ? tab.activeEntry.name : ""; width: theme.space(48); placeholder: "Named after its address"
                    onEdited: function(t) { tab.edit(function(d) { tab.setActive(d, "name", t) }) } } }
        SettingRow { label: "Address"; helper: tab.managed ? "Only this machine can reach it, and it needs no password." : "The Web UI's address."
            Field { objectName: "downloads-client-url"; text: tab.draft ? tab.draft.torrent_url : ""; mono: true; width: theme.space(48); placeholder: "http://localhost:8080"; onEdited: function(t) { tab.edit(function(d) { d.torrent_url = t; tab.setActive(d, "url", t) }) } } }
        SettingRow { visible: !tab.managed; label: "Username"
            Field { text: tab.draft ? tab.draft.torrent_username : ""; mono: true; width: theme.space(48); onEdited: function(t) { tab.edit(function(d) { d.torrent_username = t; tab.setActive(d, "username", t) }) } } }
        SettingRow { visible: !tab.managed; label: "Password"; helper: tab.passwordLine
            Field { objectName: "downloads-client-password"; text: tab.passwordText; secret: true; mono: true; width: theme.space(48); placeholder: tab.hasPassword ? "Saved" : "Not set"
                    onEdited: function(t) { tab.passwordText = t; tab.passwordTouched = true; tab.dirty = true } } }
        SettingRow { label: "Connection"; line: tab.torrentLine; status: tab.mismatch
            Button { text: tab.testing ? "Trying" : tab.dirty ? "Save and test" : "Test"; icon: "check-check"; flat: true; enabled: !tab.testing; onClicked: tab.test() } }
    }
    Panel {
        title: "Search with anirss"
        helper: "anirss searches the release feeds and hands what you pick to the torrent client. AniBeam can run it for you, or use one you already run."
        SettingRow { label: "How it runs"; helper: tab.sidecar ? "AniBeam starts anirss itself when it opens and stops it when it quits." : "AniBeam uses an anirss you already run, at the address and with the key below."
            Seg { options: [{ text: "AniBeam runs it", icon: "play" }, { text: "I run it", icon: "radio" }]; index: tab.sidecar ? 0 : 1
                  onPicked: function(i) { tab.edit(function(d) { d.anirss_mode = i === 0 ? "Sidecar" : "Remote" }) } } }
        SettingRow { visible: tab.sidecar; label: "Command"; helper: "A name on PATH or a full path. ~/.local/bin is searched too."
            Field { objectName: "downloads-anirss-command"; text: tab.draft ? tab.draft.anirss_command : ""; mono: true; width: theme.space(48); onEdited: function(t) { tab.edit(function(d) { d.anirss_command = t }) } } }
        SettingRow { visible: !tab.sidecar; label: "Address"; helper: "The server's base, without /api/v1."
            Field { text: tab.draft ? tab.draft.anirss_url : ""; mono: true; width: theme.space(48); onEdited: function(t) { tab.edit(function(d) { d.anirss_url = t }) } } }
        SettingRow { visible: !tab.sidecar; label: "Token"; helper: tab.settings && tab.settings.has_anirss_token ? "A token is saved. Type a new one to replace it, or clear the field to remove it." : "The key anirss showed when it started. It lets AniBeam in."
            Field { text: tab.tokenText; secret: true; mono: true; width: theme.space(48); placeholder: tab.settings && tab.settings.has_anirss_token ? "Saved" : "Not set"
                    onEdited: function(t) { tab.tokenText = t; tab.tokenTouched = true; tab.dirty = true } } }
        SettingRow { label: "Connection"; line: tab.anirssLine
            Button { text: tab.anirss && tab.anirss.state === "Starting" ? "Starting" : tab.anirss && tab.anirss.state === "Ready" ? "Reconnect" : "Connect"; icon: "refresh-cw"; flat: true; enabled: !tab.dirty && !(tab.anirss && tab.anirss.state === "Starting"); onClicked: tab.connect()
                Tooltip { text: tab.dirty ? "Save your changes first" : "" } } }
    }
    Panel {
        title: "Rolling downloads"
        helper: "A long show downloads a few episodes at a time, and each one you finish starts the next. The whole run never has to fit on the disk."
        SettingRow { label: "Roll long downloads"; helper: "Anything over " + tab.roll.over + " episodes rolls by itself. Its row on the Downloads page can still get everything at once."
            Switch { objectName: "downloads-roll-big"; checked: tab.roll.big !== false; onToggled: function(on) { tab.edit(function(d) { tab.rolling(d).big = on }) } } }
        SettingRow { label: "Episodes ahead"; helper: "How many stay downloaded past the one you are on. 12 is about a season."
            SliderRow { objectName: "downloads-roll-ahead"; unit: "episodes"; from: 1; to: 52; stepSize: 1; trackWidth: theme.space(32); value: tab.roll.ahead
                        onMoved: function(v) { tab.edit(function(d) { tab.rolling(d).ahead = Math.round(v) }) } } }
        SettingRow { label: "Roll above"; helper: "Downloads with more episodes than this roll."
            SliderRow { objectName: "downloads-roll-over"; unit: "episodes"; from: 12; to: 200; stepSize: 1; trackWidth: theme.space(32); value: tab.roll.over
                        onMoved: function(v) { tab.edit(function(d) { tab.rolling(d).over = Math.round(v) }) } } }
        SettingRow { label: "Watched episodes"; helper: tab.roll.delete_watched !== false ? "Deleted once you are further along, so the disk only holds what is left to watch." : "Every file stays on the disk after you watch it."
            Seg { objectName: "downloads-roll-watched"; options: [{ text: "Delete", icon: "trash-2" }, { text: "Keep all", icon: "archive" }]; index: tab.roll.delete_watched !== false ? 0 : 1
                  onPicked: function(i) { tab.edit(function(d) { tab.rolling(d).delete_watched = i === 0 }) } } }
        SettingRow { visible: tab.roll.delete_watched !== false; label: "Keep behind"
            helper: tab.keepLine
            SliderRow { objectName: "downloads-roll-keep"; unit: "episodes"; from: 0; to: 12; stepSize: 1; trackWidth: theme.space(32); value: tab.roll.keep_watched === undefined ? 1 : tab.roll.keep_watched
                        onMoved: function(v) { tab.edit(function(d) { tab.rolling(d).keep_watched = Math.round(v) }) } } }
    }
    Panel {
        title: "Subscriptions and searches"
        SettingRow { label: "Keep the last search"; helper: "The Downloads page opens with the search you last ran. Off, it opens empty."
            Switch { checked: frame.prefs.keepDownloadsQuery; onToggled: function(on) { frame.prefs.keepDownloadsQuery = on } } }
        SettingRow { label: "Unsubscribe from finished shows"; helper: "16 days after a show's last episode airs, its feed is dropped. The show and its files stay."
            Switch { objectName: "downloads-prune-finished"; checked: !!tab.draft && tab.draft.prune_finished_feeds !== false; onToggled: function(on) { tab.edit(function(d) { d.prune_finished_feeds = on }) } } }
    }
    Panel {
        title: "Search sources"
        helper: "Where a search goes, in the order anirss tries them. One that finds nothing falls through to the next."
        SettingRow { visible: tab.endpoints.length === 0; label: tab.anirss && tab.anirss.state === "Ready" ? "anirss has no sources set up" : "Connect to anirss to list them" }
        Repeater {
            model: tab.endpoints
            SettingRow {
                required property var modelData
                required property int index
                label: modelData.name
                line: modelData.url
                Chip { text: String(modelData.kind); small: true; color: theme.raised }
            }
        }
        SettingRow { label: "anirss's config"; helper: "Holds the sources, the save folders, the tags and the best-fit rules."
            status: tab.anirss && tab.anirss.config_path ? tab.anirss.config_path : "~/.config/anirss/config.toml"
            Button { text: "Open"; icon: "external-link"; flat: true; enabled: !!(tab.anirss && tab.anirss.config_path); onClicked: frame.openLink(Fmt.fileUrl(tab.anirss.config_path)) } }
    }
    EndpointReposPanel { grows: false }

    // The save bar: floats over the section's foot while the draft differs from what is saved
    Rectangle {
        objectName: "downloads-save-bar"
        parent: tab
        z: 10
        x: tab.blockX
        width: tab.blockWidth
        height: theme.space(18)
        y: tab.height - (tab.dirty ? height + theme.space(5) : -theme.space(2))
        Behavior on y { enabled: !Theme.reducedMotion; NumberAnimation { duration: 220; easing.type: Easing.OutCubic } }
        visible: y < tab.height
        radius: theme.radiusTile
        color: theme.glassFloat
        border.width: 1
        border.color: theme.accentEdge
        Column {
            anchors.left: parent.left; anchors.leftMargin: theme.space(6)
            anchors.right: saveRow.left; anchors.rightMargin: theme.space(4)
            anchors.verticalCenter: parent.verticalCenter
            spacing: theme.space(0.5)
            Label { text: "Unsaved changes"; role: "title"; font.pointSize: theme.typeNormal }
            Label { width: parent.width; text: "Saving restarts anirss on the new settings."; role: "ui"; font.pointSize: theme.typeSmall; color: theme.textDim; elide: Text.ElideRight }
        }
        Row {
            id: saveRow
            anchors.right: parent.right; anchors.rightMargin: theme.space(4)
            anchors.verticalCenter: parent.verticalCenter
            spacing: theme.space(2)
            Button { text: "Discard"; flat: true; enabled: tab.dirty; onClicked: tab.discard() }
            Button { objectName: "downloads-save"; text: "Save"; icon: "check"; primary: true; enabled: tab.dirty; onClicked: tab.save() }
        }
    }
}
