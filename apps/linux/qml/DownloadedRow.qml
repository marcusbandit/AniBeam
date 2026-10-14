// One show's downloads: every torrent the client holds for one folder, as one row. The
// poster when the library knows the show, the title over what the torrents add up to
// (how many, how big, what they are doing), Subscribed and Complete beside the title,
// and two ways out at the right, each asking twice in place: Remove torrents takes them
// out of the client and leaves the files, Remove files deletes what they downloaded as
// well. A click elsewhere on the row opens the show inside the page.
import QtQuick
import com.marcusrosado.AniBeam

Item {
    id: root
    // { key, title, folder, show, rule, torrents, size, active, done, broken, upspeed }
    property var group: ({})
    property bool busy: false
    signal opened()
    signal removeTorrents(bool files)

    readonly property var g: group || ({})
    readonly property var show: g.show || null
    readonly property int count: g.torrents ? g.torrents.length : 0
    readonly property string facts: {
        var parts = [Fmt.plural(root.count, "torrent", "torrents"), Fmt.bytes(root.g.size || 0)]
        if (root.g.active > 0) parts.push(root.g.active + " downloading")
        else if (root.g.upspeed > 0) parts.push("seeding at " + Fmt.bytes(root.g.upspeed) + "/s")
        else if (root.g.done === root.count) parts.push("all done")
        return parts.join(", ")
    }
    readonly property string trouble: root.g.broken > 0 ? (root.g.broken === root.count ? "Files missing" : root.g.broken + " with files missing") : ""

    width: parent ? parent.width : theme.space(100)
    height: theme.space(19)
    opacity: busy ? 0.5 : 1

    Rectangle { anchors.fill: parent; radius: theme.radiusSm; color: hov.hovered ? theme.raised : "transparent" }
    HoverHandler { id: hov; cursorShape: Qt.PointingHandCursor }
    MouseArea { anchors.fill: parent; onClicked: root.opened() }

    Corner {
        id: thumb
        x: theme.space(2)
        anchors.verticalCenter: parent.verticalCenter
        width: theme.space(10); height: width * 1.5
        radius: 2; smoothing: 0
        color: theme.tileSunken
        borderColor: theme.line; borderWidth: 1
        Image {
            id: poster
            visible: false
            width: thumb.width; height: thumb.height
            source: root.show && root.show.poster ? "file://" + root.show.poster : ""
            sourceSize: theme.posterDecodeSize
            fillMode: Image.PreserveAspectCrop
            smooth: true
            asynchronous: true
            cache: true
        }
        fillItem: poster.status === Image.Ready ? poster : null
        Icon { visible: poster.status !== Image.Ready; anchors.centerIn: parent; glyph: "folder"; size: theme.space(4.5); color: theme.textFaint }
    }

    Column {
        anchors.left: thumb.right
        anchors.leftMargin: theme.space(4)
        anchors.right: controls.left
        anchors.rightMargin: theme.space(4)
        anchors.verticalCenter: parent.verticalCenter
        spacing: theme.space(1)
        Row {
            width: parent.width
            spacing: theme.space(2)
            Label {
                width: Math.min(implicitWidth, parent.width - tags.width - parent.spacing)
                anchors.verticalCenter: parent.verticalCenter
                text: root.g.title || ""
                role: "title"
                font.pointSize: theme.typeNormal
                elide: Text.ElideRight
            }
            Row {
                id: tags
                anchors.verticalCenter: parent.verticalCenter
                spacing: theme.space(1.5)
                Chip { visible: !!root.g.rule; text: "Subscribed"; icon: "rss"; small: true }
                Chip { visible: !!root.show && root.show.complete; text: "Complete"; small: true; textColor: theme.red }
            }
        }
        Row {
            width: parent.width
            spacing: theme.space(2)
            Label { text: root.facts; role: "ledger"; color: theme.textDim; elide: Text.ElideRight; width: Math.min(implicitWidth, parent.width - troubleLabel.width - parent.spacing) }
            Label { id: troubleLabel; visible: text !== ""; text: root.trouble; role: "ledger"; color: theme.red }
        }
        Label { width: parent.width; text: root.g.folder || ""; role: "ledger"; color: theme.textFaint; elide: Text.ElideMiddle }
    }

    Row {
        id: controls
        anchors.right: parent.right
        anchors.rightMargin: theme.space(2)
        anchors.verticalCenter: parent.verticalCenter
        spacing: theme.space(1.5)
        ConfirmButton {
            objectName: "downloaded-remove-torrents"
            label: "Remove torrents"
            icon: "x"
            small: true
            enabled: !root.busy
            onConfirmed: root.removeTorrents(false)
            Tooltip { text: "Takes " + (root.count === 1 ? "the torrent" : "all " + root.count + " torrents") + " out of the client. The files stay." }
        }
        ConfirmButton {
            objectName: "downloaded-remove-files"
            label: "Remove files"
            icon: "trash-2"
            small: true
            enabled: !root.busy
            onConfirmed: root.removeTorrents(true)
            Tooltip { text: "Takes the torrents out of the client and deletes what they downloaded." }
        }
    }
    Rectangle { anchors.bottom: parent.bottom; x: theme.space(2); width: parent.width - theme.space(4); height: 1; color: theme.line; opacity: 0.5 }
}
