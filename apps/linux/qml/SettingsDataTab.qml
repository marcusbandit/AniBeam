// Storage and backup: Back up the library (one file, with private data or without), Restore
// from a backup with what the last restore did, the image cache with its size and a clear,
// and where AniBeam keeps its files. Export, import and the clear are jobs; their results
// come back as exportFinished, importFinished and imagesCleared.
import QtQuick
import QtQuick.Dialogs
import com.marcusrosado.AniBeam

SettingsTab {
    id: tab
    title: "Storage and backup"
    lead: "Move your library to another computer, keep a copy safe, or free the space images take."
    property bool includePrivate: false
    property string lastExport: ""
    property var importSummary: null
    property var storage: ({ image_count: 0, image_bytes: 0 })
    property bool confirmingClear: false
    readonly property var about: Door.about || ({})
    function loadStorage() { var r = Door.getStorage(); if (!r.error) storage = r.reply }
    function path(url) { return Fmt.localPath(String(url)) }
    function dir(p) { var i = Math.max(String(p || "").lastIndexOf("/"), String(p || "").lastIndexOf("\\")); return i > 0 ? p.slice(0, i) : p }
    Component.onCompleted: loadStorage()
    Connections {
        target: Door
        function onExportFinished(p) { tab.lastExport = p; frame.toast("Backed up to " + p) }
        function onImportFinished(summary) { tab.importSummary = summary; frame.toast("Restore finished") }
        function onImagesCleared(removed) { tab.loadStorage(); frame.toast("Removed " + Fmt.plural(removed, "image", "images")) }
        function onJobFailed(job, kind, error) { if (kind === "Export" || kind === "Import" || kind === "ClearImages") frame.toast(error && error.message ? error.message : "That did not finish") }
    }
    function summaryLine(s) {
        if (!s) return ""
        var parts = [Fmt.plural(s.sources_added, "folder", "folders") + " added",
                     Fmt.plural(s.series_created, "series", "series") + " created",
                     Fmt.plural(s.matches_applied, "match", "matches") + " applied",
                     Fmt.plural(s.views_merged, "view", "views") + " and " + Fmt.plural(s.completed_merged, "finished episode", "finished episodes") + " merged",
                     Fmt.plural(s.resume_points_merged, "resume point", "resume points") + " merged",
                     Fmt.plural(s.accounts_imported, "tracker account", "tracker accounts") + " brought in"]
        var out = parts.join(", ") + "."
        if (s.sources_unavailable > 0) out += " " + Fmt.plural(s.sources_unavailable, "folder was", "folders were") + " not found on this computer."
        if (s.fields_ignored && s.fields_ignored.length) out += " Left out: " + s.fields_ignored.join(", ") + "."
        return out
    }

    Panel {
        title: "Backup"
        SettingRow { label: "Back up the library"; helper: "One file with your folders, matches and what you have watched."
            line: tab.lastExport !== "" ? "Last written to " + tab.lastExport : ""
            Row { spacing: theme.space(2)
                Button { visible: tab.lastExport !== ""; text: "Open folder"; icon: "folder-open"; flat: true; onClicked: frame.openLink(Fmt.fileUrl(tab.dir(tab.lastExport))) }
                Button { objectName: "data-export"; text: "Back up"; icon: "file-up"; primary: true; onClicked: exportDialog.open() } } }
        SettingRow { label: "Include private data"; helper: "Adds tracker sign-ins, watch history and preferences. Sign-ins are written as plain text, so keep the file to yourself."
            Switch { objectName: "data-private"; checked: tab.includePrivate; onToggled: function(on) { tab.includePrivate = on } } }
        SettingRow { label: "Restore from a backup"; helper: "Reads a backup into this library. What is on disk wins; the file fills in the rest."
            status: tab.summaryLine(tab.importSummary)
            Button { objectName: "data-import"; text: "Restore"; icon: "file-down"; flat: true; onClicked: importDialog.open() } }
    }
    Panel {
        objectName: "data-images"
        title: "Image cache"
        helper: "Posters, banners and portraits are kept on disk so the library works offline. Cleared images download again as pages need them."
        SettingRow {
            label: tab.storage.image_count ? Fmt.bytes(tab.storage.image_bytes || 0) + " in " + Fmt.plural(tab.storage.image_count, "image", "images") : "No images cached"
            Item {
                width: tab.confirmingClear ? confirm.implicitWidth : clearBtn.width
                height: tab.confirmingClear ? confirm.implicitHeight : clearBtn.height
                Button { id: clearBtn; objectName: "data-clear-images"; visible: !tab.confirmingClear; text: "Clear"; icon: "trash-2"; flat: true; enabled: tab.storage.image_count > 0; onClicked: tab.confirmingClear = true }
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
    Panel {
        title: "Where AniBeam keeps its files"
        Repeater {
            model: [["Library database", "db_path", true], ["Data", "data_dir", false], ["Settings", "config_dir", false], ["Cache", "cache_dir", false]]
            SettingRow {
                required property var modelData
                label: modelData[0]
                status: tab.about[modelData[1]] || ""
                Button { text: "Open"; icon: "folder-open"; flat: true; small: true; onClicked: frame.openLink(Fmt.fileUrl(modelData[2] ? tab.dir(tab.about[modelData[1]]) : tab.about[modelData[1]])) }
            }
        }
    }
    FileDialog {
        id: exportDialog
        title: "Back up the library"
        fileMode: FileDialog.SaveFile
        defaultSuffix: "json"
        currentFile: Fmt.fileUrl((tab.about.data_dir ? tab.dir(tab.about.data_dir) : "") + "/anibeam-export.json")
        nameFilters: ["AniBeam backup (*.json)"]
        onAccepted: { var r = Door.exportLibrary(tab.path(selectedFile), tab.includePrivate); frame.toast(r.error ? r.error.message : "Backing up") }
    }
    FileDialog {
        id: importDialog
        title: "Restore from a backup"
        nameFilters: ["AniBeam backup (*.json)", "All files (*)"]
        onAccepted: { var r = Door.importLibrary(tab.path(selectedFile)); frame.toast(r.error ? r.error.message : "Restoring") }
    }
}
