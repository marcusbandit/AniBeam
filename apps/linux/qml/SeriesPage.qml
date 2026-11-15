// A series, flat (DESIGN.md, the flat home pass): a band at the head in the poster's own
// colour, the way Home's hero is, carrying the poster, the title, the other titles on one
// quiet line (panned on hover, unfolded on a click when they run past the band), the facts
// as one ledger line, Continue with the progress beside it, and a quiet row of the scores,
// the list status and the actions. Under it two columns on a wide window: the episodes (then the
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
    // Escape leaves picking files to delete before it leaves the page
    function escapePressed() { if (page.deleting) { page.deleting = false; page.deleteError = ""; return true } return false }
    property bool spoilers: false
    property bool synopsisOpen: false
    property bool tagsOpen: false
    property real nowMs: Date.now()
    property real optimisticProgress: -1
    property var optimisticFile: -1
    property bool optimisticWatched: false
    readonly property bool isMovie: card.kind === "Movie"
    readonly property bool hasTracker: !!(card.match_info && (card.match_info.anilist_id || card.match_info.mal_id))
    // The connected trackers this series' writes reach, as a person names them
    readonly property string trackerSites: {
        var m = card.match_info, out = []
        if (m && m.anilist_id && frame.trackerOn("anilist")) out.push("AniList")
        if (m && m.mal_id && frame.trackerOn("mal")) out.push("MyAnimeList")
        return out.length === 2 ? words.t("series-trackers-both", { first: out[0], second: out[1] }) : out.join("")
    }
    readonly property bool matched: !!card.match_info
    // The site the record came from, named on the button that opens it
    readonly property string siteName: frame.providerName(card.match_info ? card.match_info.provider : "Anilist") || "AniList"
    readonly property var titles: card.titles || ({})
    // The show's other titles under the display one: romaji, english, then the native script,
    // each once and never the one already shown
    readonly property var otherTitles: [titles.romaji, titles.english, titles.native]
        .filter(function(s, i, all) { return !!s && s !== card.title && all.indexOf(s) === i })
    // The band's tone, read off the poster; kept as a string for old readers of `tone`
    readonly property string tone: String(toneOf.tone)

    // The episode count set by hand (#239); null clears it
    function setEpisodeCount(n) {
        var r = Door.call("SetEpisodeCount", { series: page.props.id, count: n })
        if (r && r.error) frame.fail(r.error)
        else frame.toast(n === null ? words.t("series-count-cleared") : words.t("series-count-set", { count: n }))
    }
    // Schedule notes (#418): what the user told the schedule, for the card's season
    readonly property var scheduleNotes: detail && detail.schedule_notes ? detail.schedule_notes : []
    function openSchedule() {
        // The cells run over the card's season: what is out is faint, a few spare cells after
        var season = card.count_season === undefined ? null : card.count_season
        var rows = page.episodeRows.filter(function(r) { return season === null || (r.season || 1) === season })
        var top = 0, out = []
        rows.forEach(function(r) {
            top = Math.max(top, Math.floor(r.number))
            if (!r.upcoming) out.push(Math.floor(r.number))
        })
        var upTo = Math.max(card.total_episodes || 0, top + 8, 16)
        // What the record already says: each episode's air date, from the files, the aired
        // ones with no file, and the ones still to come
        var known = {}
        var noted = function(n) {
            return page.scheduleNotes.some(function(x) { return n >= x.first && n <= x.last })
        }
        var mark = function(n, at, guess) {
            // A date from the user's own notes is not the record's
            if (at === null || at === undefined || noted(Math.floor(n))) return
            known[Math.floor(n)] = { at: at, guess: !!guess }
        }
        ;(page.detail ? page.detail.episodes : []).forEach(function(e) { if (season === null || (e.season || 1) === season) mark(e.number, e.air_date, false) })
        ;(page.detail && page.detail.absent ? page.detail.absent : []).forEach(function(e) { if (season === null || (e.season || 1) === season) mark(e.number, e.air_date, false) })
        ;(page.detail && page.detail.upcoming ? page.detail.upcoming : []).forEach(function(e) { if (season === null || (e.season || 1) === season) mark(e.number, e.at, e.predicted) })
        var endAt = card.episodes_set === undefined || card.episodes_set === null ? 0 : card.episodes_set
        scheduleSheet.openFor(page.props.id, page.title, season, page.scheduleNotes, Math.max(upTo, endAt + 2), out, endAt, known, page.siteName)
    }
    // The notes without a date, in words: "Episodes 8 to 15 TBC in 2027"
    readonly property string heldLine: {
        var held = scheduleNotes.filter(function(n) { return n.at === null || n.at === undefined })
        if (!held.length) return ""
        var n = held[0]
        var one = n.first === n.last
        if (n.year === null || n.year === undefined)
            return words.t(one ? "series-held-one-undated" : "series-held-range-undated", { first: n.first, last: n.last })
        var when = n.month ? Qt.formatDate(new Date(n.year, n.month - 1, 1), words.t("series-held-month-format")) : String(n.year)
        return words.t(one ? "series-held-one-due" : "series-held-range-due", { first: n.first, last: n.last, when: when })
    }
    // Fixes (#283): every way to correct what the page shows, in one menu
    // Three questions (#432): how many episodes and when, what each file is, which show this is
    function fixItems() {
        var items = []
        if (!page.isMovie)
            items.push({ text: words.t("series-fix-schedule"), icon: "calendar-clock", action: page.openSchedule })
        items.push(page.filesHere > 0
            ? { text: words.t("series-fix-files"), icon: "folder", action: function() { page.openFix(page.detail ? page.detail.suggested_fix : null, false) } }
            : { text: words.t("series-fix-rescan-folder"), icon: "refresh-cw", action: page.rescan })
        items.push({ text: page.matched ? words.t("series-fix-match") : words.t("series-find-match"), icon: "link", action: function() { matchPanel.openFor(page.props.id, page.title, card.match_info) } })
        return items
    }
    function contextItems() {
        return [
            { text: words.t("series-rescan-show"), icon: "refresh-cw", action: page.rescan },
            { text: words.t("series-open-in-show-info"), icon: "database", action: function() { frame.go("metadata", { q: page.title }) } }
        ]
    }
    // Deleting (#158, #168), the phone's way: Delete in the first section's heading, then a bar
    // that counts the picks and says where they go, and any row with a file picks on a click:
    // episodes, extras and the stray files alike. The picked files, by id.
    property bool deleting: false
    property var deletePicked: []
    property string deleteError: ""
    // Every file on the page, and which of them are episodes
    readonly property var episodeFiles: detail ? detail.episodes.map(function(e) { return e.file }) : []
    readonly property var pageFiles: detail ? episodeFiles
        .concat(detail.extras.map(function(x) { return x.file }), detail.unmatched_files.map(function(e) { return e.file })) : []
    readonly property int filesHere: pageFiles.length
    // The section whose heading carries Delete and whose head the bar sits under
    readonly property string deleteAt: filesHere === 0 ? ""
        : episodeRows.length > 0 ? "episodes" : detail && detail.extras.length > 0 ? "extras" : "stray"
    readonly property string pickNoun: deletePicked.every(function(f) { return page.episodeFiles.indexOf(f) >= 0 })
        ? (isMovie ? "film" : "episode") : "file"
    // The words' arguments for n picks: the count and what they are
    function pickArgs(n) { return { count: n, kind: pickNoun } }
    function startDelete() { page.deletePicked = []; page.deleteError = ""; page.deleting = true }
    function togglePick(file) {
        var i = deletePicked.indexOf(file), next = deletePicked.slice()
        if (i >= 0) next.splice(i, 1); else next.push(file)
        deletePicked = next
    }
    // A pick whose file went away (a scan, a delete elsewhere) drops out
    onPageFilesChanged: if (deletePicked.some(function(f) { return page.pageFiles.indexOf(f) < 0 }))
        deletePicked = deletePicked.filter(function(f) { return page.pageFiles.indexOf(f) >= 0 })
    // Off the Qt thread: trashing big files can take a while. The picks stay until it worked.
    property real deleteToken: -1
    // What the toast names once the trash answers: the picks as sent, since a reload while the
    // call runs can drop picks whose files are already gone
    property var deleteSent: ({ count: 0, kind: "file" })
    function deletePickedFiles() {
        if (deleteToken >= 0 || deletePicked.length === 0) return
        page.deleteError = ""
        page.deleteSent = page.pickArgs(deletePicked.length)
        deleteToken = Door.callAsync("DeleteEpisodes", { series: page.props.id, files: deletePicked.slice() })
    }
    Connections {
        target: Door
        function onCallDone(token, reply) {
            if (token !== page.deleteToken) return
            page.deleteToken = -1
            if (reply.error) { page.deleteError = frame.errorText(reply.error); return }
            page.deleting = false
            page.deletePicked = []
            frame.toast(words.t("series-delete-done", page.deleteSent))
        }
    }
    // Get episodes (#169): the Downloads page's show view for this series
    function getEpisodes() { frame.go("downloads", { series: page.props.id }) }
    // The quick fix (#170): every file on the page as the sheet lists them, by name
    function fixRows() {
        if (!detail) return []
        function season(e) { return e.season === undefined ? null : e.season }
        var rows = detail.episodes.concat(detail.unmatched_files).map(function(e) {
            return { file: e.file, name: Fmt.baseName(e.path || ""), reading: { Episode: { number: e.number, season: season(e) } }, fixed: !!e.fixed }
        }).concat(detail.extras.map(function(x) {
            return { file: x.file, name: Fmt.baseName(x.path || ""), reading: { Extra: { kind: x.kind, index: x.index === undefined ? null : x.index } }, fixed: !!x.fixed }
        }))
        // Natural order: OVA 2 before OVA 10
        function chunks(s) { return String(s).toLowerCase().match(/\d+|\D+/g) || [] }
        return rows.sort(function(a, b) {
            var x = chunks(a.name), y = chunks(b.name)
            for (var i = 0; i < Math.min(x.length, y.length); i++) {
                if (x[i] === y[i]) continue
                var nx = /^\d/.test(x[i]), ny = /^\d/.test(y[i])
                if (nx && ny) return Number(x[i]) - Number(y[i]) || x[i].length - y[i].length
                return x[i] < y[i] ? -1 : 1
            }
            return x.length - y.length
        })
    }
    // From the page's Quick fix the sheet comes filled in; from Fix files the suggestion waits
    // behind Fill in, so a fix of another file never carries it along
    function openFix(suggested, fill) { fixSheet.openFor(page.props.id, page.title, page.fixRows(), suggested || null, !!fill) }
    function rescan() { var r = Door.rescanSeries(page.props.id); r.error ? frame.fail(r.error) : frame.toast(words.t("series-rescan-started")) }
    function load() {
        var r = Door.getSeries(props.id)
        if (r.error) { frame.fail(r.error); return }
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
        // A write a tracker refused says so; the cubes go back to what the tracker holds
        function onProgressSet(series, progress, outcomes) {
            if (series !== page.props.id) return
            reloadDebounce.restart()
            var bad = page.failedOutcomes(outcomes)
            if (bad !== "") frame.toast(bad, 8)
        }
        function onScored(series, score, outcomes) { if (series === page.props.id) { reloadDebounce.restart(); frame.toast(!outcomes.every(function(o) { return o.ok }) ? words.t("series-score-failed") : score < 0 ? words.t("series-rated-cleared") : words.t("series-rated", { score: words.score(score) })) } }
        function onMarked(series, episode, outcomes) {
            if (series !== page.props.id) return
            reloadDebounce.restart()
            var bad = page.failedOutcomes(outcomes)
            if (bad !== "") frame.toast(bad, 8)
        }
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
    // Never a file a torrent is still writing; the core refuses it too, behind every Play
    function openFile(file) {
        for (var i = 0; i < episodes.length; i++) if (episodes[i].file === file && isDownloading(episodes[i])) { frame.toast(words.t("series-still-downloading", { episode: words.episodeName("", episodes[i].number) })); return }
        frame.go("player", { file: file }, page.title)
    }
    function isDownloading(ep) { return ep.downloading !== undefined && ep.downloading !== null }
    function status(s) { return words.releaseStatus(s) || s }
    function listStatus(s) {
        var id = ({ Watching: "status-watching", Planning: "status-planning", Completed: "status-completed", Paused: "status-on-hold", Dropped: "status-dropped", Repeating: "status-rewatching" })[s]
        return id ? words.t(id) : s
    }
    function formatLabel(f) {
        if (isMovie) return words.t("anilist-format-film")
        if (!f) return words.t("series-format-series")
        return words.formatLabel(f) || f.replace(/_/g, " ")
    }
    // AniList's relation types, as a person reads them; one it adds later reads off its code
    function relationLabel(r) {
        if (!r) return ""
        var said = words.relationLabel(r)
        if (said) return said
        var s = String(r).replace(/_/g, " ").toLowerCase(); return s.charAt(0).toUpperCase() + s.slice(1)
    }
    // An episode's name: Fmt.episodeName, the one rule the player and the feed share (its
    // own title, else "Episode 13"; the file name is never a title). A lone film is the film.
    function episodeName(ep) {
        if (!ep.title && isMovie && detail && detail.episodes.length === 1) return page.title
        return words.episodeName(ep.title || "", ep.number)
    }

    // Track to here / untrack to here, optimistic; the core confirms through progressSet.
    // The progress sent to the core is always the floor of ep.number (progress counts
    // whole episodes), but a half-numbered episode (a split cour, a Part 2) then sits on
    // the wrong side of that integral cutoff for its own row: marking 12.5 floors to a
    // target of 12, and 12.5 <= 12 is false, so the row just clicked would not flip.
    // optimisticFile pins the clicked row to the intended state regardless of the number
    // comparison; every other row still reads the integral target normally.
    // "AniList did not take it: ..." for each tracker that refused a write, "" when all did
    function failedOutcomes(outcomes) {
        var names = { Anilist: "AniList", Mal: "MyAnimeList" }
        var list = []
        for (var i = 0; i < outcomes.length; i++) {
            var o = outcomes[i]
            if (o.ok) continue
            list.push(words.t(o.message ? "series-tracker-refused-why" : "series-tracker-refused", { tracker: names[o.tracker] || o.tracker, why: o.message || "" }))
        }
        return list.join("; ")
    }
    function marker(ep) {
        var watched = !!ep.watched
        var target = watched ? Math.max(0, Math.floor(ep.number) - 1) : Math.floor(ep.number)
        var before = card.watched !== null && card.watched !== undefined ? card.watched : -1
        optimisticProgress = target
        // An absent episode has no file; its integral number reads the target on its own
        optimisticFile = ep.file !== undefined ? ep.file : -1
        optimisticWatched = !watched
        // The season goes along for a show with no tracker, whose marks are kept per season
        var id = props.id, season = ep.season || 1
        var r = Door.setProgress(id, target, season)
        if (r.error) { optimisticProgress = -1; optimisticFile = -1; frame.fail(r.error); return }
        // A mark that moves more than one episode says so, with the way back
        if (before >= 0 && Math.abs(target - before) > 1)
            frame.toast(target > before ? words.t("series-watched-up-to", { episode: words.episodeName("", target) }) : words.t("series-unwatched-from", { episode: words.episodeName("", target + 1) }), 8, words.t("series-undo"),
                        function() { Door.setProgress(id, before, season); optimisticProgress = -1; optimisticFile = -1 })
    }
    function watchedWithOptimism(ep) {
        if (optimisticProgress < 0) return !!ep.watched
        if (ep.file !== undefined && ep.file === optimisticFile) return optimisticWatched
        return ep.number <= optimisticProgress
    }

    // ---- What Continue plays: the next episode, else (a film, or caught up) the first
    readonly property var episodes: detail ? detail.episodes : []
    // The rows: the episodes on disk, the aired ones that have no file between them and the
    // ones still to come after, in season then number order
    readonly property var episodeRows: {
        if (!detail) return []
        var absent = (detail.absent || []).map(function(a) {
            return { number: a.number, season: a.season, code: a.code, title: a.title, air_date: a.air_date, cube: a.cube, absent: true, watched: false, next_up: false }
        })
        var ahead = (detail.upcoming || []).map(function(u) {
            return { number: u.number, season: u.season, code: u.code, title: u.title, air_date: u.at, predicted: u.predicted, cube: "f", upcoming: true, watched: false, next_up: false }
        })
        return grouped(episodes).concat(absent, ahead).sort(function(a, b) {
            return ((a.season || 1) - (b.season || 1)) || (a.number - b.number)
        })
    }
    // Two files of one episode (two releases of it) are one row: the row plays the one the
    // core put next up, else the one with a resume point, else the first. Which files are one
    // episode is the core's call, each carrying the same `versions`, so the row can offer the
    // others (#324, #336). While picking files to delete, every file stands on its own row so
    // each can be picked.
    function versionKey(e) {
        var v = e.versions || []
        return v.length > 1 ? "v" + v[0].file : "f" + e.file
    }
    // How many files are second copies of an episode
    readonly property int extraVersions: {
        var keys = {}, n = 0
        for (var i = 0; i < episodes.length; i++) {
            var k = versionKey(episodes[i])
            if (keys[k]) n++
            keys[k] = true
        }
        return n
    }
    function grouped(list) {
        if (page.deleting) return list
        var by = {}, order = []
        for (var i = 0; i < list.length; i++) {
            var k = versionKey(list[i])
            if (!by[k]) { by[k] = []; order.push(k) }
            by[k].push(list[i])
        }
        return order.map(function(k) {
            var all = by[k]
            if (all.length === 1) return all[0]
            // A copy a torrent is still writing never leads while a whole one is there
            var whole = all.filter(function(e) { return !page.isDownloading(e) })
            var lead = all.find(function(e) { return e.next_up }) || whole.find(function(e) { return !!e.resume }) || whole[0] || all[0]
            var row = Object.assign({}, lead)
            row.watched = all.some(function(e) { return e.watched })
            return row
        })
    }
    readonly property var nextEpisode: {
        if (!detail || !detail.next_up) return null
        for (var i = 0; i < episodes.length; i++) if (episodes[i].file === detail.next_up) return isDownloading(episodes[i]) ? null : episodes[i]
        return null
    }
    // The episodes a torrent is still writing: not playable, and never what Continue lands on
    readonly property var downloadingNow: card && card.downloading ? card.downloading : []
    readonly property var firstWhole: {
        for (var i = 0; i < episodes.length; i++) if (!isDownloading(episodes[i])) return episodes[i]
        return null
    }
    readonly property var playEpisode: nextEpisode ? nextEpisode : firstWhole
    // Caught up on what is whole while the next one downloads: Continue says so instead of
    // offering a rewatch
    readonly property bool waitingOnDownload: !nextEpisode && downloadingNow.length > 0 && anyWatched && !isMovie
    readonly property bool tracked: !!detail && detail.progress.watched !== null && detail.progress.watched !== undefined
    readonly property int watchedCount: tracked ? detail.progress.watched : 0
    // Watched by the tracker or here: an untracked series still has its own completed episodes.
    // Read through the rows' optimism so the button flips with them on a marker click.
    readonly property bool anyWatched: watchedCount > 0 || episodes.some(watchedWithOptimism)
    // Every whole episode on disk watched. No next episode alone is not enough: the core walks
    // on from the latest completion, so watching only the finale leaves nothing after it.
    readonly property bool caughtUp: !!detail && !nextEpisode && firstWhole !== null
        && episodes.every(function(e) { return isDownloading(e) || watchedWithOptimism(e) })
    readonly property string playVerb: nextEpisode && anyWatched ? "resume" : caughtUp ? "again" : "play"
    // "Continue episode 8", "Play episode 1", "Watch again from episode 1", "Play" for a lone film
    readonly property string playText: {
        if (waitingOnDownload) {
            var downloading = words.episodeNumber(downloadingNow[0].number)
            return downloading === null ? words.t("series-play-downloading-unnumbered")
                                        : words.t("series-play-downloading", { number: downloading })
        }
        var number = playEpisode ? words.episodeNumber(playEpisode.number) : null
        if (number === null || (isMovie && episodes.length === 1))
            return words.t(({ resume: "series-play-continue", again: "series-play-again", play: "action-play" })[playVerb])
        return words.t(({ resume: "series-play-continue-episode", again: "series-play-again-from", play: "series-play-episode" })[playVerb], { number: number })
    }
    readonly property string progressText: {
        if (!detail) return ""
        if (!tracked) return isMovie ? words.t("series-page-not-tracked") : words.t("series-not-tracked-on-disk", { count: detail.progress.on_disk })
        var p = detail.progress
        if (!p.total) return words.t("series-watched", { count: watchedCount })
        return words.t(p.estimate ? "series-watched-of-estimate" : "series-watched-of", { watched: watchedCount, total: p.total })
    }
    readonly property real progressFraction: tracked && detail.progress.total ? Math.min(1, watchedCount / detail.progress.total) : 0
    readonly property bool airsAhead: !!card.next_airing && card.next_airing.at * 1000 > nowMs
    readonly property bool hasBanner: !!(detail && detail.banner)
    // The art needs the right two thirds to itself; on a narrow band the words reach into
    // them, so the band goes plain
    readonly property bool showArt: hasBanner && band.width >= theme.space(275)
    readonly property bool hasScore: card.my_score !== null && card.my_score !== undefined
    readonly property bool hasAverage: card.community_score !== null && card.community_score !== undefined
    readonly property bool hasImdb: card.imdb_score !== null && card.imdb_score !== undefined
    function listIcon(s) { return { Watching: "play", Repeating: "rotate-ccw", Completed: "circle-check", Paused: "pause", Dropped: "ban", Planning: "bookmark" }[s] || "bookmark" }

    // ---- The facts, one ledger line: format, year, studio, a film's runtime, episodes and how
    // long each runs, on disk, status. A film is a loose file or a MOVIE record in its own
    // folder, the core's `film` flag. The average score, the IMDb score and the
    // list status sit in the quiet row, beside your own score.
    readonly property string factsLine: {
        if (!detail) return ""
        var film = !!card.film
        var f = [formatLabel(card.format)]
        if (detail.year) f.push(String(detail.year))
        if (detail.studio) f.push(detail.studio)
        if (film && detail.episode_minutes) f.push(words.runtime(detail.episode_minutes))
        if (!isMovie && card.total_episodes) f.push(words.t(card.total_is_estimate ? "series-facts-episodes-estimate" : "count-episodes", { count: card.total_episodes }))
        if (!film && detail.episode_minutes) f.push(words.t("series-facts-each", { runtime: words.runtime(detail.episode_minutes) }))
        if (!isMovie) f.push(words.t("series-facts-on-disk", { count: detail.progress.on_disk }))
        if (card.status) f.push(status(card.status))
        return f.join("  ")
    }
    readonly property string episodesFact: {
        if (!detail) return ""
        if (nextEpisode) return words.t("series-next", { episode: words.episodeName("", nextEpisode.number) })
        if (downloadingNow.length > 0) return words.t("series-episodes-downloading", { count: downloadingNow.length })
        if (caughtUp) return words.t("series-all-watched")
        return ""
    }
    // The core hands the synopsis over as plain text in paragraphs, whatever the provider.
    readonly property string synopsisText: detail ? detail.synopsis : ""
    readonly property var tagList: detail ? detail.tags.filter(function(t) { return page.spoilers || (!t.spoiler && !t.adult) }).sort(function(a, b) { return b.rank - a.rank }) : []
    readonly property bool anySpoiler: detail ? detail.tags.some(function(t) { return t.spoiler || t.adult }) : false
    readonly property var relationNodes: graph ? graph.nodes.filter(function(n) { return !n.current }).sort(function(a, b) { return a.y - b.y || a.x - b.x }) : []

    // ---- The band's colour: the poster's tone over a dark base, so white ink holds in
    // both modes; the ink accent is the tone lifted toward white (HomePage's hero). Both
    // are contrast-checked (Tokens.bandBase, bandAccent), so a bright teal poster still
    // reads.
    PosterTone { id: toneOf; source: card.poster ? Fmt.fileUrl(card.poster) : "" }
    readonly property color bandTone: toneOf.tone
    // No poster, no tone: the base stays close to the dark ground
    readonly property color bandBase: theme.bandBase(bandTone, card.poster ? 0.5 : 0.25)
    readonly property color inkAccent: theme.bandAccent(bandTone, bandBase)
    readonly property color ink: "#f4f6f8"
    readonly property color inkDim: Qt.rgba(1, 1, 1, 0.68)
    // The downloading purple, lifted toward white like the accent so it holds on the band
    readonly property color downloadInk: theme.tone(theme.cubeState("t").fill, Qt.rgba(1, 1, 1, 1), 0.3)

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
            Label { id: headingText; text: parent.parent.text; role: "display"; font.pointSize: theme.typeHeading; font.variableAxes: ({ "wdth": 70, "wght": 420 }) }
            Label { visible: parent.parent.count >= 0; anchors.baseline: headingText.baseline; text: String(parent.parent.count); role: "ledger"; color: theme.textFaint }
            Label { anchors.baseline: headingText.baseline; text: parent.parent.fact; role: "ui"; color: theme.textDim }
        }
        Item { id: actionSlot; anchors.right: parent.right; anchors.bottom: parent.bottom; anchors.bottomMargin: theme.space(1); width: childrenRect.width; height: childrenRect.height }
    }
    // A fact in the band's quiet row: a ghost button's size and ink, outlined instead of
    // filled, since it reads rather than acts
    component Fact: Corner {
        id: fact
        property string icon: ""
        property string text: ""
        // A pill that opens a menu: a chevron after the text, and the pointer says so
        property bool menu: false
        signal clicked()
        implicitWidth: factContent.implicitWidth + theme.space(3) * 2
        implicitHeight: theme.space(6)
        radius: height / 2
        smoothing: theme.cornerSmoothing
        color: "transparent"
        borderColor: Qt.rgba(1, 1, 1, 0.22)
        borderWidth: 1
        Row {
            id: factContent
            anchors.centerIn: parent
            spacing: theme.space(1.5)
            Icon { anchors.verticalCenter: parent.verticalCenter; glyph: fact.icon; color: Qt.rgba(1, 1, 1, 0.82); size: theme.space(3.5) }
            Label { anchors.verticalCenter: parent.verticalCenter; text: fact.text; role: "ui"; font.pointSize: theme.typeSmall; color: Qt.rgba(1, 1, 1, 0.82) }
            Icon { visible: fact.menu; anchors.verticalCenter: parent.verticalCenter; glyph: "chevron-down"; color: Qt.rgba(1, 1, 1, 0.82); size: theme.space(3) }
        }
        // The menu pill's own name, for drive scripts to click
        property string pressName: ""
        MouseArea {
            Accessible.role: Accessible.Button
            Accessible.onPressAction: fact.clicked()
            Accessible.name: theme.nameNear(parent)
            objectName: fact.pressName
            anchors.fill: parent
            visible: fact.menu
            cursorShape: Qt.PointingHandCursor
            onClicked: fact.clicked()
        }
    }
    component Caption: Label {
        role: "ui"
        font.pointSize: theme.typeSmall
        color: theme.textDim
    }
    // Delete, at the right of the first section with a file, while nothing is being picked
    component DeleteButton: Button {
        property string where: ""
        objectName: "delete-files-" + where
        visible: page.deleteAt === where && !page.deleting
        small: true; flat: true
        icon: "trash-2"
        msg: "action-delete"
        onClicked: page.startDelete()
    }
    // The phone's delete bar: how many are picked, what deleting does, and the two ways out
    component DeleteBar: Corner {
        id: bar
        property string where: ""
        objectName: "delete-bar-" + where
        visible: page.deleting && page.deleteAt === where
        width: parent ? parent.width : 0
        height: visible ? barCol.implicitHeight + theme.space(4) * 2 : 0
        radius: theme.radiusTile
        smoothing: theme.cornerSmoothing
        color: theme.tile
        borderColor: theme.line
        borderWidth: 1
        Column {
            id: barCol
            x: theme.space(5)
            y: theme.space(4)
            width: parent.width - theme.space(10)
            spacing: theme.space(2)
            Label {
                width: parent.width
                text: page.deletePicked.length === 0 ? words.t("series-delete-click") : words.t("series-delete-picked", page.pickArgs(page.deletePicked.length))
                role: "title"; font.pointSize: theme.typeNormal
            }
            Label {
                width: parent.width
                msg: "series-delete-where"
                role: "ui"; font.pointSize: theme.typeSmall; color: theme.textDim; wrapMode: Text.Wrap
            }
            Label {
                width: parent.width
                visible: page.deleteError !== ""
                text: page.deleteError
                role: "ui"; font.pointSize: theme.typeSmall; color: theme.red; wrapMode: Text.Wrap
            }
            Item { width: 1; height: theme.space(1) }
            Row {
                spacing: theme.space(2)
                Button {
                    objectName: "delete-confirm"
                    danger: true; small: true
                    icon: "trash-2"
                    busy: page.deleteToken >= 0
                    enabled: page.deletePicked.length > 0 && page.deleteToken < 0
                    text: page.deletePicked.length === 0 ? words.t("series-delete-move") : words.t("series-delete-move-picked", page.pickArgs(page.deletePicked.length))
                    onClicked: page.deletePickedFiles()
                }
                // Every file on the page but the ones a torrent is still writing; again to drop them all
                Button {
                    objectName: "delete-all"
                    small: true; flat: true
                    readonly property var all: page.pageFiles.filter(function(f) {
                        return !page.episodes.some(function(e) { return e.file === f && page.isDownloading(e) })
                    })
                    readonly property bool allPicked: all.length > 0 && all.every(function(f) { return page.deletePicked.indexOf(f) >= 0 })
                    text: allPicked ? words.t("series-pick-none") : words.t("series-pick-all")
                    icon: allPicked ? "x" : "check-check"
                    enabled: page.deleteToken < 0
                    onClicked: page.deletePicked = allPicked ? [] : all
                }
                Button { objectName: "delete-cancel"; small: true; flat: true; msg: "action-cancel"; onClicked: { page.deleting = false; page.deleteError = "" } }
            }
        }
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
            height: Math.max(posterH, bandWords.implicitHeight) + pad * 2

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
                        source: page.hasBanner ? Fmt.fileUrl(page.detail.banner) : ""
                        fillMode: Image.PreserveAspectCrop; asynchronous: true
                    }
                }
                // The wash shares the band's rounding, so no square corner of it shows past
                // the band's own; it stays solid until past the art's left edge, and nearly
                // solid under the words that run on over the art, so they never sit on it
                Rectangle {
                    visible: page.showArt
                    anchors.fill: parent
                    radius: theme.radiusXl
                    antialiasing: true
                    gradient: Gradient {
                        orientation: Gradient.Horizontal
                        GradientStop { position: 0.33; color: page.bandBase }
                        GradientStop { position: 0.55; color: Qt.rgba(page.bandBase.r, page.bandBase.g, page.bandBase.b, 0.9) }
                        GradientStop { position: 0.75; color: Qt.rgba(page.bandBase.r, page.bandBase.g, page.bandBase.b, 0.45) }
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
                    source: card.poster ? Fmt.fileUrl(card.poster) : ""
                    sourceSize: Qt.size(theme.posterDecodeSize.width * 2, theme.posterDecodeSize.height * 2)
                    fillMode: Image.PreserveAspectCrop; asynchronous: true
                }
                Label { visible: !card.poster; anchors.centerIn: parent; msg: "series-no-poster"; color: page.inkDim; role: "ui"; font.pointSize: theme.typeSmall }
            }

            Column {
                id: bandWords
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
                // The other titles. One line; when they run past the band, hovering pans it
                // to the end and back at a reading pace, and a click (or Return) unfolds it,
                // a title to a line, wrapped.
                Item {
                    id: aka
                    objectName: "series-other-titles"
                    visible: page.otherTitles.length > 0
                    width: parent.width
                    height: akaView.height
                    property bool open: false
                    readonly property real gap: theme.space(5)
                    readonly property bool overflows: akaRuler.width > width
                    readonly property bool foldable: overflows || open
                    // How far the line pans: its overflow past the viewport
                    readonly property real reach: Math.max(0, akaRuler.width - akaView.width)
                    readonly property bool panning: akaHover.hovered && overflows && !open && !theme.reducedMotion
                    readonly property real speed: theme.space(18)   // px a second
                    property real pan: 0
                    property int dir: 1
                    property real hold: 0
                    onPanningChanged: if (panning) { dir = 1; hold = 0.5 }
                    activeFocusOnTab: foldable
                    Keys.onReturnPressed: if (foldable) open = !open
                    Keys.onSpacePressed: if (foldable) open = !open

                    // The natural one-line width, measured off the same labels
                    Row {
                        id: akaRuler
                        visible: false
                        spacing: aka.gap
                        Repeater { model: page.otherTitles; Label { required property string modelData; text: modelData; role: "ui" } }
                    }
                    // A constant pace while hovered, a hold at either end; let go, it eases home
                    FrameAnimation {
                        running: aka.panning || aka.pan > 0
                        onTriggered: {
                            if (!aka.panning) { aka.pan = theme.follow(aka.pan, 0, frameTime); if (aka.pan < 0.5) aka.pan = 0; return }
                            if (aka.hold > 0) { aka.hold -= frameTime; return }
                            var end = aka.dir > 0 ? aka.reach : 0
                            var step = aka.speed * frameTime
                            aka.pan = aka.dir > 0 ? Math.min(end, aka.pan + step) : Math.max(end, aka.pan - step)
                            if (aka.pan === end) { aka.dir = -aka.dir; aka.hold = 1.2 }
                        }
                    }
                    Item {
                        id: akaView
                        width: aka.width - (akaChevron.visible ? akaChevron.width + theme.space(2) : 0)
                        height: aka.open ? akaFlow.implicitHeight : akaRuler.height
                        clip: true
                        Flow {
                            id: akaFlow
                            x: aka.open ? 0 : -aka.pan
                            width: aka.open ? akaView.width : akaRuler.width + 1
                            spacing: aka.open ? theme.space(1) : aka.gap
                            Repeater {
                                model: page.otherTitles
                                Label {
                                    required property string modelData
                                    // Unfolded, a title to a line
                                    width: aka.open ? akaView.width : implicitWidth
                                    text: modelData
                                    role: "ui"
                                    color: akaHover.hovered && aka.foldable ? page.ink : page.inkDim
                                    wrapMode: aka.open ? Text.Wrap : Text.NoWrap
                                }
                            }
                        }
                    }
                    Icon {
                        id: akaChevron
                        visible: aka.foldable
                        anchors.right: parent.right
                        y: Math.round((akaRuler.height - height) / 2)
                        glyph: aka.open ? "chevron-up" : "chevron-down"
                        size: theme.space(4)
                        color: akaHover.hovered || aka.activeFocus ? page.ink : page.inkDim
                    }
                    Rectangle {
                        visible: aka.activeFocus
                        anchors.fill: parent
                        anchors.margins: -theme.space(1)
                        radius: theme.radiusSm
                        color: "transparent"
                        border.color: theme.focusRing
                        border.width: 2
                    }
                    HoverHandler { id: akaHover; cursorShape: aka.foldable ? Qt.PointingHandCursor : Qt.ArrowCursor }
                    TapHandler { enabled: aka.foldable; onTapped: aka.open = !aka.open }
                }
                // The facts, and the next episode's countdown in the band's ink accent
                Row {
                    width: parent.width
                    spacing: theme.space(2)
                    Label {
                        id: factsLabel
                        width: Math.min(implicitWidth, parent.width - (airing.visible ? airing.width + parent.spacing : 0) - (downloadingFact.visible ? downloadingFact.width + parent.spacing : 0))
                        text: page.factsLine
                        role: "ledger"
                        color: page.inkDim
                        elide: Text.ElideRight
                    }
                    Label {
                        id: airing
                        objectName: "fact-airing"
                        visible: page.airsAhead
                        text: {
                            var next = card.next_airing
                            if (!next) return ""
                            var span = words.episodeSpan(next.episode, next.through || 0)
                            if (next.season && next.season !== 1) span = words.t("series-next-season-span", { season: next.season, span: span })
                            if (next.predicted)
                                return words.t("series-next-predicted", { span: span, date: Qt.formatDate(new Date(next.at * 1000), words.t("series-next-date-format")), wait: words.countdown(next.at - page.nowMs / 1000) })
                            return words.t("series-next-airs", { span: span, count: next.through ? 2 : 1, wait: words.countdownSeconds(next.at - page.nowMs / 1000) })
                        }
                        role: "ledger"
                        font.weight: 600; font.variableAxes: ({ "wdth": 88, "wght": 600 })
                        color: page.inkAccent
                    }
                    Label {
                        id: downloadingFact
                        objectName: "fact-downloading"
                        visible: page.downloadingNow.length > 0
                        text: words.t("series-downloading-episodes", { count: page.downloadingNow.length })
                        role: "ledger"
                        font.weight: 600; font.variableAxes: ({ "wdth": 88, "wght": 600 })
                        color: page.downloadInk
                    }
                }
                // What the user told the season, on one line (#432): the episode count set by hand
                // (#239, a TVmaze show's is its current season's, #283) and a break in the
                // schedule (#418), with Change opening Episodes and dates where both are set
                Row {
                    id: countRow
                    objectName: "episode-count"
                    readonly property var set: card.episodes_set === undefined ? null : card.episodes_set
                    readonly property bool gaveWay: set !== null && card.episodes_set_applied !== true
                    // A TVmaze count is one season's, so the row names it
                    readonly property string countText: set === null || gaveWay ? ""
                        : card.count_season ? words.t("series-count-season-ends", { season: card.count_season, number: set })
                        : words.t("series-count-ends", { number: set })
                    visible: set !== null || page.heldLine !== ""
                    spacing: theme.space(2)
                    Label {
                        objectName: "schedule-held"
                        anchors.verticalCenter: parent.verticalCenter
                        text: countRow.gaveWay && !page.heldLine
                              ? words.t("series-count-gave-way", { count: countRow.set })
                              : words.t("series-count-set-by-you", { line: [countRow.countText, page.heldLine].filter(Boolean).join(" · ") })
                        role: "ledger"
                        color: page.inkDim
                    }
                    Button {
                        objectName: "episode-count-set"
                        small: true; flat: true
                        tint: page.inkAccent
                        icon: "calendar-clock"
                        msg: "series-change"
                        onClicked: page.openSchedule()
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
                        icon: page.caughtUp && !page.waitingOnDownload ? "rotate-ccw" : "play"
                        text: page.playText
                        enabled: page.playEpisode !== null && !page.waitingOnDownload
                        onClicked: if (page.playEpisode) page.openFile(page.playEpisode.file)
                    }
                    // The phone's Get episodes: primary when there is nothing to play
                    Button {
                        objectName: "series-get"
                        visible: !!page.detail && !page.isMovie
                        anchors.verticalCenter: parent.verticalCenter
                        primary: page.playEpisode === null
                        ghost: page.playEpisode !== null
                        tint: page.inkAccent
                        icon: "download"
                        msg: "series-get-episodes"
                        onClicked: page.getEpisodes()
                    }
                    Row {
                        anchors.verticalCenter: parent.verticalCenter
                        spacing: theme.space(3)
                        visible: !!page.detail
                        EpisodeCubes {
                            visible: codes.length > 0
                            anchors.verticalCenter: parent.verticalCenter
                            width: Math.min(implicitWidth, theme.space(70))
                            height: implicitHeight
                            codes: page.card && page.card.cubes ? page.card.cubes : ""
                            cubeSize: theme.space(2.25)
                            maxRows: 2
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
                // The quiet row: your score (rate, on a tracker), the average score, the IMDb
                // score with its votes on hover, the list status and rewatches, then hide, the
                // record's site, and Fixes
                Flow {
                    width: parent.width
                    spacing: theme.space(2)
                    Button {
                        id: rateButton
                        objectName: "fact-mine"
                        // With no tracker behind the series the score stays on this device (#213)
                        visible: !card.hidden
                        small: true; ghost: true
                        icon: "star"
                        text: page.hasScore ? words.t("series-your-score", { score: words.score(card.my_score) }) : words.t("series-rate")
                        onClicked: scorePicker.openAt(rateButton, page.hasScore ? card.my_score : -1)
                    }
                    Fact { objectName: "fact-average"; visible: page.hasAverage; icon: "users"; text: page.hasAverage ? words.t("series-page-average-score", { score: words.score(card.community_score) }) : "" }
                    Fact {
                        objectName: "fact-imdb"
                        visible: page.hasImdb
                        icon: "clapperboard"
                        text: page.hasImdb ? words.t("series-page-imdb", { score: words.score(card.imdb_score) }) : ""
                        Tooltip { text: page.hasImdb && card.imdb_votes ? words.t("series-page-imdb-votes", { votes: Number(card.imdb_votes).toLocaleString(Qt.locale(), "f", 0) }) : "" }
                    }
                    // The list status, and with a tracker connected the menu that changes it
                    Fact {
                        id: statusFact
                        objectName: "fact-list"
                        readonly property bool settable: frame.canSetListStatus(card)
                        visible: !!card.list_status || settable
                        menu: settable
                        icon: page.listIcon(card.list_status)
                        text: card.list_status ? page.listStatus(card.list_status) : words.t("series-add-to-list")
                        pressName: "series-status"
                        onClicked: { var p = mapToItem(frame, 0, height + theme.space(1)); frame.openMenu(p.x, p.y, frame.listStatusItems(page.props.id, card.list_status)) }
                        Tooltip { text: statusFact.settable ? frame.listHome(card) : "" }
                    }
                    Fact { visible: !!page.detail && page.detail.rewatch_count > 0; icon: "rotate-ccw"; text: page.detail && page.detail.rewatch_count > 0 ? words.t("series-page-rewatched", { count: page.detail.rewatch_count }) : "" }
                    // The list entry's notes, written to every connected tracker (#211); a show not
                    // on a list yet has no entry to write them on
                    Button {
                        id: notesButton
                        objectName: "series-notes"
                        visible: statusFact.settable && !!card.list_status
                        small: true; ghost: true
                        icon: "pencil"
                        text: page.detail && page.detail.notes ? words.t("series-your-notes") : words.t("series-add-notes")
                        onClicked: notesSheet.openFor(page.props.id, page.title, page.detail ? page.detail.notes : "", page.trackerSites)
                        Tooltip { text: page.detail && page.detail.notes ? page.detail.notes : words.t("series-notes-tip", { sites: page.trackerSites }) }
                    }
                    Button {
                        objectName: "series-hide"
                        small: true; ghost: true
                        text: card.hidden ? words.t("series-unhide") : words.t("series-hide")
                        icon: card.hidden ? "eye" : "eye-off"
                        onClicked: {
                            var hide = !card.hidden
                            // The id is kept here: Undo can come after the page is gone
                            var id = props.id
                            var r = Door.setHidden(id, hide)
                            if (r.error) { frame.fail(r.error); return }
                            frame.toast(hide ? words.t("series-hidden-toast", { title: page.title }) : words.t("series-unhidden-toast", { title: page.title }), 6, words.t("series-undo"),
                                        function() { Door.setHidden(id, !hide) })
                        }
                        Tooltip { text: page.hasTracker ? words.t("series-hide-tip-tracked") : words.t("series-hide-tip") }
                    }
                    Button { objectName: "series-site"; visible: !!(page.detail && page.detail.site_url); small: true; ghost: true; text: page.siteName; icon: "external-link"; onClicked: frame.openLink(page.detail.site_url) }
                    Button {
                        id: fixesButton
                        objectName: "series-fixes"
                        small: true; ghost: true
                        msg: "series-fixes"; icon: "wrench"
                        onClicked: { var p = mapToItem(frame, 0, height + theme.space(1)); frame.openMenu(p.x, p.y, page.fixItems()) }
                        Tooltip { msg: "series-fixes-tip" }
                    }
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

                // A misreading the core can see, with its fix one click away (#170)
                Corner {
                    objectName: "series-suggested-fix"
                    visible: !!page.detail && !!page.detail.suggested_fix && !page.deleting
                    width: parent.width
                    height: visible ? suggestedRow.implicitHeight + theme.space(3) * 2 : 0
                    radius: theme.radiusTile
                    smoothing: theme.cornerSmoothing
                    color: theme.accentSoft
                    Row {
                        id: suggestedRow
                        x: theme.space(4)
                        width: parent.width - theme.space(8)
                        anchors.verticalCenter: parent.verticalCenter
                        spacing: theme.space(3)
                        Icon { anchors.verticalCenter: parent.verticalCenter; glyph: "sparkles"; size: theme.space(4.5); color: theme.accent }
                        Label {
                            anchors.verticalCenter: parent.verticalCenter
                            width: parent.width - theme.space(7.5) - suggestedFix.width - parent.spacing * 2
                            text: page.detail && page.detail.suggested_fix ? page.detail.suggested_fix.text : ""
                            role: "ui"; color: theme.text; wrapMode: Text.Wrap
                        }
                        Button {
                            id: suggestedFix
                            objectName: "series-suggested-fix-open"
                            anchors.verticalCenter: parent.verticalCenter
                            small: true; primary: true
                            msg: "series-quick-fix"; icon: "wrench"
                            onClicked: page.openFix(page.detail.suggested_fix, true)
                        }
                    }
                }

                // The episodes
                Column {
                    objectName: "series-episodes"
                    visible: page.episodeRows.length > 0
                    width: parent.width
                    spacing: theme.space(4)
                    Heading {
                        text: page.isMovie ? words.t("series-film") : words.t("series-episodes")
                        // Episodes, not files: the same while picking, when each file has a row
                        count: page.episodeRows.filter(function(r) { return !r.upcoming }).length - (page.deleting ? page.extraVersions : 0)
                        fact: page.episodesFact
                        action: DeleteButton { where: "episodes" }
                    }
                    DeleteBar { where: "episodes" }
                    Column {
                        width: parent.width
                        spacing: theme.space(4)
                        Repeater {
                            model: page.episodeRows
                            EpisodeRow {
                                required property var modelData
                                required property int index
                                objectName: "episode-row-" + index
                                episode: modelData
                                hasTracker: page.hasTracker
                                title: page.episodeName(modelData)
                                watched: page.watchedWithOptimism(modelData)
                                versions: page.deleting ? [] : (modelData.versions || [])
                                onPlay: page.openFile(modelData.file)
                                onPlayVersion: function(file) { page.openFile(file) }
                                onMarker: page.marker(modelData)
                                picking: page.deleting
                                picked: page.deletePicked.indexOf(modelData.file) >= 0
                                onPick: page.togglePick(modelData.file)
                                gettable: !page.isMovie
                                onDownload: page.getEpisodes()
                            }
                        }
                    }
                }

                // Openings, endings and the rest, grouped
                Column {
                    visible: !!page.detail && page.detail.extras.length > 0
                    width: parent.width
                    spacing: theme.space(4)
                    Heading { text: words.t("series-extras"); count: page.detail ? page.grouped(page.detail.extras).length : 0; action: DeleteButton { where: "extras" } }
                    DeleteBar { where: "extras" }
                    Repeater {
                        // The extra's kind from the core, and the message that names it
                        model: [["Op", "series-extras-openings"], ["Ed", "series-extras-endings"], ["Pv", "series-extras-previews"], ["Sp", "series-extras-specials"], ["Other", "series-extras-other"]] // i18n: none
                        Column {
                            required property var modelData
                            // Two releases of one extra are one row, as for an episode (#468)
                            readonly property var group: page.detail ? page.grouped(page.detail.extras.filter(function(x) { return x.kind === modelData[0] })) : []
                            visible: group.length > 0
                            width: parent.width
                            spacing: theme.space(2)
                            Caption { text: words.t("series-extras-group", { kind: words.t(modelData[1]), count: group.length }) }
                            Column {
                                width: parent.width
                                spacing: theme.space(4)
                                Repeater {
                                    model: group
                                    EpisodeRow {
                                        required property var modelData
                                        episode: ({ code: modelData.code, title: modelData.label, resume: modelData.resume, watched: false, next_up: false })
                                        title: modelData.label
                                        versions: page.deleting ? [] : (modelData.versions || [])
                                        onPlay: page.openFile(modelData.file)
                                        onPlayVersion: function(file) { page.openFile(file) }
                                        picking: page.deleting
                                        picked: page.deletePicked.indexOf(modelData.file) >= 0
                                        onPick: page.togglePick(modelData.file)
                                    }
                                }
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
                    Heading { text: words.t("series-extra-files"); count: page.detail ? page.detail.unmatched_files.length : 0; action: DeleteButton { where: "stray" } }
                    DeleteBar { where: "stray" }
                    Row {
                        width: parent.width
                        spacing: theme.space(2)
                        Icon { glyph: "triangle-alert"; size: theme.space(4); color: theme.yellow }
                        Label {
                            width: parent.width - theme.space(6)
                            wrapMode: Text.Wrap; role: "ui"; font.pointSize: theme.typeSmall; color: theme.textDim
                            text: page.detail ? words.t("series-extra-files-warning", { count: page.detail.unmatched_files.length, total: card.total_episodes || 0 }) : ""
                        }
                    }
                    Column {
                        width: parent.width
                        spacing: theme.space(4)
                        Repeater { model: page.detail ? page.detail.unmatched_files : []
                            EpisodeRow {
                                required property var modelData
                                episode: modelData; extra: true
                                title: words.episodeName(modelData.title || "", modelData.number)
                                detailText: Fmt.baseName(modelData.path)
                                onPlay: page.openFile(modelData.file)
                                picking: page.deleting
                                picked: page.deletePicked.indexOf(modelData.file) >= 0
                                onPick: page.togglePick(modelData.file)
                            } }
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
                    Heading { text: words.t("series-synopsis") }
                    // Selectable, so a line of it can be copied; folded to eight lines until More
                    TextEdit {
                        id: synopsis
                        width: Math.min(parent.width, theme.space(140))
                        readonly property real lineStep: synopsisMetrics.height * 1.2
                        readonly property real folded: lineStep * 8
                        readonly property bool truncated: implicitHeight > folded + 1
                        height: page.synopsisOpen ? implicitHeight : Math.min(implicitHeight, folded)
                        clip: true
                        readOnly: true
                        selectByMouse: true
                        textFormat: TextEdit.RichText
                        text: "<div style=\"line-height:120%\">" + page.synopsisText.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;").replace(/\n/g, "<br>") + "</div>" // i18n: none
                        color: theme.textDim
                        selectionColor: theme.accentSoft
                        selectedTextColor: theme.text
                        wrapMode: TextEdit.Wrap
                        font.family: theme.roleFamily("ui"); font.pointSize: theme.roleSize("ui"); font.weight: theme.roleWeight("ui")
                        font.variableAxes: theme.roleAxes("ui")
                        FontMetrics { id: synopsisMetrics; font: synopsis.font }
                    }
                    Button { visible: synopsis.truncated || page.synopsisOpen; small: true; flat: true; text: page.synopsisOpen ? words.t("series-less") : words.t("series-more"); icon: page.synopsisOpen ? "chevron-up" : "chevron-down"; onClicked: page.synopsisOpen = !page.synopsisOpen }
                }

                // Unmatched: the match, where the synopsis would be
                Column {
                    objectName: "series-match"
                    visible: !!page.detail && !page.matched
                    width: parent.width
                    spacing: theme.space(3)
                    Heading { text: words.t("series-match") }
                    Label {
                        width: Math.min(parent.width, theme.space(140))
                        msg: "series-not-matched"
                        role: "ui"; color: theme.textDim
                        wrapMode: Text.Wrap
                    }
                    Row {
                        spacing: theme.space(2)
                        Button { objectName: "series-find-match"; msg: "series-find-match"; icon: "search"; primary: true; onClicked: matchPanel.openFor(page.props.id, page.title, card.match_info) }
                        Button { msg: "series-open-in-show-info"; icon: "database"; flat: true; onClicked: frame.go("metadata", { q: page.title }) }
                    }
                    // What a name that matches looks like (#207): the show's title as AniList
                    // writes it, then the season, then the episode
                    Tile {
                        objectName: "series-naming-help"
                        width: Math.min(parent.width, theme.space(140))
                        height: namingCol.implicitHeight + padding * 2
                        padding: theme.space(4)
                        Column {
                            id: namingCol
                            width: parent.width
                            spacing: theme.space(2)
                            Label { msg: "series-naming-title"; role: "title"; font.pointSize: theme.typeNormal }
                            Label { width: parent.width; wrapMode: Text.Wrap; role: "ui"; color: theme.textDim
                                msg: "series-naming-body" }
                            Repeater {
                                // Folder and file names the way the scanner reads them, in every language
                                model: [["Spy x Family Season 2", "Spy x Family S02E01.mkv"], // i18n: none
                                        ["Spy x Family Season 2", "[Group] Spy x Family Season 2 - 01 [1080p].mkv"], // i18n: none
                                        ["Your Name (2016)", "Your Name (2016).mkv"]] // i18n: none
                                Row {
                                    required property var modelData
                                    spacing: theme.space(2)
                                    Icon { anchors.verticalCenter: parent.verticalCenter; glyph: "folder"; size: theme.space(4); color: theme.textFaint }
                                    Label { anchors.verticalCenter: parent.verticalCenter; text: modelData[0] + "/"; role: "ledger"; color: theme.text }
                                    Label { anchors.verticalCenter: parent.verticalCenter; text: modelData[1]; role: "ledger"; color: theme.textDim }
                                }
                            }
                            Button { msg: "series-fix-how-files-read"; icon: "wrench"; flat: true; small: true; onClicked: page.openFix(null, false) }
                        }
                    }
                }

                // The characters, a row of round portraits
                Column {
                    visible: !!page.detail && page.detail.characters.length > 0
                    width: parent.width
                    spacing: theme.space(3)
                    Heading { text: words.t("series-characters"); count: page.detail ? page.detail.characters.length : 0 }
                    Flow { width: parent.width; spacing: theme.space(3); Repeater { model: page.detail ? page.detail.characters : []; PersonCard { required property var modelData; person: modelData } } }
                }

                // Relations: the franchise as small rows, in the crawl's own order
                Column {
                    objectName: "series-relations"
                    visible: page.relationNodes.length > 0
                    width: parent.width
                    spacing: theme.space(3)
                    Heading { text: words.t("series-related"); count: page.relationNodes.length }
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
                    Caption { visible: !!page.graph && !page.graph.complete; msg: "series-related-filling" }
                }

                // Tags by rank; spoiler and adult tags behind the toggle
                Column {
                    visible: !!page.detail && page.detail.tags.length > 0
                    width: parent.width
                    spacing: theme.space(3)
                    Heading {
                        text: words.t("series-tags")
                        count: page.tagList.length
                        action: Row {
                            spacing: theme.space(2)
                            Button { visible: page.anySpoiler; small: true; flat: true; text: page.spoilers ? words.t("series-hide-spoilers") : words.t("series-spoilers"); icon: page.spoilers ? "eye-off" : "eye"; onClicked: page.spoilers = !page.spoilers }
                            Button { visible: tagFlow.implicitHeight > tagClip.height || page.tagsOpen; small: true; flat: true; text: page.tagsOpen ? words.t("series-less") : words.t("series-all-tags"); onClicked: page.tagsOpen = !page.tagsOpen }
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
                                Chip { required property var modelData; text: words.t("series-tag", { name: modelData.name, rank: modelData.rank }); mono: false; color: theme.tile; borderColor: "transparent"; textColor: modelData.spoiler || modelData.adult ? theme.text : theme.textDim }
                            }
                        }
                    }
                }

                // Recommendations last: AniList's top eight, as one even row of eight when the
                // column fits them at a readable width, else 4x2; the cards share the width
                Column {
                    id: recCol
                    objectName: "series-recommendations"
                    visible: !!page.detail && page.detail.recommendations.length > 0
                    width: parent.width
                    spacing: theme.space(3)
                    readonly property real gap: theme.space(3)
                    // AniList keeps up to eight. One row when every card stays readable,
                    // else two even rows; a column too narrow for two rows wraps at the
                    // smallest readable card instead of shrinking past it
                    readonly property int count: page.detail ? page.detail.recommendations.length : 0
                    readonly property int fit: Math.max(1, Math.floor((width + gap) / (theme.space(22) + gap)))
                    readonly property int cols: count < 1 ? 1
                        : (width - (count - 1) * gap) / count >= theme.space(28) ? count
                        : Math.min(Math.ceil(count / 2), fit)
                    readonly property real cardW: Math.max(0, Math.floor((width - (cols - 1) * gap) / cols))
                    Heading { text: words.t("series-recommendations"); count: page.detail ? page.detail.recommendations.length : 0 }
                    Grid { columns: recCol.cols; spacing: recCol.gap
                        Repeater { model: page.detail ? page.detail.recommendations : []
                            RecommendationCard { required property var modelData; rec: modelData; width: recCol.cardW
                                onOpened: modelData.owned ? frame.go("series", { id: modelData.owned }, modelData.title) : frame.openLink("https://anilist.co/anime/" + modelData.anilist_id) } } }
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
                source: rel.node.poster ? Fmt.fileUrl(rel.node.poster) : ""
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
                text: [page.relationLabel(rel.node.relation), page.formatLabel(rel.node.format), rel.node.year ? String(rel.node.year) : ""].filter(function(s) { return s !== "" && s !== words.t("series-format-series") }).join(", ") }
            Row {
                spacing: theme.space(1.5)
                StatusDot { anchors.verticalCenter: parent.verticalCenter; status: rel.node.list_status || "" }
                Label { role: "ledger"; text: rel.owned ? words.t("series-relation-owned") : !rel.node.released ? words.t("series-relation-unreleased") : words.t("series-relation-on-anilist"); color: rel.owned ? theme.accent : theme.textDim }
            }
        }
        MouseArea {
            Accessible.role: Accessible.Button
            Accessible.onPressAction: rel.owned ? frame.go("series", { id: rel.node.owned }, rel.node.title) : (rel.node.site_url ? frame.openLink(rel.node.site_url) : frame.openLink("https://anilist.co/anime/" + rel.node.anilist_id))
            Accessible.name: theme.nameNear(parent)
            id: relMouse
            anchors.fill: parent; hoverEnabled: true; cursorShape: Qt.PointingHandCursor
            onClicked: rel.owned ? frame.go("series", { id: rel.node.owned }, rel.node.title) : (rel.node.site_url ? frame.openLink(rel.node.site_url) : frame.openLink("https://anilist.co/anime/" + rel.node.anilist_id))
        }
    }

    ScorePicker { id: scorePicker; parent: frame.overlay
        // A score kept on this device has no job to report back, so it says so here (#213)
        onSaved: function(v) { var r = Door.setScore(page.props.id, v); if (r.error) frame.fail(r.error); else if (!r.reply || r.reply.job === undefined) frame.toast(words.t("series-score-saved")) }
        onCleared: { var r = Door.setScore(page.props.id, -1); if (r.error) frame.fail(r.error); else if (!r.reply || r.reply.job === undefined) frame.toast(words.t("series-score-cleared")) } }

    // The quick fix (#170), over the overlay like the match
    FixSheet {
        id: fixSheet
        parent: frame.overlay
        anchors.centerIn: parent
    }
    Blocker {
        parent: frame.overlay
        anchors.fill: parent
        z: -1
        visible: fixSheet.visible
        dismissOnPress: true
        onDismissed: fixSheet.close()
    }

    // The list entry's notes (#211)
    NotesSheet {
        id: notesSheet
        parent: frame.overlay
        anchors.centerIn: parent
    }
    Blocker {
        parent: frame.overlay
        anchors.fill: parent
        z: -1
        visible: notesSheet.visible
        dismissOnPress: true
        onDismissed: notesSheet.close()
    }

    // The schedule notes, from Fixes or the held line (#418)
    ScheduleSheet {
        id: scheduleSheet
        parent: frame.overlay
        anchors.centerIn: parent
    }
    Blocker {
        parent: frame.overlay
        anchors.fill: parent
        z: -1
        visible: scheduleSheet.visible
        dismissOnPress: true
        onDismissed: scheduleSheet.close()
    }

    // The match, from Find a match or Fixes: the same popover Metadata opens, over the overlay
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
