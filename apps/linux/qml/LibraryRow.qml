// The library's row, in the demo's stream idiom: a square thumbnail, the serif title over
// a caps mono subline, and the readouts on the right — watched over total in big mono,
// the airing countdown in the accent. Hairlines separate rows; a hover raises the row.
// The thumb decodes at the shared poster size (a cache hit when the pool or a card has it)
// and fades in fast, so a row scrolled into view paints its picture the moment it lands.
import QtQuick
import com.marcusrosado.AniBeam

Item {
    id: root
    property var item: ({})
    property real nowMs: Date.now()
    signal opened()

    width: parent ? parent.width : 0
    height: theme.space(21)

    readonly property string title: item.title || ""
    readonly property string folderName: item.titles ? item.titles.folder || "" : ""
    readonly property bool hasWatched: item.watched !== null && item.watched !== undefined
    readonly property bool totalKnown: item.total_episodes !== null && item.total_episodes !== undefined && item.total_episodes > 0
    readonly property string readout: hasWatched ? (totalKnown ? item.watched + "/" + item.total_episodes : String(item.watched)) : "—"
    readonly property bool caughtUp: item.watched_state === "CaughtUp"
    readonly property string countdown: item.next_airing && item.next_airing.at * 1000 > nowMs ? (item.next_airing.predicted ? "Predicted in " : "") + Fmt.countdown(item.next_airing.at - nowMs / 1000) : ""
    readonly property string subline: {
        var parts = []
        parts.push(item.format ? item.format.replace(/_/g, " ") : (item.kind === "Movie" ? "Film" : "TV"))
        parts.push(Fmt.plural(item.episodes_on_disk || 0, "file", "files"))
        return parts.join(", ")
    }

    Rectangle { anchors.fill: parent; color: m.containsMouse ? theme.raised : "transparent" }

    Row {
        anchors.left: parent.left; anchors.leftMargin: theme.space(2)
        anchors.verticalCenter: parent.verticalCenter
        spacing: theme.space(4)

        Corner {
            id: thumb
            width: theme.space(11); height: width * 1.5
            anchors.verticalCenter: parent.verticalCenter
            radius: 2; smoothing: 0
            color: theme.tileSunken
            borderColor: theme.line; borderWidth: 1
            Image {
                id: poster
                visible: false
                width: thumb.width; height: thumb.height
                source: root.item.poster ? Fmt.fileUrl(root.item.poster) : ""
                sourceSize: theme.posterDecodeSize
                fillMode: Image.PreserveAspectCrop
                smooth: true
                asynchronous: true
                cache: true
                opacity: 0
                Behavior on opacity { NumberAnimation { duration: theme.motionFast } }
                onStatusChanged: if (status === Image.Ready) opacity = 1
            }
            fillItem: poster.status === Image.Ready ? poster : null
            Text {
                anchors.centerIn: parent
                visible: poster.status !== Image.Ready
                text: "—"
                color: theme.textFaint
                font.family: theme.fontUi
                font.pointSize: theme.typeSmall
            }
        }
        Column {
            anchors.verticalCenter: parent.verticalCenter
            spacing: theme.space(1)
            Text {
                text: root.title
                color: m.containsMouse ? theme.accent : theme.text
                font.family: theme.fontSerif
                font.pointSize: theme.typeNormal
                elide: Text.ElideRight
                width: root.width - theme.space(80)
            }
            Text {
                text: root.subline
                color: theme.textFaint
                font.family: theme.fontUi
                font.pointSize: theme.typeSmall
            }
        }
    }

    Row {
        anchors.right: parent.right; anchors.rightMargin: theme.space(2)
        anchors.verticalCenter: parent.verticalCenter
        spacing: theme.space(4)
        Text {
            visible: root.countdown !== ""
            text: root.countdown
            color: theme.accent
            font.family: theme.fontUi
            font.pointSize: theme.typeSmall
            font.weight: Font.DemiBold
            anchors.verticalCenter: parent.verticalCenter
        }
        Text {
            text: root.readout
            color: root.caughtUp ? theme.accent : (root.hasWatched ? theme.textDim : theme.textFaint)
            font.family: theme.fontUi
            font.pointSize: theme.typeNormal
            anchors.verticalCenter: parent.verticalCenter
        }
    }

    Rectangle {
        anchors.bottom: parent.bottom
        width: parent.width
        height: 1
        color: theme.line
    }

    // The folder-name tip on hover, the Card's way
    MouseArea {
        id: m
        anchors.fill: parent
        hoverEnabled: true
        cursorShape: Qt.PointingHandCursor
        onClicked: { tipIntent.stop(); frame.hideTip(); root.opened() }
        onEntered: if (root.folderName !== "" && root.folderName !== root.title) { tipIntent.interval = frame.tipWarm() ? 1 : 600; tipIntent.restart() }
        onExited: { tipIntent.stop(); frame.releaseTip(root) }
        Timer { id: tipIntent; interval: 600; onTriggered: frame.showTip(root, root.folderName) }
    }
}
