// A slider with a ledger readout beside it, the track and the round handle drawn through
// Corner. The value is the owner's: `moved` reports drags, `value` is bound in.
//
// `takesFocus: false` keeps the slider off the keyboard entirely: clicks and drags still
// work, but the arrows go on to whoever owns them (the player's seek and volume keys).
// `ground` is the colour the slider sits on, which the handle's ring is cut from.
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
    property bool takesFocus: true
    property color ground: theme.bg
    signal moved(real v)
    spacing: theme.space(3)

    QC.Slider {
        id: s
        width: root.trackWidth
        height: theme.controlHeight
        anchors.verticalCenter: parent.verticalCenter
        focusPolicy: root.takesFocus ? Qt.StrongFocus : Qt.NoFocus
        onMoved: root.moved(value)
        background: Item {
            x: s.leftPadding
            y: s.topPadding + s.availableHeight / 2 - height / 2
            width: s.availableWidth
            height: theme.space(1)
            Corner { anchors.fill: parent; radius: height / 2; smoothing: theme.cornerSmoothing; color: theme.tileSunken }
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
            borderColor: s.activeFocus ? theme.focusRing : root.ground
            borderWidth: theme.space(0.5)
        }
    }
    Label {
        anchors.verticalCenter: parent.verticalCenter
        width: theme.space(9)
        role: "ledger"
        text: Number(s.value).toFixed(root.decimals)
        color: theme.textDim
    }
}
