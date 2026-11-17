// Playback: Skipping (intro and outro on their own switches), Next episode, Languages
// (which subtitle and audio tracks a new episode starts on, by language code, most wanted
// first), then Subtitles: the preview, rendered through mpv on the episode watched last at
// its resume point (else the bundled sample), so every change under it shows on a real
// frame, then Size and place, and Style (how styled ASS files are treated, then the look
// plain SRT and VTT files get: font, colour, outline, shadow, the box behind them, bold).
// Everything subtitles write goes through the frame's one debounced save of the subtitle
// defaults. Then Discord (showing what plays on the person's profile, where the build has a
// Discord application, its state under the switch), Watch together, and Advanced (whether
// their own mpv settings file is read, the room server).
import QtQuick
import com.marcusrosado.Anibeam

SettingsTab {
    id: tab
    title: words.t("settings-playback-title")
    readonly property var defaults: frame.subtitleDefaults
    function edit(f) { frame.editSubtitleDefaults(f) }
    // An edit still waiting on the debounce is written as the tab closes
    Component.onDestruction: if (frame) frame.flushSubtitleDefaults()

    // ---- Subtitles
    property var preview: ({ path: "", subtitle: "", startAt: 0 })
    // Back to the look Anibeam ships with; the language orders stay as they are
    function resetStyle() {
        var stock = Player.stockSubtitleDefaults()
        tab.edit(function(d) { d.scale = stock.scale; d.ass_override = stock.ass_override; d.text_style = stock.text_style })
        frame.toast(words.t("settings-subtitles-reset-done"))
    }
    // A value a field cannot take: the field says so in red and nothing is saved
    property var bad: ({})
    function flag(key, ok) { var b = Object.assign({}, bad); if (ok) delete b[key]; else b[key] = true; bad = b; return ok }
    function isHex(s) { return /^#[0-9a-fA-F]{6}$/.test(s) }
    function hexOf(c) { function h(n) { return ("0" + n.toString(16)).slice(-2) } return "#" + h(c.r) + h(c.g) + h(c.b) }
    function colourOf(hex) { return { r: parseInt(hex.substr(1, 2), 16), g: parseInt(hex.substr(3, 2), 16), b: parseInt(hex.substr(5, 2), 16), a: 255 } }
    readonly property var ts: defaults.text_style || null
    readonly property string assMode: defaults.ass_override || "ScaleOnly"
    // The core's ASS override modes, in the order the switch shows them
    readonly property var assModes: ["AsScripted", "ScaleOnly", "Force"] // i18n: none
    function pickPreview() {
        // The episode watched last, at its resume point; else the sample source. The
        // Watched list rides along: the episode seen last is often watched out, and the two
        // lists are each sorted, not the pair of them, so the newest view wins explicitly.
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
        if (!presence.error) { discordAvailable = presence.reply.available; discordConnected = presence.reply.connected; discordProblem = presence.reply.problem || null }
    }
    // Why Discord shows nothing (#1029): serde's shape, the string "NoDiscord" or
    // { Refused: { message } }, read again on every PresenceChanged
    property var discordProblem: null
    readonly property string discordState: !Door.preferences.discord_presence ? ""
        : discordProblem === "NoDiscord" ? words.t("settings-playback-discord-no-discord")
        : discordProblem && discordProblem.Refused ? words.t("settings-playback-discord-refused", { reason: discordProblem.Refused.message })
        : discordConnected ? words.t("settings-playback-discord-connected") : words.t("settings-playback-discord-not-connected")
    // Whether Discord is connected right now: read once, then kept current by the core's event
    property bool discordConnected: false
    // The subtitle preview reads the library and starts an mpv frame, so it waits until its
    // panel comes near the view: opening Playback for a switch at the top costs neither
    property bool previewWanted: false
    function checkPreview() {
        if (!previewWanted && subtitlesPanel.parent && contentY + height > subtitlesPanel.parent.y + subtitlesPanel.y - theme.space(20)) {
            previewWanted = true
            pickPreview()
        }
    }
    onContentYChanged: checkPreview()
    onHeightChanged: checkPreview()
    // The people told no when they asked on Discord to join (#966)
    property var discordBlocked: []
    function readBlocked() {
        var r = Door.call("DiscordBlocked", {})
        if (!r.error) discordBlocked = r.reply.people || []
    }
    function clearBlocked() {
        var r = Door.call("DiscordUnblock", { user_id: null })
        if (r.error) { frame.fail(r.error); return }
        discordBlocked = r.reply.people || []
    }
    Component.onCompleted: { readPresence(); readBlocked(); Qt.callLater(checkPreview) }
    Connections { target: Door; function onPresenceChanged() { tab.readPresence() } }

    Panel {
        title: words.t("settings-playback-skipping")
        SettingRow { label: words.t("settings-playback-skip-intros")
            Switch { checked: !!tab.skip.intro; onToggled: function(on) { tab.setSkip(on, !!tab.skip.outro) } } }
        SettingRow { label: words.t("settings-playback-skip-outros")
            Switch { checked: !!tab.skip.outro; onToggled: function(on) { tab.setSkip(!!tab.skip.intro, on) } } }
    }
    Panel {
        title: words.t("settings-playback-next-episode")
        SettingRow { label: words.t("settings-playback-name-next")
            Switch { checked: frame.prefs.nextNamesEpisode; onToggled: function(on) { frame.prefs.nextNamesEpisode = on } } }
    }
    Panel {
        title: words.t("settings-playback-languages")
        SettingRow { label: words.t("settings-playback-subtitle-languages")
            Field { text: (tab.defaults.subtitle_languages || []).join(", "); placeholder: "en"; width: theme.space(40); onEdited: function(t) { tab.edit(function(d) { d.subtitle_languages = tab.codes(t) }) } } } // i18n: none
        SettingRow { label: words.t("settings-playback-audio-languages")
            Field { text: (tab.defaults.audio_languages || []).join(", "); placeholder: "ja, en"; width: theme.space(40); onEdited: function(t) { tab.edit(function(d) { d.audio_languages = tab.codes(t) }) } } } // i18n: none
    }
    Panel {
        id: subtitlesPanel
        objectName: "settings-subtitles"
        title: words.t("settings-subtitles-title")
        bare: true
        stretch: Item {
            anchors.fill: parent
            implicitHeight: Math.round(width * 9 / 16)
            Rectangle {
                anchors.fill: parent
                radius: theme.radiusTile
                color: "#000000"
                clip: true
                Loader {
                    anchors.fill: parent
                    active: tab.previewWanted
                    sourceComponent: SubtitlePreview {
                        defaults: tab.defaults; path: tab.preview.path; subtitle: tab.preview.subtitle; startAt: tab.preview.startAt
                    }
                }
            }
        }
    }
    Panel {
        title: words.t("settings-subtitles-size-place")
        SettingRow { label: words.t("settings-subtitles-size")
            SliderRow { from: 0.5; to: 2.0; stepSize: 0.05; decimals: 2; unit: "\u00d7"; value: tab.defaults.scale || 1; onMoved: function(v) { tab.edit(function(d) { d.scale = v }) } } }
        SettingRow { label: words.t("settings-subtitles-position")
            SliderRow { from: 0; to: 150; stepSize: 1; unit: "%"; value: tab.ts ? tab.ts.position : 100; onMoved: function(v) { tab.edit(function(d) { d.text_style.position = v }) } } }
    }
    Panel {
        title: words.t("settings-subtitles-style")
        SettingRow { label: words.t("settings-subtitles-styled")
            Seg { options: [words.t("settings-subtitles-as-made"), words.t("settings-subtitles-size-only"), words.t("settings-subtitles-mine")]; index: tab.assModes.indexOf(tab.assMode); onPicked: function(i) { tab.edit(function(d) { d.ass_override = tab.assModes[i] }) } } }
        SettingRow { label: words.t("settings-subtitles-font"); status: tab.bad.font ? words.t("settings-subtitles-font-invalid") : ""
            Field { text: tab.ts ? tab.ts.font : ""; width: theme.space(40); invalid: !!tab.bad.font; onEdited: function(t) { if (tab.flag("font", t.trim() !== "")) tab.edit(function(d) { d.text_style.font = t.trim() }) } } }
        SettingRow { label: words.t("settings-subtitles-bold")
            Switch { checked: !!(tab.ts && tab.ts.bold); onToggled: function(on) { tab.edit(function(d) { d.text_style.bold = on }) } } }
        SettingRow { label: words.t("settings-subtitles-text-colour"); status: tab.bad.colour ? words.t("settings-subtitles-colour-invalid") : ""
            Row { spacing: theme.space(2)
                Corner { anchors.verticalCenter: parent.verticalCenter; width: theme.space(7); height: width; radius: width / 2; color: tab.ts ? tab.hexOf(tab.ts.colour) : "#ffffff"; borderColor: theme.lineStrong; borderWidth: 1 }
                Field { text: tab.ts ? tab.hexOf(tab.ts.colour) : ""; mono: true; width: theme.space(28); invalid: !!tab.bad.colour; onEdited: function(t) { if (tab.flag("colour", tab.isHex(t))) tab.edit(function(d) { d.text_style.colour = tab.colourOf(t) }) } } } }
        SettingRow { label: words.t("settings-subtitles-outline"); status: tab.bad.outline ? words.t("settings-subtitles-outline-invalid") : tab.bad.outlineColour ? words.t("settings-subtitles-outline-colour-invalid") : ""
            Row { spacing: theme.space(2)
                Field { text: tab.ts ? String(tab.ts.outline_size) : ""; mono: true; width: theme.space(16); invalid: !!tab.bad.outline; onEdited: function(t) { var v = Number(t); if (tab.flag("outline", t.trim() !== "" && v >= 0)) tab.edit(function(d) { d.text_style.outline_size = v }) } }
                Corner { anchors.verticalCenter: parent.verticalCenter; width: theme.space(7); height: width; radius: width / 2; color: tab.ts ? tab.hexOf(tab.ts.outline_colour) : "#000000"; borderColor: theme.lineStrong; borderWidth: 1 }
                Field { text: tab.ts ? tab.hexOf(tab.ts.outline_colour) : ""; mono: true; width: theme.space(28); invalid: !!tab.bad.outlineColour; onEdited: function(t) { if (tab.flag("outlineColour", tab.isHex(t))) tab.edit(function(d) { d.text_style.outline_colour = tab.colourOf(t) }) } } } }
        SettingRow { label: words.t("settings-subtitles-shadow"); status: tab.bad.shadow ? words.t("settings-subtitles-shadow-invalid") : ""
            Field { text: tab.ts ? String(tab.ts.shadow_offset) : ""; mono: true; width: theme.space(16); invalid: !!tab.bad.shadow; onEdited: function(t) { var v = Number(t); if (tab.flag("shadow", t.trim() !== "" && v >= 0)) tab.edit(function(d) { d.text_style.shadow_offset = v }) } } }
        SettingRow { label: words.t("settings-subtitles-box")
            SliderRow { from: 0; to: 1; stepSize: 0.05; decimals: 2; value: tab.ts ? tab.ts.box_opacity : 0; onMoved: function(v) { tab.edit(function(d) { d.text_style.box_opacity = v }) } } }
        SettingRow { label: words.t("settings-subtitles-start-over")
            Button { objectName: "subtitle-reset"; msg: "settings-subtitles-reset"; icon: "rotate-ccw"; flat: true; onClicked: tab.resetStyle() } }
    }
    Panel {
        title: "Discord"
        SettingRow { label: words.t("settings-playback-discord")
            status: tab.presenceFailed ? words.t("settings-playback-discord-check-failed")
                : !tab.discordAvailable ? words.t("settings-playback-discord-unavailable")
                : tab.discordState
            Row { spacing: theme.space(2)
                Button { visible: tab.presenceFailed; msg: "action-try-again"; icon: "refresh-cw"; flat: true; small: true; onClicked: tab.readPresence() }
                Switch { objectName: "settings-discord-switch"; enabled: tab.discordAvailable; opacity: enabled ? 1 : theme.disabledOpacity; checked: !!Door.preferences.discord_presence; onToggled: function(on) { tab.setDiscord(on) } } } }
        SettingRow { label: words.t("settings-playback-discord-blocked")
            visible: tab.discordAvailable
            status: tab.discordBlocked.map(function(p) { return p.name }).join(", ")
            Button { objectName: "settings-discord-unblock"; msg: "action-clear"; flat: true; small: true
                enabled: tab.discordBlocked.length > 0; opacity: enabled ? 1 : theme.disabledOpacity
                onClicked: tab.clearBlocked() } }
    }
    // Watch together (#846): the name others in a room see. Kept per machine, like the relay.
    Panel {
        title: words.t("settings-playback-together")
        SettingRow { label: words.t("settings-playback-room-name")
            Field { objectName: "settings-room-name"; text: Door.preferences.room_name || ""; placeholder: frame.room.suggestedName || words.t("settings-playback-room-name-placeholder"); width: theme.space(48)
                accessibleName: words.t("settings-playback-room-name")
                onEdited: function(t) { if (t.trim() !== "" && t.trim() !== (Door.preferences.room_name || "") && !frame.room.setName(t)) frame.toast(frame.room.error) } } }
    }
    Panel {
        title: words.t("settings-playback-advanced")
        SettingRow { label: words.t("settings-playback-my-mpv")
            status: Player.configLayers.length ? Player.configLayers.join("\n") : words.t("settings-playback-no-mpv-conf")
            Switch { checked: Player.useMyMpvConf; onToggled: function(on) { Player.setUseMyMpvConf(on) } } }
        SettingRow { label: words.t("settings-playback-room-relay")
            Field { objectName: "settings-room-relay"; text: Door.preferences.room_relay || ""; mono: true; width: theme.space(64)
                accessibleName: words.t("settings-playback-room-relay")
                onEdited: function(t) { if (t.trim() !== (Door.preferences.room_relay || "")) frame.room.setRelay(t) } } }
    }
}
