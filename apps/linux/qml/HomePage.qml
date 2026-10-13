// Home: the page the app opens on, ordered by what the app is for. First, the series last
// watched, but only when there is something to go on with (an episode left half-way, or the
// next one on disk); then what is airing, the series that have episodes waiting, a row of
// things not started yet, and the library's genres as a way in. Each shelf is a plain grid
// whose column count comes from the width, cards a size up for the airing shelf, and a
// shelf that could run long stops at a row or two with "See all" into the library.
import QtQuick
import com.marcusrosado.AniBeam

Item {
    id: page
    objectName: "home-page"
    property var props: ({})
    property string title: "Home"
    property alias scrollY: scroll.contentY
    function escapePressed() { return false }

    property real nowMs: Date.now()
    property var shelf: []        // the library, last viewed first
    property var airingRows: []   // releasing, on disk, not done with
    property var resume: null     // { card, detail, episode, partial }
    property bool libraryEmpty: false
    Timer { interval: 30000; running: true; repeat: true; onTriggered: page.nowMs = Date.now() }

    readonly property real gap: theme.space(4)

    function viewedAt(c) { return c.last_viewed_at ? c.last_viewed_at : 0 }
    function watchedOf(c) { return c.watched !== null && c.watched !== undefined ? c.watched : 0 }

    // The hero: the most recently viewed series with something to go on with. An episode
    // stopped short of its end comes first, else the series' next episode on disk; a
    // series with neither is caught up and is passed over for the one viewed before it.
    function pickResume() {
        var viewed = shelf.filter(function(c) { return page.viewedAt(c) > 0 })
        for (var i = 0; i < viewed.length && i < 8; i++) {
            var c = viewed[i]
            var r = Door.getSeries(c.id)
            if (r.error) continue
            var d = r.reply.detail
            var eps = d.episodes || []
            var partial = null, next = null
            for (var k = 0; k < eps.length; k++) {
                var e = eps[k]
                if (!e.watched && e.resume && e.resume.duration > 0 && e.resume.position / e.resume.duration < 0.9)
                    partial = e
                if (d.next_up && e.file === d.next_up) next = e
            }
            var ep = partial || next
            if (ep) return { card: c, detail: d, episode: ep, partial: partial !== null }
        }
        return null
    }

    function reload() {
        var r = Door.listLibrary("All", "", "LastViewed", "Desc", false)
        if (r.error) { frame.toast(r.error.message); return }
        shelf = r.reply.series
        airingRows = r.reply.airing || []
        var n = r.reply.counts || {}
        libraryEmpty = (n.all || 0) === 0 && (n.watched || 0) === 0
        resume = pickResume()
        nowMs = Date.now()
    }
    Timer { id: debounce; interval: 500; onTriggered: page.reload() }
    Connections {
        target: Door
        function onSeriesChanged() { if (!debounce.running) debounce.start() }
        function onSeriesRemoved() { if (!debounce.running) debounce.start() }
        function onPreferencesChanged(p) { if (!debounce.running) debounce.start() }
    }
    Component.onCompleted: reload()

    function playFile(file, title) { if (file) frame.go("player", { file: file }, title) }
    function openSeries(c) { frame.go("series", { id: c.id }, c.title) }
    function playSeries(c) {
        var r = Door.getSeries(c.id)
        var d = !r.error ? r.reply.detail : null
        var file = d ? (d.next_up ? d.next_up : (c.kind === "Movie" && d.episodes && d.episodes.length ? d.episodes[0].file : null)) : null
        if (file) playFile(file, c.title)
        else openSeries(c)
    }

    // ---- The shelves' lists. The hero's series shows only in the hero.
    readonly property var heroId: resume ? resume.card.id : -1
    function behindOf(c) {
        // Episodes out and not seen: aired or on disk past the watched count
        var w = watchedOf(c), reach = Math.max(c.episodes_on_disk || 0, 0)
        return c.watched_state === "Behind" ? Math.max(1, reach - w) : 0
    }
    readonly property var airingList: {
        var list = airingRows.filter(function(c) { return c.id !== page.heroId })
        // Episodes waiting first, then the soonest to air
        return list.slice().sort(function(a, b) {
            var ba = page.behindOf(a) > 0 ? 1 : 0, bb = page.behindOf(b) > 0 ? 1 : 0
            if (ba !== bb) return bb - ba
            var na = a.next_airing ? a.next_airing.at : 1e12, nb = b.next_airing ? b.next_airing.at : 1e12
            return na - nb
        })
    }
    readonly property var airingIds: {
        var m = {}
        for (var i = 0; i < airingRows.length; i++) m[airingRows[i].id] = true
        return m
    }
    // Episodes waiting: behind on the tracker, or watched recently and not caught up
    readonly property var catchUpList: {
        var month = 45 * 86400, now = nowMs / 1000
        return shelf.filter(function(c) {
            if (c.id === page.heroId || page.airingIds[c.id]) return false
            if (c.watched_state === "Behind") return true
            return page.viewedAt(c) > now - month && c.watched_state !== "CaughtUp"
        })
    }
    // Never started, best rated first
    readonly property var freshList: {
        var list = shelf.filter(function(c) {
            return c.id !== page.heroId && !page.airingIds[c.id] && page.viewedAt(c) === 0
                && page.watchedOf(c) === 0 && (c.episodes_on_disk || 0) > 0
        })
        return list.sort(function(a, b) { return (b.community_score || 0) - (a.community_score || 0) })
    }
    // Every genre in the library with its count, the biggest first
    readonly property var genreList: {
        var counts = {}
        for (var i = 0; i < shelf.length; i++) {
            var g = shelf[i].genres || []
            for (var k = 0; k < g.length; k++) counts[g[k]] = (counts[g[k]] || 0) + 1
        }
        var out = Object.keys(counts).map(function(name) { return { name: name, count: counts[name] } })
        return out.sort(function(a, b) { return b.count - a.count || (a.name < b.name ? -1 : 1) })
    }

    // "4d 18h", "3h 12m", "9m"
    function shortCountdown(secs) {
        var t = Math.max(0, Math.floor(secs))
        var d = Math.floor(t / 86400), h = Math.floor(t % 86400 / 3600), m = Math.floor(t % 3600 / 60)
        return d > 0 ? d + "d " + h + "h" : h > 0 ? h + "h " + m + "m" : Math.max(1, m) + "m"
    }
    readonly property string airingFact: {
        var waiting = airingList.filter(function(c) { return page.behindOf(c) > 0 }).length
        if (waiting > 0) return Fmt.plural(waiting, "series has", "series have") + " new episodes"
        var now = nowMs / 1000, next = null
        for (var i = 0; i < airingList.length; i++) {
            var n = airingList[i].next_airing
            if (n && n.at > now && (!next || n.at < next.n.at)) next = { n: n, c: airingList[i] }
        }
        return next ? next.c.title + " in " + shortCountdown(next.n.at - now) : ""
    }

    // ---- A shelf: a heading with its one fact and a way into the library, over a grid
    component Shelf: Column {
        id: shelfBox
        property string heading: ""
        property string fact: ""
        property var list: []
        property real minWidth: theme.posterWidth
        property int maxRows: 0
        property string moreText: "See all"
        signal more()
        readonly property int cols: Math.max(1, Math.floor((width + page.gap) / (minWidth + page.gap)))
        readonly property real cardWidth: Math.floor((width - page.gap * (cols - 1)) / cols)
        readonly property var shown: maxRows > 0 ? list.slice(0, cols * maxRows) : list
        visible: list.length > 0
        spacing: theme.space(4)

        Item {
            width: shelfBox.width
            height: theme.controlHeight
            Row {
                anchors.left: parent.left
                anchors.verticalCenter: parent.verticalCenter
                spacing: theme.space(3)
                Label { id: shelfTitle; text: shelfBox.heading; role: "title"; anchors.verticalCenter: parent.verticalCenter }
                Label {
                    anchors.baseline: shelfTitle.baseline
                    text: shelfBox.fact
                    role: "ui"
                    color: theme.textDim
                }
            }
            Button {
                anchors.right: parent.right
                anchors.verticalCenter: parent.verticalCenter
                visible: shelfBox.list.length > shelfBox.shown.length
                text: shelfBox.moreText
                small: true
                flat: true
                onClicked: shelfBox.more()
            }
        }
        Flow {
            width: shelfBox.width
            spacing: page.gap
            Repeater {
                model: shelfBox.shown
                Card {
                    required property var modelData
                    item: modelData
                    posterWidth: shelfBox.cardWidth
                    nowMs: page.nowMs
                    onOpened: page.openSeries(modelData)
                    onPlayed: page.playSeries(modelData)
                }
            }
        }
    }

    PageScroll {
        id: scroll
        anchors.fill: parent
        visible: !page.libraryEmpty

        // ---- The hero: the series to go on with, across the page over its banner art
        Item {
            id: hero
            visible: page.resume !== null
            width: parent.width
            height: visible ? theme.space(88) : 0
            readonly property var r: page.resume
            readonly property var d: r ? r.detail : null
            readonly property var c: r ? r.card : null
            readonly property var ep: r ? r.episode : null
            readonly property bool hasArt: !!(d && d.banner)
            readonly property color ink: hasArt ? theme.onPoster : theme.text
            readonly property color inkDim: hasArt ? theme.onPosterDim : theme.textDim
            readonly property real pad: theme.space(8)

            Corner {
                id: band
                anchors.fill: parent
                radius: theme.radiusXl
                color: theme.tile
                fillItem: hero.hasArt && art.status === Image.Ready ? art : null
                Image {
                    id: art; visible: false
                    width: parent.width; height: parent.height
                    source: hero.hasArt ? "file://" + hero.d.banner : ""
                    fillMode: Image.PreserveAspectCrop; asynchronous: true
                }
            }
            // Legibility for the words on the art: one flat wash over the whole band
            Rectangle {
                visible: hero.hasArt
                anchors.fill: parent
                radius: theme.radiusXl
                color: Qt.rgba(0.03, 0.035, 0.045, 0.74)
            }

            Corner {
                id: heroPoster
                x: hero.pad
                anchors.verticalCenter: parent.verticalCenter
                height: parent.height - hero.pad * 2
                width: Math.round(height * 2 / 3)
                radius: theme.radiusTile
                color: theme.raised
                fillItem: hero.c && hero.c.poster && heroPosterImage.status === Image.Ready ? heroPosterImage : null
                Image {
                    id: heroPosterImage; visible: false
                    anchors.fill: parent
                    source: hero.c && hero.c.poster ? "file://" + hero.c.poster : ""
                    sourceSize: Qt.size(theme.posterDecodeSize.width * 2, theme.posterDecodeSize.height * 2)
                    fillMode: Image.PreserveAspectCrop; asynchronous: true
                }
                MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: page.openSeries(hero.c) }
            }

            Column {
                anchors.left: heroPoster.right
                anchors.leftMargin: theme.space(8)
                anchors.right: parent.right
                anchors.rightMargin: hero.pad
                anchors.verticalCenter: parent.verticalCenter
                spacing: theme.space(3)

                Label {
                    text: hero.r && hero.r.partial ? "Pick up where you stopped" : "Up next"
                    role: "ui"
                    color: theme.accentOnPoster
                }
                Label {
                    width: parent.width
                    text: hero.c ? hero.c.title : ""
                    role: "display"
                    color: hero.ink
                    wrapMode: Text.Wrap
                    maximumLineCount: 2
                    elide: Text.ElideRight
                }
                Label {
                    width: parent.width
                    text: hero.ep ? Fmt.episodeName(hero.ep.title || "", hero.ep.number) : ""
                    role: "title"
                    font.pointSize: theme.typeNormal
                    color: hero.ink
                    elide: Text.ElideRight
                }
                // How far into the half-watched episode, or how far into the series
                Row {
                    spacing: theme.space(3)
                    readonly property real frac: hero.r && hero.r.partial
                        ? hero.ep.resume.position / hero.ep.resume.duration
                        : hero.c && hero.c.total_episodes > 0 ? page.watchedOf(hero.c) / hero.c.total_episodes : 0
                    visible: frac > 0
                    Rectangle {
                        anchors.verticalCenter: parent.verticalCenter
                        width: theme.space(60); height: theme.space(1)
                        radius: height / 2
                        color: Qt.rgba(1, 1, 1, hero.hasArt ? 0.18 : 0)
                        border.width: hero.hasArt ? 0 : 1
                        border.color: theme.line
                        Rectangle {
                            width: parent.width * Math.min(1, parent.parent.frac)
                            height: parent.height; radius: parent.radius
                            color: theme.accent
                        }
                    }
                    Label {
                        anchors.verticalCenter: parent.verticalCenter
                        role: "ledger"
                        color: hero.inkDim
                        text: {
                            if (!hero.r) return ""
                            if (hero.r.partial) {
                                var left = Math.max(1, Math.round((hero.ep.resume.duration - hero.ep.resume.position) / 60))
                                return left + " min left"
                            }
                            var w = page.watchedOf(hero.c)
                            return hero.c.total_episodes > 0 ? w + " of " + hero.c.total_episodes + " watched" : ""
                        }
                    }
                }
                Item { width: 1; height: theme.space(2) }
                Row {
                    spacing: theme.space(3)
                    Button {
                        objectName: "home-resume"
                        primary: true
                        icon: "play"
                        text: hero.ep ? (hero.r.partial ? "Resume " : "Play ") + Fmt.episodeName("", hero.ep.number).toLowerCase() : ""
                        onClicked: page.playFile(hero.ep.file, hero.c.title)
                    }
                    Button {
                        text: "Open series"
                        onClicked: page.openSeries(hero.c)
                    }
                }
            }
        }

        Item { width: 1; height: hero.visible ? theme.space(6) : 0 }

        Shelf {
            objectName: "home-airing"
            width: parent.width
            heading: "Airing now"
            fact: page.airingFact
            list: page.airingList
            minWidth: Math.round(theme.posterWidth * 1.2)
        }
        Item { width: 1; height: page.airingList.length ? theme.space(6) : 0 }

        Shelf {
            objectName: "home-catch-up"
            width: parent.width
            heading: "Catch up"
            fact: page.catchUpList.length ? Fmt.plural(page.catchUpList.length, "series", "series") + " with episodes waiting" : ""
            list: page.catchUpList
            maxRows: 2
            onMore: frame.go("library", { arrange: "WatchState" })
        }
        Item { width: 1; height: page.catchUpList.length ? theme.space(6) : 0 }

        Shelf {
            objectName: "home-fresh"
            width: parent.width
            heading: "Start something new"
            fact: Fmt.plural(page.freshList.length, "series", "series") + " not started"
            list: page.freshList
            maxRows: 1
            onMore: frame.go("library")
        }
        Item { width: 1; height: page.freshList.length ? theme.space(6) : 0 }

        // ---- Genres: a way into the library by kind
        Column {
            objectName: "home-genres"
            visible: page.genreList.length > 0
            width: parent.width
            spacing: theme.space(4)
            Item {
                width: parent.width
                height: theme.controlHeight
                Label { anchors.verticalCenter: parent.verticalCenter; text: "Genres"; role: "title" }
                Button {
                    anchors.right: parent.right
                    anchors.verticalCenter: parent.verticalCenter
                    text: "Open library"
                    icon: "layers"
                    small: true
                    flat: true
                    onClicked: frame.go("library")
                }
            }
            Flow {
                width: parent.width
                spacing: theme.space(2)
                Repeater {
                    model: page.genreList
                    Rectangle {
                        id: genre
                        required property var modelData
                        width: genreRow.implicitWidth + theme.space(6)
                        height: theme.space(10)
                        radius: height / 2
                        color: gm.containsMouse ? theme.raised : theme.tile
                        border.color: gm.containsMouse ? theme.lineStrong : theme.line
                        border.width: 1
                        scale: gm.pressed ? 0.97 : 1
                        Row {
                            id: genreRow
                            anchors.centerIn: parent
                            spacing: theme.space(2)
                            Label { text: genre.modelData.name; role: "ui"; anchors.verticalCenter: parent.verticalCenter }
                            Label { text: genre.modelData.count; role: "ledger"; color: theme.textFaint; anchors.verticalCenter: parent.verticalCenter }
                        }
                        MouseArea {
                            id: gm
                            anchors.fill: parent
                            hoverEnabled: true
                            cursorShape: Qt.PointingHandCursor
                            onClicked: frame.go("library", { genre: genre.modelData.name }, genre.modelData.name)
                        }
                    }
                }
            }
        }
    }

    EmptyState {
        anchors.centerIn: parent
        visible: page.libraryEmpty
        icon: "folder-plus"
        title: "No series yet"
        body: "Add the folder your anime lives in and the library fills itself."
        Button { text: "Add a folder"; primary: true; onClicked: frame.go("settings") }
    }
}
