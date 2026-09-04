// The floating prototype bar: every theme knob live, drawn inverted with a dashed edge so it
// is obviously not part of the design being judged. H hides it for screenshots.
import QtQuick
import QtQuick.Controls.Basic

Corner {
    id: root
    radius: 18
    smoothing: 0.6
    color: theme.text
    borderColor: theme.accent
    borderWidth: 2
    dashed: 1
    property real maxWidth: 1000
    implicitWidth: Math.min(maxWidth, body.implicitWidth + 32)
    implicitHeight: body.implicitHeight + 24

    readonly property var themeList: theme.palettes.themes || []
    readonly property var darkThemes: themeList.filter(function(t) { return t.variant === "dark" })
    readonly property var lightThemes: themeList.filter(function(t) { return t.variant === "light" })
    function slugIndex(list, slug) { for (var i = 0; i < list.length; i++) if (list[i].slug === slug) return i; return 0 }

    component Pick: Row {
        id: pick
        property var options: []
        property int index: 0
        signal picked(int i)
        spacing: 2
        Repeater {
            model: pick.options
            Rectangle {
                required property int index
                required property string modelData
                width: t.implicitWidth + 16; height: 24; radius: 6
                color: index === pick.index ? theme.accent : "transparent"
                border.color: theme.bg; border.width: 1
                Text {
                    id: t
                    anchors.centerIn: parent
                    text: parent.modelData
                    color: parent.index === pick.index ? theme.accentText : theme.bg
                    font.family: theme.fontMono
                    font.pointSize: theme.typeSmall
                }
                MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: { pick.index = parent.index; pick.picked(parent.index) } }
            }
        }
    }

    component Dial: Row {
        id: dial
        property alias from: s.from
        property alias to: s.to
        property alias value: s.value
        property alias stepSize: s.stepSize
        property int decimals: 0
        signal moved(real v)
        spacing: 8
        Slider {
            id: s
            width: 120; height: 24
            onMoved: dial.moved(value)
            background: Rectangle {
                x: s.leftPadding; y: s.topPadding + s.availableHeight / 2 - height / 2
                width: s.availableWidth; height: 4; radius: 2
                color: theme.bg; opacity: 0.35
                Rectangle { width: s.visualPosition * parent.width; height: parent.height; color: theme.accent; radius: 2 }
            }
            handle: Rectangle {
                x: s.leftPadding + s.visualPosition * (s.availableWidth - width)
                y: s.topPadding + s.availableHeight / 2 - height / 2
                width: 16; height: 16; radius: 8
                color: theme.accent; border.color: theme.bg; border.width: 2
            }
        }
        Text {
            anchors.verticalCenter: parent.verticalCenter
            text: Number(s.value).toFixed(dial.decimals)
            color: theme.bg
            font.family: theme.fontMono
            font.pointSize: theme.typeSmall
            width: 36
        }
    }

    Flow {
        id: body
        x: 16; y: 12
        width: root.maxWidth - 32
        spacing: 20
        Row {
            width: body.width
            spacing: 16
            Text { text: "PROTOTYPE"; color: theme.accent; font.family: theme.fontMono; font.pointSize: theme.typeSmall; font.weight: Font.Bold; font.letterSpacing: 2 }
            Text { width: body.width - 120; wrapMode: Text.Wrap; text: theme.sourceLabel + "   mode " + theme.resolvedMode + "   font " + theme.fontSans + " " + theme.systemPointSize + "pt   H hides this bar"; color: theme.bg; opacity: 0.8; font.family: theme.fontMono; font.pointSize: theme.typeSmall }
        }
        Flow {
            width: body.width
            spacing: 20
            Knob { label: "Mode"; Pick { options: ["Dark", "Light", "System"]; index: theme.mode === "dark" ? 0 : theme.mode === "light" ? 1 : 2; onPicked: function(i) { theme.mode = ["dark", "light", "system"][i] } } }
            Knob { label: "Colour source"; Pick { options: ["System", "Theme"]; index: theme.colourSource === "theme" ? 1 : 0; onPicked: function(i) { theme.colourSource = i === 1 ? "theme" : "system" } } }
            Knob { label: "Accent slot"; Pick { options: ["1", "2", "3", "4", "5", "6"]; index: theme.accentSlot - 1; onPicked: function(i) { theme.accentSlot = i + 1 } } }
            Knob { label: "Dark theme"; Pick { options: root.darkThemes.map(function(t) { return t.name }); index: root.slugIndex(root.darkThemes, theme.themeDark); onPicked: function(i) { theme.themeDark = root.darkThemes[i].slug } } }
            Knob { label: "Light theme"; Pick { options: root.lightThemes.map(function(t) { return t.name }); index: root.slugIndex(root.lightThemes, theme.themeLight); onPicked: function(i) { theme.themeLight = root.lightThemes[i].slug } } }
        }
        Flow {
            width: body.width
            spacing: 20
            Knob { label: "Density"; Pick { options: ["Compact", "Normal", "Comfortable"]; index: theme.density === "compact" ? 0 : theme.density === "comfortable" ? 2 : 1; onPicked: function(i) { theme.density = ["compact", "normal", "comfortable"][i] } } }
            Knob { label: "Poster width"; Row { spacing: 8
                Pick { options: ["S", "M", "L"]; index: theme.posterWidth <= 140 ? 0 : theme.posterWidth >= 240 ? 2 : 1; onPicked: function(i) { theme.posterWidth = [140, 180, 240][i] } }
                Dial { from: 110; to: 340; stepSize: 10; value: theme.posterWidth; onMoved: function(v) { theme.posterWidth = v } } } }
            Knob { label: "Corner smoothing"; Dial { from: 0; to: 1; stepSize: 0.05; decimals: 2; value: theme.cornerSmoothing; onMoved: function(v) { theme.cornerSmoothing = v } } }
            Knob { label: "Corner base"; Dial { from: 0; to: 24; stepSize: 1; value: theme.cornerBase; onMoved: function(v) { theme.cornerBase = v } } }
            Knob { label: "Surface step"; Dial { from: 0; to: 0.2; stepSize: 0.01; decimals: 2; value: theme.stepSurface; onMoved: function(v) { theme.stepSurface = v; theme.stepRaised = v * 2 } } }
            Knob { label: "Line step"; Dial { from: 0; to: 0.5; stepSize: 0.01; decimals: 2; value: theme.stepLine; onMoved: function(v) { theme.stepLine = v; theme.stepLineStrong = Math.min(1, v * 1.6) } } }
        }
    }
}
