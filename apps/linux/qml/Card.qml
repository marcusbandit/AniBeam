// One series as a pane of the glass mosaic: the poster full-bleed, cropped to the rounded
// tile, with everything written on it. A scrim rises from the poster's foot (near-black, clear by
// theme.scrimReach of the height; the only gradient in the shell), and on it, bottom left,
// the title (two lines at most) over one dim ledger line, the score bottom right. The
// episode code is a small square glass tag top left; Dropped, On hold, Hidden and the ghost's
// AniList tag sit top right, and so does a purple Downloading while a torrent is still writing an episode. A dropped series reads quieter, its poster dimmed like a ghost's. Progress is a short rounded line along the foot, inside the padding. Hover
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
// A page that picks several cards at once sets `selectable`: pressing and holding a card, or
// Ctrl+click, asks for it to be picked (`selectToggled`), and while the page is `selecting`
// a plain click, Return or Space picks too instead of opening. A picked card wears a filled
// accent check top left and an accent edge; while selecting, the others show an empty ring.
//
// An airing countdown is one treatment everywhere: "Episode 2" in the dim ledger, then a
// small clock and the time ("4d 18h") in accent text.
//
// Kept for the layout code that measures a probe card: posterHeight is the whole tile, and
// minPlate, heroPlate and plateH are 0 (nothing sits under the poster any more).
// `plateHeight` is accepted and unused.
//
// Drawn natively since #289: CardFace (cpp/cardface.cpp) draws the poster and everything
// written over it in one item, where this file used to build about sixty objects. What
// stays here is what QML does best: the Image that decodes the poster (shared with the
// pixmap cache, off the Qt thread), the hover, the click, the keys, and the hero's Play.

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
    // Dropped on the tracker: the poster reads quieter, as a ghost's does
    readonly property bool dropped: item.list_status === "Dropped"
    property bool hero: false
    property real tileHeight: 0
    property real plateHeight: 0
    property bool live: true
    // Airing now's Not interested (#202): a small x at the top right on hover or focus, and
    // Delete from the keyboard. The page says what it does; the card only asks
    property bool dismissible: false
    // Right-click offers deleting the series' files; off for a sample card that stands for none
    property bool offersDelete: true
    // Picking several at once: see the head of the file
    property bool selectable: false
    property bool selecting: false
    property bool selected: false
    signal opened()
    signal played()
    signal dismissed()
    signal selectToggled()
    // A stop for the page's arrow keys (GridKeys), and a Tab stop after the page's controls
    readonly property bool gridStop: true
    activeFocusOnTab: true
    Keys.onReturnPressed: root.selecting ? root.selectToggled() : root.opened()
    Keys.onEnterPressed: root.selecting ? root.selectToggled() : root.opened()
    Keys.onSpacePressed: root.selecting ? root.selectToggled() : root.played()
    Keys.onDeletePressed: if (root.dismissible) root.dismissed()

    readonly property real radius: theme.radiusTile
    // Kept for the layout code that measures a probe card
    readonly property real posterHeight: tileHeight > 0 ? tileHeight : Math.round(posterWidth * 1.5)
    readonly property real minPlate: 0
    readonly property real heroPlate: 0
    readonly property real plateH: 0
    width: posterWidth
    height: posterHeight
    implicitWidth: posterWidth
    implicitHeight: posterHeight

    // 0 rest, 1 hover; smoothed like every other tracking value
    property real lift: 0
    readonly property real liftTarget: hoverer.hovered ? 1 : 0
    FrameAnimation {
        running: Math.abs(root.lift - root.liftTarget) > 0.002
        onTriggered: root.lift = theme.follow(root.lift, root.liftTarget, frameTime)
    }
    // Hover lifts the tile a touch (smoothed with `lift`); a press pulls it back under
    scale: theme.reducedMotion ? 1 : (press.pressed ? 0.98 : 1 + 0.015 * lift)
    z: lift > 0.01 || activeFocus ? 1 : 0

    HoverHandler {
        id: hoverer
        cursorShape: Qt.PointingHandCursor
        onHoveredChanged: {
            if (root.offersDelete) frame.hoverCard(root, root.item, hovered)
            if (hovered && face.folderName !== "") { tipIntent.interval = frame.tipWarm() ? 1 : 600; tipIntent.restart() }
            else { tipIntent.stop(); frame.releaseTip(root) }
        }
    }
    Component.onDestruction: frame.hoverCard(root, null, false)
    // Focus stands in for the pointer, so Shift+F10 opens the focused card's menu
    onActiveFocusChanged: if (root.offersDelete && !hoverer.hovered) frame.hoverCard(root, root.item, activeFocus)
    Timer { id: tipIntent; interval: 600; onTriggered: frame.showTip(root, face.folderName, root) }

    // The click covers the whole tile; the Play button and the dismiss sit above it
    MouseArea {
        id: press
        anchors.fill: parent
        cursorShape: Qt.PointingHandCursor
        pressAndHoldInterval: 500
        onPressed: root.forceActiveFocus()
        onClicked: function(mouse) {
            tipIntent.stop(); frame.hideTip()
            if (root.selectable && (root.selecting || (mouse.modifiers & Qt.ControlModifier))) root.selectToggled()
            else root.opened()
        }
        // A held press picks the card and starts picking; Qt sends no click after it
        onPressAndHold: if (root.selectable) { tipIntent.stop(); frame.hideTip(); root.selectToggled() }
    }

    // ---- The pane under the poster: only while there is no poster to show. Under a decoded
    // poster its light edge showed through the poster's antialiased corners as a pale rim.
    property bool posterReady: false
    Rectangle {
        visible: !root.posterReady
        anchors.fill: parent
        radius: root.radius
        antialiasing: true
        color: theme.tile
        border.color: theme.line
        border.width: 1
    }

    // ---- The poster: decoded at the shared poster size (the pool's, so a warmed poster is a
    // cache hit, not a second decode), a hero at twice it, off the Qt thread. The Image only
    // decodes; the face draws its texture, cropped to the tile and rounded by a shader.
    Image {
        id: posterImage
        visible: false
        width: root.width; height: root.height
        source: root.live && !!root.item.poster ? Fmt.fileUrl(root.item.poster) : ""
        sourceSize: root.hero ? Qt.size(theme.posterDecodeSize.width * 2, theme.posterDecodeSize.height * 2) : theme.posterDecodeSize
        fillMode: Image.PreserveAspectCrop
        smooth: true
        asynchronous: true
        cache: true
        onStatusChanged: root.posterReady = status === Image.Ready
        Component.onCompleted: root.posterReady = status === Image.Ready
    }

    // ---- Everything written on the poster: one native item
    CardFace {
        id: face
        anchors.fill: parent
        item: root.item
        poster: root.posterReady ? posterImage : null
        posterOpacity: root.ghost || root.dropped ? 0.6 : 1
        metaOverride: root.metaOverride
        countdown: root.countdown
        ghost: root.ghost
        hero: root.hero
        nowMs: root.nowMs
        lift: root.lift
        focused: root.activeFocus
        hideCode: root.selecting || root.selected
        rightInset: dismissLoader.item && dismissLoader.item.visible ? dismissLoader.item.width + theme.space(1) : 0
    }

    // ---- The hero's Play, on the row the face leaves for it
    Loader {
        active: root.hero
        x: face.pad
        y: face.playY
        height: theme.controlHeight
        sourceComponent: Item {
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
    }

    // ---- Picked: an accent edge, and the check top left where the episode code was; while
    // picking, an empty ring on the cards not picked yet. Built only while picking.
    Loader {
        anchors.fill: parent
        active: root.selecting || root.selected
        sourceComponent: Item {
            Rectangle {
                anchors.fill: parent
                visible: root.selected
                radius: root.radius
                antialiasing: true
                color: Qt.rgba(theme.accent.r, theme.accent.g, theme.accent.b, 0.14)
                border.width: 2
                border.color: theme.accent
            }
            Rectangle {
                objectName: "tile-check-" + (root.item.id === undefined ? "" : root.item.id)
                x: face.pad * 0.8; y: face.pad * 0.8
                width: Math.round(theme.space(6)); height: width
                radius: width / 2
                antialiasing: true
                color: root.selected ? theme.accent : Qt.rgba(8 / 255, 9 / 255, 11 / 255, 0.55)
                border.color: root.selected ? theme.accent : Qt.rgba(1, 1, 1, 0.7)
                border.width: root.selected ? 0 : 1.5
                Icon { visible: root.selected; anchors.centerIn: parent; glyph: "check"; size: Math.round(parent.width * 0.66); color: theme.contrast(theme.accent, Qt.rgba(1, 1, 1, 1)) >= 4.5 ? "#ffffff" : "#0d0e11" }
            }
        }
    }

    // ---- Not interested: above the tile's own click, so it takes its own
    Loader {
        id: dismissLoader
        active: root.dismissible
        x: root.width - width - face.pad * 0.8; y: face.pad * 0.8
        sourceComponent: Rectangle {
            id: dismiss
            readonly property real pill: Math.ceil(ledgerMetrics.height + theme.space(1))
            objectName: "tile-dismiss-" + (root.item.id === undefined ? "" : root.item.id)
            visible: !root.selecting && (hoverer.hovered || root.activeFocus)
            width: Math.round(pill + theme.space(1.5)); height: width
            radius: theme.radiusSm
            antialiasing: true
            color: dismissArea.containsMouse ? Qt.rgba(8 / 255, 9 / 255, 11 / 255, 0.85) : Qt.rgba(8 / 255, 9 / 255, 11 / 255, 0.62)
            border.color: dismissArea.containsMouse ? theme.accentOnPoster : Qt.rgba(1, 1, 1, 0.14)
            border.width: 1
            FontMetrics { id: ledgerMetrics; font.family: theme.fontFamily; font.pointSize: theme.typeSmall; font.weight: theme.roleWeight("ledger"); font.variableAxes: theme.roleAxes("ledger") }
            Icon { anchors.centerIn: parent; glyph: "x"; size: Math.round(parent.width * 0.62); color: theme.onPoster }
            MouseArea {
                id: dismissArea
                anchors.fill: parent
                hoverEnabled: true
                cursorShape: Qt.PointingHandCursor
                onClicked: root.dismissed()
            }
            Tooltip { text: "Not interested" }
        }
    }
}
