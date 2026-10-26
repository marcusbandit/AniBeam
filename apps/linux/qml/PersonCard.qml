// A character as a small round portrait with the name under it and the role in sentence
// case under that. No portrait: a users glyph on the tile fill.
import QtQuick
import com.marcusrosado.AniBeam
Item {
    id: root
    property var person: ({})
    width: theme.space(22)
    height: col.implicitHeight
    implicitWidth: width
    implicitHeight: height
    Column {
        id: col
        width: parent.width
        spacing: theme.space(1.5)
        Corner {
            id: portrait
            anchors.horizontalCenter: parent.horizontalCenter
            width: theme.space(16); height: width
            radius: width / 2
            smoothing: 0
            color: theme.tile
            fillItem: art.status === Image.Ready ? art : null
            Image {
                id: art; visible: false
                width: parent.width; height: parent.height
                source: root.person.image ? Fmt.fileUrl(root.person.image) : ""
                sourceSize: Qt.size(Math.ceil(width * theme.devicePixelRatio), Math.ceil(height * theme.devicePixelRatio))
                fillMode: Image.PreserveAspectCrop; asynchronous: true; smooth: true
            }
            Icon { visible: !root.person.image; anchors.centerIn: parent; glyph: "users"; size: theme.space(5); color: theme.textFaint }
        }
        Label {
            width: parent.width
            text: root.person.name || "Unknown"
            role: "ui"
            horizontalAlignment: Text.AlignHCenter
            wrapMode: Text.Wrap; maximumLineCount: 2; elide: Text.ElideRight
            lineHeight: 0.95
        }
        Label {
            width: parent.width
            text: (function(r) { r = String(r || "").toLowerCase(); return r.charAt(0).toUpperCase() + r.slice(1) })(root.person.role)
            role: "ledger"
            color: theme.textDim
            horizontalAlignment: Text.AlignHCenter
            elide: Text.ElideRight
        }
    }
}
