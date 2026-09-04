// One series in the grid: the poster in a G2 frame with four corner facts and a progress
// strip, the title and a meta line beneath. Hover lifts the poster with exponential smoothing.
import QtQuick

Item {
    id: root
    property var item: ({})
    property real posterWidth: 180
    property string titleLang: "jp"
    property real nowMs: Date.now()

    width: posterWidth
    implicitHeight: posterWidth * 1.5 + theme.space(2) + info.implicitHeight

    readonly property string displayTitle: {
        var i = item || {}
        if (titleLang === "en") return i.titleEnglish || i.titleRomaji || i.folderName || ""
        return i.titleRomaji || i.titleEnglish || i.folderName || ""
    }
    readonly property bool hasWatched: item.watched !== null && item.watched !== undefined
    readonly property int available: Math.max(item.latestAired || 0, item.latestFile || 0)
    readonly property bool totalKnown: item.total !== null && item.total !== undefined && item.total > 0
    readonly property string watchState: !hasWatched ? "" : (!totalKnown ? "unknown" : (item.watched < available && item.watched < item.total ? "behind" : "caught-up"))
    readonly property string watchedLabel: !hasWatched ? "" : item.watched + "/" + (totalKnown ? item.total : "?")
    readonly property real denom: totalKnown ? item.total : Math.max(available, item.watched || 0, 1)
    readonly property real watchedPct: hasWatched ? Math.min(1, item.watched / denom) : 0
    readonly property real availablePct: Math.min(1, available / denom)
    readonly property string epBadge: item.isMovie ? "Movie" : (item.latestFile ? "EP " + String(item.latestFile).padStart(2, "0") : "")
    readonly property string metaLeft: item.lastViewedAt ? ago(nowMs - item.lastViewedAt) : (item.fileCount === 1 ? "1 file" : item.fileCount + " files")
    readonly property string countdown: item.nextAirMs && item.nextAirMs > nowMs ? until(item.nextAirMs - nowMs) : ""

    // Lift: exponential smoothing toward the hover target
    property real lift: 0
    readonly property real liftTarget: hover.containsMouse ? -3 : 0
    FrameAnimation {
        running: Math.abs(root.lift - root.liftTarget) > 0.05
        onTriggered: root.lift += (root.liftTarget - root.lift) * (1 - Math.exp(-12 * frameTime))
    }

    Image {
        id: poster
        visible: false
        width: frame.width
        height: frame.height
        source: item.poster ? "file://" + item.poster : ""
        sourceSize.width: 480
        fillMode: Image.PreserveAspectCrop
        asynchronous: true
        cache: true
    }

    Corner {
        id: frame
        x: 0
        y: root.lift
        width: root.posterWidth
        height: width * 1.5
        radius: theme.radiusLg
        smoothing: theme.cornerSmoothing
        color: theme.surface
        fillItem: poster.status === Image.Ready ? poster : null
        borderColor: hover.containsMouse ? theme.lineStrong : theme.line
        borderWidth: 1

        Text {
            visible: !item.poster
            anchors.centerIn: parent
            text: "No poster"
            color: theme.textFaint
            font.family: theme.fontSans
            font.pointSize: theme.typeSmall
        }

        Chip {
            visible: root.epBadge !== ""
            x: theme.space(2); y: theme.space(2)
            text: root.epBadge
        }
        Chip {
            visible: root.watchedLabel !== ""
            anchors.right: parent.right; anchors.rightMargin: theme.space(2)
            y: theme.space(2)
            text: root.watchedLabel
            textColor: root.watchState === "behind" ? theme.behind : root.watchState === "unknown" ? theme.textDim : theme.caughtUp
        }
        Row {
            x: theme.space(2)
            anchors.bottom: parent.bottom
            anchors.bottomMargin: theme.space(2) + (strip.visible ? strip.height + theme.space(1.5) : 0)
            spacing: theme.space(1)
            Chip { visible: item.score !== null && item.score !== undefined; small: true; text: item.score !== null && item.score !== undefined ? Number(item.score).toFixed(1) : ""; textColor: theme.textDim }
            Chip { visible: item.myScore !== null && item.myScore !== undefined; small: true; text: item.myScore !== null && item.myScore !== undefined ? Number(item.myScore).toFixed(1) : ""; textColor: theme.accent }
        }

        // Progress strip: watched in accent over an aired-or-downloaded underlay, unknown in line
        Item {
            id: strip
            visible: root.hasWatched
            x: theme.space(2)
            width: parent.width - theme.space(2) * 2
            height: Math.max(2, theme.space(0.75))
            anchors.bottom: parent.bottom
            anchors.bottomMargin: theme.space(2)
            Corner { width: parent.width; height: parent.height; radius: height / 2; smoothing: theme.cornerSmoothing; color: root.totalKnown ? theme.scrim : theme.line }
            Corner { width: parent.width * root.availablePct; height: parent.height; radius: height / 2; smoothing: theme.cornerSmoothing; color: theme.behind; visible: root.availablePct > root.watchedPct }
            Corner { width: parent.width * root.watchedPct; height: parent.height; radius: height / 2; smoothing: theme.cornerSmoothing; color: theme.caughtUp }
        }
    }

    Column {
        id: info
        anchors.top: frame.bottom
        anchors.topMargin: theme.space(2) - root.lift
        width: root.posterWidth
        spacing: theme.space(0.5)
        Text {
            width: parent.width
            text: root.displayTitle
            color: hover.containsMouse ? theme.accent : theme.text
            font.family: theme.fontSans
            font.pointSize: theme.typeNormal
            font.weight: Font.DemiBold
            elide: Text.ElideRight
            maximumLineCount: 2
            wrapMode: Text.Wrap
        }
        Item {
            width: parent.width
            height: metaLeftText.implicitHeight
            Text {
                id: metaLeftText
                anchors.left: parent.left
                text: root.metaLeft
                color: theme.textFaint
                font.family: theme.fontMono
                font.pointSize: theme.typeSmall
            }
            Text {
                anchors.right: parent.right
                text: root.countdown
                color: theme.accent
                font.family: theme.fontMono
                font.pointSize: theme.typeSmall
            }
        }
    }

    MouseArea {
        id: hover
        anchors.fill: parent
        hoverEnabled: true
        cursorShape: Qt.PointingHandCursor
    }

    function ago(ms) {
        var s = ms / 1000
        if (s < 60) return "just now"
        var m = s / 60; if (m < 60) return Math.floor(m) + "m ago"
        var h = m / 60; if (h < 24) return Math.floor(h) + "h ago"
        var d = h / 24; if (d < 14) return Math.floor(d) + "d ago"
        var w = d / 7; if (w < 9) return Math.floor(w) + "w ago"
        return Math.floor(d / 30) + "mo ago"
    }
    function until(ms) {
        var m = Math.floor(ms / 60000)
        if (m < 60) return m + "m"
        var h = Math.floor(m / 60)
        if (h < 24) return h + "h " + (m % 60) + "m"
        var d = Math.floor(h / 24)
        return d + "d " + (h % 24) + "h"
    }
}
