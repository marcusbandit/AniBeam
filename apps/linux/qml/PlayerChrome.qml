// The player's furniture: the header, the seek preview, the controls island, the skip intro
// tile, the auto-next pill and the replay button. Split out of PlayerPage.qml so the page
// holds the session, the mpv wiring and the rules while this file holds what they look like.
// Every value drawn here is a property on `page`, reached through the context chain, and
// every control calls one of the page's own functions; this file keeps no state of its own
// beyond the preview's.
//
// Glass mosaic: the chrome is two square panes of the video glass (black at 0.55, a faint
// edge, the top highlight) set a grout in from the window's edge, the header at the top left
// as small as its text and the controls island at the foot. Nothing else is drawn on the
// picture except the seek preview, the skip intro tile, the auto-next tile, the replay button
// and the page's own notices, each a pane of its own; there is no scrim and no blur.
// PlayerPage forces its tokens dark, so this reads the same on a light theme.
import QtQuick
import QtQuick.Window
import com.marcusrosado.AniBeam

Item {
    id: root
    objectName: "player-chrome"
    anchors.fill: parent
    // Where the controls tile starts, for the page's subtitle lift. Its geometry holds while
    // it is hidden, so this is the same number whether or not the chrome is up.
    readonly property real controlsTop: controls.y
    // A held scrub keeps the chrome up, even with the pointer still.
    readonly property bool scrubbing: seek.scrubbing

    // ---- Header: Back, then the series, the episode and its code, in one small tile.
    Tile {
        id: header
        objectName: "player-header"
        raised: true
        color: theme.glassVideo; borderColor: Qt.rgba(1, 1, 1, 0.1)
        padding: theme.space(3)
        anchors.left: parent.left; anchors.top: parent.top
        anchors.leftMargin: theme.grout; anchors.topMargin: theme.grout
        width: headerRow.implicitWidth + padding * 2
        // As tall as its three lines, so a larger system font or a denser scale grows it.
        height: headerRow.implicitHeight + padding * 2
        opacity: page.chromeVisible ? 1 : 0
        // Zero opacity still takes clicks, so the hidden chrome has to leave the scene.
        visible: opacity > 0
        Behavior on opacity { NumberAnimation { duration: theme.motionNormal } }
        // The picture's MouseArea is under the tile: a click on its padding must not pause.
        MouseArea { enabled: page.chromeVisible; anchors.fill: parent; anchors.margins: -header.padding; hoverEnabled: true; onPositionChanged: page.showChrome() }
        Row {
            id: headerRow
            anchors.verticalCenter: parent.verticalCenter
            spacing: theme.space(3)
            PlayerButton { objectName: "player-back"; glyph: "chevron-left"; tip: "Back"; anchors.verticalCenter: parent.verticalCenter; onClicked: page.leave() }
            Column {
                id: headerText
                anchors.verticalCenter: parent.verticalCenter
                // As wide as the longest line, capped so a long romaji title elides rather
                // than pushing the tile across the picture.
                readonly property real cap: Math.max(theme.space(40), root.width * 0.4)
                width: Math.min(cap, Math.max(seriesLine.implicitWidth, episodeLine.implicitWidth, codeLine.implicitWidth))
                Label { id: seriesLine; width: parent.width; elide: Text.ElideRight; role: "title"; font.pointSize: theme.typeNormal; text: page.session ? page.session.series_title : "" }
                Label { id: episodeLine; width: parent.width; elide: Text.ElideRight; role: "ui"; text: page.episodeLine }
                Label { id: codeLine; width: parent.width; elide: Text.ElideRight; role: "ledger"; color: theme.textDim; text: page.session ? page.session.code : "" }
            }
        }
    }

    // ---- The seek preview: a second mpv core, nothing audible, moved by time-pos. It is
    // its own core rather than a thumbnail file, so it costs nothing on disk and answers
    // any position; it is declared above the video and below the island so it can never
    // cover the controls it sits over.
    Tile {
        id: preview
        raised: true
        color: theme.glassVideo; borderColor: Qt.rgba(1, 1, 1, 0.1)
        padding: theme.space(1)
        // It hides with the chrome as well as on exit: the island can go while the pointer
        // rests on the bar, and the bar goes with it, so no exit would ever arrive.
        property bool shown: false
        visible: shown && page.chromeVisible
        width: theme.space(60); height: (width - padding * 2) * 9 / 16 + padding * 2 + theme.space(6)
        y: controls.y - height - theme.grout
        property bool loaded: false
        function show(secs, centerX) {
            x = Math.max(theme.grout, Math.min(centerX - width / 2, page.width - width - theme.grout))
            stamp.text = Fmt.clock(secs)
            shown = true
            if (loaded) previewVideo.setPropertyAsync("time-pos", secs)
        }
        function hide() { shown = false }
        VideoItem {
            id: previewVideo
            anchors.left: parent.left; anchors.right: parent.right; anchors.top: parent.top
            height: width * 9 / 16
            onReady: {
                var o = Player.previewOptions
                for (var i = 0; i < o.length; i++) setProperty(o[i][0], o[i][1])
                if (page.session) command(["loadfile", page.session.path])
            }
            onLoaded: preview.loaded = true
        }
        Label { id: stamp; anchors.bottom: parent.bottom; anchors.horizontalCenter: parent.horizontalCenter; height: theme.space(6); verticalAlignment: Text.AlignVCenter; role: "ledger" }
    }

    // Whether the volume shows in the controls; a narrow window hides it (PlayerPage's
    // setVolume then flashes the level).
    readonly property bool volumeShown: volumeRow.visible

    // ---- The overflow (#518). Below about 440 px the right-hand group no longer fits beside
    // play, the skips and the mute button, and slid over them. Then the group folds into one
    // More button that opens the frame's menu with the same actions; play, the skips and the
    // mute button stay, and the volume gets back the room the group gave up.
    //
    // The group, one entry a button: its name, glyph and tip, whether it applies, whether it
    // is lit, what it does from a given anchor, and its words in the menu when they differ
    // from the tip. The buttons, the width the fold is decided on and the menu's rows all read
    // this one list. Each field is a function, so a binding that calls one follows what it
    // reads, and the list itself never changes, so the buttons are made once.
    readonly property bool fullscreen: frame.hostWindow.visibility === Window.FullScreen
    readonly property var group: [
        // Shown with no tracks to choose as well: one audio track can still be out of sync.
        { name: "player-delays", glyph: () => "clock", tip: () => "Subtitle and audio delay", shown: () => true,
          active: () => page.subDelay !== 0 || page.audioDelay !== 0, run: anchor => page.openDelays(anchor) },
        { name: "player-audio", glyph: () => "audio-lines", tip: () => "Audio track", shown: () => page.audioTracks.length > 1,
          active: () => false, run: anchor => page.openAudioPicker(anchor) },
        { name: "player-subs", glyph: () => "captions", tip: () => page.sid >= 0 ? "Subtitles" : "Subtitles off", shown: () => page.subTracks.length > 0,
          active: () => page.sid >= 0, run: anchor => page.openSubPicker(anchor) },
        { name: "player-cast", glyph: () => "cast", tip: () => page.casting ? "Casting to " + page.castDevice.name : "Cast to a TV", label: () => page.casting ? "Casting to " + page.castDevice.name : "Cast to a TV",
          shown: () => true, active: () => page.casting, run: anchor => page.openCastPicker(anchor) },
        { name: "player-mark", glyph: () => "check-check", tip: () => "Mark watched", shown: () => page.canMark,
          active: () => false, run: () => page.markWatched() },
        { name: "player-help", glyph: () => "keyboard", tip: () => "Keyboard shortcuts", shown: () => true,
          active: () => false, run: () => page.showHelp() },
        { name: "player-fullscreen", glyph: () => root.fullscreen ? "minimize" : "maximize", tip: () => "Fullscreen",
          label: () => root.fullscreen ? "Exit fullscreen" : "Fullscreen", shown: () => true,
          active: () => false, run: () => page.toggleFullscreen() }
    ]
    readonly property var shownGroup: group.filter(e => e.shown())
    // What the row has between the mute button and a right-hand group `groupWidth` wide, with
    // a gap on each side. Measured from the mute button, since a Row leaves a hidden child's x
    // where it was. The volume takes its room from this, and the fold is decided on it.
    function roomFor(groupWidth) { return bottomRow.width - (muteBtn.x + muteBtn.width) - groupWidth - 3 * bottomRow.spacing }
    // The whole group's width when it shows: from which buttons apply, never read off the
    // row, which is one button wide once folded, so reading it would be a binding loop.
    readonly property real groupWidth: shownGroup.length * moreBtn.width + (shownGroup.length - 1) * rightGroup.spacing
    // Folded once the whole group would leave the volume less than its readout, the point
    // where the volume would otherwise go.
    readonly property bool folded: roomFor(groupWidth) < volumeRow.readoutWidth
    readonly property var menuRows: shownGroup.map(e => ({ text: e.label ? e.label() : e.tip(), icon: e.glyph(), action: () => e.run(moreBtn) }))
    // While the menu is up the chrome stays, as it does for the pickers, and its rows follow
    // the tracks, which can arrive after it opened.
    property bool moreOpen: false
    onMenuRowsChanged: if (moreOpen) frame.menuPanel.items = menuRows
    function openMore() {
        if (!moreOpen) page.openMenus++
        moreOpen = true
        var p = moreBtn.mapToItem(frame, moreBtn.width, -theme.space(1))
        frame.openMenu(p.x, p.y, menuRows, true)
    }
    Connections {
        target: frame.menuPanel
        function onOpenChanged() {
            if (frame.menuPanel.open || !root.moreOpen) return
            root.moreOpen = false
            page.openMenus--
            page.showChrome()
        }
    }
    // A fold or an unfold hides the button the menu, a picker or the delays were opened from,
    // and they would point at where it was.
    onFoldedChanged: {
        if (moreOpen) frame.closeMenu()
        page.closePopovers()
    }
    // A navigation (auto-next, Back) takes the page and this with it while the frame's menu
    // stays, and its rows would call into a page that is gone. Focus is not handed back to
    // the page going away.
    Component.onDestruction: {
        if (!moreOpen) return
        moreOpen = false
        frame.menuPanel.prevFocus = null
        frame.closeMenu()
    }

    // ---- Controls island: the time as a ledger line, the seek groove, then the buttons.
    Tile {
        id: controls
        objectName: "player-controls"
        raised: true
        color: theme.glassVideo; borderColor: Qt.rgba(1, 1, 1, 0.1)
        padding: theme.space(3)
        anchors.horizontalCenter: parent.horizontalCenter
        anchors.bottom: parent.bottom; anchors.bottomMargin: theme.grout
        width: Math.min(parent.width - 2 * theme.grout, theme.space(220))
        height: ledgerRow.height + theme.space(1) + seek.height + theme.space(2) + bottomRow.height + padding * 2
        opacity: page.chromeVisible ? 1 : 0
        visible: opacity > 0
        Behavior on opacity { NumberAnimation { duration: theme.motionNormal } }
        // Keeps the chrome awake while the pointer rests anywhere on the tile, its padding
        // included, and keeps a click on the tile off the picture (which would pause).
        MouseArea { enabled: page.chromeVisible; anchors.fill: parent; anchors.margins: -controls.padding; hoverEnabled: true; onPositionChanged: page.showChrome() }

        Item {
            id: ledgerRow
            anchors.left: parent.left; anchors.right: parent.right; anchors.top: parent.top
            height: elapsed.implicitHeight
            Label { id: elapsed; objectName: "player-elapsed"; anchors.left: parent.left; role: "ledger"; text: Fmt.clock(page.timePos) }
            Label { id: speedNote; objectName: "player-speed"; anchors.horizontalCenter: parent.horizontalCenter; role: "ledger"; color: theme.accent; visible: page.speed !== 1; text: page.speedLine(page.speed) }
            Label { id: total; anchors.right: parent.right; role: "ledger"; color: theme.textDim; text: Fmt.clock(page.duration) }
        }
        // The bar is above the island's own hover MouseArea, so the hover that keeps the
        // chrome up while the pointer sits on the bar comes from the bar itself.
        SeekBar { id: seek; anchors.left: parent.left; anchors.right: parent.right; anchors.top: ledgerRow.bottom; anchors.topMargin: theme.space(1)
            position: page.timePos; duration: page.duration; windows: page.windows; chapters: page.chapters.map(function(c) { return c.start })
            onSeeked: function(s) { page.seekTo(s) }
            onHovered: function(s) { page.showChrome(); preview.show(s, seek.mapToItem(page, page.duration > 0 ? s / page.duration * seek.width : 0, 0).x) }
            onUnhovered: preview.hide() }
        Row {
            id: bottomRow
            anchors.left: parent.left; anchors.right: parent.right; anchors.bottom: parent.bottom
            // No height: a Row is as tall as its tallest child, so the island follows the
            // buttons rather than a number that has to be kept in step.
            spacing: theme.space(1)
            PlayerButton { objectName: "player-prev"; glyph: "skip-back"; interactive: page.hasPrevious; tip: interactive ? "Previous episode" : "No previous episode"; onClicked: page.openPrevious() }
            PlayerButton { objectName: "player-play"; glyph: page.paused ? "play" : "pause"; tip: page.paused ? "Play" : "Pause"; onClicked: page.togglePause() }
            PlayerButton { objectName: "player-next"; glyph: "skip-forward"; interactive: page.hasNext; tip: interactive ? "Next episode" : "No next episode"; onClicked: page.openNext() }
            Item { width: theme.space(2); height: 1 }
            PlayerButton { id: muteBtn; objectName: "player-mute"; glyph: Player.mute || page.volume === 0 ? "volume-x" : "volume-2"; tip: Player.mute ? "Unmute" : "Mute"; onClicked: page.setMute(!Player.mute) }
            // Never on the keyboard: a click on it must leave the arrows to seek and the
            // Up and Down keys to step the volume by 5.
            // On a narrow window the track gives up width first, so the right-hand group
            // never runs into it, and goes when a usable track no longer fits. The readout
            // stays while it fits; when it goes, the page flashes the level a key sets.
            SliderRow { id: volumeRow; objectName: "player-volume"; anchors.verticalCenter: parent.verticalCenter; from: 0; to: 100; value: Player.mute ? 0 : page.volume; stepSize: 1
                // What is left between the mute button and the right-hand group, and what of
                // that the track gets after the readout.
                readonly property real free: root.roomFor(rightGroup.width)
                readonly property real room: free - readoutWidth - spacing
                trackWidth: Math.min(theme.space(24), Math.max(0, room))
                trackShown: room >= theme.space(10)
                visible: free >= readoutWidth
                takesFocus: false; ground: theme.raised; onMoved: function(v) { page.setVolume(v) } }
        }
        // The right-hand group sits on the row's right edge rather than in the row: a slot
        // sized to what the row has left reached zero width on a narrow window, a Row stops
        // placing a child with no width, and the slot then stayed where it was.
        Row { id: rightGroup; anchors.right: bottomRow.right; anchors.verticalCenter: bottomRow.verticalCenter; spacing: theme.space(1)
            Repeater {
                model: root.group
                PlayerButton { id: groupBtn; required property var modelData; objectName: modelData.name
                    glyph: modelData.glyph(); tip: modelData.tip(); active: modelData.active()
                    visible: !root.folded && modelData.shown(); onClicked: modelData.run(groupBtn) }
            }
            // The whole group in one button on a window too narrow for it. Lit like
            // the clock when a delay is set, since the clock it holds would be.
            PlayerButton { id: moreBtn; objectName: "player-more"; glyph: "ellipsis"; tip: "More"; visible: root.folded; active: root.moreOpen || page.subDelay !== 0 || page.audioDelay !== 0; onClicked: root.openMore() } }
    }

    // ---- Auto-next. Not chrome: it stays on screen once the outro starts even after the
    // controls have hidden, since it is the only thing that says what happens next. One
    // raised tile holding Stay and the Next pill, a grout in from the right edge.
    Tile {
        id: nextTile
        objectName: "player-next-tile"
        visible: page.nextVisible
        raised: true
        color: theme.glassVideo; borderColor: Qt.rgba(1, 1, 1, 0.1)
        padding: theme.space(1.5)
        anchors.right: parent.right; anchors.rightMargin: theme.grout
        anchors.bottom: controls.top; anchors.bottomMargin: theme.grout
        width: nextRow.implicitWidth + padding * 2; height: nextRow.implicitHeight + padding * 2
        MouseArea { enabled: page.chromeVisible; anchors.fill: parent; anchors.margins: -nextTile.padding }
        Row {
            id: nextRow
            spacing: theme.space(1.5)
            Button { anchors.verticalCenter: parent.verticalCenter; text: "Stay"; flat: true; takesFocus: false; onClicked: page.stay() }
            Corner {
                anchors.verticalCenter: parent.verticalCenter
                width: nextLabel.implicitWidth + theme.space(8); height: theme.controlHeight
                color: theme.raised; borderColor: theme.accent; borderWidth: 1
                Corner { id: countFill; width: 0; height: parent.height; color: theme.accent; opacity: 0.35
                    // The animation owns the width while it runs and leaves it wherever it
                    // stopped, so a countdown cancelled part way would keep a half filled pill.
                    NumberAnimation on width { running: page.nextCounting; from: 0; to: countFill.parent.width; duration: page.nextCountMs
                        onRunningChanged: if (!running) countFill.width = 0 } }
                Label { id: nextLabel; anchors.centerIn: parent; width: Math.min(implicitWidth, theme.space(90)); elide: Text.ElideRight
                    text: page.nextLine; role: "title"; font.pointSize: theme.typeNormal }
                MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: page.openNext(); Accessible.role: Accessible.Button; Accessible.name: theme.nameNear(parent)
                    Accessible.onPressAction: page.openNext() }
            }
        }
    }

    // ---- Skip intro and Skip outro. Not chrome either, and in the auto-next tile's corner,
    // above it when both are up: each is there for as long as the position is inside its
    // window, whether or not the controls are, and whether or not auto-skip is on. A click
    // lets the controls go rather than calling them up, so the video is clear at once.
    readonly property var skipping: page.inside(page.intro, page.timePos) ? page.intro : page.inside(page.outro, page.timePos) ? page.outro : null
    Tile {
        id: skipTile
        objectName: "player-skip-intro-tile"
        visible: !!root.skipping
        raised: true
        color: theme.glassVideo; borderColor: Qt.rgba(1, 1, 1, 0.1)
        padding: theme.space(1.5)
        anchors.right: parent.right; anchors.rightMargin: theme.grout
        anchors.bottom: nextTile.visible ? nextTile.top : controls.top; anchors.bottomMargin: theme.grout
        width: skipButton.implicitWidth + padding * 2; height: skipButton.implicitHeight + padding * 2
        MouseArea { enabled: page.chromeVisible; anchors.fill: parent; anchors.margins: -skipTile.padding }
        Button { id: skipButton; objectName: root.skipping && root.skipping.kind === "Outro" ? "player-skip-outro" : "player-skip-intro"; anchors.centerIn: parent
            text: root.skipping && root.skipping.kind === "Outro" ? "Skip outro" : "Skip intro"; icon: "chevrons-right"; primary: true; takesFocus: false
            onClicked: page.skipAndHide(root.skipping, page.mapFromItem(null, skipHover.point.scenePosition))
            // Where the pointer is for the click, so the page knows the spot it then rests on.
            HoverHandler { id: skipHover } }
    }

    // ---- Casting (#756): the picture is on the TV, so the middle says where it went
    Tile {
        id: castTile
        objectName: "player-cast-tile"
        visible: page.casting
        raised: true
        color: theme.glassVideo; borderColor: Qt.rgba(1, 1, 1, 0.1)
        padding: theme.space(5)
        anchors.centerIn: parent
        width: castCol.implicitWidth + padding * 2; height: castCol.implicitHeight + padding * 2
        MouseArea { anchors.fill: parent; anchors.margins: -castTile.padding }
        Column {
            id: castCol
            anchors.centerIn: parent
            spacing: theme.space(2)
            Icon { anchors.horizontalCenter: parent.horizontalCenter; glyph: "cast"; size: theme.space(10) }
            Label { anchors.horizontalCenter: parent.horizontalCenter; objectName: "player-cast-device"; role: "title"; font.pointSize: theme.typeNormal
                text: page.casting ? "Playing on " + page.castDevice.name : "" }
            Label { anchors.horizontalCenter: parent.horizontalCenter; objectName: "player-cast-line"; role: "ui"; color: theme.textDim; text: page.castLine }
        }
    }

    // ---- Replay: the last frame is still on screen because keep-open holds it, so this sits
    // in the middle of it. A replay opens a fresh session, since the core closed this one.
    Button {
        objectName: "player-replay"
        visible: page.replayVisible
        anchors.centerIn: parent
        primary: true; icon: "rotate-ccw"; text: "Replay"; takesFocus: false
        onClicked: frame.nav.replace("player", { file: page.props.file }, page.title)
    }
}
