// A character: portrait or a users glyph, name, role in lower case.
import QtQuick
Column {
    property var person: ({})
    width: theme.space(28)
    spacing: theme.space(1)
    Corner {
        width: parent.width; height: width * 1.4
        radius: theme.radiusMd; smoothing: theme.cornerSmoothing; color: theme.surface; borderColor: theme.line; borderWidth: 1
        fillItem: portrait.status === Image.Ready ? portrait : null
        Image { id: portrait; visible: false; width: parent.width; height: parent.height; source: person.image ? "file://" + person.image : ""; fillMode: Image.PreserveAspectCrop; asynchronous: true; sourceSize.width: 240 }
        Icon { visible: !person.image; anchors.centerIn: parent; glyph: "users"; size: theme.space(6); color: theme.textFaint }
    }
    Text { width: parent.width; text: person.name || "Unknown"; color: theme.text; font.family: theme.fontSans; font.pointSize: theme.typeNormal; font.weight: Font.DemiBold; elide: Text.ElideRight }
    Text { width: parent.width; text: (person.role || "").toLowerCase(); color: theme.textFaint; font.family: theme.fontSans; font.pointSize: theme.typeSmall; elide: Text.ElideRight }
}
