// Endpoint repos (#143) on the desktop: the repos added by address, each repo's key
// fingerprint shown before it is trusted, the endpoints they offer installed one at a time,
// and endpoints added by hand beside them for a site no repo carries (one through
// Jackett). What is installed goes into Anibeam's own anirss, so the search sees it; one
// anirss already has from its own config.toml keeps its place and the row says so.
// The calls that fetch or store a secret run off the Qt thread (Door.callAsync), so a slow or
// dead repo never freezes the window; the tab refreshes the repos each time it opens.
import QtQuick
import com.marcusrosado.AniBeam

Panel {
    id: root
    objectName: "endpoint-repos"
    title: words.t("endpoint-repos-panel-title"); icon: "archive"; grows: true
    helper: words.t("endpoint-repos-panel-helper")

    property var catalog: null
    property var preview: null
    property string repoUrl: ""
    property string error: ""
    property bool busy: false
    // An install or update waiting for values: { call, args, fields, values, title, note },
    // title and note each a message as { id, args }, and action the button's message ID.
    property var asking: null
    property string customName: ""
    property string customUrl: ""
    property string customError: ""

    readonly property var repos: catalog ? catalog.repos : []
    readonly property var installed: catalog ? catalog.installed : []
    readonly property var offered: {
        if (!catalog) return []
        var out = []
        for (var i = 0; i < catalog.offered.length; i++) if (!catalog.offered[i].installed) out.push(catalog.offered[i])
        return out
    }

    function replyOf(r, name) { return r && r.kind === name ? r.reply : null }
    function take(r) {
        root.busy = false
        var c = replyOf(r, "EndpointCatalog")
        if (c) { root.catalog = c.catalog; root.error = ""; return true }
        if (r && r.error) { root.error = frame.plainText(r.error); root.errorFix = r.error.fix }
        return false
    }
    // Replies to calls run off the Qt thread, by token.
    property var waiting: ({})
    function later(call, args, done) {
        var t = Door.callAsync(call, args)
        if (t < 0) { root.busy = false; root.error = words.t("endpoint-repos-panel-could-not-start"); root.errorFix = "retry"; return }
        root.waiting[t] = done
    }
    Connections {
        target: Door
        function onCallDone(token, reply) {
            var done = root.waiting[token]
            if (!done) return
            delete root.waiting[token]
            done(reply)
        }
    }
    // What the error line's Try again runs: the last load, refresh or add
    property var again: null
    // The bridge's fix for the error shown: Try again only where trying again can help
    property var errorFix: undefined
    function load() { root.again = root.load; take(Door.call("ListRepoEndpoints", {})) }
    function refresh() { root.again = root.refresh; root.busy = true; later("RefreshEndpointRepos", {}, root.take) }
    function add() {
        if (root.repoUrl.trim() === "") return
        root.again = root.add
        root.busy = true; root.error = ""
        later("AddEndpointRepo", { url: root.repoUrl.trim() }, function(r) {
            root.busy = false
            var p = replyOf(r, "EndpointRepoPreview")
            if (p) root.preview = p.preview
            else if (r.error) { root.error = frame.plainText(r.error); root.errorFix = r.error.fix }
        })
    }
    function trust() {
        if (!root.preview) return
        root.again = root.trust
        root.busy = true
        var p = root.preview
        later("TrustEndpointRepo", { url: p.url, key: p.key }, function(r) {
            if (root.take(r)) { root.preview = null; root.repoUrl = "" }
        })
    }
    function remove(url) { root.again = function() { root.remove(url) }; root.take(Door.call("RemoveEndpointRepo", { url: url })) }
    function hostLine(e) { return e.host + "  " + e.kind + (e.lang ? "  " + e.lang : "") }
    function run(call, args) {
        root.again = function() { root.run(call, args) }
        root.busy = true
        later(call, args, function(r) { if (root.take(r)) root.asking = null })
    }
    function install(o) {
        if (o.auth.length === 0) { run("InstallEndpoint", { repo: o.repo_url, id: o.id, credentials: [] }); return }
        root.asking = { call: "InstallEndpoint", args: { repo: o.repo_url, id: o.id }, fields: o.auth, values: o.auth.map(function() { return "" }),
                        title: { id: "endpoint-repos-panel-needs-value", args: { name: o.name } }, note: { id: "endpoint-repos-panel-secret-note", args: {} } }
    }
    function update(e) {
        if (!e.update_asks) { run("UpdateEndpoint", { install: e.install_id, credentials: [] }); return }
        var fields = e.auth
        for (var i = 0; i < root.catalog.offered.length; i++) {
            var o = root.catalog.offered[i]
            if (o.repo_url === e.repo_url && o.id === e.id) fields = o.auth
        }
        root.asking = { call: "UpdateEndpoint", action: "endpoint-repos-panel-update", args: { install: e.install_id }, fields: fields, values: fields.map(function() { return "" }),
                        title: { id: "endpoint-repos-panel-asks-again", args: { name: e.name, version: String(e.update) } },
                        note: e.update_host ? { id: "endpoint-repos-panel-update-sends-to", args: { host: e.update_host } } : { id: "endpoint-repos-panel-fields-changed", args: {} } }
    }
    // The values an installed endpoint is missing, or with `all` every one, to change: a
    // field left empty keeps what is stored.
    function fill(e, all) {
        var fields = all ? e.auth : e.auth.filter(function(a) { return e.missing.indexOf(a.key) >= 0 })
        root.asking = { call: "SetEndpointCredentials", args: { install: e.install_id }, fields: fields, values: fields.map(function() { return "" }),
                        action: "action-save", title: { id: all ? "endpoint-repos-panel-values-of" : "endpoint-repos-panel-needs-value", args: { name: e.name } },
                        note: { id: all ? "endpoint-repos-panel-secret-note-keep" : "endpoint-repos-panel-secret-note", args: {} } }
    }
    function confirm() {
        var a = root.asking, creds = []
        for (var i = 0; i < a.fields.length; i++) creds.push({ key: a.fields[i].key, value: a.values[i] })
        var args = JSON.parse(JSON.stringify(a.args)); args.credentials = creds
        run(a.call, args)
    }
    function addCustom() {
        root.customError = ""
        later("AddCustomEndpoint", { name: root.customName.trim(), url: root.customUrl.trim(), credentials: [] }, function(r) {
            var c = replyOf(r, "EndpointCatalog")
            if (c) { root.catalog = c.catalog; root.customName = ""; root.customUrl = "" }
            else if (r.error) root.customError = frame.plainText(r.error)
        })
    }
    function installedNote(e) {
        if (e.missing.length > 0) return words.t("endpoint-repos-panel-needs-fields", { fields: e.auth.filter(function(a) { return e.missing.indexOf(a.key) >= 0 }).map(function(a) { return a.label }).join(", ") })
        if (e.desktop_note) return e.desktop_note + "."
        if (!e.offered) return words.t("endpoint-repos-panel-no-longer-offered")
        if (e.update_host) return words.t("endpoint-repos-panel-update-requests-to", { host: e.update_host })
        return ""
    }

    Component.onCompleted: {
        load()
        if (repos.length > 0) refresh()
    }

    // The repos.
    Repeater {
        model: root.repos
        SettingRow {
            required property var modelData
            label: modelData.name + (modelData.nsfw ? "  (18+)" : "")
            line: modelData.fingerprint
            helper: modelData.url
            status: modelData.error ? words.t("endpoint-repos-panel-unreadable") : ""
            Button { msg: "action-remove"; icon: "trash-2"; flat: true; small: true; onClicked: root.remove(modelData.url) }
        }
    }
    SettingRow {
        visible: !root.preview
        label: words.t("endpoint-repos-panel-add-list")
        helper: words.t("endpoint-repos-panel-add-list-helper")
        Row {
            spacing: theme.space(1.5)
            Field { objectName: "repos-url"; text: root.repoUrl; mono: true; width: theme.space(56); placeholder: "https://.../index.json"; onEdited: function(t) { root.repoUrl = t } }
            Button { text: root.busy ? words.t("endpoint-repos-panel-checking") : words.t("endpoint-repos-panel-add"); icon: "plus"; busy: root.busy; enabled: !root.busy && root.repoUrl.trim() !== ""; onClicked: root.add() }
        }
    }
    SettingRow {
        visible: !!root.preview
        label: root.preview ? words.t("endpoint-repos-panel-trust-question", { name: root.preview.name + (root.preview.nsfw ? " (18+)" : "") }) : ""
        line: root.preview ? words.t("endpoint-repos-panel-fingerprint", { fingerprint: root.preview.fingerprint }) : ""
        helper: root.preview ? words.t(root.preview.trusted ? "endpoint-repos-panel-preview-trusted" : "endpoint-repos-panel-preview", { url: root.preview.url, count: root.preview.endpoints.length }) : ""
        Row {
            spacing: theme.space(1.5)
            Button { msg: "endpoint-repos-panel-trust"; icon: "badge-check"; primary: true; enabled: !root.busy; onClicked: root.trust() }
            Button { msg: "action-cancel"; flat: true; onClicked: root.preview = null }
        }
    }
    Row {
        visible: root.error !== ""
        width: parent ? parent.width : theme.space(100)
        spacing: theme.space(2)
        Note { width: Math.min(implicitWidth, parent.width - (retry.visible ? retry.width + theme.space(2) : 0)); text: root.error; color: theme.red; anchors.verticalCenter: parent.verticalCenter }
        Button { id: retry; objectName: "repos-try-again"; visible: !!root.again && !root.busy && (root.errorFix === undefined || root.errorFix === "retry"); msg: "action-try-again"; icon: "refresh-cw"; flat: true; small: true; onClicked: root.again() }
    }

    // What an install asks for.
    Column {
        visible: !!root.asking
        width: parent ? parent.width : theme.space(100)
        spacing: theme.space(1)
        Note { text: root.asking ? words.t(root.asking.title.id, root.asking.title.args) : ""; color: theme.text }
        Note { text: root.asking ? words.t(root.asking.note.id, root.asking.note.args) : "" }
        Repeater {
            model: root.asking ? root.asking.fields : []
            SettingRow {
                required property var modelData
                required property int index
                label: modelData.label
                helper: modelData.help + (modelData.help_url ? "\n" + modelData.help_url : "")
                Field { secret: modelData.secret; mono: true; width: theme.space(44); placeholder: modelData.label
                        onEdited: function(t) { root.asking.values[index] = t } }
            }
        }
        Row {
            spacing: theme.space(1.5)
            Button { text: root.busy ? words.t("endpoint-repos-panel-saving") : words.t(root.asking && root.asking.action ? root.asking.action : "endpoint-repos-panel-install"); primary: true; busy: root.busy; onClicked: root.confirm() }
            Button { msg: "action-cancel"; flat: true; onClicked: root.asking = null }
        }
    }

    // Installed, in search order.
    Repeater {
        model: root.installed
        SettingRow {
            required property var modelData
            label: modelData.name
            line: modelData.host + "  " + modelData.kind + "  " + (modelData.custom ? words.t("endpoint-repos-panel-added-by-you") : words.t("endpoint-repos-panel-version", { version: String(modelData.version) }))
            status: root.installedNote(modelData)
            Row {
                spacing: theme.space(1.5)
                Button { visible: modelData.update !== null && modelData.update !== undefined; text: words.t("endpoint-repos-panel-update-to", { version: String(modelData.update) }); small: true; primary: true; onClicked: root.update(modelData) }
                Button { visible: modelData.missing.length > 0 && !(modelData.update !== null && modelData.update !== undefined); msg: "endpoint-repos-panel-add-value"; small: true; primary: true; onClicked: root.fill(modelData, false) }
                Button { visible: modelData.auth.length > 0 && modelData.missing.length === 0 && (modelData.update === null || modelData.update === undefined); text: words.t("endpoint-repos-panel-change", { fields: modelData.auth.map(function(a) { return a.label }).join(", ") }); small: true; flat: true; onClicked: root.fill(modelData, true) }
                Switch { anchors.verticalCenter: parent.verticalCenter; checked: modelData.enabled; onToggled: function(on) { root.take(Door.call("SetEndpointEnabled", { install: modelData.install_id, enabled: on })) } }
                Button { msg: "action-remove"; icon: "trash-2"; flat: true; small: true; onClicked: root.later("RemoveEndpoint", { install: modelData.install_id }, root.take) }
            }
        }
    }
    // Offered, not installed.
    Repeater {
        model: root.offered
        SettingRow {
            required property var modelData
            label: modelData.name + (modelData.nsfw ? "  (18+)" : "")
            line: root.hostLine(modelData)
            helper: modelData.auth.length > 0 ? words.t("endpoint-repos-panel-asks-for", { fields: modelData.auth.map(function(a) { return a.label }).join(", ") }) : ""
            Button { msg: "endpoint-repos-panel-install"; icon: "download"; small: true; onClicked: root.install(modelData) }
        }
    }

    // By hand.
    SettingRow {
        label: words.t("endpoint-repos-panel-by-hand")
        helper: words.t("endpoint-repos-panel-by-hand-helper")
        hairline: false
        Column {
            spacing: theme.space(1)
            Row {
                spacing: theme.space(1.5)
                Field { objectName: "custom-name"; text: root.customName; width: theme.space(28); placeholder: words.t("endpoint-repos-panel-name"); onEdited: function(t) { root.customName = t } }
                Field { objectName: "custom-url"; text: root.customUrl; mono: true; width: theme.space(40); placeholder: "http://...{query}..."; onEdited: function(t) { root.customUrl = t } }
                Button { msg: "endpoint-repos-panel-add"; icon: "plus"; enabled: root.customName.trim() !== "" && root.customUrl.trim() !== ""; onClicked: root.addCustom() }
            }
            Note { visible: root.customError !== ""; width: theme.space(60); text: root.customError; color: theme.red }
        }
    }
    foot: Row {
        spacing: theme.space(1.5)
        Button { text: root.busy ? words.t("endpoint-repos-panel-refreshing") : words.t("endpoint-repos-panel-refresh"); icon: "refresh-cw"; flat: true; small: true; busy: root.busy; enabled: root.repos.length > 0 && !root.busy; onClicked: root.refresh() }
    }
}
