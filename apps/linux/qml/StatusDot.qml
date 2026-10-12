// A list-status dot in the status hue. Static: an infinite pulse kept the whole window
// rendering at the display's rate for as long as a dot was on screen.
import QtQuick
import com.marcusrosado.AniBeam

Corner {
    property string status: ""
    width: theme.space(2.5); height: width
    radius: width / 2; smoothing: theme.cornerSmoothing
    color: theme.hue(Theme.statusHue(status))
    visible: status !== ""
    Tooltip { text: "On your list: " + (status === "Repeating" ? "Rewatching" : status) }
}
