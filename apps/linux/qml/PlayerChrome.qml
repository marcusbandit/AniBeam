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
            position: page.timePos; duration: page.duration; windows: page.windows
            onSeeked: function(s) { page.seekTo(s) }
            onHovered: function(s) { page.showChrome(); preview.show(s, seek.mapToItem(page, page.duration > 0 ? s / page.duration * seek.width : 0, 0).x) }
            onUnhovered: preview.hide() }
        Row {
            id: bottomRow
            anchors.left: parent.left; anchors.right: parent.right; anchors.bottom: parent.bottom
            // No height: a Row is as tall as its tallest child, so the island follows the
            // buttons rather than a number that has to be kept in step.
            spacing: theme.space(1)
            readonly property bool outroUp: page.inside(page.outro, page.timePos)
            // A narrow window: how far the row would run past the island with every control
            // the file asks for up, measured from the items' own widths so it follows the
            // tokens and the density. While it runs over, the controls that have a key of
            // their own give way one at a time, in this order, until it fits: the volume
            // slider (Up and Down), Keyboard shortcuts (?), the delays (z and a), then
            // Previous and Next (p and n). Nothing here reads a visibility it sets, so it
            // settles in one pass.
            readonly property real over: {
                var s = spacing
                var w = prevBtn.width + playBtn.width + nextBtn.width + gap.width + muteBtn.width + volumeSlider.fullWidth + 5 * s
                if (outroUp) w += skipOutro.width + s
                w += s + delayBtn.width + helpBtn.width + fullBtn.width + 2 * s
                if (page.audioTracks.length > 1) w += audioBtn.width + s
                if (page.subTracks.length > 0) w += subBtn.width + s
                if (page.canMark) w += markBtn.width + s
                return w - width
            }
            readonly property real volumeRoom: volumeSlider.fullWidth + spacing
            readonly property real helpRoom: helpBtn.width + spacing
            readonly property real delayRoom: delayBtn.width + spacing
            readonly property real neighbourRoom: prevBtn.width + nextBtn.width + 2 * spacing
            readonly property bool keepVolume: over <= 0
            readonly property bool keepHelp: over <= volumeRoom
            readonly property bool keepDelays: over <= volumeRoom + helpRoom
            readonly property bool keepNeighbours: over <= volumeRoom + helpRoom + delayRoom
            // Narrower still, the right group keeps its slot (below) and the row clips at the
            // island's edge rather than drawing the group over the left half.
            clip: over > volumeRoom + helpRoom + delayRoom + neighbourRoom
            PlayerButton { id: prevBtn; objectName: "player-prev"; glyph: "skip-back"; visible: bottomRow.keepNeighbours; interactive: page.hasPrevious; tip: interactive ? "Previous episode" : "No previous episode"; onClicked: page.openPrevious() }
            PlayerButton { id: playBtn; objectName: "player-play"; glyph: page.paused ? "play" : "pause"; tip: page.paused ? "Play" : "Pause"; onClicked: page.togglePause() }
            PlayerButton { id: nextBtn; objectName: "player-next"; glyph: "skip-forward"; visible: bottomRow.keepNeighbours; interactive: page.hasNext; tip: interactive ? "Next episode" : "No next episode"; onClicked: page.openNext() }
            Item { id: gap; width: theme.space(2); height: 1 }
            PlayerButton { id: muteBtn; objectName: "player-mute"; glyph: Player.mute || page.volume === 0 ? "volume-x" : "volume-2"; tip: Player.mute ? "Unmute" : "Mute"; onClicked: page.setMute(!Player.mute) }
            // Never on the keyboard: a click on it must leave the arrows to seek and the
            // Up and Down keys to step the volume by 5.
            SliderRow { id: volumeSlider; objectName: "player-volume"; visible: bottomRow.keepVolume; anchors.verticalCenter: parent.verticalCenter; from: 0; to: 100; value: Player.mute ? 0 : page.volume; stepSize: 1; trackWidth: theme.space(24)
                takesFocus: false; ground: theme.raised; onMoved: function(v) { page.setVolume(v) } }
            // Skip outro, up only while the position is inside the outro window, and there
            // whether or not auto-skip is on. Skip intro is a tile of its own, below: the
            // corner it takes belongs to the auto-next tile during the outro.
            Button { id: skipOutro; objectName: "player-skip-outro"; visible: bottomRow.outroUp; anchors.verticalCenter: parent.verticalCenter; text: "Skip outro"; icon: "chevrons-right"; primary: true; small: true; takesFocus: false; onClicked: page.skipWindow(page.outro) }
            // The slot takes its height from what it holds, not from the row: the row
            // takes its own height from its children, and parent.height here would be a loop.
            // Never narrower than the group, so the group can never be drawn over the left half.
            Item { id: rightSlot; width: Math.max(rightGroup.width, parent.width - x); height: rightGroup.height
                anchors.verticalCenter: parent.verticalCenter
                Row { id: rightGroup; anchors.right: parent.right; spacing: bottomRow.spacing
                    // Shown with no tracks to choose as well: one audio track can still be out of sync.
                    PlayerButton { id: delayBtn; objectName: "player-delays"; glyph: "clock"; tip: "Subtitle and audio delay"; visible: bottomRow.keepDelays; active: page.subDelay !== 0 || page.audioDelay !== 0; onClicked: page.openDelays(delayBtn) }
                    PlayerButton { id: audioBtn; objectName: "player-audio"; glyph: "audio-lines"; tip: "Audio track"; visible: page.audioTracks.length > 1; onClicked: page.openAudioPicker(audioBtn) }
                    PlayerButton { id: subBtn; objectName: "player-subs"; glyph: "captions"; tip: page.sid >= 0 ? "Subtitles" : "Subtitles off"; active: page.sid >= 0; visible: page.subTracks.length > 0; onClicked: page.openSubPicker(subBtn) }
                    PlayerButton { id: markBtn; objectName: "player-mark"; glyph: "check-check"; tip: "Mark watched"; visible: page.canMark; onClicked: page.markWatched() }
                    PlayerButton { id: helpBtn; objectName: "player-help"; glyph: "keyboard"; tip: "Keyboard shortcuts"; visible: bottomRow.keepHelp; onClicked: page.showHelp() }
                    PlayerButton { id: fullBtn; objectName: "player-fullscreen"; glyph: frame.hostWindow.visibility === Window.FullScreen ? "minimize" : "maximize"; tip: "Fullscreen"; onClicked: page.toggleFullscreen() } } }
        }
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
                Label { id: nextLabel; anchors.centerIn: parent; text: "Next"; role: "title"; font.pointSize: theme.typeNormal }
                MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: page.openNext() }
            }
        }
    }

    // ---- Skip intro. Not chrome either, and in the auto-next tile's corner: it is up for as
    // long as the position is inside the intro window, whether or not the controls are, and
    // whether or not auto-skip is on. A click lets the controls go rather than calling them
    // up, so the picture is clear the moment the intro is. The two never share the corner in
    // practice; if they ever do, this one sits above.
    Tile {
        id: skipTile
        objectName: "player-skip-intro-tile"
        visible: page.inside(page.intro, page.timePos)
        raised: true
        color: theme.glassVideo; borderColor: Qt.rgba(1, 1, 1, 0.1)
        padding: theme.space(1.5)
        anchors.right: parent.right; anchors.rightMargin: theme.grout
        anchors.bottom: nextTile.visible ? nextTile.top : controls.top; anchors.bottomMargin: theme.grout
        width: skipButton.implicitWidth + padding * 2; height: skipButton.implicitHeight + padding * 2
        MouseArea { enabled: page.chromeVisible; anchors.fill: parent; anchors.margins: -skipTile.padding }
        Button { id: skipButton; objectName: "player-skip-intro"; anchors.centerIn: parent; text: "Skip intro"; icon: "chevrons-right"; primary: true; takesFocus: false
            onClicked: page.skipIntroAndHide(page.mapFromItem(null, skipHover.point.scenePosition))
            // Where the pointer is for the click, so the page knows the spot it then rests on.
            HoverHandler { id: skipHover } }
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
