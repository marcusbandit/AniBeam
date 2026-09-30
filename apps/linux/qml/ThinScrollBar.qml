// The shell's one vertical scroll bar: a thin Corner pill thumb, visible only while there
// is more to scroll. Attach with `QC.ScrollBar.vertical: ThinScrollBar {}` on any Flickable
// or view (a QtQuick.Controls.Basic import as QC is still needed for the attached property
// itself, just not for this type).
import QtQuick
import QtQuick.Controls.Basic as QC

QC.ScrollBar {
    policy: QC.ScrollBar.AsNeeded
    visible: size < 1
    contentItem: Corner {
        implicitWidth: theme.space(1)
        radius: height / 2
        smoothing: theme.cornerSmoothing
        color: theme.lineStrong
        opacity: parent.active ? 1 : 0.4
    }
}
