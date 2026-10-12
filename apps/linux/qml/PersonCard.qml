// A character as a small portrait pane: the portrait full-bleed (or a users glyph on glass),
// the name and the role in sentence case written on the rising scrim at its foot, the way
// Card writes a title on a poster.
import QtQuick
Item {
    property var person: ({})
    width: theme.space(28)
    height: Math.round(width * 1.4)
    implicitWidth: width
    implicitHeight: height
    Rectangle { anchors.fill: parent; color: theme.tile; border.color: theme.line; border.width: 1 }
    Image { anchors.fill: parent; source: person.image ? "file://" + person.image : ""; fillMode: Image.PreserveAspectCrop; asynchronous: true; smooth: true; sourceSize.width: 240 }
    Icon { visible: !person.image; anchors.centerIn: parent; anchors.verticalCenterOffset: -parent.height * 0.12; glyph: "users"; size: theme.space(6); color: theme.textFaint }
    Rectangle {
        anchors.bottom: parent.bottom
        width: parent.width
        height: Math.max(parent.height * theme.scrimReach, words.height + theme.space(10))
        gradient: Gradient {
            GradientStop { position: 0; color: "transparent" }
            GradientStop { position: 0.45; color: Qt.rgba(theme.posterScrim.r, theme.posterScrim.g, theme.posterScrim.b, theme.posterScrim.a * 0.62) }
            GradientStop { position: 1; color: theme.posterScrim }
        }
    }
    Column {
        id: words
        x: theme.space(2.5)
        width: parent.width - theme.space(5)
        anchors.bottom: parent.bottom; anchors.bottomMargin: theme.space(2.5)
        spacing: theme.space(0.5)
        Label { width: parent.width; text: person.name || "Unknown"; role: "title"; font.pointSize: theme.typeNormal; color: theme.onPoster; wrapMode: Text.Wrap; maximumLineCount: 2; elide: Text.ElideRight; lineHeight: 0.95 }
        Label { width: parent.width; text: (function(r) { r = String(r || "").toLowerCase(); return r.charAt(0).toUpperCase() + r.slice(1) })(person.role); role: "ledger"; color: theme.onPosterDim; elide: Text.ElideRight }
    }
}
