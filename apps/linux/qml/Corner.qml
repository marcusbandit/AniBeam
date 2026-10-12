// The one shape primitive. The glass mosaic is all straight lines, so every Corner is square:
// `radius` and `smoothing` are still accepted (every caller sets them) and ignored, and so
// are flatTop and flatBottom. The squircle code below is kept, unreached, so a later
// direction can bring corners back by restoring `reach`. `highlight` draws the glass's
// 1 px top line inside the edge.
//
// Was: a G2 squircle (Figma corner smoothing, reach semantics), `radius` the corner's REACH.
//
// Cost: a plain corner (smoothing 0) with a flat colour is a Rectangle, which is what it
// looks like anyway, and the Shape is never built. Only smoothing, a fill item, a gradient
// or a dash build the Shape (CurveRenderer, the JS squircle, the PathSvg parse).
import QtQuick
import QtQuick.Shapes

Item {
    id: root

    property real radius: 10
    property real smoothing: 0.6
    property color color: "transparent"
    property color borderColor: "transparent"
    property real borderWidth: 0
    // Any Item painted inside the shape instead of `color`, as it is laid out at this
    // shape's size; hide it on screen (visible: false) while it is the fill.
    //
    // An Image (anything with a fillMode and a decoded texture) is sampled straight from its
    // own texture, with no offscreen pass: ShapePath maps a fill texture at its native pixel
    // size from the shape's top left, ignoring the Image's width, height and fillMode (the
    // old quarter-poster bug was exactly that, a 480 px decode drawn 1:1 into a 180 px
    // frame). The fillTransform below scales and centres the texture the way the Image's own
    // fillMode would (PreserveAspectCrop, or Stretch), so decode the Image at its displayed
    // size (sourceSize width AND height, times the device pixel ratio) and it is a pointer
    // to an already-cached texture. Any other item (an effect, a video) is rendered into a
    // ShaderEffectSource first, which exists only while such an item is the fill.
    property Item fillItem: null
    // A ShapeGradient (LinearGradient and friends) painted instead of `color`.
    property ShapeGradient fillGradient: null
    property int dashed: 0
    // Square off the top or bottom pair of corners: two Corners stacked edge to edge, one
    // flatBottom over one flatTop, read as one rounded stone (a poster over its plate).
    property bool flatTop: false
    property bool flatBottom: false
    // The glass's top highlight: a 1 px line along the top edge, inside any border
    property color highlight: "transparent"
    // The corner reach actually drawn: always 0 (see the header)
    readonly property real reach: 0

    readonly property real inset: borderWidth / 2
    readonly property string pathData: plain ? "" : squircle(width, height, reach, 0, inset, false, false)
    readonly property bool plain: fillItem === null && fillGradient === null && dashed === 0
    readonly property bool fillIsImage: fillItem !== null && fillItem.fillMode !== undefined && fillItem.implicitWidth > 0

    // One child, picked by what the corner has to paint: a Rectangle, a flat Shape, or a
    // Shape with a fill item and the transform that lays the fill out
    Rectangle {
        visible: root.plain
        anchors.fill: parent
        color: root.color
        border.color: root.borderWidth > 0 ? root.borderColor : "transparent"
        border.width: root.borderWidth
        antialiasing: false
    }
    Loader {
        anchors.fill: parent
        active: !root.plain
        sourceComponent: root.fillItem !== null ? filledShape : curvedShape
    }

    // Above the fill (and a fill item), below the caller's children
    Rectangle {
        visible: root.highlight.a > 0
        x: root.borderWidth; y: root.borderWidth
        width: Math.max(0, root.width - root.borderWidth * 2)
        height: 1
        color: root.highlight
        z: 0
    }

    Component {
        id: curvedShape
        Shape {
            preferredRendererType: Shape.CurveRenderer
            ShapePath {
                strokeColor: root.borderWidth > 0 ? root.borderColor : "transparent"
                strokeWidth: root.borderWidth
                strokeStyle: root.dashed ? ShapePath.DashLine : ShapePath.SolidLine
                dashPattern: [3, 3]
                fillColor: root.color
                fillGradient: root.fillGradient
                PathSvg { path: root.pathData }
            }
        }
    }

    Component {
        id: filledShape
        Shape {
            id: shape
            preferredRendererType: Shape.CurveRenderer
            // Anything but an Image is rendered into a texture first, and only then
            readonly property bool image: root.fillIsImage
            ShaderEffectSource {
                id: layer
                visible: false
                sourceItem: shape.image ? null : root.fillItem
                hideSource: !shape.image
                live: !shape.image
                smooth: true
                readonly property real dpr: Screen.devicePixelRatio > 0 ? Screen.devicePixelRatio : 1
                textureSize: shape.image || !root.fillItem ? Qt.size(1, 1) : Qt.size(Math.ceil(root.fillItem.width * dpr), Math.ceil(root.fillItem.height * dpr))
            }
            // The fill texture's size in its own units, and the transform that lays it over
            // this shape's box the way the fill item itself would draw (the fill item is laid
            // out at this shape's size and origin)
            readonly property real tw: image ? root.fillItem.implicitWidth : layer.textureSize.width
            readonly property real th: image ? root.fillItem.implicitHeight : layer.textureSize.height
            readonly property bool stretch: !image || root.fillItem.fillMode === Image.Stretch
            readonly property real sx: tw > 0 ? (stretch ? root.width / tw : Math.max(root.width / tw, root.height / th)) : 1
            readonly property real sy: th > 0 ? (stretch ? root.height / th : sx) : 1
            ShapePath {
                strokeColor: root.borderWidth > 0 ? root.borderColor : "transparent"
                strokeWidth: root.borderWidth
                strokeStyle: root.dashed ? ShapePath.DashLine : ShapePath.SolidLine
                dashPattern: [3, 3]
                fillColor: "white"
                fillItem: shape.image ? root.fillItem : layer
                fillTransform: Qt.matrix4x4(shape.sx, 0, 0, (root.width - shape.tw * shape.sx) / 2,
                                            0, shape.sy, 0, (root.height - shape.th * shape.sy) / 2,
                                            0, 0, 1, 0, 0, 0, 0, 1)
                PathSvg { path: root.pathData }
            }
        }
    }

    function squircle(w, h, reach, s, ins, flatTop, flatBottom) {
        var x0 = ins, y0 = ins, x1 = w - ins, y1 = h - ins
        var W = x1 - x0, H = y1 - y0
        if (W <= 0 || H <= 0) return ""
        var p = Math.max(0, Math.min(reach, W / 2, H / 2))
        s = Math.max(0, Math.min(0.99, s))
        function f(v) { return Math.round(v * 1000) / 1000 }
        if (p < 0.05)
            return "M" + f(x0) + " " + f(y0) + " H" + f(x1) + " V" + f(y1) + " H" + f(x0) + " Z"
        var r = p / (1 + s)
        var arcMeasure = 90 * (1 - s)
        var toRad = Math.PI / 180
        var arcSection = Math.sin(arcMeasure / 2 * toRad) * r * Math.SQRT2
        var alpha = (90 - arcMeasure) / 2
        var p3p4 = r * Math.tan(alpha / 2 * toRad)
        var beta = 45 * s
        var c = p3p4 * Math.cos(beta * toRad)
        var d = c * Math.tan(beta * toRad)
        var b = (p - arcSection - c - d) / 3
        var a = 2 * b
        var curved = (a + b + c + d) > 0.001
        var arced = arcSection > 0.001
        function cc(ax, ay, bx, by, cx, cy) {
            return curved ? " c" + f(ax) + " " + f(ay) + " " + f(bx) + " " + f(by) + " " + f(cx) + " " + f(cy) : ""
        }
        function arc(dx, dy) {
            return arced ? " a" + f(r) + " " + f(r) + " 0 0 1 " + f(dx) + " " + f(dy) : ""
        }
        // top right, then clockwise; a flat pair is a plain corner point instead
        var path = flatTop ? "M" + f(x0) + " " + f(y0) + " L" + f(x1) + " " + f(y0)
                           : "M" + f(x1 - p) + " " + f(y0)
                             + cc(a, 0, a + b, 0, a + b + c, d) + arc(arcSection, arcSection) + cc(d, c, d, b + c, d, a + b + c)
        if (flatBottom) {
            path += " L" + f(x1) + " " + f(y1) + " L" + f(x0) + " " + f(y1)
        } else {
            path += " L" + f(x1) + " " + f(y1 - p)
            path += cc(0, a, 0, a + b, -d, a + b + c) + arc(-arcSection, arcSection) + cc(-c, d, -(b + c), d, -(a + b + c), d)
            path += " L" + f(x0 + p) + " " + f(y1)
            path += cc(-a, 0, -(a + b), 0, -(a + b + c), -d) + arc(-arcSection, -arcSection) + cc(-d, -c, -d, -(b + c), -d, -(a + b + c))
        }
        if (flatTop) return path + " Z"
        path += " L" + f(x0) + " " + f(y0 + p)
        path += cc(0, -a, 0, -(a + b), d, -(a + b + c)) + arc(arcSection, -arcSection) + cc(c, -d, b + c, -d, a + b + c, -d)
        return path + " Z"
    }
}
