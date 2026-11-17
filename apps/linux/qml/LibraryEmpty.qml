// The library with nothing in it, on Library and on Home alike (#1006). Four states, each
// a title and the one thing that fixes it: no folder yet (add one), folders still being
// read (wait), folders read with no video in them (get something, or add another), and
// folders that are not on disk (an unplugged drive or a moved folder: fix them in Settings).
// The folders it watches are named, so a new empty folder reads as empty, not as broken.
import QtQuick
import QtQuick.Dialogs
import com.marcusrosado.Anibeam

Column {
    id: root
    spacing: theme.space(3)

    property var sources: []
    // Downloads has a source it would search (on, and running if it starts on demand), so
    // Get something to watch leads somewhere
    property bool canDownload: false
    readonly property bool scanning: {
        var jobs = Door.runningJobs
        for (var i = 0; i < jobs.length; i++) if (String(jobs[i].kind) === "Scan") return true
        return false
    }
    // A scan that just ended has the page's own debounced read still to land: keep saying
    // "looking" until it does, so a folder full of videos never flashes as empty
    property bool settling: false
    onScanningChanged: { if (!scanning) settle.restart(); debounce.restart() }
    Timer { id: settle; interval: 1200; onRunningChanged: root.settling = running }
    readonly property bool allMissing: {
        if (sources.length === 0) return false
        for (var i = 0; i < sources.length; i++) if (sources[i].available !== false) return false
        return true
    }
    // "none", "scanning", "missing" or "nothing", for a --drive script's evals too
    readonly property string mode: sources.length === 0 ? "none"
        : scanning || settling ? "scanning"
        : allMissing ? "missing" : "nothing"
    // Get something to watch leads the row only where a folder can take what it fetches
    readonly property bool getLeads: canDownload && mode === "nothing"
    // A few folders by name; past that, how many more
    readonly property int shownFolders: 3

    function reload() {
        var s = Door.listSources()
        sources = s.error ? [] : s.reply.sources
        var d = Door.getDownloads()
        var ends = !d.error && d.reply.status ? (d.reply.status.endpoints || []) : []
        var on = false
        for (var i = 0; i < ends.length && !on; i++) {
            var e = ends[i]
            if (e.on_demand && e.running !== true) continue
            on = frame.downloadsSources[e.name] !== false
        }
        canDownload = on
    }
    Component.onCompleted: if (visible) reload()
    onVisibleChanged: if (visible) reload()
    Timer { id: debounce; interval: 250; onTriggered: root.reload() }
    Connections {
        target: Door
        enabled: root.visible
        function onSourceChanged(s) { debounce.restart() }
        function onSourceRemoved(s) { debounce.restart() }
        function onScanFinished(s, a, c, r) { debounce.restart() }
    }

    Item {
        width: theme.space(12); height: width
        anchors.horizontalCenter: parent.horizontalCenter
        Icon {
            anchors.fill: parent
            size: parent.width
            glyph: root.mode === "none" ? "folder-plus" : root.mode === "scanning" ? "loader-circle"
                : root.mode === "missing" ? "folder-x" : "folder-open"
            color: theme.textFaint
            RotationAnimator on rotation { running: root.mode === "scanning" && root.visible; from: 0; to: 360; duration: 1000; loops: Animation.Infinite }
        }
    }
    Label {
        objectName: "library-empty-heading"
        anchors.horizontalCenter: parent.horizontalCenter
        role: "title"
        text: root.mode === "none" ? words.t("library-empty-none")
            : root.mode === "scanning" ? words.t("library-empty-scanning", { count: root.sources.length })
            : root.mode === "missing" ? words.t("library-empty-missing", { count: root.sources.length })
            : words.t("library-empty-nothing", { count: root.sources.length })
    }
    Column {
        objectName: "library-empty-folders"
        visible: root.sources.length > 0
        anchors.horizontalCenter: parent.horizontalCenter
        spacing: theme.space(1)
        Repeater {
            model: root.sources.slice(0, root.shownFolders)
            Label {
                anchors.horizontalCenter: parent.horizontalCenter
                width: Math.min(implicitWidth, theme.space(160))
                horizontalAlignment: Text.AlignHCenter
                elide: Text.ElideMiddle
                text: modelData.path || ""
                role: "ui"
                color: theme.textDim
            }
        }
        Label {
            anchors.horizontalCenter: parent.horizontalCenter
            visible: root.sources.length > root.shownFolders
            text: words.t("library-empty-more-folders", { count: root.sources.length - root.shownFolders })
            role: "ui"
            color: theme.textFaint
        }
    }
    Row {
        visible: root.mode !== "scanning"
        anchors.horizontalCenter: parent.horizontalCenter
        spacing: theme.space(2)
        // Something to watch first once a folder is there and Downloads can search
        Button {
            objectName: "library-empty-get"
            visible: root.getLeads
            primary: true
            msg: "library-empty-get"; icon: "download"
            onClicked: frame.go("downloads")
        }
        Button {
            objectName: "library-empty-add-folder"
            primary: !root.getLeads
            msg: "library-empty-add-folder"; icon: "folder-plus"
            onClicked: folderDialog.open()
        }
        Button {
            objectName: "library-empty-scan"
            visible: root.mode === "nothing"
            flat: true
            msg: "library-empty-scan-again"; icon: "refresh-cw"
            onClicked: { var r = Door.scan(-1); if (r.error) frame.fail(r.error, clicked) }
        }
        // The folder list, to remove a wrong folder or point a moved one at its new place
        Button {
            objectName: "library-empty-settings"
            visible: root.mode === "nothing" || root.mode === "missing"
            flat: true
            msg: "library-empty-settings"; icon: "settings"
            onClicked: frame.go("settings", { tab: "library" })
        }
    }
    FolderDialog {
        id: folderDialog
        title: words.t("library-empty-pick-folder")
        onAccepted: { var r = Door.addSource(Fmt.localPath(String(selectedFolder))); if (r.error) frame.fail(r.error, folderDialog.open) }
    }
}
