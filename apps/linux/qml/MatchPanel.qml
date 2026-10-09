// The match popover: search AniList for the series's real record, or resolve a
// pasted AniList or MAL link, and apply. Search and resolve are jobs whose
// replies come back as events (SearchFinished, LinkResolved), matched here by
// job id. Applying writes the match confirmed at once; the assets follow and
// SeriesChanged re-lists the table behind this panel.
import QtQuick
import com.marcusrosado.AniBeam

Corner {
    id: root
    property real seriesId: 0
    property string seriesTitle: ""
    property var currentMatch: null
    property var results: []
    property var resolved: null          // a MatchTarget from a pasted link, waiting to be applied
    property int searchJob: -1
    property bool searching: false

    function openFor(id, title, matchInfo) {
        root.seriesId = id
        root.seriesTitle = title
        root.currentMatch = matchInfo || null
        root.results = []
        root.resolved = null
        search.text = title
        visible = true
        search.focusInput()
        runSearch(title)
    }
    function runSearch(q) {
        var r = Door.searchProvider(q, 12)
        if (r.error) { frame.toast(r.error.message); return }
        root.searchJob = r.reply.Started.job
        root.searching = true
    }
    function apply(target) {
        var r = Door.applyMatch(root.seriesId, target)
        if (r.error) { frame.toast(r.error.message); return }
        root.visible = false
    }
    function targetText(t) {
        if (!t) return ""
        if (t.Anilist !== undefined) return "AniList " + t.Anilist.id + (t.Anilist.season !== null && t.Anilist.season !== undefined ? ", season " + t.Anilist.season : "")
        return "MyAnimeList " + t.Mal.id
    }
    function matchText(m) {
        var bits = []
        if (m.anilist_id !== null && m.anilist_id !== undefined) bits.push("AniList " + m.anilist_id)
        if (m.mal_id !== null && m.mal_id !== undefined) bits.push("MyAnimeList " + m.mal_id)
        return bits.join(", ")
    }

    width: theme.space(150)
    height: theme.space(110)
    visible: false
    radius: theme.radiusLg
    smoothing: theme.cornerSmoothing
    color: theme.surfaceRaised
    borderColor: theme.lineStrong
    borderWidth: 1

    onVisibleChanged: if (!visible) { results = []; resolved = null; searching = false }

    Connections {
        target: Door
        function onSearchFinished(job, res) {
            if (job !== root.searchJob) return
            root.searching = false
            root.results = res || []
        }
        function onLinkResolved(job, target) { root.resolved = target }
    }

    Column {
        anchors.fill: parent
        anchors.margins: theme.space(5)
        spacing: theme.space(3)

        Row {
            width: parent.width
            spacing: theme.space(3)
            Text {
                text: "Match"
                color: theme.text
                font.family: theme.fontDisplay
                font.pointSize: theme.typeNormal * 1.15
                font.weight: Font.DemiBold
                anchors.verticalCenter: parent.verticalCenter
            }
            Text {
                text: root.seriesTitle
                color: theme.textDim
                font.family: theme.fontUi
                font.pointSize: theme.typeSmall
                elide: Text.ElideRight
                width: parent.width - closeBtn.width - parent.spacing * 2
                anchors.verticalCenter: parent.verticalCenter
            }
            Icon {
                id: closeBtn
                glyph: "x"
                size: theme.space(4.5)
                color: theme.textDim
                anchors.verticalCenter: parent.verticalCenter
                MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: root.visible = false }
            }
        }

        // What the series is matched to right now, with the clear
        Row {
            visible: root.currentMatch !== null
            spacing: theme.space(3)
            Text {
                text: root.currentMatch ? root.matchText(root.currentMatch) : ""
                color: theme.textDim
                font.family: theme.fontUi
                font.pointSize: theme.typeSmall
                anchors.verticalCenter: parent.verticalCenter
            }
            Button {
                text: "Clear match"
                small: true
                flat: true
                onClicked: {
                    var r = Door.clearMatch(root.seriesId)
                    if (r.error) frame.toast(r.error.message)
                    root.visible = false
                }
            }
        }

        SearchField {
            id: search
            width: parent.width
            placeholder: "Search AniList"
            hint: ""
            onTextChanged: searchDebounce.restart()
        }
        Timer { id: searchDebounce; interval: 300; onTriggered: root.runSearch(search.text) }

        // A pasted link, resolved into a target
        Row {
            width: parent.width
            spacing: theme.space(2)
            Field {
                id: linkField
                width: parent.width - resolveBtn.width - parent.spacing
                placeholder: "Paste an AniList or MAL link"
                mono: true
            }
            Button {
                id: resolveBtn
                anchors.verticalCenter: parent.verticalCenter
                text: "Resolve"
                flat: true
                onClicked: {
                    var r = Door.resolveLink(linkField.text)
                    if (r.error) frame.toast(r.error.message)
                }
            }
        }

        // The resolved target, waiting to be applied
        Row {
            visible: root.resolved !== null
            spacing: theme.space(3)
            Text {
                text: root.resolved ? root.targetText(root.resolved) : ""
                color: theme.text
                font.family: theme.fontUi
                font.pointSize: theme.typeNormal
                anchors.verticalCenter: parent.verticalCenter
            }
            Button {
                text: "Apply"
                small: true
                onClicked: root.apply(root.resolved)
            }
        }

        // The results
        Text {
            visible: root.results.length === 0 && !root.searching
            text: search.text === "" ? "Type to search AniList." : "No titles found."
            color: theme.textFaint
            font.family: theme.fontUi
            font.pointSize: theme.typeSmall
        }
        Column {
            width: parent.width
            spacing: theme.space(2)
            Repeater {
                model: root.results
                Row {
                    id: resultRow
                    required property var modelData
                    width: parent.width
                    spacing: theme.space(3)
                    Corner {
                        width: theme.space(7); height: width * 1.5
                        radius: theme.radiusSm; smoothing: 0
                        color: theme.surfaceSunken
                        anchors.verticalCenter: parent.verticalCenter
                        Image {
                            anchors.fill: parent
                            source: modelData.cover_url || ""
                            fillMode: Image.PreserveAspectCrop
                            asynchronous: true
                        }
                        Text {
                            visible: !modelData.cover_url
                            anchors.centerIn: parent
                            text: modelData.title ? modelData.title.charAt(0) : "?"
                            color: theme.textFaint
                            font.family: theme.fontDisplay
                            font.pointSize: theme.typeNormal
                        }
                    }
                    Column {
                        anchors.verticalCenter: parent.verticalCenter
                        width: resultRow.width - parent.width - applyBtn.width - theme.space(6)
                        Text {
                            width: parent.width
                            text: modelData.title
                            color: theme.text
                            font.family: theme.fontUi
                            font.pointSize: theme.typeNormal
                            font.weight: Font.DemiBold
                            elide: Text.ElideRight
                        }
                        Text {
                            text: [modelData.year, modelData.format, modelData.episodes ? modelData.episodes + " ep" : ""].filter(Boolean).join(", ")
                            color: theme.textDim
                            font.family: theme.fontUi
                            font.pointSize: theme.typeSmall
                        }
                    }
                    Button {
                        id: applyBtn
                        anchors.verticalCenter: parent.verticalCenter
                        text: "Apply"
                        small: true
                        flat: true
                        onClicked: root.apply({ "Anilist": { "id": modelData.id, "season": null } })
                    }
                }
            }
        }
    }
}
