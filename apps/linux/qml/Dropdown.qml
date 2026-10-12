// A sunken pill showing the current value and a chevron glyph; opens a raised tile listing
// the options under it, rows highlighted on hover, the current one in the accent. Up and Down move
// the highlight, Return picks, Escape closes. Picking emits `picked`; the owner sets `index`.
// `interactive`, not the root's own `enabled`: `enabled` cascades to every child, so turning
// it off would take the popup's own focus and hover handling with it for no visual gain
// beyond what the caller's own row opacity already gives, exactly as PlayerButton's own
// `interactive` avoids disabling its Tooltip's MouseArea.
import QtQuick
import QtQuick.Controls.Basic as QC

Corner {
    id: root
    property var options: []
    property int index: 0
    property real minWidth: theme.space(40)
    property bool interactive: true
    signal picked(int i)

    readonly property string current: options.length > index && index >= 0 ? String(options[index]) : ""
    implicitWidth: Math.max(minWidth, label.implicitWidth + chevron.width + theme.space(4) * 2 + theme.space(2))
    scale: hover.pressed ? 0.98 : 1
    implicitHeight: theme.controlHeight
    radius: height / 2
    smoothing: theme.cornerSmoothing
    color: hover.containsMouse ? theme.tone(theme.tileSunken, theme.text, 0.03) : theme.tileSunken
    borderColor: activeFocus || pop.opened ? theme.focusRing : (hover.containsMouse ? theme.lineStrong : theme.line)
    borderWidth: activeFocus || pop.opened ? theme.space(0.5) : 1
    activeFocusOnTab: root.interactive

    function open() { if (!root.interactive) return; pop.hi = index; pop.open() }
    Keys.onSpacePressed: open()
    Keys.onReturnPressed: open()
    Keys.onDownPressed: open()

    Label {
        id: label
        anchors.left: parent.left
        anchors.leftMargin: theme.space(4)
        anchors.right: chevron.left
        anchors.rightMargin: theme.space(2)
        anchors.verticalCenter: parent.verticalCenter
        text: root.current
        role: "ui"
        elide: Text.ElideRight
    }
    Icon {
        id: chevron
        anchors.right: parent.right
        anchors.rightMargin: theme.space(3)
        anchors.verticalCenter: parent.verticalCenter
        glyph: "chevron-down"
        size: theme.space(4)
        color: theme.textDim
        rotation: pop.opened ? 180 : 0
    }
    MouseArea {
        id: hover
        anchors.fill: parent
        enabled: root.interactive
        hoverEnabled: root.interactive
        cursorShape: Qt.PointingHandCursor
        onClicked: { root.forceActiveFocus(); pop.opened ? pop.close() : root.open() }
    }

    QC.Popup {
        id: pop
        property int hi: 0
        readonly property real rowHeight: theme.controlHeight
        readonly property real inset: theme.space(1.5)
        y: root.height + theme.space(1)
        width: root.width
        height: Math.min(root.options.length, 8) * rowHeight + inset * 2
        padding: 0
        focus: true
        background: null
        closePolicy: QC.Popup.CloseOnEscape | QC.Popup.CloseOnPressOutsideParent
        onOpened: list.forceActiveFocus()
        contentItem: Corner {
            // A named Dropdown names its list "<name>-popup" and rows "<name>-option-<i>",
            // for a --drive script; the popup lives under the window's overlay, not here.
            objectName: root.objectName !== "" ? root.objectName + "-popup" : ""
            radius: theme.radiusTile
            smoothing: theme.cornerSmoothing
            color: theme.raised
            borderColor: theme.line
            borderWidth: 1
            ListView {
                id: list
                anchors.fill: parent
                anchors.margins: pop.inset
                clip: true
                model: root.options
                currentIndex: pop.hi
                keyNavigationEnabled: false
                Keys.onUpPressed: pop.hi = Math.max(0, pop.hi - 1)
                Keys.onDownPressed: pop.hi = Math.min(root.options.length - 1, pop.hi + 1)
                Keys.onReturnPressed: { root.picked(pop.hi); pop.close() }
                Keys.onEnterPressed: { root.picked(pop.hi); pop.close() }
                Keys.onSpacePressed: { root.picked(pop.hi); pop.close() }
                delegate: Corner {
                    required property int index
                    required property var modelData
                    objectName: root.objectName !== "" ? root.objectName + "-option-" + index : ""
                    width: list.width
                    height: pop.rowHeight
                    radius: theme.radiusInner(pop.inset)
                    smoothing: theme.cornerSmoothing
                    color: index === pop.hi ? theme.raisedHover : "transparent"
                    Label {
                        anchors.left: parent.left
                        anchors.leftMargin: theme.space(3)
                        anchors.right: parent.right
                        anchors.rightMargin: theme.space(3)
                        anchors.verticalCenter: parent.verticalCenter
                        text: String(modelData)
                        elide: Text.ElideRight
                        role: "ui"
                        color: index === root.index ? theme.accent : theme.text
                    }
                    MouseArea {
                        anchors.fill: parent
                        hoverEnabled: true
                        cursorShape: Qt.PointingHandCursor
                        onEntered: pop.hi = index
                        onClicked: { root.picked(index); pop.close() }
                    }
                }
            }
        }
    }
}
