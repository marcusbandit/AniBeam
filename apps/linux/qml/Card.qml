// One series in the grid: the poster in a G2 frame with four corner facts and a progress
// strip, the title and a meta line beneath. Hover edges the frame in the accent.
import QtQuick
import com.marcusrosado.AniBeam

Item {
    id: root
    property var item: ({})
    property real posterWidth: 180
    property real nowMs: Date.now()
    // Overrides for the pages that own the card's story: the feed says what
    // brought a series here ("Episode aired 3d ago") and when its next
    // episode lands; the watching page dates an entry from AniList. Empty
    // leaves the library's own readouts standing.
    property string metaOverride: ""
    property string countdownOverride: ""
    // An external card: the series is not in the library (the watching page's
    // AniList-only entries), so the frame reads quieter and a chip names where it lives.
    property bool ghost: false
    signal opened()

    width: posterWidth
    implicitHeight: posterWidth * 1.5 + theme.space(2) + info.implicitHeight

    readonly property string displayTitle: item.title || ""
    readonly property string folderName: item.titles ? item.titles.folder || "" : ""
    readonly property bool hasWatched: item.watched !== null && item.watched !== undefined
    readonly property string watchedLabel: Fmt.watchedChip(hasWatched ? item.watched : -1, item.total_episodes === null || item.total_episodes === undefined ? -1 : item.total_episodes, !!item.total_is_estimate)
    readonly property color watchedColor: item.watched_state === "Behind" ? theme.behind : item.watched_state === "Unknown" ? theme.textDim : theme.caughtUp
    readonly property bool totalKnown: item.total_episodes !== null && item.total_episodes !== undefined && item.total_episodes > 0
    readonly property real watchedPct: item.strip ? item.strip.watched : 0
    readonly property real airedPct: item.strip ? item.strip.watched + item.strip.aired_unwatched : 0
    readonly property real unknownPct: item.strip ? item.strip.unknown : 0
    readonly property string epBadge: item.code || ""
    readonly property string metaText: metaOverride !== "" ? metaOverride : (item.last_viewed_at ? Fmt.relative(item.last_viewed_at, nowMs / 1000) : Fmt.plural(item.episodes_on_disk || 0, "file", "files"))
    readonly property string countdownText: countdownOverride !== "" ? countdownOverride : (item.next_airing && item.next_airing.at * 1000 > nowMs ? Fmt.countdown(item.next_airing.at - nowMs / 1000) : "")

    // The poster, laid out at the shape's size and cropped to it; the shape samples its
    // texture straight through Corner.fillItem. Decoded at the shared poster size (the pool's,
    // so a warmed poster is a cache hit, not a second decode).
    Image {
        id: poster
        visible: false
        width: shape.width
        height: shape.height
        source: item.poster ? "file://" + item.poster : ""
        sourceSize: theme.posterDecodeSize
        fillMode: Image.PreserveAspectCrop
        smooth: true
        asynchronous: true
        cache: true
    }

    // Named shape, not frame: an id may not reuse a context-chain name (frame, theme,
    // page, window, nav), or everything below it in this file resolves that name to this
    // shape instead of the shell's own Frame, including a Tooltip's own internal script.
    Corner {
        id: shape
        x: 0
        y: 0
        width: root.posterWidth
        height: width * 1.5
        radius: theme.radiusSm
        smoothing: 0
        color: theme.surface
        fillItem: poster.status === Image.Ready ? poster : null
        borderColor: hover.containsMouse ? theme.accent : (root.ghost ? theme.lineStrong : theme.line)
        borderWidth: 1

        Text {
            visible: !item.poster
            anchors.centerIn: parent
            text: "No poster"
            color: theme.textFaint
            font.family: theme.fontSans
            font.pointSize: theme.typeSmall
        }

        Chip {
            visible: root.ghost
            x: theme.space(2); anchors.bottom: parent.bottom; anchors.bottomMargin: theme.space(2) + (strip.visible ? strip.height + theme.space(1.5) : 0)
            text: item.site_url ? "AniList" : ""
            small: true
            color: theme.scrim
            textColor: theme.text
        }

        Chip {
            visible: root.epBadge !== ""
            x: theme.space(2); y: theme.space(2)
            text: root.epBadge
        }
        Chip {
            visible: root.watchedLabel !== ""
            anchors.right: parent.right; anchors.rightMargin: theme.space(2)
            y: theme.space(2)
            text: root.watchedLabel
            textColor: root.watchedColor
        }
        Row {
            x: theme.space(2)
            anchors.bottom: parent.bottom
            anchors.bottomMargin: theme.space(2) + (strip.visible ? strip.height + theme.space(1.5) : 0)
            spacing: theme.space(1)
            Chip { visible: item.community_score !== null && item.community_score !== undefined; small: true; text: item.community_score !== null && item.community_score !== undefined ? Number(item.community_score).toFixed(1) : ""; textColor: theme.textDim }
            Chip { visible: item.my_score !== null && item.my_score !== undefined; small: true; text: item.my_score !== null && item.my_score !== undefined ? Number(item.my_score).toFixed(1) : ""; textColor: theme.accent }
        }
        Chip {
            visible: !!item.hidden
            text: "Hidden"
            small: true
            textColor: theme.textDim
            anchors.right: parent.right; anchors.rightMargin: theme.space(2)
            anchors.bottom: parent.bottom; anchors.bottomMargin: theme.space(2) + (strip.visible ? strip.height + theme.space(1.5) : 0)
        }

        // Progress strip: a base track, the unknown remainder from the right, aired
        // underneath watched, both from the left
        Item {
            id: strip
            visible: root.hasWatched
            x: theme.space(2)
            width: parent.width - theme.space(2) * 2
            height: Math.max(2, theme.space(0.75))
            anchors.bottom: parent.bottom
            anchors.bottomMargin: theme.space(2)
            Rectangle { width: parent.width; height: parent.height; color: root.totalKnown ? theme.scrim : theme.line }
            Rectangle { anchors.right: parent.right; width: parent.width * root.unknownPct; height: parent.height; color: theme.line; visible: root.unknownPct > 0 }
            Rectangle { width: parent.width * root.airedPct; height: parent.height; color: theme.behind }
            Rectangle { width: parent.width * root.watchedPct; height: parent.height; color: theme.caughtUp }
        }
    }

    Column {
        id: info
        anchors.top: shape.bottom
        anchors.topMargin: theme.space(2)
        width: root.posterWidth
        spacing: theme.space(0.5)
        Text {
            id: titleText
            width: parent.width
            text: root.displayTitle
            color: hover.containsMouse ? theme.accent : theme.text
            font.family: theme.fontSerif
            font.pointSize: theme.typeNormal
            font.weight: Font.DemiBold
            elide: Text.ElideRight
            maximumLineCount: 2
            wrapMode: Text.Wrap
        }
        Item {
            width: parent.width
            height: metaLeftText.implicitHeight
            Text {
                id: metaLeftText
                anchors.left: parent.left
                // Elided short of the countdown, so the two never overprint
                width: Math.min(implicitWidth, parent.width - (countdownText.text !== "" ? countdownText.implicitWidth + theme.space(2) : 0))
                text: root.metaText
                color: theme.textFaint
                font.family: theme.fontUi
                font.pointSize: theme.typeSmall
                elide: Text.ElideRight
            }
            Text {
                id: countdownText
                anchors.right: parent.right
                text: root.countdownText
                color: theme.accent
                font.family: theme.fontUi
                font.pointSize: theme.typeSmall
                font.weight: Font.DemiBold
            }
        }
    }

    // The folder-name tip rides this MouseArea rather than a nested Tooltip: this one
    // covers the whole card, on top of the title text in paint order, so a second hover
    // area stacked under it would never see the pointer at all. Same 600 ms intent as the
    // Tooltip primitive.
    MouseArea {
        id: hover
        anchors.fill: parent
        hoverEnabled: true
        cursorShape: Qt.PointingHandCursor
        onClicked: { tipIntent.stop(); frame.hideTip(); root.opened() }
        onEntered: if (root.folderName !== "") tipIntent.start()
        onExited: { tipIntent.stop(); frame.hideTip() }
        Timer { id: tipIntent; interval: 600; onTriggered: frame.showTip(titleText, root.folderName) }
    }
    onFolderNameChanged: if (hover.containsMouse && folderName === "") frame.hideTip()
}
