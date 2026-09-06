// One library source: a folder glyph (crossed when the folder is missing), the path, its
// counts, then Open, Rescan and Remove. Remove asks first: the tab owns the decision to
// show the question (it sets `confirming` from `onRemove`), and the row answers back with
// `removeAccepted` once the question is answered yes. When the row is too narrow for the
// path and the three buttons side by side (a stacked settings tab at a narrow window), the
// buttons drop under the path instead of overlapping it, the same rule `SettingRow` uses
// for its own label-versus-control width.
import QtQuick
import com.marcusrosado.AniBeam

Item {
    id: root
    property var source: ({})
    property bool confirming: false
    signal open()
    signal rescan()
    signal remove()
    signal removeAccepted()

    readonly property bool available: source.available !== false
    readonly property string meta: Fmt.plural(source.series_count || 0, "series", "series")
        + ((source.movie_folders || []).length ? " · " + Fmt.plural(source.movie_folders.length, "movie folder", "movie folders") : "")
    readonly property real gutter: theme.space(6)
    readonly property bool stacked: width < lead.width + controls.width + gutter + theme.space(4)

    width: parent.width
    height: stacked ? theme.space(3) + lead.height + theme.space(2) + controls.height + theme.space(3)
                    : Math.max(theme.space(13), Math.max(lead.height, controls.height) + theme.space(3) * 2)

    Row {
        id: lead
        x: theme.space(4)
        y: root.stacked ? theme.space(3) : Math.round((root.height - height) / 2)
        spacing: theme.space(3)
        Icon {
            anchors.verticalCenter: parent.verticalCenter
            glyph: root.available ? "folder" : "folder-x"
            color: root.available ? theme.text : theme.textDim
        }
        Column {
            anchors.verticalCenter: parent.verticalCenter
            spacing: theme.space(0.5)
            Row {
                spacing: theme.space(2)
                Text {
                    anchors.verticalCenter: parent.verticalCenter
                    text: root.source.path || ""
                    color: root.available ? theme.text : theme.textDim
                    font.family: theme.fontMono
                    font.pointSize: theme.typeNormal
                }
                Chip {
                    anchors.verticalCenter: parent.verticalCenter
                    visible: !root.available
                    text: "Unavailable"
                    small: true
                    mono: false
                    color: theme.surface
                    textColor: theme.textDim
                }
            }
            Text {
                visible: root.meta !== ""
                text: root.meta
                color: theme.textDim
                font.family: theme.fontSans
                font.pointSize: theme.typeSmall
            }
        }
    }
    Row {
        id: controls
        x: root.stacked ? theme.space(4) : root.width - width - theme.space(3)
        y: root.stacked ? lead.y + lead.height + theme.space(2) : Math.round((root.height - height) / 2)
        Row {
            visible: !root.confirming
            spacing: theme.space(1)
            Button { text: "Open"; icon: "folder-open"; flat: true; onClicked: root.open() }
            Button { text: "Rescan"; icon: "refresh-cw"; flat: true; onClicked: root.rescan() }
            Button { text: "Remove"; icon: "trash-2"; flat: true; onClicked: root.remove() }
        }
        InlineConfirm {
            visible: root.confirming
            question: "Remove " + (root.source.path || "").split("/").pop() + ", " + Fmt.plural(root.source.series_count || 0, "series", "series") + " and their history?"
            onAccepted: root.removeAccepted()
            onKept: root.confirming = false
        }
    }
}
