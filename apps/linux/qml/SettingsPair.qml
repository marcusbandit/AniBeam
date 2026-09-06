// Two columns that fill the width together, the leading one `split` of it, or one under
// the other when the width holds fewer than two of theme.space(100) and the gap between.
// `leftPanels` and `rightPanels` are lists of Components, each instantiated into a Loader
// in its column's ColumnLayout: a panel's own `Layout.*` attached properties apply only
// when the panel sits directly in a Layout, so once it is a Loader's item instead, the
// Loader forwards them in its place. Named `leftPanels`/`rightPanels` rather than the bare
// `left`/`right` the spec calls them: `QQuickItem` already has a final `left` and `right`
// property of its own (the anchor line `anchors.left` reads), and qmllint flags shadowing
// either as a property-override hazard, so this Item-rooted component picks names that do
// not collide with its base type. Each column is a ColumnLayout: with something in it that
// grows it is the pair's full height, otherwise its natural height. The pair itself grows
// in the tab when either column does, and never shrinks under its natural height. Stacked,
// the spare height goes to the trailing column when it grows, else to the leading one.
import QtQuick
import QtQuick.Layouts

Item {
    id: pair
    property real split: 0.5
    property real columnGap: theme.space(6)
    property real rowGap: theme.space(6)
    property var leftPanels: []
    property var rightPanels: []

    readonly property bool twoUp: width >= 2 * theme.space(100) + columnGap
    readonly property real leftW: twoUp ? Math.round((width - columnGap) * split) : width
    function anyGrows(rep) { for (var i = 0; i < rep.count; i++) { var d = rep.itemAt(i); if (d && d.item && d.item.grows === true) return true } return false }
    readonly property bool leadGrows: anyGrows(leadRep)
    readonly property bool trailGrows: anyGrows(trailRep)
    readonly property bool grows: leadGrows || trailGrows
    readonly property real natural: twoUp ? Math.max(leadCol.implicitHeight, trailCol.implicitHeight)
                                          : leadCol.implicitHeight + rowGap + trailCol.implicitHeight
    readonly property real extra: Math.max(0, height - natural)
    width: parent ? parent.width : theme.space(100)
    implicitHeight: natural
    Layout.fillWidth: true
    Layout.fillHeight: grows
    Layout.minimumHeight: natural
    Layout.preferredHeight: natural
    Layout.alignment: Qt.AlignTop

    ColumnLayout {
        id: leadCol
        width: pair.leftW
        height: pair.twoUp ? (pair.leadGrows ? pair.height : implicitHeight)
                           : implicitHeight + (pair.leadGrows && !pair.trailGrows ? pair.extra : 0)
        spacing: pair.rowGap
        Repeater {
            id: leadRep
            model: pair.leftPanels
            Loader {
                required property var modelData
                sourceComponent: modelData
                Layout.fillWidth: true
                Layout.fillHeight: item ? item.grows === true : false
                Layout.minimumHeight: item ? item.implicitHeight : 0
            }
        }
    }
    ColumnLayout {
        id: trailCol
        x: pair.twoUp ? pair.leftW + pair.columnGap : 0
        y: pair.twoUp ? 0 : leadCol.height + pair.rowGap
        width: pair.twoUp ? pair.width - pair.leftW - pair.columnGap : pair.width
        height: pair.twoUp ? (pair.trailGrows ? pair.height : implicitHeight)
                           : implicitHeight + (pair.trailGrows ? pair.extra : 0)
        spacing: pair.rowGap
        Repeater {
            id: trailRep
            model: pair.rightPanels
            Loader {
                required property var modelData
                sourceComponent: modelData
                Layout.fillWidth: true
                Layout.fillHeight: item ? item.grows === true : false
                Layout.minimumHeight: item ? item.implicitHeight : 0
            }
        }
    }
}
