import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import dev.anibeam.spike

ApplicationWindow {
    id: window
    width: 640
    height: 480
    visible: true
    title: "AniBeam cxx-qt spike"

    property string ticks: ""

    Connections {
        target: Spike
        function onTick(n, workerThread) {
            window.ticks += "tick " + n + " from " + workerThread + "\n"
            console.log("SPIKE tick", n, "counter", Spike.counter, "worker", workerThread)
        }
        function onStatusChanged() { console.log("SPIKE status", Spike.status) }
    }

    // A run needs no clicking: the job starts once the window is up.
    Component.onCompleted: Spike.startJob(5)

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: 16
        spacing: 12

        RowLayout {
            spacing: 12
            Image {
                source: "qrc:/qt/qml/dev/anibeam/spike/assets/icon.png"
                sourceSize: Qt.size(48, 48)
            }
            Label {
                text: "counter " + Spike.counter + "   " + Spike.status
                Layout.fillWidth: true
            }
        }

        Button {
            text: "start job"
            onClicked: Spike.startJob(5)
        }

        Label {
            id: videoLabel
            text: "SpikeVideo (C++ MpvAbstractItem) constructed, mpv " + video.mpvVersion
            onTextChanged: console.log("SPIKE", videoLabel.text)
        }

        SpikeVideo {
            id: video
            Layout.fillWidth: true
            Layout.preferredHeight: 96
        }

        Label {
            text: window.ticks
            Layout.fillWidth: true
            Layout.fillHeight: true
            verticalAlignment: Text.AlignTop
        }
    }
}
