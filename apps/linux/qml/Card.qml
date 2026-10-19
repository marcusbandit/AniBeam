// One series as a pane of the glass mosaic: the poster full-bleed, cropped to the rounded
// tile, with everything written on it. A scrim rises from the poster's foot (near-black, clear by
// theme.scrimReach of the height; the only gradient in the shell), and on it, bottom left,
// the title (two lines at most) over one dim ledger line, the score bottom right. The
// episode code is a small square glass tag top left; Dropped, Hidden and the ghost's AniList
// tag sit top right, and so does a purple Downloading while a torrent is still writing an episode. A dropped series reads quieter, its poster dimmed like a ghost's. Progress is a short rounded line along the foot, inside the padding. Hover
// draws a 1 px accent outline, lifts the tile a touch and deepens the scrim; press scales
// 0.98. Keyboard focus draws that same outline at full strength and a 2 px ring in
// theme.focusRing outside the tile, so focus never reads as hover; Return opens the series,
// Space plays its next episode, and the arrows are the page's (GridKeys through PageScroll).
// A click gives the tile focus so the arrows go on from there; hover never does. A card
// with no poster is a plain pane: no scrim, and the writing in the theme's own inks.
//
// The tile's size comes from the layout, never from the image: `tileHeight` when the
// mosaic sets one, else a 2:3 tile at `posterWidth`. `hero` is the 2x2 tile: the title in
// the display cut, the next episode, the countdown, the progress, and a square Play button
// (`played`). `live` false drops the poster (the mosaic's tiles far outside the viewport).
//
// An airing countdown is one treatment everywhere: "Episode 2" in the dim ledger, then a
// small clock and the time ("4d 18h") in accent text.
//
// Kept for the layout code that measures a probe card: posterHeight is the whole tile, and
// minPlate, heroPlate and plateH are 0 (nothing sits under the poster any more).
// `plateHeight` is accepted and unused.
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
    // { episode, at } in seconds: a countdown the owning page supplies (the feed's
    // scheduled cards); null leaves the series' own next airing
    property var countdown: null
    property alias countdownOverride: root.countdown
    // An external card: the series is not in the library (the watching page's
    // AniList-only entries), so the poster reads quieter and a tag names where it lives.
    property bool ghost: false
    // Dropped on the tracker: the one list status a card names, since every other one is
    // what a series on disk is anyway
    readonly property bool dropped: item.list_status === "Dropped"
    property bool hero: false
    property real tileHeight: 0
    property real plateHeight: 0
    property bool live: true
    signal opened()
    signal played()
    // A stop for the page's arrow keys (GridKeys), and a Tab stop after the page's controls
    readonly property bool gridStop: true
    activeFocusOnTab: true
    Keys.onReturnPressed: root.opened()
    Keys.onEnterPressed: root.opened()
    Keys.onSpacePressed: root.played()

    readonly property real pad: theme.space(hero ? 5 : 3)
    readonly property real radius: theme.radiusTile
    readonly property real pillHeight: Math.ceil(ledgerMetrics.height + theme.space(1))
    readonly property real minPlate: 0
    readonly property real heroPlate: 0
    readonly property real plateH: 0
    readonly property real posterHeight: tileHeight > 0 ? tileHeight : Math.round(posterWidth * 1.5)
    width: posterWidth
    height: posterHeight
    implicitWidth: posterWidth
    implicitHeight: posterHeight

    FontMetrics { id: ledgerMetrics; font.family: theme.fontFamily; font.pointSize: theme.typeSmall; font.weight: theme.roleWeight("ledger"); font.variableAxes: theme.roleAxes("ledger") }

    readonly property string displayTitle: item.title || ""
    // No poster: the card is a plain pane of the surface, so the scrim goes (it read as a
    // grey smear on a light surface) and the writing takes the theme's own inks
    readonly property bool bare: !item.poster
    readonly property color inkMain: bare ? theme.text : theme.onPoster
    readonly property color inkDim: bare ? theme.textDim : theme.onPosterDim
    readonly property color inkAccent: bare ? theme.accent : theme.accentOnPoster
    readonly property string folderName: item.titles ? item.titles.folder || "" : ""
    readonly property bool hasWatched: item.watched !== null && item.watched !== undefined
    readonly property bool totalKnown: item.total_episodes !== null && item.total_episodes !== undefined && item.total_episodes > 0
    // The core's strip scales every share by 0.85 to leave an unknown tail; a card wants the
    // plain fraction, so a finished run fills the line
    readonly property real watchedPct: totalKnown && hasWatched ? Math.min(1, item.watched / item.total_episodes)
                                     : item.strip ? item.strip.watched / 0.85 : 0
    readonly property real airedPct: item.strip ? item.strip.watched + item.strip.aired_unwatched : 0
    readonly property real unknownPct: item.strip ? item.strip.unknown : 0
    readonly property string epBadge: item.code || ""
    readonly property int downloadingCount: item.downloading ? item.downloading.length : 0
    // The downloading purple lifted toward white, so it reads on the dark glass tag
    readonly property color downloadInk: theme.tone(theme.cubeState("t").fill, Qt.rgba(1, 1, 1, 1), 0.3)
    readonly property bool hasScore: item.my_score !== null && item.my_score !== undefined
                                     || item.community_score !== null && item.community_score !== undefined
    readonly property bool mine: item.my_score !== null && item.my_score !== undefined
    readonly property string scoreText: mine ? Number(item.my_score).toFixed(1)
                                      : hasScore ? Number(item.community_score).toFixed(1) : ""

    // "2d 5h", "4h 12m", "12m": short enough to share a line with the episode
    function shortCountdown(secs) {
        var t = Math.max(0, Math.floor(secs))
        var d = Math.floor(t / 86400), h = Math.floor(t % 86400 / 3600), m = Math.floor(t % 3600 / 60)
        return d > 0 ? d + "d " + h + "h" : h > 0 ? h + "h " + m + "m" : Math.max(1, m) + "m"
    }
    // The countdown: the page's own, else the series' next airing while it is ahead and no
    // page story stands in its place
    readonly property var airing: countdown ? countdown
                                : metaOverride === "" && !!item.next_airing && item.next_airing.at * 1000 > nowMs ? item.next_airing : null
    readonly property string airingTime: airing ? shortCountdown(airing.at - nowMs / 1000) : ""
    // A TVmaze season past the first is named, since its numbers restart: "S2 Episode 1"
    readonly property string airingEpisode: airing ? (airing.season && airing.season !== 1 ? "S" + airing.season + " " : "") + Fmt.episodeName("", airing.episode) : ""
    readonly property string watchedText: hasWatched ? (totalKnown ? item.watched + " of " + item.total_episodes + (item.total_is_estimate ? "+" : "") : item.watched + " watched") : ""
    readonly property string filesText: Fmt.plural(item.episodes_on_disk || 0, "file", "files")
    // The one ledger line under the title: the countdown when one runs, else what the owning
    // page says, else the progress, else what is on disk
    readonly property string ledgerText: metaOverride !== "" ? metaOverride
                                       : watchedText !== "" ? watchedText : filesText
    // The hero's next episode: one past what was watched while the series is in progress
    readonly property string nextText: {
        if (!hasWatched || item.watched_state === "CaughtUp") return ""
        var w = item.watched
        if (totalKnown && w >= item.total_episodes) return ""
        return Fmt.episodeName("", w + 1) + " next"
    }

    // The countdown line: the episode in the dim ledger, then a clock and the time in accent
    component Countdown: Row {
        id: cd
        property real room: 0
        height: root.pillHeight
        spacing: theme.space(1.5)
        Label {
            id: cdEpisode
            anchors.verticalCenter: parent.verticalCenter
            width: Math.min(implicitWidth, Math.max(0, cd.room - cdClock.width - cdTime.implicitWidth - cd.spacing * 2 - theme.space(0.5)))
            text: root.airingEpisode
            role: "ledger"
            color: root.inkDim
            elide: Text.ElideRight
        }
        Row {
            anchors.verticalCenter: parent.verticalCenter
            spacing: theme.space(0.5)
            Icon { id: cdClock; anchors.verticalCenter: parent.verticalCenter; glyph: "clock"; size: Math.round(ledgerMetrics.height * 0.85); color: root.inkAccent }
            Label {
                id: cdTime
                anchors.verticalCenter: parent.verticalCenter
                text: root.airingTime
                role: "ledger"
                font.weight: 650
                font.variableAxes: ({ "wdth": 88, "wght": 650 })
                color: root.inkAccent
            }
        }
    }

    // A small rounded tag of dark glass on the poster
    component PosterTag: Rectangle {
        radius: theme.radiusSm
        antialiasing: true
        property string text: ""
        property color ink: theme.onPoster
        implicitWidth: tagLabel.implicitWidth + theme.space(root.hero ? 2.5 : 2) * 2
        implicitHeight: tagLabel.implicitHeight + theme.space(root.hero ? 1 : 0.75) * 2
        width: implicitWidth; height: implicitHeight
        color: Qt.rgba(8 / 255, 9 / 255, 11 / 255, 0.62)
        border.color: Qt.rgba(1, 1, 1, 0.14)
        border.width: 1
        Label {
            id: tagLabel
            anchors.centerIn: parent
            text: parent.text
            role: "ledger"
            font.pointSize: root.hero ? theme.typeNormal * 0.92 : theme.typeSmall
            color: parent.ink
        }
    }

    // 0 rest, 1 hover; smoothed like every other tracking value
    property real lift: 0
    readonly property real liftTarget: hoverer.hovered ? 1 : 0
    FrameAnimation {
        running: Math.abs(root.lift - root.liftTarget) > 0.002
        onTriggered: root.lift = theme.follow(root.lift, root.liftTarget, frameTime)
    }
    // Hover lifts the tile a touch (smoothed with `lift`); a press pulls it back under
    scale: press.pressed ? 0.98 : 1 + 0.015 * lift
    Behavior on scale { enabled: !theme.reducedMotion && press.pressed; NumberAnimation { duration: theme.motionFast } }
    z: lift > 0.01 || activeFocus ? 1 : 0

    HoverHandler {
        id: hoverer
        cursorShape: Qt.PointingHandCursor
        onHoveredChanged: {
            if (hovered && root.folderName !== "") tipIntent.start()
            else { tipIntent.stop(); frame.hideTip() }
        }
    }
    Timer { id: tipIntent; interval: 600; onTriggered: frame.showTip(titleText, root.folderName, root) }
    onFolderNameChanged: if (hoverer.hovered && folderName === "") frame.hideTip()

    // The click covers the whole tile; the Play button sits above it and takes its own
    MouseArea {
        id: press
        anchors.fill: parent
        cursorShape: Qt.PointingHandCursor
        onPressed: root.forceActiveFocus()
        onClicked: { tipIntent.stop(); frame.hideTip(); root.opened() }
    }

    // ---- The pane under the poster: glass, seen only until the poster decodes or when
    // there is none
    Rectangle {
        anchors.fill: parent
        radius: root.radius
        antialiasing: true
        color: theme.tile
        border.color: theme.line
        border.width: 1
    }
    Label {
        visible: root.bare
        anchors.centerIn: parent
        anchors.verticalCenterOffset: -root.height * 0.12
        text: "No poster"
        color: theme.textFaint
        role: "ui"
        font.pointSize: theme.typeSmall
    }

    // ---- The poster: full-bleed, cropped to the rounded tile through a Corner that samples
    // the Image's own texture (no offscreen pass). Built only while live; decoded at the
    // shared poster size (the pool's, so a warmed poster is a cache hit, not a second
    // decode), a hero at twice it.
    Loader {
        anchors.fill: parent
        active: root.live && !!root.item.poster
        opacity: root.ghost || root.dropped ? 0.6 : 1
        sourceComponent: Corner {
            radius: root.radius
            fillItem: posterImage
            Image {
                id: posterImage
                visible: false
                anchors.fill: parent
                source: "file://" + root.item.poster
                sourceSize: root.hero ? Qt.size(theme.posterDecodeSize.width * 2, theme.posterDecodeSize.height * 2) : theme.posterDecodeSize
                fillMode: Image.PreserveAspectCrop
                smooth: true
                asynchronous: true
                cache: true
            }
        }
    }

    // ---- The scrim: rises from the foot, always tall enough to sit under every line
    Rectangle {
        id: scrim
        visible: !root.bare
        anchors.bottom: parent.bottom
        width: parent.width
        height: Math.min(parent.height, Math.max(parent.height * (theme.scrimReach + 0.06 * root.lift),
                                                 foot.height + root.pad * 2 + theme.space(6)))
        radius: root.radius
        antialiasing: true
        gradient: Gradient {
            GradientStop { position: 0; color: "transparent" }
            GradientStop { position: 0.45; color: Qt.rgba(theme.posterScrim.r, theme.posterScrim.g, theme.posterScrim.b, theme.posterScrim.a * (0.7 + 0.12 * root.lift)) }
            GradientStop { position: 1; color: theme.posterScrim }
        }
    }

    // ---- Tags along the top: the episode code left, Dropped, Hidden and AniList right
    PosterTag {
        visible: root.epBadge !== ""
        x: root.pad * 0.8; y: root.pad * 0.8
        text: root.epBadge
    }
    Row {
        x: root.width - width - root.pad * 0.8; y: root.pad * 0.8
        spacing: theme.space(1)
        // An episode out and not seen: the one tag in the accent
        PosterTag { visible: root.item.watched_state === "Behind" && !root.ghost && !root.dropped; text: "New"; ink: theme.accentOnPoster }
        // An episode a torrent is still writing: it is not playable until it is whole
        PosterTag { objectName: "tag-downloading"; visible: root.downloadingCount > 0 && !root.ghost; text: "Downloading"; ink: root.downloadInk }
        PosterTag { visible: root.dropped; text: "Dropped"; ink: theme.onPosterDim }
        PosterTag { visible: !!root.item.hidden; text: "Hidden"; ink: theme.onPosterDim }
        PosterTag { visible: root.ghost && !!root.item.site_url; text: "AniList" }
    }

    // ---- The writing on the foot
    Column {
        id: foot
        x: root.pad
        width: root.width - root.pad * 2
        y: root.height - root.pad - height - (strip.visible ? strip.height + theme.space(1) : 0)
        spacing: theme.space(root.hero ? 1.5 : 1)

        Label {
            id: titleText
            width: parent.width
            text: root.displayTitle
            role: root.hero ? "display" : "title"
            font.pointSize: root.hero ? theme.typeLarge * 1.2 : theme.typeNormal
            color: root.inkMain
            wrapMode: Text.Wrap
            maximumLineCount: 2
            elide: Text.ElideRight
            lineHeight: 0.95
        }

        // Small: one ledger line, the score at its right
        Item {
            visible: !root.hero
            width: parent.width
            height: root.pillHeight
            readonly property real room: width - (score.text !== "" ? score.implicitWidth + theme.space(2) : 0)
            Countdown { visible: root.airing !== null; room: parent.room; anchors.verticalCenter: parent.verticalCenter }
            Label {
                visible: root.airing === null
                anchors.verticalCenter: parent.verticalCenter
                width: Math.min(implicitWidth, parent.room)
                text: root.ledgerText
                role: "ledger"
                color: root.inkDim
                elide: Text.ElideRight
            }
            Label {
                id: score
                anchors.right: parent.right
                anchors.verticalCenter: parent.verticalCenter
                text: root.scoreText
                role: "ledger"
                color: root.mine ? root.inkAccent : root.inkDim
            }
        }

        // Hero: the next episode, the countdown, the page's line, the progress, then Play
        // with the score at its right
        Label {
            visible: root.hero && text !== ""
            width: parent.width
            text: root.nextText
            role: "ui"
            color: root.inkMain
            elide: Text.ElideRight
        }
        Countdown { visible: root.hero && root.airing !== null; room: foot.width }
        Label {
            visible: root.hero && text !== ""
            width: parent.width
            text: root.metaOverride
            role: "ledger"
            color: root.inkDim
            elide: Text.ElideRight
        }
        Label {
            visible: root.hero
            width: parent.width
            text: root.watchedText !== "" ? root.watchedText : root.filesText
            role: "ledger"
            color: root.inkDim
            elide: Text.ElideRight
        }
        Item { visible: root.hero; width: 1; height: theme.space(1.5) }
        Loader {
            active: root.hero
            visible: root.hero
            width: parent.width
            height: active ? theme.controlHeight : 0
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
                    color: root.mine ? root.inkAccent : root.inkDim
                }
            }
        }
    }

    // ---- Progress: the episode cubes along the foot, inside the padding, one row; a long
    // run folds several episodes into a cube rather than wrapping over the title
    EpisodeCubes {
        id: strip
        visible: !root.ghost && (root.item.cubes || "").length > 0
        x: root.pad
        y: parent.height - root.pad * 0.6 - height
        width: parent.width - root.pad * 2
        height: implicitHeight
        codes: root.item.cubes || ""
        cubeSize: theme.space(1.75)
        minSize: 2
        maxRows: 1
        tipLimit: 0
    }

    // ---- The 1 px accent outline on hover, at full strength on focus
    Rectangle {
        anchors.fill: parent
        visible: root.lift > 0.01 || root.activeFocus
        radius: root.radius
        antialiasing: true
        color: "transparent"
        border.width: 1
        border.color: Qt.rgba(theme.accent.r, theme.accent.g, theme.accent.b, root.activeFocus ? 1 : root.lift)
    }
    // ---- The focus ring: 2 px outside the tile, so it is never mistaken for hover
    Rectangle {
        anchors.fill: parent
        anchors.margins: -2
        visible: root.activeFocus
        radius: root.radius + 2
        antialiasing: true
        color: "transparent"
        border.width: 2
        border.color: theme.focusRing
    }
}
