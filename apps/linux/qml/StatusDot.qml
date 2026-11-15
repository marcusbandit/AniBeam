// A list-status square in the status hue. Static: an infinite pulse kept the whole window
// rendering at the display's rate for as long as a dot was on screen.
import QtQuick
import com.marcusrosado.AniBeam

Corner {
    property string status: ""
    width: theme.space(2.5); height: width
    // The quiet mapping (Tokens, status colours): the accent for the live states, greys
    // for the rest, rather than Theme.statusHue's six hues
    color: status === "Watching" ? theme.statusWatching
         : status === "Repeating" ? theme.statusRewatching
         : status === "Completed" ? theme.statusCompleted
         : status === "Paused" ? theme.statusPaused
         : status === "Dropped" ? theme.statusDropped
         : theme.statusPlanning
    visible: status !== ""
    Tooltip {
        readonly property string statusId: ({ Watching: "status-watching", Planning: "status-planning", Completed: "status-completed", Paused: "status-on-hold",
                                              Dropped: "status-dropped", Repeating: "status-rewatching" })[status] || ""
        text: words.t("status-dot-on-list", { status: statusId ? words.t(statusId) : status })
    }
}
