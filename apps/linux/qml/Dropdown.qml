// A pill of glass showing the current value; opens a floating pane listing the
// options, rows highlighted on hover, the current one in the accent. No chevron: an appended
// arrow on a control is a killed idiom (DESIGN.md), the hover edge and the accent focus
// edge say it opens.
//
// The list opens scrolled to the current row, under the pill, or above it when the window has
// more room there, and never taller than the room it has, so it never runs off the window.
// Up and Down move the highlight, Return or Enter picks, Escape closes the list alone (the
// popup takes the key, the frame never sees it). Closing hands key focus back to the pill,
// unless the press that closed it gave focus to something else. Picking emits `picked`; the
// owner sets `index`. While open the list is a popover on the frame's escape stack, so the
// frame's other layers (tooltips) know a popover is up.
//
// `interactive`, not the root's own `enabled`: `enabled` cascades to every child, so turning
// it off would take the popup's own focus and hover handling with it for no visual gain
// beyond what the caller's own row opacity already gives.
import QtQuick
import QtQuick.Window
import QtQuick.Controls.Basic as QC

Corner {
    id: root
    property var options: []
    property int index: 0
    property real minWidth: theme.space(40)
    property bool interactive: true
    signal picked(int i)

    readonly property string current: options.length > index && index >= 0 ? String(options[index]) : ""
    readonly property bool opened: pop.opened
    implicitWidth: Math.max(minWidth, label.implicitWidth + theme.space(4) * 2)
    scale: hover.pressed ? 0.98 : 1
    implicitHeight: theme.controlHeight
    radius: height / 2
    smoothing: theme.cornerSmoothing
    color: hover.containsMouse || pop.opened ? theme.raised : theme.tile
    borderColor: root.activeFocus || pop.opened ? theme.focusRing : hover.containsMouse ? theme.lineStrong : theme.line
    borderWidth: 1
    activeFocusOnTab: root.interactive

    function open() {
        if (!root.interactive || root.options.length === 0) return
        pop.hi = Math.max(0, Math.min(root.index, root.options.length - 1))
        place()
        pop.open()
    }
    function pick(i) {
        if (i < 0 || i >= root.options.length) return
        pop.close()
        root.picked(i)
    }

    // Under the pill when the list fits there, else on whichever side has more room, cut to
    // that room. Worked out on open (and on a window resize while open), not bound: the pill's
    // window position is a mapToItem, which no binding tracks.
    function place() {
        var win = root.Window.window
        if (!win) return
        var top = root.mapToItem(null, 0, 0).y
        var below = win.height - (top + root.height) - pop.gap - pop.edge
        var above = top - pop.gap - pop.edge
        var up = below < pop.wanted && above > below
        pop.height = Math.max(pop.rowHeight + pop.inset * 2, Math.min(pop.wanted, up ? above : below))
        pop.y = up ? -pop.height - pop.gap : root.height + pop.gap
        // As wide as the longest option, so no name is cut ("Tokyo Night Storm" in a pill
        // sized for the current one); never narrower than the pill, never past the window.
        var widest = 0
        for (var i = 0; i < root.options.length; i++) { metrics.text = String(root.options[i]); widest = Math.max(widest, metrics.advanceWidth) }
        var left = root.mapToItem(null, 0, 0).x
        pop.width = Math.min(win.width - pop.edge * 2, Math.max(root.width, Math.ceil(widest) + theme.space(3) * 2 + pop.inset * 2 + 2))
        pop.x = Math.min(0, win.width - pop.edge - (left + pop.width))
    }
    TextMetrics {
        id: metrics
        font.family: theme.fontFamily
        font.pointSize: theme.roleSize("ui")
        font.weight: theme.roleWeight("ui")
        font.variableAxes: theme.roleAxes("ui")
    }
    Connections {
        target: root.Window.window
        enabled: pop.opened
        function onHeightChanged() { root.place() }
    }

    Keys.onSpacePressed: open()
    Keys.onReturnPressed: open()
    Keys.onEnterPressed: open()
    Keys.onDownPressed: open()

    Label {
        id: label
        anchors.left: parent.left
        anchors.leftMargin: theme.space(4)
        anchors.right: parent.right
        anchors.rightMargin: theme.space(4)
        anchors.verticalCenter: parent.verticalCenter
        text: root.current
        role: "ui"
        elide: Text.ElideRight
    }
    MouseArea {
        id: hover
        anchors.fill: parent
        enabled: root.interactive
        hoverEnabled: root.interactive
        cursorShape: Qt.PointingHandCursor
        onClicked: { root.forceActiveFocus(); pop.opened ? pop.close() : root.open() }
    }

    QC.Popup {
        id: pop
        property int hi: 0
        readonly property real rowHeight: theme.controlHeight
        readonly property real inset: theme.space(1.5)
        readonly property real gap: theme.space(1)
        // Kept clear of the window's edge, the grout's width
        readonly property real edge: theme.space(2)
        readonly property real wanted: Math.min(root.options.length, 8) * rowHeight + inset * 2
        y: root.height + gap
        height: wanted
        // Qt's own positioner as the backstop for anything place() missed (x, mostly)
        margins: edge
        padding: 0
        focus: true
        background: null
        closePolicy: QC.Popup.CloseOnEscape | QC.Popup.CloseOnPressOutsideParent

        onOpened: {
            list.positionViewAtIndex(pop.hi, ListView.Center)
            list.forceActiveFocus()
            if (typeof frame !== "undefined" && frame.escapeStack) frame.escapeStack.push("popover", pop)
        }
        onClosed: {
            if (typeof frame !== "undefined" && frame.escapeStack) frame.escapeStack.pop(pop)
            // After the press that closed it has been delivered: a field it landed in keeps
            // the focus; focus left inside the list, on nothing, or on the window comes home.
            Qt.callLater(function() {
                var win = root.Window.window
                if (!win || !root.visible) return
                var f = win.activeFocusItem
                for (var p = f; p; p = p.parent) if (p === list) { f = null; break }
                if (!f || f === win.contentItem || (typeof frame !== "undefined" && f === frame)) root.forceActiveFocus()
            })
        }
        Component.onDestruction: if (typeof frame !== "undefined" && frame.escapeStack) frame.escapeStack.pop(pop)

        contentItem: Corner {
            // A named Dropdown names its list "<name>-popup" and rows "<name>-option-<i>",
            // for a --drive script; the popup lives under the window's overlay, not here.
            objectName: root.objectName !== "" ? root.objectName + "-popup" : ""
            radius: theme.radiusTile
            smoothing: theme.cornerSmoothing
            color: theme.glassFloat
            borderColor: theme.lineStrong
            borderWidth: 1
            highlight: theme.glassHighlight
            ListView {
                id: list
                anchors.fill: parent
                anchors.margins: pop.inset
                clip: true
                model: root.options
                currentIndex: pop.hi
                keyNavigationEnabled: false
                boundsBehavior: Flickable.StopAtBounds
                function move(to) { pop.hi = Math.max(0, Math.min(root.options.length - 1, to)); positionViewAtIndex(pop.hi, ListView.Contain) }
                Keys.onUpPressed: move(pop.hi - 1)
                Keys.onDownPressed: move(pop.hi + 1)
                Keys.onPressed: function(e) {
                    if (e.key === Qt.Key_PageUp) { move(pop.hi - 8); e.accepted = true }
                    else if (e.key === Qt.Key_PageDown) { move(pop.hi + 8); e.accepted = true }
                    else if (e.key === Qt.Key_Home) { move(0); e.accepted = true }
                    else if (e.key === Qt.Key_End) { move(root.options.length - 1); e.accepted = true }
                }
                Keys.onReturnPressed: root.pick(pop.hi)
                Keys.onEnterPressed: root.pick(pop.hi)
                Keys.onSpacePressed: root.pick(pop.hi)
                // Tab out of an open list closes it rather than walking the page behind it
                Keys.onTabPressed: pop.close()
                Keys.onBacktabPressed: pop.close()
                delegate: Corner {
                    required property int index
                    required property var modelData
                    objectName: root.objectName !== "" ? root.objectName + "-option-" + index : ""
                    width: list.width
                    height: pop.rowHeight
                    radius: theme.radiusInner(pop.inset)
                    smoothing: theme.cornerSmoothing
                    color: index === pop.hi ? theme.glassRaised : index === root.index ? theme.accentSoft : "transparent"
                    Label {
                        anchors.left: parent.left
                        anchors.leftMargin: theme.space(3)
                        anchors.right: parent.right
                        anchors.rightMargin: theme.space(3)
                        anchors.verticalCenter: parent.verticalCenter
                        text: String(modelData)
                        elide: Text.ElideRight
                        role: "ui"
                        color: index === root.index ? theme.accent : theme.text
                    }
                    MouseArea {
                        anchors.fill: parent
                        hoverEnabled: true
                        cursorShape: Qt.PointingHandCursor
                        onEntered: pop.hi = index
                        onClicked: root.pick(index)
                    }
                }
            }
        }
    }
}
