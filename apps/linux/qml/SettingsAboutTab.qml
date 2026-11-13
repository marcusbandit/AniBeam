// About: the version with What's new, the update channel with a check, developer options
// (the switch, hidden only when ANIBEAM_DEV=1 forces them on), and IMDb's credit,
// which its terms ask for wherever its data is used.
import QtQuick
import com.marcusrosado.AniBeam

SettingsTab {
    id: tab
    title: "About"
    lead: "Which AniBeam this is, and how it hears about the next one."
    readonly property var upd: frame.updatePanel
    // An update waits for this track, put away or not; About keeps the way to take it until
    // it is installed or the track changes
    readonly property bool offered: upd.pending && upd.phase === "offer"
    readonly property string offerLine: upd.phase === "failed" ? "The update did not finish. Try again, or get it from the release page."
        : upd.pending && (upd.phase === "downloading" || upd.phase === "ready") ? upd.headline + "  \u00b7  " + upd.detail
        : !offered ? ""
        : upd.how === "install" ? upd.version + " is ready to install."
        : upd.how === "command" ? upd.version + " is out. Update it with " + upd.command
        : upd.how === "source" ? upd.version + " is out. This copy is built from source: pull and rebuild."
        : upd.version + " is out, with no download here for this computer."

    Panel {
        title: "Version"
        SettingRow { label: "AniBeam " + Shell.build
            helper: tab.offerLine
            Row { spacing: theme.space(2)
                Button { objectName: "data-whats-new"; text: "What's new"; icon: "sparkles"; flat: true; onClicked: frame.openWhatsNew() }
                Button { objectName: "about-update-install"; visible: tab.offered && tab.upd.how === "install"; text: "Install " + tab.upd.version; icon: "download"; primary: true; onClicked: tab.upd.install() }
                Button { objectName: "about-update-copy"; visible: tab.offered && tab.upd.how === "command"; text: tab.upd.copyText; icon: tab.upd.copied ? "check" : "copy"; primary: true; onClicked: tab.upd.copyCommand() }
                Button { objectName: "about-update-restart"; visible: tab.upd.phase === "ready"; text: tab.upd.restartText; icon: "refresh-cw"; primary: true; onClicked: tab.upd.restart() }
                Button { objectName: "about-update-retry"; visible: tab.upd.phase === "failed"; text: tab.upd.retryText; icon: "rotate-cw"; primary: true; onClicked: tab.upd.install() }
                Button { objectName: "about-update-cancel"; visible: tab.upd.phase === "downloading"; text: tab.upd.cancelText; icon: "x"; flat: true; onClicked: tab.upd.cancel() }
                Button { objectName: "about-update-page"; visible: (tab.upd.phase === "failed" || (tab.offered && (tab.upd.how === "page" || tab.upd.how === "source"))) && tab.upd.url !== ""
                         text: tab.upd.pageText; icon: "external-link"; primary: tab.upd.how === "page" && tab.upd.phase !== "failed"; flat: !primary; onClicked: frame.openLink(tab.upd.url) } } }
        SettingRow { label: "Setup"; helper: "The walkthrough a new install opens with: your folder, your list and downloads."
            Button { objectName: "about-run-setup"; text: "Run setup again"; icon: "rotate-ccw"; flat: true; onClicked: frame.openOnboarding(0) } }
        SettingRow { label: "Updates"
            helper: tab.upd.track === "nightly"
                ? "A new build most days. Newer, and less tested."
                : "A new version once it has been tested and released."
            Row { spacing: theme.space(2)
                Seg { objectName: "data-update-track"; options: [{ text: "Stable", icon: "badge-check" }, { text: "Nightly", icon: "moon" }]
                      index: tab.upd.track === "nightly" ? 1 : 0
                      onPicked: function(i) { tab.upd.setTrack(i === 1 ? "nightly" : "stable") } }
                Button { objectName: "data-check-updates"; text: "Check now"; icon: "refresh-cw"; flat: true; onClicked: tab.upd.checkNow() } } }
    }
    Panel {
        visible: !Dev.forced
        title: "Developer options"
        // Developer options (#287): off by default in every build; ANIBEAM_DEV=1 forces them
        // on and hides this switch
        SettingRow { label: "Developer options"; helper: "Adds a Developer section that times every page, shows memory and the full technical log, and can turn images off."
            Switch { objectName: "data-developer"; checked: frame.devKept.enabled; onToggled: function(on) { frame.devKept.enabled = on; frame.devApply(); if (on) frame.go("settings", { tab: "developer" }) } } }
    }
    Panel {
        title: "Credits"
        SettingRow { label: "Ratings"; helper: "Information courtesy of IMDb (https://www.imdb.com). Used with permission." }
        SettingRow { label: "Series and episodes"; helper: "AniList and TVmaze. Skip times from AniSkip." }
    }
}
