// A character as a small round portrait with the name under it and the role in sentence
// case under that. No portrait: a users glyph on the tile fill. One with an AniList id is a
// link to its page there: a pointer, a ring round the portrait and the name lit on hover.
import QtQuick
import com.marcusrosado.AniBeam
Item {
    id: root
    property var person: ({})
    readonly property bool linked: !!person.anilist_id
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
            borderColor: link.containsMouse ? theme.accent : "transparent"; borderWidth: link.containsMouse ? 2 : 0
            scale: theme.reducedMotion ? 1 : (link.pressed ? 0.96 : link.containsMouse ? 1.04 : 1)
            Behavior on scale { enabled: !theme.reducedMotion; NumberAnimation { duration: theme.motionFast } }
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
            color: link.containsMouse ? theme.accent : theme.text
            font.underline: link.containsMouse
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
    MouseArea {
        id: link
        anchors.fill: parent
        enabled: root.linked
        hoverEnabled: enabled
        cursorShape: enabled ? Qt.PointingHandCursor : Qt.ArrowCursor
        onClicked: frame.openLink("https://anilist.co/character/" + root.person.anilist_id)
    }
    Tooltip { text: root.linked ? "Open " + (root.person.name || "the character") + " on AniList" : "" }
}
