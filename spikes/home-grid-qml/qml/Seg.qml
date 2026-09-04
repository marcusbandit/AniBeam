// A segmented switch: options in a sunken pill, the active one carried by a raised thumb
// that follows with exponential smoothing. Picking emits `picked`; the owner sets `index`,
// so a binding on it survives.
import QtQuick

Corner {
    id: root
    property var options: []
    property int index: 0
    property bool small: false
    signal picked(int i)

    readonly property real pad: theme.space(0.75)
    implicitHeight: (small ? theme.space(6) : theme.controlHeight)
    implicitWidth: row.implicitWidth + pad * 2
    radius: height / 2
    smoothing: theme.cornerSmoothing
    color: theme.surfaceSunken
    borderColor: theme.line
    borderWidth: 1

    property real thumbX: 0
    property real thumbW: 0
    property real targetX: labels.count > index && labels.itemAt(index) ? labels.itemAt(index).x + pad : pad
    property real targetW: labels.count > index && labels.itemAt(index) ? labels.itemAt(index).width : 0
    Component.onCompleted: { thumbX = targetX; thumbW = targetW }
    onTargetWChanged: if (thumbW === 0) { thumbW = targetW; thumbX = targetX }

    FrameAnimation {
        running: Math.abs(root.thumbX - root.targetX) > 0.2 || Math.abs(root.thumbW - root.targetW) > 0.2
        onTriggered: {
            var k = 1 - Math.exp(-14 * frameTime)
            root.thumbX += (root.targetX - root.thumbX) * k
            root.thumbW += (root.targetW - root.thumbW) * k
        }
    }

    Corner {
        x: root.thumbX
        y: root.pad
        width: root.thumbW
        height: root.height - root.pad * 2
        radius: height / 2
        smoothing: theme.cornerSmoothing
        color: theme.surfaceRaised
        borderColor: theme.lineStrong
        borderWidth: 1
    }

    Row {
        id: row
        x: root.pad
        anchors.verticalCenter: parent.verticalCenter
        Repeater {
            id: labels
            model: root.options
            Item {
                width: t.implicitWidth + theme.space(3) * 2
                height: root.height - root.pad * 2
                Text {
                    id: t
                    anchors.centerIn: parent
                    text: modelData
                    color: index === root.index ? theme.text : theme.textDim
                    font.family: theme.fontSans
                    font.pointSize: root.small ? theme.typeSmall : theme.typeNormal
                    font.weight: index === root.index ? Font.DemiBold : Font.Normal
                }
                MouseArea {
                    anchors.fill: parent
                    cursorShape: Qt.PointingHandCursor
                    onClicked: root.picked(index)
                }
            }
        }
    }
}
