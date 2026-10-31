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

    // LazyTile's budget (#289): the area of cards built at once when the page is made,
    // about a screen and a tenth; the rest are built as they come near, once the page
    // has painted and its layout is settled (lazyArmed).
    property real lazyLeft: 0
    property bool lazyStarted: false
    property bool lazyArmed: false
    function takeLazy(area) {
        if (!lazyStarted) {
            lazyStarted = true
            var w = root.Window.window
            lazyLeft = 1.1 * (w ? w.width * w.height : 1920 * 1080)
        }
        var had = lazyLeft
        lazyLeft -= area
        return had > 0
    }
    Connections {
        target: root.Window.window
        enabled: !root.lazyArmed
        function onFrameSwapped() { root.lazyArmed = true }
    }
    Timer { running: !root.lazyArmed; interval: 250; onTriggered: root.lazyArmed = true }
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
