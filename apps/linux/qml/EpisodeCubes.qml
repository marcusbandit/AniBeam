// The episode cubes: one small rounded square per episode, its colour its state, in rows
// that wrap. The standard for "how far along is this series" everywhere in the shell; a
// single bar hides which episodes are there and which are not.
//
// `codes` is the core's string (SeriesCard.cubes, one code per episode from 1) or an array
// of codes; the colours are Tokens.cubeStates, so a new state needs no change here. A row
// longer than `maxRows` rows of the smallest cube folds several episodes into one cube,
// which takes the most urgent state among them (error, missing, downloading first), so a
// gap is never averaged away.
import QtQuick

Item {
    id: root
    property var codes: ""
    // The largest a cube grows, and the smallest it shrinks to before episodes fold together
    property real cubeSize: theme.space(2.5)
    property real minSize: theme.space(1.25)
    property real gap: Math.max(2, Math.round(cubeSize * 0.22))
    property int maxRows: 2
    // Hover names the episode and its state, up to this many cubes (one tip per cube)
    property int tipLimit: 80
    // The number the first cube stands for
    property int first: 1
    // A cube that takes a click (the Downloads show view picks releases this way). Folded
    // rows never do: a cube there is several episodes.
    property bool clickable: false
    // Overrides the hover text per cube: function(index, code) returning a string
    property var tipFor: null
    // A span of episodes that come as one (a batch): drawn in one container with a tab
    // naming it, and a hover on any of its cubes lifts them all. 0 means no span.
    property int spanFirst: 0
    property int spanLast: 0
    property string spanLabel: ""
    property bool spanChosen: false
    signal cubeClicked(int index)

    readonly property var list: typeof codes === "string" ? codes.split("") : (codes || [])
    readonly property int count: list.length
    // Cubes per row at the full size, then shrink toward minSize, then fold
    readonly property int perRowFull: Math.max(1, Math.floor((width + gap) / (cubeSize + gap)))
    readonly property real size: {
        if (count <= perRowFull * maxRows || width <= 0) return cubeSize
        var rows = maxRows
        var need = Math.ceil(count / rows)
        var s = (width + gap) / need - gap
        return Math.max(minSize, Math.min(cubeSize, Math.floor(s * 2) / 2))
    }
    readonly property int perRow: Math.max(1, Math.floor((width + gap) / (size + gap)))
    readonly property int capacity: perRow * maxRows
    readonly property int fold: count > capacity ? Math.ceil(count / capacity) : 1
    // The urgency a folded cube keeps: lower wins
    readonly property var urgency: ({ e: 0, m: 1, t: 2, p: 3, g: 4, a: 5, d: 6, l: 7, w: 8, f: 9 })
    readonly property var cells: {
        if (fold === 1) return list
        var out = []
        for (var i = 0; i < count; i += fold) {
            var best = list[i]
            for (var j = i + 1; j < Math.min(count, i + fold); j++) {
                var c = list[j]
                if ((urgency[c] ?? 4) < (urgency[best] ?? 4)) best = c
            }
            out.push(best)
        }
        return out
    }
    readonly property int rows: Math.ceil(cells.length / perRow)
    // The span in cells, the room its tab takes above the grid, and which cube is hovered
    readonly property bool spanned: spanFirst > 0 && spanLast >= spanFirst && count > 0
    readonly property int spanA: spanned ? Math.floor((spanFirst - 1) / fold) : -1
    readonly property int spanB: spanned ? Math.floor((Math.min(spanLast, count) - 1) / fold) : -1
    readonly property real pad: Math.max(2, Math.round(gap * 0.6))
    readonly property real tabHeight: spanned ? Math.round(tabText.implicitHeight + theme.space(1)) : 0
    property int hoverCell: -1
    readonly property bool spanHot: spanned && hoverCell >= spanA && hoverCell <= spanB
    function inSpan(i) { return spanned && i >= spanA && i <= spanB }
    implicitHeight: count === 0 ? 0 : tabHeight + rows * size + (rows - 1) * gap + (spanned ? pad : 0)
    implicitWidth: Math.max(0, count * (cubeSize + gap) - gap)

    // The span's container: one rounded band per grid row it crosses, a tab on the first
    readonly property color spanFill: spanChosen ? theme.accentSoft : spanHot ? theme.glassFloat : Qt.rgba(theme.text.r, theme.text.g, theme.text.b, 0.05)
    readonly property color spanEdge: spanChosen || spanHot ? theme.accentEdge : Qt.rgba(theme.text.r, theme.text.g, theme.text.b, 0.18)
    Repeater {
        model: root.spanned ? Math.floor(root.spanB / root.perRow) - Math.floor(root.spanA / root.perRow) + 1 : 0
        Rectangle {
            required property int index
            readonly property int row: Math.floor(root.spanA / root.perRow) + index
            readonly property int c0: index === 0 ? root.spanA % root.perRow : 0
            readonly property int c1: row === Math.floor(root.spanB / root.perRow) ? root.spanB % root.perRow : root.perRow - 1
            x: c0 * (root.size + root.gap) - root.pad
            y: root.tabHeight + row * (root.size + root.gap) - root.pad
            width: (c1 - c0 + 1) * (root.size + root.gap) - root.gap + 2 * root.pad
            height: root.size + 2 * root.pad
            radius: Math.max(2, root.size * 0.3 + root.pad)
            color: root.spanFill
            border.width: 1
            border.color: root.spanEdge
            Behavior on color { enabled: !theme.reducedMotion; ColorAnimation { duration: theme.motionFast } }
        }
    }
    Rectangle {
        visible: root.spanned
        x: (root.spanA % root.perRow) * (root.size + root.gap) - root.pad
        y: Math.floor(root.spanA / root.perRow) * (root.size + root.gap) - root.pad
        width: tabText.implicitWidth + theme.space(3)
        height: root.tabHeight + 1
        radius: theme.space(1.5)
        color: root.spanEdge
        Behavior on color { enabled: !theme.reducedMotion; ColorAnimation { duration: theme.motionFast } }
        // Square the bottom so the tab joins the band
        Rectangle { anchors.bottom: parent.bottom; width: parent.width; height: parent.radius; color: parent.color }
        Label {
            id: tabText
            anchors.centerIn: parent
            text: root.spanLabel
            role: "figure"
            font.pointSize: theme.typeSmall
            color: theme.text
        }
    }

    Grid {
        y: root.tabHeight
        columns: root.perRow
        spacing: root.gap
        Repeater {
            model: root.cells.length
            Rectangle {
                id: cube
                required property int index
                readonly property string code: root.cells[index] || "g"
                readonly property var st: theme.cubeState(code)
                width: root.size; height: root.size
                radius: Math.max(1, root.size * 0.3)
                color: st.fill
                border.width: st.edge === "transparent" ? 0 : 1
                border.color: st.edge
                scale: cubeMouse.containsMouse || (root.spanHot && root.inSpan(cube.index)) ? (root.inSpan(cube.index) ? 1.1 : 1.18) : 1
                Behavior on scale { enabled: !theme.reducedMotion; NumberAnimation { duration: theme.motionFast } }
                MouseArea {
                    id: cubeMouse
                    anchors.fill: parent
                    anchors.margins: -root.gap / 2
                    enabled: root.clickable && root.fold === 1
                    hoverEnabled: enabled
                    cursorShape: enabled ? Qt.PointingHandCursor : Qt.ArrowCursor
                    onContainsMouseChanged: if (containsMouse) root.hoverCell = cube.index; else if (root.hoverCell === cube.index) root.hoverCell = -1
                    onClicked: root.cubeClicked(cube.index)
                }
                Tooltip {
                    text: root.count > root.tipLimit || root.fold !== 1 ? ""
                        : root.tipFor ? root.tipFor(cube.index, cube.code)
                        : "Episode " + (root.first + cube.index) + ": " + cube.st.label.toLowerCase()
                }
            }
        }
    }
}
