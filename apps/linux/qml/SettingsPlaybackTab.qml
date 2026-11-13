// Playback: Skipping (intro and outro on their own switches), Next episode, Languages
// (which subtitle and audio tracks a new episode starts on, by language code, most wanted
// first), Discord (showing what plays on the person's profile, where the build has a
// Discord application) and mpv (whether their own mpv settings file is read).
import QtQuick
import com.marcusrosado.AniBeam

SettingsTab {
    id: tab
    title: words.t("settings-playback-title")
    lead: words.t("settings-playback-lead")
    readonly property var defaults: frame.subtitleDefaults
    function edit(f) { frame.editSubtitleDefaults(f) }
    // An edit still waiting on the debounce is written as the tab closes
    Component.onDestruction: if (frame) frame.flushSubtitleDefaults()
    function codes(t) { return t.split(",").map(function(s) { return s.trim() }).filter(function(s) { return s }) }
    readonly property var skip: Door.settings.auto_skip || ({})
    // Discord rich presence: a build with no Discord application has nothing to switch on
    property bool discordAvailable: false
    function setDiscord(on) {
        var p = JSON.parse(JSON.stringify(Door.preferences))
        p.discord_presence = on
        var r = Door.setPreferences(p)
        if (r.error) frame.fail(r.error, function() { tab.setDiscord(on) })
    }
    function setSkip(intro, outro) {
        var r = Door.setAutoSkip(intro, outro)
        if (r && r.error) frame.fail(r.error, function() { tab.setSkip(intro, outro) })
    }
    // A presence check that failed is not a build without Discord: it says so, with Try again
    property bool presenceFailed: false
    function readPresence() {
        var presence = Door.call("GetPresence", {})
        presenceFailed = !!presence.error
        if (!presence.error) { discordAvailable = presence.reply.available; discordConnected = presence.reply.connected }
    }
    // Whether Discord is connected right now: read once, then kept current by the core's event
    property bool discordConnected: false
    Component.onCompleted: readPresence()
    Connections { target: Door; function onPresenceChanged(connected) { tab.discordConnected = connected } }

    Panel {
        title: words.t("settings-playback-skipping")
        helper: words.t("settings-playback-skipping-helper")
        SettingRow { label: words.t("settings-playback-skip-intros")
            Switch { checked: !!tab.skip.intro; onToggled: function(on) { tab.setSkip(on, !!tab.skip.outro) } } }
        SettingRow { label: words.t("settings-playback-skip-outros"); helper: words.t("settings-playback-skip-outros-helper")
            Switch { checked: !!tab.skip.outro; onToggled: function(on) { tab.setSkip(!!tab.skip.intro, on) } } }
    }
    Panel {
        title: words.t("settings-playback-next-episode")
        SettingRow { label: words.t("settings-playback-name-next"); helper: words.t("settings-playback-name-next-helper")
            Switch { checked: frame.prefs.nextNamesEpisode; onToggled: function(on) { frame.prefs.nextNamesEpisode = on } } }
    }
    Panel {
        title: words.t("settings-playback-languages")
        helper: words.t("settings-playback-languages-helper")
        SettingRow { label: words.t("settings-playback-subtitle-languages")
            Field { text: (tab.defaults.subtitle_languages || []).join(", "); placeholder: "en"; width: theme.space(40); onEdited: function(t) { tab.edit(function(d) { d.subtitle_languages = tab.codes(t) }) } } } // i18n: none
        SettingRow { label: words.t("settings-playback-audio-languages")
            Field { text: (tab.defaults.audio_languages || []).join(", "); placeholder: "ja, en"; width: theme.space(40); onEdited: function(t) { tab.edit(function(d) { d.audio_languages = tab.codes(t) }) } } } // i18n: none
    }
    Panel {
        title: "Discord"
        SettingRow { label: words.t("settings-playback-discord")
            helper: tab.presenceFailed ? words.t("settings-playback-discord-check-failed")
                : tab.discordAvailable ? words.t("settings-playback-discord-helper") : words.t("settings-playback-discord-unavailable")
            status: !tab.discordAvailable || !Door.preferences.discord_presence ? "" : tab.discordConnected ? words.t("settings-playback-discord-connected") : words.t("settings-playback-discord-not-connected")
            Row { spacing: theme.space(2)
                Button { visible: tab.presenceFailed; msg: "action-try-again"; icon: "refresh-cw"; flat: true; small: true; onClicked: tab.readPresence() }
                Switch { objectName: "settings-discord-switch"; enabled: tab.discordAvailable; opacity: enabled ? 1 : theme.disabledOpacity; checked: !!Door.preferences.discord_presence; onToggled: function(on) { tab.setDiscord(on) } } } }
    }
    Panel {
        title: words.t("settings-playback-advanced")
        SettingRow { label: words.t("settings-playback-my-mpv"); helper: words.t("settings-playback-my-mpv-helper")
            status: Player.configLayers.length ? Player.configLayers.join("\n") : words.t("settings-playback-no-mpv-conf")
            Switch { checked: Player.useMyMpvConf; onToggled: function(on) { Player.setUseMyMpvConf(on) } } }
    }
}
