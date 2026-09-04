// The one rounded-shape primitive: a G2 squircle (Figma corner smoothing, reach semantics).
// `radius` is the corner's REACH along each side, not the arc radius; the arc radius is
// reach / (1 + smoothing), so a G2 corner and a plain corner of the same radius are the
// same size. Reach is clamped to half the shorter side, so a pill is radius: height / 2.
import QtQuick
import QtQuick.Shapes

Item {
    id: root

    property real radius: 10
    property real smoothing: 0.6
    property color color: "transparent"
    property color borderColor: "transparent"
    property real borderWidth: 0
    // A texture-providing item (an Image) painted inside the shape instead of `color`.
    property Item fillItem: null
    property int dashed: 0

    readonly property real inset: borderWidth / 2
    readonly property string pathData: squircle(width, height, radius, smoothing, inset)

    Shape {
        anchors.fill: parent
        preferredRendererType: Shape.CurveRenderer
        ShapePath {
            strokeColor: root.borderWidth > 0 ? root.borderColor : "transparent"
            strokeWidth: root.borderWidth
            strokeStyle: root.dashed ? ShapePath.DashLine : ShapePath.SolidLine
            dashPattern: [3, 3]
            fillColor: root.fillItem ? "white" : root.color
            fillItem: root.fillItem
            PathSvg { path: root.pathData }
        }
    }

    function squircle(w, h, reach, s, ins) {
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
        var path = "M" + f(x1 - p) + " " + f(y0)
        // top right, then clockwise
        path += cc(a, 0, a + b, 0, a + b + c, d) + arc(arcSection, arcSection) + cc(d, c, d, b + c, d, a + b + c)
        path += " L" + f(x1) + " " + f(y1 - p)
        path += cc(0, a, 0, a + b, -d, a + b + c) + arc(-arcSection, arcSection) + cc(-c, d, -(b + c), d, -(a + b + c), d)
        path += " L" + f(x0 + p) + " " + f(y1)
        path += cc(-a, 0, -(a + b), 0, -(a + b + c), -d) + arc(-arcSection, -arcSection) + cc(-d, -c, -d, -(b + c), -d, -(a + b + c))
        path += " L" + f(x0) + " " + f(y0 + p)
        path += cc(0, -a, 0, -(a + b), d, -(a + b + c)) + arc(arcSection, -arcSection) + cc(c, -d, b + c, -d, a + b + c, -d)
        return path + " Z"
    }
}
