// A room code: six boxes, one character each. Whatever is typed or pasted becomes the code,
// upper case, from the room alphabet only; a pasted room link gives its code. The boxes draw a
// one line input that fills them with its own text and caret transparent, so typing, paste,
// backspace and selection behave as in any field, and a screen reader finds a text field.
import QtQuick

Item {
    id: root
    // The code as typed so far, never more than `length` characters
    property alias text: input.text
    property int length: 6
    property bool invalid: false
    property string accessibleName: ""
    // All boxes filled, once per code: more keys on a full field send nothing again
    signal filled(string code)
    // Return or Enter, unless the code in the boxes already went out as `filled`
    signal submitted(string code)
    // The full code `filled` last sent, so neither signal sends it twice
    property string sent: ""

    // The core's room::code::ALPHABET and LEN (room/src/code.rs); change them there and here
    readonly property string alphabet: "ABCDEFGHJKLMNPQRSTUVWXYZ23456789"
    readonly property real gap: theme.space(2)
    readonly property real box: Math.min(theme.controlHeight * 1.25, (width - gap * (length - 1)) / length)

    implicitWidth: theme.controlHeight * 1.25 * length + gap * (length - 1)
    implicitHeight: box

    function focusInput() { input.forceActiveFocus() }

    // A link's code read the way the relay reads one (code_from_query in room/src/page.rs): no
    // query or fragment, the last path segment that is not empty. Then upper case, and nothing
    // outside the alphabet. A stray "/" typed after a few characters leaves them alone.
    function clean(raw) {
        var parts = String(raw).split(/[?#]/)[0].split("/").filter(function(p) { return p.trim() !== "" })
        var s = parts.length ? parts[parts.length - 1].toUpperCase() : ""
        var out = ""
        for (var i = 0; i < s.length && out.length < root.length; i++)
            if (root.alphabet.indexOf(s[i]) >= 0) out += s[i]
        return out
    }

    TextInput {
        id: input
        objectName: "code-input-text"
        // The size of the boxes and fully opaque, so the accessibility tree reports it as
        // showing; its text, caret and selection are transparent, since the boxes draw them
        anchors.fill: parent
        verticalAlignment: TextInput.AlignVCenter
        clip: true
        color: "transparent"
        selectionColor: "transparent"
        selectedTextColor: "transparent"
        cursorDelegate: Item {}
        Accessible.role: Accessible.EditableText
        Accessible.name: theme.spokenName(root.accessibleName, "", root, "")
        inputMethodHints: Qt.ImhUppercaseOnly | Qt.ImhNoPredictiveText | Qt.ImhNoAutoUppercase
        selectByMouse: false
        onTextChanged: {
            var c = root.clean(text)
            if (c !== text) { text = c; cursorPosition = c.length; return }
            if (c.length < root.length) root.sent = ""
            else if (c !== root.sent) { root.sent = c; root.filled(c) }
        }
        onAccepted: if (text !== root.sent) root.submitted(text)
        // The caret always sits at the end: boxes fill left to right
        onCursorPositionChanged: if (cursorPosition !== text.length) cursorPosition = text.length
    }

    MouseArea {
        anchors.fill: parent
        cursorShape: Qt.IBeamCursor
        onClicked: input.forceActiveFocus()
    }

    Row {
        spacing: root.gap
        Repeater {
            model: root.length
            Corner {
                id: cell
                required property int index
                readonly property bool current: input.activeFocus
                    && (index === input.text.length || (index === root.length - 1 && input.text.length === root.length))
                width: root.box
                height: root.box
                radius: Math.min(theme.radiusTile, height / 3)
                smoothing: theme.cornerSmoothing
                color: current ? theme.raised : theme.tile
                borderColor: root.invalid ? theme.red : current ? theme.focusRing : index < input.text.length ? theme.lineStrong : theme.line
                borderWidth: current ? 2 : 1

                Text {
                    anchors.centerIn: parent
                    text: input.text.charAt(cell.index)
                    color: theme.text
                    font.family: theme.fontFamily
                    font.pointSize: theme.typeLarge
                    font.weight: theme.roleWeight("ledger")
                    font.variableAxes: theme.roleAxes("ledger")
                    font.features: theme.roleFeatures("ledger")
                }
                // The caret, in the box the next character goes to
                Rectangle {
                    visible: cell.current && input.text.length < root.length
                    anchors.centerIn: parent
                    width: 2
                    height: parent.height * 0.42
                    radius: 1
                    color: theme.focusRing
                    SequentialAnimation on opacity {
                        running: cell.current
                        loops: Animation.Infinite
                        NumberAnimation { to: 0; duration: 530; easing.type: Easing.InOutQuad }
                        NumberAnimation { to: 1; duration: 530; easing.type: Easing.InOutQuad }
                    }
                }
            }
        }
    }
}
