// A Button that starts a tracker's sign-in, standing disabled while the other tracker's
// browser is out: the core runs one sign-in at a time, so starting this one would cancel
// that one mid-flow. The reason is the tip, which lives on this wrapper rather than on the
// Button because a disabled Button's own hover area is disabled with it.
import QtQuick
import com.marcusrosado.AniBeam

Item {
    id: root
    property string tracker: "Anilist"
    property alias text: button.text
    property alias icon: button.icon
    property alias primary: button.primary
    property alias flat: button.flat
    property alias small: button.small
    signal clicked()

    readonly property bool blocked: frame.connectBlockedBy(tracker) !== ""
    implicitWidth: button.implicitWidth
    implicitHeight: button.implicitHeight
    width: implicitWidth
    height: implicitHeight

    Button {
        id: button
        objectName: root.objectName !== "" ? root.objectName + "-button" : ""
        anchors.fill: parent
        enabled: !root.blocked
        onClicked: root.clicked()
    }
    Tooltip { text: frame.blockedTip(root.tracker) }
}
