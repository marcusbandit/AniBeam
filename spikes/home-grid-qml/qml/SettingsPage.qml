// The settings page: a title, a Seg of four tabs, and a Flickable per tab. Each tab lays its
// panels out in two columns that fill the page together, one under the other when the page
// is too narrow for both. A tab fills the viewport: every column is a ColumnLayout that is
// as tall as the viewport when its content fits, and one panel per column, marked `grows`,
// takes the spare height; a panel is never shorter than its content, so when the natural
// height passes the viewport (the rotated portrait monitor) the Flickable scrolls instead.
// The Appearance tab drives the real theme knobs and shows both modes beside them; the
// Playback tab draws the subtitle style over a still; everything else is fake state so the
// copy and the controls can be judged. The current tab is remembered while the app runs.
// The page is a FocusScope: a press on empty space takes the focus off whatever control
// had it.
import QtQuick
import QtQuick.Layouts
import QtQuick.Controls.Basic as QC

FocusScope {
    id: root
    property real footInset: theme.space(10)
    property var library: []
    property string titleLang: "jp"
    property bool demoConfirm: false     // preset confirm=1 opens the first source's Remove question

    readonly property var tabNames: ["Library", "Appearance", "Playback", "Data"]
    readonly property var tabIcons: ["folder", "palette", "play", "hard-drive"]
    property int tab: 0
    readonly property var tabs: [libraryTab, lookTab, playbackTab, dataTab]
    readonly property Flickable current: tabs[tab]
    // preset scroll=<px> lands on the tab shown at that moment
    property real scrollY: 0
    onScrollYChanged: current.contentY = scrollY

    // The block the header and the panels share: the page's width up to a cap, centred when
    // the page is wider, so a row's label and its control never drift metres apart
    readonly property real gap: theme.space(6)
    readonly property real maxWidth: theme.space(560)
    readonly property real blockWidth: Math.min(width, maxWidth)
    readonly property real blockX: Math.round((width - blockWidth) / 2)

    readonly property var themeList: theme.palettes.themes || []
    readonly property var darkThemes: themeList.filter(function(t) { return t.variant === "dark" })
    readonly property var lightThemes: themeList.filter(function(t) { return t.variant === "light" })
    function slugIndex(list, slug) { for (var i = 0; i < list.length; i++) if (list[i].slug === slug) return i; return 0 }
    function isHex(t) { return /^#[0-9a-fA-F]{6}$/.test(t) }
    function num(t, d) { var v = parseFloat(t); return isNaN(v) ? d : v }
    // Whether any item in a column asks for the spare height (a Panel or a Pair with `grows`)
    function anyGrows(items) { for (var i = 0; i < items.length; i++) if (items[i].grows === true) return true; return false }
    // Subtitle style values are data, not chrome: the outline swatch shows the stored colour
    property color outlineColour: "#000000"
    // The eight hues, in the order the swatches show them
    readonly property var hues: [theme.red, theme.orange, theme.yellow, theme.green, theme.cyan, theme.blue, theme.purple, theme.brown]

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
        Button { id: yesButton; anchors.verticalCenter: parent.verticalCenter; text: confirm.yes; icon: "trash-2"; danger: true; onClicked: confirm.accepted() }
        Button { id: keepButton; anchors.verticalCenter: parent.verticalCenter; text: "Keep"; flat: true; onClicked: confirm.kept() }
    }

    // A row of stat tiles: a big fixed-face number over a caption, as many per line as the
    // width holds at theme.space(36) a tile, the lines evened out so four tiles never split
    // three and one
    component Tiles: Grid {
        id: tiles
        property var stats: []                 // [{ value, label }]
        readonly property real gap: theme.space(2)
        readonly property real minTile: theme.space(36)
        readonly property int fit: Math.max(1, Math.floor((width + gap) / (minTile + gap)))
        readonly property int lines: Math.max(1, Math.ceil(stats.length / fit))
        readonly property real tileWidth: (width - (columns - 1) * gap) / columns
        width: parent ? parent.width : theme.space(100)
        columns: Math.max(1, Math.ceil(stats.length / lines))
        columnSpacing: gap
        rowSpacing: gap
        Repeater {
            model: tiles.stats
            Corner {
                required property var modelData
                width: tiles.tileWidth
                height: words.height + theme.space(3) * 2
                radius: theme.radiusMd
                smoothing: theme.cornerSmoothing
                color: theme.surfaceSunken
                borderColor: theme.line
                borderWidth: 1
                Column {
                    id: words
                    x: theme.space(4)
                    anchors.verticalCenter: parent.verticalCenter
                    width: parent.width - theme.space(4) * 2
                    spacing: theme.space(0.5)
                    Text {
                        text: modelData.value
                        color: theme.text
                        font.family: theme.fontMono
                        font.pointSize: theme.typeLarge
                        font.weight: Font.Bold
                    }
                    Text {
                        width: parent.width
                        text: modelData.label
                        color: theme.textDim
                        elide: Text.ElideRight
                        font.family: theme.fontSans
                        font.pointSize: theme.typeSmall
                    }
                }
            }
        }
    }

    // A pill split into segments by share, with a legend line under it
    component UsageBar: Column {
        id: usage
        property var parts: []                 // [{ label, amount, text, color }]
        readonly property real total: parts.reduce(function(s, p) { return s + p.amount }, 0)
        readonly property real segGap: theme.space(0.5)
        width: parent ? parent.width : theme.space(100)
        spacing: theme.space(2)
        Item {
            width: parent.width
            height: theme.space(2)
            Corner {
                anchors.fill: parent
                radius: height / 2
                smoothing: theme.cornerSmoothing
                color: theme.surfaceSunken
                borderColor: theme.line
                borderWidth: 1
            }
            Row {
                id: segments
                anchors.fill: parent
                spacing: usage.segGap
                // Each part's share of the room; a share too thin to see is widened to a dot
                // the bar's height and the widest part gives that width back
                readonly property var widths: {
                    var n = usage.parts.length
                    var room = width - usage.segGap * (n - 1)
                    var w = usage.parts.map(function(p) { return usage.total > 0 ? room * p.amount / usage.total : 0 })
                    var owed = 0, big = 0
                    for (var i = 0; i < n; i++) {
                        if (w[i] > w[big]) big = i
                        if (w[i] < height) { owed += height - w[i]; w[i] = height }
                    }
                    if (n > 0) w[big] = Math.max(height, w[big] - owed)
                    return w
                }
                Repeater {
                    model: usage.parts
                    Corner {
                        required property int index
                        required property var modelData
                        width: segments.widths[index]
                        height: segments.height
                        radius: height / 2
                        smoothing: theme.cornerSmoothing
                        color: modelData.color
                    }
                }
            }
        }
        Row {
            spacing: theme.space(4)
            Repeater {
                model: usage.parts
                Row {
                    required property var modelData
                    spacing: theme.space(1.5)
                    Corner {
                        anchors.verticalCenter: parent.verticalCenter
                        width: theme.space(2); height: width
                        radius: width / 2
                        smoothing: theme.cornerSmoothing
                        color: modelData.color
                    }
                    Text {
                        anchors.verticalCenter: parent.verticalCenter
                        text: modelData.label + " " + modelData.text
                        color: theme.textDim
                        font.family: theme.fontSans
                        font.pointSize: theme.typeSmall
                    }
                }
            }
        }
    }

    // One library source: a folder glyph (crossed when the folder is missing), the path,
    // its facts, then Open, Rescan and Remove; Remove asks first
    component SourceRow: Item {
        id: source
        property string path: ""
        property string meta: ""
        property bool available: true
        property string question: ""
        property bool confirming: false
        width: parent.width
        height: Math.max(theme.space(13), Math.max(lead.height, controls.height) + theme.space(3) * 2)
        Keys.onEscapePressed: confirming = false

        Row {
            id: lead
            anchors.left: parent.left
            anchors.leftMargin: theme.space(4)
            anchors.verticalCenter: parent.verticalCenter
            spacing: theme.space(3)
            Icon {
                anchors.verticalCenter: parent.verticalCenter
                glyph: source.available ? "folder" : "folder-x"
                color: source.available ? theme.text : theme.textDim
            }
            Column {
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
                        color: theme.surface
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
        }
        Row {
            id: controls
            anchors.right: parent.right
            anchors.rightMargin: theme.space(3)
            anchors.verticalCenter: parent.verticalCenter
            Row {
                visible: !source.confirming
                spacing: theme.space(1)
                Button { text: "Open"; icon: "folder-open"; flat: true }
                Button { text: "Rescan"; icon: "refresh-cw"; flat: true }
                Button { text: "Remove"; icon: "trash-2"; flat: true; onClicked: { source.confirming = true; source.forceActiveFocus() } }
            }
            Confirm {
                visible: source.confirming
                question: source.question
                maxWidth: source.width - lead.width - theme.space(4) - theme.space(3) - theme.space(6)
                onAccepted: source.confirming = false
                onKept: source.confirming = false
            }
        }
    }

    // One tracker: a round avatar with the provider's initials, lit in the accent while it
    // is connected, the name, the connection line and the list counts, and a control
    component TrackerRow: Item {
        id: tracker
        property string name: ""
        property string initials: ""
        property bool connected: false
        property string line: ""
        property string stats: ""
        default property alias control: slot.data
        width: parent ? parent.width : theme.space(100)
        implicitHeight: Math.max(theme.space(12), Math.max(avatar.height, words.implicitHeight, slot.height) + theme.space(2) * 2)
        Layout.fillWidth: true

        Corner {
            id: avatar
            anchors.left: parent.left
            anchors.verticalCenter: parent.verticalCenter
            width: theme.space(9); height: width
            radius: width / 2
            smoothing: theme.cornerSmoothing
            color: tracker.connected ? theme.accentSoft : theme.surfaceSunken
            borderColor: tracker.connected ? "transparent" : theme.line
            borderWidth: tracker.connected ? 0 : 1
            Text {
                anchors.centerIn: parent
                text: tracker.initials
                color: tracker.connected ? theme.accent : theme.textDim
                font.family: theme.fontMono
                font.pointSize: theme.typeSmall
                font.weight: Font.Bold
            }
        }
        Column {
            id: words
            anchors.left: avatar.right
            anchors.leftMargin: theme.space(3)
            anchors.right: slot.left
            anchors.rightMargin: theme.space(6)
            anchors.verticalCenter: parent.verticalCenter
            spacing: theme.space(0.5)
            Text {
                width: parent.width
                text: tracker.name
                color: theme.text
                wrapMode: Text.Wrap
                font.family: theme.fontSans
                font.pointSize: theme.typeNormal
            }
            Text {
                visible: tracker.line !== ""
                width: parent.width
                text: tracker.line
                color: theme.textDim
                wrapMode: Text.Wrap
                font.family: theme.fontMono
                font.pointSize: theme.typeSmall
            }
            Text {
                visible: tracker.connected && tracker.stats !== ""
                width: parent.width
                text: tracker.stats
                color: theme.textDim
                wrapMode: Text.Wrap
                font.family: theme.fontSans
                font.pointSize: theme.typeSmall
            }
        }
        Item {
            id: slot
            anchors.right: parent.right
            anchors.verticalCenter: parent.verticalCenter
            width: childrenRect.width
            height: childrenRect.height
        }
    }

    // One tab: a Flickable over a ColumnLayout that holds whatever the tab lays out. The
    // layout is the viewport's height while its content fits and something in it grows, its
    // natural height otherwise, so the Flickable only scrolls when it has to. Under it sits
    // the focus sink: a press that no control claimed lands on it and takes the focus, so a
    // Field, Dropdown or Slider lets go when the user clicks elsewhere.
    component Tab: Flickable {
        id: tabFlick
        default property alias body: page.data
        readonly property real viewport: Math.max(0, height - root.footInset)
        anchors.fill: parent
        contentWidth: width
        contentHeight: page.height + root.footInset
        clip: true
        boundsBehavior: Flickable.StopAtBounds
        MouseArea {
            id: sink
            width: tabFlick.width
            height: Math.max(tabFlick.height, tabFlick.contentHeight)
            onPressed: sink.forceActiveFocus()
        }
        ColumnLayout {
            id: page
            readonly property bool grows: root.anyGrows(children)
            x: root.blockX
            width: root.blockWidth
            height: grows ? Math.max(tabFlick.viewport, implicitHeight) : implicitHeight
            spacing: root.gap
        }
        // The bar only exists while there is somewhere to scroll to
        QC.ScrollBar.vertical: QC.ScrollBar {
            policy: QC.ScrollBar.AsNeeded
            visible: size < 1
            contentItem: Corner { implicitWidth: theme.space(1); radius: implicitWidth / 2; smoothing: theme.cornerSmoothing; color: theme.lineStrong; opacity: parent.active ? 1 : 0.4 }
        }
    }

    // Two columns that fill the width together, the leading one `split` of it, or one under
    // the other when the width holds fewer than two of theme.space(100) and the gap between.
    // Children land in the leading column; `trail` takes the other's. Each column is a
    // ColumnLayout: with something in it that grows it is the pair's full height, otherwise
    // its natural height. The pair itself grows in the page when either column does, and
    // never shrinks under its natural height. Stacked, the spare height goes to the trailing
    // column when it grows, else to the leading one.
    component Pair: Item {
        id: pair
        property real split: 0.5
        property real columnGap: root.gap
        property real rowGap: root.gap
        default property alias lead: leadCol.data
        property alias trail: trailCol.data
        readonly property bool twoUp: width >= 2 * theme.space(100) + columnGap
        readonly property real leftW: twoUp ? Math.round((width - columnGap) * split) : width
        readonly property bool leadGrows: root.anyGrows(leadCol.children)
        readonly property bool trailGrows: root.anyGrows(trailCol.children)
        readonly property bool grows: leadGrows || trailGrows
        readonly property real natural: twoUp ? Math.max(leadCol.implicitHeight, trailCol.implicitHeight)
                                              : leadCol.implicitHeight + rowGap + trailCol.implicitHeight
        readonly property real extra: Math.max(0, height - natural)
        width: parent ? parent.width : root.blockWidth
        implicitHeight: natural
        Layout.fillWidth: true
        Layout.fillHeight: grows
        Layout.minimumHeight: natural
        Layout.preferredHeight: natural
        Layout.alignment: Qt.AlignTop
        ColumnLayout {
            id: leadCol
            width: pair.leftW
            height: pair.twoUp ? (pair.leadGrows ? pair.height : implicitHeight)
                               : implicitHeight + (pair.leadGrows && !pair.trailGrows ? pair.extra : 0)
            spacing: root.gap
        }
        ColumnLayout {
            id: trailCol
            x: pair.twoUp ? pair.leftW + pair.columnGap : 0
            y: pair.twoUp ? 0 : leadCol.height + pair.rowGap
            width: pair.twoUp ? pair.width - pair.leftW - pair.columnGap : pair.width
            height: pair.twoUp ? (pair.trailGrows ? pair.height : implicitHeight)
                               : implicitHeight + (pair.trailGrows ? pair.extra : 0)
            spacing: root.gap
        }
    }

    // The corner option's glyph: a stroked square at the smoothing the option stands for,
    // in the option's text colour, both read off the Seg's Loader
    Component {
        id: cornerGlyph
        Corner {
            width: theme.space(5)
            height: width
            radius: width * 0.45
            smoothing: parent && parent.option ? parent.option.smoothing : 0.6
            borderColor: parent ? parent.tint : "transparent"
            borderWidth: 2
        }
    }

    Column {
        id: header
        anchors.top: parent.top
        anchors.topMargin: theme.space(7)
        x: root.blockX
        width: root.blockWidth
        spacing: theme.space(4)

        Text {
            text: "Settings"
            color: theme.text
            font.family: theme.fontSans
            font.pointSize: theme.typeLarge
            font.weight: Font.Bold
        }
        Seg {
            options: root.tabNames.map(function(n, i) { return { text: n, icon: root.tabIcons[i] } })
            index: root.tab
            onPicked: function(i) { root.tab = i }
        }
    }

    Item {
        id: pages
        anchors.top: header.bottom
        anchors.topMargin: theme.space(6)
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.bottom: parent.bottom

        // Library: the counts, the sources in the growing space, the actions and the two
        // switches at the foot; the trackers beside
        Tab {
            id: libraryTab
            visible: root.tab === 0
            Pair {
                split: 3 / 5
                Panel {
                    title: "Library"
                    icon: "folder-open"
                    grows: true
                    Tiles {
                        stats: [
                            { value: "412", label: "Series" }, { value: "38", label: "Films" },
                            { value: "6,120", label: "Episodes" }, { value: "12:04", label: "Last scan" }
                        ]
                    }
                    stretch: Corner {
                        anchors.fill: parent
                        implicitHeight: sources.height
                        radius: theme.radiusMd
                        smoothing: theme.cornerSmoothing
                        color: theme.surfaceSunken
                        borderColor: theme.line
                        borderWidth: 1
                        Column {
                            id: sources
                            width: parent.width
                            SourceRow { path: "/mnt/media/Anime"; meta: "412 series · 2 movie folders · 6,120 episodes"; question: "Remove Anime, 412 series and their history?"; confirming: root.demoConfirm }
                            Rectangle { width: parent.width - theme.space(4) * 2; anchors.horizontalCenter: parent.horizontalCenter; height: 1; color: theme.line }
                            SourceRow { path: "/mnt/media/Films"; meta: "38 films · last seen 2026-08-30"; available: false; question: "Remove Films and its history?" }
                        }
                    }
                    foot: [
                        Row {
                            spacing: theme.space(2)
                            Button { text: "Add folder"; icon: "folder-plus" }
                            Button { text: "Scan all"; icon: "refresh-cw" }
                        },
                        Note { text: "AniBeam scans these folders for video files. A folder is a series; a file at the top level of a Movies folder is a film." },
                        SettingRow {
                            label: "Show hidden shows"
                            helper: "Shows hidden series on every page until AniBeam closes."
                            Switch {}
                        },
                        SettingRow {
                            label: "Subscriptions"
                            helper: "The feeds anirss watches for you."
                            Button { text: "Open"; icon: "arrow-up-right" }
                        }
                    ]
                }
                trail: Panel {
                    title: "Trackers"
                    icon: "user-check"
                    helper: "Episodes are marked on every connected tracker when you reach the outro or mark them by hand. Counts only go up."
                    TrackerRow {
                        name: "AniList"
                        initials: "AL"
                        connected: true
                        line: "Connected as marcusbandit · synced 4 min ago"
                        stats: "Watching 12 · Completed 231 · Planning 40"
                        Button { text: "Disconnect"; icon: "log-out" }
                    }
                    TrackerRow {
                        name: "MyAnimeList"
                        initials: "MAL"
                        line: "Not connected"
                        Button { text: "Log in"; icon: "log-in" }
                    }
                    SettingRow {
                        id: mainTracker
                        property int choice: 0
                        label: "Main tracker"
                        helper: "Whose count the cards show. The other tracker still receives every mark."
                        Seg { options: ["AniList", "MyAnimeList"]; index: mainTracker.choice; onPicked: function(i) { mainTracker.choice = i } }
                    }
                }
            }
        }

        // Appearance: the controls on the left, the preview of both modes on the right
        Tab {
            id: lookTab
            visible: root.tab === 1
            Pair {
                split: 2 / 5
                Panel {
                    title: "Colours"
                    icon: "palette"
                    SettingRow {
                        label: "Mode"
                        Seg {
                            options: [{ text: "Dark", icon: "moon" }, { text: "Light", icon: "sun" }, { text: "System", icon: "monitor" }]
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
                }
                Panel {
                    title: "Shape"
                    icon: "shapes"
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
                            options: [{ text: "Smooth", delegate: cornerGlyph, smoothing: 0.6 }, { text: "Plain", delegate: cornerGlyph, smoothing: 0 }]
                            index: theme.cornerSmoothing > 0 ? 0 : 1
                            onPicked: function(i) { theme.cornerSmoothing = i === 0 ? 0.6 : 0 }
                        }
                    }
                }
                Item {
                    Layout.fillWidth: true
                    implicitHeight: themeNote.implicitHeight
                    Note {
                        id: themeNote
                        x: theme.space(6)
                        width: parent.width - theme.space(6) * 2
                        text: "All of this lives in ~/.config/anibeam/theme.toml and reloads when the file changes."
                    }
                }
                trail: Panel {
                    title: "Preview"
                    icon: "eye"
                    grows: true
                    stretch: Item {
                        anchors.fill: parent
                        implicitHeight: lookPreview.implicitHeight
                        LookPreview {
                            id: lookPreview
                            width: parent.width
                            library: root.library
                            titleLang: root.titleLang
                        }
                    }
                }
            }
        }

        // Playback: the switches, the track languages and the subtitle defaults on the left;
        // the preview on the right, filling its panel with the picture's aspect kept
        Tab {
            id: playbackTab
            visible: root.tab === 2
            Pair {
                split: 2 / 5
                Panel {
                    title: "Playback"
                    icon: "play"
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
                Panel {
                    title: "Tracks"
                    icon: "captions"
                    SettingRow {
                        label: "Subtitle languages"
                        Field { text: "en"; implicitWidth: theme.space(30) }
                    }
                    SettingRow {
                        label: "Audio languages"
                        Field { text: "ja"; implicitWidth: theme.space(30) }
                    }
                    Note { text: "Comma separated, first match wins." }
                }
                Panel {
                    title: "Subtitle defaults"
                    icon: "type"
                    helper: "What every session starts from. Change tracks in the player and AniBeam remembers them per series."
                    Pair {
                        rowGap: theme.space(2)
                        SettingRow {
                            id: scaleRow
                            property real scale: 1
                            label: "Scale"
                            SliderRow { from: 0.5; to: 2.0; stepSize: 0.05; decimals: 2; value: scaleRow.scale; onMoved: function(v) { scaleRow.scale = v } }
                        }
                        trail: SettingRow {
                            id: assRow
                            property int choice: 0
                            label: "ASS override"
                            helper: "Force applies the text style to styled subtitles and may break signs and karaoke."
                            Seg { options: ["As scripted", "Scale only", "Force"]; index: assRow.choice; onPicked: function(i) { assRow.choice = i } }
                        }
                    }
                    Item { width: parent.width; height: theme.space(2) }
                    Note { text: "TEXT STYLE, FOR SRT AND VTT"; font.letterSpacing: 1 }
                    Pair {
                        rowGap: theme.space(2)
                        SettingRow {
                            label: "Font"
                            Field { id: fontField; text: "sans-serif"; implicitWidth: theme.space(30) }
                        }
                        trail: SettingRow {
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
                    }
                    Pair {
                        rowGap: theme.space(2)
                        SettingRow {
                            label: "Outline"
                            Row {
                                spacing: theme.space(2)
                                Field { id: outlineField; text: "1.65"; mono: true; implicitWidth: theme.space(20) }
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
                        trail: SettingRow {
                            label: "Shadow"
                            Field { id: shadowField; text: "0"; mono: true; implicitWidth: theme.space(20) }
                        }
                    }
                    Pair {
                        rowGap: theme.space(2)
                        SettingRow {
                            label: "Box opacity"
                            Field { id: boxField; text: "0"; mono: true; implicitWidth: theme.space(20) }
                        }
                        trail: SettingRow {
                            label: "Bold"
                            Switch { id: boldSwitch }
                        }
                    }
                    SettingRow {
                        id: positionRow
                        property real pos: 100
                        label: "Position"
                        SliderRow { from: 0; to: 150; stepSize: 1; decimals: 0; value: positionRow.pos; onMoved: function(v) { positionRow.pos = v } }
                    }
                }
                trail: Panel {
                    title: "Preview"
                    icon: "eye"
                    grows: true
                    // The still at the widest 16:9 the slot holds, centred, so it is
                    // letterboxed inside the panel rather than stretched
                    stretch: Item {
                        anchors.fill: parent
                        implicitHeight: Math.round(width * 9 / 16)
                        SubtitlePreview {
                            anchors.centerIn: parent
                            width: Math.round(Math.min(parent.width, parent.height * 16 / 9))
                            fontFamily: fontField.text
                            bold: boldSwitch.checked
                            fill: root.isHex(textColour.text) ? textColour.text : "#FFFFFF"
                            outlineColour: root.outlineColour
                            outline: root.num(outlineField.text, 0)
                            shadow: root.num(shadowField.text, 0)
                            boxOpacity: root.num(boxField.text, 0)
                            position: positionRow.pos
                            textScale: scaleRow.scale
                        }
                    }
                }
            }
        }

        // Data: storage and export side by side, the about panel across the foot
        Tab {
            id: dataTab
            visible: root.tab === 3
            Pair {
                split: 1 / 2
                Panel {
                    title: "Storage"
                    icon: "hard-drive"
                    Tiles {
                        stats: [
                            { value: "312 MB", label: "Images" }, { value: "4.1 MB", label: "Database" },
                            { value: "2,000", label: "Events kept" }, { value: "1,204", label: "Posters" }
                        ]
                    }
                    UsageBar {
                        parts: [
                            { label: "Images", amount: 312, text: "312 MB", color: theme.blue },
                            { label: "Database", amount: 4.1, text: "4.1 MB", color: theme.purple }
                        ]
                    }
                    SettingRow {
                        id: imagesRow
                        property bool confirming: false
                        label: "Images"
                        line: "1,204 files · 312 MB"
                        helper: "Posters come back on the next launch."
                        Keys.onEscapePressed: confirming = false
                        Row {
                            Button { visible: !imagesRow.confirming; text: "Clear images"; icon: "trash-2"; onClicked: { imagesRow.confirming = true; imagesRow.forceActiveFocus() } }
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
                    SettingRow { label: "Database"; line: "~/.local/share/anibeam/anibeam.db"; Button { text: "Open"; icon: "folder-open"; flat: true } }
                    SettingRow { label: "Data"; line: "~/.local/share/anibeam"; Button { text: "Open"; icon: "folder-open"; flat: true } }
                    SettingRow { label: "Config"; line: "~/.config/anibeam"; Button { text: "Open"; icon: "folder-open"; flat: true } }
                    SettingRow { label: "Cache"; line: "~/.cache/anibeam"; Button { text: "Open"; icon: "folder-open"; flat: true } }
                }
                trail: Panel {
                    title: "Export and import"
                    icon: "archive"
                    SettingRow {
                        label: "Include private data"
                        helper: "Tracker logins, API keys, watch history and preferences, in plain text."
                        Switch { id: privateData }
                    }
                    SettingRow {
                        label: "Export"
                        helper: privateData.checked ? "Writes anibeam-export-full-<date>.json." : "Writes anibeam-export-<date>.json."
                        status: "Last export: never."
                        Button { text: "Export"; icon: "upload" }
                    }
                    SettingRow {
                        label: "Import"
                        helper: "Merges a file into this library. The file wins for matches and accounts, the newer entry wins for history, nothing is deleted."
                        status: "Last import: 2026-09-01, 2 sources, 380 series, 4,912 episodes merged."
                        Button { text: "Import"; icon: "download" }
                    }
                }
            }
            // About: the icon, the name with its version and licence, the maker, one line
            // of what it is, the links; and the eight hues as dots at the right edge, a
            // flourish that doubles as a palette check. Nothing here is live.
            Panel {
                title: "About"
                icon: "info"
                grows: true
                stretch: Item {
                    id: aboutBox
                    anchors.fill: parent
                    implicitHeight: Math.max(appTile.height, blurb.height)
                    Corner {
                        id: appTile
                        anchors.left: parent.left
                        anchors.top: parent.top
                        width: theme.space(16); height: width
                        radius: theme.radiusMd
                        smoothing: theme.cornerSmoothing
                        color: theme.accentSoft
                        Image {
                            anchors.centerIn: parent
                            width: parent.width * 0.7; height: width
                            source: "qrc:/qt/qml/dev/anibeam/proto/assets/icon.png"
                            sourceSize: Qt.size(128, 128)
                            smooth: true
                        }
                    }
                    Column {
                        id: blurb
                        anchors.left: appTile.right
                        anchors.leftMargin: theme.space(5)
                        anchors.right: dots.left
                        anchors.rightMargin: theme.space(6)
                        anchors.top: parent.top
                        spacing: theme.space(1.5)
                        Row {
                            spacing: theme.space(2)
                            Text {
                                anchors.verticalCenter: parent.verticalCenter
                                text: "AniBeam"
                                color: theme.text
                                font.family: theme.fontSans
                                font.pointSize: theme.typeLarge
                                font.weight: Font.Bold
                            }
                            Chip { anchors.verticalCenter: parent.verticalCenter; text: "2.0.0"; small: true; selected: true }
                            Chip { anchors.verticalCenter: parent.verticalCenter; text: "GPL-3.0-or-later"; small: true; color: theme.surfaceSunken; textColor: theme.textDim }
                        }
                        Text {
                            width: parent.width
                            text: "Made by Marcus Rosado"
                            color: theme.text
                            wrapMode: Text.Wrap
                            font.family: theme.fontSans
                            font.pointSize: theme.typeNormal
                        }
                        Note { text: "A local anime library: scans your folders, matches them against AniList, plays them through mpv, and keeps your trackers up to date." }
                        Item { width: 1; height: theme.space(1) }
                        Flow {
                            width: parent.width
                            spacing: theme.space(1)
                            Button { text: "github.com/marcusbandit/AniBeam"; icon: "git-branch"; flat: true }
                            Button { text: "marcusrosado.com"; icon: "globe"; flat: true }
                            Button { text: "AniList marcusbandit"; icon: "heart"; flat: true }
                        }
                    }
                    Row {
                        id: dots
                        anchors.right: parent.right
                        anchors.verticalCenter: appTile.verticalCenter
                        spacing: theme.space(1.5)
                        Repeater {
                            model: root.hues
                            Corner {
                                required property var modelData
                                width: theme.space(3); height: width
                                radius: width / 2
                                smoothing: theme.cornerSmoothing
                                color: modelData
                            }
                        }
                    }
                }
            }
        }
    }
}
