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
    // The cells a click would pick, by index: each shows a small square of the pick colour
    // inside it, which grows to fill the cube once it is picked (coded `p`). The owner of the
    // cubes says which, from hoverCell and whatever else it knows (a batch, a held Ctrl).
    property var lit: ({})
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

    // One cube is one Rectangle. The hover, the click and the tip belong to the whole
    // grid and find their cube from the pointer, rather than every cube carrying its own
    // MouseArea, hover box and Tooltip: on a card that was seven objects a cube, most of
    // the items a library page built (#289).
    readonly property real gridY: tabHeight + pad
    function cellAt(x, y) {
        var step = size + gap
        var c = Math.floor((x + gap / 2) / step), r = Math.floor((y - gridY + gap / 2) / step)
        if (c < 0 || c >= perRow || r < 0) return -1
        var i = r * perRow + c
        return i < cells.length ? i : -1
    }
    // The cube under the pointer, for the tip; the hover a click acts on is hoverCell
    readonly property int pointCell: hover.hovered ? cellAt(hover.point.position.x, hover.point.position.y) : -1
    // The cube a click would act on, which grows under the pointer
    readonly property int hoverCell: clickable ? pointCell : -1
    // How much a cube grows: under the pointer, or as part of a hot span
    function scaleOf(i) { return !theme.reducedMotion && (hoverCell === i || (spanHot && inSpan(i))) ? (inSpan(i) ? 1.1 : 1.18) : 1 }
    HoverHandler { id: hover; cursorShape: root.clickable && root.pointCell >= 0 ? Qt.PointingHandCursor : Qt.ArrowCursor }

    Grid {
        y: root.gridY
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
                scale: root.scaleOf(cube.index)
                Behavior on scale { enabled: !theme.reducedMotion; NumberAnimation { duration: theme.motionFast } }
            }
        }
    }
    // A part-downloaded episode fills from the foot. One Repeater over just those cubes,
    // usually none, rather than a fill slot in every cube.
    readonly property var partial: {
        var out = []
        if (!fills || fold !== 1) return out
        for (var i = 0; i < cells.length; i++)
            if (cells[i] === "t" && typeof fills[i] === "number" && fills[i] > 0) out.push(i)
        return out
    }
    Repeater {
        model: root.partial
        Item {
            required property int modelData
            readonly property real filled: Math.min(1, root.fills[modelData])
            readonly property var st: theme.cubeState("t")
            x: (modelData % root.perRow) * (root.size + root.gap)
            y: root.gridY + Math.floor(modelData / root.perRow) * (root.size + root.gap) + root.size - height
            width: root.size
            height: Math.round(root.size * filled)
            clip: true
            // Grows with its cube, about the cube's centre rather than its own
            property real grown: root.scaleOf(modelData)
            Behavior on grown { enabled: !theme.reducedMotion; NumberAnimation { duration: theme.motionFast } }
            transform: Scale { origin.x: root.size / 2; origin.y: root.size / 2 - (root.size - height); xScale: grown; yScale: grown }
            Rectangle {
                y: parent.height - root.size
                width: root.size; height: root.size
                radius: Math.max(1, root.size * 0.3)
                color: parent.st.fill
            }
        }
    }
    // The pick to come: a small square of the pick colour inside each cube a click would
    // pick, grown to the whole cube once it is picked (coded `p`). Only a grid that takes
    // clicks has them, one a cube, so a library card builds none.
    Repeater {
        model: root.clickable ? root.cells.length : 0
        Rectangle {
            required property int index
            readonly property string code: root.cells[index] || "g"
            readonly property real grow: code === "p" ? 1 : root.lit[index] && code !== "n" ? 0.42 : 0
            property real shown: grow
            Behavior on shown { enabled: !theme.reducedMotion; NumberAnimation { duration: theme.motionFast; easing.type: Easing.OutCubic } }
            visible: shown > 0.01
            width: Math.max(2, root.size * shown); height: width
            x: (index % root.perRow) * (root.size + root.gap) + (root.size - width) / 2
            y: root.gridY + Math.floor(index / root.perRow) * (root.size + root.gap) + (root.size - height) / 2
            radius: Math.max(1, root.size * 0.3) * shown
            color: theme.cubeState("p").fill
            opacity: code === "p" ? 1 : 0.9
            scale: root.scaleOf(index)
            Behavior on scale { enabled: !theme.reducedMotion; NumberAnimation { duration: theme.motionFast } }
        }
    }
    MouseArea {
        y: root.gridY
        width: parent.width
        height: Math.max(0, parent.height - root.gridY)
        enabled: root.clickable
        // A press between or past the cubes is not the grid's: it goes to what lies under
        onPressed: function(mouse) { mouse.accepted = root.cellAt(mouse.x, mouse.y + root.gridY) >= 0 }
        onClicked: function(mouse) {
            var i = root.cellAt(mouse.x, mouse.y + root.gridY)
            if (i < 0) return
            if (root.fold === 1) root.cubeClicked(i, mouse.modifiers)
            else root.rangeClicked(i * root.fold, Math.min(root.count, (i + 1) * root.fold) - 1, mouse.modifiers)
        }
    }

    // The tip: one for the grid, under the hovered cube. The probe stands where that cube
    // is, so the frame places the tip as it did when each cube had its own; Tooltip in
    // manual mode keeps the hover intent, the warm handover and the popover guard.
    // The tip's words for a cube, read when the pointer reaches it: worked out in the
    // handler below, not by a binding, so a warm handover never shows the cube before's
    function tipTextOf(i) {
        if (i < 0 || count > tipLimit || fold !== 1) return ""
        return tipFor ? tipFor(i, cells[i] || "g") : "Episode " + (first + i) + ": " + theme.cubeState(cells[i] || "g").label.toLowerCase()
    }
    Item {
        id: probe
        width: root.size; height: root.size
        Tooltip { id: cubeTip; manual: true; area: root; held: hover.hovered && root.pointCell >= 0 }
    }
    // Placed before the tip is asked for, so it never reads the last cube's place; asked on
    // every move to another cube, even one whose tip reads the same
    onPointCellChanged: {
        if (pointCell < 0) { cubeTip.release(); return }
        probe.x = (pointCell % perRow) * (size + gap)
        probe.y = gridY + Math.floor(pointCell / perRow) * (size + gap)
        cubeTip.text = tipTextOf(pointCell)
        cubeTip.show()
    }
    Connections {
        target: hover
        function onHoveredChanged() { if (!hover.hovered) cubeTip.release() }
    }
}
