// A floating surface (menus, tips, toasts, the drawer, the player's islands): a raised tile
// with a hairline. It was frosted glass, a live blur of the whole page sampled every frame;
// the mosaic has no glass, so the name stays for its callers and the fill is solid.
// `sourceItem`, `frostTint` and `frostSaturation` are accepted and ignored, so a caller
// written for the glass still loads. `fillOpacity` below 1 lets a surface over video (the
// player's chrome) show a trace of the picture through it.
import QtQuick

Corner {
    id: root
    property Item sourceItem: null
    property real frostTint: 0
    property real frostSaturation: 0
    property real fillOpacity: 1
    color: Qt.rgba(theme.raised.r, theme.raised.g, theme.raised.b, fillOpacity)
    smoothing: theme.cornerSmoothing
    borderColor: theme.line
    borderWidth: 1
}
