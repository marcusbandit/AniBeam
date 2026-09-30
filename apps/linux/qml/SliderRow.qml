// A slider with a fixed-face readout beside it, the track and the round handle drawn through
// Corner. The value is the owner's: `moved` reports drags, `value` is bound in.
import QtQuick
import QtQuick.Controls.Basic as QC

Row {
    id: root
    property alias from: s.from
    property alias to: s.to
    property alias value: s.value
    property alias stepSize: s.stepSize
    property int decimals: 0
    property real trackWidth: theme.space(40)
    signal moved(real v)
    spacing: theme.space(3)

    QC.Slider {
        id: s
        width: root.trackWidth
        height: theme.controlHeight
        anchors.verticalCenter: parent.verticalCenter
        onMoved: root.moved(value)
        background: Item {
            x: s.leftPadding
            y: s.topPadding + s.availableHeight / 2 - height / 2
            width: s.availableWidth
            height: theme.space(1)
            Corner { anchors.fill: parent; radius: height / 2; smoothing: theme.cornerSmoothing; color: theme.surfaceSunken; borderColor: theme.line; borderWidth: 1 }
            Corner { width: s.visualPosition * parent.width; height: parent.height; radius: height / 2; smoothing: theme.cornerSmoothing; color: theme.accent }
        }
        handle: Corner {
            x: s.leftPadding + s.visualPosition * (s.availableWidth - width)
            y: s.topPadding + s.availableHeight / 2 - height / 2
            width: theme.space(4)
            height: width
            radius: width / 2
            smoothing: theme.cornerSmoothing
            color: theme.accent
            borderColor: s.activeFocus ? theme.focusRing : theme.bg
            borderWidth: theme.space(0.5)
        }
    }
    Text {
        anchors.verticalCenter: parent.verticalCenter
        width: theme.space(9)
        text: Number(s.value).toFixed(root.decimals)
        color: theme.textDim
        font.family: theme.fontMono
        font.pointSize: theme.typeSmall
    }
}
