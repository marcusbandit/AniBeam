// A search's releases as a table: a header of sortable columns, then a row per release.
// Each row is a tick, the release's name cleaned of its brackets with its facts as chips
// under it (the group, the resolution, the episode or Batch, On disk when the library
// already has that episode), then the numbers in the ledger cut, right-aligned: size,
// seeders (green when healthy, red at none), leechers, completed downloads, and when it
// was posted. A click ticks the row; hovering offers Download for that release alone.
//
// The column heads sort: the first click sorts highest first (newest, for Released), a
// second click turns it round, and Release sorts by name. Until one is clicked the rows
// keep anirss's order. Seeders, leechers and downloads come from anirss when its feed
// carried them, else from the core's own read of the same feed (`extras`, by link), which
// is also where the dates come from; until that read lands the Released column waits.
import QtQuick
import com.marcusrosado.AniBeam

Item {
    id: root
    property var items: []
    property var facts: []
    // link -> FeedEntry, and title -> FeedEntry for a feed whose links differ
    property var extras: ({})
    property var extrasByTitle: ({})
    // "loading", "ready" or "none"
    property string datesState: "loading"
    property var selected: ({})
    // Indexes of the releases in the group under the pointer, drawn lit
    property var highlight: []
    property string sortKey: ""
    property bool sortDesc: true
    property real nowMs: Date.now()
    property int cap: 200
    signal toggled(var item)
    signal toggledAll(bool on)
    signal sortBy(string key)
    signal downloadOne(var item)

    // ---- The values a column sorts and shows
    function extraOf(i) {
        var it = root.items[i]
        return root.extras[it.link] || root.extrasByTitle[it.title] || null
    }
    function count(i, field) {
        var v = root.items[i][field]
        if (v !== null && v !== undefined) return v
        var e = root.extraOf(i)
        return e && e[field] !== null && e[field] !== undefined ? e[field] : null
    }
    function bytes(i) {
        var f = root.facts[i]
        if (f && f.size_bytes) return f.size_bytes
        var e = root.extraOf(i)
        return e && e.size_bytes ? e.size_bytes : null
    }
    function published(i) { var e = root.extraOf(i); return e && e.published ? e.published : null }
    function value(i, key) {
        if (key === "title") return root.clean(root.items[i].title).toLowerCase()
        if (key === "episode") { var f = root.facts[i]; return f && f.episode !== null && f.episode !== undefined ? f.episode : null }
        if (key === "size") return root.bytes(i)
        if (key === "released") return root.published(i)
        return root.count(i, key)
    }
    readonly property var order: {
        var idx = []
        for (var i = 0; i < items.length; i++) idx.push(i)
        if (sortKey === "") return idx
        var key = sortKey, desc = sortDesc
        return idx.sort(function(a, b) {
            var va = root.value(a, key), vb = root.value(b, key)
            // A row with nothing to sort by goes last whichever way
            if (va === null && vb === null) return a - b
            if (va === null) return 1
            if (vb === null) return -1
            var d = typeof va === "string" ? va.localeCompare(vb) : va - vb
            if (d === 0) return a - b
            return desc ? -d : d
        })
    }
    readonly property bool allSelected: {
        if (!items.length) return false
        for (var i = 0; i < items.length; i++) if (selected[items[i].link] !== true) return false
        return true
    }

    // The name a person reads: no leading group, no trailing brackets, no extension, and for
    // a scene name nothing from the resolution on
    function clean(t) {
        var s = String(t || "")
        s = s.replace(/^\s*\[[^\]]*\]\s*/, "").replace(/\.(mkv|mp4|avi|webm)$/i, "")
        var prev
        do { prev = s; s = s.replace(/\s*[\[(][^\[\]()]*[\])]\s*$/, "") } while (s !== prev && s.length)
        var scene = s.replace(/[\s.](2160|1440|1080|720|576|480)[pi]\b.*$/i, "")
        s = scene.trim() !== "" ? scene : s
        s = s.replace(/[._]/g, function(c, off, str) { return /\s/.test(str) ? c : " " })
        return s.trim() !== "" ? s.trim() : String(t || "")
    }

    // ---- The columns: the tick, the name, then the numbers, then the row's own action
    readonly property real wCheck: theme.space(9)
    readonly property real wEp: theme.space(13)
    readonly property real wSize: theme.space(19)
    readonly property real wSeed: theme.space(17)
    readonly property real wLeech: theme.space(17)
    readonly property real wDl: theme.space(19)
    readonly property real wDate: theme.space(22)
    readonly property real wAct: theme.space(28)
    readonly property real wTitle: Math.max(theme.space(40), width - wCheck - wEp - wSize - wSeed - wLeech - wDl - wDate - wAct)
    readonly property var columns: [
        { key: "episode", text: "Ep", w: wEp },
        { key: "size", text: "Size", w: wSize },
        { key: "seeders", text: "Seeders", w: wSeed },
        { key: "leechers", text: "Leechers", w: wLeech },
        { key: "downloads", text: "Downloads", w: wDl },
        { key: "released", text: "Released", w: wDate }
    ]

    implicitHeight: head.height + body.implicitHeight + (more.visible ? more.height : 0)

    // ---- The header
    Item {
        id: head
        width: parent.width
        height: theme.space(10)
        Rectangle { anchors.bottom: parent.bottom; width: parent.width; height: 1; color: theme.lineStrong }
        Check {
            x: (root.wCheck - width) / 2
            anchors.verticalCenter: parent.verticalCenter
            on: root.allSelected
            onClicked: root.toggledAll(!root.allSelected)
            Tooltip { text: root.allSelected ? "Untick every release" : "Tick every release" }
        }
        HeadCell { x: root.wCheck; width: root.wTitle; key: "title"; text: "Release"; alignRight: false }
        Row {
            x: root.wCheck + root.wTitle
            Repeater {
                model: root.columns
                HeadCell { required property var modelData; width: modelData.w; key: modelData.key; text: modelData.text; alignRight: true }
            }
        }
    }

    // ---- The rows
    Column {
        id: body
        y: head.height
        width: parent.width
        Repeater {
            model: root.order.slice(0, root.cap)
            Item {
                id: row
                required property int modelData
                readonly property int i: modelData
                readonly property var item: root.items[i]
                readonly property var fact: root.facts[i] || ({})
                readonly property bool ticked: root.selected[item.link] === true
                readonly property bool lit: root.highlight.indexOf(i) >= 0
                readonly property var seeders: root.count(i, "seeders")
                width: body.width
                height: theme.space(15)
                Rectangle {
                    anchors.fill: parent
                    radius: theme.radiusSm
                    color: row.ticked ? theme.accentSoft : rowHover.hovered ? theme.raised : row.lit ? theme.glassOf(theme.dark ? 0.04 : 0.05) : "transparent"
                }
                HoverHandler { id: rowHover; cursorShape: Qt.PointingHandCursor }
                MouseArea { anchors.fill: parent; onClicked: root.toggled(row.item) }
                Check { x: (root.wCheck - width) / 2; anchors.verticalCenter: parent.verticalCenter; on: row.ticked; onClicked: root.toggled(row.item) }

                Column {
                    x: root.wCheck
                    width: root.wTitle - theme.space(4)
                    anchors.verticalCenter: parent.verticalCenter
                    spacing: theme.space(1)
                    Label {
                        id: nameLabel
                        width: parent.width
                        text: root.clean(row.item.title)
                        role: "ui"
                        color: theme.text
                        elide: Text.ElideRight
                        Tooltip { text: row.item.title }
                    }
                    Row {
                        spacing: theme.space(1.25)
                        Tag { visible: row.fact.on_disk === true; text: "On disk"; dot: theme.accent }
                        Tag { visible: !!row.fact.group; text: row.fact.group || "" }
                        Tag { visible: !!row.fact.resolution; text: row.fact.resolution || "" }
                        Tag { visible: row.fact.batch === true; text: row.fact.season ? "Season " + row.fact.season + " batch" : "Batch"; dot: theme.orange }
                        Tag { visible: !!row.item.infomkv; text: row.item.infomkv || "" }
                        Label {
                            visible: !!row.item.source
                            anchors.verticalCenter: parent.verticalCenter
                            text: "via " + row.item.source
                            role: "ledger"
                            font.pointSize: theme.typeSmall
                            color: theme.textFaint
                        }
                    }
                }
                Row {
                    x: root.wCheck + root.wTitle
                    height: parent.height
                    Cell {
                        width: root.wEp
                        text: row.fact.episode !== null && row.fact.episode !== undefined
                              ? (row.fact.season && row.fact.season > 1 ? "S" + row.fact.season + " " : "") + row.fact.episode : ""
                    }
                    Cell { width: root.wSize; text: { var b = root.bytes(row.i); return b ? Fmt.bytes(b) : (row.item.size || "") } }
                    Cell {
                        width: root.wSeed
                        text: row.seeders === null ? "" : String(row.seeders)
                        ink: row.seeders === null ? theme.textFaint : row.seeders === 0 ? theme.red : row.seeders >= 20 ? theme.green : theme.text
                    }
                    Cell { width: root.wLeech; ink: theme.textDim; text: { var v = root.count(row.i, "leechers"); return v === null ? "" : String(v) } }
                    Cell { width: root.wDl; ink: theme.textDim; text: { var v = root.count(row.i, "downloads"); return v === null ? "" : String(v) } }
                    Cell {
                        width: root.wDate
                        readonly property var at: root.published(row.i)
                        text: at ? Fmt.relative(at, root.nowMs / 1000) : (root.datesState === "loading" ? "..." : "")
                        ink: at ? theme.textDim : theme.textFaint
                        Tooltip { visible: !!parent.at; text: parent.at ? new Date(parent.at * 1000).toLocaleString(Qt.locale(), "d MMMM yyyy, HH:mm") : "" }
                    }
                }
                Button {
                    anchors.right: parent.right
                    anchors.rightMargin: theme.space(2)
                    anchors.verticalCenter: parent.verticalCenter
                    visible: rowHover.hovered
                    text: "Download"
                    icon: "download"
                    flat: true
                    small: true
                    onClicked: root.downloadOne(row.item)
                }
                Rectangle { anchors.bottom: parent.bottom; x: theme.space(2); width: parent.width - theme.space(4); height: 1; color: theme.line; opacity: 0.45 }
            }
        }
    }
    Label {
        id: more
        visible: root.items.length > root.cap
        y: head.height + body.implicitHeight
        topPadding: theme.space(2)
        height: implicitHeight
        text: "and " + (root.items.length - root.cap) + " more. Narrow the list or sort it to see others first."
        role: "ui"
        color: theme.textDim
    }

    component HeadCell: Item {
        id: hc
        property string key: ""
        property string text: ""
        property bool alignRight: true
        readonly property bool active: root.sortKey === key
        height: head.height
        HoverHandler { id: hh; cursorShape: Qt.PointingHandCursor }
        MouseArea { anchors.fill: parent; onClicked: root.sortBy(hc.key) }
        Row {
            anchors.verticalCenter: parent.verticalCenter
            anchors.right: hc.alignRight ? parent.right : undefined
            anchors.rightMargin: hc.alignRight ? theme.space(3) : 0
            anchors.left: hc.alignRight ? undefined : parent.left
            spacing: theme.space(1)
            Label {
                anchors.verticalCenter: parent.verticalCenter
                text: hc.text
                role: "ledger"
                color: hc.active ? theme.accent : hh.hovered ? theme.text : theme.textDim
            }
            Icon {
                visible: hc.active
                anchors.verticalCenter: parent.verticalCenter
                glyph: root.sortDesc ? "chevron-down" : "chevron-up"
                size: theme.space(3.5)
                color: theme.accent
            }
        }
    }
    component Cell: Item {
        property string text: ""
        property color ink: theme.text
        height: parent ? parent.height : 0
        Label {
            anchors.right: parent.right
            anchors.rightMargin: theme.space(3)
            anchors.verticalCenter: parent.verticalCenter
            text: parent.text
            role: "ledger"
            color: parent.ink
        }
    }
    component Tag: Rectangle {
        id: tag
        property string text: ""
        property color dot: "transparent"
        radius: theme.radiusSm
        height: tagRow.implicitHeight + theme.space(0.5) * 2
        width: tagRow.implicitWidth + theme.space(1.5) * 2
        color: theme.raised
        border.color: theme.line
        border.width: 1
        Row {
            id: tagRow
            anchors.centerIn: parent
            spacing: theme.space(1)
            Rectangle { visible: tag.dot.a > 0; anchors.verticalCenter: parent.verticalCenter; width: theme.space(1.75); height: width; radius: width / 2; color: tag.dot }
            Label { text: tag.text; role: "ledger"; font.pointSize: theme.typeSmall; color: theme.textDim }
        }
    }
    component Check: Rectangle {
        property bool on: false
        signal clicked()
        width: theme.space(4.5); height: width
        radius: theme.radiusSm
        color: on ? theme.accent : "transparent"
        border.color: on ? theme.accent : theme.lineStrong
        border.width: 1.5
        Icon { visible: parent.on; anchors.centerIn: parent; glyph: "check"; size: theme.space(3.25); color: theme.contrast(theme.accent, Qt.rgba(1, 1, 1, 1)) >= 3 ? "#ffffff" : "#0d0e11" }
        MouseArea { anchors.fill: parent; anchors.margins: -theme.space(1.5); cursorShape: Qt.PointingHandCursor; onClicked: parent.clicked() }
    }
}
