// Help: the questions people ask most, each with a short answer and the button that goes to
// the fix. What it says about the library and the data reads live (how many shows are not
// recognised, where the files are), so the answer is about this install, not a manual.
import QtQuick
import com.marcusrosado.AniBeam

SettingsTab {
    id: tab
    title: "Help"
    lead: "Short answers to what people ask most, each with a way straight to the fix."
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
        title: "How should my folders look?"
        helper: "One folder per show, with its episodes inside. Name each file with its episode number, the way releases do: \"Frieren - 01.mkv\" or \"Frieren S01E01.mkv\". A second season can be a folder of its own or a \"Season 2\" folder inside the show. Films go in a folder named Movies, one file each."
        SettingRow {
            label: "For example"
            line: "Anime / Frieren / Frieren - 01.mkv\nAnime / Frieren / Season 2 / Frieren S02E01.mkv\nAnime / Movies / Suzume.mkv"
        }
        SettingRow {
            label: tab.folders === 0 ? "No folder added yet" : tab.folders === 1 ? "AniBeam reads one folder" : "AniBeam reads " + tab.folders + " folders"
            helper: "Add the folder that holds your shows, not each show on its own."
            Button { objectName: "help-open-folders"; text: tab.folders === 0 ? "Add a folder" : "Open your folders"; icon: "folder"; flat: true; onClicked: tab.openSection("library", "Folders") }
        }
    }
    Panel {
        objectName: "help-match"
        title: "Why didn't my show get recognised?"
        helper: "AniBeam looks each folder up by its name, on AniList first, then TVmaze. Extra words, a typo or a name in another language can miss. Pick the right show by hand on Show info; a choice you make is never replaced."
        SettingRow {
            label: tab.unrecognised === 0 ? "Every show is recognised" : tab.unrecognised === 1 ? "One show is not recognised" : tab.unrecognised + " shows are not recognised"
            Button { objectName: "help-open-show-info"; text: "Open Show info"; icon: "database"; flat: true; onClicked: tab.visit("metadata", {}) }
        }
    }
    Panel {
        objectName: "help-connect"
        title: "How do I connect AniList?"
        helper: "Connect it and every episode you finish is marked on your list, so your phone and your profile keep up. AniList asks you to make a small app of your own once; AniBeam walks you through it, and it takes about a minute. MyAnimeList works the same way."
        SettingRow {
            label: frame.trackerOn("anilist") && frame.trackerOn("mal") ? "Both are connected"
                 : frame.trackerOn("anilist") ? "AniList is connected" : frame.trackerOn("mal") ? "MyAnimeList is connected" : "Neither is connected yet"
            Row { spacing: theme.space(2)
                ConnectButton { objectName: "help-connect-anilist"; visible: !frame.trackerOn("anilist"); tracker: "Anilist"; text: "Connect AniList"; icon: "log-in"; flat: true; onClicked: frame.openConnect("Anilist") }
                ConnectButton { objectName: "help-connect-mal"; visible: !frame.trackerOn("mal"); tracker: "Mal"; text: "Connect MyAnimeList"; icon: "log-in"; flat: true; onClicked: frame.openConnect("Mal") } }
        }
    }
    Panel {
        objectName: "help-downloads"
        title: "Why is Downloads empty?"
        helper: "AniBeam comes with no places to search. Add a source list in Settings, under Downloads, and install the sources you want from it. Downloading also needs a torrent client: AniBeam can run its own, or use a qBittorrent you already have."
        SettingRow {
            label: "Sources and the torrent client"
            Button { objectName: "help-open-downloads"; text: "Open Downloads settings"; icon: "download"; flat: true; onClicked: tab.openSection("downloads", "Source lists") }
        }
    }
    Panel {
        objectName: "help-data"
        title: "Where is my data?"
        helper: "Your library, what you watched and your settings stay on this computer. Sign-ins for AniList and MyAnimeList go in the system password store, or in a file only you can read where there is none. Back it all up to one file from Storage and backup."
        SettingRow {
            label: "Library and history"
            status: tab.about.data_dir || ""
            Button { objectName: "help-open-data"; text: "Show the folder"; icon: "folder-open"; flat: true; enabled: !!tab.about.data_dir; onClicked: frame.openLink(Fmt.fileUrl(tab.about.data_dir)) }
        }
        SettingRow {
            label: "Back up or restore"
            Button { objectName: "help-open-storage"; text: "Open Storage and backup"; icon: "hard-drive"; flat: true; onClicked: tab.openSection("storage") }
        }
    }
    Panel {
        objectName: "help-more"
        title: "Something else?"
        helper: "Run the first setup again, or tell us what went wrong. A screenshot helps a lot."
        SettingRow {
            label: "The first setup"
            helper: "Your folder, your list and downloads, one step at a time."
            Button { objectName: "help-run-setup"; text: "Run setup again"; icon: "rotate-ccw"; flat: true; onClicked: frame.openOnboarding(0) }
        }
        SettingRow {
            label: "Report a problem"
            helper: "Opens a bug report on GitHub in your browser, with this version filled in."
            Button { objectName: "help-report"; text: "Report a problem"; icon: "external-link"; flat: true
                     onClicked: frame.openLink("https://github.com/marcusbandit/AniBeam/issues/new?template=2-bug-report.yml&version=" + encodeURIComponent(Shell.release)) }
        }
    }
}
