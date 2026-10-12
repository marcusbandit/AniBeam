// A floating pane of glass (menus, tips, toasts, the drawer, the player's islands): square,
// nearly opaque (theme.glassFloat) so it reads over anything, with the glass edge and the
// 1 px top highlight. It was frosted glass, a live blur of the page sampled every frame; that
// was removed for its cost, and the glass is faked with translucency and hairlines.
// `sourceItem`, `frostTint` and `frostSaturation` are accepted and ignored, so a caller
// written for the blur still loads. `fillOpacity` below 1 marks a surface over video (the
// player's chrome): it takes the video glass, black at 0.55 or more.
import QtQuick

Corner {
    id: root
    property Item sourceItem: null
    property real frostTint: 0
    property real frostSaturation: 0
    property real fillOpacity: 1
    readonly property bool overVideo: fillOpacity < 1
    color: overVideo ? Qt.rgba(theme.glassVideo.r, theme.glassVideo.g, theme.glassVideo.b, Math.max(theme.glassVideo.a, fillOpacity))
                     : theme.glassFloat
    highlight: theme.glassHighlight
    borderColor: overVideo ? Qt.rgba(1, 1, 1, 0.1) : theme.line
    borderWidth: 1
}
