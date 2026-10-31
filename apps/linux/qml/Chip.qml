// A square tag of glass, with an optional Lucide glyph before the text.
// `mono` (the default) sets the label in the ledger cut, tabular figures for counts and
// codes; off, the ui cut. `maxLabelWidth` is opt-in (-1, the default, leaves the label at
// its natural width): set it to cap the label and elide the middle instead of growing the
// chip past the room the caller has, a URL or path being the case that needs it.
// `dimmed` greys the chip out (a switch that is off: faint text, no fill) and `hint` is a
// small faint word after the label, the state a lit or greyed chip cannot say on its own.
import QtQuick

Corner {
    id: root
    property string text: ""
    property string icon: ""
    property color textColor: theme.text
    property bool small: false
    property bool mono: true
    property bool selected: false
    property bool clickable: false
    property bool dimmed: false
    property string hint: ""
    property real maxLabelWidth: -1
    signal clicked()

    implicitWidth: content.implicitWidth + theme.space(small ? 2 : 2.5) * 2
    implicitHeight: label.implicitHeight + theme.space(small ? 0.75 : 1.25) * 2
    radius: height / 2
    smoothing: theme.cornerSmoothing
    color: selected ? theme.accentSoft : clickable && hover.containsMouse ? theme.glassPressed : dimmed ? "transparent" : theme.raised
    borderColor: selected ? Qt.rgba(theme.accent.r, theme.accent.g, theme.accent.b, 0.5) : clickable && hover.containsMouse ? theme.lineStrong : theme.line
    borderWidth: 1
    scale: theme.reducedMotion ? 1 : (hover.pressed ? 0.98 : 1)

    readonly property color ink: selected ? theme.accent : dimmed ? theme.textFaint : textColor
    Row {
        id: content
        anchors.centerIn: parent
        spacing: theme.space(1)
        Icon {
            visible: root.icon !== ""
            anchors.verticalCenter: parent.verticalCenter
            glyph: root.icon
            color: root.ink
            size: theme.space(root.small ? 3.5 : 4)
        }
        Label {
            id: label
            anchors.verticalCenter: parent.verticalCenter
            text: root.text
            color: root.ink
            role: root.mono ? "ledger" : "ui"
            font.pointSize: root.small ? theme.typeSmall : theme.typeNormal
            width: root.maxLabelWidth >= 0 ? Math.min(implicitWidth, root.maxLabelWidth) : implicitWidth
            elide: root.maxLabelWidth >= 0 ? Text.ElideMiddle : Text.ElideNone
        }
        Label {
            visible: root.hint !== ""
            anchors.verticalCenter: parent.verticalCenter
            text: root.hint
            color: theme.textFaint
            role: "ui"
            font.pointSize: theme.typeSmall
        }
    }
    MouseArea {
        id: hover
        anchors.fill: parent
        enabled: root.clickable
        hoverEnabled: root.clickable
        cursorShape: Qt.PointingHandCursor
        onClicked: root.clicked()
    }
}
