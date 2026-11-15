// The library with nothing in it, on Library and on Home alike (#1006). Three states, each
// a title and the one thing that fixes it: no folder yet (add one), folders still being
// read (wait), and folders read with no video in them (get something, or add another).
// The folders it watches are named, so a new empty folder reads as empty, not as broken.
import QtQuick
import QtQuick.Dialogs
import com.marcusrosado.AniBeam

Column {
    id: root
    anchors.centerIn: parent
    spacing: theme.space(3)

    property var sources: []
    // Downloads has a source to search, so Get something to watch leads somewhere
    property bool canDownload: false
    readonly property bool scanning: {
        var jobs = Door.runningJobs
        for (var i = 0; i < jobs.length; i++) if (String(jobs[i].kind) === "Scan") return true
        return false
    }
    // "none", "scanning" or "nothing", for a --drive script's evals too
    readonly property string mode: sources.length === 0 ? "none" : scanning ? "scanning" : "nothing"
    // A few folders by name; past that, how many more
    readonly property int shownFolders: 3

    function reload() {
        var s = Door.listSources()
        sources = s.error ? [] : s.reply.sources
        var d = Door.getDownloads()
        canDownload = !d.error && !!d.reply.status && (d.reply.status.endpoints || []).length > 0
    }
    Component.onCompleted: if (visible) reload()
    onVisibleChanged: if (visible) reload()
    // A scan starts on a folder just added: read the folders again so it is named at once
    onScanningChanged: if (visible) reload()
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
            glyph: root.mode === "none" ? "folder-plus" : root.mode === "scanning" ? "loader-circle" : "folder-open"
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
            visible: root.canDownload && root.mode === "nothing"
            primary: true
            msg: "library-empty-get"; icon: "download"
            onClicked: frame.go("downloads")
        }
        Button {
            objectName: "library-empty-add-folder"
            primary: !(root.canDownload && root.mode === "nothing")
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
    }
    FolderDialog {
        id: folderDialog
        title: words.t("library-empty-pick-folder")
        onAccepted: { var r = Door.addSource(Fmt.localPath(String(selectedFolder))); if (r.error) frame.fail(r.error, folderDialog.open) }
    }
}
