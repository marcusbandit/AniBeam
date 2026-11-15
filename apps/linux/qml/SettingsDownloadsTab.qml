// Downloads: the torrent client first (which one, which of the person's saved clients,
// where, the login, a test), then how searches run (the search helper, anirss, or Anibeam
// itself) and, for the helper, how Anibeam reaches it and its state, Rolling
// downloads, finished shows, the search sources anirss was configured with and the endpoint
// repos. Edits sit in a draft until Save: the secrets only cross once, and saving stops a
// running sidecar (its environment carries the password), so a save is one deliberate act
// followed by a reconnect. While the draft differs, a bar floats at the foot of the section
// with Discard and Save, so the way to keep a change is never a scroll away from it.
import QtQuick
import com.marcusrosado.AniBeam

SettingsTab {
    id: tab
    title: words.t("settings-downloads-title")
    lead: words.t("settings-downloads-lead")
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
        if (k === 0) return words.t("settings-downloads-keep-none")
        return words.t("settings-downloads-keep-line", { count: k, finishing: k + 2 })
    }
    readonly property bool sidecar: !!draft && draft.anirss_mode === "Sidecar"
    // The engine the draft picks. A stored engine this build does not know comes in as
    // Anibeam itself, the default (#552), and the core keeps it stored until the search
    // helper is picked here (#515)
    readonly property bool nativeEngine: !!draft && draft.engine === "Native"
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
        if (r.error) { frame.fail(r.error, tab.reload); return }
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
        // Another client, or another engine reading the rules, makes the subscriptions read so far stale
        var switched = !settings || settings.torrent_url !== draft.torrent_url || settings.torrent_mode !== draft.torrent_mode || settings.engine !== draft.engine
        var r = Door.setDownloadSettings(draft, secrets)
        if (r.error) { frame.fail(r.error, tab.save); return }
        if (switched) frame.forgetDownloadsRules()
        dirty = false
        tokenText = ""; tokenTouched = false
        passwordText = ""; passwordTouched = false
        // The sidecar was stopped by the save; bring both halves back up on the new settings
        var c = Door.connectDownloads()
        if (c.error) frame.fail(c.error, tab.connect)
    }
    function discard() { dirty = false; tokenText = ""; tokenTouched = false; passwordText = ""; passwordTouched = false; reload() }
    // Unsaved edits are saved first, then the test runs on them
    function test() {
        if (dirty) { save(); if (dirty) return }
        var r = Door.testTorrentClient()
        if (r.error) { frame.fail(r.error, tab.test); return }
        tab.testJob = r.reply.job
        tab.testing = true
    }
    // A helper's or client's own words, one click away: the toast's Details opens and copies them
    function details(said, raw) { frame.toast(said, 10, "", null, raw) }
    function connect() {
        var r = Door.connectDownloads()
        if (r.error) frame.fail(r.error, tab.connect)
    }
    Connections {
        target: Door
        function onDownloadsStatusChanged(status) { tab.status = status; if (!tab.dirty) tab.draft = JSON.parse(JSON.stringify(status.settings)); tab.testing = false }
        function onJobFailed(job, kind, error) { if (job === tab.testJob) { tab.testing = false; frame.fail(error, tab.test) } }
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
        if (anirss.state === "Ready") return words.t("settings-downloads-anirss-ready", { version: anirss.version, url: anirss.url })
        if (anirss.state === "Starting") return words.t("settings-downloads-anirss-starting")
        if (anirss.state === "Missing") return words.t("settings-downloads-anirss-missing")
        if (anirss.state === "Failed") return words.t("settings-downloads-anirss-failed")
        return words.t("settings-downloads-anirss-idle")
    }
    readonly property string torrentLine: {
        if (!torrent) return ""
        if (tab.testing) return words.t("settings-downloads-torrent-trying", { client: clientName })
        if (torrent.connected) return words.t("settings-downloads-torrent-answered", { client: clientName, version: torrent.version })
        return torrent.message ? words.t("settings-downloads-torrent-failed", { client: clientName }) : words.t("settings-downloads-torrent-untried")
    }
    readonly property bool managed: !!draft && draft.torrent_mode === "Managed"
    readonly property string modeLine: {
        if (!draft) return ""
        if (draft.torrent_mode === "Existing") return words.t("settings-downloads-mode-existing")
        if (draft.torrent_mode === "Managed") return words.t("settings-downloads-mode-managed")
        return words.t("settings-downloads-mode-auto")
    }
    readonly property string passwordLine: {
        if (!settings) return ""
        if (tab.hasPassword) return words.t("settings-downloads-password-saved")
        if (torrent && torrent.password_source === "anirss") return words.t("settings-downloads-password-anirss")
        return words.t("settings-downloads-password-none")
    }
    // anirss and this app each have their own idea of where the client is; say so when they differ
    readonly property string mismatch: {
        if (tab.nativeEngine || !anirss || !settings || anirss.state !== "Ready" || !anirss.qbt_url) return ""
        var a = String(anirss.qbt_url).replace(/\/+$/, ""), b = String(settings.torrent_url).replace(/\/+$/, "")
        if (a === b) return ""
        return words.t("settings-downloads-mismatch", { helperUrl: a, url: b })
    }

    Panel {
        title: words.t("settings-downloads-torrent-title")
        helper: words.t("settings-downloads-torrent-helper")
        SettingRow { label: words.t("settings-downloads-which-client"); helper: tab.modeLine
            Seg { objectName: "downloads-client-mode"; options: [{ text: words.t("settings-downloads-pick-for-me"), icon: "sparkles" }, { text: words.t("settings-downloads-my-own"), icon: "user-check" }, { text: words.t("settings-downloads-built-in"), icon: "play" }]
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
        SettingRow { visible: tab.mine; label: words.t("settings-downloads-saved-clients"); helper: words.t("settings-downloads-saved-clients-helper")
            Row { spacing: theme.space(2)
                Dropdown { objectName: "downloads-client"; options: tab.savedClients.map(function(c) { return c.name || c.url }); minWidth: theme.space(36)
                           visible: tab.savedClients.length > 0; index: Math.max(0, tab.activeIndex)
                           onPicked: function(i) { tab.pickClient(i) } }
                Button { objectName: "downloads-client-add"; msg: "settings-downloads-add-client"; icon: "plus"; flat: true; onClicked: tab.addClient() }
                Button { objectName: "downloads-client-remove"; msg: "action-remove"; icon: "trash-2"; flat: true; visible: tab.savedClients.length > 1; onClicked: tab.removeClient() } } }
        SettingRow { visible: tab.mine; label: words.t("settings-downloads-name")
            Field { objectName: "downloads-client-name"; text: tab.activeEntry ? tab.activeEntry.name : ""; width: theme.space(48); placeholder: words.t("settings-downloads-name-placeholder")
                    onEdited: function(t) { tab.edit(function(d) { tab.setActive(d, "name", t) }) } } }
        SettingRow { label: words.t("settings-downloads-address"); helper: tab.managed ? words.t("settings-downloads-address-managed-helper") : words.t("settings-downloads-address-helper")
            Field { objectName: "downloads-client-url"; text: tab.draft ? tab.draft.torrent_url : ""; mono: true; width: theme.space(48); placeholder: "http://localhost:8080"; onEdited: function(t) { tab.edit(function(d) { d.torrent_url = t; tab.setActive(d, "url", t) }) } } }
        SettingRow { visible: !tab.managed; label: words.t("settings-downloads-username")
            Field { text: tab.draft ? tab.draft.torrent_username : ""; mono: true; width: theme.space(48); onEdited: function(t) { tab.edit(function(d) { d.torrent_username = t; tab.setActive(d, "username", t) }) } } }
        SettingRow { visible: !tab.managed; label: words.t("settings-downloads-password"); helper: tab.passwordLine
            Field { objectName: "downloads-client-password"; text: tab.passwordText; secret: true; mono: true; width: theme.space(48); placeholder: tab.hasPassword ? words.t("settings-downloads-secret-saved") : words.t("settings-downloads-secret-unset")
                    onEdited: function(t) { tab.passwordText = t; tab.passwordTouched = true; tab.dirty = true } } }
        SettingRow { label: words.t("settings-downloads-connection"); line: tab.torrentLine; status: tab.mismatch
            Row { spacing: theme.space(2)
                Button { visible: tab.mismatch !== "" && !!(tab.anirss && tab.anirss.config_path); msg: "settings-downloads-open-config"; icon: "external-link"; flat: true; onClicked: frame.openLink(Fmt.fileUrl(tab.anirss.config_path)) }
                Button { visible: !!tab.torrent && !tab.torrent.connected && !tab.testing && !!tab.torrent.message; msg: "settings-downloads-details"; icon: "info"; flat: true; onClicked: tab.details(words.t("settings-downloads-details-torrent", { client: tab.clientName }), tab.torrent.message) }
                Button { text: tab.testing ? words.t("settings-downloads-test-trying") : tab.dirty ? words.t("settings-downloads-test-save") : words.t("settings-downloads-test"); icon: "check-check"; flat: true; enabled: !tab.testing; onClicked: tab.test() } } }
    }
    Panel {
        title: tab.nativeEngine ? words.t("settings-downloads-searching") : words.t("settings-downloads-search-helper")
        helper: tab.nativeEngine ? words.t("settings-downloads-searching-helper") : words.t("settings-downloads-search-helper-helper")
        SettingRow { label: words.t("settings-downloads-runs-in"); helper: tab.nativeEngine ? words.t("settings-downloads-runs-in-native-helper") : words.t("settings-downloads-runs-in-helper")
            Seg { objectName: "downloads-engine"; options: [{ text: words.t("settings-downloads-search-helper"), icon: "wrench", name: "helper" }, { text: words.t("settings-downloads-engine-native"), icon: "sparkles", name: "anibeam" }]; index: tab.nativeEngine ? 1 : 0
                  onPicked: function(i) { tab.edit(function(d) { d.engine = i === 1 ? "Native" : "Anirss" }) } } }
        SettingRow { visible: !tab.nativeEngine; label: words.t("settings-downloads-how-runs"); helper: tab.sidecar ? words.t("settings-downloads-how-runs-sidecar") : words.t("settings-downloads-how-runs-remote")
            Seg { options: [{ text: words.t("settings-downloads-runs-sidecar"), icon: "play" }, { text: words.t("settings-downloads-runs-remote"), icon: "radio" }]; index: tab.sidecar ? 0 : 1
                  onPicked: function(i) { tab.edit(function(d) { d.anirss_mode = i === 0 ? "Sidecar" : "Remote" }) } } }
        SettingRow { visible: !tab.nativeEngine && tab.sidecar; label: words.t("settings-downloads-command"); helper: words.t("settings-downloads-command-helper")
            Field { objectName: "downloads-anirss-command"; text: tab.draft ? tab.draft.anirss_command : ""; mono: true; width: theme.space(48); onEdited: function(t) { tab.edit(function(d) { d.anirss_command = t }) } } }
        SettingRow { visible: !tab.nativeEngine && !tab.sidecar; label: words.t("settings-downloads-address"); helper: words.t("settings-downloads-server-address-helper")
            Field { text: tab.draft ? tab.draft.anirss_url : ""; mono: true; width: theme.space(48); onEdited: function(t) { tab.edit(function(d) { d.anirss_url = t }) } } }
        SettingRow { visible: !tab.nativeEngine && !tab.sidecar; label: words.t("settings-downloads-key"); helper: tab.settings && tab.settings.has_anirss_token ? words.t("settings-downloads-key-saved-helper") : words.t("settings-downloads-key-helper")
            Field { text: tab.tokenText; secret: true; mono: true; width: theme.space(48); placeholder: tab.settings && tab.settings.has_anirss_token ? words.t("settings-downloads-secret-saved") : words.t("settings-downloads-secret-unset")
                    onEdited: function(t) { tab.tokenText = t; tab.tokenTouched = true; tab.dirty = true } } }
        SettingRow { visible: !tab.nativeEngine; label: words.t("settings-downloads-connection"); line: tab.anirssLine
            Row { spacing: theme.space(2)
                Button { visible: !!tab.anirss && (tab.anirss.state === "Missing" || tab.anirss.state === "Failed") && !!tab.anirss.message; msg: "settings-downloads-details"; icon: "info"; flat: true; onClicked: tab.details(words.t("settings-downloads-details-anirss"), tab.anirss.message) }
                Button { text: tab.anirss && tab.anirss.state === "Starting" ? words.t("settings-downloads-starting") : tab.anirss && tab.anirss.state === "Ready" ? words.t("settings-downloads-reconnect") : tab.anirss && tab.anirss.state === "Failed" ? words.t("action-try-again") : words.t("action-connect"); icon: "refresh-cw"; flat: true; enabled: !tab.dirty && !(tab.anirss && tab.anirss.state === "Starting"); onClicked: tab.connect()
                    Tooltip { text: tab.dirty ? words.t("settings-downloads-save-first") : "" } } } }
    }
    Panel {
        title: words.t("settings-downloads-roll-title")
        helper: words.t("settings-downloads-roll-helper")
        SettingRow { label: words.t("settings-downloads-roll-big"); helper: words.t("settings-downloads-roll-big-helper", { count: tab.roll.over })
            Switch { objectName: "downloads-roll-big"; checked: tab.roll.big !== false; onToggled: function(on) { tab.edit(function(d) { tab.rolling(d).big = on }) } } }
        SettingRow { label: words.t("settings-downloads-roll-ahead"); helper: words.t("settings-downloads-roll-ahead-helper")
            SliderRow { objectName: "downloads-roll-ahead"; unit: words.t("settings-downloads-unit-episodes"); from: 1; to: 52; stepSize: 1; trackWidth: theme.space(32); value: tab.roll.ahead
                        onMoved: function(v) { tab.edit(function(d) { tab.rolling(d).ahead = Math.round(v) }) } } }
        SettingRow { label: words.t("settings-downloads-roll-over"); helper: words.t("settings-downloads-roll-over-helper")
            SliderRow { objectName: "downloads-roll-over"; unit: words.t("settings-downloads-unit-episodes"); from: 12; to: 200; stepSize: 1; trackWidth: theme.space(32); value: tab.roll.over
                        onMoved: function(v) { tab.edit(function(d) { tab.rolling(d).over = Math.round(v) }) } } }
        SettingRow { label: words.t("settings-downloads-roll-watched"); helper: tab.roll.delete_watched !== false ? words.t("settings-downloads-roll-watched-delete-helper") : words.t("settings-downloads-roll-watched-keep-helper")
            Seg { objectName: "downloads-roll-watched"; options: [{ text: words.t("action-delete"), icon: "trash-2" }, { text: words.t("settings-downloads-keep-all"), icon: "archive" }]; index: tab.roll.delete_watched !== false ? 0 : 1
                  onPicked: function(i) { tab.edit(function(d) { tab.rolling(d).delete_watched = i === 0 }) } } }
        SettingRow { visible: tab.roll.delete_watched !== false; label: words.t("settings-downloads-keep-behind")
            helper: tab.keepLine
            SliderRow { objectName: "downloads-roll-keep"; unit: words.t("settings-downloads-unit-episodes"); from: 0; to: 12; stepSize: 1; trackWidth: theme.space(32); value: tab.roll.keep_watched === undefined ? 1 : tab.roll.keep_watched
                        onMoved: function(v) { tab.edit(function(d) { tab.rolling(d).keep_watched = Math.round(v) }) } } }
    }
    Panel {
        title: words.t("settings-downloads-subscriptions-title")
        SettingRow { label: words.t("settings-downloads-last-search"); helper: words.t("settings-downloads-last-search-helper")
            Switch { checked: frame.prefs.keepDownloadsQuery; onToggled: function(on) { frame.prefs.keepDownloadsQuery = on } } }
        SettingRow { label: words.t("settings-downloads-prune"); helper: words.t("settings-downloads-prune-helper")
            Switch { objectName: "downloads-prune-finished"; checked: !!tab.draft && tab.draft.prune_finished_feeds !== false; onToggled: function(on) { tab.edit(function(d) { d.prune_finished_feeds = on }) } } }
    }
    Panel {
        title: words.t("settings-downloads-sources-title")
        helper: words.t("settings-downloads-sources-helper")
        SettingRow { visible: tab.endpoints.length === 0; label: tab.nativeEngine || (tab.anirss && tab.anirss.state === "Ready") ? words.t("settings-downloads-sources-empty") : words.t("settings-downloads-sources-connect")
            helper: tab.nativeEngine || (tab.anirss && tab.anirss.state === "Ready") ? words.t("settings-downloads-sources-empty-helper") : ""
            Button { visible: !(tab.nativeEngine || (tab.anirss && tab.anirss.state === "Ready")); msg: "action-connect"; icon: "refresh-cw"; enabled: !tab.dirty; onClicked: tab.connect() } }
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
        SettingRow { label: words.t("settings-downloads-config-file"); helper: tab.nativeEngine ? words.t("settings-downloads-config-native-helper") : words.t("settings-downloads-config-helper")
            status: tab.anirss && tab.anirss.config_path ? tab.anirss.config_path : "~/.config/anirss/config.toml"
            Button { msg: "action-open"; icon: "external-link"; flat: true; enabled: !!(tab.anirss && tab.anirss.config_path); onClicked: frame.openLink(Fmt.fileUrl(tab.anirss.config_path)) } }
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
            Label { msg: "settings-downloads-unsaved"; role: "title"; font.pointSize: theme.typeNormal }
            Label { width: parent.width; msg: "settings-downloads-unsaved-helper"; role: "ui"; font.pointSize: theme.typeSmall; color: theme.textDim; elide: Text.ElideRight }
        }
        Row {
            id: saveRow
            anchors.right: parent.right; anchors.rightMargin: theme.space(4)
            anchors.verticalCenter: parent.verticalCenter
            spacing: theme.space(2)
            Button { msg: "settings-downloads-discard"; flat: true; enabled: tab.dirty; onClicked: tab.discard() }
            Button { objectName: "downloads-save"; msg: "action-save"; icon: "check"; primary: true; enabled: tab.dirty; onClicked: tab.save() }
        }
    }
}
