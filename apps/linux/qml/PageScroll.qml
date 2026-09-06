// The scrolling base most pages sit on: a Flickable with the shell's thin scroll bar, a
// focus sink so a click on empty space releases a field, and scrollY for the trail.
import QtQuick
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
    QC.ScrollBar.vertical: QC.ScrollBar {
        policy: QC.ScrollBar.AsNeeded
        visible: size < 1
        contentItem: Corner {
            implicitWidth: theme.space(1)
            radius: height / 2
            smoothing: theme.cornerSmoothing
            color: theme.lineStrong
            opacity: parent.active ? 1 : 0.4
        }
    }
    MouseArea { anchors.fill: parent; onPressed: function(m) { root.forceActiveFocus(); m.accepted = false } }
    Column {
        id: inner
        x: theme.space(8)
        y: theme.space(7)
        width: root.width - theme.space(16)
        spacing: theme.space(4)
    }
}
