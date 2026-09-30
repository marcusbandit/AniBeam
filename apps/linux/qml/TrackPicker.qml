// A track picker: Off (for subtitles) plus every track, the current one in the accent.
// Opens above its anchor; a pick closes it; Escape closes it through the frame.
import QtQuick
import QtQuick.Controls.Basic as QC

Item {
    id: root
    property string title: ""
    property var tracks: []
    property int selected: -1
    property bool offRow: false
    property bool open: false
    signal picked(int id)
    anchors.fill: parent
    visible: open
    z: 900

    // Guarded the same way close() is: a second openAt on a picker already up would count
    // itself twice into page.openMenus and the chrome would never hide again.
    function openAt(anchor) {
        if (open) return
        var p = anchor.mapToItem(root, anchor.width / 2, 0)
        panel.x = Math.max(theme.space(2), Math.min(p.x - panel.width / 2, root.width - panel.width - theme.space(2)))
        panel.y = Math.max(theme.space(2), p.y - panel.height - theme.space(2))
        open = true
        page.openMenus++
        frame.escapeStack.push("popover", root)
    }
    function close() { if (!open) return; open = false; page.openMenus--; frame.escapeStack.pop(root); page.showChrome() }
    // A navigation destroys the page and this picker with it, without ever calling close(),
    // so the count and the escape stack would keep an entry whose closer is gone. close()
    // is already guarded for the never-opened case, so this is safe unconditionally.
    Component.onDestruction: root.close()

    MouseArea { anchors.fill: parent; onPressed: root.close() }
    Corner {
        id: panel
        width: theme.space(70); height: Math.min(theme.space(80), column.implicitHeight + theme.space(4))
        radius: theme.radiusMd; smoothing: theme.cornerSmoothing
        color: theme.surfaceRaised; borderColor: theme.lineStrong; borderWidth: 1
        MouseArea { anchors.fill: parent }
        Flickable {
            anchors.fill: parent; anchors.margins: theme.space(2); clip: true
            contentWidth: width; contentHeight: column.implicitHeight
            boundsBehavior: Flickable.StopAtBounds
            QC.ScrollBar.vertical: ThinScrollBar {}
            Column {
                id: column
                width: parent.width
                Text { text: root.title; color: theme.textDim; font.family: theme.fontSans; font.pointSize: theme.typeSmall; font.weight: Font.DemiBold; height: theme.space(6); verticalAlignment: Text.AlignVCenter; leftPadding: theme.space(2) }
                Repeater {
                    model: (root.offRow ? [{ id: -1, label: "Off" }] : []).concat(root.tracks)
                    Corner {
                        required property var modelData
                        width: column.width; height: theme.controlHeight
                        radius: theme.radiusSm; smoothing: theme.cornerSmoothing
                        color: m.containsMouse ? theme.surfacePressed : "transparent"
                        Text { anchors.left: parent.left; anchors.leftMargin: theme.space(2); anchors.verticalCenter: parent.verticalCenter; text: modelData.label; color: modelData.id === root.selected ? theme.accent : theme.text; font.family: theme.fontSans; font.pointSize: theme.typeNormal; font.weight: modelData.id === root.selected ? Font.DemiBold : Font.Normal; elide: Text.ElideRight; width: parent.width - theme.space(4) }
                        MouseArea { id: m; anchors.fill: parent; hoverEnabled: true; cursorShape: Qt.PointingHandCursor; onClicked: { root.picked(modelData.id); root.close() } }
                    }
                }
            }
        }
    }
}
