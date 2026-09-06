// The player's furniture: the header, the seek preview, the controls island, the auto-next
// pill and the replay button. Split out of PlayerPage.qml so the page holds the session, the
// mpv wiring and the rules while this file holds what they look like. Every value drawn here
// is a property on `page`, reached through the context chain, and every control calls one of
// the page's own functions; this file keeps no state of its own beyond the preview's.
import QtQuick
import QtQuick.Window
import com.marcusrosado.AniBeam

Item {
    id: root
    anchors.fill: parent

    // ---- Header
    Rectangle {
        id: header
        anchors.left: parent.left; anchors.right: parent.right; anchors.top: parent.top
        // The bar is as tall as its three lines and one padding step either side, so a
        // larger system font or a denser scale grows it instead of clipping inside it.
        height: headerRow.implicitHeight + theme.space(4) * 2
        color: theme.scrim
        opacity: page.chromeVisible ? 1 : 0
        // Zero opacity still takes clicks, so the hidden chrome has to leave the scene.
        visible: opacity > 0
        Behavior on opacity { NumberAnimation { duration: theme.motionNormal } }
        Row {
            id: headerRow
            anchors.left: parent.left; anchors.leftMargin: theme.space(4); anchors.verticalCenter: parent.verticalCenter
            spacing: theme.space(4)
            PlayerButton { glyph: "arrow-left"; tip: "Back"; onClicked: page.leave() }
            Column {
                id: headerText
                anchors.verticalCenter: parent.verticalCenter
                width: header.width - parent.x - x - theme.space(4)
                Text { width: parent.width; elide: Text.ElideRight; text: page.session ? page.session.series_title : ""; color: theme.textDim; font.family: theme.fontSans; font.pointSize: theme.typeSmall }
                Text { width: parent.width; elide: Text.ElideRight; text: page.session ? (page.session.episode_title || page.session.path.split("/").pop()) : ""; color: theme.text; font.family: theme.fontSans; font.pointSize: theme.typeNormal; font.weight: Font.DemiBold }
                Text { width: parent.width; elide: Text.ElideRight; text: page.session ? page.session.code : ""; color: theme.textFaint; font.family: theme.fontMono; font.pointSize: theme.typeSmall }
            }
        }
    }

    // ---- The seek preview: a second mpv core, nothing audible, moved by time-pos. It is
    // its own core rather than a thumbnail file, so it costs nothing on disk and answers
    // any position; it is declared above the video and below the island so it can never
    // cover the controls it sits over.
    Corner {
        id: preview
        // It hides with the chrome as well as on exit: the island can go while the pointer
        // rests on the bar, and the bar goes with it, so no exit would ever arrive.
        property bool shown: false
        visible: shown && page.chromeVisible
        width: theme.space(60); height: width * 9 / 16 + theme.space(6)
        radius: theme.radiusMd; smoothing: theme.cornerSmoothing
        color: theme.scrim; borderColor: theme.line; borderWidth: 1
        y: controls.y - height - theme.space(2)
        property bool loaded: false
        function show(secs, centerX) {
            x = Math.max(theme.space(2), Math.min(centerX - width / 2, page.width - width - theme.space(2)))
            stamp.text = Fmt.clock(secs)
            shown = true
            if (loaded) previewVideo.setPropertyAsync("time-pos", secs)
        }
        function hide() { shown = false }
        VideoItem {
            id: previewVideo
            anchors.left: parent.left; anchors.right: parent.right; anchors.top: parent.top; anchors.margins: theme.space(1)
            height: width * 9 / 16
            onReady: {
                var o = Player.previewOptions
                for (var i = 0; i < o.length; i++) setProperty(o[i][0], o[i][1])
                if (page.session) command(["loadfile", page.session.path])
            }
            onLoaded: preview.loaded = true
        }
        Text { id: stamp; anchors.bottom: parent.bottom; anchors.bottomMargin: theme.space(1); anchors.horizontalCenter: parent.horizontalCenter; color: theme.text; font.family: theme.fontMono; font.pointSize: theme.typeSmall }
    }

    // ---- Controls island
    Corner {
        id: controls
        anchors.horizontalCenter: parent.horizontalCenter
        anchors.bottom: parent.bottom; anchors.bottomMargin: theme.space(6)
        width: Math.min(parent.width - theme.space(12), theme.space(220))
        height: bottomRow.height + seekSlot.height + theme.space(9)
        radius: theme.radiusLg; smoothing: theme.cornerSmoothing
        color: theme.scrim; borderColor: theme.line; borderWidth: 1
        opacity: page.chromeVisible ? 1 : 0
        visible: opacity > 0
        Behavior on opacity { NumberAnimation { duration: theme.motionNormal } }
        MouseArea { anchors.fill: parent; hoverEnabled: true; onPositionChanged: page.showChrome() }
        // The bar is above the island's own hover MouseArea, so the hover that keeps the
        // chrome up while the pointer sits on the bar comes from the bar itself.
        Item { id: seekSlot; anchors.left: parent.left; anchors.right: parent.right; anchors.top: parent.top; anchors.margins: theme.space(3); height: seek.height
            SeekBar { id: seek; width: parent.width; position: page.timePos; duration: page.duration; windows: page.windows
                onSeeked: function(s) { page.seekTo(s) }
                onHovered: function(s) { page.showChrome(); preview.show(s, seek.mapToItem(page, page.duration > 0 ? s / page.duration * seek.width : 0, 0).x) }
                onUnhovered: preview.hide() } }
        Row {
            id: bottomRow
            anchors.left: parent.left; anchors.right: parent.right; anchors.bottom: parent.bottom; anchors.margins: theme.space(3)
            // No height: a Row is as tall as its tallest child, so the island follows the
            // buttons and the readout rather than a number that has to be kept in step.
            spacing: theme.space(1)
            PlayerButton { glyph: "skip-back"; interactive: !!(page.session && page.session.prev && !page.session.is_extra); tip: interactive ? "Previous episode" : "No previous episode"; onClicked: page.openNeighbour(page.session.prev) }
            PlayerButton { glyph: page.paused ? "play" : "pause"; tip: page.paused ? "Play" : "Pause"; onClicked: page.togglePause() }
            PlayerButton { glyph: "skip-forward"; interactive: !!(page.session && page.session.next && !page.session.is_extra); tip: interactive ? "Next episode" : "No next episode"; onClicked: page.openNeighbour(page.session.next) }
            Text { anchors.verticalCenter: parent.verticalCenter; text: Fmt.clock(page.timePos) + " / " + Fmt.clock(page.duration); color: theme.text; font.family: theme.fontMono; font.pointSize: theme.typeSmall; leftPadding: theme.space(2); rightPadding: theme.space(2) }
            PlayerButton { glyph: Player.mute || Player.volume === 0 ? "volume-x" : "volume-2"; tip: Player.mute ? "Unmute" : "Mute"; onClicked: page.setMute(!Player.mute) }
            SliderRow { anchors.verticalCenter: parent.verticalCenter; from: 0; to: 100; value: Player.mute ? 0 : Player.volume; stepSize: 1; trackWidth: theme.space(24); onMoved: function(v) { page.setVolume(v) } }
            // The two skip buttons, each up only while the position is inside its own
            // window, and there whether or not auto-skip is on. Two, not a Repeater over
            // the windows: SkipKind is a closed pair in the contract, and a model rebuilt
            // from a filter on every observed time-pos would rebuild its delegates with it.
            Button { visible: page.inside(page.intro, page.timePos); anchors.verticalCenter: parent.verticalCenter; text: "Skip Intro"; small: true; onClicked: page.skipWindow(page.intro) }
            Button { visible: page.inside(page.outro, page.timePos); anchors.verticalCenter: parent.verticalCenter; text: "Skip Outro"; small: true; onClicked: page.skipWindow(page.outro) }
            // The slot takes its height from what it holds, not from the row: the row now
            // takes its own height from its children, and parent.height here would be a loop.
            Item { id: rightSlot; width: parent.width - x - theme.space(1); height: rightGroup.height
                anchors.verticalCenter: parent.verticalCenter
                Row { id: rightGroup; anchors.right: parent.right; spacing: theme.space(1)
                    PlayerButton { id: audioBtn; glyph: "audio-lines"; tip: "Audio track"; visible: page.audioTracks.length > 1; onClicked: page.openAudioPicker(audioBtn) }
                    PlayerButton { id: subBtn; glyph: "captions"; tip: page.sid >= 0 ? "Subtitles" : "Subtitles off"; active: page.sid >= 0; visible: page.subTracks.length > 0; onClicked: page.openSubPicker(subBtn) }
                    PlayerButton { glyph: "check-check"; tip: "Mark watched"; visible: page.canMark; onClicked: page.markWatched() }
                    PlayerButton { glyph: "circle-question-mark"; tip: "Keyboard shortcuts"; onClicked: page.showHelp() }
                    PlayerButton { glyph: frame.hostWindow.visibility === Window.FullScreen ? "minimize" : "maximize"; tip: "Fullscreen"; onClicked: page.toggleFullscreen() } } }
        }
    }

    // ---- Auto-next. The pill is not chrome: it stays on screen once the outro starts even
    // after the controls have hidden, since it is the only thing that says what happens next.
    Row {
        visible: page.nextVisible
        anchors.right: parent.right; anchors.bottom: controls.top; anchors.margins: theme.space(6)
        spacing: theme.space(2)
        // Not flat: this button stands on the picture rather than on the island's scrim, and
        // a flat one is unreadable over a bright frame. Button says "a fill at rest" by not
        // being flat, which is the same treatment the replay button's restColor gives.
        Button { anchors.verticalCenter: parent.verticalCenter; text: "Stay"; onClicked: page.stay() }
        Corner {
            anchors.verticalCenter: parent.verticalCenter
            width: nextLabel.implicitWidth + theme.space(8); height: theme.controlHeight
            radius: height / 2; smoothing: theme.cornerSmoothing
            color: theme.surfaceRaised; borderColor: theme.accent; borderWidth: 1
            Corner { id: countFill; width: 0; height: parent.height; radius: height / 2; smoothing: theme.cornerSmoothing; color: theme.accentSoft
                // The animation owns the width while it runs and leaves it wherever it
                // stopped, so a countdown cancelled part way would keep a half filled pill.
                NumberAnimation on width { running: page.nextCounting; from: 0; to: countFill.parent.width; duration: page.nextCountMs
                    onRunningChanged: if (!running) countFill.width = 0 } }
            Text { id: nextLabel; anchors.centerIn: parent; text: "Next"; color: theme.text; font.family: theme.fontSans; font.pointSize: theme.typeNormal; font.weight: Font.DemiBold }
            MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: page.openNeighbour(page.session.next) }
        }
    }

    // ---- Replay: the last frame is still on screen because keep-open holds it, so this sits
    // in the middle of it. A replay opens a fresh session, since the core closed this one.
    PlayerButton {
        visible: page.replayVisible
        anchors.centerIn: parent
        glyph: "rotate-ccw"; tip: "Replay"
        width: theme.space(20); glyphSize: theme.space(8)
        restColor: theme.scrim; borderColor: theme.line; borderWidth: 1
        onClicked: frame.nav.replace("player", { file: page.props.file }, page.title)
    }
}
