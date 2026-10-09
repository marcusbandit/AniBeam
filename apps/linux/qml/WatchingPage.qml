// The watching page: the user's AniList list, CURRENT and REPEATING. The core
// owns the cache and the refresh (ListWatching replies the cached list at
// once plus the id of a RefreshWatching job when it started one;
// WatchingRefreshed brings the fresh list). An entry the library owns opens
// the series in-app; the rest are ghost cards that open AniList.
import QtQuick
import com.marcusrosado.AniBeam

PageScroll {
    id: page
    property var props: ({})
    property string title: "Watching"
    property real scrollY: 0
    function escapePressed() { return false }

    property var entries: []
    property bool anilistConnected: false
    property bool loadFailed: false
    property real nowMs: Date.now()

    function entryItem(e) {
        return {
            title: e.title,
            poster: e.poster || "",
            watched: e.progress > 0 ? e.progress : null,
            total_episodes: e.total || null,
            total_is_estimate: false,
            watched_state: "Unknown",
            strip: null,
            code: e.progress > 0 ? "EP " + String(Math.floor(e.progress)).padStart(2, "0") : "",
            community_score: null,
            my_score: null,
            hidden: false,
            next_airing: null,
            episodes_on_disk: e.owned ? 0 : 0,
            site_url: e.owned ? "" : "https://anilist.co/anime/" + e.anilist_id
        }
    }
    function updatedText(e) {
        return e.updated_at ? "AniList updated " + Fmt.relative(e.updated_at, nowMs / 1000) : ""
    }

    function reload() {
        var t = Door.getTrackers()
        if (!t.error) page.anilistConnected = !!(t.reply.state.anilist.connected)
        var r = Door.listWatching()
        if (r.error) { page.loadFailed = true; frame.toast(r.error.message); return }
        page.loadFailed = false
        page.entries = r.reply.list.entries || []
    }
    Timer { id: debounce; interval: 250; onTriggered: page.reload() }
    Timer { interval: 30000; running: page.entries.length > 0; repeat: true; onTriggered: page.nowMs = Date.now() }
    Connections {
        target: Door
        function onWatchingRefreshed(list) { page.entries = list.entries || [] }
        function onSeriesChanged(changed) { debounce.restart() }
        function onTrackersChanged() { page.reload() }
    }
    Component.onCompleted: reload()

    // The head
    Item {
        id: head
        width: parent.width
        height: titleText.height + sub.height + theme.space(1)
        Text {
            id: titleText
            anchors.left: parent.left; anchors.top: parent.top
            text: "Watching"
            color: theme.text
            font.family: theme.fontDisplay
            font.pointSize: theme.typeLarge
            font.weight: Font.DemiBold
        }
        Text {
            id: sub
            anchors.left: parent.left; anchors.top: titleText.bottom; anchors.topMargin: theme.space(1)
            text: "Your AniList list, CURRENT and REPEATING. A quiet card lives only there."
            color: theme.textDim
            font.family: theme.fontUi
            font.pointSize: theme.typeSmall
        }
    }

    // EmptyState centers itself, and an anchored child inside a positioner
    // breaks the positioner's layout, so each one sits in a plain wrapper the
    // Column can position.
    Item {
        width: parent.width
        height: emptyNotConnected.height
        visible: emptyNotConnected.visible
        EmptyState {
            id: emptyNotConnected
            visible: page.anilistConnected === false
            icon: "log-in"
            title: "AniList is not connected"
            body: "Connect AniList in Settings, under Trackers, to see your watching list here."
            Button { text: "Open Settings"; icon: "settings"; flat: true; onClicked: frame.go("settings") }
        }
    }

    // The refresh failed and the cache is empty: the sanitised failure
    Item {
        width: parent.width
        height: emptyFailed.height
        visible: emptyFailed.visible
        EmptyState {
            id: emptyFailed
            visible: page.anilistConnected && page.loadFailed && page.entries.length === 0
            icon: "triangle-alert"
            title: "The watching list did not load"
            body: "AniList could not be reached. The list comes back on the next refresh."
        }
    }

    // Connected, empty
    Item {
        width: parent.width
        height: emptyNone.height
        visible: emptyNone.visible
        EmptyState {
            id: emptyNone
            visible: page.anilistConnected && !page.loadFailed && page.entries.length === 0
            icon: "eye"
            title: "Nothing on your watching list"
            body: "Mark a show as Watching on AniList and it lands here."
        }
    }

    Flow {
        width: parent.width
        spacing: theme.space(5)
        visible: page.entries.length > 0
        Repeater {
            model: page.entries
            Card {
                required property var modelData
                readonly property bool owned: modelData.owned !== null && modelData.owned !== undefined
                item: page.entryItem(modelData)
                ghost: !owned
                nowMs: page.nowMs
                metaOverride: page.updatedText(modelData)
                onOpened: {
                    if (owned) frame.go("series", { id: modelData.owned }, modelData.title)
                    else Qt.openUrlExternally("https://anilist.co/anime/" + modelData.anilist_id)
                }
            }
        }
    }
}
