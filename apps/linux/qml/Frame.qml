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
        var keep = escapeStack.entries.filter(function(e) { return e.closer === drawer || e.closer === connectSheet })
        escapeStack.clear()
        keep.forEach(function(e) {
            if (e.closer === drawer ? drawer.open : connectSheet.visible) escapeStack.push(e.kind, e.closer)
        })
    }

    // Pages by name; a page task swaps its placeholder for the real file
    readonly property var pages: ({
        library: "LibraryPage.qml", feed: "FeedPage.qml", watching: "WatchingPage.qml", metadata: "MetadataPage.qml",
        settings: "SettingsPage.qml", subscriptions: "SubscriptionsPage.qml", series: "SeriesPage.qml", player: "PlayerPage.qml"
    })

    // The settings page's own session state: the current tab and each tab's scroll
    // position, kept here rather than on the page itself so they survive the page being
    // torn down and rebuilt on every navigation away and back.
    property int settingsTab: 0
    property var settingsScroll: [0, 0, 0, 0]

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
    Connections { target: nav; function onChanged() { frame.loadPage(); frame.hideTip(); frame.closeMenu(); frame.reclaimFrameOwned() } }

    Rail {
        id: rail
        visible: !frame.fullWindow
        anchors.left: parent.left; anchors.top: parent.top; anchors.bottom: parent.bottom
        active: nav.railIndex
        onPicked: function(i) { if (i !== nav.railIndex || nav.current.page !== nav.railPages[i]) frame.go(nav.railPages[i]) }
    }

    Item {
        id: content
        anchors.left: frame.fullWindow ? parent.left : rail.right
        anchors.right: parent.right; anchors.top: parent.top; anchors.bottom: parent.bottom

        Loader {
            id: page
            anchors.left: parent.left; anchors.right: parent.right; anchors.top: parent.top
            anchors.bottom: frame.fullWindow ? parent.bottom : strip.top
            focus: true
            asynchronous: false
            onLoaded: {
                if (nav.pendingScroll > 0 && item.scrollY !== undefined) Qt.callLater(function() { if (page.item) page.item.scrollY = nav.pendingScroll })
                item.forceActiveFocus()
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
            anchors.left: parent.left; anchors.right: parent.right; anchors.bottom: parent.bottom
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
    // A metadata provider's name as people write it: the core's enum word in, the brand out
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
    function reopenSignIn(t) { if (signInUrls[t]) Qt.openUrlExternally(signInUrls[t]) }
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
            if (Shell.shoot === "" && Shell.driveOut === "") Qt.openUrlExternally(openUrl)
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

    // Overlay: menus, tips, toasts, sheets. Above the drawer (z 500), so a menu or a
    // sheet opened over the drawer's region draws over it, not under it.
    Item {
        id: overlay
        anchors.fill: parent
        z: 600
        Menu { id: menu }
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
    function openMenu(x, y, items) { menu.openAt(x, y, items) }
    function closeMenu() { menu.close() }
    function showTip(item, text) {
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
    function hideTip() { tip.visible = false }
    function toast(text, seconds) { toastText.text = text; toastBox.visible = true; toastTimer.interval = (seconds || 4) * 1000; toastTimer.restart() }

    Shortcut { sequence: "Ctrl+K"; onActivated: { if (nav.current.page !== "library") frame.go("library"); Qt.callLater(function() { if (page.item && page.item.focusSearch) page.item.focusSearch() }) } }
    Shortcut { sequence: "/"; enabled: nav.current.page === "library"; onActivated: if (page.item && page.item.focusSearch) page.item.focusSearch() }
    Shortcut { sequence: "Ctrl+,"; onActivated: if (nav.current.page !== "settings") frame.go("settings") }
    Shortcut { sequence: "Ctrl+L"; enabled: !frame.fullWindow; onActivated: frame.toggleDrawer() }
    Shortcut { sequence: "Ctrl+Q"; onActivated: Qt.quit() }
    Shortcut { sequence: "Alt+Left"; onActivated: nav.back() }
}
