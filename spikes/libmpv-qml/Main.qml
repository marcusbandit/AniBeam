import QtQuick
import QtQuick.Window
import Spike

Window {
    id: root
    width: 1280
    height: 720
    visible: true
    color: "black"
    title: "mpvspike"

    MpvItem {
        id: player
        anchors.fill: parent
        focus: true
        onFullscreenRequested: (on) => root.visibility = on ? Window.FullScreen : Window.Windowed
        onQuitRequested: Qt.quit()
        Keys.onPressed: (e) => {
            if (e.key === Qt.Key_Space) player.togglePause()
            else if (e.key === Qt.Key_F) root.visibility = root.visibility === Window.FullScreen ? Window.Windowed : Window.FullScreen
            else if (e.key === Qt.Key_Comma) player.frameStep(-1)
            else if (e.key === Qt.Key_Period) player.frameStep(+1)
            else if (e.key === Qt.Key_M) player.toggleMute()
            else if (e.key === Qt.Key_R) player.report("manual")
            else if (e.key === Qt.Key_Q || e.key === Qt.Key_Escape) Qt.quit()
        }
    }

    MpvItem {
        id: preview
        visible: spikePreview
        previewMode: true
        width: 384
        height: 216
        anchors.right: parent.right
        anchors.bottom: parent.bottom
        anchors.margins: 24
    }

    Rectangle {
        visible: !spikeQuality
        anchors.left: parent.left
        anchors.top: parent.top
        anchors.margins: 12
        width: status.implicitWidth + 16
        height: status.implicitHeight + 8
        color: "#c0000000"
        Text {
            id: status
            anchors.centerIn: parent
            color: "white"
            font.family: "monospace"
            text: player.statusLine + (spikePreview ? "\n" + preview.statusLine : "")
        }
    }
}
