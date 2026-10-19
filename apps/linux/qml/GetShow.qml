// The show view on the Downloads page: how a show, or a feed for it, gets added. Opening a
// show never downloads anything. The band says what the show is and what the library has;
// "Where to get it" lists the providers (anirss's endpoints), each saying whether it can
// answer now without asking it anything; clicking one fetches what it has for this show.
// Its releases open as a grid: one row per release group and quality, one cube per episode,
// under the library's own row so the gaps line up. Clicking a group picks its releases for
// every episode the library lacks; clicking a cube picks or drops one. The plan at the
// right says what Get will do, and whether new episodes will follow.
//
// The page owns the searches and the actions (DownloadsPage.getShow*); this view draws them
// and reports clicks.
import QtQuick
import com.marcusrosado.AniBeam

Item {
    id: root
    // { title, query, poster, cubes, total, seriesId, status, format, year, nextAiring, airing }
    property var show: null
    property var endpoints: []
    // name -> { state: "idle" | "searching" | "done" | "error", items, facts, error, label, elapsed }
    property var results: ({})
    property string active: ""
    property var sourceOn: function(name) { return true }
    property bool acting: false
    property string actionLine: ""
    property string actionError: ""
    property real nowMs: Date.now()
    signal back()
    signal fetch(string name)
    signal sourceToggled(string name, bool on)
    signal checkSources()
    signal openList(string name)
    signal getAndFollow(var plan)

    implicitHeight: body.implicitHeight

    readonly property real planWidth: theme.space(88)
    readonly property real colGap: theme.space(6)
    readonly property bool wide: width > theme.space(220)
    readonly property bool inLibrary: !!show && show.seriesId >= 0

    // ---- The library's row and the episode count the grid runs to
    readonly property var libCodes: show && show.cubes ? String(show.cubes).split("") : []
    readonly property var current: active !== "" && results[active] ? results[active] : null
    readonly property int total: {
        var t = show && show.total > 0 ? show.total : libCodes.length
        if (current && current.items) for (var i = 0; i < current.facts.length; i++) {
            var f = current.facts[i]
            if (f && !f.batch && f.episode !== null && f.episode !== undefined && f.episode === Math.floor(f.episode)) t = Math.max(t, f.episode)
        }
        return Math.min(t, 400)
    }
    readonly property var library: {
        var out = []
        for (var n = 1; n <= total; n++) out.push(n <= libCodes.length ? libCodes[n - 1] : "g")
        return out
    }
    function have(n) { var c = library[n - 1]; return c === "d" || c === "l" || c === "w" }

    // ---- The active provider's releases as rows: group and quality, best-seeded per episode
    function seedsOf(item) { return item && item.seeders !== null && item.seeders !== undefined ? item.seeders : -1 }
    readonly property var rows: {
        if (!current || !current.items) return []
        var by = {}, order = []
        for (var i = 0; i < current.items.length; i++) {
            var f = current.facts[i] || {}, it = current.items[i]
            var gname = f.group || root.active
            var key = gname + "|" + (f.resolution || "")
            var r = by[key]
            if (!r) {
                r = by[key] = { key: key, group: gname, resolution: f.resolution || "", eps: {}, batches: [], count: 0, seeds: 0, size: 0 }
                order.push(r)
            }
            if (f.batch || f.episode === null || f.episode === undefined || f.episode !== Math.floor(f.episode)) { r.batches.push(i); continue }
            var n = f.episode, prev = r.eps[n]
            if (prev === undefined || seedsOf(it) > seedsOf(current.items[prev])) r.eps[n] = i
        }
        for (var k = 0; k < order.length; k++) {
            var row = order[k]
            for (var e in row.eps) { row.count++; row.seeds += Math.max(0, seedsOf(current.items[row.eps[e]])); var fb = current.facts[row.eps[e]]; row.size += fb && fb.size_bytes ? fb.size_bytes : 0 }
        }
        order.sort(function(a, b) { return b.count - a.count || b.seeds - a.seeds })
        return order
    }
    readonly property var batchRows: rows.filter(function(r) { return r.batches.length > 0 })

    // ---- The picks: episode number -> item index, and the row the plan follows
    property var picks: ({})
    property string chosen: ""
    property bool follow: !!show && !!show.airing
    onActiveChanged: { picks = ({}); chosen = "" }
    onShowChanged: { picks = ({}); chosen = ""; follow = !!show && !!show.airing }
    // When a provider's releases arrive, pick the group that covers the most gaps
    onRowsChanged: if (chosen === "" && rows.length) pickRow(bestRow())
    function wanted(row) {
        var out = []
        for (var n = 1; n <= total; n++) if (!have(n) && row.eps[n] !== undefined) out.push(n)
        return out
    }
    function bestRow() {
        var best = rows[0], score = -1
        for (var i = 0; i < rows.length; i++) { var s = wanted(rows[i]).length * 1000 + rows[i].count; if (s > score) { score = s; best = rows[i] } }
        return best
    }
    function pickRow(row) {
        if (!row) return
        var p = {}, w = wanted(row)
        for (var i = 0; i < w.length; i++) p[w[i]] = row.eps[w[i]]
        picks = p
        chosen = row.key
    }
    function toggleCube(row, n) {
        if (row.eps[n] === undefined || have(n)) return
        var p = JSON.parse(JSON.stringify(picks))
        if (p[n] === row.eps[n]) delete p[n]; else p[n] = row.eps[n]
        picks = p
        chosen = row.key
    }
    function codesOf(row) {
        var out = []
        for (var n = 1; n <= total; n++) {
            if (row.eps[n] === undefined) out.push("n")
            else if (have(n)) out.push("o")
            else if (picks[n] === row.eps[n]) out.push("p")
            else out.push("a")
        }
        return out
    }
    readonly property var pickList: {
        var out = []
        for (var n in picks) out.push({ n: Number(n), i: picks[n] })
        out.sort(function(a, b) { return a.n - b.n })
        return out
    }
    readonly property real pickBytes: {
        var s = 0
        for (var k = 0; k < pickList.length; k++) { var f = current && current.facts[pickList[k].i]; s += f && f.size_bytes ? f.size_bytes : 0 }
        return s
    }
    readonly property var chosenRow: { for (var i = 0; i < rows.length; i++) if (rows[i].key === chosen) return rows[i]; return null }
    readonly property var pickGroups: {
        var seen = {}, out = []
        for (var k = 0; k < pickList.length; k++) { var f = current && current.facts[pickList[k].i]; var g = f && f.group ? f.group : root.active; if (!seen[g]) { seen[g] = true; out.push(g) } }
        return out
    }
    function episodesPhrase(list) {
        var ns = list.map(function(p) { return p.n })
        if (!ns.length) return ""
        // Runs: 1 to 4, 7, 9 to 12
        var parts = [], a = ns[0], b = ns[0]
        for (var i = 1; i <= ns.length; i++) {
            if (i < ns.length && ns[i] === b + 1) { b = ns[i]; continue }
            parts.push(a === b ? String(a) : a + " to " + b)
            if (i < ns.length) { a = ns[i]; b = ns[i] }
        }
        return (ns.length === 1 ? "Episode " : "Episodes ") + parts.join(", ")
    }
    function plan() {
        var items = pickList.map(function(p) { return current.items[p.i] })
        var row = chosenRow
        return { endpoint: active, query: current ? current.query : "", items: items, follow: follow && !!row,
                 followQuery: row ? (current.query + " " + row.group + (row.resolution ? " " + row.resolution : "")).trim() : "",
                 followItems: row ? Object.keys(row.eps).map(function(n) { return current.items[row.eps[n]] }) : [],
                 name: show ? show.folderName || show.title : "" }
    }
    readonly property string goText: {
        var n = pickList.length
        if (acting) return "Working"
        if (n && follow && chosenRow) return "Get " + Fmt.plural(n, "episode", "episodes") + " and follow"
        if (n) return "Get " + Fmt.plural(n, "episode", "episodes")
        if (follow && chosenRow) return "Follow new episodes"
        return "Pick an episode"
    }

    // ---- Provider words
    function kindWords(e) {
        if (e.on_demand) return "Jackett, started on demand"
        if (e.kind === "nyaa") return "Anime torrents"
        if (e.kind === "rss") return "RSS feed"
        return e.kind
    }
    function readyWords(e) {
        if (!e.on_demand) return "Ready"
        return e.running === true ? "Running" : "Not running"
    }
    function resultWords(e) {
        if (!root.sourceOn(e.name)) return e.on_demand && e.running !== true ? "Off. AniBeam never starts " + e.name + "; start it yourself, then check again." : "Switched off"
        var r = results[e.name]
        if (!r || r.state === "idle") return "Click to see what it has for this show"
        if (r.state === "searching") return (r.label || "Searching") + ", " + Math.floor(r.elapsed || 0) + " s"
        if (r.state === "error") return r.error
        var cover = 0
        for (var n = 1; n <= total; n++) {
            var any = false
            for (var i = 0; i < r.items.length; i++) { var f = r.facts[i]; if (f && f.episode === n && !f.batch) { any = true; break } }
            if (any) cover++
        }
        return Fmt.plural(r.items.length, "release", "releases") + (total ? ", " + cover + " of " + total + " episodes covered" : "")
    }

    Column {
        id: body
        width: parent.width
        spacing: theme.space(6)

        // ---- The band: poster, title, what the library has
        Item {
            width: parent.width
            height: Math.max(poster.height, bandText.implicitHeight) + theme.space(2)
            Corner {
                id: poster
                width: theme.space(34); height: width * 1.5
                radius: theme.radiusMd
                smoothing: theme.cornerSmoothing
                color: theme.tileSunken
                Image {
                    id: posterImage
                    visible: false
                    anchors.fill: parent
                    source: !root.show || !root.show.poster ? "" : String(root.show.poster).indexOf("://") > 0 ? root.show.poster : "file://" + root.show.poster
                    sourceSize: theme.posterDecodeSize
                    fillMode: Image.PreserveAspectCrop
                    asynchronous: true
                }
                fillItem: posterImage.status === Image.Ready ? posterImage : null
            }
            Column {
                id: bandText
                anchors.left: poster.right
                anchors.leftMargin: theme.space(7)
                anchors.right: parent.right
                anchors.bottom: poster.bottom
                spacing: theme.space(3)
                Button { text: "Downloads"; icon: "arrow-left"; flat: true; small: true; onClicked: root.back() }
                Label {
                    width: parent.width
                    text: root.show ? root.show.title : ""
                    role: "display"
                    font.pointSize: theme.typePage
                    wrapMode: Text.Wrap
                    maximumLineCount: 2
                    elide: Text.ElideRight
                    lineHeight: 0.9
                }
                Flow {
                    width: parent.width
                    spacing: theme.space(5)
                    Repeater {
                        model: {
                            var s = root.show, out = []
                            if (!s) return out
                            if (s.format) out.push(s.format)
                            if (s.year) out.push(String(s.year))
                            if (s.airing) out.push(s.nextAiring ? "Episode " + s.nextAiring.episode + " in " + Fmt.countdown(s.nextAiring.at - root.nowMs / 1000) : "Airing")
                            else if (s.status === "Finished") out.push("Finished")
                            if (s.total > 0) out.push(s.total + " episodes")
                            out.push(root.inLibrary ? "In your library" : "Not in your library")
                            return out
                        }
                        Label { required property string modelData; text: modelData; role: "ui"; font.pointSize: theme.typeNormal * 1.1; color: theme.textDim }
                    }
                }
                EpisodeCubes {
                    width: Math.min(parent.width, theme.space(160))
                    height: implicitHeight
                    visible: root.library.length > 0
                    codes: root.library
                    cubeSize: theme.space(3)
                    maxRows: 3
                }
            }
        }

        // ---- Where to get it, and the plan beside it
        Item {
            width: parent.width
            height: Math.max(left.implicitHeight, root.wide ? planTile.height : 0)
            Column {
                id: left
                width: root.wide ? parent.width - root.planWidth - root.colGap : parent.width
                spacing: theme.space(5)

                Item {
                    width: parent.width
                    height: theme.space(12)
                    Label { id: whereTitle; anchors.left: parent.left; anchors.bottom: parent.bottom; anchors.bottomMargin: theme.space(2); text: "Where to get it"; role: "display"; font.pointSize: theme.typeHeading; font.variableAxes: ({ "wdth": 70, "wght": 420 }) }
                    Label { anchors.left: whereTitle.right; anchors.leftMargin: theme.space(3); anchors.baseline: whereTitle.baseline; text: "Pick a provider to see its releases. Nothing downloads until you say so."; role: "ui"; color: theme.textDim; elide: Text.ElideRight; width: parent.width - whereTitle.width - checkButton.width - theme.space(8) }
                    Button { id: checkButton; anchors.right: parent.right; anchors.bottom: parent.bottom; anchors.bottomMargin: theme.space(1); text: "Check again"; icon: "refresh-cw"; flat: true; small: true; onClicked: root.checkSources(); Tooltip { text: "Asks each on-demand provider whether it is running. Starts nothing." } }
                }

                Tile {
                    width: parent.width
                    implicitHeight: providerList.implicitHeight + padding * 2
                    height: implicitHeight
                    padding: theme.space(2)
                    lit: false
                    Column {
                        id: providerList
                        width: parent.width
                        Repeater {
                            model: root.endpoints
                            Item {
                                id: prow
                                required property var modelData
                                required property int index
                                readonly property bool on: root.sourceOn(modelData.name)
                                readonly property bool isActive: root.active === modelData.name
                                readonly property var res: root.results[modelData.name] || null
                                objectName: "provider-" + modelData.name
                                width: parent.width
                                height: theme.space(19)
                                Rectangle {
                                    anchors.fill: parent
                                    radius: theme.radiusMd
                                    color: prow.isActive ? theme.accentSoft : rowMouse.containsMouse && prow.on ? theme.glassFloat : "transparent"
                                }
                                MouseArea {
                                    id: rowMouse
                                    anchors.fill: parent
                                    hoverEnabled: true
                                    enabled: prow.on
                                    cursorShape: prow.on ? Qt.PointingHandCursor : Qt.ArrowCursor
                                    onClicked: root.fetch(prow.modelData.name)
                                }
                                // Ready, running or not: a dot in its colour
                                Rectangle {
                                    id: readyDot
                                    x: theme.space(4)
                                    anchors.verticalCenter: parent.verticalCenter
                                    width: theme.space(2.5); height: width; radius: width / 2
                                    color: !prow.modelData.on_demand || prow.modelData.running === true ? theme.green : theme.textFaint
                                }
                                Column {
                                    anchors.left: readyDot.right
                                    anchors.leftMargin: theme.space(4)
                                    anchors.right: stateCol.left
                                    anchors.rightMargin: theme.space(4)
                                    anchors.verticalCenter: parent.verticalCenter
                                    spacing: theme.space(1)
                                    opacity: prow.on ? 1 : 0.6
                                    Row {
                                        spacing: theme.space(3)
                                        Label { id: pname; text: prow.modelData.name; role: "title"; font.pointSize: theme.typeLarge }
                                        Label { anchors.baseline: pname.baseline; text: root.kindWords(prow.modelData); role: "ui"; color: theme.textDim }
                                    }
                                    Label {
                                        width: parent.width
                                        text: root.resultWords(prow.modelData)
                                        role: "ui"
                                        color: prow.res && prow.res.state === "error" ? theme.red : prow.res && prow.res.state === "searching" ? theme.text : theme.textDim
                                        elide: Text.ElideRight
                                    }
                                }
                                Row {
                                    id: stateCol
                                    anchors.right: parent.right
                                    anchors.rightMargin: theme.space(4)
                                    anchors.verticalCenter: parent.verticalCenter
                                    spacing: theme.space(4)
                                    Label { anchors.verticalCenter: parent.verticalCenter; text: root.readyWords(prow.modelData); role: "ui"; color: !prow.modelData.on_demand || prow.modelData.running === true ? theme.text : theme.textFaint }
                                    Switch {
                                        objectName: "provider-switch-" + prow.modelData.name
                                        anchors.verticalCenter: parent.verticalCenter
                                        checked: prow.on
                                        onToggled: function(on) { root.sourceToggled(prow.modelData.name, on) }
                                    }
                                    Icon { anchors.verticalCenter: parent.verticalCenter; glyph: "chevron-right"; size: theme.space(5); color: prow.on ? theme.textDim : "transparent" }
                                }
                                Rectangle { visible: prow.index < root.endpoints.length - 1; anchors.bottom: parent.bottom; x: theme.space(4); width: parent.width - theme.space(8); height: 1; color: theme.line }
                            }
                        }
                    }
                }

                // ---- The active provider's releases, as a grid of cubes
                Column {
                    id: grid
                    visible: !!root.current && root.current.state === "done"
                    width: parent.width
                    spacing: theme.space(2)
                    readonly property real whoWidth: theme.space(62)
                    readonly property real cubeWidth: width - whoWidth - theme.space(4)
                    Item {
                        width: parent.width
                        height: theme.space(14)
                        Label { id: gridTitle; anchors.left: parent.left; anchors.bottom: parent.bottom; anchors.bottomMargin: theme.space(2); text: root.active; role: "display"; font.pointSize: theme.typeHeading; font.variableAxes: ({ "wdth": 70, "wght": 420 }) }
                        Label {
                            anchors.left: gridTitle.right; anchors.leftMargin: theme.space(3); anchors.baseline: gridTitle.baseline
                            width: parent.width - gridTitle.width - listButton.width - theme.space(8)
                            text: root.rows.length ? "Click a group to take its releases for what you are missing, or click single episodes." : "Nothing here names an episode. The list may still have batches."
                            role: "ui"; color: theme.textDim; elide: Text.ElideRight
                        }
                        Button { id: listButton; anchors.right: parent.right; anchors.bottom: parent.bottom; anchors.bottomMargin: theme.space(1); text: "Every release as a list"; icon: "list"; flat: true; small: true; onClicked: root.openList(root.active) }
                    }
                    // The library's own row first, so the gaps line up
                    GridRow {
                        title: "Your library"
                        line: root.inLibrary ? root.library.filter(function(c) { return c === "d" || c === "l" || c === "w" }).length + " of " + root.total + " on disk" : "Nothing yet"
                        codes: root.library
                        lit: true
                    }
                    Repeater {
                        model: root.rows.filter(function(r) { return r.count > 0 })
                        GridRow {
                            required property var modelData
                            title: modelData.group
                            line: (modelData.resolution ? modelData.resolution + ", " : "") + Fmt.plural(modelData.count, "episode", "episodes") + (modelData.size > 0 ? ", " + Fmt.bytes(modelData.size) : "")
                            codes: root.codesOf(modelData)
                            chosen: root.chosen === modelData.key
                            clickable: true
                            rowData: modelData
                            onPicked: root.pickRow(modelData)
                            onCubePicked: function(i) { root.toggleCube(modelData, i + 1) }
                        }
                    }
                    // Batches: one long pill each, never picked by a click on a group
                    Repeater {
                        model: root.batchRows
                        Item {
                            required property var modelData
                            width: grid.width
                            height: theme.space(11)
                            Label { x: theme.space(4); width: grid.whoWidth - theme.space(4); anchors.verticalCenter: parent.verticalCenter; text: modelData.group; role: "ui"; elide: Text.ElideRight }
                            Corner {
                                x: grid.whoWidth + theme.space(4)
                                width: grid.cubeWidth; height: theme.space(8)
                                anchors.verticalCenter: parent.verticalCenter
                                radius: height / 2
                                color: theme.glassFloat
                                Label {
                                    anchors.verticalCenter: parent.verticalCenter
                                    x: theme.space(4); width: parent.width - theme.space(8)
                                    text: Fmt.plural(modelData.batches.length, "batch", "batches") + ": " + root.current.items[modelData.batches[0]].title
                                    role: "ui"; font.pointSize: theme.typeSmall; color: theme.textDim; elide: Text.ElideRight
                                }
                            }
                        }
                    }
                }
                // What the colours mean, read from the same table the cubes are
                Flow {
                    visible: grid.visible
                    width: parent.width
                    spacing: theme.space(5)
                    Repeater {
                        model: ["w", "l", "d", "g", "m", "a", "p", "o"]
                        Row {
                            required property string modelData
                            spacing: theme.space(2)
                            Rectangle {
                                anchors.verticalCenter: parent.verticalCenter
                                width: theme.space(3); height: width; radius: width * 0.3
                                color: theme.cubeState(parent.modelData).fill
                                border.width: theme.cubeState(parent.modelData).edge === "transparent" ? 0 : 1
                                border.color: theme.cubeState(parent.modelData).edge
                            }
                            Label { text: theme.cubeState(parent.modelData).label; role: "ui"; font.pointSize: theme.typeSmall; color: theme.textDim }
                        }
                    }
                }
                // Searching or failed: one line where the grid will be
                Label {
                    visible: !!root.current && root.current.state !== "done" && root.current.state !== "idle"
                    width: parent.width
                    text: root.current ? (root.current.state === "error" ? root.current.error : (root.current.label || "Searching " + root.active) + ", " + Math.floor(root.current.elapsed || 0) + " s") : ""
                    role: "ui"
                    font.pointSize: theme.typeLarge
                    color: root.current && root.current.state === "error" ? theme.red : theme.textDim
                    wrapMode: Text.Wrap
                }
            }

            // ---- The plan: what Get will do
            Tile {
                id: planTile
                x: root.wide ? parent.width - root.planWidth : 0
                y: root.wide ? 0 : left.implicitHeight + theme.space(6)
                width: root.wide ? root.planWidth : parent.width
                implicitHeight: planBody.implicitHeight + padding * 2
                height: implicitHeight
                padding: theme.space(6)
                lit: false
                Column {
                    id: planBody
                    width: parent.width
                    spacing: theme.space(4)
                    Label {
                        width: parent.width
                        text: root.pickList.length ? "Get " + Fmt.plural(root.pickList.length, "episode", "episodes") : root.current && root.current.state === "done" ? "Nothing picked" : "Pick a provider"
                        role: "display"; font.pointSize: theme.typeHeading; font.variableAxes: ({ "wdth": 70, "wght": 420 })
                        wrapMode: Text.Wrap
                    }
                    Label {
                        width: parent.width
                        text: {
                            if (root.pickList.length) return root.episodesPhrase(root.pickList) + " from " + root.pickGroups.join(" and ") + " on " + root.active + "."
                            if (root.current && root.current.state === "done") return "Click a group or an episode in the grid."
                            return "Its releases show up here as a grid, an episode to a cube."
                        }
                        role: "ui"; color: theme.textDim; wrapMode: Text.Wrap
                    }
                    Flow {
                        visible: root.pickList.length > 0
                        width: parent.width
                        spacing: theme.space(6)
                        Column { Label { text: Fmt.bytes(root.pickBytes); role: "figure"; font.pointSize: theme.typeLarge }
                            Label { text: "together"; role: "ui"; font.pointSize: theme.typeSmall; color: theme.textFaint } }
                        Column { Label { text: String(root.pickList.length); role: "figure"; font.pointSize: theme.typeLarge }
                            Label { text: root.pickList.length === 1 ? "torrent" : "torrents"; role: "ui"; font.pointSize: theme.typeSmall; color: theme.textFaint } }
                    }
                    Rectangle { width: parent.width; height: 1; color: theme.line }
                    Column {
                        width: parent.width
                        spacing: theme.space(1)
                        Label { text: "Saves to"; role: "ui"; font.pointSize: theme.typeSmall; color: theme.textFaint }
                        Label { width: parent.width; text: root.show ? (root.inLibrary ? "Its library folder, " + (root.show.folderName || root.show.title) : "A new folder, " + root.show.title) : ""; role: "ui"; wrapMode: Text.Wrap }
                    }
                    Rectangle { width: parent.width; height: 1; color: theme.line }
                    Item {
                        width: parent.width
                        height: Math.max(followSwitch.height, followText.implicitHeight)
                        Switch {
                            id: followSwitch
                            objectName: "get-follow"
                            checked: root.follow
                            onToggled: function(on) { root.follow = on }
                        }
                        Column {
                            id: followText
                            anchors.left: followSwitch.right
                            anchors.leftMargin: theme.space(4)
                            anchors.right: parent.right
                            spacing: theme.space(1)
                            Label { text: "Follow new episodes"; role: "title"; font.pointSize: theme.typeNormal }
                            Label {
                                width: parent.width
                                text: !root.follow ? "Off. New episodes show up here as gaps; nothing downloads by itself."
                                    : !root.chosenRow ? "Pick a group first: a feed follows one group and quality."
                                    : "A feed for " + root.chosenRow.group + (root.chosenRow.resolution ? " " + root.chosenRow.resolution : "") + " on " + root.active + ". Each new episode downloads as it appears."
                                role: "ui"; font.pointSize: theme.typeSmall; color: theme.textDim; wrapMode: Text.Wrap
                            }
                        }
                    }
                    Button {
                        objectName: "get-go"
                        width: parent.width
                        text: root.goText
                        icon: root.follow && root.chosenRow ? "rss" : "download"
                        primary: true
                        enabled: !root.acting && (root.pickList.length > 0 || (root.follow && !!root.chosenRow))
                        onClicked: root.getAndFollow(root.plan())
                    }
                    Label { visible: root.actionLine !== ""; width: parent.width; text: root.actionLine; role: "ui"; font.pointSize: theme.typeSmall; color: theme.caughtUp; wrapMode: Text.Wrap }
                    Label { visible: root.actionError !== ""; width: parent.width; text: root.actionError; role: "ui"; font.pointSize: theme.typeSmall; color: theme.red; wrapMode: Text.Wrap }
                    Label {
                        visible: root.actionLine === "" && root.actionError === ""
                        width: parent.width
                        text: "Torrents show up under Activity at once, and in the library when they finish."
                        role: "ui"; font.pointSize: theme.typeSmall; color: theme.textFaint; wrapMode: Text.Wrap
                    }
                }
            }
        }
    }

    // One row of the grid: who, one line, and the cubes; a click on the name picks the row
    component GridRow: Item {
        id: gr
        property string title: ""
        property string line: ""
        property var codes: []
        property bool chosen: false
        property bool lit: false
        property bool clickable: false
        property var rowData: null
        signal picked()
        signal cubePicked(int index)
        width: grid.width
        height: Math.max(theme.space(15), cubes.implicitHeight + theme.space(4))
        Rectangle {
            anchors.fill: parent
            radius: theme.radiusMd
            color: gr.chosen ? theme.accentSoft : gr.lit ? theme.glassFloat : whoMouse.containsMouse ? theme.glassFloat : "transparent"
            border.width: gr.chosen ? 1 : 0
            border.color: theme.accentEdge
        }
        MouseArea {
            id: whoMouse
            width: grid.whoWidth; height: parent.height
            enabled: gr.clickable
            hoverEnabled: enabled
            cursorShape: enabled ? Qt.PointingHandCursor : Qt.ArrowCursor
            onClicked: gr.picked()
        }
        Column {
            x: theme.space(4)
            width: grid.whoWidth - theme.space(6)
            anchors.verticalCenter: parent.verticalCenter
            spacing: theme.space(0.5)
            Label { width: parent.width; text: gr.title; role: "title"; font.pointSize: theme.typeNormal; elide: Text.ElideRight }
            Label { width: parent.width; text: gr.line; role: "ui"; font.pointSize: theme.typeSmall; color: theme.textDim; elide: Text.ElideRight }
        }
        EpisodeCubes {
            id: cubes
            x: grid.whoWidth
            anchors.verticalCenter: parent.verticalCenter
            width: grid.cubeWidth
            height: implicitHeight
            codes: gr.codes
            cubeSize: theme.space(6)
            minSize: theme.space(2)
            maxRows: 4
            clickable: gr.clickable
            tipFor: gr.rowData ? function(i, code) {
                var idx = gr.rowData.eps[i + 1]
                if (idx === undefined) return "Episode " + (i + 1) + ": no release from " + gr.rowData.group
                var it = root.current.items[idx], f = root.current.facts[idx]
                var bits = [it.title]
                if (it.seeders !== null && it.seeders !== undefined) bits.push(it.seeders + " seeders")
                if (f && f.size_bytes) bits.push(Fmt.bytes(f.size_bytes))
                if (code === "o") bits.push("already on disk")
                return bits.join("\n")
            } : null
            onCubeClicked: function(i) { gr.cubePicked(i) }
        }
    }
}
