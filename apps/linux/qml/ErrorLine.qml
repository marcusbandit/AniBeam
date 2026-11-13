// An error that stays on the page, the inline sibling of frame.fail (#220): what happened and
// what to do in plain words, the button that does it, and the core's own message under
// Details, opened in place and copied. Hidden while `text` is empty.
import QtQuick

Column {
    id: root
    // The words by message ID (#792), `msgArgs` filled in; `text` set directly wins
    property string msg: ""
    property var msgArgs: ({})
    property string text: msg ? words.t(msg, msgArgs) : ""
    // The core's own message, word for word; no Details when it says nothing more
    property string detail: ""
    property string actionText: ""
    property var action: null
    property bool detailOpen: false
    readonly property bool hasDetail: detail !== "" && detail !== text

    visible: text !== ""
    spacing: theme.space(1)
    onTextChanged: detailOpen = false

    Flow {
        width: parent.width
        spacing: theme.space(2)
        Label {
            id: words
            text: root.text
            role: "ui"
            font.pointSize: theme.typeSmall
            color: theme.red
            wrapMode: Text.Wrap
            width: Math.min(implicitWidth, root.width)
            height: Math.max(implicitHeight, theme.controlHeight * 0.8)
            verticalAlignment: Text.AlignVCenter
        }
        Button {
            objectName: "error-line-action"
            visible: root.actionText !== "" && !!root.action
            text: root.actionText
            small: true
            onClicked: { var a = root.action; if (a) a() }
        }
        Button {
            objectName: "error-line-details"
            visible: root.hasDetail
            text: root.detailOpen ? "Hide details" : "Details"
            small: true
            flat: true
            onClicked: {
                root.detailOpen = !root.detailOpen
                if (root.detailOpen) frame.copyQuiet(root.detail)
            }
            Tooltip { text: root.detailOpen ? "" : "Show what went wrong, word for word, and copy it" }
        }
    }
    Label {
        visible: root.detailOpen
        width: parent.width
        text: root.detail + "\nCopied."
        role: "ui"
        font.pointSize: theme.typeSmall
        color: theme.textDim
        wrapMode: Text.Wrap
    }
}
