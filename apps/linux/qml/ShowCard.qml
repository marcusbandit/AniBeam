// One show as a pane of the glass mosaic, for the downloads page's grids: the poster
// full-bleed, the title and one ledger line on the scrim at its foot, a thin line for what
// is on disk (or how far a download has come), and a state tag top left: dark glass, a
// coloured dot, light writing, so it reads on any poster. Hovering puts the pane's one
// action (Unsubscribe, Remove) across its foot; it opens the removal sheet. A click
// elsewhere opens the show. `compact` is a pane with no poster to show: a short text tile
// with the glyph, for the shows the library has not found.
import QtQuick
import com.marcusrosado.AniBeam

Item {
    id: root
    objectName: "show-card"
    property string poster: ""
    property string title: ""
    property string tag: ""
    property color tagDot: "transparent"
    property string ledger: ""
    property color ledgerColor: hasPoster ? theme.onPosterDim : theme.textDim
    property real progress: -1
    property color progressColor: hasPoster ? theme.accentOnPoster : theme.accent
    property string actionText: "Remove"
    property string actionIcon: "x"
    property string glyph: "rss"
    property bool compact: false
    property bool busy: false
    signal opened()
    signal action(Item anchor)

    readonly property bool hasPoster: poster !== "" && !compact
    readonly property real pad: theme.space(2.5)
    readonly property color inkMain: hasPoster ? theme.onPoster : theme.text

    implicitHeight: compact ? theme.space(26) : width * 1.5
    opacity: busy ? 0.45 : 1

    property real lift: 0
    FrameAnimation {
        running: Math.abs(root.lift - (hov.hovered ? 1 : 0)) > 0.002
        onTriggered: root.lift = theme.follow(root.lift, hov.hovered ? 1 : 0, frameTime)
    }
    HoverHandler { id: hov; cursorShape: Qt.PointingHandCursor }
    scale: press.pressed ? 0.985 : 1

    Corner {
        anchors.fill: parent
        radius: theme.radiusTile
        color: root.lift > 0.02 ? theme.tone(theme.tile, theme.tileHover, root.lift) : theme.tile
        borderColor: Qt.rgba(theme.accent.r, theme.accent.g, theme.accent.b, 0.6 * root.lift)
        borderWidth: root.lift > 0.02 ? 1 : 0
    }
    Icon {
        visible: !root.hasPoster && !root.compact
        anchors.horizontalCenter: parent.horizontalCenter
        y: parent.height * 0.3
        glyph: root.glyph
        size: theme.space(10)
        color: theme.textFaint
        opacity: 0.6
    }
    Loader {
        anchors.fill: parent
        active: root.hasPoster
        sourceComponent: Corner {
            radius: theme.radiusTile
            fillItem: posterImage
            Image {
                id: posterImage
                visible: false
                anchors.fill: parent
                source: "file://" + root.poster
                sourceSize: theme.posterDecodeSize
                fillMode: Image.PreserveAspectCrop
                smooth: true
                asynchronous: true
                cache: true
            }
        }
    }
    Rectangle {
        visible: root.hasPoster
        anchors.bottom: parent.bottom
        width: parent.width
        height: parent.height * (theme.scrimReach + 0.08 * root.lift)
        radius: theme.radiusTile
        antialiasing: true
        gradient: Gradient {
            GradientStop { position: 0; color: "transparent" }
            GradientStop { position: 0.45; color: Qt.rgba(theme.posterScrim.r, theme.posterScrim.g, theme.posterScrim.b, theme.posterScrim.a * (0.7 + 0.12 * root.lift)) }
            GradientStop { position: 1; color: theme.posterScrim }
        }
    }
    MouseArea { id: press; anchors.fill: parent; onClicked: root.opened() }

    // ---- The state tag
    Rectangle {
        visible: root.tag !== ""
        x: root.pad * 0.8; y: root.pad * 0.8
        radius: theme.radiusSm
        antialiasing: true
        width: tagRow.implicitWidth + theme.space(2) * 2
        height: tagRow.implicitHeight + theme.space(0.9) * 2
        color: root.hasPoster ? Qt.rgba(8 / 255, 9 / 255, 11 / 255, 0.72) : theme.raised
        border.color: root.hasPoster ? Qt.rgba(1, 1, 1, 0.14) : theme.line
        border.width: 1
        Row {
            id: tagRow
            anchors.centerIn: parent
            spacing: theme.space(1.25)
            Rectangle { visible: root.tagDot.a > 0; anchors.verticalCenter: parent.verticalCenter; width: theme.space(2); height: width; radius: width / 2; color: root.tagDot }
            Label { text: root.tag; role: "ledger"; font.pointSize: theme.typeSmall; color: root.hasPoster ? theme.onPoster : theme.text }
        }
    }
    Icon {
        visible: root.compact
        x: parent.width - width - root.pad
        y: root.pad
        glyph: root.glyph
        size: theme.space(4.5)
        color: theme.textFaint
    }

    // ---- The foot
    Column {
        x: root.pad
        width: root.width - root.pad * 2
        y: root.height - root.pad - height
        spacing: theme.space(1)
        Label {
            width: parent.width
            text: root.title
            role: "title"
            font.pointSize: theme.typeNormal
            color: root.inkMain
            wrapMode: Text.Wrap
            maximumLineCount: 2
            elide: Text.ElideRight
            lineHeight: 0.95
        }
        Label {
            visible: !actionButton.visible || root.compact
            width: parent.width
            text: root.ledger
            role: "ledger"
            color: root.ledgerColor
            elide: Text.ElideRight
        }
        Item {
            visible: root.progress >= 0 && !actionButton.visible
            width: parent.width
            height: theme.space(1)
            Rectangle { anchors.fill: parent; radius: height / 2; color: root.hasPoster ? Qt.rgba(1, 1, 1, 0.16) : theme.line }
            Rectangle {
                width: parent.width * Math.max(0, Math.min(1, root.progress))
                height: parent.height
                radius: height / 2
                color: root.progressColor
            }
        }
        Button {
            id: actionButton
            objectName: "show-card-action"
            visible: !root.busy && hov.hovered
            width: parent.width
            text: root.actionText
            icon: root.actionIcon
            small: true
            ghost: root.hasPoster
            flat: !root.hasPoster
            onClicked: root.action(actionButton)
        }
    }
}
