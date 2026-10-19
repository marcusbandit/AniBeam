// The one shape primitive: a rounded pane. `radius` is the corner radius and is drawn as a
// plain circular corner: `smoothing` is accepted and ignored (the G2 squircle code below is
// kept, unreached, in case a later direction wants it back by passing it through). flatTop
// and flatBottom square off one pair of corners. `highlight` draws the glass's 1 px top
// line inside the edge, inset past the curve so it never leaves the pane.
//
// Cost: a plain corner (smoothing 0) with a flat colour is a Rectangle, which is what it
// looks like anyway, and the Shape is never built. Only smoothing, a fill item, a gradient
// or a dash build the Shape (CurveRenderer, the JS squircle, the PathSvg parse).
import QtQuick
import QtQuick.Shapes

Item {
    id: root

    property real radius: theme.radiusTile
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
    // The corner radius actually drawn: the caller's, never past half the pane
    readonly property real reach: Math.max(0, Math.min(radius, width / 2, height / 2))
    // How far in from the side the top edge is still straight at 1 px down a corner of
    // `reach`: the curve's x offset at y = 1, so the highlight line stops where the curve
    // starts
    readonly property real highlightInset: reach >= 1 ? reach - Math.sqrt(2 * reach - 1) : 0

    readonly property real inset: borderWidth / 2
    readonly property string pathData: plain ? "" : squircle(width, height, reach, 0, inset, flatTop, flatBottom)
    readonly property bool plain: fillItem === null && fillGradient === null && dashed === 0
    readonly property bool fillIsImage: fillItem !== null && fillItem.fillMode !== undefined && fillItem.implicitWidth > 0
    // The software scene graph (a headless shoot with no display) cannot paint a Shape's
    // fill item: every poster came back white. There an Image fill is drawn as a plain
    // Image at this shape's box instead, square-cornered rather than missing.
    readonly property bool softwareRender: GraphicsInfo.api === GraphicsInfo.Software
    readonly property bool imageFallback: softwareRender && fillItem !== null && fillItem.fillMode !== undefined

    // One child, picked by what the corner has to paint: a Rectangle, a flat Shape, or a
    // Shape with a fill item and the transform that lays the fill out
    Rectangle {
        visible: root.plain
        anchors.fill: parent
        color: root.color
        border.color: root.borderWidth > 0 ? root.borderColor : "transparent"
        border.width: root.borderWidth
        radius: root.reach
        antialiasing: root.reach > 0
    }
    Loader {
        anchors.fill: parent
        active: !root.plain && !root.imageFallback
        sourceComponent: root.fillItem !== null ? filledShape : curvedShape
    }
    Loader {
        anchors.fill: parent
        active: root.imageFallback
        sourceComponent: Image {
            source: root.fillItem.source
            sourceSize: root.fillItem.sourceSize
            fillMode: root.fillItem.fillMode
            smooth: true
            asynchronous: true
            cache: true
        }
    }

    // The sheen: a lit pane catches light from above, a soft wash that fades out by
    // `theme.sheenReach` of its height (capped so a tall pane's sheen stays a band at its
    // top). Drawn only on panes with a highlight, so a quiet strip stays flat.
    Rectangle {
        visible: root.highlight.a > 0 && theme.sheen.a > 0
        x: root.borderWidth; y: root.borderWidth
        width: Math.max(0, root.width - root.borderWidth * 2)
        height: Math.min(Math.max(0, root.height - root.borderWidth * 2) * theme.sheenReach, theme.sheenMax)
        radius: Math.max(0, root.reach - root.borderWidth)
        antialiasing: radius > 0
        gradient: Gradient {
            GradientStop { position: 0; color: theme.sheen }
            GradientStop { position: 1; color: Qt.rgba(theme.sheen.r, theme.sheen.g, theme.sheen.b, 0) }
        }
        z: 0
    }
    // Above the fill (and a fill item), below the caller's children
    Rectangle {
        visible: root.highlight.a > 0
        x: root.borderWidth + root.highlightInset; y: root.borderWidth
        width: Math.max(0, root.width - (root.borderWidth + root.highlightInset) * 2)
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
            // The fill texture's size in item units, and the transform that lays it over this
            // shape's box the way the fill item itself would draw (the fill item is laid out
            // at this shape's size and origin). The texture is decoded in device pixels, but
            // ShapePath lays it down in item units, one texture pixel per 1/dpr of an item
            // unit, so the size the transform works from is the pixel size over the pixel
            // ratio. Without the division a Retina display draws the poster at 1/dpr size and
            // smears its edge pixels over the rest of the card.
            readonly property real dpr: Screen.devicePixelRatio > 0 ? Screen.devicePixelRatio : 1
            readonly property real tw: (image ? root.fillItem.implicitWidth : layer.textureSize.width) / dpr
            readonly property real th: (image ? root.fillItem.implicitHeight : layer.textureSize.height) / dpr
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
