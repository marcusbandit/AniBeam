// The Data tab: export and import of the library (the anibeam-export format), the image
// cache with its size and a clear, and where AniBeam keeps its files. Export, import and the
// clear are jobs; their results come back as exportFinished, importFinished and
// imagesCleared, and each lands in its own tile.
import QtQuick
import QtQuick.Dialogs
import com.marcusrosado.AniBeam

SettingsTab {
    id: tab
    property bool includePrivate: false
    property string lastExport: ""
    property var importSummary: null
    property var storage: ({ image_count: 0, image_bytes: 0 })
    property bool confirmingClear: false
    readonly property var about: Door.about || ({})
    function loadStorage() { var r = Door.getStorage(); if (!r.error) storage = r.reply }
    function path(url) { return decodeURIComponent(String(url).replace("file://", "")) }
    function dir(p) { var i = String(p || "").lastIndexOf("/"); return i > 0 ? p.slice(0, i) : p }
    Component.onCompleted: loadStorage()
    Connections {
        target: Door
        function onExportFinished(p) { tab.lastExport = p; frame.toast("Exported to " + p) }
        function onImportFinished(summary) { tab.importSummary = summary; frame.toast("Import finished") }
        function onImagesCleared(removed) { tab.loadStorage(); frame.toast("Removed " + Fmt.plural(removed, "image", "images")) }
        function onJobFailed(job, kind, error) { if (kind === "Export" || kind === "Import" || kind === "ClearImages") frame.toast(error && error.message ? error.message : "That did not finish") }
    }
    function summaryLines(s) {
        if (!s) return []
        var out = [
            [Fmt.plural(s.sources_added, "folder", "folders") + " added", s.sources_unavailable > 0 ? s.sources_unavailable + " not found on this computer" : ""],
            [Fmt.plural(s.series_created, "series", "series") + " created", Fmt.plural(s.matches_applied, "match", "matches") + " applied"],
            [Fmt.plural(s.views_merged, "view", "views") + " merged", Fmt.plural(s.completed_merged, "completed episode", "completed episodes") + " merged"],
            [Fmt.plural(s.resume_points_merged, "resume point", "resume points") + " merged", Fmt.plural(s.accounts_imported, "tracker account", "tracker accounts") + " imported"]
        ]
        return out.map(function(p) { return p.filter(Boolean).join(", ") })
    }

    SettingsPair {
        split: 1 / 2
        leftPanels: [ exportPanel, importPanel ]
        rightPanels: [ storagePanel, aboutPanel ]
    }
    Component {
        id: exportPanel
        Panel {
            title: "Export"; icon: "file-up"
            helper: "Writes your library to one file: folders, matches and what you have watched. Import it on another computer or after a reinstall."
            SettingRow { label: "Include private data"; helper: "Adds tracker sign-ins, watch history and preferences. Sign-ins are stored as plain text, so keep the file to yourself."
                Switch { objectName: "data-private"; checked: tab.includePrivate; onToggled: function(on) { tab.includePrivate = on } } }
            Row {
                spacing: theme.space(3)
                Button { objectName: "data-export"; text: "Export library"; icon: "download"; primary: true; onClicked: exportDialog.open() }
                Label { visible: tab.lastExport !== ""; anchors.verticalCenter: parent.verticalCenter; text: "Last written to " + tab.lastExport; role: "ledger"; color: theme.textDim; elide: Text.ElideMiddle; width: Math.min(implicitWidth, theme.space(120)) }
            }
        }
    }
    Component {
        id: importPanel
        Panel {
            title: "Import"; icon: "file-down"; grows: true
            helper: "Reads an AniBeam export into this library. What is on disk wins; the file fills in the rest."
            Button { objectName: "data-import"; text: "Import a file"; icon: "upload"; onClicked: importDialog.open() }
            Column {
                visible: tab.importSummary !== null
                width: parent.width
                spacing: theme.space(1)
                Label { text: "Last import"; role: "title"; font.pointSize: theme.typeNormal; bottomPadding: theme.space(1) }
                Repeater { model: tab.summaryLines(tab.importSummary); Label { required property var modelData; text: modelData; role: "ledger"; color: theme.textDim } }
                Label { visible: !!tab.importSummary && tab.importSummary.fields_ignored.length > 0; width: parent.width; wrapMode: Text.Wrap; role: "ui"; font.pointSize: theme.typeSmall; color: theme.textDim
                    text: tab.importSummary ? "Left out: " + tab.importSummary.fields_ignored.join(", ") : "" }
            }
        }
    }
    Component {
        id: storagePanel
        Panel {
            title: "Images"; icon: "image"
            helper: "Posters, banners and portraits are kept on disk so the library works offline. Clearing them frees the space; they download again as pages need them."
            Tiles { stone: theme.raised; tiles: [{ value: String(tab.storage.image_count), caption: "Images" }, { value: Fmt.bytes(tab.storage.image_bytes || 0), caption: "On disk" }] }
            Item {
                width: parent.width
                height: tab.confirmingClear ? confirm.implicitHeight : clearBtn.height
                Button { id: clearBtn; objectName: "data-clear-images"; visible: !tab.confirmingClear; text: "Clear images"; icon: "trash-2"; enabled: tab.storage.image_count > 0; onClicked: tab.confirmingClear = true }
                InlineConfirm {
                    id: confirm
                    visible: tab.confirmingClear
                    question: "Remove " + Fmt.plural(tab.storage.image_count, "image", "images") + "?"
                    confirmText: "Clear"
                    onAccepted: { tab.confirmingClear = false; var r = Door.clearImages(); if (r.error) frame.toast(r.error.message) }
                    onKept: tab.confirmingClear = false
                }
            }
        }
    }
    Component {
        id: aboutPanel
        Panel {
            title: "About"; icon: "info"; grows: true
            SettingRow { label: "Version"; Label { text: tab.about.version || ""; role: "ledger" } }
            Repeater {
                model: [["Library database", "db_path", true], ["Data", "data_dir", false], ["Settings", "config_dir", false], ["Cache", "cache_dir", false]]
                SettingRow {
                    required property var modelData
                    label: modelData[0]
                    line: tab.about[modelData[1]] || ""
                    Button { text: "Open"; icon: "folder-open"; small: true; onClicked: Qt.openUrlExternally("file://" + (modelData[2] ? tab.dir(tab.about[modelData[1]]) : tab.about[modelData[1]])) }
                }
            }
        }
    }
    FileDialog {
        id: exportDialog
        title: "Export the library"
        fileMode: FileDialog.SaveFile
        defaultSuffix: "json"
        currentFile: "file://" + (tab.about.data_dir ? tab.dir(tab.about.data_dir) : "") + "/anibeam-export.json"
        nameFilters: ["AniBeam export (*.json)"]
        onAccepted: { var r = Door.exportLibrary(tab.path(selectedFile), tab.includePrivate); frame.toast(r.error ? r.error.message : "Exporting") }
    }
    FileDialog {
        id: importDialog
        title: "Import an AniBeam export"
        nameFilters: ["AniBeam export (*.json)", "All files (*)"]
        onAccepted: { var r = Door.importLibrary(tab.path(selectedFile)); frame.toast(r.error ? r.error.message : "Importing") }
    }
}
