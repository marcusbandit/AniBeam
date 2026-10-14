// One release in the search results: a selection mark, the title, and its facts in the
// ledger cut (size, seeders and leechers, downloads). A feed that carries no counts
// reports null, which is unknown rather than nought, so those columns say nothing rather
// than 0. A click toggles the selection; Download at the right hands this one release to
// the torrent client on its own. `inGroup` lifts the row while a group chip is hovered.
import QtQuick
import com.marcusrosado.AniBeam

Corner {
    id: root
    property var release: ({})
    property bool selected: false
    property bool inGroup: false
    signal toggled()
    signal downloadOne()

    readonly property var r: release || ({})
    readonly property bool hasCounts: r.seeders !== null && r.seeders !== undefined
    readonly property string peers: hasCounts ? String(r.seeders) + " seeding, " + String(r.leechers === null || r.leechers === undefined ? 0 : r.leechers) + " leeching" : ""
    readonly property string pulls: r.downloads !== null && r.downloads !== undefined ? String(r.downloads) + " downloads" : ""

    width: parent ? parent.width : theme.space(100)
    height: theme.space(11)
    radius: theme.radiusSm
    smoothing: theme.cornerSmoothing
    color: selected ? theme.accentSoft : hover.containsMouse ? theme.raised : inGroup ? theme.tileHover : "transparent"
    borderColor: selected ? Qt.rgba(theme.accent.r, theme.accent.g, theme.accent.b, 0.4) : "transparent"
    borderWidth: 1

    MouseArea {
        id: hover
        anchors.fill: parent
        hoverEnabled: true
        cursorShape: Qt.PointingHandCursor
        onClicked: root.toggled()
    }
    Icon {
        id: mark
        anchors.left: parent.left
        anchors.leftMargin: theme.space(2)
        anchors.verticalCenter: parent.verticalCenter
        glyph: root.selected ? "square-check" : "square"
        size: theme.space(4.5)
        color: root.selected ? theme.accent : hover.containsMouse ? theme.text : theme.textFaint
    }
    Label {
        id: title
        anchors.left: mark.right
        anchors.leftMargin: theme.space(3)
        anchors.right: facts.left
        anchors.rightMargin: theme.space(4)
        anchors.verticalCenter: parent.verticalCenter
        text: root.r.title || ""
        role: "ui"
        color: root.selected ? theme.text : theme.text
        elide: Text.ElideMiddle
    }
    Row {
        id: facts
        anchors.right: dl.visible ? dl.left : parent.right
        anchors.rightMargin: theme.space(3)
        anchors.verticalCenter: parent.verticalCenter
        spacing: theme.space(4)
        Label { visible: text !== ""; text: root.r.size || ""; role: "ledger"; color: theme.textDim; anchors.verticalCenter: parent.verticalCenter }
        Label { visible: text !== ""; text: root.peers; role: "ledger"; color: theme.textDim; anchors.verticalCenter: parent.verticalCenter }
        Label { visible: text !== ""; text: root.pulls; role: "ledger"; color: theme.textFaint; anchors.verticalCenter: parent.verticalCenter }
        Label { visible: !!root.r.source; text: "from " + (root.r.source || ""); role: "ledger"; color: theme.textFaint; anchors.verticalCenter: parent.verticalCenter }
    }
    Button {
        id: dl
        anchors.right: parent.right
        anchors.rightMargin: theme.space(1.5)
        anchors.verticalCenter: parent.verticalCenter
        visible: hover.containsMouse || root.selected
        text: "Download"
        icon: "download"
        flat: true
        small: true
        onClicked: root.downloadOne()
    }
    Rectangle { anchors.bottom: parent.bottom; x: theme.space(2); width: parent.width - theme.space(4); height: 1; color: theme.line; opacity: 0.5 }
}
