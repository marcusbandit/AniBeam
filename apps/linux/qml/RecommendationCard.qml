// A recommended series as a small poster pane, written the way Card is: the poster
// full-bleed, the title on the rising scrim at its foot, an Available or AniList tag top
// left, the list-status square bottom right. Hover draws the 1 px accent inner outline.
import QtQuick
import com.marcusrosado.AniBeam
Item {
    id: root
    property var rec: ({})
    signal opened()
    // A Tab stop that opens on Return or Space
    activeFocusOnTab: true
    Keys.onReturnPressed: opened()
    Keys.onEnterPressed: opened()
    Keys.onSpacePressed: opened()
    Rectangle { anchors.fill: parent; anchors.margins: -2; visible: root.activeFocus; radius: theme.radiusTile + 2; antialiasing: true; color: "transparent"; border.width: 2; border.color: theme.focusRing; z: 10 }
    width: theme.space(32)
    height: Math.round(width * 1.5)
    implicitWidth: width
    implicitHeight: height
    scale: theme.reducedMotion ? 1 : (m.pressed ? 0.98 : 1)

    // Rounded like Card: the pane shows only until the poster decodes, then the poster is the
    // Corner's fill, so its corners are cut by the same shape
    Corner {
        anchors.fill: parent
        radius: theme.radiusTile; smoothing: 0
        color: theme.tile
        borderColor: art.status === Image.Ready ? "transparent" : theme.line
        borderWidth: art.status === Image.Ready ? 0 : 1
        fillItem: art.status === Image.Ready ? art : null
        Image {
            id: art
            visible: false
            width: parent.width; height: parent.height
            source: rec.poster ? Fmt.fileUrl(rec.poster) : ""
            fillMode: Image.PreserveAspectCrop; asynchronous: true; smooth: true
            sourceSize: Qt.size(Math.ceil(width * theme.devicePixelRatio), Math.ceil(height * theme.devicePixelRatio))
        }
    }
    Rectangle {
        radius: theme.radiusTile; antialiasing: true
        anchors.bottom: parent.bottom
        width: parent.width
        height: Math.max(parent.height * theme.scrimReach, title.height + theme.space(10))
        gradient: Gradient {
            GradientStop { position: 0; color: "transparent" }
            GradientStop { position: 0.45; color: Qt.rgba(theme.posterScrim.r, theme.posterScrim.g, theme.posterScrim.b, theme.posterScrim.a * (m.containsMouse ? 0.76 : 0.62)) }
            GradientStop { position: 1; color: theme.posterScrim }
        }
    }
    Rectangle {
        x: theme.space(1.5); y: theme.space(1.5)
        width: tag.implicitWidth + theme.space(4); height: tag.implicitHeight + theme.space(1.5)
        radius: theme.radiusSm; antialiasing: true
        color: Qt.rgba(8 / 255, 9 / 255, 11 / 255, 0.62)
        border.color: Qt.rgba(1, 1, 1, 0.14); border.width: 1
        Label { id: tag; anchors.centerIn: parent; role: "ledger"; text: rec.owned ? "Available" : "AniList"; color: rec.owned ? theme.accentOnPoster : theme.onPosterDim }
    }
    Label {
        id: title
        x: theme.space(2.5)
        width: parent.width - theme.space(5) - (dot.visible ? dot.width + theme.space(2) : 0)
        anchors.bottom: parent.bottom; anchors.bottomMargin: theme.space(2.5)
        text: rec.title || ""
        role: "title"; font.pointSize: theme.typeNormal
        color: theme.onPoster
        wrapMode: Text.Wrap; maximumLineCount: 2; elide: Text.ElideRight
        lineHeight: 0.95
    }
    StatusDot { id: dot; anchors.right: parent.right; anchors.bottom: parent.bottom; anchors.margins: theme.space(2.5); status: rec.list_status || "" }
    Rectangle {
        anchors.fill: parent
        visible: m.containsMouse
        radius: theme.radiusTile; antialiasing: true
        color: "transparent"
        border.color: theme.accent; border.width: 1
    }
    MouseArea { id: m; anchors.fill: parent; hoverEnabled: true; cursorShape: Qt.PointingHandCursor; onClicked: root.opened(); Accessible.role: Accessible.Button; Accessible.name: theme.nameNear(parent)
        Accessible.onPressAction: root.opened() }
    Tooltip { text: rec.owned ? "Open " + rec.title + " in your library" : "Open " + rec.title + " on AniList" }
}
