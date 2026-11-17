// One library source: a folder glyph (crossed when the folder is missing), the path, its
// counts, then Open, Rescan and Remove. A missing folder carries an Unavailable chip whose
// tip says why and what fixes it; the row itself only states the counts. Remove asks first: the tab owns the decision to
// show the question (it sets `confirming` from `onRemove`), and the row answers back with
// `removeAccepted` once the question is answered yes. The path takes the room the glyph, the
// chip and the buttons leave and elides in the middle, so the chip stays in sight (#665).
// The buttons drop under the path only once it would get less than `minPath` beside them,
// the same threshold `SettingRow` uses for its label against its control.
import QtQuick
import com.marcusrosado.Anibeam

Item {
    id: root
    property var source: ({})
    // Panel draws a hairline under the row, except the group's last
    readonly property bool hairline: true
    property bool confirming: false
    signal open()
    signal rescan()
    signal remove()
    signal removeAccepted()

    readonly property bool available: source.available !== false
    readonly property string meta: (source.movie_folders || []).length
            ? words.t("source-row-counts-movies", { count: source.series_count || 0, movies: source.movie_folders.length })
            : words.t("source-row-counts", { count: source.series_count || 0 })
    readonly property real gutter: theme.space(6)
    readonly property real glyphPart: folderIcon.width + lead.spacing
    readonly property real chipPart: chip.visible ? pathLine.spacing + chip.width : 0
    readonly property real minPath: Math.min(pathText.implicitWidth, theme.space(60))
    readonly property bool stacked: width < glyphPart + Math.max(chipPart + minPath, metaText.implicitWidth) + controls.width + gutter
    // What the lead may span: the whole row when stacked, else the part left of the buttons
    readonly property real leadRoom: Math.max(0, (stacked ? width : width - controls.width - gutter) - glyphPart)
    readonly property real pathRoom: Math.max(0, leadRoom - chipPart)

    width: parent.width
    height: stacked ? theme.space(3) + lead.height + theme.space(2) + controls.height + theme.space(3)
                    : Math.max(theme.space(13), Math.max(lead.height, controls.height) + theme.space(3) * 2)

    Row {
        id: lead
        x: 0
        y: root.stacked ? theme.space(3) : Math.round((root.height - height) / 2)
        spacing: theme.space(3)
        Icon {
            id: folderIcon
            anchors.verticalCenter: parent.verticalCenter
            glyph: root.available ? "folder" : "folder-x"
            color: root.available ? theme.text : theme.textDim
        }
        Column {
            anchors.verticalCenter: parent.verticalCenter
            spacing: theme.space(0.5)
            Row {
                id: pathLine
                spacing: theme.space(2)
                Text {
                    id: pathText
                    anchors.verticalCenter: parent.verticalCenter
                    width: Math.min(implicitWidth, root.pathRoom)
                    elide: Text.ElideMiddle
                    text: root.source.path || ""
                    color: root.available ? theme.text : theme.textDim
                    font.family: theme.fontUi
                    font.pointSize: theme.typeNormal
                    Tooltip { text: parent.truncated ? parent.text : "" }
                }
                Chip {
                    id: chip
                    anchors.verticalCenter: parent.verticalCenter
                    visible: !root.available
                    msg: "source-row-unavailable"
                    small: true
                    mono: false
                    color: theme.tileSunken
                    textColor: theme.textDim
                    // Why, and the fix, when the pointer asks
                    Tooltip { msg: "source-row-unavailable.tip" }
                }
            }
            Text {
                id: metaText
                visible: root.meta !== ""
                width: Math.min(implicitWidth, root.leadRoom)
                elide: Text.ElideRight
                text: root.meta
                color: theme.textDim
                font.family: theme.fontSans
                font.pointSize: theme.typeSmall
            }
        }
    }
    Row {
        id: controls
        x: root.stacked ? 0 : root.width - width
        y: root.stacked ? lead.y + lead.height + theme.space(2) : Math.round((root.height - height) / 2)
        Row {
            visible: !root.confirming
            spacing: theme.space(1)
            Button { msg: "action-open"; icon: "folder-open"; flat: true; onClicked: root.open() }
            Button { msg: "source-row-rescan"; icon: "refresh-cw"; flat: true; onClicked: root.rescan() }
            Button { msg: "action-remove"; icon: "trash-2"; flat: true; onClicked: root.remove() }
        }
        InlineConfirm {
            visible: root.confirming
            question: words.t("source-row-remove-question", { name: Fmt.baseName(root.source.path || ""), count: root.source.series_count || 0 })
            onAccepted: root.removeAccepted()
            onKept: root.confirming = false
        }
    }
}
