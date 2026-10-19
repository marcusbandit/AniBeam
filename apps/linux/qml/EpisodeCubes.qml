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
    implicitHeight: count === 0 ? 0 : rows * size + (rows - 1) * gap
    implicitWidth: Math.min(cells.length, perRow) * (size + gap) - gap

    Grid {
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
                Tooltip {
                    text: root.count <= root.tipLimit && root.fold === 1
                        ? "Episode " + (root.first + cube.index) + ": " + cube.st.label.toLowerCase() : ""
                }
            }
        }
    }
}
