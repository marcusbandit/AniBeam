// Developer options (#287): the page load overlay. The last load in four lines, in the
// window's bottom right corner over whatever page is up, and the process's memory now. A
// click opens the Developer tab with that load picked.
import QtQuick
import com.marcusrosado.AniBeam

Rectangle {
    id: root
    property var recorder
    readonly property var load: recorder ? recorder.last : null
    property var mem: ({})
    width: column.implicitWidth + theme.space(5)
    height: column.implicitHeight + theme.space(3)
    radius: theme.space(1.5)
    color: Qt.rgba(theme.bg.r, theme.bg.g, theme.bg.b, 0.88)
    border.color: theme.lineStrong
    border.width: 1

    function memRead() { mem = JSON.parse(Dev.memory()) }
    Timer { interval: 1000; repeat: true; running: root.visible; triggeredOnStart: true; onTriggered: root.memRead() }

    function tone(ms) { return ms < 100 ? theme.green : ms < 300 ? theme.yellow : theme.red }

    Column {
        id: column
        x: theme.space(2.5); y: theme.space(1.5)
        spacing: theme.space(0.5)
        Row {
            spacing: theme.space(2)
            Label { role: "ledger"; text: root.load ? root.load.label : "No page timed yet"; color: theme.text; font.weight: Font.DemiBold }
            Label { role: "ledger"; visible: !!root.load && !root.load.images; text: "images off"; color: theme.orange }
            Label { role: "ledger"; visible: !!root.recorder && !!root.recorder.tourState; text: "tour " + (root.recorder && root.recorder.tourState ? root.recorder.tourState.at + " of " + root.recorder.tourState.steps.length : ""); color: theme.accent }
        }
        Row {
            visible: !!root.load
            spacing: theme.space(3)
            Label { role: "ledger"; text: root.load ? "built " + Math.round(root.load.built) + " ms" : ""; color: root.load ? root.tone(root.load.built) : theme.text }
            Label { role: "ledger"; text: root.load ? "painted " + Math.round(root.load.firstFrame) + " ms" : ""; color: theme.textDim }
            Label { role: "ledger"; text: root.load ? "settled " + Math.round(root.load.settled) + " ms" : ""; color: root.load ? root.tone(root.load.settled) : theme.text }
        }
        Label {
            visible: !!root.load
            role: "ledger"; color: theme.textDim
            text: root.load ? root.load.calls.n + " calls, " + Math.round(root.load.calls.ms) + " ms, " + root.recorder.kb(root.load.calls.bytes)
                              + "   " + root.load.decodes.n + " decodes, " + Math.round(root.load.decodes.ms) + " ms, " + root.recorder.mb(root.load.decodes.bytes) : ""
        }
        Label {
            visible: !!root.load
            role: "ledger"; color: theme.textDim
            text: root.load ? root.load.scene.items.toLocaleString(Qt.locale(), "f", 0) + " items, " + root.load.scene.images.total + " images, "
                              + root.load.scene.images.offscreen + " outside the window" : ""
        }
        Label {
            role: "ledger"; color: theme.textDim
            text: root.mem.VmRSS ? "memory " + root.recorder.mb(root.mem.VmRSS * 1024) + ", peak " + root.recorder.mb(root.mem.VmHWM * 1024) : "memory not readable here"
        }
    }
    MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: frame.go("settings", { tab: "developer" }) }
}
