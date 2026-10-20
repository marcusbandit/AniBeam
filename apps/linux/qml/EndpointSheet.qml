// Adding search endpoints, the one way a fresh install gets any: paste a site's search or RSS
// URL, or import an endpoint pack someone wrote out with Export pack. The core works out from
// a URL's shape where the search words go, so a person pastes what their browser shows. A
// pack is read here to show what it holds before anything is added; the core checks it again
// and adds its endpoints after the ones already there, a name in use replaced in place.
//
// openWith("url") or openWith("pack"). It runs AddEndpointUrl or ImportEndpointPack itself and
// closes when the job finishes; a failure stays on the sheet as a line.
import QtQuick
import QtQuick.Dialogs
import com.marcusrosado.AniBeam

Item {
    id: root
    anchors.fill: parent
    visible: false
    property int mode: 0            // 0 a URL, 1 a pack
    property string error: ""
    property real job: -1
    readonly property bool busy: job !== -1
    property string packText: ""
    property var pack: null          // the parsed pack, for the preview
    property Item prevFocus: null

    function openWith(which) {
        root.mode = which === "pack" ? 1 : 0
        root.error = ""
        root.job = -1
        urlField.text = ""
        nameField.text = ""
        root.packText = ""
        root.pack = null
        root.prevFocus = frame.hostWindow ? frame.hostWindow.activeFocusItem : null
        root.visible = true
        frame.escapeStack.push("sheet", root)
        if (root.mode === 0) urlField.focusInput(); else panel.forceActiveFocus()
    }
    function close() {
        if (!root.visible) return
        root.visible = false
        frame.escapeStack.pop(root)
        frame.focusPage(root, root.prevFocus)
        root.prevFocus = null
    }
    Component.onDestruction: frame.escapeStack.pop(root)

    // A pack's text into the preview, or a line saying why it is not one
    function takePack(text) {
        root.packText = text
        root.error = ""
        root.pack = null
        try {
            var p = JSON.parse(text)
            if (!p || p.format !== "anibeam-endpoints" || !Array.isArray(p.endpoints)) throw new Error("not an endpoint pack")
            root.pack = p
        } catch (e) {
            root.error = "That is not an endpoint pack: it should be the .json file Export pack writes."
        }
    }
    function pasteText() {
        clip.text = ""
        clip.paste()
        var t = clip.text.trim()
        if (root.mode === 0) { urlField.text = t; return }
        if (t === "") { root.error = "The clipboard is empty."; return }
        takePack(t)
    }
    function submit() {
        if (root.busy) return
        root.error = ""
        var r
        if (root.mode === 0) {
            var url = urlField.text.trim()
            if (url === "") { root.error = "Paste a URL first."; return }
            r = Door.call("AddEndpointUrl", { url: url, name: nameField.text.trim() })
        } else {
            if (!root.pack) { root.error = "Choose a pack file or paste one first."; return }
            r = Door.call("ImportEndpointPack", { text: root.packText })
        }
        if (r.error) { root.error = r.error.message; return }
        root.job = r.reply.job
    }
    readonly property string bestFitLine: {
        var b = pack && pack.best_fit
        if (!b || !b.preferred_groups || b.preferred_groups.length === 0) return ""
        return "It also ranks release groups: " + b.preferred_groups.join(", ") + ". The ranking stays in the file for now; best fit keeps anirss's own."
    }

    Connections {
        target: Door
        function onJobFinished(job, kind, ok) {
            if (job !== root.job || !ok) return
            root.job = -1
            frame.toast(root.mode === 0 ? "Endpoint added" : "Endpoints imported")
            root.close()
        }
        function onJobFailed(job, kind, error) { if (job === root.job) { root.job = -1; root.error = error.message } }
        function onJobCancelled(job, kind) { if (job === root.job) root.job = -1 }
    }

    TextEdit { id: clip; visible: false }
    FileDialog {
        id: packDialog
        nameFilters: ["Endpoint pack (*.json)", "All files (*)"]
        onAccepted: {
            var r = Door.readTextFile(decodeURIComponent(String(selectedFile).replace("file://", "")))
            if (r.error) { root.error = r.error.message; return }
            root.takePack(r.reply.text)
        }
    }

    // The window dims behind it; a press outside closes it
    Rectangle { anchors.fill: parent; color: "#000000"; opacity: 0.35 }
    MouseArea { anchors.fill: parent; acceptedButtons: Qt.AllButtons; onPressed: root.close(); onWheel: function(w) { w.accepted = true } }

    Frost {
        id: panel
        objectName: "endpoint-sheet"
        width: Math.min(theme.space(140), root.width - theme.space(8))
        height: body.implicitHeight + theme.space(6) * 2
        x: (root.width - width) / 2
        y: Math.max(theme.space(4), (root.height - height) / 2)
        radius: theme.radiusTile
        focus: true
        Keys.onReturnPressed: root.submit()
        Keys.onEnterPressed: root.submit()
        Keys.onEscapePressed: root.close()
        MouseArea { anchors.fill: parent }

        Column {
            id: body
            x: theme.space(6)
            y: theme.space(6)
            width: parent.width - theme.space(12)
            spacing: theme.space(4)
            Label { width: parent.width; text: "Add search endpoints"; role: "title"; font.pointSize: theme.typeNormal * 1.15 }
            Seg {
                objectName: "endpoint-sheet-mode"
                options: [{ text: "Search URL", icon: "link" }, { text: "Endpoint pack", icon: "package" }]
                index: root.mode
                onPicked: function(i) { root.mode = i; root.error = "" }
            }

            // ---- One URL
            Column {
                visible: root.mode === 0
                width: parent.width
                spacing: theme.space(3)
                Label {
                    width: parent.width
                    text: "Paste the RSS or search address of a site you use, the way your browser shows it. AniBeam works out where the search words go. A personal token in the address stays in it."
                    role: "ui"; color: theme.textDim; wrapMode: Text.Wrap
                }
                Row {
                    width: parent.width
                    spacing: theme.space(2)
                    Field { id: urlField; objectName: "endpoint-sheet-url"; width: parent.width - pasteUrl.width - parent.spacing; mono: true; placeholder: "https://" }
                    Button { id: pasteUrl; text: "Paste"; icon: "clipboard-paste"; flat: true; onClicked: root.pasteText() }
                }
                Field { id: nameField; objectName: "endpoint-sheet-name"; width: theme.space(50); placeholder: "Name (optional)" }
            }

            // ---- A pack
            Column {
                visible: root.mode === 1
                width: parent.width
                spacing: theme.space(3)
                Label {
                    width: parent.width
                    text: "An endpoint pack is the .json file Export pack writes in Settings. Its endpoints go after the ones you have; one with a name you already use replaces it."
                    role: "ui"; color: theme.textDim; wrapMode: Text.Wrap
                }
                Row {
                    spacing: theme.space(2)
                    Button { objectName: "endpoint-sheet-choose"; text: "Choose file"; icon: "folder-open"; onClicked: packDialog.open() }
                    Button { text: "Paste"; icon: "clipboard-paste"; flat: true; onClicked: root.pasteText() }
                }
                Column {
                    visible: !!root.pack
                    width: parent.width
                    spacing: theme.space(1.5)
                    Label { width: parent.width; text: root.pack ? (root.pack.name || "Unnamed pack") : ""; role: "ui"; font.weight: Font.DemiBold; elide: Text.ElideRight }
                    Repeater {
                        model: root.pack ? root.pack.endpoints : []
                        Label {
                            required property var modelData
                            required property int index
                            width: body.width
                            text: (index + 1) + ".  " + modelData.name + "  " + modelData.kind + "  " + modelData.url
                            role: "ledger"; color: theme.textDim; elide: Text.ElideRight
                        }
                    }
                    Label { visible: root.bestFitLine !== ""; width: parent.width; text: root.bestFitLine; role: "ui"; font.pointSize: theme.typeSmall; color: theme.textFaint; wrapMode: Text.Wrap }
                }
            }

            Label { width: parent.width; visible: root.error !== ""; text: root.error; role: "ui"; color: theme.red; wrapMode: Text.Wrap }
            Item {
                width: parent.width
                height: theme.controlHeight
                Row {
                    anchors.right: parent.right
                    spacing: theme.space(2)
                    Button { text: "Cancel"; flat: true; onClicked: root.close() }
                    Button {
                        objectName: "endpoint-sheet-add"
                        text: root.busy ? "Adding" : root.mode === 0 ? "Add" : root.pack ? "Import " + root.pack.endpoints.length + (root.pack.endpoints.length === 1 ? " endpoint" : " endpoints") : "Import"
                        icon: "plus"
                        primary: true
                        enabled: !root.busy && (root.mode === 0 || !!root.pack)
                        onClicked: root.submit()
                    }
                }
            }
        }
    }
}
