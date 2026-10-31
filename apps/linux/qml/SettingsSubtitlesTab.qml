// Subtitles: the preview first, rendered through mpv on the episode watched last at its
// resume point (else the bundled sample), so every change below shows on a real frame;
// then Size and place, and Style (how styled ASS files are treated, then the look plain
// SRT and VTT files get: font, colour, outline, shadow, the box behind them, bold).
// Everything writes the subtitle defaults through one debounced save.
import QtQuick
import com.marcusrosado.AniBeam

SettingsTab {
    id: tab
    title: "Subtitles"
    lead: "What every episode starts with. Changes show on the frame below as you make them."
    property var defaults: JSON.parse(JSON.stringify(Door.settings.subtitle_defaults || {}))
    property var preview: ({ path: "", subtitle: "", startAt: 0 })
    Connections { target: Door; function onSettingsChanged() { tab.defaults = JSON.parse(JSON.stringify(Door.settings.subtitle_defaults)) } }
    Timer { id: save; interval: 250; onTriggered: { var r = Door.setSubtitleDefaults(tab.defaults); if (r.error) frame.toast(r.error.message) } }
    // A change still waiting on the debounce is written on the way out
    Component.onDestruction: if (save.running) { save.stop(); Door.setSubtitleDefaults(tab.defaults) }
    // Back to the look AniBeam ships with; the language orders stay as they are
    function resetStyle() {
        var stock = Player.stockSubtitleDefaults()
        tab.edit(function(d) { d.scale = stock.scale; d.ass_override = stock.ass_override; d.text_style = stock.text_style })
        frame.toast("Subtitles look as they did at first")
    }
    // A value a field cannot take: the field says so in red and nothing is saved
    property var bad: ({})
    function flag(key, ok) { var b = Object.assign({}, bad); if (ok) delete b[key]; else b[key] = true; bad = b; return ok }
    function edit(f) { var d = JSON.parse(JSON.stringify(defaults)); f(d); defaults = d; save.restart() }
    function isHex(s) { return /^#[0-9a-fA-F]{6}$/.test(s) }
    function hexOf(c) { function h(n) { return ("0" + n.toString(16)).slice(-2) } return "#" + h(c.r) + h(c.g) + h(c.b) }
    function colourOf(hex) { return { r: parseInt(hex.substr(1, 2), 16), g: parseInt(hex.substr(3, 2), 16), b: parseInt(hex.substr(5, 2), 16), a: 255 } }
    readonly property var ts: defaults.text_style || null
    readonly property string assMode: defaults.ass_override || "ScaleOnly"
    Component.onCompleted: {
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

    Panel {
        bare: true
        stretch: Item {
            anchors.fill: parent
            implicitHeight: Math.round(width * 9 / 16)
            Rectangle {
                anchors.fill: parent
                radius: theme.radiusTile
                color: "#000000"
                clip: true
                SubtitlePreview {
                    anchors.fill: parent
                    defaults: tab.defaults; path: tab.preview.path; subtitle: tab.preview.subtitle; startAt: tab.preview.startAt
                }
            }
        }
    }
    Panel {
        title: "Size and place"
        SettingRow { label: "Size"
            SliderRow { from: 0.5; to: 2.0; stepSize: 0.05; decimals: 2; unit: "\u00d7"; value: tab.defaults.scale || 1; onMoved: function(v) { tab.edit(function(d) { d.scale = v }) } } }
        SettingRow { label: "Position"; helper: "100 sits on the bottom edge. Lower numbers lift the lines up the frame."
            SliderRow { from: 0; to: 150; stepSize: 1; unit: "%"; value: tab.ts ? tab.ts.position : 100; onMoved: function(v) { tab.edit(function(d) { d.text_style.position = v }) } } }
    }
    Panel {
        title: "Style"
        SettingRow { label: "Styled subtitles"
            helper: tab.assMode === "AsScripted" ? "Styled (ASS) files look exactly as their makers set them."
                  : tab.assMode === "Force" ? "Your style below goes on every file. Signs and karaoke can break."
                  : "Styled (ASS) files keep their look and take your size. Plain files take your style."
            Seg { options: ["As made", "Size only", "Mine"]; index: ["AsScripted", "ScaleOnly", "Force"].indexOf(tab.assMode); onPicked: function(i) { tab.edit(function(d) { d.ass_override = ["AsScripted", "ScaleOnly", "Force"][i] }) } } }
        SettingRow { label: "Font"
            Field { text: tab.ts ? tab.ts.font : ""; width: theme.space(40); invalid: !!tab.bad.font; onEdited: function(t) { if (tab.flag("font", t.trim() !== "")) tab.edit(function(d) { d.text_style.font = t.trim() }) } } }
        SettingRow { label: "Bold"
            Switch { checked: !!(tab.ts && tab.ts.bold); onToggled: function(on) { tab.edit(function(d) { d.text_style.bold = on }) } } }
        SettingRow { label: "Text colour"
            Row { spacing: theme.space(2)
                Corner { anchors.verticalCenter: parent.verticalCenter; width: theme.space(7); height: width; radius: width / 2; color: tab.ts ? tab.hexOf(tab.ts.colour) : "#ffffff"; borderColor: theme.lineStrong; borderWidth: 1 }
                Field { text: tab.ts ? tab.hexOf(tab.ts.colour) : ""; mono: true; width: theme.space(28); invalid: !!tab.bad.colour; onEdited: function(t) { if (tab.flag("colour", tab.isHex(t))) tab.edit(function(d) { d.text_style.colour = tab.colourOf(t) }) } } } }
        SettingRow { label: "Outline"; helper: "Its width, then its colour."
            Row { spacing: theme.space(2)
                Field { text: tab.ts ? String(tab.ts.outline_size) : ""; mono: true; width: theme.space(16); invalid: !!tab.bad.outline; onEdited: function(t) { var v = Number(t); if (tab.flag("outline", t.trim() !== "" && v >= 0)) tab.edit(function(d) { d.text_style.outline_size = v }) } }
                Corner { anchors.verticalCenter: parent.verticalCenter; width: theme.space(7); height: width; radius: width / 2; color: tab.ts ? tab.hexOf(tab.ts.outline_colour) : "#000000"; borderColor: theme.lineStrong; borderWidth: 1 }
                Field { text: tab.ts ? tab.hexOf(tab.ts.outline_colour) : ""; mono: true; width: theme.space(28); invalid: !!tab.bad.outlineColour; onEdited: function(t) { if (tab.flag("outlineColour", tab.isHex(t))) tab.edit(function(d) { d.text_style.outline_colour = tab.colourOf(t) }) } } } }
        SettingRow { label: "Shadow"; helper: "How far the shadow falls, 0 for none."
            Field { text: tab.ts ? String(tab.ts.shadow_offset) : ""; mono: true; width: theme.space(16); invalid: !!tab.bad.shadow; onEdited: function(t) { var v = Number(t); if (tab.flag("shadow", t.trim() !== "" && v >= 0)) tab.edit(function(d) { d.text_style.shadow_offset = v }) } } }
        SettingRow { label: "Background box"; helper: "A dark box behind the lines. 0 leaves it out."
            SliderRow { from: 0; to: 1; stepSize: 0.05; decimals: 2; value: tab.ts ? tab.ts.box_opacity : 0; onMoved: function(v) { tab.edit(function(d) { d.text_style.box_opacity = v }) } } }
        SettingRow { label: "Start over"; helper: "Puts the size, font, colours, outline, box and position back to how they came."
            Button { objectName: "subtitle-reset"; text: "Reset"; icon: "rotate-ccw"; flat: true; onClicked: tab.resetStyle() } }
    }
}
