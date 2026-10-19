// The tab base every settings tab extends: a Flickable over a ColumnLayout that holds
// whatever the tab declares as children. The layout is the viewport's height while its
// content fits and something in it grows, its natural height otherwise, so the Flickable
// only scrolls when it has to. Under it sits the focus sink: a press that no control
// claimed lands on it and takes the focus, so a Field, Dropdown or Slider lets go when the
// user clicks elsewhere. The block is the tab's whole width: flat sections on the ground,
// one joint apart, with no tile around any of them.
import QtQuick
import QtQuick.Layouts
import QtQuick.Controls.Basic as QC

Flickable {
    id: root
    default property alias body: layout.data
    property real footInset: theme.space(10)
    readonly property real viewport: Math.max(0, height - footInset)
    readonly property real gap: theme.space(8)
    // The sections sit in the centred column every page uses, while the Flickable spans the
    // page so the wheel works anywhere
    readonly property real blockWidth: Math.min(width, theme.contentMax) - theme.space(16)
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
    // The scroll bar rides the window's right edge, as on every other page
    QC.ScrollBar.vertical: ThinScrollBar {}
}
