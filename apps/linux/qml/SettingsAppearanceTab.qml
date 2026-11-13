// Appearance: Colours (mode, where the colours come from, the theme pair, the accent), then
// the preview right under them (the same sample page drawn once per mode, so the pair being
// chosen is what is shown, in view of the knobs that change it), then Size and motion
// (density, poster size, reduced motion). Every knob writes through the Theme singleton's pick*
// invokables; the engine pushes the fresh resolution back and the panes re-tone through
// their own Tokens, so nothing here polls.
import QtQuick
import QtQuick.Layouts
import QtCore
import com.marcusrosado.AniBeam

SettingsTab {
    // The text sizes, as theme.toml names them and as the row says them
    readonly property var textSizes: [{ id: "smaller", msg: "settings-appearance-text-smaller" }, { id: "normal", msg: "settings-appearance-text-normal" }, { id: "large", msg: "settings-appearance-text-large" },
                                      { id: "larger", msg: "settings-appearance-text-larger" }, { id: "largest", msg: "settings-appearance-text-largest" }]
    id: tab
    title: words.t("settings-appearance-title")
    lead: words.t("settings-appearance-lead")
    readonly property var darkThemes: Theme.themes.filter(function(t) { return t.mode === "dark" })
    readonly property var lightThemes: Theme.themes.filter(function(t) { return t.mode === "light" })
    function indexOf(list, stem) { for (var i = 0; i < list.length; i++) if (list[i].stem === stem) return i; return 0 }
    readonly property bool fromTheme: Theme.source === "theme"
    // Out now on Airing: cards or a list, and the cards' grid, columns by rows; AiringPage
    // reads the same keys
    Settings { id: airingKept; location: Shell.settingsUrl; category: "airing"; property string outLayout: "4x2"; property string outStyle: "cards" }
    readonly property var outLayouts: ["3x2", "4x2", "5x2", "4x3"]

    // The preview's sample cards carry relative labels (last-viewed, countdowns); this
    // tick keeps them fresh. It only runs while this section exists.
    property real nowMs: Date.now()
    Timer { interval: 30000; running: true; repeat: true; onTriggered: tab.nowMs = Date.now() }

    Panel {
        title: words.t("settings-appearance-colours")
        SettingRow { label: words.t("settings-appearance-mode")
            Seg { options: [{ text: words.t("settings-appearance-mode-dark"), icon: "moon" }, { text: words.t("settings-appearance-mode-light"), icon: "sun" }, { text: words.t("settings-appearance-mode-system"), icon: "monitor" }]
                index: ["dark", "light", "system"].indexOf(Theme.mode); onPicked: function(i) { Theme.pickMode(["dark", "light", "system"][i]) } } }
        SettingRow { label: words.t("settings-appearance-colours-from")
            helper: tab.fromTheme ? words.t("settings-appearance-from-theme-helper")
                                  : words.t("settings-appearance-from-desktop-helper")
            Seg { objectName: "colour-source"; options: [words.t("settings-appearance-from-desktop"), words.t("settings-appearance-from-theme")]; index: tab.fromTheme ? 1 : 0; onPicked: function(i) { Theme.pickSource(i === 1 ? "theme" : "system") } } }
        SettingRow { visible: tab.fromTheme; label: words.t("settings-appearance-dark-theme")
            Dropdown { objectName: "theme-dark"; options: tab.darkThemes.map(function(t) { return t.name }); index: tab.indexOf(tab.darkThemes, Theme.themeDark)
                onPicked: function(i) { Theme.pickTheme("dark", tab.darkThemes[i].stem) } } }
        SettingRow { visible: tab.fromTheme; label: words.t("settings-appearance-light-theme"); helper: words.t("settings-appearance-themes-helper")
            Dropdown { objectName: "theme-light"; options: tab.lightThemes.map(function(t) { return t.name }); index: tab.indexOf(tab.lightThemes, Theme.themeLight)
                onPicked: function(i) { Theme.pickTheme("light", tab.lightThemes[i].stem) } } }
        SettingRow { label: words.t("settings-appearance-accent"); helper: words.t("settings-appearance-accent-helper")
            Swatches { slot: Theme.accent; onPicked: function(s) { Theme.pickAccent(s) } } }
    }
    Panel {
        bare: true
        stretch: LookPreview { nowMs: tab.nowMs }
    }
    Panel {
        title: words.t("settings-appearance-icon")
        SettingRow { label: words.t("settings-appearance-app-icon"); helper: words.t("settings-appearance-app-icon-helper")
            Row {
                spacing: theme.space(3)
                Repeater {
                    model: [{ id: "default", msg: "settings-appearance-icon-default", nameMsg: "settings-appearance-icon-default-name", file: "icon.png" }, { id: "dark", msg: "settings-appearance-icon-dark", nameMsg: "settings-appearance-icon-dark-name", file: "icon-dark.png" }]
                    Column {
                        id: pick
                        required property var modelData
                        readonly property bool on: Theme.appIcon === modelData.id
                        objectName: "app-icon-" + modelData.id   // a --drive target
                        spacing: theme.space(1.5)
                        activeFocusOnTab: true
                        Keys.onReturnPressed: Theme.pickAppIcon(modelData.id)
                        Keys.onSpacePressed: Theme.pickAppIcon(modelData.id)
                        Item {
                            width: theme.space(16); height: width
                            Corner {
                                anchors.fill: parent
                                radius: width * 0.26
                                smoothing: theme.cornerSmoothing
                                borderColor: pick.activeFocus ? theme.focusRing : pick.on ? theme.accent : (m.containsMouse ? theme.lineStrong : "transparent")
                                borderWidth: theme.space(0.5)
                            }
                            Image {
                                anchors.fill: parent
                                anchors.margins: theme.space(1.5)
                                source: "qrc:/qt/qml/com/marcusrosado/AniBeam/assets/" + pick.modelData.file
                                sourceSize: Qt.size(128, 128)
                                smooth: true
                            }
                            MouseArea {
                                Accessible.role: Accessible.RadioButton
                                Accessible.onPressAction: Theme.pickAppIcon(pick.modelData.id)
                                Accessible.name: words.t(pick.modelData.nameMsg)
                                Accessible.checkable: true
                                Accessible.checked: pick.on
                                id: m
                                anchors.fill: parent
                                hoverEnabled: true
                                cursorShape: Qt.PointingHandCursor
                                onClicked: Theme.pickAppIcon(pick.modelData.id)
                            }
                        }
                        Text {
                            anchors.horizontalCenter: parent.horizontalCenter
                            text: words.t(pick.modelData.msg)
                            color: pick.on ? theme.text : theme.textDim
                            font.family: theme.fontUi
                            font.pointSize: theme.typeSmall
                        }
                    }
                }
            }
        }
    }
    Panel {
        title: words.t("settings-appearance-size-motion")
        SettingRow { label: words.t("settings-appearance-text-size"); helper: words.t("settings-appearance-text-size-helper")
            Seg { objectName: "text-size"; options: tab.textSizes.map(function(s) { return words.t(s.msg) })
                  index: Math.max(0, tab.textSizes.map(function(s) { return s.id }).indexOf(Theme.textSize))
                  onPicked: function(i) { Theme.pickTextSize(tab.textSizes[i].id) } } }
        SettingRow { label: words.t("settings-appearance-density"); helper: words.t("settings-appearance-density-helper")
            Seg { options: [words.t("settings-appearance-density-compact"), words.t("settings-appearance-density-normal"), words.t("settings-appearance-density-comfortable")]; index: ["compact", "normal", "comfortable"].indexOf(Theme.density); onPicked: function(i) { Theme.pickDensity(["compact", "normal", "comfortable"][i]) } } }
        SettingRow { label: words.t("settings-appearance-poster-size"); helper: words.t("settings-appearance-poster-size-helper")
            Seg { options: [words.t("settings-appearance-poster-small"), words.t("settings-appearance-poster-medium"), words.t("settings-appearance-poster-large")]; index: ["s", "m", "l"].indexOf(Theme.poster); onPicked: function(i) { Theme.pickPoster(["s", "m", "l"][i]) } } }
        SettingRow { label: words.t("settings-appearance-out-now"); helper: words.t("settings-appearance-out-now-helper")
            Seg { objectName: "out-now-style"; options: [words.t("settings-appearance-out-now-cards"), words.t("settings-appearance-out-now-list")]; index: airingKept.outStyle === "list" ? 1 : 0; onPicked: function(i) { airingKept.outStyle = i === 1 ? "list" : "cards" } } }
        SettingRow { visible: airingKept.outStyle !== "list"; label: words.t("settings-appearance-out-now-grid"); helper: words.t("settings-appearance-out-now-grid-helper")
            Seg { objectName: "out-now-layout"; options: tab.outLayouts.map(function(l) { var cr = l.split("x"); return words.t("settings-appearance-grid", { columns: Number(cr[0]), rows: Number(cr[1]) }) }); index: Math.max(0, tab.outLayouts.indexOf(airingKept.outLayout)); onPicked: function(i) { airingKept.outLayout = tab.outLayouts[i] } } }
        SettingRow { label: words.t("settings-appearance-reduce-motion"); helper: words.t("settings-appearance-reduce-motion-helper")
            Switch { objectName: "reduce-motion"; checked: Theme.reducedMotion; onToggled: function(on) { Theme.pickReducedMotion(on) } } }
    }
}
