// Playback: Skipping (intro and outro on their own switches), Languages (which subtitle
// and audio tracks a new episode starts on, by language code, most wanted first) and mpv
// (whether the person's own mpv settings file is read, with the files it found).
import QtQuick
import com.marcusrosado.AniBeam

SettingsTab {
    id: tab
    title: "Playback"
    lead: "What the player does on its own, so you only reach for it when you want to."
    property var defaults: JSON.parse(JSON.stringify(Door.settings.subtitle_defaults || {}))
    Connections { target: Door; function onSettingsChanged() { tab.defaults = JSON.parse(JSON.stringify(Door.settings.subtitle_defaults)) } }
    Timer { id: save; interval: 250; onTriggered: { var r = Door.setSubtitleDefaults(tab.defaults); if (r.error) frame.toast(r.error.message) } }
    // A change still waiting on the debounce is written on the way out
    Component.onDestruction: if (save.running) { save.stop(); Door.setSubtitleDefaults(tab.defaults) }
    function edit(f) { var d = JSON.parse(JSON.stringify(defaults)); f(d); defaults = d; save.restart() }
    function codes(t) { return t.split(",").map(function(s) { return s.trim() }).filter(function(s) { return s }) }
    readonly property var skip: Door.settings.auto_skip || ({})

    Panel {
        title: "Skipping"
        helper: "Uses the file's chapters, else AniSkip. Undo in the player turns skipping off until AniBeam closes."
        SettingRow { label: "Skip intros"
            Switch { checked: !!tab.skip.intro; onToggled: function(on) { Door.setAutoSkip(on, !!tab.skip.outro) } } }
        SettingRow { label: "Skip outros"; helper: "Reaching the outro also marks the episode watched."
            Switch { checked: !!tab.skip.outro; onToggled: function(on) { Door.setAutoSkip(!!tab.skip.intro, on) } } }
    }
    Panel {
        title: "Next episode"
        SettingRow { label: "Name the next episode"; helper: "The Next button at the end of an episode shows the next one's title. Off, it shows only its number."
            Switch { checked: frame.prefs.nextNamesEpisode; onToggled: function(on) { frame.prefs.nextNamesEpisode = on } } }
    }
    Panel {
        title: "Languages"
        helper: "Language codes, most wanted first. The first one a file has wins. A track you pick in the player is remembered for that series."
        SettingRow { label: "Subtitle languages"
            Field { text: (tab.defaults.subtitle_languages || []).join(", "); placeholder: "en"; width: theme.space(40); onEdited: function(t) { tab.edit(function(d) { d.subtitle_languages = tab.codes(t) }) } } }
        SettingRow { label: "Audio languages"
            Field { text: (tab.defaults.audio_languages || []).join(", "); placeholder: "ja, en"; width: theme.space(40); onEdited: function(t) { tab.edit(function(d) { d.audio_languages = tab.codes(t) }) } } }
    }
    Panel {
        title: "mpv"
        SettingRow { label: "Use my mpv settings"; helper: "Reads your mpv.conf. Options that only work at startup are skipped, and scripts never load."
            status: Player.configLayers.length ? Player.configLayers.join("\n") : "No mpv settings file found."
            Switch { checked: Player.useMyMpvConf; onToggled: function(on) { Player.setUseMyMpvConf(on) } } }
    }
}
