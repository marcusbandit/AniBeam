// One setting: a label, an optional status line in the fixed face, an optional helper
// under it and an optional result line (`status`, a fact in the fixed face) under that,
// and a control slot on the right. When the control would leave the words less than
// theme.space(60) the control drops under them instead, right-aligned, so a helper is
// never crushed into a ribbon. Never shorter than theme.space(12). Sized through
// implicitHeight so it sits in a Column and a ColumnLayout alike; the two blocks are
// placed by x and y rather than anchors, so the switch between the two arrangements is
// one binding each and never leaves a stale anchor behind. A hairline closes the row at
// its foot (`hairline`), so a group reads as a list; the group's last row leaves it off,
// since the pane's own edge closes it. Search finds a row by its label and `pulse` lights
// it in the accent for a moment, so the eye lands on the row it asked for.
import QtQuick
import QtQuick.Layouts

Item {
    id: root
    property string label: ""
    property string line: ""
    property string helper: ""
    property string status: ""
    property bool hairline: !(parent && parent.lastRow === root)
    readonly property bool settingTarget: true
    property real flash: 0
    function pulse() { flashAnim.restart() }
    SequentialAnimation {
        id: flashAnim
        NumberAnimation { target: root; property: "flash"; to: 1; duration: 160 }
        PauseAnimation { duration: 700 }
        NumberAnimation { target: root; property: "flash"; to: 0; duration: 600 }
    }
    default property alias control: slot.data

    readonly property real pad: theme.space(2.5)
    readonly property real gutter: theme.space(6)
    readonly property bool wordy: line !== "" || helper !== "" || status !== ""
    readonly property bool stacked: wordy && width - slot.width - gutter < theme.space(60)
    width: parent ? parent.width : theme.space(100)
    implicitHeight: stacked ? words.implicitHeight + pad + slot.height + pad * 2
                            : Math.max(theme.space(12), words.implicitHeight + pad * 2, slot.height + pad * 2)
    Layout.fillWidth: true

    // The search highlight: a soft accent wash under the row, reaching past its sides into
    // the pane's padding
    Rectangle {
        visible: root.flash > 0
        x: -theme.space(3); width: root.width + theme.space(6)
        y: 1; height: root.height - 2
        radius: theme.radiusSm
        color: theme.accentSoft
        opacity: root.flash
    }
    Column {
        id: words
        x: 0
        y: root.stacked ? root.pad : Math.round((root.height - height) / 2)
        width: root.stacked ? root.width : Math.max(0, root.width - slot.width - root.gutter)
        spacing: theme.space(0.5)
        Text {
            width: parent.width
            text: root.label
            color: theme.text
            wrapMode: Text.Wrap
            font.family: theme.fontSans
            font.pointSize: theme.typeNormal
        }
        Text {
            visible: root.line !== ""
            width: parent.width
            text: root.line
            color: theme.textDim
            wrapMode: Text.Wrap
            font.family: theme.fontUi
            font.pointSize: theme.typeSmall
        }
        Text {
            visible: root.helper !== ""
            width: parent.width
            text: root.helper
            color: theme.textDim
            wrapMode: Text.Wrap
            font.family: theme.fontSans
            font.pointSize: theme.typeSmall
        }
        Text {
            visible: root.status !== ""
            width: parent.width
            text: root.status
            color: theme.textDim
            wrapMode: Text.Wrap
            font.family: theme.fontUi
            font.pointSize: theme.typeSmall
        }
    }
    Item {
        id: slot
        x: root.width - width
        y: root.stacked ? root.height - root.pad - height : Math.round((root.height - height) / 2)
        width: childrenRect.width
        height: childrenRect.height
    }
    Rectangle { visible: root.hairline; anchors.bottom: parent.bottom; width: parent.width; height: 1; color: theme.line }
}
