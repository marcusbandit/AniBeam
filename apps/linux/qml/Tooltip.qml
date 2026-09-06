// Hover text for one item. The tip itself is drawn by the frame in its overlay after a
// 600 ms hover intent, so it is never clipped by the item's own parent.
import QtQuick

MouseArea {
    id: root
    property string text: ""
    anchors.fill: parent
    hoverEnabled: true
    acceptedButtons: Qt.NoButton
    propagateComposedEvents: true
    onEntered: if (text !== "") intent.start()
    onExited: { intent.stop(); frame.hideTip() }
    onTextChanged: if (containsMouse && text === "") frame.hideTip()
    Timer { id: intent; interval: 600; onTriggered: frame.showTip(root, root.text) }
}
