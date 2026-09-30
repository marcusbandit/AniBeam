// A segmented switch: options in a sunken pill, the active one carried by a raised thumb
// that follows with exponential smoothing. Picking emits `picked`; the owner sets `index`,
// so a binding on it survives. An option is a string, or a record { text, icon, delegate }:
// `icon` names a Lucide glyph drawn before the text, `delegate` is a Component drawn before
// it instead, which reads the option's colour and record off its parent Loader as `tint`
// and `option`.
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
                id: cell
                required property int index
                required property var modelData
                readonly property bool record: typeof modelData === "object" && modelData !== null
                readonly property string label: record ? String(modelData.text || "") : String(modelData)
                readonly property string icon: record && modelData.icon ? String(modelData.icon) : ""
                readonly property var custom: record && modelData.delegate ? modelData.delegate : null
                readonly property bool on: index === root.index
                readonly property color tint: on ? theme.text : theme.textDim
                width: content.implicitWidth + theme.space(3) * 2
                height: root.height - root.pad * 2
                Row {
                    id: content
                    anchors.centerIn: parent
                    spacing: theme.space(1.5)
                    Loader {
                        id: glyph
                        readonly property color tint: cell.tint
                        readonly property var option: cell.modelData
                        anchors.verticalCenter: parent.verticalCenter
                        active: cell.custom !== null
                        visible: active
                        sourceComponent: cell.custom
                    }
                    Icon {
                        visible: cell.icon !== ""
                        anchors.verticalCenter: parent.verticalCenter
                        glyph: cell.icon
                        color: cell.tint
                        size: theme.space(root.small ? 3.5 : 4.5)
                    }
                    Text {
                        id: t
                        visible: cell.label !== ""
                        anchors.verticalCenter: parent.verticalCenter
                        text: cell.label
                        color: cell.tint
                        font.family: theme.fontSans
                        font.pointSize: root.small ? theme.typeSmall : theme.typeNormal
                        font.weight: cell.on ? Font.DemiBold : Font.Normal
                    }
                }
                MouseArea {
                    anchors.fill: parent
                    cursorShape: Qt.PointingHandCursor
                    onClicked: root.picked(cell.index)
                }
            }
        }
    }
}
