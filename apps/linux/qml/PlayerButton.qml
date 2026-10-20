// A square glyph button for the player's islands. Nothing but the glyph at rest; the fill
// arrives on hover and deepens on press, and the tip comes from the frame's own overlay so
// it is never clipped by the island.
//
// One MouseArea owns both the hover and the click: a Tooltip MouseArea stacked on top of it
// took the hover, so the fill never showed. The tip runs off the same hover with its own
// 600 ms intent, and a click cancels it.
//
// `interactive`, not the root's own `enabled`: a disabled MouseArea loses its hover too, and
// the button could never say why it is dim. Only the click is gated.
import QtQuick

Corner {
    id: root
    property string glyph: ""
    property string tip: ""
    property bool active: false
    property bool interactive: true
    property real glyphSize: theme.space(4.5)
    property color restColor: "transparent"
    signal clicked()

    width: theme.space(9); height: width
    readonly property bool hovered: m.containsMouse
    color: m.pressed && interactive ? theme.glassOf(0.16) : (hovered && interactive ? theme.glassOf(0.1) : restColor)
    opacity: interactive ? 1 : theme.disabledOpacity

    Icon { anchors.centerIn: parent; glyph: root.glyph; size: root.glyphSize; color: root.active ? theme.accent : theme.text }
    // Whether this button's tip is the one on screen, so a text change can follow it and a
    // click can take it down without a hover that has since moved on re-showing it.
    property bool tipShown: false
    function showTip() { tipShown = true; frame.showTip(root, root.tip) }
    function dropTip() { tipIntent.stop(); if (tipShown) frame.hideTip(); tipShown = false }
    MouseArea {
        id: m
        anchors.fill: parent
        hoverEnabled: true
        cursorShape: root.interactive ? Qt.PointingHandCursor : Qt.ArrowCursor
        onClicked: { root.dropTip(); if (root.interactive) root.clicked() }
    }
    Timer { id: tipIntent; interval: 600; onTriggered: root.showTip() }
    onHoveredChanged: { if (hovered && tip !== "") { tipIntent.interval = frame.tipWarm() ? 1 : 600; tipIntent.restart() } else dropTip() }
    // A tip whose text changes under the pointer (Play becoming Pause) follows it.
    onTipChanged: if (tipShown) { if (tip === "") dropTip(); else showTip() }
    // Hidden with the chrome while the pointer rests on it: no exit would ever arrive.
    onVisibleChanged: if (!visible) dropTip()
    Component.onDestruction: dropTip()
}
