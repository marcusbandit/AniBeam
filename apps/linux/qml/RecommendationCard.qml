// A recommended series: poster, an Available or AniList pill, the list-status dot, title.
import QtQuick
Column {
    id: root
    property var rec: ({})
    signal opened()
    width: theme.space(32)
    spacing: theme.space(1)
    Corner {
        width: parent.width; height: width * 1.5
        radius: theme.radiusMd; smoothing: theme.cornerSmoothing; color: theme.surface; borderColor: m.containsMouse ? theme.lineStrong : theme.line; borderWidth: 1
        fillItem: art.status === Image.Ready ? art : null
        Image { id: art; visible: false; width: parent.width; height: parent.height; source: rec.poster ? "file://" + rec.poster : ""; fillMode: Image.PreserveAspectCrop; asynchronous: true; sourceSize.width: 320 }
        Chip { x: theme.space(1.5); y: theme.space(1.5); small: true; mono: false; text: rec.owned ? "Available" : "AniList"; textColor: rec.owned ? theme.accent : theme.textDim }
        StatusDot { anchors.right: parent.right; anchors.bottom: parent.bottom; anchors.margins: theme.space(2); status: rec.list_status || "" }
        MouseArea { id: m; anchors.fill: parent; hoverEnabled: true; cursorShape: Qt.PointingHandCursor; onClicked: root.opened() }
        Tooltip { text: rec.owned ? "Open " + rec.title + " in your library" : "Open " + rec.title + " on AniList" }
    }
    Text { width: parent.width; text: rec.title || ""; color: theme.text; font.family: theme.fontSans; font.pointSize: theme.typeNormal; font.weight: Font.DemiBold; elide: Text.ElideRight; maximumLineCount: 2; wrapMode: Text.Wrap }
}
