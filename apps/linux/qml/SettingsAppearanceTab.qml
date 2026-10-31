// Appearance: the preview first, since it is what every knob below is for (the same sample
// page drawn once per mode, so the pair being chosen is what is shown), then Colours (mode,
// where the colours come from, the theme pair, the accent) and Size and motion (density,
// poster size, reduced motion). Every knob writes through the Theme singleton's pick*
// invokables; the engine pushes the fresh resolution back and the panes re-tone through
// their own Tokens, so nothing here polls.
import QtQuick
import QtQuick.Layouts
import com.marcusrosado.AniBeam

SettingsTab {
    id: tab
    title: "Appearance"
    lead: "Changes show at once, here and on every page. They live in ~/.config/anibeam/theme.toml, which you can edit by hand too."
    readonly property var darkThemes: Theme.themes.filter(function(t) { return t.mode === "dark" })
    readonly property var lightThemes: Theme.themes.filter(function(t) { return t.mode === "light" })
    function indexOf(list, stem) { for (var i = 0; i < list.length; i++) if (list[i].stem === stem) return i; return 0 }
    readonly property bool fromTheme: Theme.source === "theme"

    // The preview's sample cards carry relative labels (last-viewed, countdowns); this
    // tick keeps them fresh. It only runs while this section exists.
    property real nowMs: Date.now()
    Timer { interval: 30000; running: true; repeat: true; onTriggered: tab.nowMs = Date.now() }

    Panel {
        bare: true
        stretch: LookPreview { nowMs: tab.nowMs }
    }
    Panel {
        title: "Colours"
        SettingRow { label: "Mode"
            Seg { options: [{ text: "Dark", icon: "moon" }, { text: "Light", icon: "sun" }, { text: "System", icon: "monitor" }]
                index: ["dark", "light", "system"].indexOf(Theme.mode); onPicked: function(i) { Theme.pickMode(["dark", "light", "system"][i]) } } }
        SettingRow { label: "Colours from"
            helper: tab.fromTheme ? "The theme pair below, one for dark and one for light."
                                  : "Your terminal's colours, else the desktop's colour scheme and accent."
            Seg { objectName: "colour-source"; options: ["Desktop", "Theme"]; index: tab.fromTheme ? 1 : 0; onPicked: function(i) { Theme.pickSource(i === 1 ? "theme" : "system") } } }
        SettingRow { visible: tab.fromTheme; label: "Dark theme"
            Dropdown { objectName: "theme-dark"; options: tab.darkThemes.map(function(t) { return t.name }); index: tab.indexOf(tab.darkThemes, Theme.themeDark)
                onPicked: function(i) { Theme.pickTheme("dark", tab.darkThemes[i].stem) } } }
        SettingRow { visible: tab.fromTheme; label: "Light theme"; helper: "Theme files you drop in ~/.config/anibeam/themes show up in both lists."
            Dropdown { objectName: "theme-light"; options: tab.lightThemes.map(function(t) { return t.name }); index: tab.indexOf(tab.lightThemes, Theme.themeLight)
                onPicked: function(i) { Theme.pickTheme("light", tab.lightThemes[i].stem) } } }
        SettingRow { label: "Accent"; helper: "Marks what is selected, playing or new."
            Swatches { slot: Theme.accent; onPicked: function(s) { Theme.pickAccent(s) } } }
    }
    Panel {
        title: "Size and motion"
        SettingRow { label: "Density"; helper: "The room around everything, from tight to airy."
            Seg { options: ["Compact", "Normal", "Comfortable"]; index: ["compact", "normal", "comfortable"].indexOf(Theme.density); onPicked: function(i) { Theme.pickDensity(["compact", "normal", "comfortable"][i]) } } }
        SettingRow { label: "Poster size"; helper: "How big a series is in the library and on Home."
            Seg { options: ["Small", "Medium", "Large"]; index: ["s", "m", "l"].indexOf(Theme.poster); onPicked: function(i) { Theme.pickPoster(["s", "m", "l"][i]) } } }
        SettingRow { label: "Reduce motion"; helper: "Skips the library's opening sweep and other movement that is only for show."
            Switch { objectName: "reduce-motion"; checked: Theme.reducedMotion; onToggled: function(on) { Theme.pickReducedMotion(on) } } }
    }
}
