// The Airing section's pager: earlier, the page number, later. Disabled at the ends. The two
// steps are Lucide chevrons rather than text glyphs, each with its tip.
import QtQuick

Row {
    id: root
    property int page: 0
    property bool hasMore: false
    signal prev()
    signal next()
    spacing: theme.space(1)
    Button { icon: "chevron-left"; small: true; flat: true; takesFocus: false; enabled: root.page > 0; opacity: enabled ? 1 : theme.disabledOpacity; onClicked: root.prev()
        Tooltip { text: "Earlier" } }
    Chip { text: String(root.page + 1); small: true; color: theme.surface; textColor: theme.textDim; anchors.verticalCenter: parent.verticalCenter }
    Button { icon: "chevron-right"; small: true; flat: true; takesFocus: false; enabled: root.hasMore; opacity: enabled ? 1 : theme.disabledOpacity; onClicked: root.next()
        Tooltip { text: "Later" } }
}
