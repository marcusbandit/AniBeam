// Downloads: anirss's front, in the library's own column math. The head names the page and
// says how the two halves stand (anirss and the torrent client, each a chip that opens the
// Downloads settings). Under it the search: the field, the endpoint and Search. A search
// up is a query line (Back, the query, the count, what anirss said), the groups it can be
// narrowed by as chips, a Narrow field with Best fit beside it, then the releases at the
// left and the actions pane at the right (a name, Subscribe, Download, Movie, and what the
// last action did). Under everything the two lists the page always shows: the
// subscriptions anirss registered, and the torrent client's transfers, side by side when
// the width holds both.
//
// The search state is the page's, as anirss asks of a client: `{endpoint, query, items}`
// plus the groups and best fit the last step returned, with a history stack for Back that
// the server never sees. Every step is a job; the page keeps the id of the one it waits
// on and ignores a late answer from one it has replaced. The transfers list is the one
// place in the app that polls, every two seconds while this page is up and only then:
// there is no event for a byte arriving.
import QtQuick
import com.marcusrosado.AniBeam

Item {
    id: page
    objectName: "downloads-page"
    property var props: ({})
    property string title: "Downloads"
    property alias scrollY: scroll.contentY
    function escapePressed() { return false }
    function focusSearch() { scroll.contentY = 0; search.focusInput() }

    // ---- The connection
    property var status: null
    readonly property var anirss: status ? status.anirss : null
    readonly property var torrent: status ? status.torrent : null
    readonly property bool anirssReady: !!anirss && anirss.state === "Ready"
    readonly property bool anirssStarting: !!anirss && anirss.state === "Starting"
    readonly property bool torrentOn: !!torrent && torrent.connected
    readonly property var endpoints: status && status.endpoints ? status.endpoints : []
    readonly property string clientName: status && status.settings ? (status.settings.torrent_client === "Qbittorrent" ? "qBittorrent" : String(status.settings.torrent_client)) : "qBittorrent"
    property real connectJob: -1
    property bool connecting: false
    property int endpointIndex: 0

    // ---- The search: the state anirss asks the client to hold, and the trail for Back
    property var state: null
    property var history: []
    property real searchJob: -1
    property bool searching: false
    property string notice: ""
    property string searchError: ""
    property var selected: ({})
    property int hoveredGroup: -1
    property string nameText: ""
    property var preview: null
    property real previewJob: -1
    property real actionJob: -1
    property bool acting: false
    property string actionLine: ""
    property string actionError: ""
    readonly property var items: state && state.items ? state.items : []
    readonly property var groups: state && state.groups ? state.groups : []
    readonly property int selectedCount: { var n = 0; for (var k in selected) if (selected[k]) n++; return n }
    readonly property int shown: Math.min(items.length, 200)

    // ---- The lists
    property var rules: []
    property bool rulesLoaded: false
    property string rulesError: ""
    property var torrents: []
    property var transfer: null
    property string torrentsError: ""
    property int transferFilter: 0
    property var plans: ({})
    property var planJobs: ({})
    property var planning: ({})

    function reload() {
        var r = Door.getDownloads()
        if (r.error) { frame.toast(r.error.message); return }
        page.status = r.reply.status
    }
    function connect() {
        var r = Door.connectDownloads()
        if (r.error) { frame.toast(r.error.message); return }
        page.connectJob = r.reply.job !== undefined ? r.reply.job : -1
        page.connecting = true
    }
    function loadRules() {
        var r = Door.listSubscriptionRules()
        if (r.error) page.rulesError = r.error.message
    }
    function pollTorrents() {
        var r = Door.listTorrents()
        if (r.error) page.torrentsError = r.error.message
    }
    function errorWords(e) {
        var m = String(e && e.message ? e.message : "")
        if (e && e.kind === "Service" && e.code === "qbt_auth_required") return "anirss cannot log in to " + page.clientName + ". Save its password in Settings."
        if (e && e.kind === "Service" && e.code === "unauthorized") return m + ". Check the address and password in Settings."
        return m === "" ? "Something went wrong." : m
    }

    // ---- The search
    function runSearch() {
        var q = search.text.trim()
        if (q === "") { search.focusInput(); return }
        var endpoint = page.endpoints.length > page.endpointIndex ? page.endpoints[page.endpointIndex].name : ""
        var r = Door.searchReleases(q, endpoint)
        if (r.error) { page.searchError = r.error.message; return }
        page.searchJob = r.reply.job
        page.searching = true
        page.searchError = ""
        page.notice = ""
        page.history = []
        page.state = null
        page.selected = {}
        page.preview = null
        page.actionLine = ""
        page.actionError = ""
    }
    function refine(op) {
        if (!page.state || page.searching) return
        var r = Door.refineReleases(page.state.query, page.state.endpoint, page.state.items, op)
        if (r.error) { page.searchError = r.error.message; return }
        page.searchJob = r.reply.job
        page.searching = true
        page.searchError = ""
    }
    function narrow(text) {
        var t = text.trim()
        if (t === "") return
        if (t.charAt(0) === "-" && t.length > 1) refine({ type: "exclude", term: t.slice(1).trim() })
        else refine({ type: "term", term: t })
    }
    function adopt(s) {
        page.state = { query: s.query, endpoint: s.endpoint, items: s.items || [], groups: s.groups || [], bestFit: s.best_fit || { available: false }, clientSide: !!s.client_side }
        page.selected = {}
        page.preview = null
        page.hoveredGroup = -1
    }
    function back() {
        if (!page.history.length) return
        var h = page.history.slice()
        page.state = h.pop()
        page.history = h
        page.selected = {}
        page.preview = null
        page.notice = ""
    }
    function reasonWords(reason) {
        if (reason === "zero_results") return "That would leave nothing, so the list stays as it is."
        if (reason === "no_match") return "No release matches that term."
        if (reason === "empty_term") return "Type a term to narrow by."
        return "That would change nothing."
    }
    function toggle(item) {
        var s = JSON.parse(JSON.stringify(page.selected))
        if (s[item.link]) delete s[item.link]; else s[item.link] = true
        page.selected = s
    }
    function selectedItems() { return page.items.filter(function(i) { return page.selected[i.link] === true }) }
    function act(r) {
        if (r.error) { page.actionError = page.errorWords(r.error); return }
        page.actionJob = r.reply.job
        page.acting = true
        page.actionLine = ""
        page.actionError = ""
    }
    function subscribe() { if (page.state) act(Door.subscribe(page.state.query, page.state.endpoint, page.state.items, page.nameText)) }
    function download(items) { if (items.length) act(Door.downloadReleases(items, page.nameText)) }
    function movie() {
        var one = selectedItems()
        if (one.length !== 1) { page.actionError = "Select exactly one release to download as a movie."; return }
        act(Door.downloadMovie(one[0].link))
    }
    function previewSubscription() {
        if (!page.state) return
        var r = Door.previewSubscription(page.state.query, page.state.endpoint, page.state.items)
        if (r.error) { page.actionError = page.errorWords(r.error); return }
        page.previewJob = r.reply.job
    }

    // ---- The subscriptions
    function askRemove(name) {
        var r = Door.planRemoveSubscription(name, false, false)
        if (r.error) { frame.toast(page.errorWords(r.error)); return }
        var j = JSON.parse(JSON.stringify(page.planJobs)); j[String(r.reply.job)] = name; page.planJobs = j
        var p = JSON.parse(JSON.stringify(page.planning)); p[name] = true; page.planning = p
    }
    function confirmRemove(name) {
        var r = Door.removeSubscription(name, false, false)
        if (r.error) { frame.toast(page.errorWords(r.error)); return }
        page.keepRule(name)
    }
    function keepRule(name) {
        var p = JSON.parse(JSON.stringify(page.plans)); delete p[name]; page.plans = p
    }

    // ---- The transfers
    function control(hash, action, files) {
        var r = Door.controlTorrents([hash], action, files === true)
        if (r.error) frame.toast(page.errorWords(r.error))
    }
    // Active is what is moving or waiting to: not done, not stopped, not broken. Done is
    // complete whatever it is doing now. All is everything, the stopped and the broken last.
    function rank(t) {
        var s = String(t.state || "")
        if (s === "error" || s === "missingFiles") return 3
        if (s.indexOf("paused") === 0 || s.indexOf("stopped") === 0) return (t.progress || 0) >= 1 ? 1 : 2
        return (t.progress || 0) >= 1 ? 1 : 0
    }
    readonly property var activeTorrents: torrents.filter(function(t) { return page.rank(t) === 0 })
    readonly property var doneTorrents: torrents.filter(function(t) { return page.rank(t) === 1 })
    readonly property var shownTorrents: {
        var list = transferFilter === 0 ? activeTorrents : transferFilter === 1 ? doneTorrents : torrents
        return list.slice().sort(function(a, b) {
            var ar = page.rank(a), br = page.rank(b)
            if (ar !== br) return ar - br
            return (b.added_on || 0) - (a.added_on || 0)
        })
    }
    readonly property int shownTorrentCap: 120
    readonly property string transferFact: {
        if (!page.torrentOn) return page.torrent && page.torrent.message ? page.torrent.message : "Not connected."
        if (page.torrentsError) return page.torrentsError
        var parts = []
        if (page.transfer && page.transfer.dl_speed > 0) parts.push(Fmt.bytes(page.transfer.dl_speed) + "/s down")
        if (page.transfer && page.transfer.up_speed > 0) parts.push(Fmt.bytes(page.transfer.up_speed) + "/s up")
        if (!parts.length) parts.push(page.activeTorrents.length ? "Nothing moving right now." : "Nothing downloading.")
        return parts.join(", ")
    }

    // ---- The head's one line
    readonly property string summary: {
        if (!page.status) return "Connecting."
        if (page.connecting && !page.anirssReady) return "Starting anirss."
        var a = page.anirss
        if (a.state === "Missing") return "anirss is not installed. " + a.message
        if (a.state === "Failed") return "anirss is not running: " + a.message
        if (a.state === "Off") return "anirss is not running yet."
        var t = page.torrentOn ? page.clientName + " " + page.torrent.version + " is connected." : page.clientName + " is not connected: " + (page.torrent.message || "no answer") + "."
        return "Searches run through anirss " + a.version + ". " + t
    }

    Connections {
        target: Door
        function onDownloadsStatusChanged(status) {
            page.status = status
            page.connecting = false
            if (page.anirssReady && !page.rulesLoaded) page.loadRules()
        }
        function onReleasesFound(job, search) {
            if (job !== page.searchJob) return
            page.searching = false
            page.adopt(search)
            var lines = []
            if (search.fell_back) {
                var asked = search.tried && search.tried.length ? search.tried[0].endpoint : ""
                lines.push(asked + ": no results, switched to " + search.endpoint + " (" + search.items.length + ")")
            }
            if (search.auto_resolution) lines.push("Narrowed to " + search.auto_resolution + ", the sharpest on offer")
            page.notice = lines.join(". ")
        }
        function onReleasesRefined(job, refinement) {
            if (job !== page.searchJob) return
            page.searching = false
            if (refinement.applied) {
                var h = page.history.slice(); h.push(page.state); page.history = h
                page.adopt(refinement)
                page.notice = ""
            } else {
                page.notice = page.reasonWords(refinement.reason)
            }
        }
        function onSubscriptionPreviewed(job, spec) { if (job === page.previewJob) page.preview = spec }
        function onSubscribed(job, result) {
            if (job !== page.actionJob) return
            page.acting = false
            page.actionLine = "Subscribed as " + result.name + ". New episodes save to " + result.save_path + "."
            page.loadRules()
        }
        function onTorrentsAdded(job, result) {
            if (job !== page.actionJob) return
            page.acting = false
            page.actionLine = "Added " + Fmt.plural(result.count, "torrent", "torrents") + " under " + result.save_path + "."
            page.selected = {}
            page.pollTorrents()
        }
        function onSubscriptionsRead(rules) { page.rules = rules; page.rulesLoaded = true; page.rulesError = "" }
        function onTorrentsRead(torrents, transfer) { page.torrents = torrents; page.transfer = transfer; page.torrentsError = "" }
        function onDownloadsUnavailable(what, error) {
            if (what === "torrents") page.torrentsError = page.errorWords(error)
            else { page.rulesError = page.errorWords(error); page.rulesLoaded = true }
        }
        function onRemovalPlanned(job, plans) {
            var name = page.planJobs[String(job)]
            if (name === undefined) return
            var j = JSON.parse(JSON.stringify(page.planJobs)); delete j[String(job)]; page.planJobs = j
            var p = JSON.parse(JSON.stringify(page.planning)); delete p[name]; page.planning = p
            if (!plans.length) return
            var all = JSON.parse(JSON.stringify(page.plans)); all[name] = plans[0]; page.plans = all
        }
        function onSubscriptionRemoved(name) { page.keepRule(name); page.loadRules() }
        function onTorrentsControlled(hashes, action) { page.pollTorrents() }
        function onJobFailed(job, kind, error) {
            if (job === page.searchJob) { page.searching = false; page.searchError = page.errorWords(error) }
            else if (job === page.actionJob) { page.acting = false; page.actionError = page.errorWords(error) }
            else if (job === page.connectJob) { page.connecting = false; frame.toast(page.errorWords(error)) }
            else if (job === page.previewJob) { page.previewJob = -1 }
            else if (page.planJobs[String(job)] !== undefined) {
                var name = page.planJobs[String(job)]
                var j = JSON.parse(JSON.stringify(page.planJobs)); delete j[String(job)]; page.planJobs = j
                var p = JSON.parse(JSON.stringify(page.planning)); delete p[name]; page.planning = p
                frame.toast(page.errorWords(error))
            } else if (kind === "RemoveSubscription" || kind === "ControlTorrents") frame.toast(page.errorWords(error))
        }
        function onJobCancelled(job, kind) {
            if (job === page.searchJob) page.searching = false
            else if (job === page.actionJob) page.acting = false
            else if (job === page.connectJob) page.connecting = false
        }
    }
    // The one poll in the app: the transfers, while this page is up
    Timer { interval: 2000; running: page.visible && Door.ready && page.torrentOn; repeat: true; onTriggered: page.pollTorrents() }
    Component.onCompleted: {
        reload()
        connect()
        if (page.anirssReady) loadRules()
        pollTorrents()
        if (props.query) { search.text = props.query; Qt.callLater(page.runSearch) }
    }

    // ---- The geometry
    readonly property real gap: theme.space(4)
    readonly property real paneWidth: theme.space(90)
    readonly property bool wide: scroll.innerWidth >= theme.space(220)

    PageScroll {
        id: scroll
        anchors.fill: parent
        readonly property real innerWidth: width - theme.space(16)

        // ---- The head: the title, the one line, the two halves as chips
        Item {
            width: parent.width
            height: theme.space(14)
            Label {
                id: titleLabel
                anchors.left: parent.left
                anchors.verticalCenter: parent.verticalCenter
                text: "Downloads"
                role: "display"
            }
            Label {
                objectName: "downloads-summary"
                anchors.left: titleLabel.right
                anchors.leftMargin: theme.space(3)
                anchors.right: chips.left
                anchors.rightMargin: theme.space(4)
                anchors.baseline: titleLabel.baseline
                text: page.summary
                role: "ui"
                color: theme.textDim
                elide: Text.ElideRight
            }
            Row {
                id: chips
                anchors.right: parent.right
                anchors.verticalCenter: parent.verticalCenter
                spacing: theme.space(2)
                Chip {
                    objectName: "downloads-anirss-chip"
                    icon: "rss"
                    text: page.anirssReady ? "anirss " + page.anirss.version : page.connecting ? "anirss starting" : "anirss off"
                    textColor: page.anirssReady ? theme.caughtUp : page.connecting ? theme.textDim : theme.red
                    clickable: true
                    onClicked: frame.go("settings", { tab: "downloads" })
                    Tooltip { text: page.anirssReady ? "anirss at " + page.anirss.url : (page.anirss && page.anirss.message ? page.anirss.message : "Open the Downloads settings") }
                }
                Chip {
                    objectName: "downloads-client-chip"
                    icon: "download"
                    text: page.torrentOn ? page.clientName + " " + page.torrent.version : page.clientName + " off"
                    textColor: page.torrentOn ? theme.caughtUp : theme.red
                    clickable: true
                    onClicked: frame.go("settings", { tab: "downloads" })
                    Tooltip { text: page.torrentOn ? (page.status.settings.torrent_url) : (page.torrent && page.torrent.message ? page.torrent.message : "Open the Downloads settings") }
                }
                Button {
                    text: page.connecting ? "Connecting" : "Reconnect"
                    icon: "refresh-cw"
                    flat: true
                    enabled: !page.connecting
                    onClicked: page.connect()
                }
            }
        }

        // ---- The search row: the field, the endpoint, Search
        Row {
            width: parent.width
            spacing: theme.space(2)
            SearchField {
                id: search
                objectName: "downloads-search"
                width: parent.width - endpointPick.width - go.width - parent.spacing * 2
                placeholder: "Search the release feeds"
                hint: "/ then Return"
                onAccepted: page.runSearch()
                onCleared: {}
            }
            Dropdown {
                id: endpointPick
                objectName: "downloads-endpoint"
                options: page.endpoints.length ? page.endpoints.map(function(e) { return e.name }) : ["(no endpoints)"]
                index: page.endpointIndex
                minWidth: theme.space(28)
                interactive: page.endpoints.length > 0
                onPicked: function(i) { page.endpointIndex = i; if (page.state) page.runSearch() }
                Tooltip { text: page.endpoints.length > page.endpointIndex ? page.endpoints[page.endpointIndex].url : "The search endpoints come from anirss's config" }
            }
            Button {
                id: go
                objectName: "downloads-go"
                text: page.searching ? "Searching" : "Search"
                icon: "search"
                primary: true
                enabled: !page.searching
                onClicked: page.runSearch()
            }
        }

        // ---- The search up: the query line, the groups, Narrow and Best fit, then the
        // releases and the actions pane
        Column {
            visible: page.state !== null || page.searching || page.searchError !== ""
            width: parent.width
            spacing: theme.space(3)
            Row {
                width: parent.width
                height: theme.controlHeight
                spacing: theme.space(3)
                Button { text: "Back"; icon: "arrow-left"; flat: true; small: true; enabled: page.history.length > 0; anchors.verticalCenter: parent.verticalCenter; onClicked: page.back() }
                Label { visible: !!page.state; text: page.state ? page.state.query : ""; role: "title"; font.pointSize: theme.typeNormal; anchors.verticalCenter: parent.verticalCenter; elide: Text.ElideMiddle; width: Math.min(implicitWidth, parent.width * 0.5) }
                Label { visible: !!page.state; text: page.state ? Fmt.plural(page.items.length, "release", "releases") + " on " + page.state.endpoint : ""; role: "ledger"; color: theme.textDim; anchors.verticalCenter: parent.verticalCenter }
                Label { visible: page.searching; text: page.state ? "Narrowing" : "Searching"; role: "ui"; color: theme.textDim; anchors.verticalCenter: parent.verticalCenter }
                Label { visible: page.notice !== "" && !page.searching; text: page.notice; role: "ui"; color: theme.textDim; anchors.verticalCenter: parent.verticalCenter; elide: Text.ElideRight }
                Label { visible: page.searchError !== ""; text: page.searchError; role: "ui"; color: theme.red; anchors.verticalCenter: parent.verticalCenter; elide: Text.ElideRight }
            }
            // The groups: a token or a few that split the list, a count on each
            Flow {
                visible: page.groups.length > 0
                width: parent.width
                spacing: theme.space(1.5)
                Repeater {
                    model: page.groups
                    Chip {
                        required property var modelData
                        required property int index
                        text: modelData.label + "  " + modelData.member_count
                        small: true
                        clickable: !page.searching
                        selected: page.hoveredGroup === index
                        onClicked: page.refine({ type: "tokens", tokens: modelData.tokens })
                        HoverHandler { onHoveredChanged: page.hoveredGroup = hovered ? index : (page.hoveredGroup === index ? -1 : page.hoveredGroup) }
                    }
                }
            }
            Row {
                visible: !!page.state
                width: parent.width
                spacing: theme.space(2)
                Field {
                    id: narrow
                    objectName: "downloads-narrow"
                    width: Math.min(theme.space(90), parent.width - bestFit.width - parent.spacing)
                    placeholder: "Narrow by a term, or -term to drop"
                    onEdited: function(t) { if (t.trim() !== "") { page.narrow(t); text = "" } }
                }
                Button {
                    id: bestFit
                    objectName: "downloads-best-fit"
                    text: "Best fit"
                    icon: "sparkles"
                    flat: true
                    enabled: !!page.state && page.state.bestFit.available && !page.searching
                    onClicked: page.refine({ type: "best_fit" })
                    Tooltip { text: page.state && page.state.bestFit.available ? "Pins the best release's profile: " + page.state.bestFit.query : "Best fit has nothing to pin here" }
                }
            }
            // The releases at the left, the actions at the right
            Item {
                width: parent.width
                height: Math.max(list.implicitHeight, pane.implicitHeight)
                Column {
                    id: list
                    x: 0
                    width: parent.width - page.paneWidth - page.gap
                    Repeater {
                        model: page.items.slice(0, page.shown)
                        ReleaseRow {
                            required property var modelData
                            required property int index
                            release: modelData
                            selected: page.selected[modelData.link] === true
                            inGroup: page.hoveredGroup >= 0 && page.groups[page.hoveredGroup] && page.groups[page.hoveredGroup].members.indexOf(index) >= 0
                            onToggled: page.toggle(modelData)
                            onDownloadOne: page.download([modelData])
                        }
                    }
                    Label {
                        visible: page.items.length > page.shown
                        topPadding: theme.space(2)
                        text: "and " + (page.items.length - page.shown) + " more. Narrow the list to see them."
                        role: "ui"
                        color: theme.textDim
                    }
                    Item {
                        visible: page.state && page.items.length === 0 && !page.searching
                        width: parent.width
                        height: visible ? theme.space(40) : 0
                        EmptyState { icon: "search"; title: "No releases"; body: "Nothing on " + (page.state ? page.state.endpoint : "") + " matches. Try another endpoint or a shorter query." }
                    }
                }
                Tile {
                    id: pane
                    x: parent.width - page.paneWidth
                    width: page.paneWidth
                    implicitHeight: paneBody.implicitHeight + padding * 2
                    height: implicitHeight
                    padding: theme.space(4)
                    lit: false
                    Column {
                        id: paneBody
                        width: parent.width
                        spacing: theme.space(2.5)
                        Label { text: "Name"; role: "ledger"; color: theme.textDim }
                        Field {
                            objectName: "downloads-name"
                            width: parent.width
                            placeholder: "anirss picks one from the first release"
                            text: page.nameText
                            onEdited: function(t) { page.nameText = t }
                        }
                        Label { width: parent.width; text: "The folder under anirss's save base. A subscription keeps this name as its rule."; role: "ui"; font.pointSize: theme.typeSmall; color: theme.textFaint; wrapMode: Text.Wrap }
                        Item { width: 1; height: theme.space(1) }
                        Button {
                            objectName: "downloads-subscribe"
                            width: parent.width
                            text: page.acting ? "Working" : "Subscribe to this search"
                            icon: "rss"
                            primary: true
                            enabled: !!page.state && page.items.length > 0 && !page.acting && !page.searching
                            onClicked: page.subscribe()
                        }
                        Button {
                            width: parent.width
                            text: page.preview ? "Preview again" : "Preview the rule"
                            icon: "eye"
                            flat: true
                            small: true
                            enabled: !!page.state && page.items.length > 0 && !page.searching
                            onClicked: page.previewSubscription()
                        }
                        Column {
                            visible: !!page.preview
                            width: parent.width
                            spacing: theme.space(1)
                            Label { width: parent.width; text: page.preview ? "Feed " + page.preview.feed_url : ""; role: "ledger"; color: theme.textDim; elide: Text.ElideMiddle }
                            Label { visible: !!(page.preview && page.preview.must_contain); width: parent.width; text: page.preview ? "Must contain " + page.preview.must_contain : ""; role: "ledger"; color: theme.textDim; elide: Text.ElideRight }
                            Label { visible: !!(page.preview && page.preview.must_not_contain); width: parent.width; text: page.preview ? "Must not contain " + page.preview.must_not_contain : ""; role: "ledger"; color: theme.textDim; elide: Text.ElideRight }
                        }
                        Rectangle { width: parent.width; height: 1; color: theme.line }
                        Button {
                            objectName: "downloads-download"
                            width: parent.width
                            text: page.selectedCount > 0 ? "Download " + Fmt.plural(page.selectedCount, "selected release", "selected releases") : "Download all " + page.items.length
                            icon: "download"
                            enabled: !!page.state && page.items.length > 0 && !page.acting && !page.searching
                            onClicked: page.download(page.selectedCount > 0 ? page.selectedItems() : page.items)
                        }
                        Button {
                            width: parent.width
                            text: "Download as a movie"
                            icon: "clapperboard"
                            flat: true
                            enabled: page.selectedCount === 1 && !page.acting
                            onClicked: page.movie()
                            Tooltip { text: "One selected release, straight into the movie folder" }
                        }
                        Label { visible: page.actionLine !== ""; width: parent.width; text: page.actionLine; role: "ui"; font.pointSize: theme.typeSmall; color: theme.caughtUp; wrapMode: Text.Wrap }
                        Label { visible: page.actionError !== ""; width: parent.width; text: page.actionError; role: "ui"; font.pointSize: theme.typeSmall; color: theme.red; wrapMode: Text.Wrap }
                    }
                }
            }
        }

        // ---- Nothing to search with: anirss is not here
        Item {
            visible: !!page.status && !page.anirssReady && !page.connecting && page.state === null
            width: parent.width
            height: visible ? theme.space(50) : 0
            EmptyState {
                icon: page.anirss && page.anirss.state === "Missing" ? "circle-alert" : "rss"
                title: page.anirss && page.anirss.state === "Missing" ? "anirss is not installed" : "anirss is not running"
                body: page.anirss && page.anirss.message ? page.anirss.message : "Searches, subscriptions and downloads run through anirss."
                Button { text: "Try again"; icon: "refresh-cw"; onClicked: page.connect() }
                Button { text: "Settings"; icon: "settings"; flat: true; onClicked: frame.go("settings", { tab: "downloads" }) }
            }
        }

        // ---- The two lists, side by side when the width holds both
        Item {
            width: parent.width
            height: page.wide ? Math.max(subs.implicitHeight, transfers.implicitHeight) : subs.implicitHeight + transfers.implicitHeight + page.gap * 2
            Column {
                id: subs
                x: 0
                y: 0
                width: page.wide ? Math.floor((parent.width - page.gap * 2) * 0.42) : parent.width
                spacing: theme.space(2)
                Item {
                    width: parent.width
                    height: theme.space(12)
                    Row {
                        anchors.left: parent.left
                        anchors.bottom: parent.bottom
                        anchors.bottomMargin: theme.space(2)
                        spacing: theme.space(3)
                        Label { id: subsTitle; text: "Subscriptions"; role: "display"; font.pointSize: theme.typeLarge; font.variableAxes: ({ "wdth": 74, "wght": 500 }) }
                        Label { anchors.baseline: subsTitle.baseline; text: page.rules.length; role: "ledger"; color: theme.textFaint; visible: page.rules.length > 0 }
                        Label { anchors.baseline: subsTitle.baseline; text: page.rulesError !== "" ? page.rulesError : page.rulesLoaded ? (page.rules.length ? "Feeds anirss watches for you." : "None yet.") : (page.anirssReady ? "Reading." : ""); role: "ui"; color: page.rulesError !== "" ? theme.red : theme.textDim }
                    }
                    Button { anchors.right: parent.right; anchors.bottom: parent.bottom; text: "Refresh"; icon: "refresh-cw"; flat: true; small: true; enabled: page.anirssReady; onClicked: page.loadRules() }
                }
                Repeater {
                    model: page.rules
                    SubscriptionRow {
                        required property var modelData
                        rule: modelData
                        plan: page.plans[modelData.name] || null
                        planning: page.planning[modelData.name] === true
                        onAskRemove: page.askRemove(modelData.name)
                        onConfirmRemove: page.confirmRemove(modelData.name)
                        onKeep: page.keepRule(modelData.name)
                    }
                }
                Item {
                    visible: page.rulesLoaded && page.rules.length === 0 && page.rulesError === ""
                    width: parent.width
                    height: visible ? theme.space(30) : 0
                    EmptyState { icon: "rss"; title: "No subscriptions"; body: "Search above and subscribe to keep a show coming." }
                }
            }
            Column {
                id: transfers
                x: page.wide ? subs.width + page.gap * 2 : 0
                y: page.wide ? 0 : subs.height + page.gap * 2
                width: page.wide ? parent.width - subs.width - page.gap * 2 : parent.width
                spacing: theme.space(2)
                Item {
                    width: parent.width
                    height: theme.space(12)
                    Row {
                        anchors.left: parent.left
                        anchors.bottom: parent.bottom
                        anchors.bottomMargin: theme.space(2)
                        spacing: theme.space(3)
                        Label { id: trTitle; text: "Transfers"; role: "display"; font.pointSize: theme.typeLarge; font.variableAxes: ({ "wdth": 74, "wght": 500 }) }
                        Label { anchors.baseline: trTitle.baseline; text: page.shownTorrents.length; role: "ledger"; color: theme.textFaint; visible: page.torrents.length > 0 }
                        Label { anchors.baseline: trTitle.baseline; text: page.transferFact; role: "ui"; color: page.torrentOn && page.torrentsError === "" ? theme.textDim : theme.red; elide: Text.ElideRight; width: Math.min(implicitWidth, transfers.width - trTitle.width - filterSeg.width - theme.space(20)) }
                    }
                    Seg {
                        id: filterSeg
                        objectName: "downloads-transfer-filter"
                        anchors.right: parent.right
                        anchors.bottom: parent.bottom
                        small: true
                        options: ["Active " + page.activeTorrents.length, "Done " + page.doneTorrents.length, "All"]
                        index: page.transferFilter
                        onPicked: function(i) { page.transferFilter = i }
                    }
                }
                Repeater {
                    model: page.shownTorrents.slice(0, page.shownTorrentCap)
                    TorrentRow {
                        required property var modelData
                        torrent: modelData
                        onStop: page.control(modelData.hash, "Stop", false)
                        onStart: page.control(modelData.hash, "Start", false)
                        onRemove: function(files) { page.control(modelData.hash, "Delete", files) }
                    }
                }
                Label {
                    visible: page.shownTorrents.length > page.shownTorrentCap
                    topPadding: theme.space(2)
                    text: "and " + (page.shownTorrents.length - page.shownTorrentCap) + " more in the client."
                    role: "ui"
                    color: theme.textDim
                }
                Item {
                    visible: page.shownTorrents.length === 0
                    width: parent.width
                    height: visible ? theme.space(30) : 0
                    EmptyState {
                        icon: page.torrentOn ? "download" : "circle-alert"
                        title: page.torrentOn ? (page.transferFilter === 0 ? "Nothing downloading" : "Nothing here") : page.clientName + " is not connected"
                        body: page.torrentOn ? "What you download or subscribe to shows up here as it moves." : (page.torrent && page.torrent.message ? page.torrent.message : "Check its address and password in Settings.")
                        Button { visible: !page.torrentOn; text: "Settings"; icon: "settings"; onClicked: frame.go("settings", { tab: "downloads" }) }
                    }
                }
            }
        }
    }
}
