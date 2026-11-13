// The page for one AniList show the library does not have (#779): everything the core holds
// for it, on the Series page's band. It says plainly that the library does not have it, and
// Get episodes opens Downloads' Get view with the title filled in; nothing downloads until
// a release is picked. The list status pill and the score work as on any series, through the
// tracker. The first opening reads AniList once for the relations, AniList's own
// recommendations and the trailer; the page fills in when that read lands.
//
// Once files for the show land in the library, the page becomes its Series page in place.
//
// props: { anilist } - the AniList id.
import QtQuick
import com.marcusrosado.AniBeam

Item {
    id: page
    objectName: "show-page"
    property var props: ({})
    property string title: show ? show.title : words.t("show-title")
    property alias scrollY: scroll.contentY
    function escapePressed() { return false }
    Keys.onPressed: function(event) { scroll.keyPressed(event) }

    readonly property int anilistId: props.anilist || 0
    property var show: null
    property var reading: null
    function load() {
        var r = Door.call("GetShowPage", { anilist_id: page.anilistId })
        if (r.error) { frame.fail(r.error, page.load); return }
        page.show = r.reply.page
        if (r.reply.reading !== undefined && r.reply.reading !== null) page.reading = r.reply.reading
        // In the library now: this is its series page
        if (page.show.owned) frame.nav.replace("series", { id: page.show.owned }, page.show.title)
    }
    Component.onCompleted: load()
    Connections {
        target: Door
        function onEvent(envelope) {
            if (envelope.kind === "ShowPageRead" && envelope.body.anilist_id === page.anilistId) { page.reading = null; page.load() }
            if (envelope.kind === "RecommendedRefreshed") page.load()
        }
        // Only to see the show land in the library; the core never reads AniList twice in a row for it
        function onSeriesChanged() { seriesDebounce.restart() }
        function onJobFinished(job) { if (page.writing !== null && job === page.writing) { page.writing = null; page.load() } }
        function onJobFailed(job) { if (job === page.reading) page.reading = null; if (job === page.writing) page.writing = null }
        function onJobCancelled(job) { if (job === page.reading) page.reading = null; if (job === page.writing) page.writing = null }
    }
    property var writing: null
    Timer { id: seriesDebounce; interval: 500; onTriggered: page.load() }

    readonly property var s: show || ({})
    function formatLabel(f) {
        var id = ({ TV: "show-format-tv", TV_SHORT: "show-format-tv-short", MOVIE: "show-format-film", SPECIAL: "show-format-special", OVA: "show-format-ova",
                   ONA: "show-format-ona", MUSIC: "show-format-music" })[f]
        return id ? words.t(id) : ""
    }
    function statusLabel(st) {
        var id = ({ FINISHED: "show-status-finished", RELEASING: "show-status-airing", NOT_YET_RELEASED: "show-status-not-yet-aired", CANCELLED: "show-status-cancelled",
                   HIATUS: "show-status-hiatus" })[st]
        return id ? words.t(id) : ""
    }
    readonly property string factsLine: {
        var f = []
        if (formatLabel(s.format)) f.push(formatLabel(s.format))
        if (s.year) f.push(String(s.year))
        if (s.studios && s.studios.length) f.push(s.studios[0])
        if (s.format === "MOVIE" && s.duration) f.push(Fmt.runtime(s.duration))
        else {
            if (s.episodes) f.push(words.t("count-episodes", { count: s.episodes }))
            if (s.duration) f.push(words.t("show-runtime-each", { runtime: Fmt.runtime(s.duration) }))
        }
        if (statusLabel(s.status)) f.push(statusLabel(s.status))
        return f.join("  ")
    }
    readonly property var otherTitles: {
        var out = []
        var names = [s.romaji, s.english, s.native]
        for (var i = 0; i < names.length; i++) if (names[i] && names[i] !== s.title && out.indexOf(names[i]) < 0) out.push(names[i])
        return out
    }
    readonly property bool tracked: !!Door.trackers && ((!!Door.trackers.anilist && Door.trackers.anilist.connected === true)
                                                      || (!!Door.trackers.mal && Door.trackers.mal.connected === true))
    readonly property bool hasScore: s.score !== null && s.score !== undefined
    function listIcon(st) { return { Watching: "play", Repeating: "rotate-ccw", Completed: "circle-check", Paused: "pause", Dropped: "ban", Planning: "bookmark" }[st] || "bookmark" }
    function listText(st) {
        var id = { Watching: "status-watching", Repeating: "status-rewatching", Completed: "status-completed", Paused: "status-on-hold",
                   Dropped: "status-dropped", Planning: "status-planning" }[st]
        return words.t(id || "show-add-to-list")
    }

    function setStatus(status, label) {
        var r = Door.call("SetShowListStatus", { anilist_id: page.anilistId, status: status })
        if (r.error) { frame.fail(r.error); return }
        if (r.reply && r.reply.job !== undefined) { page.writing = r.reply.job; frame.toast(words.t("show-moving-to", { list: label })) }
        else { frame.toast(words.t("show-moved-to", { list: label })); page.load() }
    }
    function statusItems() {
        var lists = [{ status: "Watching", msg: "status-watching", icon: "play" },
                     { status: "Planning", msg: "status-planning", icon: "bookmark" },
                     { status: "Completed", msg: "status-completed", icon: "circle-check" },
                     { status: "Paused", msg: "status-on-hold", icon: "pause" },
                     { status: "Dropped", msg: "status-dropped", icon: "ban" }]
        return lists.map(function(l) {
            var text = words.t(l.msg)
            return { text: text, icon: l.icon, enabled: l.status !== page.s.list_status,
                     action: function() { page.setStatus(l.status, text) } }
        })
    }
    function setScore(v) {
        var r = Door.call("SetShowScore", { anilist_id: page.anilistId, score: v })
        if (r.error) { frame.fail(r.error); return }
        if (r.reply && r.reply.job !== undefined) page.writing = r.reply.job
        else page.load()
    }
    function getEpisodes() {
        frame.go("downloads", { hit: { id: page.anilistId, title: s.title, romaji: s.romaji || "", english: s.english || "", native: s.native || "",
                                       cover_url: s.poster || "", episodes: s.episodes || 0, format: s.format || "", year: s.year || 0 } }, words.t("show-downloads-title"))
    }
    function open(x) {
        if (x.owned) frame.go("series", { id: x.owned }, x.title)
        else frame.go("show", { anilist: x.anilist_id }, x.title)
    }
    function relationLabel(k) {
        var id = ({ PREQUEL: "show-relation-prequel", SEQUEL: "show-relation-sequel", PARENT: "show-relation-parent", SIDE_STORY: "show-relation-side-story",
                   SPIN_OFF: "show-relation-spin-off", ALTERNATIVE: "show-relation-alternative", SUMMARY: "show-relation-summary",
                   COMPILATION: "show-relation-compilation", SOURCE: "show-relation-source", ADAPTATION: "show-relation-adaptation" })[k]
        return words.t(id || "show-relation-related")
    }

    // ---- The band's colour, the way the Series page reads it off the poster
    PosterTone { id: toneOf; source: page.s.poster ? Fmt.fileUrl(page.s.poster) : "" }
    readonly property color bandBase: theme.bandBase(toneOf.tone, page.s.poster ? 0.5 : 0.25)
    readonly property color inkAccent: theme.bandAccent(toneOf.tone, bandBase)
    readonly property color ink: "#f4f6f8"
    readonly property color inkDim: Qt.rgba(1, 1, 1, 0.68)
    readonly property bool showArt: !!page.s.banner && band.width >= theme.space(275)

    component Heading: Item {
        property string text: ""
        property string fact: ""
        width: parent ? parent.width : 0
        height: theme.space(10)
        Row {
            anchors.left: parent.left
            anchors.bottom: parent.bottom
            anchors.bottomMargin: theme.space(1)
            spacing: theme.space(3)
            Label { id: headingText; text: parent.parent.text; role: "display"; font.pointSize: theme.typeHeading; font.variableAxes: ({ "wdth": 70, "wght": 420 }) }
            Label { anchors.baseline: headingText.baseline; text: parent.parent.fact; role: "ui"; color: theme.textDim }
        }
    }
    // A fact in the band's quiet row, outlined since it reads rather than acts; `menu` adds
    // the chevron of a pill that opens one
    component Fact: Corner {
        id: fact
        property string icon: ""
        property string text: ""
        property bool menu: false
        property bool clickable: menu
        signal clicked()
        implicitWidth: factContent.implicitWidth + theme.space(3) * 2
        implicitHeight: theme.space(6)
        radius: height / 2
        smoothing: theme.cornerSmoothing
        color: factMouse.containsMouse && fact.clickable ? Qt.rgba(1, 1, 1, 0.08) : "transparent"
        borderColor: Qt.rgba(1, 1, 1, 0.22)
        borderWidth: 1
        Row {
            id: factContent
            anchors.centerIn: parent
            spacing: theme.space(1.5)
            Icon { visible: fact.icon !== ""; anchors.verticalCenter: parent.verticalCenter; glyph: fact.icon; color: Qt.rgba(1, 1, 1, 0.82); size: theme.space(3.5) }
            Label { anchors.verticalCenter: parent.verticalCenter; text: fact.text; role: "ui"; font.pointSize: theme.typeSmall; color: Qt.rgba(1, 1, 1, 0.82) }
            Icon { visible: fact.menu; anchors.verticalCenter: parent.verticalCenter; glyph: "chevron-down"; color: Qt.rgba(1, 1, 1, 0.82); size: theme.space(3) }
        }
        MouseArea {
            id: factMouse
            anchors.fill: parent
            enabled: fact.clickable
            hoverEnabled: true
            cursorShape: Qt.PointingHandCursor
            Accessible.role: Accessible.Button
            Accessible.name: fact.text
            onClicked: fact.clicked()
        }
    }
    // A row of small posters that open their show
    component PosterRow: Flow {
        id: posterRow
        property var items: []
        property bool labelled: false
        width: parent ? parent.width : 0
        spacing: theme.space(3)
        Repeater {
            model: posterRow.items
            Column {
                required property var modelData
                spacing: theme.space(1)
                RecommendationCard {
                    objectName: "show-poster-" + modelData.anilist_id
                    rec: modelData
                    onOpened: page.open(modelData)
                }
                Label {
                    visible: posterRow.labelled
                    width: theme.space(32)
                    text: page.relationLabel(modelData.relation) + (modelData.format ? "  " + page.formatLabel(modelData.format) : "")
                    role: "ledger"; color: theme.textDim
                    elide: Text.ElideRight
                }
            }
        }
    }

    PageScroll {
        id: scroll
        anchors.fill: parent

        Button {
            objectName: "show-back"
            small: true
            flat: true
            icon: "chevron-left"
            text: frame.nav.backLabel
            onClicked: frame.nav.back()
        }

        // ---- The band: the poster, the names, the facts and the list, the banner fading in
        Item {
            id: band
            width: parent.width
            readonly property real pad: theme.space(8)
            readonly property real posterH: theme.space(72)
            height: Math.max(posterH, bandText.implicitHeight) + pad * 2

            Rectangle { anchors.fill: parent; radius: theme.radiusXl; color: page.bandBase; antialiasing: true }
            Item {
                anchors.fill: parent
                clip: true
                Corner {
                    visible: page.showArt
                    x: Math.round(parent.width * 0.3); y: 0
                    width: parent.width - x; height: parent.height
                    radius: theme.radiusXl
                    color: "transparent"
                    fillItem: page.showArt && art.status === Image.Ready ? art : null
                    Image {
                        id: art; visible: false
                        width: parent.width; height: parent.height
                        source: page.s.banner ? Fmt.fileUrl(page.s.banner) : ""
                        fillMode: Image.PreserveAspectCrop; asynchronous: true
                    }
                }
                Rectangle {
                    visible: page.showArt
                    anchors.fill: parent
                    radius: theme.radiusXl
                    antialiasing: true
                    gradient: Gradient {
                        orientation: Gradient.Horizontal
                        GradientStop { position: 0.33; color: page.bandBase }
                        GradientStop { position: 0.55; color: Qt.rgba(page.bandBase.r, page.bandBase.g, page.bandBase.b, 0.9) }
                        GradientStop { position: 0.75; color: Qt.rgba(page.bandBase.r, page.bandBase.g, page.bandBase.b, 0.45) }
                        GradientStop { position: 1; color: Qt.rgba(page.bandBase.r, page.bandBase.g, page.bandBase.b, 0.1) }
                    }
                }
            }

            Corner {
                id: posterBox
                x: band.pad
                anchors.verticalCenter: parent.verticalCenter
                height: band.posterH
                width: Math.round(height * 2 / 3)
                radius: theme.radiusTile
                color: Qt.rgba(0, 0, 0, 0.3)
                fillItem: page.s.poster && poster.status === Image.Ready ? poster : null
                Image {
                    id: poster; visible: false
                    anchors.fill: parent
                    source: page.s.poster ? Fmt.fileUrl(page.s.poster) : ""
                    sourceSize: Qt.size(theme.posterDecodeSize.width * 2, theme.posterDecodeSize.height * 2)
                    fillMode: Image.PreserveAspectCrop; asynchronous: true
                }
                Label { visible: !page.s.poster; anchors.centerIn: parent; msg: "show-no-poster"; color: page.inkDim; role: "ui"; font.pointSize: theme.typeSmall }
            }

            Column {
                id: bandText
                anchors.left: posterBox.right
                anchors.leftMargin: theme.space(8)
                anchors.right: parent.right
                anchors.rightMargin: band.pad
                anchors.verticalCenter: parent.verticalCenter
                spacing: theme.space(2.5)

                Label {
                    width: parent.width
                    text: page.s.title || (page.s.known === false ? words.t("show-reading-anilist") : "")
                    role: "display"
                    color: page.ink
                    wrapMode: Text.Wrap
                    maximumLineCount: 2
                    elide: Text.ElideRight
                    lineHeight: 0.92
                }
                Label {
                    width: parent.width
                    visible: page.otherTitles.length > 0
                    text: page.otherTitles.join("   ")
                    role: "ui"
                    color: page.inkDim
                    elide: Text.ElideRight
                }
                Label {
                    width: parent.width
                    visible: text !== ""
                    text: page.factsLine
                    role: "ledger"
                    color: page.inkDim
                    wrapMode: Text.Wrap
                }

                // Not in the library, said plainly, and the way to get it
                Row {
                    objectName: "show-not-in-library"
                    spacing: theme.space(3)
                    Icon { anchors.verticalCenter: parent.verticalCenter; glyph: "hard-drive"; color: page.inkAccent; size: theme.space(4) }
                    Label { anchors.verticalCenter: parent.verticalCenter; msg: "show-not-in-library"; role: "ui"; color: page.ink }
                }
                Flow {
                    width: parent.width
                    spacing: theme.space(2)
                    Button {
                        objectName: "show-get"
                        primary: true
                        icon: "download"
                        msg: "show-get-episodes"
                        onClicked: page.getEpisodes()
                    }
                    Button {
                        objectName: "show-trailer"
                        visible: !!page.s.trailer
                        ghost: true
                        icon: "clapperboard"
                        msg: "show-trailer"
                        onClicked: frame.openLink(page.s.trailer)
                    }
                    Button {
                        objectName: "show-site"
                        ghost: true
                        icon: "external-link"
                        text: "AniList"
                        onClicked: frame.openLink(page.s.site_url || ("https://anilist.co/anime/" + page.anilistId))
                    }
                }

                // The quiet row: the list, your score, AniList's and the forecast
                Flow {
                    width: parent.width
                    spacing: theme.space(2)
                    Fact {
                        id: listFact
                        objectName: "show-list"
                        icon: page.listIcon(page.s.list_status)
                        text: page.listText(page.s.list_status)
                        menu: true
                        onClicked: {
                            if (!page.tracked) { frame.openConnect("Anilist"); return }
                            var p = listFact.mapToItem(null, 0, listFact.height + theme.space(1))
                            frame.openMenu(p.x, p.y, page.statusItems())
                        }
                    }
                    Fact {
                        id: rateFact
                        objectName: "show-score"
                        icon: "star"
                        text: page.hasScore ? words.t("show-your-score", { score: Number(page.s.score).toFixed(1) }) : words.t("show-rate-it")
                        clickable: true
                        onClicked: {
                            if (!page.tracked) { frame.openConnect("Anilist"); return }
                            scorePicker.openAt(rateFact, page.hasScore ? page.s.score : -1)
                        }
                    }
                    Fact {
                        visible: page.s.average_score !== null && page.s.average_score !== undefined
                        text: words.t("show-anilist-score", { score: (page.s.average_score / 10).toFixed(1) })
                    }
                    Fact {
                        objectName: "show-forecast"
                        visible: page.s.forecast !== null && page.s.forecast !== undefined
                        icon: "sparkles"
                        text: words.t("show-forecast", { percent: String(page.s.forecast) })
                    }
                }
            }
        }

        Item { width: 1; height: theme.space(4) }

        // ---- Because you watched: the user's own shows this one is most like
        Column {
            objectName: "show-because"
            width: parent.width
            spacing: theme.space(3)
            visible: (page.s.because || []).length > 0 || (page.s.hidden_loved || 0) > 0
            Heading { text: words.t("show-because-you-watched"); fact: (page.s.hidden_loved || 0) > 0 ? words.t("show-hidden-loved", { count: page.s.hidden_loved }) : "" }
            Flow {
                width: parent.width
                spacing: theme.space(4)
                Repeater {
                    model: page.s.because || []
                    Column {
                        required property var modelData
                        spacing: theme.space(1.5)
                        RecommendationCard {
                            width: theme.space(40)
                            rec: { "anilist_id": modelData.anilist_id, "title": modelData.title, "poster": modelData.poster, "owned": modelData.owned, "list_status": "" }
                            onOpened: page.open(modelData)
                        }
                        Label {
                            text: modelData.score !== null && modelData.score !== undefined ? words.t("show-you-gave-it", { score: Number(modelData.score).toFixed(1) }) : ""
                            role: "ledger"; color: theme.textDim
                        }
                    }
                }
            }
            Item { width: 1; height: theme.space(2) }
        }

        // ---- The story
        Column {
            width: parent.width
            spacing: theme.space(3)
            visible: !!page.s.synopsis
            Heading { text: words.t("show-story") }
            Label {
                width: Math.min(parent.width, theme.space(220))
                text: page.s.synopsis || ""
                role: "ui"
                color: theme.textDim
                wrapMode: Text.Wrap
            }
            Item { width: 1; height: theme.space(2) }
        }

        // ---- Genres and tags; a spoiler tag is never among them
        Column {
            width: parent.width
            spacing: theme.space(3)
            visible: (page.s.genres || []).length + (page.s.tags || []).length > 0
            Heading { text: words.t("show-genres-tags") }
            Flow {
                width: parent.width
                spacing: theme.space(1.5)
                Repeater { model: page.s.genres || []; Chip { required property string modelData; text: modelData; mono: false } }
                Repeater { model: (page.s.tags || []).slice(0, 16); Chip { required property var modelData; text: modelData.name; mono: false; hint: modelData.rank ? modelData.rank + "%" : "" } }
            }
            Label {
                visible: (page.s.studios || []).length > 0
                text: words.t("show-studios", { studios: (page.s.studios || []).join(", ") })
                role: "ui"; color: theme.textDim
            }
            Item { width: 1; height: theme.space(2) }
        }

        // ---- Related shows, prequels and sequels first
        Column {
            objectName: "show-relations"
            width: parent.width
            spacing: theme.space(3)
            visible: (page.s.relations || []).length > 0
            Heading { text: words.t("show-related") }
            PosterRow { items: page.s.relations || []; labelled: true }
            Item { width: 1; height: theme.space(2) }
        }

        // ---- AniList's own recommendations
        Column {
            objectName: "show-recommendations"
            width: parent.width
            spacing: theme.space(3)
            visible: (page.s.recommendations || []).length > 0
            Heading { text: words.t("show-recommendations"); fact: words.t("show-from-anilist") }
            PosterRow { items: page.s.recommendations || [] }
        }

        Label {
            visible: page.reading !== null
            text: words.t("show-reading-rest")
            role: "ui"; color: theme.textFaint
        }
    }

    ScorePicker { id: scorePicker; parent: frame.overlay
        onSaved: function(v) { page.setScore(v) }
        onCleared: page.setScore(0) }
}
