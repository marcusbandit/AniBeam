// The match popover: a raised tile over a Blocker (the owning page puts one under it), to
// search a metadata provider for the series's real record (AniList for anime, TVmaze for
// other television; the Seg opens on the current match's provider, else the first in the
// Settings order) or look up a pasted
// AniList, MyAnimeList, TVmaze or IMDb link, and apply. Search and look-up are jobs whose replies come back as events
// (SearchFinished, LinkResolved), matched here by job id. Applying writes the match
// confirmed at once; the assets follow and SeriesChanged re-lists the page behind. It opens
// onto the frame's escape stack and closes through close(), so Escape, the X, a press on
// the shield and Apply all leave the same way.
//
// A walk through several series (#206, the first scan's "Match the 5") sets walkLine ("2 of
// 5"), which adds Skip beside Close; ended says how the panel left: applied, skipped or
// closed, so the page can open the next one or stop.
import QtQuick
import QtQuick.Controls.Basic as QC
import com.marcusrosado.AniBeam

Corner {
    id: root
    objectName: "match-panel"
    property real seriesId: 0
    property string walkLine: ""
    signal ended(string how)
    property string seriesTitle: ""
    property var currentMatch: null
    property var results: []
    property var resolved: null          // a MatchTarget from a pasted link, waiting to be applied
    property int searchJob: -1
    property int resolveJob: -1
    property bool searching: false
    // The provider the search runs on, as the core names it: "Anilist" or "Tvmaze"
    property string provider: "Anilist"
    readonly property var providers: ["Anilist", "Tvmaze"]
    readonly property string providerName: frame.providerName(provider)

    function openFor(id, title, matchInfo) {
        root.seriesId = id
        root.seriesTitle = title
        root.currentMatch = matchInfo || null
        root.results = []
        root.resolved = null
        var order = Door.settings && Door.settings.provider_order ? Door.settings.provider_order : []
        var current = matchInfo ? matchInfo.provider : ""
        root.provider = root.providers.indexOf(current) >= 0 ? current
            : order.length && root.providers.indexOf(order[0]) >= 0 ? order[0] : "Anilist"
        // An unmatched series is named after its folder: the search starts from that name
        // without its release brackets and episode range
        search.text = matchInfo ? title : Fmt.folderTitle(title)
        searchDebounce.stop()
        visible = true
        frame.escapeStack.push("popover", root)
        search.focusInput()
        runSearch(search.text)
    }
    function close(how) {
        if (!visible) return
        visible = false
        frame.escapeStack.pop(root)
        root.ended(how || "closed")
    }
    Component.onDestruction: if (visible) frame.escapeStack.pop(root)
    function runSearch(q) {
        q = String(q || "").trim()
        if (q.length < 2) { root.searching = false; root.results = []; root.searchJob = -1; return }
        var r = Door.searchOn(root.provider, q, 12)
        if (r.error) { frame.fail(r.error); return }
        root.searchJob = r.reply.job
        root.searching = true
    }
    function apply(target, title) {
        var r = Door.applyMatch(root.seriesId, target)
        if (r.error) { frame.fail(r.error); return }
        frame.toast(title ? words.t("match-matched-to", { title: title }) : words.t("match-applied"))
        root.close("applied")
    }
    // A result row applies by its provider: a TVmaze show through the id-only call, an
    // AniList media as a target with no season
    function applyResult(r) {
        if (r.provider === "Tvmaze") {
            var out = Door.applyTvmaze(root.seriesId, r.id)
            if (out.error) { frame.fail(out.error); return }
            frame.toast(words.t("match-matched-to", { title: r.title }))
            root.close("applied")
        } else root.apply({ "Anilist": { "id": r.id, "season": null } }, r.title)
    }
    // A new provider drops the other's rows and asks again for the same words
    function pickProvider(p) {
        if (p === root.provider) return
        root.provider = p
        root.results = []
        searchDebounce.stop()
        root.runSearch(search.text)
    }
    // Where a looked-up link points and where the series is matched, in words: the ids are
    // the provider's business
    function targetText(t) {
        if (!t) return ""
        if (t.Tvmaze !== undefined) return words.t("match-found-tvmaze")
        if (t.Anilist !== undefined) return t.Anilist.season !== null && t.Anilist.season !== undefined
            ? words.t("match-found-anilist-season", { season: t.Anilist.season }) : words.t("match-found-anilist")
        return words.t("match-found-mal")
    }
    function matchText(m) {
        var bits = []
        if (m.anilist_id !== null && m.anilist_id !== undefined) bits.push("AniList")
        if (m.mal_id !== null && m.mal_id !== undefined) bits.push("MyAnimeList")
        if (m.tvmaze_id !== null && m.tvmaze_id !== undefined) bits.push("TVmaze")
        if (!bits.length) return ""
        var sites = bits.reduce(function(a, b) { return words.t("match-sites-and", { first: a, next: b }) })
        return words.t("match-matched-on", { sites: sites })
    }
    // A result's year, then its format (AniList) or its type as TVmaze writes it, then its length
    function resultMeta(r) {
        return [r.year ? String(r.year) : "",
                r.provider === "Tvmaze" ? (r.format || "") : root.formatLabel(r.format),
                r.episodes ? words.t("count-episodes", { count: r.episodes }) : ""].filter(Boolean).join(", ")
    }
    function formatLabel(f) {
        if (!f) return ""
        var id = ({ TV: "match-format-tv", TV_SHORT: "match-format-tv-short", OVA: "match-format-ova", ONA: "match-format-ona",
                    SPECIAL: "match-format-special", MOVIE: "match-format-film", MUSIC: "match-format-music" })[f]
        return id ? words.t(id) : f.replace(/_/g, " ")
    }

    readonly property real pad: theme.space(5)
    width: Math.min(theme.space(170), (parent ? parent.width : theme.space(170)) - theme.space(16))
    height: Math.min(theme.space(160), (parent ? parent.height : theme.space(160)) - theme.space(16))
    visible: false
    // A floating pane of glass over the metadata wall
    color: theme.glassFloat
    borderColor: theme.lineStrong
    borderWidth: 1
    highlight: theme.glassHighlight

    onVisibleChanged: if (!visible) { results = []; resolved = null; searching = false; searchDebounce.stop() }

    Connections {
        target: Door
        function onSearchFinished(job, res) {
            if (job !== root.searchJob) return
            root.searching = false
            root.results = res || []
        }
        function onLinkResolved(job, target) { if (job === root.resolveJob) root.resolved = target }
        // A search or look-up the provider refused (a rate limit, no network) stops waiting
        function onJobFailed(job, kind, error) {
            if (job === root.searchJob) { root.searching = false; root.searchJob = -1 }
            else if (job !== root.resolveJob) return
            frame.toast(error && error.message ? error.message : words.t("match-provider-silent", { provider: root.providerName }))
        }
    }

    // The panel's own press sink: a press on its padding never reaches the shield under it
    MouseArea { anchors.fill: parent; acceptedButtons: Qt.AllButtons; hoverEnabled: true; onWheel: function(w) { w.accepted = true } }

    Column {
        id: head
        x: root.pad; y: root.pad
        width: root.width - root.pad * 2
        spacing: theme.space(3)

        Item {
            width: parent.width
            height: Math.max(heading.height, closeBtn.height)
            Column {
                id: heading
                width: parent.width - closeBtn.width - theme.space(3) - (root.walkLine !== "" ? theme.space(30) : 0)
                Label { text: root.walkLine !== "" ? words.t("match-title-walk", { walk: root.walkLine }) : words.t("match-title"); role: "title" }
                Label { width: parent.width; text: root.seriesTitle; role: "ui"; color: theme.textDim; elide: Text.ElideRight }
            }
            Button {
                objectName: "match-skip"
                visible: root.walkLine !== ""
                anchors.right: closeBtn.left; anchors.rightMargin: theme.space(2); anchors.top: parent.top
                small: true; icon: "skip-forward"; msg: "match-skip"
                onClicked: root.close("skipped")
            }
            Button {
                id: closeBtn
                objectName: "match-close"
                anchors.right: parent.right; anchors.top: parent.top
                small: true; icon: "x"; msg: "action-close"
                onClicked: root.close()
            }
        }

        // What the series is matched to right now, with the clear
        Row {
            visible: root.currentMatch !== null && root.matchText(root.currentMatch) !== ""
            spacing: theme.space(3)
            Label {
                text: root.currentMatch ? root.matchText(root.currentMatch) : ""
                role: "ui"; color: theme.textDim
                anchors.verticalCenter: parent.verticalCenter
            }
            // Match is one place for the record (#432): fetch its info again from here
            Button {
                objectName: "match-refresh"
                small: true
                icon: "refresh-cw"
                msg: "match-refresh-info"
                onClicked: { var r = Door.refreshSeries(root.seriesId); r.error ? frame.fail(r.error) : frame.toast(words.t("match-refreshing", { title: root.seriesTitle })) }
            }
            // Two steps: it drops the tracker link and a hand-set episode count, and the
            // series is then left alone by the automatic match
            Button {
                id: clearBtn
                property bool armed: false
                text: armed ? words.t("match-clear-confirm") : words.t("match-clear")
                small: true
                danger: armed
                onClicked: {
                    if (!armed) { armed = true; disarm.restart(); return }
                    armed = false
                    var r = Door.clearMatch(root.seriesId)
                    if (r.error) frame.fail(r.error)
                    else frame.toast(words.t("match-cleared"))
                    root.close()
                }
                Timer { id: disarm; interval: 5000; onTriggered: clearBtn.armed = false }
            }
        }

        // Where the search runs, then the words
        Row {
            width: parent.width
            spacing: theme.space(2)
            Seg {
                id: providerSeg
                objectName: "match-provider"
                anchors.verticalCenter: parent.verticalCenter
                options: ["AniList", "TVmaze"]
                index: root.providers.indexOf(root.provider)
                onPicked: function(i) { root.pickProvider(root.providers[i]) }
            }
            SearchField {
                id: search
                objectName: "match-search"
                width: parent.width - providerSeg.width - parent.spacing
                placeholder: words.t("match-search-placeholder", { provider: root.providerName })
                hint: ""
                onTextChanged: if (root.visible) { searchDebounce.restart(); list.currentIndex = -1 }
                // Up and Down walk the results, Return applies the one walked to, else the first
                onStepped: function(by) { if (root.results.length) list.currentIndex = Math.max(0, Math.min(root.results.length - 1, list.currentIndex + by)) }
                onAccepted: if (root.results.length) root.applyResult(root.results[Math.max(0, list.currentIndex)])
            }
        }
        Timer { id: searchDebounce; interval: 300; onTriggered: root.runSearch(search.text) }

        // A pasted link, looked up into a target
        Row {
            width: parent.width
            spacing: theme.space(2)
            Field {
                id: linkField
                width: parent.width - lookUp.width - parent.spacing
                placeholder: root.provider === "Tvmaze" ? words.t("match-link-tvmaze") : words.t("match-link-anilist")
                mono: true
                onEdited: if (linkField.text.trim() !== "" && !root.resolved) lookUp.clicked()
            }
            Button {
                id: lookUp
                anchors.verticalCenter: parent.verticalCenter
                msg: "match-look-up"
                onClicked: {
                    var r = Door.resolveLink(linkField.text)
                    if (r.error) { frame.fail(r.error); return }
                    root.resolveJob = r.reply && r.reply.job !== undefined ? r.reply.job : -1
                }
            }
        }

        // The looked-up target, waiting to be applied
        Row {
            visible: root.resolved !== null
            spacing: theme.space(3)
            Label {
                text: root.resolved ? root.targetText(root.resolved) : ""
                role: "ui"
                anchors.verticalCenter: parent.verticalCenter
            }
            Button { msg: "action-apply"; small: true; primary: true; onClicked: root.apply(root.resolved, "") }
        }

        Label {
            visible: root.results.length === 0
            text: root.searching ? words.t("match-searching", { provider: root.providerName })
                : search.text.trim().length < 2 ? words.t("match-type-more", { provider: root.providerName })
                : words.t("match-no-results", { provider: root.providerName })
            role: "ui"; color: theme.textDim
        }
    }

    // The results: a list of their own, scrolling inside the panel
    ListView {
        id: list
        x: root.pad
        y: head.y + head.height + theme.space(3)
        width: root.width - root.pad * 2
        height: root.height - y - root.pad
        clip: true
        spacing: theme.space(1)
        model: root.results
        boundsBehavior: Flickable.StopAtBounds
        QC.ScrollBar.vertical: ThinScrollBar {}
        delegate: Corner {
            id: result
            required property var modelData
            required property int index
            objectName: "match-result-" + index
            width: list.width
            height: theme.space(18)
            radius: theme.radiusInner(theme.space(1))
            smoothing: theme.cornerSmoothing
            color: resultMouse.pressed ? theme.surfacePressed : resultMouse.containsMouse || ListView.isCurrentItem ? theme.floatHover : "transparent"
            borderColor: ListView.isCurrentItem ? theme.focusRing : "transparent"; borderWidth: ListView.isCurrentItem ? 1 : 0
            // The whole row applies, not only its button
            MouseArea { id: resultMouse; anchors.fill: parent; hoverEnabled: true; cursorShape: Qt.PointingHandCursor; onClicked: root.applyResult(result.modelData); Accessible.role: Accessible.Button; Accessible.name: theme.nameNear(parent)
                Accessible.onPressAction: root.applyResult(result.modelData) }
            Corner {
                id: cover
                x: theme.space(2)
                anchors.verticalCenter: parent.verticalCenter
                height: parent.height - theme.space(2); width: Math.round(height / 1.5)
                radius: theme.radiusSm; smoothing: theme.cornerSmoothing
                color: theme.tileSunken
                fillItem: coverArt.status === Image.Ready ? coverArt : null
                Image {
                    id: coverArt; visible: false
                    width: cover.width; height: cover.height
                    source: result.modelData.cover_url || ""
                    sourceSize: Qt.size(Math.ceil(cover.width * theme.devicePixelRatio), Math.ceil(cover.height * theme.devicePixelRatio))
                    fillMode: Image.PreserveAspectCrop
                    asynchronous: true
                }
            }
            Column {
                anchors.left: cover.right; anchors.leftMargin: theme.space(3)
                anchors.right: applyBtn.left; anchors.rightMargin: theme.space(3)
                anchors.verticalCenter: parent.verticalCenter
                spacing: theme.space(0.5)
                Label { width: parent.width; text: result.modelData.title || ""; role: "title"; font.pointSize: theme.typeNormal; elide: Text.ElideRight }
                Label { visible: !!result.modelData.alt_title; width: parent.width; text: result.modelData.alt_title || ""; role: "ui"; font.pointSize: theme.typeSmall; color: theme.textDim; elide: Text.ElideRight }
                Label {
                    width: parent.width
                    text: root.resultMeta(result.modelData)
                    role: "ledger"; color: theme.textDim; elide: Text.ElideRight
                }
            }
            Button {
                id: applyBtn
                objectName: "match-apply-" + result.index
                anchors.right: parent.right; anchors.rightMargin: theme.space(2)
                anchors.verticalCenter: parent.verticalCenter
                msg: "action-apply"
                small: true
                onClicked: root.applyResult(result.modelData)
            }
        }
    }
}
