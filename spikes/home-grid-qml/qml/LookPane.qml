// One mode's sample under the Look controls. The pane carries its own Theme, forced to
// `mode` and following every other knob of the app's theme; it is declared with the id
// `theme`, so the Card, Chips, Switch and Buttons drawn inside resolve `theme` to it and
// render exactly as they would with the app in that mode. Nothing else needs to know.
import QtQuick

Corner {
    id: pane
    property string mode: "dark"
    property var host: null              // the app's Theme; every knob but the mode follows it
    property var sample: ({})
    property string titleLang: "jp"
    property real nowMs: Date.now()

    Theme {
        id: theme
        mode: pane.mode
        palettes: pane.host ? pane.host.palettes : ({})
        colourSource: pane.host ? pane.host.colourSource : "system"
        themeDark: pane.host ? pane.host.themeDark : "anibeam-dark"
        themeLight: pane.host ? pane.host.themeLight : "anibeam-light"
        accentSlot: pane.host ? pane.host.accentSlot : 4
        density: pane.host ? pane.host.density : "normal"
        posterWidth: pane.host ? pane.host.posterWidth : 180
        cornerSmoothing: pane.host ? pane.host.cornerSmoothing : 0.6
        cornerBase: pane.host ? pane.host.cornerBase : 14
        stepSunken: pane.host ? pane.host.stepSunken : 0.03
        stepSurface: pane.host ? pane.host.stepSurface : 0.05
        stepRaised: pane.host ? pane.host.stepRaised : 0.10
        stepLine: pane.host ? pane.host.stepLine : 0.16
        stepLineStrong: pane.host ? pane.host.stepLineStrong : 0.26
        stepFaint: pane.host ? pane.host.stepFaint : 0.45
        stepDim: pane.host ? pane.host.stepDim : 0.70
    }

    readonly property real pad: theme.space(4)
    readonly property real gap: theme.space(4)
    // The column sits beside the card when it keeps at least this much width, else under it
    readonly property real minColumn: theme.space(48)

    radius: theme.radiusLg
    smoothing: theme.cornerSmoothing
    color: theme.bg
    borderColor: theme.line
    borderWidth: 1
    implicitHeight: body.height + pad * 2

    Item {
        id: body
        x: pane.pad
        y: pane.pad
        width: pane.width - pane.pad * 2
        readonly property bool beside: width >= card.width + pane.gap + pane.minColumn
        height: beside ? Math.max(card.height, column.height) : card.height + pane.gap + column.height

        Card {
            id: card
            item: pane.sample
            posterWidth: theme.posterWidth
            titleLang: pane.titleLang
            nowMs: pane.nowMs
        }

        Column {
            id: column
            x: body.beside ? card.width + pane.gap : 0
            y: body.beside ? 0 : card.height + pane.gap
            width: body.beside ? body.width - card.width - pane.gap : body.width
            spacing: theme.space(3)

            Flow {
                width: parent.width
                spacing: theme.space(2)
                Chip { text: "Plain"; mono: false; color: theme.surface; textColor: theme.textDim }
                Chip { text: "Selected"; mono: false; selected: true }
                Chip { text: "EP 12"; small: true; color: theme.surface }
            }
            Text {
                width: parent.width
                text: "Large"
                color: theme.text
                wrapMode: Text.Wrap
                font.family: theme.fontSans
                font.pointSize: theme.typeLarge
                font.weight: Font.Bold
            }
            Text {
                width: parent.width
                text: "Normal"
                color: theme.text
                wrapMode: Text.Wrap
                font.family: theme.fontSans
                font.pointSize: theme.typeNormal
            }
            Text {
                width: parent.width
                text: "Small, dim"
                color: theme.textDim
                wrapMode: Text.Wrap
                font.family: theme.fontSans
                font.pointSize: theme.typeSmall
            }
            Switch { checked: true }
            Flow {
                width: parent.width
                spacing: theme.space(2)
                Button { text: "Button" }
                Button { text: "Remove"; danger: true }
            }
        }
    }
}
