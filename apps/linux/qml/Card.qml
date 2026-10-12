// One series as a stone: a single rounded tile, the poster on top (2:3, cropped into the
// shape through Corner.fillItem) and, under it in the same tile, a plate in the poster's own
// tone carrying the title and one ledger line with the score at its right. The episode code
// rides the poster's top left, the progress strip its bottom edge. Nothing sits outside the
// tile. Hover lifts the plate one step and edges the tile in the accent; press scales 0.98.
//
// `hero` is the 2x2 tile of the library mosaic: a taller plate with the title in the display
// cut, the next episode, and a Play button (`played`). `plateHeight` lets the mosaic land the
// tile on its module; 0 is the smallest plate that holds its lines (the Flow pages). `live`
// false drops the poster (the mosaic's tiles far outside the viewport).
import QtQuick
import com.marcusrosado.AniBeam

Item {
    id: root
    property var item: ({})
    property real posterWidth: theme.posterWidth
    property real nowMs: Date.now()
    // Overrides for the pages that own the card's story: the feed says what
    // brought a series here ("Episode aired 3d ago") and when its next
    // episode lands; the watching page dates an entry from AniList. Empty
    // leaves the library's own readouts standing.
    property string metaOverride: ""
    property string countdownOverride: ""
    // An external card: the series is not in the library (the watching page's
    // AniList-only entries), so the plate reads quieter and a chip names where it lives.
    property bool ghost: false
    property bool hero: false
    property real plateHeight: 0
    property bool live: true
    signal opened()
    signal played()

    readonly property real posterHeight: Math.round(posterWidth * 1.5)
    readonly property real pad: theme.space(hero ? 4 : 2.5)
    // The smallest plate that holds two title lines and the ledger line
    readonly property real minPlate: Math.ceil(pad * 2 + titleMetrics.height * 2 + theme.space(1.5) + ledgerMetrics.height)
    readonly property real plateH: plateHeight > 0 ? plateHeight : minPlate
    width: posterWidth
    height: posterHeight + plateH
    implicitWidth: posterWidth
    implicitHeight: posterHeight + plateH

    FontMetrics { id: titleMetrics; font.family: theme.fontFamily; font.pointSize: theme.typeNormal; font.weight: theme.roleWeight("title"); font.variableAxes: theme.roleAxes("title") }
    FontMetrics { id: ledgerMetrics; font.family: theme.fontFamily; font.pointSize: theme.typeSmall; font.weight: theme.roleWeight("ledger"); font.variableAxes: theme.roleAxes("ledger") }

    readonly property string displayTitle: item.title || ""
    readonly property string folderName: item.titles ? item.titles.folder || "" : ""
    readonly property bool hasWatched: item.watched !== null && item.watched !== undefined
    readonly property bool totalKnown: item.total_episodes !== null && item.total_episodes !== undefined && item.total_episodes > 0
    readonly property real watchedPct: item.strip ? item.strip.watched : 0
    readonly property real airedPct: item.strip ? item.strip.watched + item.strip.aired_unwatched : 0
    readonly property real unknownPct: item.strip ? item.strip.unknown : 0
    readonly property string epBadge: item.code || ""
    readonly property bool hasScore: item.my_score !== null && item.my_score !== undefined
                                     || item.community_score !== null && item.community_score !== undefined
    readonly property bool mine: item.my_score !== null && item.my_score !== undefined
    readonly property string scoreText: mine ? Number(item.my_score).toFixed(1)
                                      : hasScore ? Number(item.community_score).toFixed(1) : ""

    // "2d 5h", "4h 12m", "12m": short enough to share a plate line with the episode
    function shortCountdown(secs) {
        var t = Math.max(0, Math.floor(secs))
        var d = Math.floor(t / 86400), h = Math.floor(t % 86400 / 3600), m = Math.floor(t % 3600 / 60)
        return d > 0 ? d + "d " + h + "h" : h > 0 ? h + "h " + m + "m" : Math.max(1, m) + "m"
    }
    readonly property bool airsAhead: !!item.next_airing && item.next_airing.at * 1000 > nowMs
    readonly property string airingText: airsAhead ? "Episode " + item.next_airing.episode + " in " + shortCountdown(item.next_airing.at - nowMs / 1000) : ""
    readonly property string watchedText: hasWatched ? (totalKnown ? item.watched + " of " + item.total_episodes + (item.total_is_estimate ? "+" : "") : item.watched + " watched") : ""
    readonly property string filesText: Fmt.plural(item.episodes_on_disk || 0, "file", "files")
    // The one ledger line: what the owning page says, else the airing countdown, else the
    // progress, else what is on disk. The countdown is the accent.
    readonly property bool ledgerAccent: countdownOverride !== "" || (metaOverride === "" && airingText !== "")
    readonly property string ledgerText: countdownOverride !== "" ? countdownOverride
                                       : metaOverride !== "" ? metaOverride
                                       : airingText !== "" ? airingText
                                       : watchedText !== "" ? watchedText : filesText
    // The hero's next episode: one past what was watched while the series is in progress
    readonly property string nextText: {
        if (!hasWatched || item.watched_state === "CaughtUp") return ""
        var w = item.watched
        if (totalKnown && w >= item.total_episodes) return ""
        return "Episode " + (w + 1) + " next"
    }

    // The poster's tone, read only once the tile is live (it decodes the poster at 8x12)
    readonly property string tone: live && item.poster ? Fmt.posterTone(item.poster) : ""
    readonly property color plateRest: ghost ? theme.tile : tone !== "" ? theme.plate(tone) : theme.raised
    readonly property color plateLift: ghost ? theme.tileHover : tone !== "" ? theme.plateHover(tone) : theme.raisedHover

    // A colour for text on the plate: itself where it holds 4.5:1 there, else the text colour
    function legible(c) { return theme.contrast(Qt.color(c), plate.color) >= 4.5 ? c : theme.text }

    // 0 rest, 1 hover; smoothed like every other tracking value
    property real lift: 0
    readonly property real liftTarget: hoverer.hovered ? 1 : 0
    FrameAnimation {
        running: Math.abs(root.lift - root.liftTarget) > 0.002
        onTriggered: root.lift = theme.follow(root.lift, root.liftTarget, frameTime)
    }
    scale: press.pressed ? 0.98 : 1
    Behavior on scale { enabled: !theme.reducedMotion; NumberAnimation { duration: theme.motionFast } }

    HoverHandler {
        id: hoverer
        cursorShape: Qt.PointingHandCursor
        onHoveredChanged: {
            if (hovered && root.folderName !== "") tipIntent.start()
            else { tipIntent.stop(); frame.hideTip() }
        }
    }
    Timer { id: tipIntent; interval: 600; onTriggered: frame.showTip(titleText, root.folderName) }
    onFolderNameChanged: if (hoverer.hovered && folderName === "") frame.hideTip()

    // The click covers the whole stone; the Play button sits above it and takes its own
    MouseArea {
        id: press
        anchors.fill: parent
        cursorShape: Qt.PointingHandCursor
        onClicked: { tipIntent.stop(); frame.hideTip(); root.opened() }
    }

    // ---- The poster: the top of the stone, square along its bottom edge where the plate
    // meets it. Built only while live.
    Loader {
        id: posterLoader
        width: root.posterWidth
        height: root.posterHeight
        active: root.live
        sourceComponent: Item {
            // Decoded at the shared poster size (the pool's, so a warmed poster is a cache
            // hit, not a second decode); a hero at twice it.
            Image {
                id: poster
                visible: false
                width: shape.width
                height: shape.height
                source: root.item.poster ? "file://" + root.item.poster : ""
                sourceSize: root.hero ? Qt.size(theme.posterDecodeSize.width * 2, theme.posterDecodeSize.height * 2) : theme.posterDecodeSize
                fillMode: Image.PreserveAspectCrop
                smooth: true
                asynchronous: true
                cache: true
            }
            // Named shape, not frame: an id may not reuse a context-chain name (frame,
            // theme, page, window, nav), or everything below resolves that name here.
            Corner {
                id: shape
                width: root.posterWidth
                height: root.posterHeight
                radius: theme.radiusTile
                smoothing: theme.cornerSmoothing
                flatBottom: true
                color: theme.tile
                fillItem: poster.status === Image.Ready ? poster : null
            }
        }
    }
    Label {
        visible: !root.item.poster
        x: 0; width: root.posterWidth; y: root.posterHeight / 2 - height / 2
        horizontalAlignment: Text.AlignHCenter
        text: "No poster"
        color: theme.textFaint
        role: "ui"
        font.pointSize: theme.typeSmall
    }

    Chip {
        visible: root.epBadge !== ""
        x: theme.space(2); y: theme.space(2)
        text: root.epBadge
        small: !root.hero
    }
    Chip {
        visible: root.ghost
        x: theme.space(2)
        y: root.posterHeight - height - theme.space(2) - (strip.visible ? strip.height : 0)
        text: root.item.site_url ? "AniList" : ""
        small: true
        textColor: theme.text
    }
    Chip {
        visible: !!root.item.hidden
        text: "Hidden"
        small: true
        textColor: theme.textDim
        x: root.posterWidth - width - theme.space(2)
        y: root.posterHeight - height - theme.space(2) - (strip.visible ? strip.height : 0)
    }

    // Progress strip along the poster's bottom edge, on the seam with the plate: a base
    // track, the unknown remainder from the right, aired under watched from the left
    Item {
        id: strip
        visible: root.hasWatched && !root.ghost
        y: root.posterHeight - height
        width: root.posterWidth
        height: Math.max(3, theme.space(1))
        Rectangle { width: parent.width; height: parent.height; color: root.totalKnown ? theme.scrim : theme.line }
        Rectangle { anchors.right: parent.right; width: parent.width * root.unknownPct; height: parent.height; color: theme.line; visible: root.unknownPct > 0 }
        Rectangle { width: parent.width * root.airedPct; height: parent.height; color: theme.behind }
        Rectangle { width: parent.width * root.watchedPct; height: parent.height; color: theme.caughtUp }
    }

    // ---- The plate: the bottom of the stone, in the poster's tone
    Corner {
        id: plate
        y: root.posterHeight
        width: root.posterWidth
        height: root.plateH
        radius: theme.radiusTile
        smoothing: theme.cornerSmoothing
        flatTop: true
        color: theme.tone(root.plateRest, root.plateLift, root.lift)

        Label {
            id: titleText
            x: root.pad; y: root.pad
            width: parent.width - root.pad * 2
            text: root.displayTitle
            role: root.hero ? "display" : "title"
            font.pointSize: root.hero ? theme.typeLarge : theme.typeNormal
            wrapMode: Text.Wrap
            maximumLineCount: 2
            elide: Text.ElideRight
            lineHeight: 0.95
        }

        // Small: one ledger line along the plate's foot, the score at its right
        Item {
            visible: !root.hero
            x: root.pad
            width: parent.width - root.pad * 2
            height: ledger.implicitHeight
            y: parent.height - root.pad - height
            Label {
                id: ledger
                width: Math.min(implicitWidth, parent.width - (score.text !== "" ? score.implicitWidth + theme.space(2) : 0))
                text: root.ledgerText
                role: "ledger"
                color: root.legible(root.ledgerAccent ? theme.accent : theme.textDim)
                elide: Text.ElideRight
            }
            Label {
                id: score
                anchors.right: parent.right
                text: root.scoreText
                role: "ledger"
                color: root.legible(root.mine ? theme.accent : theme.textDim)
            }
        }

        // Hero: the next episode, the airing line, and Play with the score at its right
        Column {
            visible: root.hero
            x: root.pad
            y: titleText.y + titleText.height + theme.space(2)
            width: parent.width - root.pad * 2
            spacing: theme.space(1)
            Label {
                visible: text !== ""
                width: parent.width
                text: root.nextText
                role: "ui"
                elide: Text.ElideRight
            }
            Label {
                visible: text !== ""
                width: parent.width
                text: root.metaOverride !== "" ? root.metaOverride : root.airingText
                role: "ledger"
                color: root.legible(root.airingText !== "" && root.metaOverride === "" ? theme.accent : theme.textDim)
                elide: Text.ElideRight
            }
            Label {
                width: parent.width
                text: root.watchedText !== "" ? root.watchedText : root.filesText
                role: "ledger"
                color: root.legible(theme.textDim)
                elide: Text.ElideRight
            }
        }
        Loader {
            active: root.hero
            x: root.pad
            width: parent.width - root.pad * 2
            y: parent.height - root.pad - height
            height: theme.controlHeight
            sourceComponent: Item {
                Item {
                    objectName: "tile-play-" + root.item.id
                    width: play.width; height: play.height
                    anchors.verticalCenter: parent.verticalCenter
                    Button {
                        id: play
                        // the first hero's Play in view, for a --drive script
                        objectName: root.objectName === "hero-tile" ? "hero-play" : ""
                        primary: true
                        icon: "play"
                        text: "Play"
                        onClicked: root.played()
                    }
                }
                Label {
                    anchors.right: parent.right
                    anchors.verticalCenter: parent.verticalCenter
                    text: root.scoreText
                    role: "ledger"
                    font.pointSize: theme.typeNormal
                    color: root.legible(root.mine ? theme.accent : theme.textDim)
                }
            }
        }
    }

    // The 1 px inner edge on hover, around the whole stone
    Loader {
        anchors.fill: parent
        active: root.lift > 0.01 || root.ghost
        sourceComponent: Corner {
            radius: theme.radiusTile
            smoothing: theme.cornerSmoothing
            color: "transparent"
            borderWidth: 1
            borderColor: root.ghost ? theme.tone(theme.lineStrong, theme.accent, root.lift) : Qt.rgba(theme.accent.r, theme.accent.g, theme.accent.b, root.lift)
        }
    }
}
