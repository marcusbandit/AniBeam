// The tone of a poster: its most saturated common colour, read off a 24 px thumbnail
// drawn on a hidden Canvas. The result is lifted to a hue that holds white text when
// used as a wash (the hero band) and as a fill (the Resume button). `ready` turns true once
// a colour is read; until then `tone` is the theme's accent, so nothing waits on it.
import QtQuick

Item {
    id: root
    property string source: ""
    readonly property bool ready: sampled !== ""
    property string sampled: ""
    readonly property color tone: ready ? Qt.color(sampled) : theme.accent
    visible: false
    width: 24; height: 36

    onSourceChanged: { sampled = ""; if (source !== "") canvas.loadImage(source) }

    Canvas {
        id: canvas
        anchors.fill: parent
        renderTarget: Canvas.Image
        renderStrategy: Canvas.Immediate
        onImageLoaded: requestPaint()
        onPaint: {
            if (root.source === "" || !isImageLoaded(root.source)) return
            var ctx = getContext("2d")
            ctx.clearRect(0, 0, width, height)
            ctx.drawImage(root.source, 0, 0, width, height)
            var data = ctx.getImageData(0, 0, width, height).data
            // Bucket the hue (24 bins) weighting by saturation, so a vivid minority beats a
            // grey majority; then the bin's mean colour, pulled toward a readable band.
            var bins = []
            for (var b = 0; b < 24; b++) bins.push({ w: 0, r: 0, g: 0, bl: 0 })
            for (var i = 0; i < data.length; i += 4) {
                var r = data[i] / 255, g = data[i + 1] / 255, bl = data[i + 2] / 255
                var mx = Math.max(r, g, bl), mn = Math.min(r, g, bl), d = mx - mn
                if (d < 0.12 || mx < 0.18) continue
                var h = d === 0 ? 0 : mx === r ? ((g - bl) / d + 6) % 6 : mx === g ? (bl - r) / d + 2 : (r - g) / d + 4
                var k = Math.floor(h / 6 * 24) % 24
                var w = d * d
                bins[k].w += w; bins[k].r += r * w; bins[k].g += g * w; bins[k].bl += bl * w
            }
            var best = -1
            for (var j = 0; j < 24; j++) if (best < 0 || bins[j].w > bins[best].w) best = j
            if (best < 0 || bins[best].w === 0) { root.sampled = String(theme.accent); return }
            var c = Qt.rgba(bins[best].r / bins[best].w, bins[best].g / bins[best].w, bins[best].bl / bins[best].w, 1)
            // Lighten the dark ones and deepen the pale ones, both toward a mid value
            var l = 0.2126 * c.r + 0.7152 * c.g + 0.0722 * c.b
            if (l < 0.3) c = theme.tone(c, Qt.rgba(1, 1, 1, 1), (0.3 - l) * 1.4)
            if (l > 0.7) c = theme.tone(c, Qt.rgba(0, 0, 0, 1), (l - 0.7) * 1.2)
            root.sampled = String(c)
        }
    }
}
