// Spec 4.5: Colours (mode, colour source, the theme pair, the accent), Shape (density,
// poster size, corners), and the two preview panes, each holding its own tokens forced to
// a mode so the pair being chosen is what is shown. Every knob writes through the Theme
// singleton's pick* invokables; the engine pushes the fresh resolution back and the panes
// re-tone through their own Tokens, so nothing here polls.
import QtQuick
import com.marcusrosado.AniBeam

SettingsTab {
    id: tab
    readonly property var darkThemes: Theme.themes.filter(function(t) { return t.mode === "dark" })
    readonly property var lightThemes: Theme.themes.filter(function(t) { return t.mode === "light" })
    function indexOf(list, stem) { for (var i = 0; i < list.length; i++) if (list[i].stem === stem) return i; return 0 }

    // The preview's sample cards carry relative labels (last-viewed, countdowns); this
    // tick keeps them fresh, the same 30 second idiom LibraryPage.qml uses. It only runs
    // while this tab exists, since the Loader that owns it destroys it on tab switch.
    property real nowMs: Date.now()
    Timer { interval: 30000; running: true; repeat: true; onTriggered: tab.nowMs = Date.now() }

    Component {
        id: cornerGlyph
        Corner {
            width: theme.space(5); height: width
            radius: width * 0.45
            smoothing: option.smoothing
            borderColor: tint; borderWidth: 1.5; color: "transparent"
        }
    }

    SettingsPair {
        split: 1 / 3
        leftPanels: [ coloursPanel, shapePanel, footNote ]
        rightPanels: [ previewPanel ]
    }
    Component {
        id: coloursPanel
        Panel {
            title: "Colours"; icon: "palette"
            SettingRow { label: "Mode"
                Seg { options: [{ text: "Dark", icon: "moon" }, { text: "Light", icon: "sun" }, { text: "System", icon: "monitor" }]
                    index: ["dark", "light", "system"].indexOf(Theme.mode); onPicked: function(i) { Theme.pickMode(["dark", "light", "system"][i]) } } }
            SettingRow { label: "Colour source"; helper: "System reads your terminal's colours, or the desktop's scheme and accent when it finds no terminal config."
                Seg { options: ["System", "Theme"]; index: Theme.source === "theme" ? 1 : 0; onPicked: function(i) { Theme.pickSource(i === 1 ? "theme" : "system") } } }
            SettingRow { label: "Dark theme"; opacity: Theme.source === "theme" ? 1 : theme.disabledOpacity
                Dropdown { options: tab.darkThemes.map(function(t) { return t.name }); index: tab.indexOf(tab.darkThemes, Theme.themeDark); interactive: Theme.source === "theme"
                    onPicked: function(i) { Theme.pickTheme("dark", tab.darkThemes[i].stem) } } }
            SettingRow { label: "Light theme"; helper: "Base16 and kitty files in ~/.config/anibeam/themes appear here."; opacity: Theme.source === "theme" ? 1 : theme.disabledOpacity
                Dropdown { options: tab.lightThemes.map(function(t) { return t.name }); index: tab.indexOf(tab.lightThemes, Theme.themeLight); interactive: Theme.source === "theme"
                    onPicked: function(i) { Theme.pickTheme("light", tab.lightThemes[i].stem) } } }
            SettingRow { label: "Accent"
                Swatches { slot: Theme.accent; onPicked: function(s) { Theme.pickAccent(s) } } }
        }
    }
    Component {
        id: shapePanel
        Panel {
            title: "Shape"; icon: "shapes"; grows: true
            SettingRow { label: "Density"
                Seg { options: ["Compact", "Normal", "Comfortable"]; index: ["compact", "normal", "comfortable"].indexOf(Theme.density); onPicked: function(i) { Theme.pickDensity(["compact", "normal", "comfortable"][i]) } } }
            SettingRow { label: "Poster size"
                Seg { options: ["S", "M", "L"]; index: ["s", "m", "l"].indexOf(Theme.poster); onPicked: function(i) { Theme.pickPoster(["s", "m", "l"][i]) } } }
            SettingRow { label: "Corners"
                Seg { options: [{ text: "Smooth", delegate: cornerGlyph, smoothing: 0.6 }, { text: "Plain", delegate: cornerGlyph, smoothing: 0 }]
                    index: Theme.corners === "plain" ? 1 : 0; onPicked: function(i) { Theme.pickCorners(i === 1 ? "plain" : "smooth") } } }
        }
    }
    Component { id: footNote; Note { text: "All of this lives in ~/.config/anibeam/theme.toml and reloads when the file changes." } }
    Component {
        id: previewPanel
        Panel {
            title: "Preview"; icon: "eye"; grows: true
            stretch: LookPreview { nowMs: tab.nowMs }
        }
    }
}
