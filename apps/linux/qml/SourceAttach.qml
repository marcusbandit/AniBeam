// Adding a source list in one step (#1003): paste its address, or pick one used before, and
// Add fetches it, checks its signature, pins its key and turns on every source in it that
// asks for nothing (AttachEndpointRepo). What it added shows at once, under the field.
// Onboarding, the Downloads page with no sources and Settings, Downloads all use this one,
// so the step is the same wherever someone meets it. A source that asks for a value (a key,
// a login) stays offered; `showWaiting` names those and offers Settings, where they are
// filled in. The call runs off the Qt thread (Door.callAsync), so a slow repo never freezes
// the window. The addresses used before live in shell.conf, never in the library.
import QtQuick
import QtCore
import com.marcusrosado.Anibeam

Column {
    id: root
    objectName: "source-attach"
    spacing: theme.space(3)

    // The catalog as the host keeps it. A host that keeps none leaves it null, and this reads
    // its own each time it shows and takes the core's reply to Add.
    property var catalog: null
    property var seen: null
    readonly property var known: catalog || seen
    function reread() {
        if (catalog) return
        var r = Door.call("ListRepoEndpoints", {})
        if (r && r.kind === "EndpointCatalog") seen = r.reply.catalog
    }
    // Only once it shows: the Downloads page builds one it keeps hidden while sources exist
    Component.onCompleted: if (visible) reread()
    onVisibleChanged: if (visible) reread()
    // Names the sources that wait for a value and offers Settings for them. Settings lists
    // them itself, so it turns this off.
    property bool showWaiting: true
    // The address the last Add tried, for Try again; the field itself decides whether Add can run
    property string tried: ""
    property bool busy: false
    property string error: ""
    // The core's own words for the error, under the line's Details
    property string errorDetail: ""
    property var errorFix: undefined
    // The address of the list this added last; the result shows for that one
    property string attachedUrl: ""
    signal attached(var catalog)
    // Someone asked to fill in a source's value, which Settings does
    signal settingsAsked()

    Settings {
        id: kept
        location: Shell.settingsUrl
        category: "sourceLists"
        // A JSON array of addresses, newest first
        property string recent: "[]"
    }
    readonly property var recent: {
        try { var r = JSON.parse(kept.recent); return Array.isArray(r) ? r : [] } catch (e) { return [] }
    }
    readonly property var repos: known ? known.repos : []
    // Used before and not added now
    readonly property var picks: recent.filter(function(u) { return !root.repos.some(function(r) { return r.url === u }) })
    function remember(u) {
        kept.recent = JSON.stringify([u].concat(recent.filter(function(x) { return x !== u })).slice(0, 5))
    }

    // The list the result speaks for, and what it holds
    readonly property var result: {
        if (attachedUrl === "") return null
        for (var i = 0; i < repos.length; i++) if (repos[i].url === attachedUrl) return repos[i]
        return null
    }
    // The sources this Add turned on, not every one the list ever had: the install ids from
    // before the call are left out
    property var before: ({})
    readonly property var added: result ? known.installed.filter(function(e) { return e.repo_url === root.attachedUrl && !root.before[e.install_id] }) : []
    // The ones that wait for a value. An NSFW source in a list that is not NSFW, or one whose
    // id is on already, asks for nothing and is left out
    readonly property var waiting: result ? known.offered.filter(function(o) { return o.repo_url === root.attachedUrl && !o.installed && o.auth.length > 0 && (root.result.nsfw || !o.nsfw) }) : []

    property var pending: ({})
    Connections {
        target: Door
        function onCallDone(token, reply) {
            var u = root.pending[token]
            if (u === undefined) return
            delete root.pending[token]
            root.busy = false
            if (reply && reply.kind === "EndpointCatalog") {
                var c = reply.reply.catalog
                root.seen = c
                // The core keeps the address as it normalised it: match the reply's own
                var hit = ""
                for (var i = 0; i < c.repos.length; i++) {
                    var r = c.repos[i].url
                    if (r === u || r.toLowerCase() === u.toLowerCase()) hit = r
                }
                root.attachedUrl = hit
                field.text = ""
                if (hit !== "") root.remember(hit)
                root.attached(c)
            } else if (reply && reply.error) {
                // The bridge's plain words without its next step: that step points here
                root.error = reply.error.plain || frame.errorText(reply.error)
                root.errorDetail = String(reply.error.message || "")
                root.errorFix = reply.error.fix
            }
        }
    }
    function attach(given) {
        var u = String(given === undefined ? field.text : given).trim()
        if (u === "" || busy) return
        root.tried = u
        var had = ({})
        var ins = known ? known.installed : []
        for (var i = 0; i < ins.length; i++) had[ins[i].install_id] = true
        root.before = had
        root.error = ""
        root.errorDetail = ""
        root.busy = true
        var t = Door.callAsync("AttachEndpointRepo", { url: u })
        if (t < 0) {
            root.busy = false
            root.error = words.t("endpoint-repos-panel-could-not-start")
            root.errorFix = "retry"
            return
        }
        root.pending[t] = u
    }
    function focusInput() { field.focusInput() }

    // The step itself: the address, Add, and what a source list is. Where the three leave the
    // field too short to read its placeholder, the link moves under them.
    Flow {
        id: ask
        width: parent.width
        spacing: theme.space(2)
        readonly property real beside: width - add.width - wiki.width - spacing * 2
        Field {
            id: field
            objectName: "source-attach-url"
            width: ask.beside >= theme.space(70) ? Math.min(theme.space(110), ask.beside) : Math.max(theme.space(30), ask.width - add.width - ask.spacing)
            mono: text !== ""
            placeholder: words.t("source-attach-placeholder")
            accessibleName: words.t("source-attach-placeholder")
            Keys.onReturnPressed: root.attach()
            Keys.onEnterPressed: root.attach()
        }
        Button {
            id: add
            objectName: "source-attach-add"
            primary: true
            icon: "plus"
            text: root.busy ? words.t("endpoint-repos-panel-checking") : words.t("endpoint-repos-panel-add")
            busy: root.busy
            enabled: !root.busy && field.text.trim() !== ""
            onClicked: root.attach()
        }
        Button {
            id: wiki
            objectName: "source-attach-wiki"
            msg: "source-attach-what"
            icon: "book-open"
            flat: true
            onClicked: frame.openLink("https://anibeam.app/wiki/sources/") // i18n: none
        }
    }

    // The addresses used before and not added now, one click each
    Flow {
        objectName: "source-attach-recent"
        visible: root.picks.length > 0 && !root.busy
        width: parent.width
        spacing: theme.space(2)
        Label { height: theme.space(7); verticalAlignment: Text.AlignVCenter; msg: "source-attach-recent"; role: "ui"; color: theme.textFaint; rightPadding: theme.space(1) }
        Repeater {
            model: root.picks
            Chip {
                required property var modelData
                required property int index
                objectName: "source-attach-pick-" + index
                text: modelData
                icon: "clock"
                clickable: true
                maxLabelWidth: theme.space(90)
                onClicked: root.attach(modelData)
            }
        }
    }

    ErrorLine {
        objectName: "source-attach-error"
        width: parent.width
        text: root.error
        detail: root.errorDetail
        // Every failure here is tried again from here: the fix a list error names points at
        // this very step, and a host that was down is the case trying again helps
        actionText: words.t("action-try-again")
        action: function() { root.attach(root.tried) }
    }

    // What the Add just did: the list, the sources it turned on, the ones that wait
    Column {
        objectName: "source-attach-result"
        visible: !!root.result && !root.busy && root.error === ""
        width: parent.width
        spacing: theme.space(3)
        Row {
            spacing: theme.space(2)
            Icon { anchors.verticalCenter: parent.verticalCenter; glyph: "circle-check"; color: theme.accent }
            Label {
                anchors.verticalCenter: parent.verticalCenter
                text: root.result ? words.t(root.added.length > 0 ? "source-attach-added" : "source-attach-added-none", { name: root.result.name, count: root.added.length }) : ""
                role: "title"
                font.pointSize: theme.typeNormal
            }
        }
        Flow {
            visible: root.added.length > 0
            width: parent.width
            spacing: theme.space(2)
            Repeater {
                model: root.added
                Chip {
                    required property var modelData
                    objectName: "source-attach-source-" + modelData.id
                    text: modelData.name
                    icon: "check"
                    mono: false
                    selected: true
                    Tooltip { text: modelData.host }
                }
            }
        }
        Row {
            visible: root.showWaiting && root.waiting.length > 0
            spacing: theme.space(3)
            Label {
                anchors.verticalCenter: parent.verticalCenter
                text: words.t("source-attach-waiting", { count: root.waiting.length, names: root.waiting.map(function(o) { return o.name }).join(", ") })
                color: theme.textDim
            }
            Button {
                anchors.verticalCenter: parent.verticalCenter
                objectName: "source-attach-settings"
                msg: "source-attach-fill-in"
                icon: "arrow-right"
                small: true
                onClicked: root.settingsAsked()
            }
        }
    }
}
