// The tab base every settings tab extends: a Flickable over a ColumnLayout that holds
// whatever the tab declares as children. The layout is the viewport's height while its
// content fits and something in it grows, its natural height otherwise, so the Flickable
// only scrolls when it has to. Under it sits the focus sink: a press that no control
// claimed lands on it and takes the focus, so a Field, Dropdown or Slider lets go when the
// user clicks elsewhere. The block is the tab's whole width: its tiles fill the viewport on
// the mosaic, one grout apart, with no cap and no centring column.
import QtQuick
import QtQuick.Layouts
import QtQuick.Controls.Basic as QC

Flickable {
    id: root
    default property alias body: layout.data
    property real footInset: 0
    readonly property real viewport: Math.max(0, height - footInset)
    readonly property real gap: theme.grout
    readonly property real blockWidth: width
    readonly property real blockX: 0

    // Whether any direct child asks for the spare height (a Panel or a SettingsPair with
    // `grows`)
    function anyGrows(items) { for (var i = 0; i < items.length; i++) if (items[i].grows === true) return true; return false }

    anchors.fill: parent
    contentWidth: width
    contentHeight: layout.height + footInset
    clip: true
    boundsBehavior: Flickable.StopAtBounds
    WheelStep { flick: root }
    MouseArea {
        id: sink
        width: root.width
        height: Math.max(root.height, root.contentHeight)
        onPressed: sink.forceActiveFocus()
    }
    ColumnLayout {
        id: layout
        readonly property bool grows: root.anyGrows(children)
        x: root.blockX
        width: root.blockWidth
        height: grows ? Math.max(root.viewport, implicitHeight) : implicitHeight
        spacing: root.gap
    }
    // The scroll bar rides the right-hand joint, beside the tiles
    QC.ScrollBar.vertical: ThinScrollBar { parent: root.parent; x: root.x + root.width + Math.max(0, (theme.grout - width) / 2); y: root.y; height: root.height; width: Math.min(theme.grout, theme.space(1)) }
}
