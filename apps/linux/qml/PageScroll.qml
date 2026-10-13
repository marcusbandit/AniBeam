// The scrolling base most pages sit on: a Flickable with the shell's thin scroll bar, a
// focus sink so a click on empty space releases a field, scrollY for the trail, and the
// arrow keys across whatever grids the page holds (GridKeys). A page whose root takes the
// keys first forwards them with keyPressed(event), so Down from a page with nothing focused
// lands on its first card.
import QtQuick
import QtQuick.Window
import QtQuick.Controls.Basic as QC

Flickable {
    id: root
    property alias scrollY: root.contentY
    default property alias content: inner.data
    property real footInset: theme.space(10)
    contentWidth: width
    contentHeight: inner.implicitHeight + footInset
    clip: true
    boundsBehavior: Flickable.StopAtBounds
    QC.ScrollBar.vertical: ThinScrollBar {}
    WheelStep { id: wheel; flick: root }
    // A programmatic scroll rides the wheel's target, so it follows with the same smoothing
    // as a notch and a drag takes over the same way
    function scrollTo(y) {
        wheel.target = wheel.bound(y)
        wheel.following = true
        root.cancelFlick()
    }
    GridKeys { id: gridKeys; flick: root }
    function keyPressed(event) { gridKeys.handle(event, root.Window.window ? root.Window.window.activeFocusItem : null) }
    Keys.onPressed: function(event) { root.keyPressed(event) }
    MouseArea { anchors.fill: parent; onPressed: function(m) { root.forceActiveFocus(); m.accepted = false } }
    Column {
        id: inner
        x: theme.space(8)
        y: theme.space(7)
        width: root.width - theme.space(16)
        spacing: theme.space(4)
    }
}
