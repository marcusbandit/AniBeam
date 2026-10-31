// A pill split into segments by share, with a legend line under it. `value` is the part's
// already-formatted amount (as Tiles' own `value` is); the leading number in it is what
// sets the segment's share, so a caller can hand this "312 MB" and "4.1 MB" directly. Parts in
// different units ("412 GB" and "1.2 TB") hand their raw size as `amount`, which wins.
import QtQuick

Column {
    id: root
    property var parts: []                 // [{ label, value, color, amount? }]
    function share(p) { return p.amount !== undefined ? p.amount : (parseFloat(p.value) || 0) }
    readonly property real total: parts.reduce(function(s, p) { return s + root.share(p) }, 0)
    readonly property real segGap: theme.space(0.5)
    width: parent ? parent.width : theme.space(100)
    spacing: theme.space(2)
    Item {
        width: parent.width
        height: theme.space(2)
        Corner {
            anchors.fill: parent
            radius: height / 2
            smoothing: theme.cornerSmoothing
            color: theme.tileSunken
            borderColor: theme.line
            borderWidth: 1
        }
        Row {
            id: segments
            anchors.fill: parent
            spacing: root.segGap
            // Each part's share of the room; a share too thin to see is widened to a dot
            // the bar's height and the widest part gives that width back
            readonly property var widths: {
                var n = root.parts.length
                var room = width - root.segGap * (n - 1)
                var w = root.parts.map(function(p) { return root.total > 0 ? room * root.share(p) / root.total : 0 })
                var owed = 0, big = 0
                for (var i = 0; i < n; i++) {
                    if (w[i] > w[big]) big = i
                    if (w[i] < height) { owed += height - w[i]; w[i] = height }
                }
                if (n > 0) w[big] = Math.max(height, w[big] - owed)
                return w
            }
            Repeater {
                model: root.parts
                Corner {
                    required property int index
                    required property var modelData
                    width: segments.widths[index]
                    height: segments.height
                    radius: height / 2
                    smoothing: theme.cornerSmoothing
                    color: modelData.color
                }
            }
        }
    }
    Row {
        spacing: theme.space(4)
        Repeater {
            model: root.parts
            Row {
                required property var modelData
                spacing: theme.space(1.5)
                Corner {
                    anchors.verticalCenter: parent.verticalCenter
                    width: theme.space(2); height: width
                    radius: width / 2
                    smoothing: theme.cornerSmoothing
                    color: modelData.color
                }
                Text {
                    anchors.verticalCenter: parent.verticalCenter
                    text: modelData.label + " " + modelData.value
                    color: theme.textDim
                    font.family: theme.fontSans
                    font.pointSize: theme.typeSmall
                }
            }
        }
    }
}
