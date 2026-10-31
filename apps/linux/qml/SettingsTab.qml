// One settings section, the base every section extends: a Flickable over the section's
// heading (its name in the display cut and one plain sentence on what lives here) and a
// ColumnLayout of groups (`Panel.qml`), at a readable measure (`measure`) set at the left of
// the content column beside the index. `wide` lets a section with tables (Developer) take
// the whole column. The layout is the viewport's height while its content fits and
// something in it grows, its natural height otherwise, so the Flickable only scrolls when
// it has to. Under it sits the focus sink: a press that no control claimed lands on it and
// takes the focus, so a Field, Dropdown or Slider lets go when the user clicks elsewhere.
// `reveal(text)` is search's way in: it finds the row or group named `text`, scrolls it
// into view and pulses it.
import QtQuick
import QtQuick.Layouts
import QtQuick.Controls.Basic as QC

Flickable {
    id: root
    default property alias body: layout.data
    property string title: ""
    property string lead: ""
    property bool wide: false
    property real footInset: theme.space(16)
    readonly property real viewport: Math.max(0, height - footInset - layout.y)
    readonly property real gap: theme.space(10)
    readonly property real measure: theme.space(210)
    readonly property real blockWidth: wide ? width - theme.space(6) : Math.min(width - theme.space(6), measure)
    readonly property real blockX: 0

    // Whether any direct child asks for the spare height
    function anyGrows(items) { for (var i = 0; i < items.length; i++) if (items[i].grows === true) return true; return false }

    // Depth first through the content for a row or group whose label or title is `text`
    function findTarget(item, text) {
        if (!item) return null
        if (item.settingTarget === true && item.visible && (item.label === text || item.title === text)) return item
        var kids = item.children
        for (var i = 0; i < kids.length; i++) { var f = findTarget(kids[i], text); if (f) return f }
        return null
    }
    function reveal(text) {
        var t = findTarget(layout, text)
        if (!t) return false
        var y = t.mapToItem(root.contentItem, 0, 0).y - theme.space(10)
        root.contentY = Math.max(0, Math.min(y, root.contentHeight - root.height))
        t.pulse()
        return true
    }

    anchors.fill: parent
    contentWidth: width
    contentHeight: layout.y + layout.height + footInset
    clip: true
    boundsBehavior: Flickable.StopAtBounds
    WheelStep { flick: root }
    MouseArea {
        id: sink
        width: root.width
        height: Math.max(root.height, root.contentHeight)
        onPressed: sink.forceActiveFocus()
    }
    Column {
        id: heading
        x: root.blockX
        width: root.blockWidth
        spacing: theme.space(2)
        Label {
            text: root.title
            role: "display"
            font.pointSize: theme.typeHeading
        }
        Label {
            visible: root.lead !== ""
            width: Math.min(parent.width, theme.space(180))
            text: root.lead
            color: theme.textDim
            wrapMode: Text.Wrap
            role: "ui"
        }
    }
    ColumnLayout {
        id: layout
        readonly property bool grows: root.anyGrows(children)
        x: root.blockX
        y: heading.height + theme.space(9)
        width: root.blockWidth
        height: grows ? Math.max(root.viewport, implicitHeight) : implicitHeight
        spacing: root.gap
    }
    QC.ScrollBar.vertical: ThinScrollBar {}
}
