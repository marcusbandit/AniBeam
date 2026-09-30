// The Playback tab: auto-skip, Use my mpv.conf, the language orders, the subtitle
// defaults, and the preview rendered through mpv.
import QtQuick
import com.marcusrosado.AniBeam

SettingsTab {
    id: tab
    property var defaults: JSON.parse(JSON.stringify(Door.settings.subtitle_defaults || {}))
    property var preview: ({ path: "", subtitle: "", startAt: 0 })
    Connections { target: Door; function onSettingsChanged() { tab.defaults = JSON.parse(JSON.stringify(Door.settings.subtitle_defaults)) } }
    Timer { id: save; interval: 250; onTriggered: { var r = Door.setSubtitleDefaults(tab.defaults); if (r.error) frame.toast(r.error.message) } }
    function edit(f) { var d = JSON.parse(JSON.stringify(defaults)); f(d); defaults = d; save.restart() }
    function isHex(s) { return /^#[0-9a-fA-F]{6}$/.test(s) }
    function hexOf(c) { function h(n) { return ("0" + n.toString(16)).slice(-2) } return "#" + h(c.r) + h(c.g) + h(c.b) }
    function colourOf(hex) { return { r: parseInt(hex.substr(1, 2), 16), g: parseInt(hex.substr(3, 2), 16), b: parseInt(hex.substr(5, 2), 16), a: 255 } }
    Component.onCompleted: {
        // The episode watched last, at its resume point; else the sample source.
        var recent = Door.listSeries("All", "", "LastViewed", "Desc", false)
        var chosen = null
        if (!recent.error) for (var i = 0; i < recent.reply.series.length && !chosen; i++) {
            if (!recent.reply.series[i].last_viewed_at) break
            var d = Door.getSeries(recent.reply.series[i].id)
            if (d.error) continue
            var eps = d.reply.detail.episodes
            var withResume = eps.filter(function(e) { return e.resume })
            var ep = withResume.length ? withResume[0] : (eps.length ? eps[0] : null)
            if (ep) chosen = { path: ep.path, subtitle: "", startAt: ep.resume ? ep.resume.position : 0 }
        }
        if (chosen) { preview = chosen; return }
        var sample = Player.samplePreview()
        preview = { path: sample.path, subtitle: sample.subtitle, startAt: 0 }
    }

    SettingsPair {
        split: 2 / 5
        leftPanels: [ playbackPanel, tracksPanel, subtitlePanel ]
        rightPanels: [ previewPanel ]
    }
    Component {
        id: playbackPanel
        Panel {
            title: "Playback"; icon: "play"
            SettingRow { label: "Auto-skip intro"; helper: "Jumps the intro when the file's chapters or AniSkip know where it is. Undo in the player turns it off for the session."
                Switch { checked: !!(Door.settings.auto_skip && Door.settings.auto_skip.intro); onToggled: function(on) { Door.setAutoSkip(on, !!(Door.settings.auto_skip && Door.settings.auto_skip.outro)) } } }
            SettingRow { label: "Auto-skip outro"; helper: "Jumps the outro when the file's chapters or AniSkip know where it is. Undo in the player turns it off for the session."
                Switch { checked: !!(Door.settings.auto_skip && Door.settings.auto_skip.outro); onToggled: function(on) { Door.setAutoSkip(!!(Door.settings.auto_skip && Door.settings.auto_skip.intro), on) } } }
            SettingRow { label: "Use my mpv.conf"; helper: "Loads ~/.config/mpv/mpv.conf under AniBeam's own settings. Lines that only apply at start-up, scripts, input-conf and config-dir, are ignored, and no script ever loads."
                status: Player.configLayers.length ? Player.configLayers.join("\n") : "No config layer found on disk."
                Switch { checked: Player.useMyMpvConf; onToggled: function(on) { Player.setUseMyMpvConf(on) } } }
        }
    }
    Component {
        id: tracksPanel
        Panel {
            title: "Tracks"; icon: "languages"
            SettingRow { label: "Subtitle languages"
                Field { text: (tab.defaults.subtitle_languages || []).join(", "); width: theme.space(30); onEdited: function(t) { tab.edit(function(d) { d.subtitle_languages = t.split(",").map(function(s) { return s.trim() }).filter(function(s) { return s }) }) } } }
            SettingRow { label: "Audio languages"; helper: "Comma separated, first match wins."
                Field { text: (tab.defaults.audio_languages || []).join(", "); width: theme.space(30); onEdited: function(t) { tab.edit(function(d) { d.audio_languages = t.split(",").map(function(s) { return s.trim() }).filter(function(s) { return s }) }) } } }
        }
    }
    Component {
        id: subtitlePanel
        Panel {
            title: "Subtitle defaults"; icon: "captions"
            helper: "What every session starts from. Change tracks in the player and AniBeam remembers them per series."
            SettingRow { label: "Scale"
                SliderRow { from: 0.5; to: 2.0; stepSize: 0.05; decimals: 2; value: tab.defaults.scale || 1; onMoved: function(v) { tab.edit(function(d) { d.scale = v }) } } }
            SettingRow { label: "ASS override"; helper: "Force applies the text style to styled subtitles and may break signs and karaoke."
                Seg { options: ["As scripted", "Scale only", "Force"]; index: ["AsScripted", "ScaleOnly", "Force"].indexOf(tab.defaults.ass_override || "ScaleOnly"); onPicked: function(i) { tab.edit(function(d) { d.ass_override = ["AsScripted", "ScaleOnly", "Force"][i] }) } } }
            Note { text: "Applies to SRT and VTT subtitles; ASS files carry their own styling." }
            SettingRow { label: "Font"
                Field { text: tab.defaults.text_style ? tab.defaults.text_style.font : ""; width: theme.space(30); onEdited: function(t) { if (t.trim() !== "") tab.edit(function(d) { d.text_style.font = t.trim() }) } } }
            SettingRow { label: "Colour"
                Row { spacing: theme.space(2)
                    Corner { width: theme.space(6); height: width; radius: theme.radiusSm; smoothing: theme.cornerSmoothing; color: tab.defaults.text_style ? tab.hexOf(tab.defaults.text_style.colour) : "#ffffff"; borderColor: theme.line; borderWidth: 1 }
                    Field { text: tab.defaults.text_style ? tab.hexOf(tab.defaults.text_style.colour).toUpperCase() : ""; mono: true; width: theme.space(24); onEdited: function(t) { if (tab.isHex(t)) tab.edit(function(d) { d.text_style.colour = tab.colourOf(t) }) } } } }
            SettingRow { label: "Outline"
                Row { spacing: theme.space(2)
                    Field { text: tab.defaults.text_style ? String(tab.defaults.text_style.outline_size) : ""; mono: true; width: theme.space(18); onEdited: function(t) { var v = Number(t); if (v >= 0) tab.edit(function(d) { d.text_style.outline_size = v }) } }
                    Corner { width: theme.space(6); height: width; radius: theme.radiusSm; smoothing: theme.cornerSmoothing; color: tab.defaults.text_style ? tab.hexOf(tab.defaults.text_style.outline_colour) : "#000000"; borderColor: theme.line; borderWidth: 1 }
                    Field { text: tab.defaults.text_style ? tab.hexOf(tab.defaults.text_style.outline_colour).toUpperCase() : ""; mono: true; width: theme.space(24); onEdited: function(t) { if (tab.isHex(t)) tab.edit(function(d) { d.text_style.outline_colour = tab.colourOf(t) }) } } } }
            SettingRow { label: "Shadow"
                Field { text: tab.defaults.text_style ? String(tab.defaults.text_style.shadow_offset) : ""; mono: true; width: theme.space(14); onEdited: function(t) { var v = Number(t); if (v >= 0) tab.edit(function(d) { d.text_style.shadow_offset = v }) } } }
            SettingRow { label: "Box opacity"
                SliderRow { from: 0; to: 1; stepSize: 0.05; decimals: 2; value: tab.defaults.text_style ? tab.defaults.text_style.box_opacity : 0; onMoved: function(v) { tab.edit(function(d) { d.text_style.box_opacity = v }) } } }
            SettingRow { label: "Bold"
                Switch { checked: !!(tab.defaults.text_style && tab.defaults.text_style.bold); onToggled: function(on) { tab.edit(function(d) { d.text_style.bold = on }) } } }
            SettingRow { label: "Position"
                SliderRow { from: 0; to: 150; stepSize: 1; value: tab.defaults.text_style ? tab.defaults.text_style.position : 100; onMoved: function(v) { tab.edit(function(d) { d.text_style.position = v }) } } }
        }
    }
    Component {
        id: previewPanel
        Panel {
            title: "Preview"; icon: "eye"; grows: true
            stretch: Item {
                anchors.fill: parent
                implicitHeight: theme.space(60)
                SubtitlePreview {
                    anchors.centerIn: parent
                    width: Math.min(parent.width, parent.height * 16 / 9); height: width * 9 / 16
                    defaults: tab.defaults; path: tab.preview.path; subtitle: tab.preview.subtitle; startAt: tab.preview.startAt
                }
            }
        }
    }
}
