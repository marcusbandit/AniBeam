// The settings page: one scrolling column of sections. The Look section drives the real
// theme knobs; everything else is fake state so the copy and the controls can be judged.
import QtQuick
import QtQuick.Controls.Basic as QC

Item {
    id: root
    property real footInset: theme.space(10)
    property alias scrollY: flick.contentY
    property bool demoConfirm: false     // preset confirm=1 opens the first source's Remove question

    readonly property var themeList: theme.palettes.themes || []
    readonly property var darkThemes: themeList.filter(function(t) { return t.variant === "dark" })
    readonly property var lightThemes: themeList.filter(function(t) { return t.variant === "light" })
    function slugIndex(list, slug) { for (var i = 0; i < list.length; i++) if (list[i].slug === slug) return i; return 0 }
    function isHex(t) { return /^#[0-9a-fA-F]{6}$/.test(t) }
    // Subtitle style values are data, not chrome: the outline swatch shows the stored colour
    property color outlineColour: "#000000"

    // A quiet line of copy that belongs to no single row
    component Note: Text {
        width: parent.width
        color: theme.textDim
        wrapMode: Text.Wrap
        font.family: theme.fontSans
        font.pointSize: theme.typeSmall
    }

    // The question and its two answers that replace a row's controls until answered
    component Confirm: Row {
        id: confirm
        property string question: ""
        property string yes: "Remove"
        property real maxWidth: theme.space(100)
        signal accepted()
        signal kept()
        spacing: theme.space(2)
        Text {
            anchors.verticalCenter: parent.verticalCenter
            width: Math.min(implicitWidth, confirm.maxWidth - yesButton.width - keepButton.width - confirm.spacing * 2)
            text: confirm.question
            color: theme.text
            wrapMode: Text.Wrap
            font.family: theme.fontSans
            font.pointSize: theme.typeNormal
        }
        Button { id: yesButton; anchors.verticalCenter: parent.verticalCenter; text: confirm.yes; danger: true; onClicked: confirm.accepted() }
        Button { id: keepButton; anchors.verticalCenter: parent.verticalCenter; text: "Keep"; flat: true; onClicked: confirm.kept() }
    }

    // One library source: the path, its facts, Rescan and Remove; Remove asks first
    component SourceRow: Item {
        id: source
        property string path: ""
        property string meta: ""
        property bool available: true
        property string question: ""
        property bool confirming: false
        width: parent.width
        height: Math.max(theme.space(12), controls.height + theme.space(3) * 2)
        Keys.onEscapePressed: confirming = false

        Column {
            id: words
            anchors.left: parent.left
            anchors.leftMargin: theme.space(4)
            anchors.verticalCenter: parent.verticalCenter
            spacing: theme.space(0.5)
            Row {
                spacing: theme.space(2)
                Text {
                    anchors.verticalCenter: parent.verticalCenter
                    text: source.path
                    color: source.available ? theme.text : theme.textDim
                    font.family: theme.fontMono
                    font.pointSize: theme.typeNormal
                }
                Chip {
                    anchors.verticalCenter: parent.verticalCenter
                    visible: !source.available
                    text: "Unavailable"
                    small: true
                    mono: false
                    color: theme.surfaceSunken
                    textColor: theme.textDim
                }
            }
            Text {
                visible: source.meta !== ""
                text: source.meta
                color: theme.textDim
                font.family: theme.fontSans
                font.pointSize: theme.typeSmall
            }
        }
        Row {
            id: controls
            anchors.right: parent.right
            anchors.rightMargin: theme.space(3)
            anchors.verticalCenter: parent.verticalCenter
            Row {
                visible: !source.confirming
                spacing: theme.space(1)
                Button { text: "Rescan"; flat: true }
                Button { text: "Remove"; flat: true; onClicked: { source.confirming = true; source.forceActiveFocus() } }
            }
            Confirm {
                visible: source.confirming
                question: source.question
                maxWidth: source.width - words.width - theme.space(4) - theme.space(3) - theme.space(6)
                onAccepted: source.confirming = false
                onKept: source.confirming = false
            }
        }
    }

    Flickable {
        id: flick
        anchors.fill: parent
        contentWidth: width
        contentHeight: col.y + col.height + root.footInset
        clip: true
        boundsBehavior: Flickable.StopAtBounds

        Column {
            id: col
            y: theme.space(7)
            width: Math.min(flick.width, theme.space(176))
            spacing: theme.space(8)

            Text {
                text: "Settings"
                color: theme.text
                font.family: theme.fontSans
                font.pointSize: theme.typeLarge
                font.weight: Font.Bold
            }

            Section {
                title: "Library"
                Item { width: parent.width; height: theme.space(1) }
                Corner {
                    width: parent.width
                    height: sources.height
                    radius: theme.radiusMd
                    smoothing: theme.cornerSmoothing
                    color: theme.surface
                    borderColor: theme.line
                    borderWidth: 1
                    Column {
                        id: sources
                        width: parent.width
                        SourceRow { path: "/mnt/media/Anime"; meta: "412 series · 2 movie folders"; question: "Remove Anime, 412 series and their history?"; confirming: root.demoConfirm }
                        Rectangle { width: parent.width - theme.space(4) * 2; anchors.horizontalCenter: parent.horizontalCenter; height: 1; color: theme.line }
                        SourceRow { path: "/mnt/media/Films"; available: false; question: "Remove Films and its history?" }
                    }
                }
                Item { width: parent.width; height: theme.space(1) }
                Row {
                    spacing: theme.space(2)
                    Button { text: "Add folder" }
                    Button { text: "Scan all" }
                }
                Note { text: "AniBeam scans these folders for video files. A folder is a series; a file at the top level of a Movies folder is a film." }
                Item { width: parent.width; height: theme.space(1) }
                SettingRow {
                    label: "Show hidden shows"
                    helper: "Shows hidden series on every page until AniBeam closes."
                    Switch {}
                }
                SettingRow {
                    label: "Subscriptions"
                    helper: "The feeds anirss watches for you."
                    Button { text: "Open" }
                }
            }

            Section {
                title: "Trackers"
                helper: "Episodes are marked on every connected tracker when you reach the outro or mark them by hand. Counts only go up."
                SettingRow {
                    label: "AniList"
                    line: "Connected as marcusbandit · synced 4 min ago"
                    Button { text: "Disconnect" }
                }
                SettingRow {
                    label: "MyAnimeList"
                    line: "Not connected"
                    Button { text: "Log in" }
                }
                SettingRow {
                    id: mainTracker
                    property int choice: 0
                    label: "Main tracker"
                    helper: "Whose count the cards show. The other tracker still receives every mark."
                    Seg { options: ["AniList", "MyAnimeList"]; index: mainTracker.choice; onPicked: function(i) { mainTracker.choice = i } }
                }
            }

            Section {
                title: "Look"
                SettingRow {
                    label: "Mode"
                    Seg {
                        options: ["Dark", "Light", "System"]
                        index: theme.mode === "dark" ? 0 : theme.mode === "light" ? 1 : 2
                        onPicked: function(i) { theme.mode = ["dark", "light", "system"][i] }
                    }
                }
                SettingRow {
                    label: "Colour source"
                    helper: "System reads your terminal's colours, or the desktop's scheme and accent when it finds no terminal config."
                    Seg {
                        options: ["System", "Theme"]
                        index: theme.colourSource === "theme" ? 1 : 0
                        onPicked: function(i) { theme.colourSource = i === 1 ? "theme" : "system" }
                    }
                }
                SettingRow {
                    label: "Dark theme"
                    opacity: theme.colourSource === "system" ? theme.disabledOpacity : 1
                    enabled: theme.colourSource !== "system"
                    Dropdown {
                        options: root.darkThemes.map(function(t) { return t.name })
                        index: root.slugIndex(root.darkThemes, theme.themeDark)
                        onPicked: function(i) { theme.themeDark = root.darkThemes[i].slug }
                    }
                }
                SettingRow {
                    label: "Light theme"
                    opacity: theme.colourSource === "system" ? theme.disabledOpacity : 1
                    enabled: theme.colourSource !== "system"
                    Dropdown {
                        options: root.lightThemes.map(function(t) { return t.name })
                        index: root.slugIndex(root.lightThemes, theme.themeLight)
                        onPicked: function(i) { theme.themeLight = root.lightThemes[i].slug }
                    }
                }
                Note { text: "Base16 and kitty files in ~/.config/anibeam/themes appear here." }
                SettingRow {
                    label: "Accent"
                    opacity: theme.colourSource === "theme" ? theme.disabledOpacity : 1
                    enabled: theme.colourSource !== "theme"
                    Swatches { slot: theme.accentSlot; onPicked: function(s) { theme.accentSlot = s } }
                }
                SettingRow {
                    label: "Density"
                    Seg {
                        options: ["Compact", "Normal", "Comfortable"]
                        index: theme.density === "compact" ? 0 : theme.density === "comfortable" ? 2 : 1
                        onPicked: function(i) { theme.density = ["compact", "normal", "comfortable"][i] }
                    }
                }
                SettingRow {
                    label: "Poster size"
                    Seg {
                        options: ["S", "M", "L"]
                        index: theme.posterWidth <= 140 ? 0 : theme.posterWidth >= 240 ? 2 : 1
                        onPicked: function(i) { theme.posterWidth = [140, 180, 240][i] }
                    }
                }
                SettingRow {
                    label: "Corners"
                    Seg {
                        options: ["Smooth", "Plain"]
                        index: theme.cornerSmoothing > 0 ? 0 : 1
                        onPicked: function(i) { theme.cornerSmoothing = i === 0 ? 0.6 : 0 }
                    }
                }
                Note { text: "All of this lives in ~/.config/anibeam/theme.toml and reloads when the file changes." }
            }

            Section {
                title: "Playback"
                SettingRow {
                    label: "Auto-skip intro"
                    helper: "Jumps the intro when the file's chapters or AniSkip know where it is. Undo in the player turns it off for the session."
                    Switch {}
                }
                SettingRow {
                    label: "Auto-skip outro"
                    helper: "Jumps the outro when the file's chapters or AniSkip know where it is. Undo in the player turns it off for the session."
                    Switch {}
                }
                SettingRow {
                    label: "Use my mpv.conf"
                    helper: "Loads ~/.config/mpv/mpv.conf under AniBeam's own settings. Lines that only apply at start-up, scripts, input-conf and config-dir, are ignored, and no script ever loads."
                    Switch {}
                }
            }

            Section {
                title: "Subtitle defaults"
                helper: "What every session starts from. Change tracks in the player and AniBeam remembers them per series."
                SettingRow {
                    label: "Subtitle languages"
                    Field { text: "en"; implicitWidth: theme.space(30) }
                }
                SettingRow {
                    label: "Audio languages"
                    Field { text: "ja"; implicitWidth: theme.space(30) }
                }
                Note { text: "Comma separated, first match wins." }
                SettingRow {
                    id: scaleRow
                    property real scale: 1
                    label: "Scale"
                    SliderRow { from: 0.5; to: 2.0; stepSize: 0.05; decimals: 2; value: scaleRow.scale; onMoved: function(v) { scaleRow.scale = v } }
                }
                SettingRow {
                    id: assRow
                    property int choice: 0
                    label: "ASS override"
                    helper: "Force applies the text style to styled subtitles and may break signs and karaoke."
                    Seg { options: ["As scripted", "Scale only", "Force"]; index: assRow.choice; onPicked: function(i) { assRow.choice = i } }
                }
                Item { width: parent.width; height: theme.space(2) }
                Note { text: "TEXT STYLE, FOR SRT AND VTT"; font.letterSpacing: 1 }
                SettingRow {
                    label: "Font"
                    Field { text: "sans-serif"; implicitWidth: theme.space(30) }
                }
                SettingRow {
                    label: "Colour"
                    Row {
                        spacing: theme.space(2)
                        Corner {
                            anchors.verticalCenter: parent.verticalCenter
                            width: theme.space(6); height: width
                            radius: theme.radiusSm
                            smoothing: theme.cornerSmoothing
                            color: root.isHex(textColour.text) ? textColour.text : "transparent"
                            borderColor: theme.line
                            borderWidth: 1
                        }
                        Field { id: textColour; text: "#FFFFFF"; mono: true; implicitWidth: theme.space(28) }
                    }
                }
                SettingRow {
                    label: "Outline"
                    Row {
                        spacing: theme.space(2)
                        Field { text: "1.65"; mono: true; implicitWidth: theme.space(20) }
                        Corner {
                            anchors.verticalCenter: parent.verticalCenter
                            width: theme.space(6); height: width
                            radius: theme.radiusSm
                            smoothing: theme.cornerSmoothing
                            color: root.outlineColour
                            borderColor: theme.line
                            borderWidth: 1
                        }
                    }
                }
                SettingRow {
                    label: "Shadow"
                    Field { text: "0"; mono: true; implicitWidth: theme.space(20) }
                }
                SettingRow {
                    label: "Box opacity"
                    Field { text: "0"; mono: true; implicitWidth: theme.space(20) }
                }
                SettingRow {
                    label: "Bold"
                    Switch {}
                }
                SettingRow {
                    id: positionRow
                    property real pos: 100
                    label: "Position"
                    SliderRow { from: 0; to: 150; stepSize: 1; decimals: 0; value: positionRow.pos; onMoved: function(v) { positionRow.pos = v } }
                }
            }

            Section {
                title: "Storage"
                SettingRow {
                    id: imagesRow
                    property bool confirming: false
                    label: "Images"
                    line: "1,204 files · 312 MB"
                    helper: "Posters come back on the next launch."
                    Keys.onEscapePressed: confirming = false
                    Row {
                        Button { visible: !imagesRow.confirming; text: "Clear images"; onClicked: { imagesRow.confirming = true; imagesRow.forceActiveFocus() } }
                        Confirm {
                            visible: imagesRow.confirming
                            question: "Clear 1,204 images?"
                            yes: "Clear"
                            maxWidth: imagesRow.width * 0.6
                            onAccepted: imagesRow.confirming = false
                            onKept: imagesRow.confirming = false
                        }
                    }
                }
                SettingRow { label: "Database"; line: "~/.local/share/anibeam/anibeam.db" }
                SettingRow { label: "Data"; line: "~/.local/share/anibeam" }
                SettingRow { label: "Config"; line: "~/.config/anibeam" }
                SettingRow { label: "Cache"; line: "~/.cache/anibeam" }
            }

            Section {
                title: "Export and import"
                SettingRow {
                    label: "Include private data"
                    helper: "Tracker logins, API keys, watch history and preferences, in plain text."
                    Switch { id: privateData }
                }
                SettingRow {
                    label: "Export"
                    helper: privateData.checked ? "Writes anibeam-export-full-<date>.json." : "Writes anibeam-export-<date>.json."
                    Button { text: "Export" }
                }
                SettingRow {
                    label: "Import"
                    helper: "Merges a file into this library. The file wins for matches and accounts, the newer entry wins for history, nothing is deleted."
                    Button { text: "Import" }
                }
            }
        }

        QC.ScrollBar.vertical: QC.ScrollBar {
            policy: QC.ScrollBar.AsNeeded
            contentItem: Corner { implicitWidth: theme.space(1); radius: implicitWidth / 2; smoothing: theme.cornerSmoothing; color: theme.lineStrong; opacity: parent.active ? 1 : 0.4 }
        }
    }
}
