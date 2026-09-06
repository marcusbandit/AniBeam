// A list-status dot: the status hue, pulsing for Watching.
import QtQuick
import com.marcusrosado.AniBeam

Corner {
    property string status: ""
    width: theme.space(2.5); height: width
    radius: width / 2; smoothing: theme.cornerSmoothing
    color: theme.hue(Theme.statusHue(status))
    visible: status !== ""
    SequentialAnimation on opacity { running: status === "Watching"; loops: Animation.Infinite; NumberAnimation { to: 0.35; duration: 900 } NumberAnimation { to: 1; duration: 900 } }
    Tooltip { text: "On your list: " + (status === "Repeating" ? "Rewatching" : status) }
}
