// An input shield for a layer over the page. It takes every press (all buttons), wheel and
// hover that lands on its box, so nothing beneath sees them. `scrim` dims what is under it;
// `dismissOnPress` makes any press on the shield emit dismissed(). A panel that sits on top
// of a shield needs its own AllButtons MouseArea, or a press on the panel's padding reaches
// the shield and dismisses it.
import QtQuick

Item {
    id: root
    property bool scrim: true
    property bool dismissOnPress: false
    signal dismissed()

    Rectangle {
        anchors.fill: parent
        visible: root.scrim
        color: theme.scrim
        opacity: 0.6
    }
    MouseArea {
        anchors.fill: parent
        acceptedButtons: Qt.AllButtons
        // hoverEnabled is what keeps hover off the page below: a hovering MouseArea on top
        // takes the hover from every sibling under it, tooltips included.
        hoverEnabled: true
        cursorShape: root.dismissOnPress ? Qt.PointingHandCursor : Qt.ArrowCursor
        onPressed: if (root.dismissOnPress) root.dismissed()
        // Accepted explicitly: the page below must not scroll behind a layer.
        onWheel: function(w) { w.accepted = true }
    }
}
