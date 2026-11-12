// One recommended show (#779) as a poster pane, written the way Card is: the poster
// full-bleed, and on the scrim at its foot the title, the episode count (a film's runtime)
// and two or three genres or tags. The forecast sits top left on a glass tag, always shown:
// "84%" is the share of the user's rated shows it is predicted to beat. "In library" sits top
// right when the library has it, and the list-status square bottom right when it is on a list.
//
// Holding the pointer on it, or keyboard focus on it, for the tooltip's delay asks the page to
// open the hover card (`rested`); the page's RecommendedHover owns the timing, the warm
// path and the closing. A click, Return or Space opens the show.
import QtQuick
import com.marcusrosado.AniBeam

Item {
    id: root
    property var rec: ({})
    property real posterWidth: theme.posterWidth
    signal opened()
    // Whether the pointer or focus is on the tile, for the page's hover card
    signal rested(bool on)
    readonly property bool gridStop: true
    activeFocusOnTab: true
    Keys.onReturnPressed: opened()
    Keys.onEnterPressed: opened()
    Keys.onSpacePressed: opened()
    Accessible.role: Accessible.Button
    Accessible.name: (rec.title || "") + ", " + rec.forecast + " percent for you"
    Accessible.onPressAction: opened()
    onActiveFocusChanged: if (!m.containsMouse) rested(activeFocus)

    // "12 episodes", a film's "1h 52m"; the hover card reads these too
    function lengthOf(r) {
        if (!r) return ""
        if (r.format === "MOVIE" && r.duration) return Fmt.runtime(r.duration)
        if (r.episodes) return Fmt.plural(r.episodes, "episode", "episodes")
        return ""
    }
    // The genres first, then the strongest tags, never more than `n`
    function traitsOf(r, n) {
        if (!r) return []
        return (r.genres || []).concat(r.tags || []).slice(0, n)
    }

    width: posterWidth
    height: Math.round(posterWidth * 1.5)
    implicitWidth: width
    implicitHeight: height
    // Hover lifts the tile a touch, the way Card does; a press pulls it back under
    property real lift: 0
    readonly property real liftTarget: m.containsMouse ? 1 : 0
    FrameAnimation {
        running: Math.abs(root.lift - root.liftTarget) > 0.002
        onTriggered: root.lift = theme.follow(root.lift, root.liftTarget, frameTime)
    }
    scale: theme.reducedMotion ? 1 : (m.pressed ? 0.98 : 1 + 0.015 * lift)
    z: lift > 0.01 || activeFocus ? 1 : 0

    Rectangle { anchors.fill: parent; anchors.margins: -2; visible: root.activeFocus; radius: theme.radiusTile + 2; antialiasing: true; color: "transparent"; border.width: 2; border.color: theme.focusRing; z: 10 }

    Corner {
        anchors.fill: parent
        radius: theme.radiusTile; smoothing: 0
        color: theme.tile
        borderColor: art.status === Image.Ready ? "transparent" : theme.line
        borderWidth: art.status === Image.Ready ? 0 : 1
        fillItem: art.status === Image.Ready ? art : null
        Image {
            id: art
            visible: false
            width: parent.width; height: parent.height
            source: root.rec.poster ? Fmt.fileUrl(root.rec.poster) : ""
            fillMode: Image.PreserveAspectCrop; asynchronous: true; smooth: true
            sourceSize: theme.posterDecodeSize
        }
    }
    // The scrim rises far enough to hold all three lines
    Rectangle {
        radius: theme.radiusTile; antialiasing: true
        anchors.bottom: parent.bottom
        width: parent.width
        height: Math.max(parent.height * theme.scrimReach, words.height + theme.space(10))
        gradient: Gradient {
            GradientStop { position: 0; color: "transparent" }
            GradientStop { position: 0.45; color: Qt.rgba(theme.posterScrim.r, theme.posterScrim.g, theme.posterScrim.b, theme.posterScrim.a * (m.containsMouse ? 0.8 : 0.66)) }
            GradientStop { position: 1; color: theme.posterScrim }
        }
    }

    // The forecast: the one number the row is sorted by
    Rectangle {
        objectName: "rec-forecast-" + root.rec.anilist_id
        x: theme.space(1.5); y: theme.space(1.5)
        width: pct.implicitWidth + theme.space(3); height: pct.implicitHeight + theme.space(1.5)
        radius: theme.radiusSm; antialiasing: true
        color: Qt.rgba(8 / 255, 9 / 255, 11 / 255, 0.7)
        border.color: Qt.rgba(1, 1, 1, 0.14); border.width: 1
        Label { id: pct; anchors.centerIn: parent; role: "ledger"; text: root.rec.forecast + "%"; color: theme.accentOnPoster }
    }
    Rectangle {
        visible: !!root.rec.owned
        anchors.right: parent.right; anchors.top: parent.top; anchors.margins: theme.space(1.5)
        width: have.implicitWidth + theme.space(3); height: have.implicitHeight + theme.space(1.5)
        radius: theme.radiusSm; antialiasing: true
        color: Qt.rgba(8 / 255, 9 / 255, 11 / 255, 0.62)
        border.color: Qt.rgba(1, 1, 1, 0.14); border.width: 1
        Label { id: have; anchors.centerIn: parent; role: "ledger"; text: "In library"; color: theme.onPosterDim }
    }

    Column {
        id: words
        x: theme.space(2.5)
        width: parent.width - theme.space(5) - (dot.visible ? dot.width + theme.space(2) : 0)
        anchors.bottom: parent.bottom; anchors.bottomMargin: theme.space(2.5)
        spacing: theme.space(0.5)
        Label {
            width: parent.width
            text: root.rec.title || ""
            role: "title"; font.pointSize: theme.typeNormal
            color: theme.onPoster
            wrapMode: Text.Wrap; maximumLineCount: 2; elide: Text.ElideRight
            lineHeight: 0.95
        }
        Label {
            width: parent.width
            visible: text !== ""
            text: root.lengthOf(root.rec)
            role: "ledger"; color: theme.onPosterDim
            elide: Text.ElideRight
        }
        Label {
            width: parent.width
            visible: text !== ""
            text: root.traitsOf(root.rec, 3).join(", ")
            role: "ui"; font.pointSize: theme.typeSmall; color: theme.onPosterDim
            elide: Text.ElideRight
        }
    }
    StatusDot { id: dot; anchors.right: parent.right; anchors.bottom: parent.bottom; anchors.margins: theme.space(2.5); status: root.rec.list_status || "" }

    Rectangle {
        anchors.fill: parent
        visible: m.containsMouse
        radius: theme.radiusTile; antialiasing: true
        color: "transparent"
        border.color: theme.accent; border.width: 1
    }
    MouseArea {
        id: m
        anchors.fill: parent
        hoverEnabled: true
        cursorShape: Qt.PointingHandCursor
        onPressed: root.forceActiveFocus()
        onClicked: root.opened()
        onContainsMouseChanged: root.rested(containsMouse || root.activeFocus)
    }
}
