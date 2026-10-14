// The Data tab: export and import of the library (the anibeam-export format), the image
// cache with its size and a clear, and where AniBeam keeps its files. Export, import and the
// clear are jobs; their results come back as exportFinished, importFinished and
// imagesCleared, and each lands in its own section. Export and Import sit beside About,
// the pair squared off; the Images section runs the width under them and takes the rest
// of the viewport with a wall of the library's cached posters, as many as it holds.
import QtQuick
import QtQuick.Layouts
import QtQuick.Dialogs
import com.marcusrosado.AniBeam

SettingsTab {
    id: tab
    property bool includePrivate: false
    property string lastExport: ""
    property var importSummary: null
    property var storage: ({ image_count: 0, image_bytes: 0 })
    property bool confirmingClear: false
    property var posters: []                // the library's cached poster files, A to Z
    function loadPosters() {
        var r = Door.listLibrary("All", "", "Alpha", "Asc", false)
        if (r.error) return
        posters = (r.reply.series || []).filter(function(c) { return !!c.poster }).map(function(c) { return { id: c.id, title: c.title, poster: c.poster } })
    }
    readonly property var about: Door.about || ({})
    function loadStorage() { var r = Door.getStorage(); if (!r.error) storage = r.reply }
    function path(url) { return decodeURIComponent(String(url).replace("file://", "")) }
    function dir(p) { var i = String(p || "").lastIndexOf("/"); return i > 0 ? p.slice(0, i) : p }
    Component.onCompleted: { loadStorage(); loadPosters() }
    Connections {
        target: Door
        function onExportFinished(p) { tab.lastExport = p; frame.toast("Exported to " + p) }
        function onImportFinished(summary) { tab.importSummary = summary; frame.toast("Import finished") }
        function onImagesCleared(removed) { tab.loadStorage(); tab.loadPosters(); frame.toast("Removed " + Fmt.plural(removed, "image", "images")) }
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
        evenColumns: true
        leftPanels: [ exportPanel, importPanel ]
        rightPanels: [ aboutPanel ]
    }
    Loader { property bool grows: true; sourceComponent: storagePanel; Layout.fillWidth: true; Layout.fillHeight: true; Layout.minimumHeight: item ? item.implicitHeight : 0 }
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
            title: "Import"; icon: "file-down"
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
            objectName: "data-images"
            title: "Images"; icon: "image"; grows: true
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
            // What the cache holds: the library's posters, whole rows only
            stretch: Item {
                id: wallBox
                anchors.fill: parent
                readonly property real gap: theme.space(2)
                // The fewest columns (the largest posters) that fit every poster in the
                // height; past the smallest poster, whole rows of the smallest
                readonly property real minW: theme.space(20)
                readonly property int cols: {
                    var n = tab.posters.length, most = Math.max(1, Math.floor((width + gap) / (minW + gap)))
                    for (var c = 1; c <= most; c++) {
                        var cw = (width - (c - 1) * gap) / c, r = Math.ceil(n / c)
                        if (r * Math.round(cw * 1.5) + (r - 1) * gap <= height) return c
                    }
                    return most
                }
                readonly property real w: (width - (cols - 1) * gap) / cols
                readonly property real h0: Math.round(w * 1.5)
                readonly property int rows: Math.min(Math.ceil(tab.posters.length / cols), Math.max(0, Math.floor((height + gap) / (h0 + gap))))
                // The rows run to the tile's foot: each poster a little taller, cropped,
                // while that stays under 1.9 to 1
                readonly property real h: rows > 0 ? Math.min(w * 1.9, (height - (rows - 1) * gap) / rows) : h0
                implicitHeight: tab.posters.length ? Math.round(minW * 1.5) : 0
                clip: true
                Grid {
                    columns: wallBox.cols
                    spacing: wallBox.gap
                    Repeater {
                        model: tab.posters.slice(0, wallBox.cols * wallBox.rows)
                        // A plain clipped Image, not a Corner fill: the Shape fill left
                        // whole runs of these cells blank in the captures
                        Rectangle {
                            id: cell
                            required property var modelData
                            width: wallBox.w; height: wallBox.h
                            color: theme.raised
                            clip: true
                            Image { anchors.fill: parent; source: "file://" + cell.modelData.poster; fillMode: Image.PreserveAspectCrop; asynchronous: true; sourceSize: theme.posterDecodeSize }
                            MouseArea { id: m; anchors.fill: parent; hoverEnabled: true; cursorShape: Qt.PointingHandCursor; onClicked: frame.go("series", { id: cell.modelData.id }, cell.modelData.title) }
                            Tooltip { text: cell.modelData.title }
                        }
                    }
                }
            }
        }
    }
    Component {
        id: aboutPanel
        Panel {
            title: "About"; icon: "info"
            SettingRow { label: "Version"; Label { text: tab.about.version || ""; role: "ledger" } }
            Repeater {
                model: [["Library database", "db_path", true], ["Data", "data_dir", false], ["Settings", "config_dir", false], ["Cache", "cache_dir", false]]
                SettingRow {
                    required property var modelData
                    label: modelData[0]
                    line: tab.about[modelData[1]] || ""
                    Button { text: "Open"; icon: "folder-open"; small: true; onClicked: frame.openLink("file://" + (modelData[2] ? tab.dir(tab.about[modelData[1]]) : tab.about[modelData[1]])) }
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
