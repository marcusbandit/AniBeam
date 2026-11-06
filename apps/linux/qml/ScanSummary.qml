// The end of a new folder's first scan (#206): what it found, in a sentence and three
// counts, the episodes and how many the tracker already has as watched, and one button
// that walks through the series that need a match, one at a time in the match panel. The
// frame opens it once the scan and the lookup after it have settled; Escape, Later, a
// press outside or a move to another page close it.
import QtQuick
import com.marcusrosado.AniBeam

Item {
    id: root
    objectName: "scan-summary"
    anchors.fill: parent
    visible: false

    property string folder: ""
    property var summary: ({ shows: 0, recognised: 0, need_you: [], episodes: 0, watched: 0 })
    readonly property int needCount: summary.need_you ? summary.need_you.length : 0
    // The tracker the watched count comes from, by name; empty with none connected. Not
    // readonly, so a drive can draw the line without signing in
    property string trackerName: Door.trackers && Door.trackers.anilist && Door.trackers.anilist.connected === true ? "AniList"
        : Door.trackers && Door.trackers.mal && Door.trackers.mal.connected === true ? "MyAnimeList" : ""

    readonly property string headline: {
        var s = summary
        if (s.shows === 0) return "Nothing to show yet"
        var line = "Found " + Fmt.plural(s.shows, "show", "shows") + ", " + s.recognised + " recognised"
        return root.needCount > 0 ? line + ", " + root.needCount + " need you" : line
    }
    readonly property string episodesLine: {
        var s = summary
        var line = Fmt.plural(s.episodes, "episode", "episodes") + " on disk"
        if (root.trackerName !== "") line += ", " + s.watched + " already watched on " + root.trackerName
        return line + "."
    }

    function open(folder, summary) {
        root.folder = folder
        root.summary = summary
        if (visible) return
        visible = true
        frame.escapeStack.push("popover", root)
        panel.forceActiveFocus()
    }
    function close() {
        if (!visible) return
        frame.escapeStack.pop(root)
        visible = false
        frame.focusPage(root)
    }
    function walk() {
        var ids = root.summary.need_you.map(function(n) { return n.id })
        root.close()
        frame.go("metadata", { walk: ids })
    }

    Blocker { anchors.fill: parent; dismissOnPress: true; onDismissed: root.close() }

    Tile {
        id: panel
        objectName: "scan-summary-panel"
        color: theme.glassFloat
        borderColor: theme.lineStrong
        padding: theme.space(7)
        anchors.centerIn: parent
        width: Math.min(parent.width - theme.space(16), theme.space(150))
        height: body.implicitHeight + padding * 2
        MouseArea { anchors.fill: parent; anchors.margins: -panel.padding; acceptedButtons: Qt.AllButtons }
        Keys.onReturnPressed: root.needCount > 0 ? root.walk() : root.close()
        Keys.onEnterPressed: root.needCount > 0 ? root.walk() : root.close()

        Column {
            id: body
            width: parent.width
            spacing: theme.space(5)
            Item {
                width: parent.width
                height: title.height + where.height + theme.space(1)
                Label { id: title; text: "Your folder is in"; role: "display"; font.pointSize: theme.typeHeading }
                Label {
                    id: where
                    anchors.top: title.bottom; anchors.topMargin: theme.space(1)
                    width: parent.width - theme.space(10)
                    text: root.folder
                    role: "ui"; color: theme.textDim; elide: Text.ElideMiddle
                }
                Icon {
                    glyph: "x"
                    size: theme.space(4.5)
                    color: closeMouse.containsMouse ? theme.text : theme.textDim
                    anchors.right: parent.right; anchors.top: parent.top
                    MouseArea { id: closeMouse; anchors.fill: parent; anchors.margins: -theme.space(1.5); hoverEnabled: true; cursorShape: Qt.PointingHandCursor; onClicked: root.close(); Accessible.role: Accessible.Button; Accessible.name: "Close"; Accessible.onPressAction: root.close() }
                }
            }
            Label {
                objectName: "scan-summary-headline"
                width: parent.width
                text: root.headline
                role: "title"
                font.pointSize: theme.typeLarge * 1.2
                wrapMode: Text.Wrap
            }
            // The three counts, the one that needs the user lit
            Row {
                width: parent.width
                spacing: theme.grout
                readonly property real cell: (width - spacing * 2) / 3
                Repeater {
                    model: [
                        { n: root.summary.shows, label: "found", icon: "folder-open", lit: false },
                        { n: root.summary.recognised, label: "recognised", icon: "circle-check", lit: false },
                        { n: root.needCount, label: "need you", icon: "search", lit: root.needCount > 0 }
                    ]
                    Tile {
                        required property var modelData
                        width: parent.cell
                        height: theme.space(24)
                        raised: modelData.lit
                        borderColor: modelData.lit ? theme.accentEdge : theme.line
                        Icon { anchors.right: parent.right; anchors.top: parent.top; glyph: modelData.icon; color: modelData.lit ? theme.accent : theme.textDim }
                        Column {
                            anchors.left: parent.left; anchors.bottom: parent.bottom
                            Label { text: String(modelData.n); role: "display"; font.pointSize: theme.typeHeading }
                            Label { text: modelData.label; color: theme.textDim }
                        }
                    }
                }
            }
            Label {
                objectName: "scan-summary-episodes"
                width: parent.width
                text: root.episodesLine
                color: theme.textDim
                wrapMode: Text.Wrap
            }
            Item {
                width: parent.width
                height: actions.height
                Row {
                    id: actions
                    anchors.right: parent.right
                    spacing: theme.space(3)
                    Button {
                        objectName: "scan-summary-later"
                        visible: root.needCount > 0
                        text: "Later"; flat: true
                        onClicked: root.close()
                    }
                    Button {
                        objectName: "scan-summary-go"
                        primary: true
                        text: root.needCount > 0 ? "Match the " + root.needCount : "Open the library"
                        icon: "arrow-right"
                        onClicked: if (root.needCount > 0) root.walk(); else { root.close(); frame.go("library") }
                    }
                }
            }
        }
    }
}
