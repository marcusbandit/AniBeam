// Help: the questions people ask most, each with a short answer and the button that goes to
// the fix. What it says about the library and the data reads live (how many shows are not
// recognised, where the files are), so the answer is about this install, not a manual.
import QtQuick
import com.marcusrosado.AniBeam

SettingsTab {
    id: tab
    title: words.t("settings-help-title")
    lead: words.t("settings-help-lead")
    readonly property var about: Door.about || ({})
    // Folders AniBeam could not put a name to, from the library as it is now
    property int unrecognised: 0
    property int folders: 0
    // Counted the way Show info sections them, hidden shows included when they are revealed
    function readLibrary() {
        var r = Door.listMetadata("All", "", Door.revealHidden)
        if (!r.error) {
            var n = 0, rows = r.reply.rows || []
            for (var i = 0; i < rows.length; i++) {
                var card = rows[i].series || {}
                if (card.missing !== true && !frame.hasMatch(card)) n++
            }
            tab.unrecognised = n
        }
        var s = Door.listSources()
        tab.folders = s.error ? 0 : s.reply.sources.length
    }
    Component.onCompleted: readLibrary()
    Timer { id: readDebounce; interval: 400; onTriggered: tab.readLibrary() }
    Connections {
        target: Door
        function onSourceChanged(s) { readDebounce.restart() }
        function onSourceRemoved(s) { readDebounce.restart() }
        function onScanFinished(s, a, c, r) { readDebounce.restart() }
        function onSeriesChanged(cards) { readDebounce.restart() }
        function onRevealHiddenChanged() { readDebounce.restart() }
    }
    // A jump to another section of Settings is a visit of its own, and this one is kept as
    // Help, so Back from there comes back here rather than to whichever section Settings
    // opened on
    function visit(page, props) {
        var n = frame.nav
        n.current = { page: n.current.page, props: Object.assign({}, n.current.props, { tab: "help" }), label: n.current.label }
        frame.go(page, props)
    }
    function openSection(id, focus) { visit("settings", { tab: id, focus: focus || "" }) }

    Panel {
        objectName: "help-folders"
        title: words.t("settings-help-folders")
        helper: words.t("settings-help-folders-helper")
        SettingRow {
            label: words.t("settings-help-for-example")
            line: words.t("settings-help-folders-example")
        }
        SettingRow {
            label: tab.folders === 0 ? words.t("settings-help-no-folder") : words.t("settings-help-reads-folders", { count: tab.folders })
            helper: words.t("settings-help-add-folder-helper")
            Button { objectName: "help-open-folders"; text: tab.folders === 0 ? words.t("settings-help-add-folder") : words.t("settings-help-open-folders"); icon: "folder"; flat: true; onClicked: tab.openSection("library", words.t("settings-library-folders")) }
        }
    }
    Panel {
        objectName: "help-match"
        title: words.t("settings-help-match")
        helper: words.t("settings-help-match-helper")
        SettingRow {
            label: tab.unrecognised === 0 ? words.t("settings-help-all-recognised") : words.t("settings-help-unrecognised", { count: tab.unrecognised })
            Button { objectName: "help-open-show-info"; msg: "settings-help-open-show-info"; icon: "database"; flat: true; onClicked: tab.visit("metadata", {}) }
        }
    }
    Panel {
        objectName: "help-connect"
        title: words.t("settings-help-connect")
        helper: words.t("settings-help-connect-helper")
        SettingRow {
            label: frame.trackerOn("anilist") && frame.trackerOn("mal") ? words.t("settings-help-both-connected")
                 : frame.trackerOn("anilist") ? words.t("settings-help-anilist-connected") : frame.trackerOn("mal") ? words.t("settings-help-mal-connected") : words.t("settings-help-none-connected")
            Row { spacing: theme.space(2)
                ConnectButton { objectName: "help-connect-anilist"; visible: !frame.trackerOn("anilist"); tracker: "Anilist"; text: words.t("settings-help-connect-anilist"); icon: "log-in"; flat: true; onClicked: frame.openConnect("Anilist") }
                ConnectButton { objectName: "help-connect-mal"; visible: !frame.trackerOn("mal"); tracker: "Mal"; text: words.t("settings-help-connect-mal"); icon: "log-in"; flat: true; onClicked: frame.openConnect("Mal") } }
        }
    }
    Panel {
        objectName: "help-downloads"
        title: words.t("settings-help-downloads")
        helper: words.t("settings-help-downloads-helper")
        SettingRow {
            label: words.t("settings-help-sources")
            Button { objectName: "help-open-downloads"; msg: "settings-help-open-downloads"; icon: "download"; flat: true; onClicked: tab.openSection("downloads", words.t("endpoint-repos-panel-title")) }
        }
    }
    Panel {
        objectName: "help-data"
        title: words.t("settings-help-data")
        helper: words.t("settings-help-data-helper")
        SettingRow {
            label: words.t("settings-help-library-history")
            status: tab.about.data_dir || ""
            Button { objectName: "help-open-data"; msg: "settings-help-show-folder"; icon: "folder-open"; flat: true; enabled: !!tab.about.data_dir; onClicked: frame.openLink(Fmt.fileUrl(tab.about.data_dir)) }
        }
        SettingRow {
            label: words.t("settings-help-backup")
            Button { objectName: "help-open-storage"; msg: "settings-help-open-storage"; icon: "hard-drive"; flat: true; onClicked: tab.openSection("storage") }
        }
    }
    Panel {
        objectName: "help-more"
        title: words.t("settings-help-more")
        helper: words.t("settings-help-more-helper")
        SettingRow {
            label: words.t("settings-help-first-setup")
            helper: words.t("settings-help-first-setup-helper")
            Button { objectName: "help-run-setup"; msg: "settings-help-run-setup"; icon: "rotate-ccw"; flat: true; onClicked: frame.openOnboarding(0) }
        }
        SettingRow {
            label: words.t("settings-help-report")
            helper: words.t("settings-help-report-helper")
            Button { objectName: "help-report"; msg: "settings-help-report"; icon: "external-link"; flat: true
                     onClicked: frame.openLink("https://github.com/marcusbandit/AniBeam/issues/new?template=2-bug-report.yml&version=" + encodeURIComponent(Shell.release)) }
        }
    }
}
