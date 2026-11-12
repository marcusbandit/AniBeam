// Recommended for you, whole (#779): Home's row opened out into a grid, best first, with the
// same switch over it and the same hover card. The scope is Home's preference, so a pick
// here holds there too.
import QtQuick
import com.marcusrosado.AniBeam

Item {
    id: page
    objectName: "recommended-page"
    property var props: ({})
    property string title: "Recommended for you"
    property alias scrollY: scroll.contentY
    function escapePressed() { return false }
    Keys.onPressed: function(event) { scroll.keyPressed(event) }

    readonly property real gap: theme.space(4)
    readonly property var scopes: ["All", "InLibrary", "OnList"]
    readonly property string scope: Door.preferences.home_recommended || "All"
    property var recommended: null
    property var refreshing: null
    readonly property var list: recommended ? recommended.shows : []
    readonly property var counts: recommended ? recommended.counts : ({ all: 0, in_library: 0, on_list: 0 })
    // Read off the Qt thread: the read parses the whole ranking
    property int token: -1
    function load() { page.token = Door.callAsync("ListRecommended", { scope: page.scope }) }
    Timer { id: debounce; interval: 500; onTriggered: page.load() }
    function setScope(s) {
        var p = JSON.parse(JSON.stringify(Door.preferences))
        p.home_recommended = s
        var r = Door.setPreferences(p)
        if (r && r.error) frame.fail(r.error)
    }
    function refresh() {
        var r = Door.call("RefreshRecommended", { force: true })
        if (r.error) { frame.fail(r.error); return }
        page.refreshing = r.reply.job !== undefined ? r.reply.job : null
    }
    function openShow(s) {
        if (s.owned) frame.go("series", { id: s.owned }, s.title)
        else frame.go("show", { anilist: s.anilist_id }, s.title)
    }
    onScopeChanged: load()
    Component.onCompleted: load()
    Connections {
        target: Door
        function onEvent(envelope) { if (envelope.kind === "RecommendedRefreshed") { page.refreshing = null; page.load() } }
        function onSeriesChanged() { debounce.restart() }
        function onTrackersChanged() { debounce.restart() }
        // A fit that failed or was stopped says nothing else; Rank again comes back
        function onJobFailed(job) { if (job === page.refreshing) page.refreshing = null }
        function onJobCancelled(job) { if (job === page.refreshing) page.refreshing = null }
        function onCallDone(token, reply) {
            if (token !== page.token) return
            page.token = -1
            if (reply.error) { frame.fail(reply.error, page.load); return }
            page.recommended = reply.reply.recommended
            page.refreshing = reply.reply.refreshing === undefined ? null : reply.reply.refreshing
        }
    }
    readonly property string fittedFact: {
        if (!recommended || !recommended.fitted_at) return ""
        var mins = Math.max(0, Math.floor((Date.now() / 1000 - recommended.fitted_at) / 60))
        return mins < 60 ? "Ranked " + Math.max(1, mins) + "m ago" : mins < 1440 ? "Ranked " + Math.floor(mins / 60) + "h ago" : "Ranked " + Math.floor(mins / 1440) + "d ago"
    }

    RecommendedHover {
        id: hover
        onOpened: function(s) { page.openShow(s) }
    }
    Connections { target: scroll; function onContentYChanged() { hover.close() } }

    PageScroll {
        id: scroll
        anchors.fill: parent

        Button {
            objectName: "recommended-back"
            small: true
            flat: true
            icon: "chevron-left"
            text: frame.nav.backLabel
            onClicked: frame.nav.back()
        }

        Item {
            width: parent.width
            height: theme.space(14)
            Row {
                anchors.left: parent.left
                anchors.bottom: parent.bottom
                anchors.bottomMargin: theme.space(2)
                spacing: theme.space(3)
                Label { id: heading; text: "Recommended for you"; role: "display"; font.pointSize: theme.typeHeading; font.variableAxes: ({ "wdth": 70, "wght": 420 }) }
                Label { anchors.baseline: heading.baseline; text: page.fittedFact; role: "ui"; color: theme.textDim }
            }
            Row {
                anchors.right: parent.right
                anchors.bottom: parent.bottom
                anchors.bottomMargin: theme.space(2)
                spacing: theme.space(3)
                Seg {
                    objectName: "recommended-scope"
                    anchors.verticalCenter: parent.verticalCenter
                    accessibleName: "Recommended from"
                    options: [{ text: "All", name: "all", count: page.counts.all },
                              { text: "In library", name: "library", count: page.counts.in_library },
                              { text: "On list", name: "list", count: page.counts.on_list }]
                    index: Math.max(0, page.scopes.indexOf(page.scope))
                    onPicked: function(i) { page.setScope(page.scopes[i]) }
                }
                Button {
                    objectName: "recommended-refresh"
                    anchors.verticalCenter: parent.verticalCenter
                    small: true
                    icon: "refresh-cw"
                    text: page.refreshing !== null ? "Ranking" : "Rank again"
                    enabled: page.refreshing === null
                    onClicked: page.refresh()
                }
            }
        }

        EmptyState {
            visible: page.recommended !== null && page.list.length === 0
            width: parent.width
            icon: "sparkles"
            title: page.recommended && page.recommended.state === "NoScores" ? "Rate a few shows first"
                : page.recommended && page.recommended.state === "NoList" ? "Nothing to learn from yet"
                : page.recommended && page.recommended.state === "NeverFitted" ? "Ranking your shows"
                : "Nothing left here"
            body: page.recommended && page.recommended.state === "NoScores" ? "Recommendations learn from the scores on your list. Score a few shows on AniList and they appear here."
                : page.recommended && page.recommended.state === "NoList" ? "Connect AniList in Settings, or keep a list on this device, and the shows you rate teach this page."
                : page.recommended && page.recommended.state === "NeverFitted" ? "The first ranking reads AniList for a minute or two. It runs in the background."
                : page.scope === "InLibrary" ? "Everything in your library that fits is watched. All finds shows you do not have yet."
                : "Everything on your lists that fits is watched. All finds shows you have not listed."
        }

        Flow {
            id: grid
            width: parent.width
            spacing: page.gap
            readonly property int cols: Math.max(1, Math.floor((width + page.gap) / (theme.posterWidth + page.gap)))
            readonly property real cardWidth: Math.floor((width - page.gap * (cols - 1)) / cols)
            Repeater {
                model: page.list
                RecommendedCard {
                    id: tile
                    required property var modelData
                    objectName: "rec-" + modelData.anilist_id
                    rec: modelData
                    posterWidth: grid.cardWidth
                    onOpened: { hover.close(); page.openShow(modelData) }
                    onRested: function(on) { on ? hover.enter(tile, modelData) : hover.leave(tile) }
                }
            }
        }
    }
}
