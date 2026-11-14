// The scrolling base most pages sit on: a Flickable with the shell's thin scroll bar, a
// focus sink so a click on empty space releases a field, scrollY for the trail, and the
// arrow keys across whatever grids the page holds (GridKeys). A page whose root takes the
// keys first forwards them with keyPressed(event), so Down from a page with nothing focused
// lands on its first card. The Flickable spans the page's whole width, so the wheel
// scrolls it from anywhere, while the content sits in a centred column no wider than
// theme.contentMax; `gutter` is the column's distance from the left edge, for a band that
// should bleed out to the edges.
import QtQuick
import QtQuick.Window
import QtQuick.Controls.Basic as QC
import com.marcusrosado.AniBeam

Flickable {
    id: root
    property alias scrollY: root.contentY
    default property alias content: inner.data
    property real footInset: theme.space(10)
    readonly property real column: Math.min(width, theme.contentMax)
    readonly property real gutter: inner.x
    contentWidth: width
    contentHeight: inner.implicitHeight + footInset
    clip: true
    boundsBehavior: Flickable.StopAtBounds
    QC.ScrollBar.vertical: ThinScrollBar {}
    WheelStep { id: wheel; flick: root }
    // The arrow keys scroll through the wheel's target, so they glide the same way and a
    // notch or a drag takes over
    function scrollTo(y) { wheel.scrollTo(y) }
    readonly property real scrollTarget: wheel.target
    GridKeys { id: gridKeys; flick: root }
    // Scrolls a card into view the way the arrow keys do: a LazyTile that Tab built asks
    function reveal(item) { gridKeys.reveal(item) }

    // LazyTile's bookkeeping (#289). A budget of one viewport of card area,
    // refilled after every painted frame: tiles made while it lasts build at once. The
    // others wait in `lazyWaiting` and are measured only after a frame (the layout is
    // settled then) and when the view has moved a quarter of a screen, never per tile and
    // never per scroll frame.
    readonly property real lazyBudget: {
        // The page's own viewport, not the window: only what is in view builds before the
        // first frame; the rest comes in the frames after, before a scroll can reach it
        if (root.width > 0 && root.height > 0) return root.width * root.height
        var w = root.Window.window
        return w ? w.width * w.height : 1920 * 1080
    }
    property real lazyLeft: lazyBudget
    property int lazyCount: 0
    property var lazyWaiting: []
    property real lazySweptAt: 0
    function takeLazy(area) {
        // Scrolled down, the first tiles made are the ones at the top, not the ones in view:
        // none build at once; the sweep after the frame builds what is in view (#289)
        if (contentY > 0) return false
        var had = lazyLeft
        lazyLeft -= area
        return had > 0
    }
    function lazyAdd(tile) { lazyWaiting.push(tile); lazyCount = lazyWaiting.length; lazyDirty = true }
    // Set when a tile joins or the layout moves; a painted frame sweeps only then, not on
    // every frame while tiles wait
    property bool lazyDirty: false
    onContentHeightChanged: if (lazyCount > 0) lazyDirty = true
    onWidthChanged: if (lazyCount > 0) lazyDirty = true
    function lazyRemove(tile) {
        var i = lazyWaiting.indexOf(tile)
        if (i >= 0) { lazyWaiting.splice(i, 1); lazyCount = lazyWaiting.length }
    }
    function lazySweep() {
        lazySweptAt = contentY
        var live = lazyWaiting.filter(function(t) { return t && !t.seen })
        // Measured in one native call: in view (2), within a screen of it (1), or not yet (0)
        var at = TileReach.measure(contentItem, live, contentY, contentY + height, contentY - height, contentY + 2 * height)
        var keep = []
        for (var i = 0; i < live.length; i++) {
            if (at[i] > 0) live[i].build(at[i] === 2)
            else keep.push(live[i])
        }
        lazyWaiting = keep
        lazyCount = keep.length
    }
    Connections {
        target: root.Window.window
        enabled: root.lazyDirty || root.lazyLeft < root.lazyBudget
        function onFrameSwapped() {
            root.lazyLeft = root.lazyBudget
            if (root.lazyCount > 0 && root.lazyDirty) { root.lazyDirty = false; root.lazySweep() }
        }
    }
    onContentYChanged: {
        if (lazyCount > 0 && Math.abs(contentY - lazySweptAt) > height / 4) lazySweep()
        // Content that moves under a still brush (the edge scroll, a wheel turn) is painted too
        if (brushing) brushHit(brushAt)
    }

    // ---- Painting picks (#695). A card whose press-and-hold began picking hands its pointer
    // here (window coordinates). Each move picks (`brushAdd`) or unpicks every card along the
    // path since the last move, so a fast stroke leaves no holes. Once the pointer has moved
    // into the top or bottom band of the view, the page scrolls on at a speed that grows
    // toward the edge, picking what slides under the pointer; a hold that never moves never
    // scrolls. A point outside the view picks nothing. Cards answer to `brushable`,
    // `selected` and `selectToggled`.
    property bool brushing: false
    property bool brushAdd: true
    property bool brushMoved: false
    property point brushStart: Qt.point(0, 0)
    property point brushAt: Qt.point(0, 0)
    readonly property real brushEdge: theme.space(18)
    readonly property real brushMaxSpeed: 1400     // px a second, at the very edge or past it
    // -1 near the top, 1 near the bottom, 0 elsewhere or before the pointer has moved
    readonly property real brushDepth: {
        if (!brushing || !brushMoved) return 0
        var y = mapFromItem(null, brushAt.x, brushAt.y).y
        if (y < brushEdge) return -Math.min(1, (brushEdge - y) / brushEdge)
        if (y > height - brushEdge) return Math.min(1, (y - (height - brushEdge)) / brushEdge)
        return 0
    }
    function brushBegin(winPoint, add) {
        wheel.glide.stop()
        root.cancelFlick()
        brushAdd = add
        brushAt = winPoint
        brushStart = winPoint
        brushMoved = false
        brushTouched = []
        brushing = true
        brushHit(brushAt, true)
    }
    function brushMove(winPoint) {
        if (!brushMoved && Math.abs(winPoint.x - brushStart.x) + Math.abs(winPoint.y - brushStart.y) < theme.space(2)) return
        brushMoved = true
        var from = brushAt
        brushAt = winPoint
        // Walk the segment in steps smaller than any card, so no card between two
        // samples is skipped
        var dx = winPoint.x - from.x, dy = winPoint.y - from.y
        var steps = Math.max(1, Math.ceil(Math.sqrt(dx * dx + dy * dy) / theme.space(4)))
        for (var i = 1; i <= steps; i++) brushHit(Qt.point(from.x + dx * i / steps, from.y + dy * i / steps))
    }
    function brushEnd() { brushing = false; brushTouched = [] }
    function brushCard(x, y) {
        var it = root.contentItem, p = root.mapToItem(it, x, y)
        for (var depth = 0; depth < 12; depth++) {
            var c = it.childAt(p.x, p.y)
            if (!c) return null
            if (c.brushable === true) return c
            p = it.mapToItem(c, p.x, p.y)
            it = c
        }
        return null
    }
    // Each card is touched once a stroke: the held card is marked, not toggled again, however
    // late its `selected` catches up
    property var brushTouched: []
    function brushHit(winPoint, markOnly) {
        var v = root.mapFromItem(null, winPoint.x, winPoint.y)
        if (v.x < 0 || v.x >= root.width) return
        if (v.y < 0 || v.y >= root.height) {
            // Past the top or bottom only while the edge scroll runs: the brush keeps the
            // column under the pointer, at the view's edge
            if (brushDepth === 0) return
            v.y = Math.max(0, Math.min(v.y, root.height - 1))
        }
        var c = brushCard(v.x, v.y)
        if (!c || brushTouched.indexOf(c) >= 0) return
        brushTouched.push(c)
        if (!markOnly && c.selected !== brushAdd) c.selectToggled()
    }
    // Runs only while the pointer sits in an edge band; each tick steps by the real time since
    // the last one, so a late tick does not slow the scroll
    Timer {
        id: brushEdgeTick
        interval: 16
        repeat: true
        running: root.brushDepth !== 0
        property real last: 0
        onRunningChanged: last = Date.now()
        onTriggered: {
            var now = Date.now(), dt = Math.min(0.1, (now - last) / 1000)
            last = now
            var to = wheel.bound(root.contentY + root.brushDepth * root.brushMaxSpeed * dt)
            if (to === root.contentY) return
            wheel.glide.stop()
            root.contentY = to
        }
    }

    function keyPressed(event) { gridKeys.handle(event, root.Window.window ? root.Window.window.activeFocusItem : null) }
    Keys.onPressed: function(event) { root.keyPressed(event) }
    MouseArea { anchors.fill: parent; onPressed: function(m) { root.forceActiveFocus(); m.accepted = false } }
    Column {
        id: inner
        x: Math.round((root.width - root.column) / 2) + theme.space(8)
        y: theme.space(7)
        width: root.column - theme.space(16)
        spacing: theme.space(4)
    }
}
