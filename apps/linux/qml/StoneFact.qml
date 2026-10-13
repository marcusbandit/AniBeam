// A label stone's one line under the section's name, in the plates' own roles: a plain
// ledger line ({ text }), or the next episode out ({ at, text }): "Next", a clock and the
// countdown in accent text (the Card's treatment, no pill), and the title under it.
import QtQuick

Column {
    id: root
    property var fact: null
    property real nowMs: Date.now()
    property bool shown: true
    readonly property bool countdown: !!fact && fact.at !== undefined
    function shortCountdown(secs) {
        var t = Math.max(0, Math.floor(secs))
        var d = Math.floor(t / 86400), h = Math.floor(t % 86400 / 3600), m = Math.floor(t % 3600 / 60)
        return d > 0 ? d + "d " + h + "h" : h > 0 ? h + "h " + m + "m" : Math.max(1, m) + "m"
    }
    visible: !!fact && shown
    spacing: theme.space(1)
    Row {
        visible: root.countdown
        spacing: theme.space(1.5)
        Label { anchors.verticalCenter: parent.verticalCenter; text: "Next"; role: "ledger"; color: theme.textDim }
        Icon { anchors.verticalCenter: parent.verticalCenter; glyph: "clock"; size: Math.round(pillText.implicitHeight * 0.85); color: theme.accent }
        Label {
            id: pillText
            anchors.verticalCenter: parent.verticalCenter
            text: root.countdown ? root.shortCountdown(root.fact.at - root.nowMs / 1000) : ""
            role: "ledger"
            font.weight: 650
            font.variableAxes: ({ "wdth": 88, "wght": 650 })
            color: theme.accent
        }
    }
    Label {
        width: parent.width
        text: root.fact ? root.fact.text : ""
        role: root.countdown ? "ui" : "ledger"
        color: root.countdown ? theme.text : theme.textDim
        wrapMode: Text.Wrap
        maximumLineCount: 2
        elide: Text.ElideRight
    }
}
