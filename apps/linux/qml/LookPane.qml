// One mode's sample in the Appearance preview: a small Library page, header to status
// strip. The pane carries its own tokens, forced to `mode` and following every other knob
// of the app's theme through the shared `Theme` singleton underneath; it is declared with
// the id `theme`, the plan's one sanctioned shadow of a context-chain name, so the Cards,
// Chips, Seg, Switch, Buttons and Icons drawn inside resolve `theme` to it and render
// exactly as they would with the app in that mode. Nothing else needs to know.
import QtQuick

Corner {
    id: pane
    property string mode: "dark"
    property var samples: []             // SeriesCard records, most telling first

    Tokens { id: theme; mode: pane.mode }

    readonly property real pad: theme.space(5)
    // Shown when the library holds nothing with a poster yet, so the pane is never blank.
    readonly property var fallback: ({
        title: "Sample series",
        titles: { folder: "Sample series" },
        poster: null,
        code: "EP 12",
        watched: 8,
        watched_state: "Behind",
        total_episodes: 12,
        total_is_estimate: false,
        strip: { watched: 0.5, aired_unwatched: 0.17, unknown: 0 },
        community_score: 8.2,
        my_score: null,
        hidden: false,
        next_airing: null,
        last_viewed_at: null,
        episodes_on_disk: 12
    })

    radius: theme.radiusLg
    smoothing: theme.cornerSmoothing
    color: theme.bg
    borderColor: theme.line
    borderWidth: 1
    implicitHeight: body.height + pad * 2

    Column {
        id: body
        x: pane.pad
        y: pane.pad
        width: pane.width - pane.pad * 2
        spacing: theme.space(4)

        // The page header
        Row {
            spacing: theme.space(3)
            Text {
                anchors.verticalCenter: parent.verticalCenter
                text: "Library"
                color: theme.text
                font.family: theme.fontSans
                font.pointSize: theme.typeLarge
                font.weight: Font.Bold
            }
            Chip {
                anchors.verticalCenter: parent.verticalCenter
                text: pane.samples.length + " series"
                small: true
                color: theme.surface
                textColor: theme.textDim
            }
        }

        // The search pill, empty, with its shortcut hint
        Corner {
            width: parent.width
            height: theme.controlHeight
            radius: height / 2
            smoothing: theme.cornerSmoothing
            color: theme.surfaceSunken
            borderColor: theme.line
            borderWidth: 1
            Text {
                anchors.left: parent.left
                anchors.leftMargin: theme.space(4)
                anchors.right: hint.left
                anchors.rightMargin: theme.space(2)
                anchors.verticalCenter: parent.verticalCenter
                text: "Search romaji, english or folder"
                color: theme.textFaint
                elide: Text.ElideRight
                font.family: theme.fontSans
                font.pointSize: theme.typeNormal
            }
            Chip {
                id: hint
                anchors.right: parent.right
                anchors.rightMargin: theme.space(2)
                anchors.verticalCenter: parent.verticalCenter
                text: "Ctrl K"
                small: true
                color: theme.surface
                textColor: theme.textFaint
            }
        }

        Seg { options: ["All", "Series", "Movies"]; index: 0 }

        // As many cards as the pane holds at the poster width, one at least
        Row {
            id: cards
            readonly property real gap: theme.space(5)
            readonly property int count: Math.max(1, Math.floor((body.width + gap) / (theme.posterWidth + gap)))
            spacing: gap
            Repeater {
                model: cards.count
                Card {
                    required property int index
                    item: pane.samples.length ? pane.samples[index % pane.samples.length] : pane.fallback
                    posterWidth: theme.posterWidth
                    nowMs: Date.now()
                }
            }
        }

        Flow {
            width: parent.width
            spacing: theme.space(2)
            Chip { text: "Plain"; mono: false; color: theme.surface; textColor: theme.textDim }
            Chip { text: "Selected"; mono: false; selected: true }
            Chip { text: "EP 12"; small: true; color: theme.surface }
            Chip { text: "2 errors"; icon: "circle-alert"; small: true; color: theme.redSoft; textColor: theme.red }
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

        Row {
            spacing: theme.space(3)
            Switch { anchors.verticalCenter: parent.verticalCenter; checked: true }
            Button { anchors.verticalCenter: parent.verticalCenter; text: "Button" }
            Button { anchors.verticalCenter: parent.verticalCenter; text: "Remove"; icon: "trash-2"; danger: true }
        }

        // The status strip, one line of it
        Corner {
            width: parent.width
            height: theme.space(7)
            radius: theme.radiusSm
            smoothing: theme.cornerSmoothing
            color: theme.surfaceSunken
            borderColor: theme.line
            borderWidth: 1
            Row {
                id: stripLine
                anchors.left: parent.left
                anchors.leftMargin: theme.space(3)
                anchors.right: stripErrors.left
                anchors.rightMargin: theme.space(3)
                anchors.verticalCenter: parent.verticalCenter
                spacing: theme.space(2)
                Chip { id: stripStage; anchors.verticalCenter: parent.verticalCenter; text: "scan"; small: true; color: theme.surface; textColor: theme.textDim }
                Text {
                    anchors.verticalCenter: parent.verticalCenter
                    width: Math.max(0, Math.min(implicitWidth, stripLine.width - stripStage.width - stripStamp.width - stripLine.spacing * 2))
                    text: "3 new episodes in Frieren"
                    color: theme.text
                    elide: Text.ElideRight
                    font.family: theme.fontSans
                    font.pointSize: theme.typeSmall
                }
                Text { id: stripStamp; anchors.verticalCenter: parent.verticalCenter; text: "12:04"; color: theme.textFaint; font.family: theme.fontMono; font.pointSize: theme.typeSmall }
            }
            Chip {
                id: stripErrors
                anchors.right: parent.right
                anchors.rightMargin: theme.space(3)
                anchors.verticalCenter: parent.verticalCenter
                text: "2 errors"
                icon: "circle-alert"
                small: true
                color: theme.redSoft
                textColor: theme.red
            }
        }
    }
}
