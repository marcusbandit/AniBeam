// About: the version with What's new, the update channel with a check, developer options
// (the switch, hidden only when ANIBEAM_DEV=1 forces them on), and IMDb's credit,
// which its terms ask for wherever its data is used.
import QtQuick
import com.marcusrosado.AniBeam

SettingsTab {
    id: tab
    title: words.t("settings-about-title")
    lead: words.t("settings-about-lead")
    readonly property var upd: frame.updatePanel
    // An update waits for this track, put away or not; About keeps the way to take it until
    // it is installed or the track changes
    readonly property bool offered: upd.pending && upd.phase === "offer"
    readonly property string offerLine: upd.phase === "failed" ? words.t("settings-about-update-failed")
        : upd.pending && (upd.phase === "downloading" || upd.phase === "ready") ? words.t("settings-about-update-progress", { headline: upd.headline, detail: upd.detail })
        : !offered ? ""
        : upd.how === "install" ? words.t("settings-about-update-ready", { version: upd.version })
        : upd.how === "command" ? words.t("settings-about-update-command", { version: upd.version, command: upd.command })
        : upd.how === "source" ? words.t("settings-about-update-source", { version: upd.version })
        : words.t("settings-about-update-elsewhere", { version: upd.version })

    Panel {
        title: words.t("settings-about-version")
        SettingRow { label: words.t("settings-about-build", { build: Shell.build })
            helper: tab.offerLine
            Row { spacing: theme.space(2)
                Button { objectName: "data-whats-new"; msg: "settings-about-whats-new"; icon: "sparkles"; flat: true; onClicked: frame.openWhatsNew() }
                Button { objectName: "about-update-install"; visible: tab.offered && tab.upd.how === "install"; msg: "settings-about-install"; msgArgs: ({ version: tab.upd.version }); icon: "download"; primary: true; onClicked: tab.upd.install() }
                Button { objectName: "about-update-copy"; visible: tab.offered && tab.upd.how === "command"; text: tab.upd.copyText; icon: tab.upd.copied ? "check" : "copy"; primary: true; onClicked: tab.upd.copyCommand() }
                Button { objectName: "about-update-restart"; visible: tab.upd.phase === "ready"; text: tab.upd.restartText; icon: "refresh-cw"; primary: true; onClicked: tab.upd.restart() }
                Button { objectName: "about-update-retry"; visible: tab.upd.phase === "failed"; text: tab.upd.retryText; icon: "rotate-cw"; primary: true; onClicked: tab.upd.install() }
                Button { objectName: "about-update-cancel"; visible: tab.upd.phase === "downloading"; text: tab.upd.cancelText; icon: "x"; flat: true; onClicked: tab.upd.cancel() }
                Button { objectName: "about-update-page"; visible: (tab.upd.phase === "failed" || (tab.offered && (tab.upd.how === "page" || tab.upd.how === "source"))) && tab.upd.url !== ""
                         text: tab.upd.pageText; icon: "external-link"; primary: tab.upd.how === "page" && tab.upd.phase !== "failed"; flat: !primary; onClicked: frame.openLink(tab.upd.url) } } }
        SettingRow { label: words.t("settings-about-setup"); helper: words.t("settings-about-setup-helper")
            Button { objectName: "about-run-setup"; msg: "settings-about-run-setup"; icon: "rotate-ccw"; flat: true; onClicked: frame.openOnboarding(0) } }
        SettingRow { label: words.t("settings-about-updates")
            helper: tab.upd.track === "nightly"
                ? words.t("settings-about-nightly-helper")
                : words.t("settings-about-stable-helper")
            Row { spacing: theme.space(2)
                Seg { objectName: "data-update-track"; options: [{ text: words.t("settings-about-stable"), icon: "badge-check" }, { text: words.t("settings-about-nightly"), icon: "moon" }]
                      index: tab.upd.track === "nightly" ? 1 : 0
                      onPicked: function(i) { tab.upd.setTrack(i === 1 ? "nightly" : "stable") } }
                Button { objectName: "data-check-updates"; msg: "settings-about-check-now"; icon: "refresh-cw"; flat: true; onClicked: tab.upd.checkNow() } } }
    }
    Panel {
        visible: !Dev.forced
        title: words.t("settings-about-developer")
        // Developer options (#287): off by default in every build; ANIBEAM_DEV=1 forces them
        // on and hides this switch
        SettingRow { label: words.t("settings-about-developer"); helper: words.t("settings-about-developer-helper")
            Switch { objectName: "data-developer"; checked: frame.devKept.enabled; onToggled: function(on) { frame.devKept.enabled = on; frame.devApply(); if (on) frame.go("settings", { tab: "developer" }) } } }
    }
    Panel {
        title: words.t("settings-about-credits")
        SettingRow { label: words.t("settings-about-ratings"); helper: words.t("settings-about-ratings-helper") }
        SettingRow { label: words.t("settings-about-series"); helper: words.t("settings-about-series-helper") }
    }
}
