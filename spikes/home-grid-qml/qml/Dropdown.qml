// A pill showing the current value and a chevron; opens a list under it. Up and Down move
// the highlight, Return picks, Escape closes. Picking emits `picked`; the owner sets `index`.
import QtQuick
import QtQuick.Controls.Basic as QC

Corner {
    id: root
    property var options: []
    property int index: 0
    property real minWidth: theme.space(40)
    signal picked(int i)

    readonly property string current: options.length > index && index >= 0 ? String(options[index]) : ""
    implicitWidth: Math.max(minWidth, label.implicitWidth + chevron.width + theme.space(4) * 2 + theme.space(2))
    implicitHeight: theme.controlHeight
    radius: height / 2
    smoothing: theme.cornerSmoothing
    color: theme.surfaceSunken
    borderColor: activeFocus || pop.opened ? theme.focusRing : (hover.containsMouse ? theme.lineStrong : theme.line)
    borderWidth: activeFocus || pop.opened ? theme.space(0.5) : 1
    activeFocusOnTab: true

    function open() { pop.hi = index; pop.open() }
    Keys.onSpacePressed: open()
    Keys.onReturnPressed: open()
    Keys.onDownPressed: open()

    Text {
        id: label
        anchors.left: parent.left
        anchors.leftMargin: theme.space(4)
        anchors.right: chevron.left
        anchors.rightMargin: theme.space(2)
        anchors.verticalCenter: parent.verticalCenter
        text: root.current
        color: theme.text
        elide: Text.ElideRight
        font.family: theme.fontSans
        font.pointSize: theme.typeNormal
    }
    // The chevron is a single guillemet turned a quarter, so it comes from the same face
    Text {
        id: chevron
        anchors.right: parent.right
        anchors.rightMargin: theme.space(4)
        anchors.verticalCenter: parent.verticalCenter
        text: "›"
        rotation: 90
        color: theme.textDim
        font.family: theme.fontSans
        font.pointSize: theme.typeNormal
        font.weight: Font.Bold
    }
    MouseArea {
        id: hover
        anchors.fill: parent
        hoverEnabled: true
        cursorShape: Qt.PointingHandCursor
        onClicked: { root.forceActiveFocus(); pop.opened ? pop.close() : root.open() }
    }

    QC.Popup {
        id: pop
        property int hi: 0
        readonly property real rowHeight: theme.controlHeight
        y: root.height + theme.space(1)
        width: root.width
        height: Math.min(root.options.length, 8) * rowHeight + theme.space(1) * 2
        padding: 0
        focus: true
        background: null
        closePolicy: QC.Popup.CloseOnEscape | QC.Popup.CloseOnPressOutside
        onOpened: list.forceActiveFocus()
        contentItem: Corner {
            radius: theme.radiusMd
            smoothing: theme.cornerSmoothing
            color: theme.surfaceRaised
            borderColor: theme.lineStrong
            borderWidth: 1
            ListView {
                id: list
                anchors.fill: parent
                anchors.margins: theme.space(1)
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
                    width: list.width
                    height: pop.rowHeight
                    radius: theme.radiusSm
                    smoothing: theme.cornerSmoothing
                    color: index === pop.hi ? theme.surfacePressed : "transparent"
                    Text {
                        anchors.left: parent.left
                        anchors.leftMargin: theme.space(3)
                        anchors.right: parent.right
                        anchors.rightMargin: theme.space(3)
                        anchors.verticalCenter: parent.verticalCenter
                        text: String(modelData)
                        elide: Text.ElideRight
                        color: index === root.index ? theme.accent : theme.text
                        font.family: theme.fontSans
                        font.pointSize: theme.typeNormal
                        font.weight: index === root.index ? Font.DemiBold : Font.Normal
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
