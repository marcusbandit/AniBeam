// What's new: the desktop's curated CHANGELOG.md, newest version first. Each release opens
// with a short note on what's new, followed by a concise list of highlights and a brief
// roundup of smaller fixes. A divider marks how far is new since the previous version, and
// older entries read dimmer. The frame opens this once after an update; the version in the
// rail and Settings > About open it any time. Got it, Return, Escape, clicking outside or
// switching pages closes it and marks the notes as read.
//
// The two versions that takes live in shell.conf. Every rule about them, where the line
// goes and what reads dimmer included, is the core's (changelog), reached through
// Shell.whatsNew; this only draws the answer.
import QtQuick
import QtQuick.Controls.Basic as QC
import QtCore
import com.marcusrosado.Anibeam

Item {
    id: root
    objectName: "whats-new"
    anchors.fill: parent
    visible: false

    Settings {
        id: kept
        location: Shell.settingsUrl
        category: "whatsNew"
        // The newest version whose notes were closed, and the version before it
        property string version: ""
        property string from: ""
    }

    property var view: ({ note: "", line: "", above: 0, versions: [] })

    function ask() { return Shell.whatsNew(kept.version, kept.from) }
    // A change's dot sits on its first line, however many lines it wraps to
    Label { id: body; visible: false; text: "x" }
    FontMetrics { id: bodyMetrics; font: body.font }
    // As the app starts: a fresh install keeps its own version as read, and the first start
    // after an update opens the notes.
    function start() {
        var s = ask()
        if (s.started) { kept.version = s.started.version; kept.from = s.started.from }
        else if (s.due) open()
    }
    function open() {
        view = ask().view
        list.contentY = 0
        if (visible) return
        visible = true
        frame.escapeStack.push("popover", root)
        panel.forceActiveFocus()
    }
    function close() {
        if (!visible) return
        var s = ask()
        kept.version = s.closed.version
        kept.from = s.closed.from
        frame.escapeStack.pop(root)
        visible = false
        frame.focusPage(root)
    }

    Blocker { anchors.fill: parent; dismissOnPress: true; onDismissed: root.close() }

    Tile {
        id: panel
        objectName: "whats-new-panel"
        color: theme.glassFloat
        borderColor: theme.lineStrong
        padding: theme.space(7)
        anchors.centerIn: parent
        width: Math.min(parent.width - theme.space(16), theme.space(150))
        height: Math.min(parent.height - theme.space(16), head.height + notes.implicitHeight + foot.height + theme.space(8) + padding * 2)
        // The Blocker dismisses on any press; one on the panel's padding must not reach it.
        MouseArea { anchors.fill: parent; anchors.margins: -panel.padding; acceptedButtons: Qt.AllButtons }

        // The keys scroll through the wheel's target, so they glide as a notch does
        Keys.onPressed: function(event) {
            // A Mac's arrows, and the keypad's, come with the keypad flag
            if ((event.modifiers & ~Qt.KeypadModifier) !== Qt.NoModifier) return
            var k = event.key, page = list.height - theme.space(10)
            var to = k === Qt.Key_Down ? wheel.target + wheel.notch
                   : k === Qt.Key_Up ? wheel.target - wheel.notch
                   : k === Qt.Key_PageDown || k === Qt.Key_Space ? wheel.target + page
                   : k === Qt.Key_PageUp ? wheel.target - page
                   : k === Qt.Key_Home ? 0
                   : k === Qt.Key_End ? list.contentHeight
                   : undefined
            if (to === undefined) return
            wheel.scrollTo(to)
            event.accepted = true
        }
        Keys.onReturnPressed: root.close()
        Keys.onEnterPressed: root.close()

        // The head: the title, where this install came from and where it is now
        Item {
            id: head
            width: parent.width
            height: title.height + note.height + theme.space(1)
            Label {
                id: title
                msg: "whats-new-title"
                role: "display"
                font.pointSize: theme.typeHeading
            }
            Label {
                id: note
                objectName: "whats-new-note"
                anchors.top: title.bottom
                anchors.topMargin: theme.space(1)
                text: root.view.note
                role: "ui"
                color: theme.textDim
            }
            Icon {
                glyph: "x"
                size: theme.space(4.5)
                color: closeMouse.containsMouse ? theme.text : theme.textDim
                anchors.right: parent.right; anchors.top: parent.top
                MouseArea { id: closeMouse; anchors.fill: parent; anchors.margins: -theme.space(1.5); hoverEnabled: true; cursorShape: Qt.PointingHandCursor; onClicked: root.close(); Accessible.role: Accessible.Button; Accessible.name: words.t("action-close")
                    Accessible.onPressAction: root.close() }
            }
        }

        Flickable {
            id: list
            anchors.top: head.bottom; anchors.topMargin: theme.space(4)
            anchors.bottom: foot.top; anchors.bottomMargin: theme.space(4)
            width: parent.width
            clip: true
            contentWidth: width
            contentHeight: notes.implicitHeight
            boundsBehavior: Flickable.StopAtBounds
            QC.ScrollBar.vertical: ThinScrollBar {}
            WheelStep { id: wheel; flick: list }

            Column {
                id: notes
                width: list.width - theme.space(4)
                Repeater {
                    model: root.view.versions
                    Column {
                        id: release
                        required property var modelData
                        required property int index
                        readonly property bool dim: !modelData.lit
                        width: notes.width
                        topPadding: index === 0 ? 0 : theme.space(9)
                        spacing: theme.space(1.5)

                        Row {
                            spacing: theme.space(3)
                            Label {
                                id: number
                                text: release.modelData.version
                                role: "display"
                                font.pointSize: theme.typeLarge * 1.25
                                color: release.dim ? theme.textDim : theme.text
                            }
                            // A version's own name, when its heading gives one
                            Label {
                                anchors.baseline: number.baseline
                                visible: text !== ""
                                text: release.modelData.title || ""
                                role: "display"
                                font.pointSize: theme.typeLarge
                                color: release.dim ? theme.textFaint : theme.textDim
                            }
                            Label {
                                anchors.baseline: number.baseline
                                text: release.modelData.day
                                role: "ledger"
                                color: theme.textFaint
                            }
                        }
                        Repeater {
                            model: release.modelData.blocks
                            Column {
                                id: block
                                required property var modelData
                                required property int index
                                width: release.width

                                Label {
                                    visible: block.modelData.kind === "paragraph"
                                    width: block.width
                                    topPadding: block.index === 0 ? theme.space(2) : theme.space(1.5)
                                    text: block.modelData.kind === "paragraph" ? block.modelData.rich : ""
                                    textFormat: Text.StyledText
                                    wrapMode: Text.Wrap
                                    lineHeight: 1.35
                                    color: release.dim ? theme.textDim : theme.text
                                }

                                Label {
                                    visible: block.modelData.kind === "heading"
                                    width: block.width
                                    topPadding: theme.space(3.5)
                                    bottomPadding: theme.space(0.5)
                                    text: block.modelData.kind === "heading" ? block.modelData.text : ""
                                    role: "title"
                                    font.pointSize: theme.typeSmall
                                    font.letterSpacing: 0.4
                                    color: release.dim ? theme.textFaint : theme.textDim
                                }

                                Column {
                                    visible: block.modelData.kind === "list"
                                    width: block.width
                                    topPadding: theme.space(1.5)
                                    spacing: theme.space(2)
                                    Repeater {
                                        model: block.modelData.kind === "list" ? block.modelData.items : []
                                        Item {
                                            id: change
                                            required property var modelData
                                            width: block.width
                                            height: changeWords.implicitHeight
                                            Rectangle {
                                                y: Math.round((bodyMetrics.height - height) / 2)
                                                width: theme.space(1.25); height: width; radius: width / 2
                                                color: release.dim ? theme.textFaint : theme.textDim
                                            }
                                            Label {
                                                id: changeWords
                                                x: theme.space(4)
                                                width: parent.width - x
                                                text: change.modelData.rich
                                                textFormat: Text.StyledText
                                                wrapMode: Text.Wrap
                                                lineHeight: 1.32
                                                color: release.dim ? theme.textDim : theme.text
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        // The line, under the last version that is new: everything above it
                        // came after the version this install had.
                        Item {
                            visible: root.view.line !== "" && release.index === root.view.above - 1
                            objectName: "whats-new-line"
                            width: release.width
                            height: visible ? pill.height + theme.space(9) : 0
                            Rectangle { anchors.left: parent.left; anchors.right: pill.left; anchors.rightMargin: theme.space(3); anchors.verticalCenter: pill.verticalCenter; height: 2; radius: 1; color: theme.accent }
                            Rectangle {
                                id: pill
                                anchors.horizontalCenter: parent.horizontalCenter
                                anchors.bottom: parent.bottom
                                width: lineWords.implicitWidth + theme.space(6)
                                height: theme.space(7)
                                radius: height / 2
                                color: theme.accent
                                Label { id: lineWords; anchors.centerIn: parent; text: root.view.line; role: "ui"; font.weight: Font.DemiBold; color: theme.accentText }
                            }
                            Rectangle { anchors.left: pill.right; anchors.leftMargin: theme.space(3); anchors.right: parent.right; anchors.verticalCenter: pill.verticalCenter; height: 2; radius: 1; color: theme.accent }
                        }
                    }
                }
            }
        }

        // The foot: link to the full list on GitHub, and Got it
        Item {
            id: foot
            anchors.bottom: parent.bottom
            width: parent.width
            height: gotIt.height
            Button {
                objectName: "whats-new-github"
                anchors.left: parent.left
                anchors.verticalCenter: parent.verticalCenter
                msg: "whats-new-github"
                icon: "external-link"
                flat: true
                onClicked: frame.openLink("https://github.com/marcusbandit/Anibeam/releases") // i18n: none
            }
            Button {
                id: gotIt
                objectName: "whats-new-got-it"
                anchors.right: parent.right
                msg: "whats-new-got-it"
                icon: "thumbs-up"
                primary: true
                onClicked: root.close()
            }
        }
    }
}
