// A series, flat (DESIGN.md, the flat home pass): a band at the head in the poster's own
// colour, the way Home's hero is, carrying the poster, the title, the other titles on one
// quiet line, the facts as one ledger line, Continue with the progress beside it, and the
// actions as a quiet row. Under it two columns on a wide window: the episodes (then the
// extras and the stray files) as flat rows at the left, and at the right the synopsis, the
// characters, the recommendations, the tags and the franchise. One GetSeries draws it all;
// the relations come from GetFranchiseGraph and follow the crawl through graphChanged. An
// unmatched series draws the same band off its folder name, with the match where the
// synopsis would be.
import QtQuick
import com.marcusrosado.AniBeam

Item {
    id: page
    objectName: "series-page"
    property var props: ({})
    property alias scrollY: scroll.contentY
    property var detail: null
    property var graph: null
    readonly property var card: detail ? detail.card : ({})
    property string title: card.title || frame.nav.current.label
    // Guarded, not a bare relabel call: title falls back to nav.current.label itself
    // before the first GetSeries reply lands, so an unguarded write here relabels the
    // trail entry to the very value it already reads, which changes nav.current, which
    // is what this binding depends on for that same fallback branch, which QML sees as
    // the binding writing back into its own dependency mid-evaluation ("Binding loop
    // detected for property title"). Skipping the write when nothing would actually
    // change breaks that cycle; once the real title arrives, card.title short-circuits
    // the fallback branch, the dependency on nav.current drops, and this relabel is the
    // only write left standing.
    onTitleChanged: if (frame.nav.current.label !== title) frame.nav.relabel(title)
    function escapePressed() { return false }
    property bool spoilers: false
    property bool synopsisOpen: false
    property bool tagsOpen: false
    property real nowMs: Date.now()
    property real optimisticProgress: -1
    property var optimisticFile: -1
    property bool optimisticWatched: false
    readonly property bool isMovie: card.kind === "Movie"
    readonly property bool hasTracker: !!(card.match_info && (card.match_info.anilist_id || card.match_info.mal_id))
    readonly property bool matched: !!card.match_info
    readonly property var titles: card.titles || ({})
    // The other language's title under the display one, then the native script
    readonly property string altTitle: {
        var t = titles
        if (t.romaji && t.english && t.romaji !== t.english) return card.title === t.english ? t.romaji : t.english
        return ""
    }
    readonly property string nativeTitle: titles.native && titles.native !== card.title ? titles.native : ""
    readonly property string otherTitles: [altTitle, nativeTitle].filter(function(s) { return s !== "" }).join("  ")
    // The band's tone, read off the poster; kept as a string for old readers of `tone`
    readonly property string tone: String(toneOf.tone)

    function contextItems() {
        return [
            { text: "Rescan show", icon: "refresh-cw", action: page.rescan },
            { text: "Open in Metadata", icon: "database", action: function() { frame.go("metadata", { q: page.title }) } }
        ]
    }
    function rescan() { var r = Door.rescanSeries(page.props.id); frame.toast(r.error ? r.error.message : "Rescan started") }
    function load() {
        var r = Door.getSeries(props.id)
        if (r.error) { frame.toast(r.error.message); return }
        detail = r.reply.detail
        optimisticProgress = -1
        optimisticFile = -1
        loadGraph()
    }
    function loadGraph() {
        if (!detail || !detail.has_graph) { graph = null; return }
        var g = Door.getFranchiseGraph(props.id)
        graph = g.error ? null : g.reply.layout
    }
    Component.onCompleted: { load(); Door.refreshAiring(props.id) }
    Timer { interval: 1000; running: !!card.next_airing && card.next_airing.at * 1000 > page.nowMs; repeat: true; onTriggered: page.nowMs = Date.now() }
    Timer { id: reloadDebounce; interval: 200; onTriggered: page.load() }
    Timer { id: graphDebounce; interval: 400; onTriggered: page.loadGraph() }
    Connections {
        target: Door
        function onSeriesChanged(cards) { for (var i = 0; i < cards.length; i++) if (cards[i].id === page.props.id) { reloadDebounce.restart(); return } }
        function onAiringRefreshed(series, updated) { if (series === page.props.id && updated) reloadDebounce.restart() }
        function onProgressSet(series, progress, outcomes) { if (series === page.props.id) reloadDebounce.restart() }
        function onScored(series, score, outcomes) { if (series === page.props.id) { reloadDebounce.restart(); frame.toast(outcomes.every(function(o) { return o.ok }) ? "Rated " + (score < 0 ? "cleared" : Fmt.score(score)) : "Score failed") } }
        function onMarked(series, episode, outcomes) { if (series === page.props.id) reloadDebounce.restart() }
        function onMatchApplied(series) { if (series === page.props.id) reloadDebounce.restart() }
        // The crawl fills the franchise in behind the first read; its root is an AniList id
        function onGraphChanged(root) { if (page.graph === null || page.graph.root === root) graphDebounce.restart() }
        function onCrawlFinished(fetched, deferred) { if (page.detail && page.detail.has_graph) graphDebounce.restart() }
        // The core resolves card.title (and every other localised field) at query time,
        // so a title-language flip from the rail's JP/EN switch needs a fresh GetSeries;
        // Door reports every preference write here regardless of what changed, exactly
        // like LibraryPage.qml's own onPreferencesChanged, so there is nothing to diff.
        function onPreferencesChanged() { reloadDebounce.restart() }
    }
    function openFile(file) { frame.go("player", { file: file }, page.title) }
    function status(s) { return { Releasing: "Airing", Finished: "Finished", NotYetReleased: "Upcoming", Cancelled: "Cancelled", Hiatus: "On hiatus" }[s] || s }
    function listStatus(s) { return { Watching: "Watching", Planning: "Planning", Completed: "Completed", Paused: "Paused", Dropped: "Dropped", Repeating: "Rewatching" }[s] || s }
    function formatLabel(f) { return isMovie ? "Film" : !f ? "Series" : ({ TV: "TV", TV_SHORT: "TV short", OVA: "OVA", ONA: "ONA", SPECIAL: "Special", MOVIE: "Film" })[f] || f.replace(/_/g, " ") }
    function relationLabel(r) { if (!r) return ""; var s = String(r).replace(/_/g, " ").toLowerCase(); return s.charAt(0).toUpperCase() + s.slice(1) }
    // An episode's name: Fmt.episodeName, the one rule the player and the feed share (its
    // own title, else "Episode 13"; the file name is never a title). A lone film is the film.
    function episodeName(ep) {
        if (!ep.title && isMovie && detail && detail.episodes.length === 1) return page.title
        return Fmt.episodeName(ep.title || "", ep.number)
    }

    // Track to here / untrack to here, optimistic; the core confirms through progressSet.
    // The progress sent to the core is always the floor of ep.number (progress counts
    // whole episodes), but a half-numbered episode (a split cour, a Part 2) then sits on
    // the wrong side of that integral cutoff for its own row: marking 12.5 floors to a
    // target of 12, and 12.5 <= 12 is false, so the row just clicked would not flip.
    // optimisticFile pins the clicked row to the intended state regardless of the number
    // comparison; every other row still reads the integral target normally.
    function marker(ep) {
        var watched = ep.watched
        var target = watched ? Math.max(0, Math.floor(ep.number) - 1) : Math.floor(ep.number)
        optimisticProgress = target
        optimisticFile = ep.file
        optimisticWatched = !watched
        var r = Door.setProgress(props.id, target)
        if (r.error) { optimisticProgress = -1; optimisticFile = -1; frame.toast(r.error.message) }
    }
    function watchedWithOptimism(ep) {
        if (optimisticProgress < 0) return ep.watched
        if (ep.file === optimisticFile) return optimisticWatched
        return ep.number <= optimisticProgress
    }

    // ---- What Continue plays: the next episode, else (a film, or caught up) the first
    readonly property var episodes: detail ? detail.episodes : []
    readonly property var nextEpisode: {
        if (!detail || !detail.next_up) return null
        for (var i = 0; i < episodes.length; i++) if (episodes[i].file === detail.next_up) return episodes[i]
        return null
    }
    readonly property var playEpisode: nextEpisode ? nextEpisode : (episodes.length ? episodes[0] : null)
    readonly property bool tracked: !!detail && detail.progress.watched !== null && detail.progress.watched !== undefined
    readonly property int watchedCount: tracked ? detail.progress.watched : 0
    readonly property bool caughtUp: tracked && !nextEpisode && watchedCount > 0
    readonly property string playVerb: nextEpisode && watchedCount > 0 ? "Continue" : caughtUp ? "Watch again" : "Play"
    // "Continue episode 8", "Play episode 1", "Play" for a lone film
    readonly property string playText: {
        if (!playEpisode) return playVerb
        if (isMovie && episodes.length === 1) return playVerb
        return playVerb + " " + Fmt.episodeName("", playEpisode.number).toLowerCase()
    }
    readonly property string progressText: {
        if (!detail) return ""
        if (!tracked) return isMovie ? "Not tracked" : "Not tracked, " + Fmt.plural(detail.progress.on_disk, "episode", "episodes") + " on disk"
        var p = detail.progress
        return p.total ? watchedCount + " of " + p.total + (p.estimate ? "+" : "") + " watched" : watchedCount + " watched"
    }
    readonly property real progressFraction: tracked && detail.progress.total ? Math.min(1, watchedCount / detail.progress.total) : 0
    readonly property bool airsAhead: !!card.next_airing && card.next_airing.at * 1000 > nowMs
    readonly property bool hasBanner: !!(detail && detail.banner)
    // The art needs the right two thirds to itself; on a narrow band the words reach into
    // them, so the band goes plain
    readonly property bool showArt: hasBanner && band.width >= theme.space(275)
    readonly property bool hasScore: card.my_score !== null && card.my_score !== undefined

    // ---- The facts, one ledger line: format, year, studio, episodes, on disk, status,
    // the average score, the list status, rewatches
    readonly property string factsLine: {
        if (!detail) return ""
        var f = [formatLabel(card.format)]
        if (detail.year) f.push(String(detail.year))
        if (detail.studio) f.push(detail.studio)
        if (!isMovie && card.total_episodes) f.push(Fmt.plural(card.total_episodes, "episode", "episodes") + (card.total_is_estimate ? "+" : ""))
        if (!isMovie) f.push(detail.progress.on_disk + " on disk")
        if (card.status) f.push(status(card.status))
        if (card.community_score !== null && card.community_score !== undefined) f.push("Average score " + Fmt.score(card.community_score))
        if (card.list_status) f.push(listStatus(card.list_status))
        if (detail.rewatch_count > 0) f.push("Rewatched " + detail.rewatch_count + (detail.rewatch_count === 1 ? " time" : " times"))
        return f.join("  ")
    }
    readonly property string episodesFact: {
        if (!detail) return ""
        if (nextEpisode) return "Next: " + Fmt.episodeName("", nextEpisode.number)
        if (caughtUp) return "All watched"
        return ""
    }
    readonly property string synopsisText: detail ? detail.synopsis.replace(/<br\s*\/?>/gi, "\n").replace(/<[^>]*>/g, "").replace(/[ \t]+/g, " ").replace(/\n\s*\n+/g, "\n").trim() : ""
    readonly property var tagList: detail ? detail.tags.filter(function(t) { return page.spoilers || (!t.spoiler && !t.adult) }).sort(function(a, b) { return b.rank - a.rank }) : []
    readonly property bool anySpoiler: detail ? detail.tags.some(function(t) { return t.spoiler || t.adult }) : false
    readonly property var relationNodes: graph ? graph.nodes.filter(function(n) { return !n.current }).sort(function(a, b) { return a.y - b.y || a.x - b.x }) : []

    // ---- The band's colour: the poster's tone over a dark base, so white ink holds in
    // both modes; the ink accent is the tone lifted toward white (HomePage's hero)
    PosterTone { id: toneOf; source: card.poster ? "file://" + card.poster : "" }
    readonly property color bandTone: toneOf.tone
    // No poster, no tone: the base stays close to the dark ground
    readonly property color bandBase: theme.tone(Qt.rgba(0.05, 0.055, 0.07, 1), bandTone, card.poster ? 0.5 : 0.25)
    readonly property color inkAccent: theme.tone(bandTone, Qt.rgba(1, 1, 1, 1), 0.45)
    readonly property color ink: "#f4f6f8"
    readonly property color inkDim: Qt.rgba(1, 1, 1, 0.68)

    // ---- A section heading: the name in the display cut, its count and one fact beside
    // it, and an action at the right
    component Heading: Item {
        property string text: ""
        property string fact: ""
        property int count: -1
        property alias action: actionSlot.data
        width: parent ? parent.width : 0
        height: theme.space(10)
        Row {
            anchors.left: parent.left
            anchors.bottom: parent.bottom
            anchors.bottomMargin: theme.space(1)
            spacing: theme.space(3)
            Label { id: headingText; text: parent.parent.text; role: "display"; font.pointSize: theme.typeLarge; font.variableAxes: ({ "wdth": 74, "wght": 500 }) }
            Label { visible: parent.parent.count >= 0; anchors.baseline: headingText.baseline; text: String(parent.parent.count); role: "ledger"; color: theme.textFaint }
            Label { anchors.baseline: headingText.baseline; text: parent.parent.fact; role: "ui"; color: theme.textDim }
        }
        Item { id: actionSlot; anchors.right: parent.right; anchors.bottom: parent.bottom; anchors.bottomMargin: theme.space(1); width: childrenRect.width; height: childrenRect.height }
    }
    component Caption: Label {
        role: "ui"
        font.pointSize: theme.typeSmall
        color: theme.textDim
    }

    PageScroll {
        id: scroll
        anchors.fill: parent

        // ---- Back, above the band
        Button {
            objectName: "series-back"
            small: true
            flat: true
            icon: "chevron-left"
            text: frame.nav.backLabel
            onClicked: frame.nav.back()
        }

        // ---- The band: the series' own colour, the banner art fading in from the right
        Item {
            id: band
            width: parent.width
            readonly property real pad: theme.space(8)
            readonly property real posterH: theme.space(72)
            height: Math.max(posterH, words.implicitHeight) + pad * 2

            Rectangle { anchors.fill: parent; radius: theme.radiusXl; color: page.bandBase; antialiasing: true }
            // The art sits in the right two thirds, under a wash of the band's colour that
            // clears to the right
            Item {
                anchors.fill: parent
                clip: true
                Corner {
                    visible: page.showArt
                    x: Math.round(parent.width * 0.3); y: 0
                    width: parent.width - x; height: parent.height
                    radius: theme.radiusXl
                    color: "transparent"
                    fillItem: page.showArt && art.status === Image.Ready ? art : null
                    Image {
                        id: art; visible: false
                        width: parent.width; height: parent.height
                        source: page.hasBanner ? "file://" + page.detail.banner : ""
                        fillMode: Image.PreserveAspectCrop; asynchronous: true
                    }
                }
                Rectangle {
                    visible: page.showArt
                    anchors.fill: parent
                    gradient: Gradient {
                        orientation: Gradient.Horizontal
                        GradientStop { position: 0.28; color: page.bandBase }
                        GradientStop { position: 0.62; color: Qt.rgba(page.bandBase.r, page.bandBase.g, page.bandBase.b, 0.55) }
                        GradientStop { position: 1; color: Qt.rgba(page.bandBase.r, page.bandBase.g, page.bandBase.b, 0.1) }
                    }
                }
            }

            // The poster
            Corner {
                id: posterBox
                x: band.pad
                anchors.verticalCenter: parent.verticalCenter
                height: band.posterH
                width: Math.round(height * 2 / 3)
                radius: theme.radiusTile
                color: Qt.rgba(0, 0, 0, 0.3)
                fillItem: card.poster ? (poster.status === Image.Ready ? poster : null) : null
                Image {
                    id: poster; visible: false
                    anchors.fill: parent
                    source: card.poster ? "file://" + card.poster : ""
                    sourceSize: Qt.size(theme.posterDecodeSize.width * 2, theme.posterDecodeSize.height * 2)
                    fillMode: Image.PreserveAspectCrop; asynchronous: true
                }
                Label { visible: !card.poster; anchors.centerIn: parent; text: "No poster"; color: page.inkDim; role: "ui"; font.pointSize: theme.typeSmall }
            }

            Column {
                id: words
                anchors.left: posterBox.right
                anchors.leftMargin: theme.space(8)
                anchors.right: parent.right
                anchors.rightMargin: band.pad
                anchors.verticalCenter: parent.verticalCenter
                spacing: theme.space(2.5)

                Label {
                    width: parent.width
                    text: page.title
                    role: "display"
                    color: page.ink
                    wrapMode: Text.Wrap
                    maximumLineCount: 2
                    elide: Text.ElideRight
                    lineHeight: 0.92
                }
                Label {
                    visible: page.otherTitles !== ""
                    width: parent.width
                    text: page.otherTitles
                    role: "ui"
                    color: page.inkDim
                    elide: Text.ElideRight
                }
                // The facts, and the next episode's countdown in the band's ink accent
                Row {
                    width: parent.width
                    spacing: theme.space(2)
                    Label {
                        id: factsLabel
                        width: Math.min(implicitWidth, parent.width - (airing.visible ? airing.width + parent.spacing : 0))
                        text: page.factsLine
                        role: "ledger"
                        color: page.inkDim
                        elide: Text.ElideRight
                    }
                    Label {
                        id: airing
                        objectName: "fact-airing"
                        visible: page.airsAhead
                        text: card.next_airing ? (card.next_airing.season && card.next_airing.season !== 1 ? "S" + card.next_airing.season + " " : "") + "Episode " + card.next_airing.episode + " airs in " + Fmt.countdownSeconds(card.next_airing.at - page.nowMs / 1000) : ""
                        role: "ledger"
                        font.weight: 600; font.variableAxes: ({ "wdth": 88, "wght": 600 })
                        color: page.inkAccent
                    }
                }
                Item { width: 1; height: theme.space(1.5) }
                // Continue, with the progress beside it
                Row {
                    spacing: theme.space(4)
                    Button {
                        objectName: "series-continue"
                        anchors.verticalCenter: parent.verticalCenter
                        primary: true
                        tint: page.inkAccent
                        icon: "play"
                        text: page.playText
                        enabled: page.playEpisode !== null
                        onClicked: if (page.playEpisode) page.openFile(page.playEpisode.file)
                    }
                    Row {
                        anchors.verticalCenter: parent.verticalCenter
                        spacing: theme.space(3)
                        visible: !!page.detail
                        Rectangle {
                            visible: page.tracked && !!page.detail.progress.total
                            anchors.verticalCenter: parent.verticalCenter
                            width: theme.space(40); height: theme.space(1)
                            radius: height / 2
                            color: Qt.rgba(1, 1, 1, 0.2)
                            Rectangle {
                                width: Math.max(page.progressFraction > 0 ? height : 0, parent.width * page.progressFraction)
                                height: parent.height; radius: parent.radius
                                color: page.inkAccent
                            }
                        }
                        Label {
                            objectName: "series-progress"
                            anchors.verticalCenter: parent.verticalCenter
                            text: page.progressText
                            role: "ledger"
                            color: page.inkDim
                        }
                    }
                }
                // The quiet row: rate, hide, AniList, rescan
                Flow {
                    width: parent.width
                    spacing: theme.space(2)
                    Button {
                        id: rateButton
                        objectName: "fact-mine"
                        small: true; ghost: true
                        icon: "star"
                        text: page.hasScore ? "Your score " + Fmt.score(card.my_score) : "Rate"
                        onClicked: scorePicker.openAt(rateButton, page.hasScore ? card.my_score : -1)
                    }
                    Button {
                        objectName: "series-hide"
                        small: true; ghost: true
                        text: card.hidden ? "Unhide" : "Hide"
                        icon: card.hidden ? "eye" : "eye-off"
                        onClicked: { var r = Door.setHidden(props.id, !card.hidden); if (r.error) frame.toast(r.error.message) }
                        Tooltip { text: "Hidden from every list, and tracker sync stops" }
                    }
                    Button { visible: !!(page.detail && page.detail.site_url); small: true; ghost: true; text: "AniList"; icon: "external-link"; onClicked: Qt.openUrlExternally(page.detail.site_url) }
                    Button { small: true; ghost: true; text: "Rescan"; icon: "refresh-cw"; onClicked: page.rescan() }
                }
            }
        }

        Item { width: 1; height: theme.space(2) }

        // ---- The body: the episodes at the left, the rest at the right; one column on a
        // narrow window
        Item {
            id: body
            width: parent.width
            readonly property bool twoCol: width >= theme.space(275)
            readonly property real gap: theme.space(12)
            readonly property real sectionGap: theme.space(8)
            height: twoCol ? Math.max(leftCol.height, rightCol.height)
                           : leftCol.height + (rightCol.height > 0 ? sectionGap + rightCol.height : 0)

            Column {
                id: leftCol
                x: 0; y: 0
                width: body.twoCol ? Math.floor((body.width - body.gap) * 0.6) : body.width
                spacing: body.sectionGap

                // The episodes
                Column {
                    objectName: "series-episodes"
                    visible: page.episodes.length > 0
                    width: parent.width
                    spacing: theme.space(4)
                    Heading { text: page.isMovie ? "Film" : "Episodes"; count: page.episodes.length; fact: page.episodesFact }
                    Column {
                        width: parent.width
                        spacing: theme.space(4)
                        Repeater {
                            model: page.episodes
                            EpisodeRow {
                                required property var modelData
                                required property int index
                                objectName: "episode-row-" + index
                                episode: modelData
                                hasTracker: page.hasTracker
                                title: page.episodeName(modelData)
                                watched: page.watchedWithOptimism(modelData)
                                onPlay: page.openFile(modelData.file)
                                onMarker: page.marker(modelData)
                            }
                        }
                    }
                }

                // Openings, endings and the rest, grouped
                Column {
                    visible: !!page.detail && page.detail.extras.length > 0
                    width: parent.width
                    spacing: theme.space(4)
                    Heading { text: "Openings, endings and more"; count: page.detail ? page.detail.extras.length : 0 }
                    Repeater {
                        model: [["Op", "Openings"], ["Ed", "Endings"], ["Pv", "Previews and trailers"], ["Sp", "Specials"], ["Other", "Other"]]
                        Column {
                            required property var modelData
                            readonly property var group: page.detail ? page.detail.extras.filter(function(x) { return x.kind === modelData[0] }) : []
                            visible: group.length > 0
                            width: parent.width
                            spacing: theme.space(2)
                            Caption { text: modelData[1] + ", " + group.length }
                            Column {
                                width: parent.width
                                spacing: theme.space(4)
                                Repeater { model: group; EpisodeRow { required property var modelData; episode: ({ code: modelData.code, title: modelData.label, resume: modelData.resume, watched: false, next_up: false }); title: modelData.label; onPlay: page.openFile(modelData.file) } }
                            }
                        }
                    }
                }

                // Files numbered past the matched count: named by their file, since that is
                // what needs reviewing
                Column {
                    visible: !!page.detail && page.detail.unmatched_files.length > 0
                    width: parent.width
                    spacing: theme.space(4)
                    Heading { text: "Extra files"; count: page.detail ? page.detail.unmatched_files.length : 0 }
                    Row {
                        width: parent.width
                        spacing: theme.space(2)
                        Icon { glyph: "triangle-alert"; size: theme.space(4); color: theme.yellow }
                        Label {
                            width: parent.width - theme.space(6)
                            wrapMode: Text.Wrap; role: "ui"; font.pointSize: theme.typeSmall; color: theme.textDim
                            text: page.detail ? (page.detail.unmatched_files.length === 1 ? "One file is" : page.detail.unmatched_files.length + " files are") + " more than the " + (card.total_episodes || 0) + " expected episodes: misnamed, duplicates or specials. Rename or remove " + (page.detail.unmatched_files.length === 1 ? "it." : "them.") : ""
                        }
                    }
                    Column {
                        width: parent.width
                        spacing: theme.space(4)
                        Repeater { model: page.detail ? page.detail.unmatched_files : []
                            EpisodeRow { required property var modelData; episode: modelData; extra: true; title: Fmt.episodeName(modelData.title || "", modelData.number); detailText: modelData.path.split("/").pop(); onPlay: page.openFile(modelData.file) } }
                    }
                }
            }

            Column {
                id: rightCol
                x: body.twoCol ? leftCol.width + body.gap : 0
                y: body.twoCol ? 0 : leftCol.height + body.sectionGap
                width: body.twoCol ? body.width - x : body.width
                spacing: body.sectionGap

                // The synopsis, at a readable measure
                Column {
                    visible: page.synopsisText !== ""
                    width: parent.width
                    spacing: theme.space(3)
                    Heading { text: "Synopsis" }
                    Label {
                        id: synopsis
                        width: Math.min(parent.width, theme.space(140))
                        text: page.synopsisText
                        role: "ui"; color: theme.textDim
                        wrapMode: Text.Wrap; elide: Text.ElideRight
                        maximumLineCount: page.synopsisOpen ? 1000 : 8
                        lineHeight: 1.2
                    }
                    Button { visible: synopsis.truncated || page.synopsisOpen; small: true; flat: true; text: page.synopsisOpen ? "Less" : "More"; icon: page.synopsisOpen ? "chevron-up" : "chevron-down"; onClicked: page.synopsisOpen = !page.synopsisOpen }
                }

                // Unmatched: the match, where the synopsis would be
                Column {
                    objectName: "series-match"
                    visible: !!page.detail && !page.matched
                    width: parent.width
                    spacing: theme.space(3)
                    Heading { text: "Match" }
                    Label {
                        width: Math.min(parent.width, theme.space(140))
                        text: "Not matched to AniList yet, so there is no poster, synopsis or tracking. Find the record it belongs to."
                        role: "ui"; color: theme.textDim
                        wrapMode: Text.Wrap
                    }
                    Row {
                        spacing: theme.space(2)
                        Button { objectName: "series-find-match"; text: "Find a match"; icon: "search"; primary: true; onClicked: matchPanel.openFor(page.props.id, page.title, card.match_info) }
                        Button { text: "Open in Metadata"; icon: "database"; flat: true; onClicked: frame.go("metadata", { q: page.title }) }
                    }
                }

                // The characters, a row of round portraits
                Column {
                    visible: !!page.detail && page.detail.characters.length > 0
                    width: parent.width
                    spacing: theme.space(3)
                    Heading { text: "Characters"; count: page.detail ? page.detail.characters.length : 0 }
                    Flow { width: parent.width; spacing: theme.space(3); Repeater { model: page.detail ? page.detail.characters : []; PersonCard { required property var modelData; person: modelData } } }
                }

                // Recommendations, a row of small cards
                Column {
                    visible: !!page.detail && page.detail.recommendations.length > 0
                    width: parent.width
                    spacing: theme.space(3)
                    Heading { text: "Recommendations"; count: page.detail ? page.detail.recommendations.length : 0 }
                    Flow { width: parent.width; spacing: theme.space(3)
                        Repeater { model: page.detail ? page.detail.recommendations : []
                            RecommendationCard { required property var modelData; rec: modelData; width: theme.space(26)
                                onOpened: modelData.owned ? frame.go("series", { id: modelData.owned }, modelData.title) : Qt.openUrlExternally("https://anilist.co/anime/" + modelData.anilist_id) } } }
                }

                // Tags by rank; spoiler and adult tags behind the toggle
                Column {
                    visible: !!page.detail && page.detail.tags.length > 0
                    width: parent.width
                    spacing: theme.space(3)
                    Heading {
                        text: "Tags"
                        count: page.tagList.length
                        action: Row {
                            spacing: theme.space(2)
                            Button { visible: page.anySpoiler; small: true; flat: true; text: page.spoilers ? "Hide spoilers" : "Spoilers"; icon: page.spoilers ? "eye-off" : "eye"; onClicked: page.spoilers = !page.spoilers }
                            Button { visible: tagFlow.implicitHeight > tagClip.height || page.tagsOpen; small: true; flat: true; text: page.tagsOpen ? "Less" : "All tags"; onClicked: page.tagsOpen = !page.tagsOpen }
                        }
                    }
                    // A hidden reference chip: chip height depends on font metrics, so the
                    // collapsed clip lands on a whole number of rows by measuring one
                    Chip { id: rowRuler; visible: false; text: "0"; mono: false }
                    Item {
                        id: tagClip
                        width: parent.width
                        readonly property real collapsed: 3 * rowRuler.height + 2 * tagFlow.spacing
                        height: page.tagsOpen ? tagFlow.implicitHeight : Math.min(tagFlow.implicitHeight, collapsed)
                        clip: true
                        Flow {
                            id: tagFlow
                            width: parent.width; spacing: theme.space(2)
                            Repeater {
                                model: page.tagList
                                Chip { required property var modelData; text: modelData.name + ", " + modelData.rank + "%"; mono: false; color: theme.tile; borderColor: "transparent"; textColor: modelData.spoiler || modelData.adult ? theme.text : theme.textDim }
                            }
                        }
                    }
                }

                // Relations: the franchise as small rows, in the crawl's own order
                Column {
                    objectName: "series-relations"
                    visible: page.relationNodes.length > 0
                    width: parent.width
                    spacing: theme.space(3)
                    Heading { text: "Related"; count: page.relationNodes.length }
                    Flow {
                        id: relationFlow
                        width: parent.width
                        spacing: theme.space(3)
                        // As many a row as fit at theme.space(64), the row filled
                        readonly property int per: Math.max(1, Math.floor((width + spacing) / (theme.space(64) + spacing)))
                        readonly property real stoneW: (width - (per - 1) * spacing) / per
                        Repeater {
                            model: page.relationNodes
                            RelationStone { required property var modelData; node: modelData; width: relationFlow.stoneW }
                        }
                    }
                    Caption { visible: !!page.graph && !page.graph.complete; text: "Still filling in from AniList." }
                }
            }
        }
    }

    // One related entry: a poster sliver, the title, the relation and year. Owned opens its
    // page; the rest open AniList.
    component RelationStone: Corner {
        id: rel
        property var node: ({})
        readonly property bool owned: node.owned !== null && node.owned !== undefined
        height: theme.space(24)
        radius: theme.radiusTile
        smoothing: theme.cornerSmoothing
        color: relMouse.containsMouse ? theme.raised : theme.tile
        objectName: "relation-" + node.anilist_id
        Corner {
            id: relThumb
            width: height / 1.5; height: parent.height
            radius: parent.radius; smoothing: theme.cornerSmoothing
            color: theme.tileSunken
            fillItem: relArt.status === Image.Ready ? relArt : null
            Image { id: relArt; visible: false; width: parent.width; height: parent.height
                source: rel.node.poster ? "file://" + rel.node.poster : ""
                sourceSize: Qt.size(Math.ceil(parent.width * theme.devicePixelRatio), Math.ceil(parent.height * theme.devicePixelRatio))
                fillMode: Image.PreserveAspectCrop; asynchronous: true }
        }
        Column {
            x: relThumb.width + theme.space(3)
            width: parent.width - x - theme.space(3)
            anchors.verticalCenter: parent.verticalCenter
            spacing: theme.space(1)
            Label { width: parent.width; text: rel.node.title || ""; role: "title"; font.pointSize: theme.typeNormal; wrapMode: Text.Wrap; maximumLineCount: 2; elide: Text.ElideRight; color: rel.node.pending ? theme.textDim : theme.text }
            Label { width: parent.width; role: "ledger"; color: theme.textDim; elide: Text.ElideRight
                text: [page.relationLabel(rel.node.relation), page.formatLabel(rel.node.format), rel.node.year ? String(rel.node.year) : ""].filter(function(s) { return s !== "" && s !== "Series" }).join(", ") }
            Row {
                spacing: theme.space(1.5)
                StatusDot { anchors.verticalCenter: parent.verticalCenter; status: rel.node.list_status || "" }
                Label { role: "ledger"; text: rel.owned ? "In your library" : !rel.node.released ? "Not yet released" : "On AniList"; color: rel.owned ? theme.accent : theme.textDim }
            }
        }
        MouseArea {
            id: relMouse
            anchors.fill: parent; hoverEnabled: true; cursorShape: Qt.PointingHandCursor
            onClicked: rel.owned ? frame.go("series", { id: rel.node.owned }, rel.node.title) : (rel.node.site_url ? Qt.openUrlExternally(rel.node.site_url) : Qt.openUrlExternally("https://anilist.co/anime/" + rel.node.anilist_id))
        }
    }

    ScorePicker { id: scorePicker; parent: frame.overlay
        onSaved: function(v) { var r = Door.setScore(page.props.id, v); if (r.error) frame.toast(r.error.message) }
        onCleared: { var r = Door.setScore(page.props.id, -1); if (r.error) frame.toast(r.error.message) } }

    // The match, for an unmatched series: the same popover Metadata opens, over the overlay
    MatchPanel {
        id: matchPanel
        parent: frame.overlay
        anchors.centerIn: parent
    }
    Blocker {
        parent: frame.overlay
        anchors.fill: parent
        z: -1
        visible: matchPanel.visible
        dismissOnPress: true
        onDismissed: matchPanel.close()
    }
}
