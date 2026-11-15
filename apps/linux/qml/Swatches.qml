// Six hue squares to pick the accent from. Each carries the theme slot it stands for; the
// picked one wears an accent ring. Picking emits `picked(slot)`; the owner sets `slot`.
import QtQuick

Row {
    id: root
    property int slot: 4
    signal picked(int slot)

    // Terminal slots: 1 red, 3 yellow, 2 green, 6 cyan, 4 blue; 7 is the derived orange
    readonly property var swatches: [
        { slot: 1, color: theme.red, msg: "swatches-accent-red" }, { slot: 7, color: theme.orange, msg: "swatches-accent-orange" }, { slot: 3, color: theme.yellow, msg: "swatches-accent-yellow" },
        { slot: 2, color: theme.green, msg: "swatches-accent-green" }, { slot: 6, color: theme.cyan, msg: "swatches-accent-cyan" }, { slot: 4, color: theme.blue, msg: "swatches-accent-blue" }
    ]
    readonly property real size: theme.space(6)
    readonly property real ring: theme.space(0.5)
    spacing: theme.space(2)

    Repeater {
        model: root.swatches
        Item {
            required property var modelData
            readonly property bool on: modelData.slot === root.slot
            width: root.size
            height: root.size
            Corner {
                anchors.fill: parent
                anchors.margins: -(root.ring + theme.space(0.5))
                radius: theme.radiusSm + root.ring + theme.space(0.5)
                smoothing: theme.cornerSmoothing
                borderColor: on ? theme.accent : (m.containsMouse ? theme.lineStrong : "transparent")
                borderWidth: root.ring
            }
            Corner {
                anchors.fill: parent
                radius: theme.radiusSm
                smoothing: theme.cornerSmoothing
                color: modelData.color
            }
            MouseArea {
                Accessible.role: Accessible.RadioButton
                Accessible.onPressAction: root.picked(modelData.slot)
                Accessible.name: words.t(modelData.msg); Accessible.checkable: true; Accessible.checked: on
                id: m
                anchors.fill: parent
                hoverEnabled: true
                cursorShape: Qt.PointingHandCursor
                onClicked: root.picked(modelData.slot)
            }
        }
    }
}
