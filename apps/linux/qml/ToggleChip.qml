// A switch written as a chip: its words, a check while on, faint while off. The control
// height, so it lines up with the pills and buttons beside it. A Tab stop: Space, Return
// or Enter flips it as a click does, and focus draws the focus ring (#325).
import QtQuick

Chip {
    id: root
    property bool checked: false
    signal toggled(bool on)
    height: theme.controlHeight
    mono: false
    clickable: interactive
    selected: checked
    dimmed: !checked
    icon: checked ? "check" : "plus"
    // Shown but not in play: what it switches has no effect right now (Group Nothing)
    property bool interactive: true
    opacity: interactive ? 1 : 0.45
    activeFocusOnTab: interactive
    borderColor: activeFocus ? theme.focusRing : selected ? Qt.rgba(theme.accent.r, theme.accent.g, theme.accent.b, 0.5) : theme.line
    function flip() { if (interactive) root.toggled(!checked) }
    onClicked: flip()
    Keys.onSpacePressed: flip()
    Keys.onReturnPressed: flip()
    Keys.onEnterPressed: flip()
}
