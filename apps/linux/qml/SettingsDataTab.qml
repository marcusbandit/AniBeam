// Storage and backup: Back up the library (one file, with private data or without), Restore
// from a backup with what the last restore did, the image cache with its size and a clear,
// and where Anibeam keeps its files. Export, import and the clear are jobs; their results
// come back as exportFinished, importFinished and imagesCleared.
import QtQuick
import QtQuick.Dialogs
import com.marcusrosado.AniBeam

SettingsTab {
    id: tab
    title: words.t("settings-data-title")
    lead: words.t("settings-data-lead")
    property bool includePrivate: false
    property string lastExport: ""
    property var importSummary: null
    property var storage: ({ image_count: 0, image_bytes: 0 })
    property bool confirmingClear: false
    readonly property var about: Door.about || ({})
    // A cache that could not be measured says so, rather than reading as empty
    property bool storageFailed: false
    function loadStorage() { var r = Door.getStorage(); storageFailed = !!r.error; if (!r.error) storage = r.reply }
    function clearImages() { var r = Door.clearImages(); if (r.error) frame.fail(r.error, tab.clearImages) }
    function path(url) { return Fmt.localPath(String(url)) }
    function dir(p) { var i = Math.max(String(p || "").lastIndexOf("/"), String(p || "").lastIndexOf("\\")); return i > 0 ? p.slice(0, i) : p }
    Component.onCompleted: loadStorage()
    Connections {
        target: Door
        function onExportFinished(p) { tab.lastExport = p; frame.toast(words.t("settings-data-backed-up", { path: p })) }
        function onImportFinished(summary) { tab.importSummary = summary; frame.toast(words.t("settings-data-restore-finished")) }
        function onImagesCleared(removed) { tab.loadStorage(); frame.toast(words.t("settings-data-images-removed", { count: removed })) }
        function onJobFailed(job, kind, error) {
            if (kind === "Export") frame.fail(error, exportDialog.open)
            else if (kind === "Import") frame.fail(error, importDialog.open)
            else if (kind === "ClearImages") frame.fail(error, tab.clearImages)
        }
    }
    function summaryLine(s) {
        if (!s) return ""
        var parts = [words.t("settings-data-summary-folders", { count: s.sources_added }),
                     words.t("settings-data-summary-series", { count: s.series_created }),
                     words.t("settings-data-summary-matches", { count: s.matches_applied }),
                     words.t("settings-data-summary-views", { views: s.views_merged, episodes: s.completed_merged }),
                     words.t("settings-data-summary-resume-points", { count: s.resume_points_merged }),
                     words.t("settings-data-summary-accounts", { count: s.accounts_imported })]
        var out = [words.t("settings-data-summary", { parts: parts.join(", ") })]
        if (s.fixes_applied > 0) out.push(words.t("settings-data-summary-fixes", { count: s.fixes_applied }))
        if (s.sources_unavailable > 0) out.push(words.t("settings-data-summary-unavailable", { count: s.sources_unavailable }))
        if (s.fields_ignored && s.fields_ignored.length) out.push(words.t("settings-data-summary-left-out", { fields: s.fields_ignored.join(", ") }))
        return out.join(" ")
    }

    Panel {
        title: words.t("settings-data-backup")
        SettingRow { label: words.t("settings-data-back-up-library"); helper: words.t("settings-data-back-up-helper")
            line: tab.lastExport !== "" ? words.t("settings-data-last-written", { path: tab.lastExport }) : ""
            Row { spacing: theme.space(2)
                Button { visible: tab.lastExport !== ""; msg: "settings-data-open-folder"; icon: "folder-open"; flat: true; onClicked: frame.openLink(Fmt.fileUrl(tab.dir(tab.lastExport))) }
                Button { objectName: "data-export"; msg: "settings-data-back-up"; icon: "file-up"; primary: true; onClicked: exportDialog.open() } } }
        SettingRow { label: words.t("settings-data-private"); helper: words.t("settings-data-private-helper")
            Switch { objectName: "data-private"; checked: tab.includePrivate; onToggled: function(on) { tab.includePrivate = on } } }
        SettingRow { label: words.t("settings-data-restore-from"); helper: words.t("settings-data-restore-helper")
            status: tab.summaryLine(tab.importSummary)
            Button { objectName: "data-import"; msg: "settings-data-restore"; icon: "file-down"; flat: true; onClicked: importDialog.open() } }
    }
    Panel {
        objectName: "data-images"
        title: words.t("settings-data-image-cache")
        helper: words.t("settings-data-image-cache-helper")
        SettingRow {
            label: tab.storageFailed ? words.t("settings-data-cache-unmeasured") : tab.storage.image_count ? words.t("settings-data-cache-size", { size: words.bytes(tab.storage.image_bytes || 0), count: tab.storage.image_count }) : words.t("settings-data-cache-empty")
            helper: tab.storageFailed ? words.t("settings-data-cache-unmeasured-helper") : ""
            Item {
                width: tab.confirmingClear ? confirm.implicitWidth : clearBtn.width
                height: tab.confirmingClear ? confirm.implicitHeight : clearBtn.height
                Button { id: clearBtn; objectName: "data-clear-images"; visible: !tab.confirmingClear; text: tab.storageFailed ? words.t("action-try-again") : words.t("action-clear"); icon: tab.storageFailed ? "refresh-cw" : "trash-2"; flat: true; enabled: tab.storageFailed || tab.storage.image_count > 0; onClicked: tab.storageFailed ? tab.loadStorage() : tab.confirmingClear = true }
                InlineConfirm {
                    id: confirm
                    visible: tab.confirmingClear
                    question: words.t("settings-data-remove-images", { count: tab.storage.image_count })
                    confirmText: words.t("action-clear")
                    onAccepted: { tab.confirmingClear = false; tab.clearImages() }
                    onKept: tab.confirmingClear = false
                }
            }
        }
    }
    Panel {
        title: words.t("settings-data-files")
        Repeater {
            model: [["settings-data-files-database", "db_path", true], ["settings-data-files-data", "data_dir", false], ["settings-data-files-settings", "config_dir", false], ["settings-data-files-cache", "cache_dir", false]]
            SettingRow {
                required property var modelData
                label: words.t(modelData[0])
                status: tab.about[modelData[1]] || ""
                Button { msg: "action-open"; icon: "folder-open"; flat: true; small: true; onClicked: frame.openLink(Fmt.fileUrl(modelData[2] ? tab.dir(tab.about[modelData[1]]) : tab.about[modelData[1]])) }
            }
        }
    }
    FileDialog {
        id: exportDialog
        title: words.t("settings-data-back-up-library")
        fileMode: FileDialog.SaveFile
        defaultSuffix: "json"
        currentFile: Fmt.fileUrl((tab.about.data_dir ? tab.dir(tab.about.data_dir) : "") + "/anibeam-export.json")
        nameFilters: [words.t("settings-data-backup-filter")]
        onAccepted: { var r = Door.exportLibrary(tab.path(selectedFile), tab.includePrivate); r.error ? frame.fail(r.error, exportDialog.open) : frame.toast(words.t("settings-data-backing-up")) }
    }
    FileDialog {
        id: importDialog
        title: words.t("settings-data-restore-from")
        nameFilters: [words.t("settings-data-backup-filter"), words.t("settings-data-all-files-filter")]
        onAccepted: { var r = Door.importLibrary(tab.path(selectedFile)); r.error ? frame.fail(r.error, importDialog.open) : frame.toast(words.t("settings-data-restoring")) }
    }
}
