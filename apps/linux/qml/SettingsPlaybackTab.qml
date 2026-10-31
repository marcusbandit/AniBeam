// The Playback tab: auto-skip, Use my mpv.conf, showing what plays on Discord, the
// language orders, the subtitle defaults, and the preview rendered through mpv.
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
    // Discord rich presence: a build with no Discord application has nothing to switch on
    property bool discordAvailable: false
    function setDiscord(on) {
        var p = JSON.parse(JSON.stringify(Door.preferences))
        p.discord_presence = on
        var r = Door.setPreferences(p)
        if (r.error) frame.toast(r.error.message)
    }
    Component.onCompleted: {
        var presence = Door.call("GetPresence", {})
        if (!presence.error) discordAvailable = presence.reply.available
        // The episode watched last, at its resume point; else the sample
        // source. The Watched tab rides along: the episode seen last is
        // often watched-out, and the two tabs' rows are each sorted, not
        // the pair of them, so the newest view wins explicitly.
        var rows = []
        var recent = Door.listSeries("All", "", "LastViewed", "Desc", false)
        if (!recent.error) rows = rows.concat(recent.reply.series)
        var watched = Door.listSeries("Watched", "", "LastViewed", "Desc", false)
        if (!watched.error) rows = rows.concat(watched.reply.series)
        // Times reach QML as plain seconds (json.rs flatten_times)
        var viewed = function(s) { return typeof s.last_viewed_at === "number" ? s.last_viewed_at : 0 }
        rows.sort(function(a, b) { return viewed(b) - viewed(a) })
        var chosen = null
        for (var i = 0; i < rows.length && !chosen; i++) {
            if (!rows[i].last_viewed_at) break
            var d = Door.getSeries(rows[i].id)
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
        rightPanels: [ previewPanel, keysPanel ]
    }
    // The player's key map, read off KeyHelp (never shown here) so the list is the one the
    // player itself shows on ?
    property Item keyMap: KeyHelp { visible: false; open: false }
    readonly property var keyRows: {
        var out = []
        for (var g = 0; g < keyMap.groups.length; g++) {
            if (keyMap.groups[g].name === "Library") continue
            for (var k = 0; k < keyMap.groups[g].keys.length; k++) out.push(keyMap.groups[g].keys[k])
        }
        return out
    }
    Component {
        id: playbackPanel
        Panel {
            title: "Playback"; icon: "play"
            SettingRow { label: "Auto-skip intro"; helper: "Jumps the intro when the file's chapters or AniSkip know where it is. Undo in the player turns it off for the session."
                Switch { checked: !!(Door.settings.auto_skip && Door.settings.auto_skip.intro); onToggled: function(on) { Door.setAutoSkip(on, !!(Door.settings.auto_skip && Door.settings.auto_skip.outro)) } } }
            SettingRow { label: "Auto-skip outro"; helper: "Jumps the outro when the file's chapters or AniSkip know where it is. Undo in the player turns it off for the session."
                Switch { checked: !!(Door.settings.auto_skip && Door.settings.auto_skip.outro); onToggled: function(on) { Door.setAutoSkip(!!(Door.settings.auto_skip && Door.settings.auto_skip.intro), on) } } }
            SettingRow { label: "Use my mpv settings"; helper: "Reads your mpv settings file. Options that only apply at startup are skipped, and scripts never load."
                status: Player.configLayers.length ? Player.configLayers.join("\n") : "No mpv settings file found."
                Switch { checked: Player.useMyMpvConf; onToggled: function(on) { Player.setUseMyMpvConf(on) } } }
            SettingRow { label: "Show what you watch on Discord"
                helper: tab.discordAvailable ? "The series, episode and poster appear on your Discord profile while you play. Nothing is sent while this is off or Discord is closed." : "This build has no Discord application."
                Switch { objectName: "settings-discord-switch"; enabled: tab.discordAvailable; opacity: enabled ? 1 : theme.disabledOpacity; checked: !!Door.preferences.discord_presence; onToggled: function(on) { tab.setDiscord(on) } } }
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
            title: "Subtitle defaults"; icon: "captions"; grows: true; spread: true
            helper: "What every session starts from. Change tracks in the player and AniBeam remembers them per series."
            SettingRow { label: "Scale"
                SliderRow { from: 0.5; to: 2.0; stepSize: 0.05; decimals: 2; value: tab.defaults.scale || 1; onMoved: function(v) { tab.edit(function(d) { d.scale = v }) } } }
            SettingRow { label: "Styled subtitles"; helper: "Force applies your text style to every subtitle, which can break signs and karaoke."
                Seg { options: ["As scripted", "Scale only", "Force"]; index: ["AsScripted", "ScaleOnly", "Force"].indexOf(tab.defaults.ass_override || "ScaleOnly"); onPicked: function(i) { tab.edit(function(d) { d.ass_override = ["AsScripted", "ScaleOnly", "Force"][i] }) } } }
            Note { text: "Applies to plain subtitle files (SRT, VTT). Styled files (ASS) keep their own look." }
            SettingRow { label: "Font"
                Field { text: tab.defaults.text_style ? tab.defaults.text_style.font : ""; width: theme.space(30); onEdited: function(t) { if (t.trim() !== "") tab.edit(function(d) { d.text_style.font = t.trim() }) } } }
            SettingRow { label: "Colour"
                Row { spacing: theme.space(2)
                    Corner { width: theme.space(6); height: width; radius: theme.radiusSm; smoothing: theme.cornerSmoothing; color: tab.defaults.text_style ? tab.hexOf(tab.defaults.text_style.colour) : "#ffffff"; borderColor: theme.line; borderWidth: 1 }
                    Field { text: tab.defaults.text_style ? tab.hexOf(tab.defaults.text_style.colour) : ""; mono: true; width: theme.space(24); onEdited: function(t) { if (tab.isHex(t)) tab.edit(function(d) { d.text_style.colour = tab.colourOf(t) }) } } } }
            SettingRow { label: "Outline"
                Row { spacing: theme.space(2)
                    Field { text: tab.defaults.text_style ? String(tab.defaults.text_style.outline_size) : ""; mono: true; width: theme.space(18); onEdited: function(t) { var v = Number(t); if (v >= 0) tab.edit(function(d) { d.text_style.outline_size = v }) } }
                    Corner { width: theme.space(6); height: width; radius: theme.radiusSm; smoothing: theme.cornerSmoothing; color: tab.defaults.text_style ? tab.hexOf(tab.defaults.text_style.outline_colour) : "#000000"; borderColor: theme.line; borderWidth: 1 }
                    Field { text: tab.defaults.text_style ? tab.hexOf(tab.defaults.text_style.outline_colour) : ""; mono: true; width: theme.space(24); onEdited: function(t) { if (tab.isHex(t)) tab.edit(function(d) { d.text_style.outline_colour = tab.colourOf(t) }) } } } }
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
            title: "Preview"; icon: "eye"
            stretch: Item {
                anchors.fill: parent
                implicitHeight: width * 9 / 16
                SubtitlePreview {
                    anchors.centerIn: parent
                    width: Math.min(parent.width, parent.height * 16 / 9); height: width * 9 / 16
                    defaults: tab.defaults; path: tab.preview.path; subtitle: tab.preview.subtitle; startAt: tab.preview.startAt
                }
            }
        }
    }
    // What the player answers to, as many lines as the column holds
    Component {
        id: keysPanel
        Panel {
            objectName: "settings-player-keys"
            title: "Player keys"; icon: "keyboard"; grows: true
            helper: "? in the player lists them all."
            stretch: FillRows {
                anchors.fill: parent
                model: tab.keyRows
                rowHeight: theme.space(9)
                delegate: Item {
                    id: keyRow
                    required property var modelData
                    width: parent ? parent.width : 0
                    height: theme.space(9)
                    Chip { id: keyChip; anchors.verticalCenter: parent.verticalCenter; text: keyRow.modelData[0]; small: true; color: theme.raised }
                    Label {
                        anchors.left: parent.left; anchors.leftMargin: theme.space(36)
                        anchors.right: parent.right
                        anchors.verticalCenter: parent.verticalCenter
                        text: keyRow.modelData[1]
                        role: "ui"; color: theme.textDim
                        elide: Text.ElideRight
                    }
                    Rectangle { anchors.bottom: parent.bottom; width: parent.width; height: 1; color: theme.line; opacity: 0.5 }
                }
            }
        }
    }
}
