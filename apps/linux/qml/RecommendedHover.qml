// The hover card of a recommended show (#779), opened beside its tile when the pointer rests
// on it, or keyboard focus does, for the tooltip's delay: the shared 600 ms, and at once
// while the card is up or was a moment ago, the way tooltips go warm, so the neighbours open
// as the pointer runs along the row. It closes once the pointer has left both the tile and
// the card.
//
// The card holds the evidence first: the three shows of the user's it is most like, as large
// posters with their score, then one line about the show, the forecast and the facts, its
// genres and tags. A spoiler tag the user loves is behind a dashed "A hidden tag you love"
// and nothing ever names it; the core hands over only how many there are.
//
// One per page, on the frame's overlay. The page's tiles call `enter` and `leave`; a click on
// the card or a poster in it says `opened`, with the show to open.
import QtQuick
import QtQuick.Shapes
import com.marcusrosado.AniBeam

Item {
    id: root
    objectName: "rec-hover"
    parent: frame.overlay
    anchors.fill: parent
    z: 900
    // Only the card itself takes the pointer; the rest of the overlay lets it through
    enabled: shown

    property Item target: null
    property var rec: null
    property var pending: null
    property bool shown: false
    property real warmUntil: 0
    // { anilist_id, owned, title } of the show to open
    signal opened(var show)

    function enter(tile, r) {
        if (root.shown && root.target === tile) { closeSoon.stop(); return }
        root.target = tile
        root.pending = r
        closeSoon.stop()
        if (root.shown || Date.now() < root.warmUntil) root.openNow()
        else intent.restart()
    }
    function leave(tile) {
        if (tile !== root.target) return
        intent.stop()
        if (root.shown) closeSoon.restart()
    }
    function openNow() {
        if (!root.target || !root.pending) return
        root.rec = root.pending
        root.shown = true
        root.place()
    }
    function close() {
        intent.stop(); closeSoon.stop()
        if (root.shown) root.warmUntil = Date.now() + 1500
        root.shown = false
        root.target = null
    }
    // Beside the tile: to its right when there is room, else to its left, its top in line with
    // the tile's and held inside the window
    function place() {
        if (!root.target) return
        var p = root.target.mapToItem(root, 0, 0)
        var gap = theme.space(3), margin = theme.space(2)
        var right = p.x + root.target.width + gap
        card.x = right + card.width + margin <= root.width ? right : Math.max(margin, p.x - card.width - gap)
        card.y = Math.max(margin, Math.min(p.y, root.height - card.height - margin))
    }
    Timer { id: intent; interval: 600; onTriggered: root.openNow() }
    // Room to cross the gap between the tile and the card
    Timer { id: closeSoon; interval: 180; onTriggered: if (!cardHover.hovered) root.close() }
    // A tile scrolled away or torn down takes its card with it
    Connections {
        target: root.target
        ignoreUnknownSignals: true
        function onXChanged() { root.close() }
        function onYChanged() { root.close() }
    }
    onWidthChanged: if (shown) place()
    onHeightChanged: if (shown) place()

    readonly property var r: rec || ({})
    readonly property color tint: r.color ? r.color : theme.accent
    readonly property real posterW: theme.space(30)

    function lengthOf(x) {
        if (x.format === "MOVIE" && x.duration) return Fmt.runtime(x.duration)
        if (x.episodes) return words.t("count-episodes", { count: x.episodes })
        return ""
    }
    function formatOf(f) {
        var id = ({ TV: "recommended-hover-format-tv", TV_SHORT: "recommended-hover-format-tv-short", MOVIE: "recommended-hover-format-movie", SPECIAL: "recommended-hover-format-special",
                  OVA: "recommended-hover-format-ova", ONA: "recommended-hover-format-ona", MUSIC: "recommended-hover-format-music" })[f]
        return id ? words.t(id) : ""
    }
    readonly property string facts: {
        var f = []
        if (r.average_score !== null && r.average_score !== undefined) f.push(words.t("recommended-hover-anilist-score", { score: (r.average_score / 10).toFixed(1) }))
        if (formatOf(r.format)) f.push(formatOf(r.format))
        if (r.year) f.push(String(r.year))
        if (lengthOf(r)) f.push(lengthOf(r))
        return f.join("  ")
    }

    Frost {
        id: card
        objectName: "rec-hover-card"
        visible: root.shown
        opacity: root.shown ? 1 : 0
        width: Math.max(theme.space(110), root.posterW * 3 + theme.space(3) * 2 + pad * 2)
        height: body.implicitHeight + pad * 2
        readonly property real pad: theme.space(5)
        radius: theme.radiusTile
        color: theme.glassFloat
        borderColor: theme.lineStrong
        // The cover's own colour, laid thin over the glass
        Rectangle {
            anchors.fill: parent
            radius: parent.radius; antialiasing: true
            color: Qt.rgba(root.tint.r, root.tint.g, root.tint.b, 0.12)
        }
        HoverHandler {
            id: cardHover
            onHoveredChanged: if (hovered) closeSoon.stop(); else closeSoon.restart()
        }
        MouseArea {
            anchors.fill: parent
            cursorShape: Qt.PointingHandCursor
            onClicked: { var s = root.r; root.close(); root.opened(s) }
        }

        Column {
            id: body
            x: card.pad; y: card.pad
            width: card.width - card.pad * 2
            spacing: theme.space(3)

            Row {
                width: parent.width
                spacing: theme.space(3)
                Label {
                    id: big
                    msg: "recommended-hover-forecast"
                    msgArgs: ({ percent: root.r.forecast !== undefined ? root.r.forecast : 0 })
                    role: "display"
                    font.pointSize: theme.typeHeading
                    color: theme.accent
                }
                Column {
                    width: parent.width - big.width - parent.spacing
                    anchors.verticalCenter: big.verticalCenter
                    spacing: theme.space(0.5)
                    Label { width: parent.width; text: root.r.title || ""; role: "title"; wrapMode: Text.Wrap; maximumLineCount: 2; elide: Text.ElideRight }
                    Label { width: parent.width; visible: text !== ""; text: root.facts; role: "ledger"; color: theme.textDim; elide: Text.ElideRight }
                }
            }

            Label {
                width: parent.width
                visible: text !== ""
                text: root.r.line || ""
                role: "ui"
                color: theme.textDim
                wrapMode: Text.Wrap
                maximumLineCount: 3
                elide: Text.ElideRight
            }

            Flow {
                width: parent.width
                spacing: theme.space(1.5)
                visible: chips.count > 0 || hidden.visible
                Repeater {
                    id: chips
                    model: (root.r.genres || []).concat(root.r.tags || []).slice(0, 7)
                    Chip { required property string modelData; text: modelData; small: true; mono: false }
                }
                // Counted, never named
                Item {
                    id: hidden
                    objectName: "rec-hover-hidden"
                    visible: (root.r.hidden_loved || 0) > 0
                    width: hiddenLabel.implicitWidth + theme.space(2.5) * 2
                    height: hiddenLabel.implicitHeight + theme.space(0.75) * 2
                    Shape {
                        anchors.fill: parent
                        preferredRendererType: Shape.CurveRenderer
                        ShapePath {
                            strokeColor: theme.accent
                            strokeWidth: 1
                            strokeStyle: ShapePath.DashLine
                            dashPattern: [3, 2]
                            fillColor: "transparent"
                            PathRectangle { x: 0.5; y: 0.5; width: hidden.width - 1; height: hidden.height - 1; radius: theme.radiusSm }
                        }
                    }
                    Label {
                        id: hiddenLabel
                        anchors.centerIn: parent
                        msg: "recommended-hover-hidden-tags"
                        msgArgs: ({ count: root.r.hidden_loved || 0 })
                        role: "ui"; font.pointSize: theme.typeSmall
                        color: theme.accent
                    }
                }
            }

            Column {
                width: parent.width
                spacing: theme.space(2)
                visible: (root.r.because || []).length > 0
                Label { msg: "recommended-hover-because"; role: "ui"; font.pointSize: theme.typeSmall; color: theme.textFaint }
                Row {
                    spacing: theme.space(3)
                    Repeater {
                        model: root.r.because || []
                        Column {
                            id: yours
                            required property var modelData
                            width: root.posterW
                            spacing: theme.space(1.5)
                            Corner {
                                width: root.posterW
                                height: Math.round(root.posterW * 1.5)
                                radius: theme.radiusTile; smoothing: 0
                                color: theme.tile
                                fillItem: cover.status === Image.Ready ? cover : null
                                Image {
                                    id: cover
                                    visible: false
                                    width: parent.width; height: parent.height
                                    source: yours.modelData.poster ? Fmt.fileUrl(yours.modelData.poster) : ""
                                    fillMode: Image.PreserveAspectCrop; asynchronous: true
                                    sourceSize: theme.posterDecodeSize
                                }
                                // Their score, as the list has it
                                Rectangle {
                                    visible: yours.modelData.score !== null && yours.modelData.score !== undefined
                                    anchors.right: parent.right; anchors.bottom: parent.bottom; anchors.margins: theme.space(1.5)
                                    width: score.implicitWidth + theme.space(3); height: score.implicitHeight + theme.space(1.5)
                                    radius: theme.radiusSm; antialiasing: true
                                    color: Qt.rgba(8 / 255, 9 / 255, 11 / 255, 0.7)
                                    border.color: Qt.rgba(1, 1, 1, 0.14); border.width: 1
                                    Label { id: score; anchors.centerIn: parent; role: "ledger"; text: yours.modelData.score !== null && yours.modelData.score !== undefined ? Number(yours.modelData.score).toFixed(1) : ""; color: theme.onPoster }
                                }
                                MouseArea {
                                    anchors.fill: parent
                                    cursorShape: Qt.PointingHandCursor
                                    onClicked: { var s = yours.modelData; root.close(); root.opened(s) }
                                }
                            }
                            Label { width: parent.width; text: yours.modelData.title; role: "ui"; font.pointSize: theme.typeSmall; wrapMode: Text.Wrap; maximumLineCount: 2; elide: Text.ElideRight }
                        }
                    }
                }
            }
        }
    }
}
