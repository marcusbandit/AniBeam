// Every token and every primitive on one page, for a capture. Reached with --page tokens.
import QtQuick
import com.marcusrosado.AniBeam

Flickable {
    id: root
    contentHeight: column.implicitHeight + theme.space(8)
    clip: true
    Column {
        id: column
        x: theme.space(8); y: theme.space(7)
        width: parent.width - theme.space(16)
        spacing: theme.space(4)
        Text { text: "Tokens"; color: theme.text; font.family: theme.fontSans; font.pointSize: theme.typeLarge; font.weight: Font.Bold }
        Text { text: theme.sourceLabel + ", " + theme.mode; color: theme.textDim; font.family: theme.fontMono; font.pointSize: theme.typeSmall }
        // Task 6 smoke line; Task 7 removes it with the rest of this page.
        Text { text: "core " + (Door.ready ? "ready, " + Door.about.version + ", " + Door.runningJobs.length + " jobs" : "starting"); color: theme.textDim; font.family: theme.fontMono; font.pointSize: theme.typeSmall }
        Flow {
            width: parent.width
            spacing: theme.space(2)
            Repeater {
                model: ["bg", "surface", "surface_raised", "surface_sunken", "surface_pressed", "line", "line_strong", "text", "text_dim", "text_faint",
                        "accent", "accent_text", "accent_soft", "red_soft", "focus", "red", "orange", "yellow", "green", "cyan", "blue", "purple", "brown"]
                Column {
                    required property string modelData
                    spacing: theme.space(1)
                    Corner { width: theme.space(16); height: theme.space(10); radius: theme.radiusSm; smoothing: theme.cornerSmoothing; color: theme.token(modelData); borderColor: theme.line; borderWidth: 1 }
                    Text { text: modelData; color: theme.textDim; font.family: theme.fontMono; font.pointSize: theme.typeSmall }
                }
            }
        }
        Row {
            spacing: theme.space(3)
            Chip { text: "EP 12" }
            Chip { text: "12/24"; textColor: theme.behind }
            Chip { text: "Selected"; selected: true; mono: false }
            Chip { text: "2 errors"; icon: "circle-alert"; small: true; color: theme.redSoft; textColor: theme.red }
            Seg { options: ["All", "Series", "Movies"]; index: 1 }
            Switch { checked: true }
            Button { text: "Button"; icon: "check" }
            Button { text: "Remove"; icon: "trash-2"; danger: true }
            Button { text: "Flat"; flat: true }
        }
        Row {
            spacing: theme.space(3)
            Field { placeholder: "A field" }
            Dropdown { options: ["AniBeam Dark", "Catppuccin Mocha"]; index: 0 }
            Swatches { slot: 4 }
            SliderRow { from: 0; to: 150; value: 100 }
        }
        Row {
            spacing: theme.space(3)
            Repeater {
                model: [theme.radiusSm, theme.radiusMd, theme.radiusLg, theme.radiusXl]
                Corner { required property real modelData; width: theme.space(24); height: theme.space(16); radius: modelData; smoothing: theme.cornerSmoothing; color: theme.surface; borderColor: theme.lineStrong; borderWidth: 1 }
            }
        }
        Column {
            spacing: theme.space(1)
            Text { text: "Large " + theme.typeLarge.toFixed(1); color: theme.text; font.family: theme.fontSans; font.pointSize: theme.typeLarge; font.weight: Font.Bold }
            Text { text: "Normal " + theme.typeNormal.toFixed(1); color: theme.text; font.family: theme.fontSans; font.pointSize: theme.typeNormal }
            Text { text: "Small, dim " + theme.typeSmall.toFixed(1); color: theme.textDim; font.family: theme.fontSans; font.pointSize: theme.typeSmall }
            Text { text: "0123456789 in the fixed face"; color: theme.textDim; font.family: theme.fontMono; font.pointSize: theme.typeNormal }
        }
    }
}
