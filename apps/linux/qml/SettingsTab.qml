// The tab base every settings tab extends: a Flickable over a ColumnLayout that holds
// whatever the tab declares as children. The layout is the viewport's height while its
// content fits and something in it grows, its natural height otherwise, so the Flickable
// only scrolls when it has to. Under it sits the focus sink: a press that no control
// claimed lands on it and takes the focus, so a Field, Dropdown or Slider lets go when the
// user clicks elsewhere. `blockX` and `blockWidth` are the block every tab shares: the
// tab's width up to a cap, centred when the tab is wider, so a row's label and its control
// never drift metres apart, and SettingsPage's header lines up with it.
import QtQuick
import QtQuick.Layouts
import QtQuick.Controls.Basic as QC

Flickable {
    id: root
    default property alias body: layout.data
    property real footInset: theme.space(10)
    readonly property real viewport: Math.max(0, height - footInset)
    readonly property real gap: theme.space(6)
    readonly property real maxWidth: theme.space(560)
    readonly property real blockWidth: Math.min(width, maxWidth)
    readonly property real blockX: Math.round((width - blockWidth) / 2)

    // Whether any direct child asks for the spare height (a Panel or a SettingsPair with
    // `grows`)
    function anyGrows(items) { for (var i = 0; i < items.length; i++) if (items[i].grows === true) return true; return false }

    anchors.fill: parent
    contentWidth: width
    contentHeight: layout.height + footInset
    clip: true
    boundsBehavior: Flickable.StopAtBounds
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
    QC.ScrollBar.vertical: ThinScrollBar {}
}
