// A labelled knob in the prototype bar: a small uppercase caption over any control.
import QtQuick

Column {
    id: root
    property string label: ""
    default property alias content: slot.data
    spacing: 4
    Text {
        text: root.label.toUpperCase()
        color: theme.bg
        opacity: 0.7
        font.family: theme.fontMono
        font.pointSize: theme.typeSmall
        font.letterSpacing: 1
    }
    Item {
        id: slot
        width: childrenRect.width
        height: childrenRect.height
    }
}
