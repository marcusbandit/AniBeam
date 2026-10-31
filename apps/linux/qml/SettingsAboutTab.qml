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
    readonly property bool offered: upd.phase === "offer" && upd.shown
    // Only a build that can replace itself installs from here; the others say where to get it
    readonly property bool installs: offered && upd.how === "install"
    readonly property string offerLine: !offered ? ""
        : upd.how === "install" ? upd.version + " is ready to install."
        : upd.how === "command" ? upd.version + " is on the AUR: " + upd.command
        : upd.how === "source" ? upd.version + " is out. This copy is built from source: pull and rebuild."
        : upd.version + " is out, with no download here for this computer."

    Panel {
        title: "Version"
        SettingRow { label: "AniBeam " + Shell.build
            helper: tab.offerLine
            Row { spacing: theme.space(2)
                Button { objectName: "data-whats-new"; text: "What's new"; icon: "sparkles"; flat: true; onClicked: frame.openWhatsNew() }
                Button { visible: tab.installs; text: "Install " + tab.upd.version; icon: "download"; primary: true; onClicked: tab.upd.install() } } }
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
