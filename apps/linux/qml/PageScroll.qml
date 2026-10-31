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

    // LazyTile's bookkeeping (#289). A budget of about a screen and a tenth of card area,
    // refilled after every painted frame: tiles made while it lasts build at once. The
    // others wait in `lazyWaiting` and are measured only after a frame (the layout is
    // settled then) and when the view has moved a quarter of a screen, never per tile and
    // never per scroll frame.
    readonly property real lazyBudget: {
        var w = root.Window.window
        return 1.1 * (w ? w.width * w.height : 1920 * 1080)
    }
    property real lazyLeft: lazyBudget
    property int lazyCount: 0
    property var lazyWaiting: []
    property real lazySweptAt: 0
    function takeLazy(area) {
        var had = lazyLeft
        lazyLeft -= area
        return had > 0
    }
    function lazyAdd(tile) { lazyWaiting.push(tile); lazyCount = lazyWaiting.length }
    function lazyRemove(tile) {
        var i = lazyWaiting.indexOf(tile)
        if (i >= 0) { lazyWaiting.splice(i, 1); lazyCount = lazyWaiting.length }
    }
    function lazySweep() {
        lazySweptAt = contentY
        var view = contentY, end = contentY + height, keep = []
        for (var i = 0; i < lazyWaiting.length; i++) {
            var t = lazyWaiting[i]
            if (!t || t.seen) continue
            var y = t.mapToItem(contentItem, 0, 0).y
            if (y + t.height > view - height && y < end + height)
                t.build(y + t.height > view && y < end)
            else keep.push(t)
        }
        lazyWaiting = keep
        lazyCount = keep.length
    }
    Connections {
        target: root.Window.window
        enabled: root.lazyCount > 0 || root.lazyLeft < root.lazyBudget
        function onFrameSwapped() {
            root.lazyLeft = root.lazyBudget
            if (root.lazyCount > 0) root.lazySweep()
        }
    }
    onContentYChanged: if (lazyCount > 0 && Math.abs(contentY - lazySweptAt) > height / 4) lazySweep()
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
