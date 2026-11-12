// The frame: the rail, the page area with the status strip at its foot and the drawer
// rising from it, and an overlay for menus and tips. Everything in spec 4.1 unit 1 that
// is not a page lives here. The player takes the whole window and hides the rest.
import QtQuick
import QtCore
import QtQuick.Window
import QtCore
import com.marcusrosado.AniBeam

FocusScope {
    id: frame
    // Named hostWindow, not window: a property named the same as the outer id it is bound
    // from ("window: window" in Main.qml) shadows that id with itself and never resolves,
    // a binding loop with no warning at the call site, only at the property it never sets.
    property Window hostWindow
    readonly property alias nav: nav
    readonly property alias overlay: overlay
    readonly property alias escapeStack: escapeStack
    // Read-only handles for --drive evals; named apart from the ids they alias (see above)
    readonly property alias menuPanel: menu
    readonly property alias drawerPanel: drawer
    readonly property alias sheetPanel: connectSheet
    readonly property alias tipShown: tip.visible
    readonly property Item pageItem: page.item
    readonly property bool fullWindow: page.item ? (page.item.fullWindow === true) : false
    readonly property string windowTitle: (page.item && page.item.title ? page.item.title : nav.current.label) + " - AniBeam"

    Nav { id: nav }
    QtObject {
        id: escapeStack
        // Openers push on open and pop on close, so entries is in opening order and the
        // last one is what the user opened most recently: Escape closes that one first.
        // kind is a record, not a rank: ranking by kind let the drawer outrank a picker
        // opened after it, so Escape closed the drawer and left the picker up.
        property var entries: []
        function push(kind, closer) { pop(closer); entries = entries.concat([{ kind: kind, closer: closer }]) }
        function pop(closer) { entries = entries.filter(function(e) { return e.closer !== closer }) }
        function clear() { entries = [] }
        // The newest entry the user can see. The drawer stays open, and on the stack, under
        // the full-window player, where it is hidden; Escape must not be spent on it there.
        // Checked by kind and fullWindow rather than closer.visible: the drawer's visible
        // also tracks its rise, which is still 0 for the first frame after show().
        function top() {
            for (var i = entries.length - 1; i >= 0; i--) {
                if (frame.fullWindow && entries[i].kind === "drawer") continue
                return entries[i]
            }
            return null
        }
    }
    // Named escapePressed, not escape: QML's compiler reserves "escape" (and unescape, eval,
    // isNaN, isFinite, parseInt, parseFloat, encodeURI, the legacy global JS functions) and
    // refuses it as a property, method or signal name on any object, frame included.
    function escapePressed() {
        var top = escapeStack.top()
        if (top) { top.closer.close(); return }
        if (page.item && page.item.escapePressed && page.item.escapePressed()) return
    }
    Keys.onEscapePressed: escapePressed()

    // Hands key focus back when a layer that took it closes: to `prev` (what had it before
    // the layer opened) while that is still on screen, else to the page. Only when the
    // focus is still inside the layer (or nowhere): a field the user clicked into while
    // the layer was up keeps its focus.
    function focusPage(layer, prev) {
        if (!page.item) return
        var f = frame.hostWindow ? frame.hostWindow.activeFocusItem : null
        for (var p = f; p; p = p.parent) if (p === layer) { f = null; break }
        if (f && f !== frame) return
        if (prev && prev.visible && prev !== frame && prev !== frame.hostWindow.contentItem) prev.forceActiveFocus()
        else page.item.forceActiveFocus()
    }

    // Frame-owned overlays outlive a page. A navigation clears the stack, so they are
    // pushed back after the clear, in the order they were opened.
    function reclaimFrameOwned() {
        var keep = escapeStack.entries.filter(function(e) { return e.closer === drawer || e.closer === connectSheet || e.closer === onboarding })
        escapeStack.clear()
        keep.forEach(function(e) {
            if (e.closer === drawer ? drawer.open : e.closer.visible) escapeStack.push(e.kind, e.closer)
        })
    }

    // Pages by name; a page task swaps its placeholder for the real file
    readonly property var pages: ({
        home: "HomePage.qml", library: "LibraryPage.qml", airing: "AiringPage.qml", feed: "FeedPage.qml", watching: "WatchingPage.qml", stats: "StatsPage.qml", metadata: "MetadataPage.qml",
        downloads: "DownloadsPage.qml", settings: "SettingsPage.qml", series: "SeriesPage.qml", player: "PlayerPage.qml"
    })

    // The settings page's own state: the section open, by its id and kept across launches
    // too, and each section's scroll position by id, kept here rather than on the page itself
    // so they survive the page being torn down and rebuilt on every navigation away and back.
    property alias settingsTab: shellPrefs.settingsTab
    property var settingsScroll: ({})
    // Settings edits not saved yet, by tab, for this run
    property var settingsDrafts: ({})

    function leavingScroll() { return page.item && page.item.scrollY !== undefined ? page.item.scrollY : 0 }
    function go(name, props, label) { nav.open(name, props, label, leavingScroll()) }
    // The next episode of a series in the library, else the series itself when nothing is
    // ready to play
    function playNext(id, title) {
        var r = Door.getSeries(id)
        var d = !r.error ? r.reply.detail : null
        var first = d && d.episodes && d.episodes.length ? d.episodes[0] : null
        var file = d ? (d.next_up ? d.next_up : (d.card.kind === "Movie" && first && (first.downloading === undefined || first.downloading === null) ? first.file : null)) : null
        if (file) go("player", { file: file }, title)
        else go("series", { id: id }, title)
    }
    function back() { nav.back(leavingScroll()) }
    function forward() { nav.forward(leavingScroll()) }

    // Loaded by URL with props as an initial property, not a Component swapped onto a
    // standing Loader, so a page's own Component.onCompleted sees nav.current.props from
    // its very first tick rather than through a later Loader.onLoaded assignment. Clearing
    // source first forces a real rebuild even when navigating between two uses of the same
    // file (this page map has none yet, but a future series-to-series move would).
    //
    // Scheduled through Qt.callLater(frame.loadPageNow), a bound method reference, not an
    // inline closure: Qt.callLater only dedups a repeat call to the very same function, and
    // two `Qt.callLater(function() { ... })` calls each build a fresh, distinct closure, so
    // it never saw them as the same request. The startup `--page X` path fires two
    // navigations in one tick (Frame's own Component.onCompleted opens the default
    // "library", then Main.qml's Loader.onLoaded immediately corrects to the requested
    // page), and with two distinct closures queued, both ran: the destination page was
    // built twice, and the first build was torn down mid-incubation, logging "Object or
    // context destroyed during incubation". A shared method reference lets Qt.callLater
    // collapse the two calls into one, so loadPageNow runs once per tick and still reads
    // nav.current fresh at execution time.
    function loadPage() { if (frame.devOn) devRecorder.begin(nav.current.page, nav.current.label); Qt.callLater(frame.loadPageNow) }
    function loadPageNow() {
        page.source = ""
        page.setSource(Qt.resolvedUrl(frame.pages[nav.current.page] || "PagePlaceholder.qml"), { props: nav.current.props })
        if (frame.devOn) devRecorder.built()
    }

    // ---- Developer options (#287). Off by default in every build: on under ANIBEAM_DEV=1,
    // else behind the switch in Settings, About. The switches live in shell.conf so a
    // measuring session survives a restart.
    Settings {
        id: devKept
        location: Shell.settingsUrl
        category: "developer"
        property bool enabled: false
        property bool images: true
        property bool hud: true
        property bool readCache: true
        property bool posterPool: true
    }
    readonly property alias devKept: devKept
    readonly property alias devRecorder: devRecorder
    readonly property bool devOn: Dev.forced || devKept.enabled
    function devApply() { Dev.configure(frame.devOn, devKept.images, devKept.readCache) }
    function devSetImages(on) { devKept.images = on; devRecorder.images = on; devApply() }
    onDevOnChanged: devApply()
    DevRecorder {
        id: devRecorder
        hostWindow: frame.hostWindow
        hostFrame: frame
        enabled: frame.devOn
        images: devKept.images
        onSetImages: function(on) { frame.devSetImages(on) }
    }
    Connections {
        target: frame.hostWindow
        enabled: frame.devOn
        function onFrameSwapped() { devRecorder.frame() }
    }
    // Start-up marks (ANIBEAM_TIMING=1): the first page's first painted frame
    property bool firstPageMarked: false
    Connections {
        id: firstPaint
        target: frame.hostWindow
        enabled: false
        function onFrameSwapped() { firstPaint.enabled = false; Dev.mark("first page painted") }
    }
    Component.onCompleted: { frame.devApply(); frame.loadPage() }
    // A hovered item destroyed by the navigation never fires its own exit handler, so a
    // tip or a menu opened on the leaving page would otherwise survive onto the next one.
    // The escape stack is cleared for the identical reason: a popover the leaving page
    // owns (a score picker, an inline confirm) is destroyed with it, and any entry it
    // pushed is now a zombie whose closer is gone, so the next escapePressed() would try
    // to call close() on a destroyed object and throw before ever reaching the new page.
    // The drawer and the connect sheet are the exceptions: the frame owns them, they stay
    // open across a navigation (the sheet keeps a typed client id), and
    // reclaimFrameOwned() keeps their entries, in this same handler, so Escape still
    // reaches them; a second Connections on nav.changed would race this one with no
    // ordering guarantee between them.
    Connections { target: nav; function onChanged() { frame.loadPage(); frame.hideTip(); frame.closeMenu(); removeSheet.close(); whatsNew.close(); scanSummary.close(); frame.reclaimFrameOwned() } }

    Rail {
        id: rail
        visible: !frame.fullWindow
        anchors.left: parent.left; anchors.top: parent.top; anchors.bottom: parent.bottom
        active: nav.railIndex
        downloadsBadge: frame.downloadsAttention
        // A rail page opens as it was left; the icon of the page already open takes it back
        // to its top
        onPicked: function(i) {
            if (i !== nav.railIndex || nav.current.page !== nav.railPages[i]) nav.openRail(nav.railPages[i], frame.leavingScroll())
            else if (page.item && page.item.scrollY !== undefined) page.item.scrollY = 0
        }
        onVersionClicked: frame.openWhatsNew()
    }

    Item {
        id: content
        anchors.left: frame.fullWindow ? parent.left : rail.right
        anchors.right: parent.right; anchors.top: parent.top; anchors.bottom: parent.bottom
        // The pages take the whole width so the wheel scrolls them from anywhere; each page
        // centres its own content in a column no wider than theme.contentMax (PageScroll,
        // SettingsTab). The status strip sits under that column; the player takes the window.
        readonly property real column: frame.fullWindow ? width : Math.min(width, theme.contentMax)
        readonly property real columnX: frame.fullWindow ? 0 : Math.round((width - column) / 2)

        Loader {
            id: page
            x: 0
            width: content.width
            anchors.top: parent.top
            anchors.bottom: frame.fullWindow ? parent.bottom : strip.top
            focus: true
            asynchronous: false
            onLoaded: {
                if (!frame.firstPageMarked) { frame.firstPageMarked = true; Dev.mark("first page built: " + nav.current.page); firstPaint.enabled = true }
                if (nav.pendingScroll > 0 && item.scrollY !== undefined) Qt.callLater(function() { if (page.item) page.item.scrollY = nav.pendingScroll })
                item.forceActiveFocus()
                frame.startNotes()
            }
        }

        // Right-click anywhere outside the player: a menu that always offers Back
        MouseArea {
            anchors.fill: page
            enabled: !frame.fullWindow
            acceptedButtons: Qt.RightButton
            propagateComposedEvents: true
            onPressed: function(m) { frame.openContextMenu(mapToItem(frame, m.x, m.y)) }
        }
        // The keyboard's way to the same menu: Shift+F10 or the Menu key, at the focused item
        Shortcut { sequences: ["Shift+F10", "Menu"]; enabled: !frame.fullWindow; onActivated: frame.openContextMenuAtFocus() }

        // The drawer is a floating panel over the page: it lives outside content, above it
        // in z, and rises from the strip's top edge, which is content's bottom edge.
        StatusStrip {
            id: strip
            visible: !frame.fullWindow
            // One band with the rail's foot tile: the same height, the same bottom joint
            height: visible ? rail.footHeight + theme.grout : 0
            x: content.columnX
            width: content.column
            anchors.bottom: parent.bottom
            readonly property var job: Door.runningJobs.length ? Door.runningJobs[0] : null
            // A person reads the plain line; with developer options on, the technical one
            readonly property var line: frame.devOn ? Door.latestLine : Door.latestPlain
            // A kind or stage as words: "ConnectTracker" reads "Connect tracker", "metadata"
            // reads "Metadata"
            function words(k) {
                var w = String(k).replace(/([a-z])([A-Z])/g, "$1 $2").replace(/_/g, " ").toLowerCase()
                return w.charAt(0).toUpperCase() + w.slice(1)
            }
            // Nothing has happened yet this run: it says so, rather than a bare stage chip
            stage: frame.devOn ? (job ? words(job.kind) : (line && line.stage ? words(line.stage) : "Ready")) : ""
            message: job ? (frame.devOn ? (job.label || words(job.kind)) : (job.doing || words(job.kind))) + (job.total > 0 ? ", " + job.done + " of " + job.total : "")
                         : (line ? (frame.devOn ? line.message : line.plain) || "Nothing is running" : "Nothing is running")
            time: job || !line || !line.at ? "" : Qt.formatTime(new Date(line.at * 1000), "hh:mm")
            running: job !== null
            fraction: job && job.total > 0 ? job.done / job.total : 0
            unseenErrors: Door.unseenErrors
            onClicked: frame.toggleDrawer()
        }
    }
    DevHud {
        visible: frame.devOn && devKept.hud && !frame.fullWindow
        recorder: devRecorder
        // Bottom right, over the page's foot: the top right is where pages put search
        anchors.right: content.right; anchors.rightMargin: theme.space(4)
        anchors.bottom: content.bottom; anchors.bottomMargin: strip.height + theme.space(2)
        z: 450
    }
    ActivityDrawer {
        id: drawer
        technical: frame.devOn
        visible: !frame.fullWindow && openness > 0.001
        anchors.left: content.left; anchors.right: content.right; anchors.bottom: content.bottom
        maxHeight: Math.round(page.height * 0.6)
        z: 500
        onClosed: frame.focusPage(drawer)
    }
    function toggleDrawer() { drawer.toggle() }

    // Tracker sign-in. It lives here, not on a page, because the frame is the one thing
    // always loaded: a connect started from the Watching page, the library's nudge or the
    // sheet over either must still open the browser and still hear back after the page
    // that started it is gone. Every surface that shows a connect reads these maps, keyed
    // by tracker ("Anilist" or "Mal"), and none of them listens to Door on its own.
    readonly property string redirectUrl: "http://127.0.0.1:53682/callback"
    property var connecting: ({})      // the running connect job's id
    property var signInUrls: ({})      // the authorize url, to open it again
    property var connectErrors: ({})   // why the last connect did not finish
    // The same sign-in errors as the bridge gave them, for their plain words (connectErrors
    // keeps the core's text, which the client secret check reads)
    property var connectProblems: ({})
    function connectWords(t) {
        // MyAnimeList asking for the app's client secret is a refusal the core words for
        // people, and the row's Add client secret button is its fix: those words stay
        if (needsSecret(t)) return connectErrors[t]
        var e = connectProblems[t]
        return e ? plainText(e) : connectErrors[t] || ""
    }
    function needsSecret(t) { return /client secret/i.test(connectErrors[t] || "") }
    function connectFailed(t, e) {
        connectErrors = withKey(connectErrors, t, errorText(e))
        connectProblems = withKey(connectProblems, t, e)
    }
    property var connectedAs: ({})     // who the last connect signed in as
    // Bookkeeping by job id rather than by tracker, because the core's events are: a late
    // event of a replaced job must never touch the job that replaced it.
    property var jobTrackers: ({})     // job id -> tracker, for every connect started here
    property var latestJob: ({})       // tracker -> the job id of its most recent connect
    property var expectedCancels: ({}) // job id -> true, for the cancels asked for here
    function trackerName(t) { return t === "Mal" ? "MyAnimeList" : "AniList" }
    // A metadata provider as people write it: the core's enum word in, the brand out
    function providerName(p) { return ({ Anilist: "AniList", Mal: "MyAnimeList", Tmdb: "TMDB", Tvmaze: "TVmaze" })[p] || p || "" }
    function trackerAccount(t) { return (t === "Mal" ? Door.trackers.mal : Door.trackers.anilist) || ({}) }
    // Bundled with the build or pasted earlier: either way one click is all a connect needs.
    function hasCredentials(t) { var a = trackerAccount(t); return a.bundled_credentials === true || (a.client_id || "") !== "" }
    // The core runs one sign-in at a time (they share the loopback port), so starting one
    // tracker's would cancel the other's mid-flow. Every Connect asks this first and stands
    // disabled, with the reason as its tip, while the other tracker's browser is out.
    function connectBlockedBy(t) {
        var other = t === "Mal" ? "Anilist" : "Mal"
        return connecting[other] !== undefined ? other : ""
    }
    function blockedTip(t) {
        var other = connectBlockedBy(t)
        return other === "" ? "" : "Finish the " + trackerName(other) + " sign-in first"
    }
    function withKey(map, key, value) {
        var out = {}
        for (var k in map) if (k !== String(key)) out[k] = map[k]
        if (value !== undefined) out[key] = value
        return out
    }
    // The core's error message reads "MyAnimeList did not respond: ..." or "Invalid client
    // id: ...", the shape the log wants; a line that shows it inline wants the sentence
    // after it. A toast goes through fail() instead, which has the plain words and a fix.
    function errorText(e) {
        var m = String(e && e.message ? e.message : "")
        m = m.replace(/^(AniList|MyAnimeList|TMDB|Anilist|Mal|Jikan|Tmdb)( did not respond)?: /, "").replace(/^(io|internal|[Ii]nvalid [a-z_ ]+): /, "")
        return m === "" ? "That did not work." : m.charAt(0).toUpperCase() + m.slice(1)
    }
    // A call error on one line of a page: the bridge's plain words and what to do next, else
    // the core's message trimmed for a person
    function plainText(e) {
        if (e && e.plain) return e.plain + (e.next ? " " + e.next : "")
        return errorText(e)
    }
    // A call error as a toast: what happened and what to do, the button that does it
    // (`retry` runs the same thing again, a settings fix opens that tab), and the core's
    // own message one click away under Details.
    function fail(e, retry) {
        if (!e) return
        var fix = String(e.fix || ""), label = "", act = null
        if (fix === "retry" && retry) { label = "Try again"; act = retry }
        else if (fix.indexOf("settings:") === 0) {
            var tab = fix.slice(9)
            label = "Open settings"; act = function() { frame.go("settings", { tab: tab }) }
        }
        var said = e.plain ? e.plain + (e.next ? " " + e.next : "") : errorText(e), raw = String(e.message || "")
        frame.toast(said, 8, label, act, raw !== "" && raw !== said ? raw : "")
    }
    // Starts the flow; the browser opens when the core says the listener is up. True when
    // a job started, false with the reason in connectErrors when the call was refused or
    // the other tracker's sign-in is still out.
    function connectTracker(t) {
        if (connectBlockedBy(t) !== "") { toast(blockedTip(t)); return false }
        connectErrors = withKey(connectErrors, t)
        connectProblems = withKey(connectProblems, t)
        connectedAs = withKey(connectedAs, t)
        signInUrls = withKey(signInUrls, t)
        // A second connect of the same tracker replaces the first: the core cancels it,
        // and that cancel was asked for.
        if (connecting[t] !== undefined) expectedCancels = withKey(expectedCancels, connecting[t], true)
        var r = Door.connectTracker(t)
        if (r.error) { connectFailed(t, r.error); return false }
        moveOffers = withKey(moveOffers, t, true)
        connecting = withKey(connecting, t, r.reply.job)
        jobTrackers = withKey(jobTrackers, r.reply.job, t)
        latestJob = withKey(latestJob, t, r.reply.job)
        return true
    }
    function cancelConnect(t) {
        if (connecting[t] === undefined) return
        expectedCancels = withKey(expectedCancels, connecting[t], true)
        Door.cancelJob(connecting[t])
    }
    // Every link in the shell: the desktop's handler through xdg-open (Shell.openUrl), with
    // a toast when it could not start. The file manager for a local path the same way.
    function openLink(url) {
        var u = String(url || "")
        if (u === "") return
        if (!Shell.openUrl(u)) frame.toast("AniBeam could not open that. Copy it and open it yourself.", 8, "Copy", function() { frame.copyText(u, "Copied") }, u)
    }
    function reopenSignIn(t) { if (signInUrls[t]) frame.openLink(signInUrls[t]) }
    // One click: with credentials the browser opens at once and the sheet only says where
    // to finish; without them the sheet walks the set-up first.
    function openConnect(t) {
        t = t || "Anilist"
        if (connectBlockedBy(t) !== "") { toast(blockedTip(t)); return }
        if (hasCredentials(t) && connecting[t] === undefined) connectTracker(t)
        connectSheet.openFor(t)
    }
    Connections {
        target: Door
        // A status write that a tracker refused says so, with the tracker's own reason
        function onListStatusSet(series, status, outcomes) {
            var names = { Anilist: "AniList", Mal: "MyAnimeList" }
            var failed = outcomes.filter(function(o) { return !o.ok }).map(function(o) {
                return (names[o.tracker] || o.tracker) + " did not take it" + (o.message ? ": " + o.message : "")
            })
            if (failed.length) frame.toast(failed.join("; "))
        }
        // A connection's first read list is when the list kept here is offered to it (#580).
        // The offer is made when the connect starts, since the read can finish before the
        // connect's own end arrives, and one tracker's offer never takes another's place.
        function onProgressRefreshed(t) {
            var key = String(t).toLowerCase() === "mal" ? "Mal" : "Anilist"
            if (frame.moveOffers[key] !== true) return
            frame.moveOffers = frame.withKey(frame.moveOffers, key)
            moveListsSheet.openFor(key)
        }
        function onListsMoved(t, moved, failed) {
            if (failed > 0) frame.toast(frame.trackerName(t) + " did not take " + (failed === 1 ? "1 show" : failed + " shows") + "; see the activity log")
            else if (moved > 0) frame.toast("Moved " + (moved === 1 ? "1 show" : moved + " shows") + " to " + frame.trackerName(t))
        }
        // Notes a tracker refused say so the same way; saved ones say where they went
        function onNotesSet(series, outcomes) {
            var names = { Anilist: "AniList", Mal: "MyAnimeList" }
            var failed = outcomes.filter(function(o) { return !o.ok }).map(function(o) {
                return (names[o.tracker] || o.tracker) + " did not take your notes" + (o.message ? ": " + o.message : "")
            })
            if (failed.length) frame.toast(failed.join("; "))
            else if (outcomes.length) frame.toast("Notes saved")
        }
        // Only the url of the job this tracker is waiting on: one from a flow since replaced
        // would open a sign-in page whose listener is already gone.
        function onAuthUrlReady(t, openUrl, redirectUrl, job) {
            if (frame.connecting[t] !== job) return
            frame.signInUrls = frame.withKey(frame.signInUrls, t, openUrl)
            // A capture run draws the waiting state; it never opens the owner's browser.
            if (Shell.shoot === "" && Shell.driveOut === "") frame.openLink(openUrl)
        }
        // Door sends this ahead of the body's own signal for every terminal event, so it is
        // where a finished job stops being the one a tracker waits on, by id: a replaced
        // job's late end cannot clear the job that replaced it.
        function onJobFinished(job, kind, ok) {
            if (kind !== "ConnectTracker") return
            var t = frame.jobTrackers[job]
            if (t === undefined) return
            if (frame.connecting[t] === job) frame.connecting = frame.withKey(frame.connecting, t)
            if (ok) frame.jobTrackers = frame.withKey(frame.jobTrackers, job)
        }
        // The toast, the receipt and the raise; the bookkeeping is onJobFinished's.
        function onTrackerConnected(t, username) {
            frame.connectedAs = frame.withKey(frame.connectedAs, t, username || frame.trackerName(t))
            frame.toast("Connected to " + frame.trackerName(t) + " as " + (username || "?"))
            // The user is in the browser tab that just closed; bring them back to the app.
            if (frame.hostWindow) Shell.raiseWindow(frame.hostWindow, "")
        }
        function onJobFailed(job, kind, error) {
            if (kind !== "ConnectTracker") return
            var t = frame.jobTrackers[job]
            if (t === undefined) return
            frame.jobTrackers = frame.withKey(frame.jobTrackers, job)
            if (frame.latestJob[t] !== job) return
            // A connect that failed has nothing to offer the list kept here to
            frame.moveOffers = frame.withKey(frame.moveOffers, t)
            frame.connectFailed(t, error)
            // The sheet says it inline when it is up for this tracker; a toast as well would
            // be the same sentence twice.
            if (!connectSheet.visible || connectSheet.tracker !== t) {
                // Asking again cannot help a missing secret: the row's button is the fix
                if (frame.needsSecret(t)) frame.toast(frame.connectWords(t), 8)
                else frame.fail(error, function() { frame.connectTracker(t) })
            }
        }
        // A cancel nobody here asked for is a sign-in that stopped under the user. Nothing in
        // the shell starts a connect behind the frame's back today, so this is the guard for
        // the day something does and replaces this one: the user is told, not left waiting.
        function onJobCancelled(job, kind) {
            if (kind !== "ConnectTracker") return
            var t = frame.jobTrackers[job]
            if (t === undefined) return
            frame.jobTrackers = frame.withKey(frame.jobTrackers, job)
            var asked = frame.expectedCancels[job] === true
            frame.expectedCancels = frame.withKey(frame.expectedCancels, job)
            if (!asked && frame.latestJob[t] === job) frame.toast("The " + frame.trackerName(t) + " sign-in stopped before it finished.", 8, "Try again", function() { frame.connectTracker(t) })
        }
    }

    // The first run (#193): over the whole frame, the rail included, and under the overlay,
    // so the connect sheet, toasts and tips still draw over it.
    Onboarding {
        id: onboarding
        z: 550
    }
    function openOnboarding(step) { whatsNew.close(); onboarding.open(step) }
    readonly property alias onboardingPanel: onboarding

    // The tracker connect sheet, over everything: the Watching page's empty
    // list, the library's nudge and the Trackers panel all send the user here.
    ConnectSheet {
        id: connectSheet
        objectName: "connect-sheet"
        parent: overlay
        z: 10
    }

    // Overlay: menus, tips, toasts, sheets. Above the drawer (z 500), so a menu or a
    // sheet opened over the drawer's region draws over it, not under it.
    Item {
        id: overlay
        anchors.fill: parent
        z: 600
        Menu { id: menu }
        RemoveSheet { id: removeSheet }
        MoveListsSheet { id: moveListsSheet }
        WhatsNew { id: whatsNew }
        ScanSummary { id: scanSummary }
        UpdateNotice {
            id: updateNotice
            covered: frame.fullWindow || whatsNew.visible || removeSheet.visible || connectSheet.visible || onboarding.visible || scanSummary.visible
            anchors.right: parent.right; anchors.rightMargin: theme.space(4)
            anchors.bottom: parent.bottom; anchors.bottomMargin: rail.footHeight + theme.space(4)
        }
        Frost {
            id: tip
            visible: false
            radius: theme.radiusTile
            width: tipText.implicitWidth + theme.space(4); height: tipText.implicitHeight + theme.space(2)
            Label { id: tipText; anchors.centerIn: parent; role: "ui"; font.pointSize: theme.typeSmall }
        }
        // A toast: one line, an optional action (Undo), and an x. The pointer on it holds it,
        // so a long line is never taken away mid-read
        Frost {
            id: toastBox
            objectName: "frame-toast"
            visible: false
            radius: theme.radiusTile
            anchors.horizontalCenter: parent.horizontalCenter
            anchors.bottom: parent.bottom; anchors.bottomMargin: theme.space(10)
            width: Math.min(parent.width - theme.space(8), toastRow.implicitWidth + theme.space(4))
            height: toastCol.implicitHeight + theme.space(1)
            property var action: null
            property string detail: ""
            property bool detailOpen: false
            HoverHandler { id: toastHover }
            Column {
                id: toastCol
                anchors.centerIn: parent
                width: toastRow.implicitWidth
            Row {
                id: toastRow
                spacing: theme.space(2)
                height: theme.controlHeight
                Label {
                    id: toastText
                    anchors.verticalCenter: parent.verticalCenter
                    leftPadding: theme.space(2)
                    width: Math.min(implicitWidth, toastBox.parent.width - theme.space(toastBox.detail !== "" ? 60 : 40))
                    elide: Text.ElideRight
                    role: "ui"
                }
                Button {
                    id: toastAct
                    objectName: "frame-toast-action"
                    anchors.verticalCenter: parent.verticalCenter
                    visible: text !== ""
                    small: true; flat: true; takesFocus: false
                    onClicked: { var a = toastBox.action; toastBox.visible = false; if (a) a() }
                }
                Button {
                    objectName: "frame-toast-details"
                    anchors.verticalCenter: parent.verticalCenter
                    visible: toastBox.detail !== ""
                    text: toastBox.detailOpen ? "Hide details" : "Details"
                    small: true; flat: true; takesFocus: false
                    onClicked: {
                        toastBox.detailOpen = !toastBox.detailOpen
                        if (toastBox.detailOpen) frame.copyQuiet(toastBox.detail)
                    }
                    Tooltip { text: toastBox.detailOpen ? "" : "Show what went wrong, word for word, and copy it" }
                }
                Button {
                    objectName: "frame-toast-close"
                    anchors.verticalCenter: parent.verticalCenter
                    small: true; flat: true; takesFocus: false
                    icon: "x"
                    onClicked: toastBox.visible = false
                    Tooltip { text: "Close" }
                }
            }
                // The raw text, copied on open, so a report can carry it word for word
                Label {
                    objectName: "frame-toast-detail"
                    visible: toastBox.detailOpen
                    width: parent.width
                    leftPadding: theme.space(2); rightPadding: theme.space(2); bottomPadding: theme.space(2)
                    text: toastBox.detail + "\nCopied."
                    role: "ledger"; color: theme.textDim
                    wrapMode: Text.Wrap
                }
            }
            Timer { id: toastTimer; onTriggered: if (toastHover.hovered) restart(); else toastBox.visible = false }
        }
    }
    // What's new: once the first page is up, so the page taking focus as it loads does not
    // take the keys from the sheet. Never in a capture run, which draws what it was asked to;
    // a drive opens it with frame.openWhatsNew(). The check for a newer release goes out
    // at the same moment, under the same rule; a drive shows it with updatePanel.show().
    property bool notesStarted: false
    function startNotes() {
        if (notesStarted) return
        notesStarted = true
        // Before What's new, which keeps a fresh install's version as read: a fresh install
        // gets the first run instead, an older one only has it marked done
        onboarding.start()
        frame.noteSources()
        if (Shell.shoot === "" && Shell.driveOut === "") { whatsNew.start(); updateNotice.check() }
    }
    function openWhatsNew() { whatsNew.open() }

    // ---- The first scan of a folder added while the app runs (#206): once its scan and the
    // lookup after it settle, the summary says what it found. Folders known at start are the
    // baseline, so a launch never summarises a library it already had.
    property var knownSources: null
    property var firstScans: ({})   // by source id: { path, scanned }
    readonly property alias scanSummaryPanel: scanSummary
    function noteSources() {
        var r = Door.listSources()
        if (r.error) return
        var ids = {}, add = Object.assign({}, frame.firstScans), fresh = false
        r.reply.sources.forEach(function(src) {
            ids[src.id] = true
            if (frame.knownSources !== null && !frame.knownSources[src.id]) { add[src.id] = { path: src.path, scanned: false }; fresh = true }
        })
        for (var k in add) if (!ids[k]) { delete add[k]; fresh = true }
        frame.knownSources = ids
        if (fresh) frame.firstScans = add
    }
    function scanBusy() {
        var jobs = Door.runningJobs
        for (var i = 0; i < jobs.length; i++) if (/^(Scan|AutoMatch)$/.test(String(jobs[i].kind))) return true
        return false
    }
    function settleFirstScans() {
        var ready = Object.keys(frame.firstScans).filter(function(k) { return frame.firstScans[k].scanned })
        if (!ready.length) return
        // Still working, or the first-run walkthrough is up and shows the same result itself
        var ob = frame["onboardingPanel"]
        if (frame.scanBusy() || (ob && ob.visible) || scanSummary.visible || whatsNew.visible) { firstScanSettle.restart(); return }
        var left = Object.assign({}, frame.firstScans)
        var id = ready[0], path = left[id].path
        delete left[id]
        frame.firstScans = left
        var sum = Door.scanSummary(path)
        if (!sum.error && sum.shows > 0) scanSummary.open(path, sum)
        if (ready.length > 1) firstScanSettle.restart()
    }
    // After a scan the lookup starts once the folder settles, a few seconds on; this waits that
    // out, and AutoMatchFinished shortens it
    Timer { id: firstScanSettle; interval: 8000; onTriggered: frame.settleFirstScans() }
    Connections {
        target: Door
        function onSourceChanged(src) { frame.noteSources() }
        // Any scan's end counts, not only one naming the new folder: a folder added while a
        // scan runs is queued onto that job, whose ScanFinished names its first pass's source.
        // The settle step still waits for every Scan and AutoMatch job, so no folder is
        // summarised before its own pass has run.
        function onScanFinished(source, added, changed, removed) {
            var keys = Object.keys(frame.firstScans)
            if (!keys.length) return
            var left = {}
            keys.forEach(function(k) { left[k] = { path: frame.firstScans[k].path, scanned: true } })
            frame.firstScans = left
            firstScanSettle.interval = 8000
            firstScanSettle.restart()
        }
        function onAutoMatchFinished(backfilled, matched, unmatched) {
            if (!Object.keys(frame.firstScans).length) return
            firstScanSettle.interval = 1500
            firstScanSettle.restart()
        }
    }
    // The shell's own choices that more than one page reads, kept in shell.conf. One object
    // for all of them: two Settings on one file do not see each other's writes until they sync.
    Settings {
        id: shellPrefs
        location: Shell.settingsUrl
        category: "shell"
        // The auto-next pill names the next episode; off, it says only its number
        property bool nextNamesEpisode: true
        // The Settings section open last, by id; an older number reads as no section
        property string settingsTab: "library"
        // Downloads comes back with its last search
        property bool keepDownloadsQuery: true
    }
    // The last Downloads search, for this run only
    property string downloadsQuery: ""
    readonly property alias prefs: shellPrefs
    KeyHelp { id: frameKeys; parent: overlay }
    readonly property alias notesPanel: whatsNew
    readonly property alias updatePanel: updateNotice

    // The right-click menu at a point: Back and Forward, then what is under the pointer (an
    // episode, a card), the page's own rows, and deleting a card's files last
    function openContextMenu(p) {
        var items = [{ split: [{ text: "Back", icon: "arrow-left", action: frame.back, enabled: nav.canBack },
                               { text: "Forward", icon: "arrow-right", action: frame.forward, enabled: nav.canForward, trailing: true }] }]
        // Each kind of row is its own group, with a line between groups
        function group(list) {
            if (!list || !list.length) return
            list[0].gapBefore = true
            items = items.concat(list)
        }
        group(frame.releaseItems())
        group(frame.episodeItems())
        group(frame.seriesCardItems())
        if (page.item && page.item.contextItems) group(page.item.contextItems())
        group(frame.seriesDeleteItems())
        frame.openMenu(p.x, p.y, items)
    }
    // From the keys: at the focused item, or the middle of the page when nothing has focus
    function openContextMenuAtFocus() {
        var f = frame.hostWindow ? frame.hostWindow.activeFocusItem : null
        var p = f && f !== frame ? f.mapToItem(frame, theme.space(4), Math.min(f.height, theme.space(10))) : Qt.point(frame.width / 2, frame.height / 3)
        openContextMenu(p)
    }
    // `above`: (x, y) is the menu's bottom right corner, for an opener at the window's foot.
    function openMenu(x, y, items, above) { menu.openAt(x, y, items, above) }
    function closeMenu() { menu.close() }
    // A tip belongs to the area the pointer is over (`area`, the anchor item itself unless a
    // caller anchors the tip on part of a bigger hover area, the way Card anchors on its
    // title). It is refused when the pointer is not inside that area by the time the
    // 600 ms intent fires, and it goes as soon as the area is destroyed (a wall's Loader
    // deactivating the tile, a page change), hidden, moved (a scroll, a re-layout after a
    // search) or left by the pointer: a tip naming a title not under the cursor is a lie.
    property Item tipArea: null
    property point tipOrigin: Qt.point(0, 0)
    onTipAreaChanged: if (!tipArea && tip.visible) tip.visible = false
    HoverHandler { id: pointerWatch }
    function pointerInside(area) {
        if (!area || !pointerWatch.hovered || !area.visible) return false
        var p = area.mapFromItem(frame, pointerWatch.point.position.x, pointerWatch.point.position.y)
        return p.x >= 0 && p.y >= 0 && p.x <= area.width && p.y <= area.height
    }
    // Where the area is, read at its centre: a hover lift scales a card about its centre, so
    // the centre holds still while it grows and only a real move or scroll shifts it (#198)
    function tipCentre(area) { return area.mapToItem(overlay, area.width / 2, area.height / 2) }
    // A tip shown for keyboard focus stays while the item has focus, wherever the pointer is
    property bool tipByFocus: false
    function tipStillTrue() {
        if (!tipArea) return false
        if (tipByFocus ? !tipArea.activeFocus : !pointerInside(tipArea)) return false
        var o = tipCentre(tipArea)
        return Math.abs(o.x - tipOrigin.x) < 1 && Math.abs(o.y - tipOrigin.y) < 1
    }
    // A tip whose area moved or went goes at once. One the pointer only left waits out
    // tipGrace first: crossing the gap to the next item of a row keeps a tip on screen, and
    // that item's tip takes its place in the same spot instead of blinking off and back.
    Timer {
        interval: 100; repeat: true; running: tip.visible
        onTriggered: {
            if (frame.tipStillTrue()) { tipGrace.stop(); return }
            if (!frame.tipArea || !frame.tipArea.visible) { frame.hideTip(); return }
            var o = frame.tipCentre(frame.tipArea)
            if (Math.abs(o.x - frame.tipOrigin.x) >= 1 || Math.abs(o.y - frame.tipOrigin.y) >= 1) frame.hideTip()
            else if (!tipGrace.running) tipGrace.start()
        }
    }
    Timer { id: tipRetry; interval: 32; property var ask: null; onTriggered: if (ask) frame.showTip(ask.item, ask.text, ask.area, true) }
    Timer { id: tipGrace; interval: 150; onTriggered: if (!frame.tipStillTrue()) frame.hideTip() }
    // A hover leaving `area`: its tip goes after the grace, unless another item's tip has
    // already taken over (the next item's hover can arrive before this one's leave)
    function releaseTip(area) { if (tip.visible && tipArea === area && !tipGrace.running) tipGrace.start() }
    // The item's own hover can reach it before the frame's pointerWatch has seen the same
    // move, so a tip refused for a pointer not yet inside is asked again once, a frame on
    // `byFocus`: the item took keyboard focus, so where the pointer is does not matter
    function showTip(item, text, area, retried, byFocus) {
        area = area || item
        if (!byFocus) {
            if (item && !retried && !frame.pointerInside(area)) { tipRetry.ask = { item: item, text: text, area: area }; tipRetry.restart(); return }
            if (!item || !frame.pointerInside(area)) { frame.hideTip(); return }
        }
        tipRetry.stop()
        tipGrace.stop()
        tipArea = area
        tipOrigin = tipCentre(area)
        tipByFocus = !!byFocus
        var p = item.mapToItem(overlay, item.width / 2, item.height)
        tipText.text = text
        tip.x = Math.max(theme.space(2), Math.min(p.x - tip.width / 2, overlay.width - tip.width - theme.space(2)))
        // Below the item, or above it when there is no room below (the player's controls
        // island sits at the window's foot); never over the item it describes. Inside the
        // player's controls tile, above the whole tile, so the tip never covers the seek bar.
        var top = item.mapToItem(overlay, 0, 0).y, inTile = false
        for (var a = item; a; a = a.parent) if (a.objectName === "player-controls") { top = a.mapToItem(overlay, 0, 0).y; inTile = true; break }
        var below = p.y + theme.space(1)
        tip.y = !inTile && below + tip.height <= overlay.height - theme.space(2) ? below
              : Math.max(theme.space(2), top - tip.height - theme.space(1))
        tip.visible = true
    }
    function hideTip() { tipGrace.stop(); tipRetry.stop(); if (tip.visible) tipWarmUntil = Date.now() + 1500; tip.visible = false; tipArea = null }
    // Someone who waited for one tip is reading tips: for a moment after one goes, the next
    // item's tip comes at once instead of after the hover intent
    property real tipWarmUntil: 0
    function tipWarm() { return tip.visible || Date.now() < tipWarmUntil }
    // `actionText` and `action` add a button, Undo for one; it closes the toast and runs.
    // `detail` adds Details, which opens the raw text under the line and copies it.
    function toast(text, seconds, actionText, action, detail) {
        toastBox.detail = detail || ""
        toastBox.detailOpen = false
        toastText.text = text
        toastAct.text = actionText || ""
        toastBox.action = action || null
        toastBox.visible = true
        toastTimer.interval = (seconds || (action ? 8 : 4)) * 1000
        toastTimer.restart()
    }

    // ---- The removal sheet: every removal on the downloads page asks through it
    readonly property bool removalOpen: removeSheet.visible
    function askRemoval(anchor, spec, done) { removeSheet.openAt(anchor, spec, done) }

    // ---- Deleting a series' files from its card (#236). A card or a week row under the
    // pointer names itself here, and the right-click menu then offers its watched episodes
    // or all of them, to the trash, through the removal sheet. Only a library series with
    // files on disk offers it.
    property var hoverSeries: null
    property Item hoverSeriesItem: null
    function hoverCard(item, data, on) {
        if (on) { hoverSeriesItem = item; hoverSeries = data }
        else if (hoverSeriesItem === item) { hoverSeriesItem = null; hoverSeries = null }
    }
    // The menu rows come off the card alone; the files are read only once a row is picked
    function seriesNoun(c) { return c.kind === "Movie" ? ["film", "films"] : ["episode", "episodes"] }
    // The episode row under the pointer, for its own rows in the right-click menu
    property var hoverEpisodeRow: null
    function hoverEpisode(row, on) {
        if (on) hoverEpisodeRow = row
        else if (hoverEpisodeRow === row) hoverEpisodeRow = null
    }
    // The release row under the pointer, on Downloads
    property var hoverReleaseItem: null
    function hoverRelease(item, on) {
        if (on) hoverReleaseItem = item
        else if (hoverReleaseItem === item) hoverReleaseItem = null
    }
    function releaseItems() {
        var r = hoverReleaseItem
        if (!r || !r.link) return []
        var magnet = String(r.link).indexOf("magnet:") === 0
        var out = [{ text: magnet ? "Copy magnet" : "Copy torrent link", icon: "link", action: function() { frame.copyText(r.link, magnet ? "Magnet copied" : "Link copied") } }]
        if (r.page) out.push({ text: "Open release page", icon: "external-link", action: function() { frame.openLink(r.page) } })
        return out
    }
    function episodeItems() {
        var row = hoverEpisodeRow
        if (!row || !row.episode || !row.episode.path) return []
        var path = row.episode.path
        var out = []
        if (row.playable && !row.picking) out.push({ text: "Play", icon: "play", action: function() { row.play() } })
        out.push({ text: "Open folder", icon: "folder-open", action: function() { frame.openLink(Fmt.fileUrl(path.replace(/[\\/][^\\/]*$/, ""))) } })
        out.push({ text: "Copy path", icon: "copy", action: function() { frame.copyText(path, "Path copied") } })
        if (row.trackable && !row.picking) out.push({ text: row.watched ? "Mark unwatched from here" : "Mark watched up to here", icon: "check-check", action: function() { row.marker() } })
        return out
    }
    // A card's own rows, above the delete ones: open it, its folder, its title, its page
    function seriesCardItems() {
        var c = hoverSeries
        // A card with no folder is not the library's (Airing, a Watching ghost): its id is not ours
        if (!c || !c.path || c.id === undefined || c.id === null) return []
        var out = [{ text: "Open", icon: "layers", action: function() { frame.go("series", { id: c.id }, c.title) } },
                   { text: "Open folder", icon: "folder-open", action: function() { frame.openLink(Fmt.fileUrl(c.path)) } }]
        out.push({ text: "Copy title", icon: "copy", action: function() { frame.copyText(c.title) } })
        var site = c.site_url || (c.match_info && c.match_info.anilist_id ? "https://anilist.co/anime/" + c.match_info.anilist_id : "")
        if (site) out.push({ text: "Open on AniList", icon: "external-link", action: function() { frame.openLink(site) } })
        if (frame.canSetListStatus(c)) {
            // The lists are a group of their own in the menu
            var lists = frame.listStatusItems(c.id, c.list_status)
            lists[0].gapBefore = true
            out = out.concat(lists)
        }
        return out
    }
    // A list status is offered for every series but a hidden one, whose tracker sync is off. It
    // goes to every connected tracker that carries an id for the series, and stays on this
    // device when none does (#213).
    // The trackers a fresh connection will offer the list kept here to, once their list is read
    property var moveOffers: ({})
    // The move sheet for a tracker, from Settings; answers whether there was anything to move
    function openMoveLists(t) { return moveListsSheet.openFor(t) }
    // A series AniBeam put a name to: any provider id at all. Show info's sections and Help's
    // count both read it, so the two never disagree.
    function hasMatch(card) {
        return !!card && !!card.match_info && ["anilist_id", "mal_id", "tvmaze_id", "tmdb_id"].some(function(k) { return card.match_info[k] !== null && card.match_info[k] !== undefined })
    }
    function trackerOn(t) { return !!Door.trackers && !!Door.trackers[t] && Door.trackers[t].connected === true }
    function canSetListStatus(c) {
        return !!c && !c.hidden
    }
    // Whether a series' status and score reach a tracker, or stay on this device (#213): a
    // series with no match, or with no account behind its ids, keeps them here
    function listsOnline(c) {
        if (!c || !c.match_info) return false
        return (frame.trackerOn("anilist") && !!c.match_info.anilist_id) || (frame.trackerOn("mal") && !!c.match_info.mal_id)
    }
    // Where a series' status and score go, in a line for a tooltip
    function listHome(c) {
        if (frame.listsOnline(c)) return "Change it on each list that is connected"
        var m = c && c.match_info
        if (!m || (!m.anilist_id && !m.mal_id)) return "Kept on this device: no tracker carries this show"
        if (frame.trackerOn("anilist") || frame.trackerOn("mal")) return "Kept on this device: your tracker does not carry this show"
        return "Kept on this device until you connect AniList or MyAnimeList"
    }
    // The five lists a status pill or a card's right-click moves a series to, the one it is on
    // already greyed out. The core writes it to every connected tracker.
    function listStatusItems(id, current) {
        var lists = [{ status: "Watching", text: "Watching", icon: "play" },
                     { status: "Planning", text: "Planning", icon: "bookmark" },
                     { status: "Completed", text: "Completed", icon: "circle-check" },
                     { status: "Paused", text: "On hold", icon: "pause" },
                     { status: "Dropped", text: "Dropped", icon: "ban" }]
        return lists.map(function(l) {
            return { text: l.text, icon: l.icon, status: l.status, enabled: l.status !== current,
                     action: function() { frame.setListStatus(id, l.status, l.text) } }
        })
    }
    // True when the write started, so a caller can go on (Start watching plays next)
    function setListStatus(id, status, label) {
        var r = Door.call("SetListStatus", { series: id, status: status })
        if (r.error) { frame.fail(r.error); return false }
        // No job is the list kept on this device, already moved
        frame.toast(r.reply && r.reply.job !== undefined ? "Moving to " + label : "Moved to " + label)
        return true
    }
    // Plan to watch (#734): Planning on every connected tracker, from a search hit or a Get
    // view. A show already on a list keeps its place there (the core never moves an entry).
    // True when the call went through, whether it started a write or found the show listed.
    function planToWatch(anilistId, title) {
        var t = Door.trackers
        if (!(t && ((t.anilist && t.anilist.connected) || (t.mal && t.mal.connected)))) { openConnect("Anilist"); return false }
        var r = Door.planToWatch(anilistId)
        if (r.error) { fail(r.error); return false }
        toast(r.reply && r.reply.job !== undefined ? title + " goes on your Planning list" : title + " is on your list already")
        return true
    }
    // Text onto the clipboard, and a word that it went
    function copyText(text, what) {
        copyQuiet(text)
        frame.toast(what || "Copied")
    }
    // Text onto the clipboard with no toast, for a caller that shows its own sign
    function copyQuiet(text) {
        clipboardLoader.active = true
        var buf = clipboardLoader.item
        buf.text = String(text || "")
        buf.selectAll()
        buf.copy()
    }
    // A TextEdit only to reach the clipboard, made on the first copy: built with the frame
    // it was 8 ms of every launch (#289)
    Loader { id: clipboardLoader; active: false; sourceComponent: TextEdit { visible: false } }

    // The subtitle defaults, one draft for the Playback and Subtitles tabs: the setting as
    // saved, with the edits still waiting on the 250 ms debounce laid over it. A refresh
    // while edits wait rebuilds the draft from the new setting and lays them on again, so
    // neither side's change is lost; a save the core rejects drops them and shows the
    // setting as it stands.
    property var subtitleDefaults: subtitleSaved()
    property var subtitlePending: []
    function subtitleSaved() { return JSON.parse(JSON.stringify(Door.settings.subtitle_defaults || {})) }
    function subtitleRebuild() {
        var d = subtitleSaved()
        for (var i = 0; i < subtitlePending.length; i++) subtitlePending[i](d)
        if (JSON.stringify(d) !== JSON.stringify(subtitleDefaults)) subtitleDefaults = d
    }
    Connections { target: Door; function onSettingsChanged() { frame.subtitleRebuild() } }

    // ---- Episode out (#212): the core says when an episode of a show on the Watching list
    // airs; the desktop shows it unless Settings turned it off. A click raises the window on
    // the series, or on Get episodes when the library does not have the show.
    Connections {
        target: Door
        function onEpisodeOut(ep) {
            if (Door.preferences.episode_notifications === false) return
            frame.episodesOut[ep.anilist_id] = ep
            Shell.notifyEpisode(ep)
        }
    }
    // The episodes said this run, by AniList id, so a click on one can hand Downloads its titles
    property var episodesOut: ({})
    Connections {
        target: Shell
        function onNotificationOpened(series, anilistId, title, token) {
            if (frame.hostWindow) Shell.raiseWindow(frame.hostWindow, token)
            if (series >= 0) frame.go("series", { id: series }, title)
            else {
                // Downloads searches by the latin titles, so each goes over as AniList has it
                var ep = frame.episodesOut[anilistId] || {}
                frame.go("downloads", { hit: { id: anilistId, title: title, romaji: ep.romaji || title, english: ep.english || "",
                                               native: ep.native || "", cover_url: "", episodes: 0, format: "", year: 0 } }, "Downloads")
            }
        }
    }
    Timer { id: subtitleSave; interval: 250; onTriggered: frame.flushSubtitleDefaults() }
    function editSubtitleDefaults(f) {
        var d = JSON.parse(JSON.stringify(subtitleDefaults))
        f(d)
        subtitlePending = subtitlePending.concat([f])
        subtitleDefaults = d
        subtitleSave.restart()
    }
    // Writes the waiting edits now. The settings tabs call it as they close, while Door is
    // still up, so leaving Settings within the debounce keeps the edit.
    function flushSubtitleDefaults() {
        subtitleSave.stop()
        if (!subtitlePending.length) return
        subtitlePending = []
        var r = Door.setSubtitleDefaults(subtitleDefaults)
        if (r.error) { frame.fail(r.error); subtitleRebuild() }
    }
    Component.onDestruction: if (Door.ready) flushSubtitleDefaults()
    function seriesDeleteItems() {
        var c = hoverSeries, anchor = hoverSeriesItem
        if (!c || !c.path || !(c.episodes_on_disk > 0)) return []
        var noun = seriesNoun(c), n = c.episodes_on_disk
        var out = []
        if ((c.watched || 0) > 0 && c.kind !== "Movie")
            out.push({ text: "Delete watched " + noun[1], icon: "trash-2", action: function() { frame.askDeleteFiles(anchor, c, true) } })
        out.push({ text: n === 1 ? "Delete the " + noun[0] : "Delete all " + n + " " + noun[1], icon: "trash-2",
                   action: function() { frame.askDeleteFiles(anchor, c, false) } })
        return out
    }
    property var deleteCalls: ({})   // token -> what the toast names
    // `whole` (Downloads' On disk, #266) takes the extras too, so the series leaves the disk
    function askDeleteFiles(anchor, c, watchedOnly, whole) {
        var r = Door.getSeries(c.id)
        if (r.error || !r.reply.detail) { r.error ? frame.fail(r.error) : frame.toast("That series is gone"); return }
        // An episode a torrent is still writing is not offered
        var files = (r.reply.detail.episodes || [])
            .filter(function(e) { return (e.downloading === undefined || e.downloading === null) && (!watchedOnly || e.watched) })
            .map(function(e) { return e.file })
        var extras = whole ? (r.reply.detail.extras || []).map(function(x) { return x.file }) : []
        if (!files.length && !extras.length) { frame.toast((watchedOnly ? "No watched episodes of " : "Nothing of ") + c.title + " on disk"); return }
        var noun = seriesNoun(c), which = watchedOnly ? "watched " : ""
        var words = files.length ? Fmt.plural(files.length, which + noun[0], which + noun[1]) : ""
        if (extras.length) words += (words ? " and " : "") + Fmt.plural(extras.length, "extra", "extras")
        files = files.concat(extras)
        removeSheet.openAt(anchor, {
            heading: "Delete " + words + " of " + c.title + "?",
            base: "The files and their subtitles go to the trash, so you can get them back from there. Their torrents leave the torrent client, so they can be downloaded again.",
            rule: false, torrentCount: 0, canFiles: false, confirmText: "Move to trash"
        }, function() {
            var t = Door.callAsync("DeleteEpisodes", { series: c.id, files: files })
            var m = Object.assign({}, frame.deleteCalls); m[t] = words + " of " + c.title; frame.deleteCalls = m
        })
    }
    // Delete watched across the library (#266): the frame keeps the reply, so its toast still
    // comes when the page has gone, and one runs at a time
    property real deleteWatchedToken: -1
    function deleteWatched(finished) {
        if (frame.deleteWatchedToken >= 0) return
        frame.deleteWatchedToken = Door.callAsync("DeleteWatched", { finished_shows: finished })
    }
    Connections {
        target: Door
        function onCallDone(token, reply) {
            if (token === frame.deleteWatchedToken) {
                frame.deleteWatchedToken = -1
                var d = reply.reply || {}
                frame.toast(reply.error ? frame.errorText(reply.error)
                    : d.episodes > 0 || d.series > 0 ? Fmt.plural(d.episodes, "episode", "episodes") + ", " + Fmt.bytes(d.bytes) + ", moved to the trash"
                    : "Nothing to delete")
                return
            }
            var words = frame.deleteCalls[token]
            if (words === undefined) return
            var m = Object.assign({}, frame.deleteCalls); delete m[token]; frame.deleteCalls = m
            reply.error ? frame.fail(reply.error) : frame.toast(words + " moved to the trash")
        }
    }

    // ---- Quick fixes (#170). The frame sends them and takes the reply, so what follows a fix
    // (the toast, a ticked issue) still happens after the sheet or the page has gone. token ->
    // { series, fixed: [file], n, undone, report, agent }
    property var fixApplies: ({})
    function applyFixes(series, changes, report, agent) {
        var token = Door.callAsync("FixFiles", { series: series, fixes: changes })
        if (token < 0) return -1
        var fixed = changes.filter(function(c) { return c.reading !== null }).map(function(c) { return c.file })
        var next = Object.assign({}, frame.fixApplies)
        next[token] = { series: series, fixed: fixed, n: changes.length, undone: changes.length - fixed.length,
                        report: !!report && fixed.length > 0, agent: agent || "" }
        frame.fixApplies = next
        return token
    }
    // A quick fix's GitHub issue: drafted off the Qt thread, a coding agent's triage first when
    // one is installed, which can take minutes. token -> the agent's name or ""
    property var fixReports: ({})
    function reportFix(series, files, agent) {
        var token = Door.callAsync("ReportFix", { series: series, files: files, app: Shell.release })
        if (token < 0) { frame.toast("Fixed. The issue could not be drafted"); return }
        var next = Object.assign({}, frame.fixReports)
        next[token] = agent || ""
        frame.fixReports = next
        frame.toast(agent ? "Fixed. " + agent + " is looking into it, and the issue opens when it is done" : "Fixed. Opening the issue", agent ? 8 : 4)
    }
    Connections {
        target: Door
        function onCallDone(token, reply) {
            var applied = frame.fixApplies[token]
            if (applied !== undefined) {
                var rest = Object.assign({}, frame.fixApplies)
                delete rest[token]
                frame.fixApplies = rest
                // The sheet says why when it is still up; the toast says it when it is not
                if (reply.error) { frame.toast("The fix did not save: " + frame.errorText(reply.error), 8); return }
                if (applied.report) frame.reportFix(applied.series, applied.fixed, applied.agent)
                else if (applied.undone === applied.n) frame.toast(Fmt.plural(applied.n, "file", "files") + " read by name again")
                else frame.toast("Fixed how " + Fmt.plural(applied.n, "file", "files") + " read")
                return
            }
            if (frame.fixReports[token] === undefined) return
            var next = Object.assign({}, frame.fixReports)
            delete next[token]
            frame.fixReports = next
            if (reply.error) { frame.toast("The issue could not be drafted: " + frame.errorText(reply.error), 8); return }
            var report = reply.reply.report
            frame.openLink(report.url)
            frame.toast(report.triage_error ? "The issue is open in your browser, without a triage. " + report.triage_error
                        : "The issue is open in your browser, ready to send", 8)
        }
    }

    // ---- Downloads: the subscriptions as last read, shared by the rail's badge and the
    // page. Read once a little after start (anirss is a sidecar the core starts), then
    // again when the library has settled after a change, since a new episode on disk can
    // finish a show. No poll: a read follows something that happened.
    property var downloadsRules: []
    // The downloads page's two bands, grid or list, for the session
    property string downloadsSubView: "grid"
    property string downloadsDlView: "grid"
    // The Downloads page's lens, and the sources switched by hand (name -> on), for the session
    property string downloadsLens: "gaps"
    property var downloadsSources: ({})
    property bool downloadsRulesRead: false
    property string downloadsRulesError: ""
    readonly property int downloadsAttention: {
        var n = 0
        for (var i = 0; i < downloadsRules.length; i++) if (downloadsRules[i].show && downloadsRules[i].show.complete) n++
        return n
    }
    // Put off while the removal sheet is up: a fresh list rebuilds what it was opened from
    // Another torrent client is in use: the rules read so far were the last one's
    function forgetDownloadsRules() { downloadsRules = []; downloadsRulesRead = false; downloadsRulesError = "" }
    function readDownloadsRules() {
        if (!Door.ready) return
        if (frame.removalOpen) { downloadsReread.restart(); return }
        Door.listSubscriptionRules()
        // One torrent read too, so the library knows what is still downloading before
        // the Downloads page is ever opened (the core keeps it fresh while it lasts)
        Door.listTorrents()
    }
    Connections {
        target: Door
        function onSubscriptionsRead(rules) { frame.downloadsRules = rules; frame.downloadsRulesRead = true; frame.downloadsRulesError = "" }
        function onDownloadsUnavailable(what, error) {
            if (what !== "subscriptions") return
            frame.downloadsRulesRead = true
            frame.downloadsRulesError = error && error.message ? error.message : "Not read."
        }
        // A run of removals (Remove RSS for all) asks once, after the last
        function onSubscriptionRemoved(name) { downloadsReread.restart() }
        function onSubscribed(job, result) { downloadsReread.restart() }
        function onScanFinished(source, added, changed, removed) { downloadsSettle.restart() }
        function onSeriesChanged(cards) { downloadsSettle.restart() }
    }
    Timer { interval: 4000; running: Door.ready; onTriggered: frame.readDownloadsRules() }
    Timer { id: downloadsSettle; interval: 10000; onTriggered: frame.readDownloadsRules() }
    Timer { id: downloadsReread; interval: 1500; onTriggered: frame.readDownloadsRules() }

    // A search behind What's new would take the keys the sheet still shows, so both close it
    Shortcut { sequence: "Ctrl+K"; onActivated: { whatsNew.close(); if (nav.current.page !== "library") frame.go("library"); Qt.callLater(function() { if (page.item && page.item.focusSearch) page.item.focusSearch() }) } }
    // Any page with a search takes "/" (the library, metadata)
    Shortcut { sequence: "/"; enabled: !!(page.item && page.item.focusSearch); onActivated: { whatsNew.close(); if (page.item && page.item.focusSearch) page.item.focusSearch() } }
    // Ctrl+F is the browser's own key for the same thing
    Shortcut { sequence: "Ctrl+F"; enabled: !!(page.item && page.item.focusSearch); onActivated: { whatsNew.close(); if (page.item && page.item.focusSearch) page.item.focusSearch() } }
    Shortcut { sequence: "Ctrl+,"; onActivated: if (nav.current.page !== "settings") frame.go("settings") }
    Shortcut { sequence: "Ctrl+L"; enabled: !frame.fullWindow; onActivated: frame.toggleDrawer() }
    Shortcut { sequence: "Ctrl+Q"; onActivated: Qt.quit() }
    // The mouse's own back and forward buttons, anywhere in the window. No other area takes
    // these two buttons, so a press reaches this one from under whatever it lands on.
    MouseArea {
        anchors.fill: parent
        acceptedButtons: Qt.BackButton | Qt.ForwardButton
        onPressed: function(m) { if (m.button === Qt.BackButton) frame.back(); else frame.forward() }
    }
    Shortcut { sequence: "Alt+Left"; onActivated: frame.back() }
    Shortcut { sequence: "Alt+Right"; onActivated: frame.forward() }
    // ? anywhere outside the player, which keeps its own list
    Shortcut { sequence: "?"; enabled: !frame.fullWindow; onActivated: frameKeys.open ? frameKeys.close() : frameKeys.show() }
    Shortcut { sequence: "F11"; onActivated: frame.hostWindow.visibility = frame.hostWindow.visibility === Window.FullScreen ? Window.Windowed : Window.FullScreen }
}
