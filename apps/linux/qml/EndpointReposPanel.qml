// Endpoint repos (#143) on the desktop: the repos added by address, each repo's key
// fingerprint shown before it is trusted, the endpoints they offer installed one at a time,
// and endpoints added by hand beside them for a site no repo carries (1337x through
// Jackett). What is installed goes into AniBeam's own anirss, so the search sees it; one
// anirss already has from its own config.toml keeps its place and the row says so.
// The calls that fetch or store a secret run off the Qt thread (Door.callAsync), so a slow or
// dead repo never freezes the window; the tab refreshes the repos each time it opens.
import QtQuick
import com.marcusrosado.AniBeam

Panel {
    id: root
    objectName: "endpoint-repos"
    title: "Endpoint repos"; icon: "archive"; grows: true
    helper: "A repo is a signed list of search endpoints, added by its address. AniBeam checks the signature and shows you the key before anything is saved, then you install the endpoints you want. Installed ones join anirss's list."

    property var catalog: null
    property var preview: null
    property string repoUrl: ""
    property string error: ""
    property bool busy: false
    // An install or update waiting for values: { call, args, fields, values, title, note }.
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
        if (r && r.error) root.error = frame.errorText(r.error)
        return false
    }
    // Replies to calls run off the Qt thread, by token.
    property var waiting: ({})
    function later(call, args, done) {
        var t = Door.callAsync(call, args)
        if (t < 0) { root.busy = false; root.error = "AniBeam could not start that call."; return }
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
    function load() { take(Door.call("ListRepoEndpoints", {})) }
    function refresh() { root.busy = true; later("RefreshEndpointRepos", {}, root.take) }
    function add() {
        if (root.repoUrl.trim() === "") return
        root.busy = true; root.error = ""
        later("AddEndpointRepo", { url: root.repoUrl.trim() }, function(r) {
            root.busy = false
            var p = replyOf(r, "EndpointRepoPreview")
            if (p) root.preview = p.preview
            else if (r.error) root.error = frame.errorText(r.error)
        })
    }
    function trust() {
        if (!root.preview) return
        root.busy = true
        var p = root.preview
        later("TrustEndpointRepo", { url: p.url, key: p.key }, function(r) {
            if (root.take(r)) { root.preview = null; root.repoUrl = "" }
        })
    }
    function hostLine(e) { return e.host + "  " + e.kind + (e.lang ? "  " + e.lang : "") }
    function run(call, args) {
        root.busy = true
        later(call, args, function(r) { if (root.take(r)) root.asking = null })
    }
    function install(o) {
        if (o.auth.length === 0) { run("InstallEndpoint", { repo: o.repo_url, id: o.id, credentials: [] }); return }
        root.asking = { call: "InstallEndpoint", args: { repo: o.repo_url, id: o.id }, fields: o.auth, values: o.auth.map(function() { return "" }),
                        title: o.name + " needs a value", note: "Kept in this machine's secret store, never in the library or an export." }
    }
    function update(e) {
        if (!e.update_asks) { run("UpdateEndpoint", { install: e.install_id, credentials: [] }); return }
        var fields = e.auth
        for (var i = 0; i < root.catalog.offered.length; i++) {
            var o = root.catalog.offered[i]
            if (o.repo_url === e.repo_url && o.id === e.id) fields = o.auth
        }
        root.asking = { call: "UpdateEndpoint", action: "Update", args: { install: e.install_id }, fields: fields, values: fields.map(function() { return "" }),
                        title: e.name + " version " + e.update + " asks again",
                        note: e.update_host ? "The new version sends this to " + e.update_host + "." : "Its fields changed." }
    }
    // The values an installed endpoint is missing, or with `all` every one, to change: a
    // field left empty keeps what is stored.
    function fill(e, all) {
        var fields = all ? e.auth : e.auth.filter(function(a) { return e.missing.indexOf(a.key) >= 0 })
        root.asking = { call: "SetEndpointCredentials", args: { install: e.install_id }, fields: fields, values: fields.map(function() { return "" }),
                        action: "Save", title: all ? e.name + "'s values" : e.name + " needs a value",
                        note: "Kept in this machine's secret store, never in the library or an export." + (all ? " A field left empty keeps what is stored." : "") }
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
            else if (r.error) root.customError = frame.errorText(r.error)
        })
    }
    function installedNote(e) {
        if (e.missing.length > 0) return "Needs its " + e.auth.filter(function(a) { return e.missing.indexOf(a.key) >= 0 }).map(function(a) { return a.label }).join(", ") + " before it searches."
        if (e.desktop_note) return e.desktop_note + "."
        if (!e.offered) return "No longer offered by its repo."
        if (e.update_host) return "The update sends requests to " + e.update_host + "."
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
            status: modelData.error ? modelData.error : ""
            Button { text: "Remove"; icon: "trash-2"; flat: true; small: true; onClicked: root.take(Door.call("RemoveEndpointRepo", { url: modelData.url })) }
        }
    }
    SettingRow {
        visible: !root.preview
        label: "Add a repo"
        helper: "The address of its index file, or a path to one on this machine."
        Row {
            spacing: theme.space(1.5)
            Field { objectName: "repos-url"; text: root.repoUrl; mono: true; width: theme.space(56); placeholder: "https://.../index.json"; onEdited: function(t) { root.repoUrl = t } }
            Button { text: root.busy ? "Checking" : "Add"; icon: "plus"; busy: root.busy; enabled: !root.busy && root.repoUrl.trim() !== ""; onClicked: root.add() }
        }
    }
    SettingRow {
        visible: !!root.preview
        label: root.preview ? "Trust " + root.preview.name + (root.preview.nsfw ? " (18+)" : "") + "?" : ""
        line: root.preview ? "Key fingerprint  " + root.preview.fingerprint : ""
        helper: root.preview ? root.preview.url + "\n" + root.preview.endpoints.length + " endpoints, signed" + (root.preview.trusted ? ", already trusted" : "") + ". Its endpoints decide where your searches go; trust a repo only from someone you trust." : ""
        Row {
            spacing: theme.space(1.5)
            Button { text: "Trust"; icon: "badge-check"; primary: true; enabled: !root.busy; onClicked: root.trust() }
            Button { text: "Cancel"; flat: true; onClicked: root.preview = null }
        }
    }
    Note { visible: root.error !== ""; text: root.error; color: theme.red }

    // What an install asks for.
    Column {
        visible: !!root.asking
        width: parent ? parent.width : theme.space(100)
        spacing: theme.space(1)
        Note { text: root.asking ? root.asking.title : ""; color: theme.text }
        Note { text: root.asking ? root.asking.note : "" }
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
            Button { text: root.busy ? "Saving" : (root.asking && root.asking.action ? root.asking.action : "Install"); primary: true; busy: root.busy; onClicked: root.confirm() }
            Button { text: "Cancel"; flat: true; onClicked: root.asking = null }
        }
    }

    // Installed, in search order.
    Repeater {
        model: root.installed
        SettingRow {
            required property var modelData
            label: modelData.name
            line: modelData.host + "  " + modelData.kind + "  " + (modelData.custom ? "added by you" : "v" + modelData.version)
            status: root.installedNote(modelData)
            Row {
                spacing: theme.space(1.5)
                Button { visible: modelData.update !== null && modelData.update !== undefined; text: "Update to v" + modelData.update; small: true; primary: true; onClicked: root.update(modelData) }
                Button { visible: modelData.missing.length > 0 && !(modelData.update !== null && modelData.update !== undefined); text: "Add its value"; small: true; primary: true; onClicked: root.fill(modelData, false) }
                Button { visible: modelData.auth.length > 0 && modelData.missing.length === 0; text: "Change " + modelData.auth.map(function(a) { return a.label }).join(", "); small: true; flat: true; onClicked: root.fill(modelData, true) }
                Switch { anchors.verticalCenter: parent.verticalCenter; checked: modelData.enabled; onToggled: function(on) { root.take(Door.call("SetEndpointEnabled", { install: modelData.install_id, enabled: on })) } }
                Button { text: "Remove"; icon: "trash-2"; flat: true; small: true; onClicked: root.later("RemoveEndpoint", { install: modelData.install_id }, root.take) }
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
            helper: modelData.auth.length > 0 ? "Asks for your " + modelData.auth.map(function(a) { return a.label }).join(", ") + "." : ""
            Button { text: "Install"; icon: "download"; small: true; onClicked: root.install(modelData) }
        }
    }

    // By hand.
    SettingRow {
        label: "Add one by hand"
        helper: "For a site no repo carries: any feed with {query} where the search goes, a Jackett or Prowlarr feed among them. A key or token in the address moves to the secret store."
        hairline: false
        Column {
            spacing: theme.space(1)
            Row {
                spacing: theme.space(1.5)
                Field { objectName: "custom-name"; text: root.customName; width: theme.space(20); placeholder: "Name"; onEdited: function(t) { root.customName = t } }
                Field { objectName: "custom-url"; text: root.customUrl; mono: true; width: theme.space(40); placeholder: "http://...{query}..."; onEdited: function(t) { root.customUrl = t } }
                Button { text: "Add"; icon: "plus"; enabled: root.customName.trim() !== "" && root.customUrl.trim() !== ""; onClicked: root.addCustom() }
            }
            Note { visible: root.customError !== ""; width: theme.space(60); text: root.customError; color: theme.red }
        }
    }
    foot: Row {
        spacing: theme.space(1.5)
        Button { text: root.busy ? "Refreshing" : "Refresh repos"; icon: "refresh-cw"; flat: true; small: true; busy: root.busy; enabled: root.repos.length > 0 && !root.busy; onClicked: root.refresh() }
    }
}
