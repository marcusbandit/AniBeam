// A 16:9 still with the subtitle text style drawn over it in QML, so the fields change the
// picture as you type. The app renders through mpv; this is the sketch of it. Sizes follow
// mpv's 720 line reference: font 55, margin 22, outline and shadow in the same units, all
// scaled by the Scale field. The still and the text are picture, not chrome: they carry
// their own colours and the text is not on the type scale, the way a poster is not.
import QtQuick
import QtQuick.Shapes

Corner {
    id: root
    property string fontFamily: "sans-serif"
    property bool bold: false
    property color fill: "#FFFFFF"
    property color outlineColour: "#000000"
    property real outline: 1.65
    property real shadow: 0
    property real boxOpacity: 0
    property real position: 100
    property real textScale: 1
    property string sample: "Sample subtitle line"

    readonly property color stillTop: "#232a38"
    readonly property color stillBottom: "#07080c"
    readonly property real unit: height / 720
    readonly property real fontPx: Math.max(1, 55 * unit * textScale)
    readonly property real outlinePx: Math.max(0, outline) * unit * textScale
    readonly property real shadowPx: Math.max(0, shadow) * unit * textScale
    readonly property real margin: 22 * unit
    // Rings of shifted copies build the outline; each ring is a full turn of copies
    readonly property int rings: outlinePx > 0 ? Math.ceil(outlinePx / 1.5) : 0
    readonly property int perRing: 8

    height: Math.round(width * 9 / 16)
    radius: theme.radiusMd
    smoothing: theme.cornerSmoothing
    borderColor: theme.line
    borderWidth: 1
    clip: true
    fillGradient: LinearGradient {
        x1: 0; y1: 0; x2: 0; y2: root.height
        GradientStop { position: 0; color: root.stillTop }
        GradientStop { position: 1; color: root.stillBottom }
    }

    // One line of the sample in the style the fields describe; the copies below stack it
    component Glyphs: Text {
        text: root.sample
        font.family: root.fontFamily
        font.bold: root.bold
        font.pixelSize: root.fontPx
    }

    // The still is dark in either mode, so the caption takes the dark mode's dim text
    Text {
        x: theme.space(3)
        y: theme.space(2)
        text: "Preview renders through mpv in the app"
        color: theme.tokensFor("dark").textDim
        font.family: theme.fontSans
        font.pointSize: theme.typeSmall
    }

    Item {
        id: line
        width: fillText.implicitWidth
        height: fillText.implicitHeight
        x: (root.width - width) / 2
        // 0 is the top edge, 100 the bottom margin, past 100 into the margin and no further
        y: Math.min(root.height - height, (root.height - root.margin - height) * root.position / 100)

        Corner {
            anchors.fill: parent
            anchors.margins: -(root.outlinePx + theme.space(1))
            radius: theme.space(1)
            smoothing: theme.cornerSmoothing
            color: root.outlineColour
            opacity: Math.max(0, Math.min(1, root.boxOpacity > 1 ? root.boxOpacity / 100 : root.boxOpacity))
        }
        Glyphs {
            visible: root.shadowPx > 0
            x: root.shadowPx + root.outlinePx
            y: root.shadowPx + root.outlinePx
            color: Qt.rgba(root.outlineColour.r, root.outlineColour.g, root.outlineColour.b, 0.65)
            style: Text.Outline
            styleColor: color
        }
        Repeater {
            model: root.rings * root.perRing
            Glyphs {
                required property int index
                readonly property real r: root.outlinePx * (Math.floor(index / root.perRing) + 1) / root.rings
                readonly property real angle: (index % root.perRing) / root.perRing * 2 * Math.PI
                x: Math.cos(angle) * r
                y: Math.sin(angle) * r
                color: root.outlineColour
                style: Text.Outline
                styleColor: root.outlineColour
            }
        }
        Glyphs {
            id: fillText
            color: root.fill
        }
    }
}
