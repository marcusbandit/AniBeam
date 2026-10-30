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
    // A cube that takes a click (the Downloads show view picks releases this way). A folded
    // cube stands for several episodes, so its click comes as rangeClicked instead.
    property bool clickable: false
    // Overrides the hover text per cube: function(index, code) returning a string
    property var tipFor: null
    // A span of episodes that come as one (a batch): drawn in one container with a tab
    // naming it, and a hover on any of its cubes lifts them all. 0 means no span.
    property int spanFirst: 0
    property int spanLast: 0
    property string spanLabel: ""
    property bool spanChosen: false
    // How far along each cube is, 0 to 1, for a cube coded `t`: it fills from the bottom
    // like a glass, over the not-downloaded ground. Unset or a number missing is a full cube.
    property var fills: null
    // One cube's tag above it (how far the episode downloading now is): its index, its text.
    // The row makes room for it above the cubes.
    property int badgeIndex: -1
    property string badgeText: ""
    signal cubeClicked(int index, int modifiers)
    // A folded cube's click: the first and last episode index it stands for
    signal rangeClicked(int first, int last, int modifiers)

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
    // The span in cells, the room its label takes above the grid, and which cube is hovered
    readonly property bool spanned: spanFirst > 0 && spanLast >= spanFirst && count > 0
    readonly property int spanA: spanned ? Math.floor((spanFirst - 1) / fold) : -1
    readonly property int spanB: spanned ? Math.floor((Math.min(spanLast, count) - 1) / fold) : -1
    readonly property real pad: spanned ? Math.max(3, Math.round(gap * 0.8)) : 0
    readonly property real tabHeight: (spanned ? Math.round(tabText.implicitHeight + theme.space(1)) : 0) + badgeRoom
    readonly property bool badged: badgeIndex >= 0 && badgeIndex < cells.length && badgeText !== ""
    // The tag overlaps its cube's top edge by half its height; only the rest needs room
    readonly property real badgeRoom: badged ? Math.round((badgeLabel.implicitHeight + theme.space(1)) / 2) : 0
    property int hoverCell: -1
    readonly property bool spanHot: spanned && hoverCell >= spanA && hoverCell <= spanB
    function inSpan(i) { return spanned && i >= spanA && i <= spanB }
    implicitHeight: count === 0 ? 0 : tabHeight + 2 * pad + rows * size + (rows - 1) * gap
    implicitWidth: Math.max(0, count * (cubeSize + gap) - gap)

    // The span's container: one band per grid row it crosses, edge to edge between rows so
    // they read as one shape, rounded only on its outside corners; no outline, the fill
    // carries the state. Its label sits above it, plain text, no tab.
    readonly property color spanFill: spanChosen ? theme.accentSoft : spanHot ? theme.glassOf(theme.dark ? 0.13 : 0.14) : theme.glassOf(theme.dark ? 0.07 : 0.08)
    readonly property int rowA: Math.floor(spanA / Math.max(1, perRow))
    readonly property int rowB: Math.floor(spanB / Math.max(1, perRow))
    function c0Of(r) { return r === rowA ? spanA % perRow : 0 }
    function c1Of(r) { return r === rowB ? spanB % perRow : perRow - 1 }
    Repeater {
        model: root.spanned ? root.rowB - root.rowA + 1 : 0
        Rectangle {
            required property int index
            readonly property int row: root.rowA + index
            readonly property bool first: row === root.rowA
            readonly property bool last: row === root.rowB
            readonly property int c0: root.c0Of(row)
            readonly property int c1: root.c1Of(row)
            readonly property real r: Math.max(3, root.size * 0.3 + root.pad)
            readonly property real lead: first ? root.pad : root.gap / 2
            x: c0 * (root.size + root.gap) - root.pad
            y: root.tabHeight + root.pad + row * (root.size + root.gap) - lead
            width: (c1 - c0 + 1) * (root.size + root.gap) - root.gap + 2 * root.pad
            height: root.size + lead + (last ? root.pad : root.gap / 2)
            // A corner rounds where nothing of the span continues past it
            topLeftRadius: first || c0 < root.c0Of(row - 1) ? r : 0
            topRightRadius: first || c1 > root.c1Of(row - 1) ? r : 0
            bottomLeftRadius: last || c0 < root.c0Of(row + 1) ? r : 0
            bottomRightRadius: last || c1 > root.c1Of(row + 1) ? r : 0
            color: root.spanFill
            Behavior on color { enabled: !theme.reducedMotion; ColorAnimation { duration: theme.motionFast } }
        }
    }
    Label {
        id: tabText
        visible: root.spanned
        x: (root.spanA % Math.max(1, root.perRow)) * (root.size + root.gap)
        y: 0
        text: root.spanLabel
        role: "figure"
        font.pointSize: theme.typeSmall
        color: root.spanChosen ? theme.accent : root.spanHot ? theme.text : theme.textDim
    }

    Rectangle {
        visible: root.badged
        readonly property int col: root.badgeIndex % Math.max(1, root.perRow)
        readonly property int row: Math.floor(root.badgeIndex / Math.max(1, root.perRow))
        width: badgeLabel.implicitWidth + theme.space(3)
        height: badgeLabel.implicitHeight + theme.space(1)
        radius: height / 2
        x: col * (root.size + root.gap) + (root.size - width) / 2
        y: root.tabHeight + root.pad + row * (root.size + root.gap) - height / 2
        z: 1
        color: theme.raised
        border.width: 1
        border.color: theme.line
        Label { id: badgeLabel; anchors.centerIn: parent; text: root.badgeText; role: "figure"; font.pointSize: theme.typeSmall * 0.9; color: theme.text }
    }

    Grid {
        y: root.tabHeight + root.pad
        columns: root.perRow
        spacing: root.gap
        Repeater {
            model: root.cells.length
            Rectangle {
                id: cube
                required property int index
                readonly property string code: root.cells[index] || "g"
                readonly property var st: theme.cubeState(code)
                readonly property real filled: code === "t" && root.fills && root.fold === 1 && typeof root.fills[index] === "number" ? Math.max(0, Math.min(1, root.fills[index])) : -1
                width: root.size; height: root.size
                radius: Math.max(1, root.size * 0.3)
                color: filled >= 0 ? theme.cubeState("g").fill : st.fill
                border.width: st.edge === "transparent" ? 0 : 1
                border.color: st.edge
                scale: cubeMouse.containsMouse || (root.spanHot && root.inSpan(cube.index)) ? (root.inSpan(cube.index) ? 1.1 : 1.18) : 1
                Behavior on scale { enabled: !theme.reducedMotion; NumberAnimation { duration: theme.motionFast } }
                Item {
                    visible: cube.filled > 0
                    anchors.bottom: parent.bottom
                    width: parent.width
                    height: Math.round(parent.height * cube.filled)
                    clip: true
                    Rectangle {
                        y: parent.height - cube.height
                        width: cube.width; height: cube.height
                        radius: cube.radius
                        color: cube.st.fill
                    }
                }
                MouseArea {
                    id: cubeMouse
                    anchors.fill: parent
                    anchors.margins: -root.gap / 2
                    enabled: root.clickable
                    hoverEnabled: enabled
                    cursorShape: enabled ? Qt.PointingHandCursor : Qt.ArrowCursor
                    onContainsMouseChanged: if (containsMouse) root.hoverCell = cube.index; else if (root.hoverCell === cube.index) root.hoverCell = -1
                    onClicked: function(mouse) {
                        if (root.fold === 1) root.cubeClicked(cube.index, mouse.modifiers)
                        else root.rangeClicked(cube.index * root.fold, Math.min(root.count, (cube.index + 1) * root.fold) - 1, mouse.modifiers)
                    }
                }
                // The hover reaches half the gap each way, so neighbouring cubes meet and a
                // move from one to the next never crosses empty ground
                Item {
                    anchors.fill: parent
                    anchors.margins: -root.gap / 2
                    Tooltip {
                        text: root.count > root.tipLimit || root.fold !== 1 ? ""
                            : root.tipFor ? root.tipFor(cube.index, cube.code)
                            : "Episode " + (root.first + cube.index) + ": " + cube.st.label.toLowerCase()
                    }
                }
            }
        }
    }
}
