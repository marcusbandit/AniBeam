// The frame: the rail, the page area with the status strip at its foot and the drawer
// rising from it, and an overlay for menus and tips. Everything in spec 4.1 unit 1 that
// is not a page lives here. The player takes the whole window and hides the rest.
import QtQuick
import QtQuick.Window
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
        home: "HomePage.qml", library: "LibraryPage.qml", feed: "FeedPage.qml", watching: "WatchingPage.qml", metadata: "MetadataPage.qml",
        downloads: "DownloadsPage.qml", settings: "SettingsPage.qml", series: "SeriesPage.qml", player: "PlayerPage.qml"
    })

    // The settings page's own session state: the current tab and each tab's scroll
    // position, kept here rather than on the page itself so they survive the page being
    // torn down and rebuilt on every navigation away and back.
    property int settingsTab: 0
    property var settingsScroll: [0, 0, 0, 0, 0]

    function leavingScroll() { return page.item && page.item.scrollY !== undefined ? page.item.scrollY : 0 }
    function go(name, props, label) { nav.open(name, props, label, leavingScroll()) }

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
    function loadPage() { Qt.callLater(frame.loadPageNow) }
    function loadPageNow() {
        page.source = ""
        page.setSource(Qt.resolvedUrl(frame.pages[nav.current.page] || "PagePlaceholder.qml"), { props: nav.current.props })
    }
    Component.onCompleted: frame.loadPage()
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
    Connections { target: nav; function onChanged() { frame.loadPage(); frame.hideTip(); frame.closeMenu(); removeSheet.close(); whatsNew.close(); frame.reclaimFrameOwned() } }

    Rail {
        id: rail
        visible: !frame.fullWindow
        anchors.left: parent.left; anchors.top: parent.top; anchors.bottom: parent.bottom
        active: nav.railIndex
        downloadsBadge: frame.downloadsAttention
        onPicked: function(i) { if (i !== nav.railIndex || nav.current.page !== nav.railPages[i]) frame.go(nav.railPages[i]) }
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
            onPressed: function(m) {
                var p = mapToItem(frame, m.x, m.y)
                var items = [{ text: "Back", icon: "arrow-left", action: nav.back }]
                if (page.item && page.item.contextItems) items = items.concat(page.item.contextItems())
                frame.openMenu(p.x, p.y, items)
            }
        }

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
            readonly property var line: Door.latestLine
            // A kind or stage as words: "ConnectTracker" reads "Connect tracker", "metadata"
            // reads "Metadata"
            function words(k) {
                var w = String(k).replace(/([a-z])([A-Z])/g, "$1 $2").replace(/_/g, " ").toLowerCase()
                return w.charAt(0).toUpperCase() + w.slice(1)
            }
            stage: job ? words(job.kind) : (line && line.stage ? words(line.stage) : "System")
            message: job ? (job.label || words(job.kind)) + (job.total > 0 ? ", " + job.done + " of " + job.total : "") : (line && line.message ? line.message : "")
            time: job || !line || !line.at ? "" : Qt.formatTime(new Date(line.at * 1000), "hh:mm")
            running: job !== null
            fraction: job && job.total > 0 ? job.done / job.total : 0
            unseenErrors: Door.unseenErrors
            onClicked: frame.toggleDrawer()
        }
    }
    ActivityDrawer {
        id: drawer
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
    // id: ...", the shape the log wants; a person reading why their sign-in stopped wants
    // the sentence after it.
    function errorText(e) {
        var m = String(e && e.message ? e.message : "")
        m = m.replace(/^(AniList|MyAnimeList|TMDB|Anilist|Mal|Jikan|Tmdb)( did not respond)?: /, "").replace(/^(io|internal|[Ii]nvalid [a-z_ ]+): /, "")
        return m === "" ? "The sign-in did not finish." : m.charAt(0).toUpperCase() + m.slice(1)
    }
    // Starts the flow; the browser opens when the core says the listener is up. True when
    // a job started, false with the reason in connectErrors when the call was refused or
    // the other tracker's sign-in is still out.
    function connectTracker(t) {
        if (connectBlockedBy(t) !== "") { toast(blockedTip(t)); return false }
        connectErrors = withKey(connectErrors, t)
        connectedAs = withKey(connectedAs, t)
        signInUrls = withKey(signInUrls, t)
        // A second connect of the same tracker replaces the first: the core cancels it,
        // and that cancel was asked for.
        if (connecting[t] !== undefined) expectedCancels = withKey(expectedCancels, connecting[t], true)
        var r = Door.connectTracker(t)
        if (r.error) { connectErrors = withKey(connectErrors, t, errorText(r.error)); return false }
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
        if (!Shell.openUrl(u)) frame.toast("Could not open " + u)
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
            var message = frame.errorText(error)
            frame.connectErrors = frame.withKey(frame.connectErrors, t, message)
            // The sheet says it inline when it is up for this tracker; a toast as well would
            // be the same sentence twice.
            if (!connectSheet.visible || connectSheet.tracker !== t) frame.toast(message, 6)
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
            if (!asked && frame.latestJob[t] === job) frame.toast("The " + frame.trackerName(t) + " sign-in stopped")
        }
    }

    // The tracker connect sheet, over everything: the Watching page's empty
    // list, the library's nudge and the Trackers panel all send the user here.
    ConnectSheet {
        id: connectSheet
        objectName: "connect-sheet"
        parent: overlay
        z: 10
    }

    // The first run (#193): over the whole frame, the rail included, and under the overlay,
    // so the connect sheet, the folder dialog's toasts and tips still draw over it.
    Onboarding {
        id: onboarding
        z: 550
    }
    function openOnboarding(step) { whatsNew.close(); onboarding.open(step) }
    readonly property alias onboardingPanel: onboarding

    // Overlay: menus, tips, toasts, sheets. Above the drawer (z 500), so a menu or a
    // sheet opened over the drawer's region draws over it, not under it.
    Item {
        id: overlay
        anchors.fill: parent
        z: 600
        Menu { id: menu }
        RemoveSheet { id: removeSheet }
        WhatsNew { id: whatsNew }
        Frost {
            id: tip
            visible: false
            radius: theme.radiusTile
            width: tipText.implicitWidth + theme.space(4); height: tipText.implicitHeight + theme.space(2)
            Label { id: tipText; anchors.centerIn: parent; role: "ui"; font.pointSize: theme.typeSmall }
        }
        Frost {
            id: toastBox
            visible: false
            radius: theme.radiusTile
            anchors.horizontalCenter: parent.horizontalCenter
            anchors.bottom: parent.bottom; anchors.bottomMargin: theme.space(10)
            width: toastText.implicitWidth + theme.space(6); height: theme.controlHeight
            Label { id: toastText; anchors.centerIn: parent; role: "ui" }
            Timer { id: toastTimer; onTriggered: toastBox.visible = false }
        }
    }
    // What's new: once the first page is up, so the page taking focus as it loads does not
    // take the keys from the sheet. Never in a capture run, which draws what it was asked to;
    // a drive opens it with frame.openWhatsNew().
    property bool notesStarted: false
    function startNotes() {
        if (notesStarted) return
        notesStarted = true
        // A fresh install gets the first run instead; What's new only records the version then
        onboarding.start()
        if (Shell.shoot === "" && Shell.driveOut === "") whatsNew.start()
    }
    function openWhatsNew() { whatsNew.open() }
    readonly property alias notesPanel: whatsNew

    function openMenu(x, y, items) { menu.openAt(x, y, items) }
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
    function tipStillTrue() {
        if (!tipArea || !pointerInside(tipArea)) return false
        var o = tipArea.mapToItem(overlay, 0, 0)
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
            var o = frame.tipArea.mapToItem(overlay, 0, 0)
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
    function showTip(item, text, area, retried) {
        area = area || item
        if (item && !retried && !frame.pointerInside(area)) { tipRetry.ask = { item: item, text: text, area: area }; tipRetry.restart(); return }
        if (!item || !frame.pointerInside(area)) { frame.hideTip(); return }
        tipRetry.stop()
        tipGrace.stop()
        tipArea = area
        tipOrigin = area.mapToItem(overlay, 0, 0)
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
    function toast(text, seconds) { toastText.text = text; toastBox.visible = true; toastTimer.interval = (seconds || 4) * 1000; toastTimer.restart() }

    // ---- The removal sheet: every removal on the downloads page asks through it
    readonly property bool removalOpen: removeSheet.visible
    function askRemoval(anchor, spec, done) { removeSheet.openAt(anchor, spec, done) }

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
    Shortcut { sequence: "Ctrl+,"; onActivated: if (nav.current.page !== "settings") frame.go("settings") }
    Shortcut { sequence: "Ctrl+L"; enabled: !frame.fullWindow; onActivated: frame.toggleDrawer() }
    Shortcut { sequence: "Ctrl+Q"; onActivated: Qt.quit() }
    Shortcut { sequence: "Alt+Left"; onActivated: nav.back() }
}
