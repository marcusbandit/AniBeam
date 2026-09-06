// A round glyph button for the player's scrim. Nothing but the glyph at rest; the fill
// arrives on hover and deepens on press, and the tip comes from the frame's own overlay so
// it is never clipped by the controls island.
//
// `interactive`, not the root's own `enabled`: `enabled` is inherited by every child, so
// switching it off would take the Tooltip's MouseArea with it and the button could never
// say why it is dim. Only the input MouseArea is gated, exactly as EpisodeRow's mark does.
import QtQuick

Corner {
    id: root
    property string glyph: ""
    property string tip: ""
    property bool active: false
    property bool interactive: true
    signal clicked()

    width: theme.space(9); height: width
    radius: width / 2; smoothing: theme.cornerSmoothing
    color: m.pressed ? theme.surfacePressed : (m.containsMouse ? theme.surfaceRaised : "transparent")
    opacity: interactive ? 1 : theme.disabledOpacity

    Icon { anchors.centerIn: parent; glyph: root.glyph; size: theme.space(4.5); color: root.active ? theme.accent : theme.text }
    MouseArea { id: m; anchors.fill: parent; hoverEnabled: true; enabled: root.interactive; cursorShape: Qt.PointingHandCursor; onClicked: root.clicked() }
    Tooltip { text: root.tip }
}
