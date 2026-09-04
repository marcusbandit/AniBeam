// Six hue squares to pick the accent from. Each carries the theme slot it stands for; the
// picked one wears an accent ring. Picking emits `picked(slot)`; the owner sets `slot`.
import QtQuick

Row {
    id: root
    property int slot: 4
    signal picked(int slot)

    // Terminal slots: 1 red, 3 yellow, 2 green, 6 cyan, 4 blue; 7 is the derived orange
    readonly property var swatches: [
        { slot: 1, color: theme.red }, { slot: 7, color: theme.orange }, { slot: 3, color: theme.yellow },
        { slot: 2, color: theme.green }, { slot: 6, color: theme.cyan }, { slot: 4, color: theme.blue }
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
                id: m
                anchors.fill: parent
                hoverEnabled: true
                cursorShape: Qt.PointingHandCursor
                onClicked: root.picked(modelData.slot)
            }
        }
    }
}
