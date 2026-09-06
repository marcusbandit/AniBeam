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
        library: libraryPage, feed: placeholder, watching: placeholder, metadata: placeholder, settings: placeholder,
        subscriptions: placeholder, series: placeholder, player: placeholder
    })
    Component { id: placeholder; PagePlaceholder {} }
    Component { id: libraryPage; PagePlaceholder {} }

    function leavingScroll() { return page.item && page.item.scrollY !== undefined ? page.item.scrollY : 0 }
    function go(name, props, label) { nav.open(name, props, label, leavingScroll()) }

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
            sourceComponent: frame.pages[nav.current.page] || placeholder
            onLoaded: {
                item.props = nav.current.props
                if (nav.pendingScroll > 0 && item.scrollY !== undefined) Qt.callLater(function() { if (page.item) page.item.scrollY = nav.pendingScroll })
                item.forceActiveFocus()
            }
        }
        Connections {
            target: nav
            function onChanged() { page.active = false; page.active = true }
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

        // Task 14 fills the drawer; the strip is wired now
        Item { id: drawerSlot; anchors.left: parent.left; anchors.right: parent.right; anchors.bottom: strip.top }
        StatusStrip {
            id: strip
            visible: !frame.fullWindow
            height: visible ? theme.space(7) : 0
            anchors.left: parent.left; anchors.right: parent.right; anchors.bottom: parent.bottom
            readonly property var job: Door.runningJobs.length ? Door.runningJobs[0] : null
            readonly property var line: Door.latestLine
            stage: job ? String(job.kind).toLowerCase() : (line && line.stage ? String(line.stage).toLowerCase() : "system")
            message: job ? (job.label || job.kind) + (job.total > 0 ? " · " + job.done + " of " + job.total : "") : (line && line.message ? line.message : "")
            time: job || !line || !line.at ? "" : Qt.formatTime(new Date(line.at * 1000), "hh:mm")
            running: job !== null
            fraction: job && job.total > 0 ? job.done / job.total : 0
            unseenErrors: Door.unseenErrors
            onClicked: frame.toggleDrawer()
        }
    }
    function toggleDrawer() {}   // Task 14

    // Overlay: menus, tips, toasts
    Item {
        id: overlay
        anchors.fill: parent
        Menu { id: menu }
        Corner {
            id: tip
            visible: false
            radius: theme.radiusSm; smoothing: theme.cornerSmoothing
            color: theme.surfaceRaised; borderColor: theme.lineStrong; borderWidth: 1
            width: tipText.implicitWidth + theme.space(4); height: tipText.implicitHeight + theme.space(2)
            Text { id: tipText; anchors.centerIn: parent; color: theme.text; font.family: theme.fontSans; font.pointSize: theme.typeSmall }
        }
        Corner {
            id: toastBox
            visible: false
            anchors.horizontalCenter: parent.horizontalCenter
            anchors.bottom: parent.bottom; anchors.bottomMargin: theme.space(10)
            radius: height / 2; smoothing: theme.cornerSmoothing
            color: theme.surfaceRaised; borderColor: theme.lineStrong; borderWidth: 1
            width: toastText.implicitWidth + theme.space(6); height: theme.controlHeight
            Text { id: toastText; anchors.centerIn: parent; color: theme.text; font.family: theme.fontSans; font.pointSize: theme.typeNormal }
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
