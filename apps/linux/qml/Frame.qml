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
    readonly property bool fullWindow: page.item ? (page.item.fullWindow === true) : false
    readonly property string windowTitle: (page.item && page.item.title ? page.item.title : nav.current.label) + " - AniBeam"

    Nav { id: nav }
    QtObject {
        id: escapeStack
        property var entries: []
        readonly property var rank: ({ drawer: 3, confirm: 2, popover: 1 })
        function push(kind, closer) { pop(closer); entries = entries.concat([{ kind: kind, closer: closer }]) }
        function pop(closer) { entries = entries.filter(function(e) { return e.closer !== closer }) }
        function clear() { entries = [] }
        function top() {
            var best = null
            entries.forEach(function(e) { if (!best || rank[e.kind] >= rank[best.kind]) best = e })
            return best
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
    // The drawer is the one exception: it is not owned by the leaving page, so it is not
    // destroyed and stays open across a navigation. Re-push it right after the clear, in
    // this same handler, so it stays first in line for the next Escape; a second
    // Connections on nav.changed would race this one with no ordering guarantee between
    // them.
    Connections { target: nav; function onChanged() { frame.loadPage(); frame.hideTip(); frame.closeMenu(); frame.escapeStack.clear(); if (drawer.open) frame.escapeStack.push("drawer", drawer) } }

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
            height: visible ? theme.space(7) + theme.grout : 0
            anchors.left: parent.left; anchors.right: parent.right; anchors.bottom: parent.bottom
            readonly property var job: Door.runningJobs.length ? Door.runningJobs[0] : null
            readonly property var line: Door.latestLine
            stage: job ? String(job.kind).toLowerCase() : (line && line.stage ? String(line.stage).toLowerCase() : "system")
            message: job ? (job.label || job.kind) + (job.total > 0 ? ", " + job.done + " of " + job.total : "") : (line && line.message ? line.message : "")
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
    function trackerName(t) { return t === "Mal" ? "MyAnimeList" : "AniList" }
    function trackerAccount(t) { return (t === "Mal" ? Door.trackers.mal : Door.trackers.anilist) || ({}) }
    // Bundled with the build or pasted earlier: either way one click is all a connect needs.
    function hasCredentials(t) { var a = trackerAccount(t); return a.bundled_credentials === true || (a.client_id || "") !== "" }
    function withKey(map, key, value) {
        var out = {}
        for (var k in map) if (k !== key) out[k] = map[k]
        if (value !== undefined) out[key] = value
        return out
    }
    function trackerOfJob(job) {
        for (var k in connecting) if (connecting[k] === job) return k
        return ""
    }
    // The core's error message reads "Mal: ..." or "invalid client_id: ...", the shape the
    // log wants; a person reading why their sign-in stopped wants the sentence after it.
    function errorText(e) {
        var m = String(e && e.message ? e.message : "")
        m = m.replace(/^(Anilist|Mal|Jikan|Tmdb): /, "").replace(/^(io|internal|invalid [a-z_]+): /, "")
        return m === "" ? "The sign-in did not finish." : m.charAt(0).toUpperCase() + m.slice(1)
    }
    // Starts the flow; the browser opens when the core says the listener is up. True when
    // a job started, false with the reason in connectErrors when the call was refused.
    function connectTracker(t) {
        connectErrors = withKey(connectErrors, t)
        connectedAs = withKey(connectedAs, t)
        signInUrls = withKey(signInUrls, t)
        var r = Door.connectTracker(t)
        if (r.error) { connectErrors = withKey(connectErrors, t, errorText(r.error)); return false }
        connecting = withKey(connecting, t, r.reply.job)
        return true
    }
    function cancelConnect(t) {
        if (connecting[t] !== undefined) Door.cancelJob(connecting[t])
        connecting = withKey(connecting, t)
    }
    function reopenSignIn(t) { if (signInUrls[t]) Qt.openUrlExternally(signInUrls[t]) }
    // One click: with credentials the browser opens at once and the sheet only says where
    // to finish; without them the sheet walks the set-up first.
    function openConnect(t) {
        t = t || "Anilist"
        if (hasCredentials(t) && connecting[t] === undefined) connectTracker(t)
        connectSheet.openFor(t)
    }
    Connections {
        target: Door
        function onAuthUrlReady(t, openUrl, redirectUrl) {
            frame.signInUrls = frame.withKey(frame.signInUrls, t, openUrl)
            // A capture run draws the waiting state; it never opens the owner's browser.
            if (Shell.shoot === "" && Shell.driveOut === "") Qt.openUrlExternally(openUrl)
        }
        function onTrackerConnected(t, username) {
            frame.connecting = frame.withKey(frame.connecting, t)
            frame.connectedAs = frame.withKey(frame.connectedAs, t, username || frame.trackerName(t))
            frame.toast("Connected to " + frame.trackerName(t) + " as " + (username || "?"))
            // The user is in the browser tab that just closed; bring them back to the app.
            if (frame.hostWindow) Shell.raiseWindow(frame.hostWindow, "")
        }
        // Failed and cancelled come after the finished-job signal, so the job is looked up
        // here, by id, rather than cleared there: a connect superseded by a newer one for
        // the same tracker is cancelled by the core and must not clear the newer job.
        function onJobFailed(job, kind, error) {
            if (kind !== "ConnectTracker") return
            var t = frame.trackerOfJob(job)
            if (t === "") return
            frame.connecting = frame.withKey(frame.connecting, t)
            var message = frame.errorText(error)
            frame.connectErrors = frame.withKey(frame.connectErrors, t, message)
            // The sheet says it inline when it is up for this tracker; a toast as well would
            // be the same sentence twice.
            if (!connectSheet.visible || connectSheet.tracker !== t) frame.toast(message, 6)
        }
        function onJobCancelled(job, kind) {
            if (kind !== "ConnectTracker") return
            var t = frame.trackerOfJob(job)
            if (t !== "") frame.connecting = frame.withKey(frame.connecting, t)
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

    // Overlay: menus, tips, toasts
    Item {
        id: overlay
        anchors.fill: parent
        Menu { id: menu }
        Frost {
            id: tip
            visible: false
            radius: theme.radiusInner(theme.space(1))
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
        tip.y = Math.min(p.y + theme.space(1), overlay.height - tip.height - theme.space(2))
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
