// One show as a row, for the downloads page's lists: the poster as a thumbnail, the title
// with its chips beside it (a coloured dot each), one line of facts with any trouble in
// red after it, the folder in the faint ledger cut, a thin line for what is on disk or how
// far a download has come, and the row's one action at the right (Unsubscribe, Remove),
// which opens the removal sheet. A click elsewhere opens the show.
import QtQuick
import com.marcusrosado.AniBeam

Item {
    id: root
    objectName: "show-row"
    property string poster: ""
    property string title: ""
    // [{ text, dot }]
    property var chips: []
    property string facts: ""
    property string trouble: ""
    property string folder: ""
    property real progress: -1
    // The episode cubes (SeriesCard.cubes); without them a progress fraction draws as steps
    property string cubes: ""
    property string progressCode: "d"
    property color progressColor: theme.accent
    property string actionText: "Remove"
    property string actionIcon: "x"
    property string glyph: "rss"
    property bool busy: false
    signal opened()
    signal action(Item anchor)

    width: parent ? parent.width : theme.space(100)
    height: theme.space(23)
    opacity: busy ? 0.45 : 1

    Rectangle { anchors.fill: parent; radius: theme.radiusSm; color: hov.hovered ? theme.raised : "transparent" }
    HoverHandler { id: hov; cursorShape: Qt.PointingHandCursor }
    MouseArea { anchors.fill: parent; onClicked: root.opened() }

    Corner {
        id: thumb
        x: theme.space(2)
        anchors.verticalCenter: parent.verticalCenter
        width: theme.space(13); height: width * 1.5
        radius: theme.radiusSm; smoothing: 0
        color: theme.tileSunken
        borderColor: theme.line; borderWidth: 1
        Image {
            id: posterImage
            visible: false
            width: thumb.width; height: thumb.height
            source: root.poster !== "" ? Fmt.fileUrl(root.poster) : ""
            sourceSize: theme.posterDecodeSize
            fillMode: Image.PreserveAspectCrop
            smooth: true
            asynchronous: true
            cache: true
        }
        fillItem: posterImage.status === Image.Ready ? posterImage : null
        Icon { visible: posterImage.status !== Image.Ready; anchors.centerIn: parent; glyph: root.glyph; size: theme.space(5); color: theme.textFaint }
    }

    Column {
        anchors.left: thumb.right
        anchors.leftMargin: theme.space(5)
        anchors.right: actionButton.left
        anchors.rightMargin: theme.space(5)
        anchors.verticalCenter: parent.verticalCenter
        spacing: theme.space(1.25)
        Row {
            width: parent.width
            spacing: theme.space(2.5)
            Label {
                width: Math.min(implicitWidth, parent.width - chipRow.width - parent.spacing)
                anchors.verticalCenter: parent.verticalCenter
                text: root.title
                role: "title"
                font.pointSize: theme.typeNormal * 1.12
                elide: Text.ElideRight
            }
            Row {
                id: chipRow
                anchors.verticalCenter: parent.verticalCenter
                spacing: theme.space(1.5)
                Repeater {
                    model: root.chips
                    Rectangle {
                        id: chip
                        required property var modelData
                        radius: theme.radiusSm
                        width: chipInner.implicitWidth + theme.space(2) * 2
                        height: chipInner.implicitHeight + theme.space(0.75) * 2
                        color: theme.raised
                        border.color: theme.line
                        border.width: 1
                        Row {
                            id: chipInner
                            anchors.centerIn: parent
                            spacing: theme.space(1.25)
                            Rectangle { visible: !!chip.modelData.dot; anchors.verticalCenter: parent.verticalCenter; width: theme.space(2); height: width; radius: width / 2; color: chip.modelData.dot || "transparent" }
                            Label { text: chip.modelData.text; role: "ledger"; font.pointSize: theme.typeSmall; color: theme.text }
                        }
                    }
                }
            }
        }
        Row {
            width: parent.width
            spacing: theme.space(2)
            Label { text: root.facts; role: "ui"; color: theme.textDim; elide: Text.ElideRight; width: Math.min(implicitWidth, parent.width - troubleLabel.width - parent.spacing) }
            Label { id: troubleLabel; visible: text !== ""; text: root.trouble; role: "ui"; color: theme.red }
        }
        Label { width: parent.width; visible: text !== ""; text: root.folder; role: "ledger"; color: theme.textFaint; elide: Text.ElideMiddle }
        EpisodeCubes {
            visible: root.cubes !== "" || root.progress >= 0
            width: Math.min(parent.width, theme.space(110))
            height: implicitHeight
            codes: root.cubes !== "" ? root.cubes : theme.cubeSteps(root.progress, 20, root.progressCode, "g")
            cubeSize: theme.space(2)
            minSize: 2
            maxRows: 2
        }
    }

    Button {
        id: actionButton
        objectName: "show-row-action"
        anchors.right: parent.right
        anchors.rightMargin: theme.space(3)
        anchors.verticalCenter: parent.verticalCenter
        visible: !root.busy
        text: root.actionText
        icon: root.actionIcon
        small: true
        onClicked: root.action(actionButton)
    }
    Rectangle { anchors.bottom: parent.bottom; x: theme.space(2); width: parent.width - theme.space(4); height: 1; color: theme.line; opacity: 0.5 }
}
