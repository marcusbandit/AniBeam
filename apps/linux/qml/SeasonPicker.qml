// Any season, for Airing now: a floating pane of years, newest first, each a row of its four
// quarters. It opens under the control that asked, scrolled to the season showing, and
// scrolls natively all the way back to 1940. The season showing is in the accent, the
// quarter this moment falls in carries a dot. Picking emits `picked(name, year)` and closes;
// Escape or a click outside closes it alone. The arrow keys move a highlight over the cells
// and Return or Space picks it, so the keys that open it can finish the job.
import QtQuick
import QtQuick.Window
import QtQuick.Controls.Basic as QC

Item {
    id: root
    // The season showing, "Spring 2019", or "" for none of these
    property string current: ""
    property real nowMs: Date.now()
    signal picked(string name, int year)

    readonly property var names: ["Winter", "Spring", "Summer", "Fall"]
    readonly property int thisYear: new Date(nowMs).getUTCFullYear()
    readonly property int thisQuarter: Math.floor(new Date(nowMs).getUTCMonth() / 3)
    // AniList lists seasons a year or so ahead; the oldest is where its seasons start
    readonly property int newest: thisYear + 1
    readonly property int oldest: 1940
    readonly property int currentYear: current !== "" ? parseInt(current.split(" ")[1]) : thisYear
    readonly property bool opened: pop.opened

    function open() {
        // The highlight starts on the season showing, else the one this moment is in
        var parts = current.split(" ")
        pop.hiYear = current !== "" ? parseInt(parts[1]) : thisYear
        pop.hiQuarter = current !== "" ? names.indexOf(parts[0]) : thisQuarter
        place()
        pop.open()
    }
    function pickHighlighted() {
        pop.close()
        root.picked(names[pop.hiQuarter], pop.hiYear)
    }

    function place() {
        var win = root.Window.window
        if (!win) return
        var top = root.mapToItem(null, 0, 0).y
        var below = win.height - (top + root.height) - pop.gap - pop.edge
        pop.height = Math.max(pop.rowHeight * 3, Math.min(pop.wanted, below))
        pop.y = root.height + pop.gap
        var left = root.mapToItem(null, 0, 0).x
        pop.x = Math.min(0, win.width - pop.edge - (left + pop.width))
    }

    QC.Popup {
        id: pop
        readonly property real rowHeight: theme.controlHeight + theme.space(1)
        readonly property real inset: theme.space(2)
        readonly property real gap: theme.space(1.5)
        readonly property real edge: theme.space(2)
        readonly property real yearWidth: theme.space(16)
        readonly property real cellWidth: theme.space(21)
        readonly property real wanted: rowHeight * 9 + inset * 2
        width: yearWidth + cellWidth * 4 + inset * 2
        height: wanted
        margins: edge
        padding: 0
        focus: true
        background: null
        // Any press outside the pane closes it, the other cells of the switch under it included
        closePolicy: QC.Popup.CloseOnEscape | QC.Popup.CloseOnPressOutside
        property int hiYear: root.thisYear
        property int hiQuarter: 0
        function move(dy, dq) {
            var q = hiQuarter + dq, y = hiYear - dy
            if (q < 0) { q = 3; y-- } else if (q > 3) { q = 0; y++ }
            hiYear = Math.max(root.oldest, Math.min(root.newest, y))
            hiQuarter = q
            years.positionViewAtIndex(root.newest - hiYear, ListView.Contain)
        }

        onOpened: {
            years.positionViewAtIndex(root.newest - root.currentYear, ListView.Center)
            years.forceActiveFocus()
            if (typeof frame !== "undefined" && frame.escapeStack) frame.escapeStack.push("popover", pop)
        }
        onClosed: if (typeof frame !== "undefined" && frame.escapeStack) frame.escapeStack.pop(pop)
        Component.onDestruction: if (typeof frame !== "undefined" && frame.escapeStack) frame.escapeStack.pop(pop)

        contentItem: Corner {
            objectName: "season-picker"
            radius: theme.radiusTile
            smoothing: theme.cornerSmoothing
            color: theme.glassFloat
            borderColor: theme.lineStrong
            borderWidth: 1
            highlight: theme.glassHighlight
            ListView {
                id: years
                anchors.fill: parent
                anchors.margins: pop.inset
                clip: true
                model: root.newest - root.oldest + 1
                boundsBehavior: Flickable.StopAtBounds
                interactive: false
                WheelStep { id: wheel; flick: years }
                Keys.onUpPressed: pop.move(-1, 0)
                Keys.onDownPressed: pop.move(1, 0)
                Keys.onLeftPressed: pop.move(0, -1)
                Keys.onRightPressed: pop.move(0, 1)
                Keys.onReturnPressed: root.pickHighlighted()
                Keys.onEnterPressed: root.pickHighlighted()
                Keys.onSpacePressed: root.pickHighlighted()
                Keys.onTabPressed: pop.close()
                Keys.onBacktabPressed: pop.close()
                Keys.onPressed: function(e) {
                    if (e.key === Qt.Key_PageUp) { pop.move(-8, 0); e.accepted = true }
                    else if (e.key === Qt.Key_PageDown) { pop.move(8, 0); e.accepted = true }
                }
                delegate: Item {
                    id: yearRow
                    required property int index
                    readonly property int year: root.newest - index
                    width: years.width
                    height: pop.rowHeight
                    Label {
                        x: theme.space(2)
                        width: pop.yearWidth - theme.space(2)
                        anchors.verticalCenter: parent.verticalCenter
                        text: yearRow.year
                        role: "ledger"
                        color: yearRow.year === root.thisYear ? theme.text : theme.textFaint
                    }
                    Row {
                        x: pop.yearWidth
                        anchors.verticalCenter: parent.verticalCenter
                        Repeater {
                            model: 4
                            Corner {
                                id: cell
                                required property int index
                                readonly property string label: root.names[index] + " " + yearRow.year
                                readonly property bool on: label === root.current
                                readonly property bool here: yearRow.year === root.thisYear && index === root.thisQuarter
                                readonly property bool lit: cm.containsMouse || (yearRow.year === pop.hiYear && index === pop.hiQuarter)
                                objectName: "season-" + root.names[index].toLowerCase() + "-" + yearRow.year
                                width: pop.cellWidth - theme.space(1)
                                height: theme.controlHeight
                                radius: height / 2
                                smoothing: theme.cornerSmoothing
                                color: on ? theme.accentSoft : lit ? theme.floatHover : "transparent"
                                borderColor: on ? theme.accentEdge : "transparent"
                                borderWidth: 1
                                Row {
                                    anchors.centerIn: parent
                                    spacing: theme.space(1.5)
                                    Rectangle {
                                        visible: cell.here
                                        anchors.verticalCenter: parent.verticalCenter
                                        width: theme.space(1.5); height: width; radius: width / 2
                                        color: theme.accent
                                    }
                                    Label {
                                        text: root.names[cell.index]
                                        role: "ui"
                                        color: cell.on ? theme.accent : cell.lit ? theme.text : theme.textDim
                                    }
                                }
                                MouseArea {
                                    Accessible.role: Accessible.Button
                                    Accessible.onPressAction: { pop.close(); root.picked(root.names[cell.index], yearRow.year) }
                                    Accessible.name: theme.nameNear(parent)
                                    id: cm
                                    anchors.fill: parent
                                    hoverEnabled: true
                                    cursorShape: Qt.PointingHandCursor
                                    onEntered: { pop.hiYear = yearRow.year; pop.hiQuarter = cell.index }
                                    onClicked: { pop.close(); root.picked(root.names[cell.index], yearRow.year) }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
