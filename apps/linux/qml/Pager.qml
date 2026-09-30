// ‹ page ›: the Airing section's pager. Disabled at the ends.
import QtQuick

Row {
    id: root
    property int page: 0
    property bool hasMore: false
    signal prev()
    signal next()
    spacing: theme.space(1)
    Button { text: "‹"; small: true; flat: true; enabled: root.page > 0; opacity: enabled ? 1 : theme.disabledOpacity; onClicked: root.prev() }
    Chip { text: String(root.page + 1); small: true; color: theme.surface; textColor: theme.textDim; anchors.verticalCenter: parent.verticalCenter }
    Button { text: "›"; small: true; flat: true; enabled: root.hasMore; opacity: enabled ? 1 : theme.disabledOpacity; onClicked: root.next() }
}
