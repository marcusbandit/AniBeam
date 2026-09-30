// Spec 4.1 unit 3: the hero, the chips, the synopsis and tags, Continue, the episodes,
// the extras, characters, recommendations and Related. One GetSeries draws it all.
import QtQuick
import QtQuick.Effects
import com.marcusrosado.AniBeam

PageScroll {
    id: page
    property var props: ({})
    property var detail: null
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
    property bool spoilers: false
    property bool synopsisOpen: false
    property bool tagsOpen: false
    property real nowMs: Date.now()
    property real optimisticProgress: -1
    property var optimisticFile: -1
    property bool optimisticWatched: false
    readonly property bool isMovie: card.kind === "Movie"
    readonly property bool hasTracker: !!(card.match_info && (card.match_info.anilist_id || card.match_info.mal_id))
    readonly property string altTitle: {
        var t = card.titles || {}
        if (t.romaji && t.english && t.romaji !== t.english) return card.title === t.english ? t.romaji : t.english
        return ""
    }
    function contextItems() {
        return [
            { text: "Rescan show", icon: "refresh-cw", action: function() { var r = Door.rescanSeries(page.props.id); frame.toast(r.error ? r.error.message : "Rescan started") } },
            { text: "To Metadata", icon: "database", action: function() { frame.go("metadata", { q: page.title }) } }
        ]
    }
    function load() {
        var r = Door.getSeries(props.id)
        if (r.error) { frame.toast(r.error.message); return }
        detail = r.reply.detail
        optimisticProgress = -1
        optimisticFile = -1
    }
    Component.onCompleted: { load(); Door.refreshAiring(props.id) }
    Timer { interval: 1000; running: !!card.next_airing && card.next_airing.at * 1000 > page.nowMs; repeat: true; onTriggered: page.nowMs = Date.now() }
    Timer { id: reloadDebounce; interval: 200; onTriggered: page.load() }
    Connections {
        target: Door
        function onSeriesChanged(cards) { for (var i = 0; i < cards.length; i++) if (cards[i].id === page.props.id) { reloadDebounce.restart(); return } }
        function onAiringRefreshed(series, updated) { if (series === page.props.id && updated) reloadDebounce.restart() }
        function onProgressSet(series, progress, outcomes) { if (series === page.props.id) reloadDebounce.restart() }
        function onScored(series, score, outcomes) { if (series === page.props.id) { reloadDebounce.restart(); frame.toast(outcomes.every(function(o) { return o.ok }) ? "Rated " + (score < 0 ? "cleared" : Fmt.score(score)) : "Score failed") } }
        function onMarked(series, episode, outcomes) { if (series === page.props.id) reloadDebounce.restart() }
        // The core resolves card.title (and every other localised field) at query time,
        // so a title-language flip from the rail's JP/EN switch needs a fresh GetSeries;
        // Door reports every preference write here regardless of what changed, exactly
        // like LibraryPage.qml's own onPreferencesChanged, so there is nothing to diff.
        function onPreferencesChanged() { reloadDebounce.restart() }
    }
    function openFile(file) { frame.go("player", { file: file }, page.title) }
    function status(s) { return { Releasing: "Airing", Finished: "Finished", NotYetReleased: "Upcoming", Cancelled: "Cancelled", Hiatus: "Hiatus" }[s] || s }
    function formatLabel(f) { return isMovie ? "Movie" : !f ? "Series" : ({ TV: "TV", TV_SHORT: "TV Short", OVA: "OVA", ONA: "ONA", SPECIAL: "Special" })[f] || f.replace(/_/g, " ") }
    function progressText() {
        var p = detail.progress
        if (p.watched === null || p.watched === undefined) return p.on_disk + " on disk"
        var denom = p.total ? p.total : "?"
        var width = p.total ? String(p.total).length : 2
        return String(p.watched).padStart(width, "0") + " / " + denom + (p.estimate ? "+" : "")
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

    // Hero
    Item {
        id: hero
        width: parent.width; height: theme.space(60)
        readonly property bool hasBanner: !!(detail && detail.banner)
        Corner {
            anchors.fill: parent
            radius: theme.radiusXl; smoothing: theme.cornerSmoothing; color: theme.surface
            // No banner: the fill is the blurred copy, not the sharp one. A sharp fillItem
            // with a separate rectangular MultiEffect blurred on top of it at 0.6 opacity
            // (the previous shape) reads as a sharp image with a ghost, and that rectangle
            // is not clipped to this Corner's rounded corners, so the blur also bled past
            // them; feeding the blur straight into fillItem fixes both at once.
            fillItem: art.status === Image.Ready ? (hero.hasBanner ? art : blurredArt) : null
            Image { id: art; visible: false; width: parent.width; height: parent.height; fillMode: Image.PreserveAspectCrop; asynchronous: true
                source: hero.hasBanner ? "file://" + detail.banner : (card.poster ? "file://" + card.poster : "") }
            MultiEffect { id: blurredArt; visible: false; anchors.fill: art; source: art; blurEnabled: true; blur: 1.0; blurMax: 64 }
        }
        Rectangle { anchors.fill: parent; color: theme.scrim; opacity: 0.55 }
        // z: 1: declared before the poster Row below, so without this it paints under the
        // poster's corner (same child-order stacking, both at the default z) and never
        // shows at all.
        Chip { z: 1; x: theme.space(4); y: theme.space(4); text: frame.nav.backLabel; icon: "arrow-left"; mono: false; clickable: true; onClicked: frame.nav.back() }
        Row {
            anchors.left: parent.left; anchors.bottom: parent.bottom; anchors.margins: theme.space(6)
            spacing: theme.space(5)
            Corner {
                width: theme.space(36); height: width * 1.5
                radius: theme.radiusLg; smoothing: theme.cornerSmoothing; color: theme.surfaceRaised; borderColor: theme.line; borderWidth: 1
                fillItem: poster.status === Image.Ready ? poster : null
                Image { id: poster; visible: false; width: parent.width; height: parent.height; source: card.poster ? "file://" + card.poster : ""; fillMode: Image.PreserveAspectCrop; asynchronous: true; sourceSize.width: 480 }
            }
            Column {
                anchors.bottom: parent.bottom
                spacing: theme.space(2)
                Text { text: page.title; color: theme.text; font.family: theme.fontSans; font.pointSize: theme.typeLarge; font.weight: Font.Bold; width: page.width - theme.space(64); elide: Text.ElideRight; wrapMode: Text.Wrap; maximumLineCount: 2 }
                Text { visible: page.altTitle !== ""; text: page.altTitle; color: theme.textDim; font.family: theme.fontSans; font.pointSize: theme.typeNormal }
                Row {
                    spacing: theme.space(2)
                    Chip { visible: card.community_score !== null && card.community_score !== undefined; icon: "star"; text: Fmt.score(card.community_score || 0); textColor: theme.yellow; Tooltip { text: "Average rating" } }
                    Chip { id: myScore; icon: "star"; mono: false; text: card.my_score !== null && card.my_score !== undefined ? Fmt.score(card.my_score) + "  You" : "Rate"; clickable: true; selected: card.my_score !== null && card.my_score !== undefined
                        onClicked: scorePicker.openAt(myScore, card.my_score === null || card.my_score === undefined ? -1 : card.my_score) }
                    Chip { visible: !!(detail && detail.site_url); text: "AniList"; icon: "external-link"; mono: false; clickable: true; onClicked: Qt.openUrlExternally(detail.site_url) }
                    Chip { text: card.hidden ? "Unhide" : "Hide"; icon: card.hidden ? "eye" : "eye-off"; mono: false; clickable: true; onClicked: { var r = Door.setHidden(props.id, !card.hidden); if (r.error) frame.toast(r.error.message) }
                        Tooltip { text: "Incognito: stops tracker sync and hides from all lists" } }
                }
            }
        }
    }

    // Info chips
    Flow {
        width: parent.width
        spacing: theme.space(2)
        Chip { text: page.formatLabel(card.format); mono: false; textColor: theme.hue(Theme.formatHue(page.isMovie ? "MOVIE" : (card.format || ""))) }
        Chip { visible: !!(detail && detail.year); text: String(detail ? detail.year : ""); }
        Chip { visible: !page.isMovie && card.total_episodes; text: card.total_episodes + " ep" }
        Chip { visible: !!(detail && detail.studio); text: detail ? detail.studio : ""; mono: false; Tooltip { text: "Animation studio" } }
        Chip { visible: !!card.status; text: page.status(card.status); mono: false }
        Chip { visible: !!card.next_airing && card.next_airing.at * 1000 > page.nowMs; icon: "clock"; textColor: theme.accent
            text: card.next_airing ? "EP " + String(card.next_airing.episode).padStart(2, "0") + " in " + Fmt.countdownSeconds(card.next_airing.at - page.nowMs / 1000) : "" }
        // Chip has no content slot of its own, so the dot sits beside it, not inside it:
        // a Row injected as a child of Chip painted over its own centred label instead of
        // widening it.
        Row {
            visible: !!card.list_status
            spacing: theme.space(1)
            StatusDot { anchors.verticalCenter: parent.verticalCenter; status: card.list_status || "" }
            Chip { mono: false; text: card.list_status === "Repeating" ? "Rewatching" : (card.list_status || "") }
        }
        Chip { visible: !!(detail && detail.rewatch_count > 0); icon: "rotate-cw"; text: detail ? detail.rewatch_count + "x rewatched" : ""; textColor: theme.purple }
    }

    // Synopsis, five lines, More and Less only when it overflows
    Column {
        width: parent.width
        spacing: theme.space(1)
        Text {
            id: synopsis
            width: parent.width
            text: detail ? detail.synopsis.replace(/<[^>]*>/g, " ").replace(/\s+/g, " ").trim() : ""
            color: theme.textDim; font.family: theme.fontSans; font.pointSize: theme.typeNormal
            wrapMode: Text.Wrap; elide: Text.ElideRight
            maximumLineCount: page.synopsisOpen ? 1000 : 5
        }
        Chip { visible: synopsis.truncated || page.synopsisOpen; text: page.synopsisOpen ? "Less" : "More"; mono: false; clickable: true; color: theme.surface; textColor: theme.textDim; onClicked: page.synopsisOpen = !page.synopsisOpen }
    }

    // Tags by rank; spoiler and adult tags behind the toggle
    Column {
        width: parent.width
        spacing: theme.space(2)
        visible: detail && detail.tags.length > 0
        readonly property var tags: detail ? detail.tags.filter(function(t) { return page.spoilers || (!t.spoiler && !t.adult) }).sort(function(a, b) { return b.rank - a.rank }) : []
        readonly property bool anySpoiler: detail ? detail.tags.some(function(t) { return t.spoiler || t.adult }) : false
        // A hidden reference chip, never shown: real chip height depends on font metrics
        // (system font, point size, density), so it cannot be a constant. Measuring one
        // lets the collapsed clip below land on a whole number of rows instead of slicing
        // through one, at every density.
        Chip { id: rowRuler; visible: false; text: "0"; mono: false }
        readonly property int collapsedRows: 3
        readonly property real collapsedHeight: collapsedRows * rowRuler.height + (collapsedRows - 1) * tagFlow.spacing
        Row {
            spacing: theme.space(2)
            Text { text: "Tags"; color: theme.text; font.family: theme.fontSans; font.pointSize: theme.typeNormal; font.weight: Font.Bold; anchors.verticalCenter: parent.verticalCenter }
            Chip { visible: parent.parent.anySpoiler; text: "Spoilers"; icon: page.spoilers ? "eye-off" : "eye"; mono: false; clickable: true; selected: page.spoilers; color: selected ? theme.accentSoft : theme.surface; textColor: theme.textDim; onClicked: page.spoilers = !page.spoilers }
            Chip { visible: tagFlow.implicitHeight > tagClip.height || page.tagsOpen; text: page.tagsOpen ? "Less" : "Show all"; mono: false; clickable: true; color: theme.surface; textColor: theme.textDim; onClicked: page.tagsOpen = !page.tagsOpen }
        }
        Item {
            id: tagClip
            width: parent.width
            height: page.tagsOpen ? tagFlow.implicitHeight : Math.min(tagFlow.implicitHeight, parent.collapsedHeight)
            clip: true
            Flow {
                id: tagFlow
                width: parent.width; spacing: theme.space(1.5)
                Repeater {
                    model: parent.parent.parent.tags
                    Chip { required property var modelData; text: modelData.name + "  " + modelData.rank; mono: false; color: theme.surface; textColor: modelData.spoiler || modelData.adult ? theme.yellow : theme.textDim }
                }
            }
        }
    }

    // Continue or Play, and the progress line
    Row {
        spacing: theme.space(4)
        readonly property var nextFile: detail ? (page.isMovie ? (detail.episodes.length && !(card.list_status === "Completed" || (card.watched > 0)) ? detail.episodes[0].file : null) : detail.next_up) : null
        readonly property var nextEpisode: detail && !page.isMovie && detail.next_up ? detail.episodes.find(function(e) { return e.file === detail.next_up }) : null
        Button { visible: parent.nextFile !== null && parent.nextFile !== undefined; icon: "play"; text: page.isMovie ? "Play" : "Continue" + (parent.nextEpisode ? "  " + parent.nextEpisode.code : ""); onClicked: page.openFile(parent.nextFile) }
        Column {
            visible: !page.isMovie && !!detail
            anchors.verticalCenter: parent.verticalCenter
            spacing: theme.space(1)
            width: theme.space(60)
            Row { width: parent.width
                Text { text: detail && detail.progress.watched !== null ? "Tracked" : "Not tracked"; color: theme.textFaint; font.family: theme.fontSans; font.pointSize: theme.typeSmall }
                Item { width: parent.width - trackedText.width - untrackedLabel.width; height: 1; Text { id: untrackedLabel; visible: false } }
                Text { id: trackedText; text: detail ? page.progressText() : ""; color: theme.textDim; font.family: theme.fontMono; font.pointSize: theme.typeSmall } }
            Corner { width: parent.width; height: theme.space(1.5); radius: height / 2; smoothing: theme.cornerSmoothing; color: theme.surfaceSunken; borderColor: theme.line; borderWidth: 1
                Corner { readonly property real f: detail && detail.progress.watched !== null && detail.progress.total ? Math.min(1, detail.progress.watched / detail.progress.total) : 0
                    width: parent.width * f; height: parent.height; radius: height / 2; smoothing: theme.cornerSmoothing; color: theme.accent } }
        }
    }

    // Episodes
    Column {
        width: parent.width
        spacing: theme.space(0.5)
        visible: detail && detail.episodes.length > 0
        SectionHeader { title: page.isMovie ? "Film" : "Episodes"; count: detail ? detail.episodes.length : 0 }
        Repeater {
            model: detail ? detail.episodes : []
            EpisodeRow { required property var modelData; episode: modelData; hasTracker: page.hasTracker; title: page.isMovie && detail.episodes.length === 1 ? page.title : (modelData.title || modelData.path.split("/").pop())
                watched: page.watchedWithOptimism(modelData)
                onPlay: page.openFile(modelData.file); onMarker: page.marker(modelData) }
        }
    }

    // Extras, grouped
    Column {
        width: parent.width
        spacing: theme.space(2)
        visible: detail && detail.extras.length > 0
        SectionHeader { title: "Openings, Endings & More"; count: detail ? detail.extras.length : 0 }
        Repeater {
            model: [["Op", "Openings"], ["Ed", "Endings"], ["Pv", "Previews & Trailers"], ["Sp", "Specials"], ["Other", "Other"]]
            Column {
                required property var modelData
                readonly property var group: detail ? detail.extras.filter(function(x) { return x.kind === modelData[0] }) : []
                visible: group.length > 0
                width: parent.width
                spacing: theme.space(0.5)
                Row { spacing: theme.space(2); Text { text: modelData[1]; color: theme.textDim; font.family: theme.fontSans; font.pointSize: theme.typeSmall; font.weight: Font.DemiBold; anchors.verticalCenter: parent.verticalCenter } Chip { text: String(group.length); small: true; color: theme.surface; textColor: theme.textFaint; anchors.verticalCenter: parent.verticalCenter } }
                Repeater { model: group; EpisodeRow { required property var modelData; episode: ({ code: modelData.code, title: modelData.label, resume: modelData.resume, watched: false, next_up: false }); title: modelData.label; onPlay: page.openFile(modelData.file) } }
            }
        }
    }

    // Files numbered past the matched count
    Column {
        width: parent.width
        spacing: theme.space(0.5)
        visible: detail && detail.unmatched_files.length > 0
        SectionHeader { title: "Extra files"; count: detail ? detail.unmatched_files.length : 0 }
        Row { spacing: theme.space(2); Icon { glyph: "triangle-alert"; size: theme.space(4); color: theme.yellow; anchors.verticalCenter: parent.verticalCenter }
            Text { width: page.width - theme.space(24); wrapMode: Text.Wrap; color: theme.textDim; font.family: theme.fontSans; font.pointSize: theme.typeSmall
                text: detail ? (detail.unmatched_files.length === 1 ? "1 file goes" : detail.unmatched_files.length + " files go") + " beyond the expected " + card.total_episodes + " episode" + (card.total_episodes === 1 ? "" : "s") + " for this title, likely misnamed, duplicates, or specials. Review them and rename or remove what doesn't belong." : "" } }
        Repeater { model: detail ? detail.unmatched_files : []; EpisodeRow { required property var modelData; episode: modelData; extra: true; title: modelData.title || modelData.path.split("/").pop(); onPlay: page.openFile(modelData.file) } }
    }

    // Characters, then Recommendations
    Column {
        width: parent.width; spacing: theme.space(2)
        visible: detail && detail.characters.length > 0
        SectionHeader { title: "Characters"; count: detail ? detail.characters.length : 0 }
        Flow { width: parent.width; spacing: theme.space(3); Repeater { model: detail ? detail.characters : []; PersonCard { required property var modelData; person: modelData } } }
    }
    Column {
        width: parent.width; spacing: theme.space(2)
        visible: detail && detail.recommendations.length > 0
        SectionHeader { title: "Recommendations"; count: detail ? detail.recommendations.length : 0 }
        Flow { width: parent.width; spacing: theme.space(3)
            Repeater { model: detail ? detail.recommendations : []
                RecommendationCard { required property var modelData; rec: modelData
                    onOpened: modelData.owned ? frame.go("series", { id: modelData.owned }, modelData.title) : Qt.openUrlExternally("https://anilist.co/anime/" + modelData.anilist_id) } } }
    }

    // Related: the franchise graph (Task 24 fills the component)
    Column {
        width: parent.width; spacing: theme.space(2)
        visible: detail && detail.has_graph
        SectionHeader { title: "Related" }
        Loader { id: related; width: parent.width; height: theme.space(120); active: detail && detail.has_graph; sourceComponent: relatedPlaceholder }
        Component { id: relatedPlaceholder; Corner { radius: theme.radiusMd; smoothing: theme.cornerSmoothing; color: theme.surfaceSunken
            Text { anchors.centerIn: parent; text: "Franchise graph, Task 24"; color: theme.textFaint; font.family: theme.fontSans; font.pointSize: theme.typeSmall } } }
    }

    ScorePicker { id: scorePicker; parent: frame.overlay
        onSaved: function(v) { var r = Door.setScore(page.props.id, v); if (r.error) frame.toast(r.error.message) }
        onCleared: { var r = Door.setScore(page.props.id, -1); if (r.error) frame.toast(r.error.message) } }
}
