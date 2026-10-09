// Frosted glass: the blurred live snapshot of the layer behind, tinted, with a hairline,
// clipped to the square. The host frame owns the one ShaderEffectSource over the content
// and hands it out as `sourceItem`; every floating panel samples the same texture, so the
// content layer renders once per frame no matter how much glass is up.
import QtQuick
import QtQuick.Effects
import com.marcusrosado.AniBeam

Corner {
    id: root
    // The frame's content-layer source; null falls back to the solid surface colour.
    property Item sourceItem: null
    // How much of the ground tint sits over the blur, 0 to 1.
    property real frostTint: 0.55
    property real frostSaturation: -0.25

    Item {
        id: frostFill
        width: root.width; height: root.height
        visible: false
        MultiEffect {
            anchors.fill: parent
            visible: root.sourceItem !== null
            source: root.sourceItem
            blurEnabled: true
            blurMax: 32
            blurMultiplier: 1.0
            saturation: root.frostSaturation
        }
        Rectangle {
            anchors.fill: parent
            visible: root.sourceItem !== null
            color: Qt.rgba(theme.bg.r, theme.bg.g, theme.bg.b, root.frostTint)
        }
    }
    fillItem: root.sourceItem ? frostFill : null
    color: root.sourceItem ? "transparent" : theme.surface
    smoothing: 0
    borderColor: theme.lineStrong
    borderWidth: 1
}
