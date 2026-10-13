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
        Label { text: "Tokens"; role: "display" }
        Label { text: theme.sourceLabel + ", " + theme.mode; color: theme.textDim; role: "ledger" }
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
                    Text { text: modelData; color: theme.textDim; font.family: theme.fontUi; font.pointSize: theme.typeSmall }
                }
            }
        }
        Row {
            spacing: theme.space(3)
            Chip { text: "Ep 12" }
            Chip { text: "12/24"; textColor: theme.behind }
            Chip { text: "Selected"; selected: true; mono: false }
            Chip { text: "2 errors"; icon: "circle-alert"; small: true; color: theme.redSoft; textColor: theme.red }
            Seg { options: ["All", "Series", "Movies"]; index: 1 }
            Switch { checked: true }
            Button { text: "Play"; icon: "play"; primary: true }
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
        // The glass ramp: resting, raised, pressed, floating, each a square pane with its edge
        // and top highlight
        Row {
            spacing: theme.grout
            Repeater {
                model: [["Glass", theme.glass], ["Raised", theme.glassRaised], ["Pressed", theme.glassPressed], ["Float", theme.glassFloat], ["Video", theme.glassVideo]]
                Corner { required property var modelData; width: theme.space(30); height: theme.space(16); color: modelData[1]; borderColor: theme.line; borderWidth: 1; highlight: theme.glassHighlight
                    Label { anchors.centerIn: parent; role: "ledger"; text: parent.modelData[0] } }
            }
        }
        // The four roles of the face, then the faint and dim text on a tile
        Column {
            spacing: theme.space(1)
            Label { role: "display"; text: "Display, wdth 125 wght 760" }
            Label { role: "title"; text: "Title, wdth 112 wght 650" }
            Label { role: "ui"; text: "UI, wdth 100 wght 450: the controls and the body" }
            Label { role: "ledger"; text: "Ledger, wdth 88 wght 500: 1111 0000 24 of 24, 23:59" }
            Label { role: "ledger"; color: theme.textDim; text: "Dim ledger: 12 files, 3 days ago" }
            Label { role: "ledger"; color: theme.textFaint; text: "Faint ledger: no poster, 0 files" }
        }
        // The poster scrim, the one gradient: near-black at the foot, clear by scrimReach
        Rectangle {
            width: theme.space(40); height: theme.space(60)
            color: theme.accentSoft
            Rectangle {
                anchors.bottom: parent.bottom; width: parent.width; height: parent.height * theme.scrimReach
                gradient: Gradient {
                    GradientStop { position: 0; color: "transparent" }
                    GradientStop { position: 0.45; color: Qt.rgba(theme.posterScrim.r, theme.posterScrim.g, theme.posterScrim.b, theme.posterScrim.a * 0.62) }
                    GradientStop { position: 1; color: theme.posterScrim }
                }
            }
            Label { x: theme.space(2.5); anchors.bottom: parent.bottom; anchors.bottomMargin: theme.space(2.5); role: "title"; font.pointSize: theme.typeNormal; color: theme.onPoster; text: "Title on the scrim" }
        }
        Column {
            spacing: theme.space(1)
            Text { text: "Large " + theme.typeLarge.toFixed(1); color: theme.text; font.family: theme.fontDisplay; font.pointSize: theme.typeLarge; font.weight: Font.DemiBold }
            Text { text: "Normal " + theme.typeNormal.toFixed(1); color: theme.text; font.family: theme.fontSans; font.pointSize: theme.typeNormal }
            Text { text: "Small, dim " + theme.typeSmall.toFixed(1); color: theme.textDim; font.family: theme.fontSans; font.pointSize: theme.typeSmall }
            Text { text: "0123456789 in the ledger cut"; color: theme.textDim; font.family: theme.fontUi; font.pointSize: theme.typeNormal }
        }
    }
}
