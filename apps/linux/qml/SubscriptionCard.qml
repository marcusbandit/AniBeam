// One subscription as a pane of the glass mosaic: the poster of the library series its
// folder fills, full-bleed, with the title and one ledger line on the scrim at its foot.
// A complete subscription (the show has finished airing and every episode is on disk)
// wears a red Complete tag top left. Hovering a pane puts Remove RSS at its foot, across
// its width, in place of the episodes line. Remove RSS asks twice in place and takes the feed, the rule
// and the torrents out of the client; the files stay in the library. A click anywhere
// else on the pane opens the show inside the page. A rule with no library series behind
// it is a plain pane with the feed's glyph.
import QtQuick
import com.marcusrosado.AniBeam

Item {
    id: root
    property var rule: ({})
    property real nowMs: Date.now()
    property bool removing: false
    signal opened()
    signal removeRss()

    readonly property var r: rule || ({})
    readonly property var show: r.show || null
    readonly property bool complete: !!show && show.complete === true
    readonly property bool paused: r.rule_enabled === false
    readonly property bool hasPoster: !!show && !!show.poster
    readonly property real pad: theme.space(2.5)
    readonly property int total: show && show.total_episodes ? show.total_episodes : 0
    readonly property int have: show ? (show.episodes_on_disk || 0) : 0
    readonly property bool airing: !!show && !!show.next_airing && show.next_airing.at * 1000 > nowMs
    readonly property string tag: removing ? "Removing" : complete ? "Complete" : paused ? "Paused" : !show ? "Not in the library" : airing || show.status === "Releasing" ? "Airing" : ""
    readonly property string ledger: {
        if (!show) return Fmt.plural(r.torrent_count || 0, "torrent", "torrents")
        if (show.kind === "Movie") return complete ? "Film, on disk" : "Film"
        if (complete) return "All " + total + " here"
        if (airing) return "Ep " + show.next_airing.episode + " in " + Fmt.countdown(show.next_airing.at - nowMs / 1000)
        if (total > 0) return have + " of " + total + " here"
        return Fmt.plural(have, "episode", "episodes") + " here"
    }
    readonly property color inkMain: hasPoster ? theme.onPoster : theme.text
    readonly property color inkDim: hasPoster ? theme.onPosterDim : theme.textDim

    implicitHeight: width * 1.5
    opacity: removing ? 0.5 : 1

    // 0 rest, 1 hover
    property real lift: 0
    FrameAnimation {
        running: Math.abs(root.lift - (hov.hovered ? 1 : 0)) > 0.002
        onTriggered: root.lift = theme.follow(root.lift, hov.hovered ? 1 : 0, frameTime)
    }
    HoverHandler { id: hov; cursorShape: Qt.PointingHandCursor }
    scale: press.pressed ? 0.985 : 1

    Corner {
        anchors.fill: parent
        radius: theme.radiusTile
        color: theme.tile
        borderColor: root.complete ? Qt.rgba(theme.red.r, theme.red.g, theme.red.b, 0.55 + 0.3 * root.lift)
                   : Qt.rgba(theme.accent.r, theme.accent.g, theme.accent.b, 0.6 * root.lift)
        borderWidth: root.complete || root.lift > 0.02 ? 1 : 0
    }
    Icon {
        visible: !root.hasPoster
        anchors.horizontalCenter: parent.horizontalCenter
        y: parent.height * 0.3
        glyph: "rss"
        size: theme.space(10)
        color: theme.textFaint
        opacity: 0.6
    }
    Loader {
        anchors.fill: parent
        active: root.hasPoster
        opacity: root.paused ? 0.6 : 1
        sourceComponent: Corner {
            radius: theme.radiusTile
            fillItem: posterImage
            Image {
                id: posterImage
                visible: false
                anchors.fill: parent
                source: "file://" + root.show.poster
                sourceSize: theme.posterDecodeSize
                fillMode: Image.PreserveAspectCrop
                smooth: true
                asynchronous: true
                cache: true
            }
        }
    }
    Rectangle {
        visible: root.hasPoster
        anchors.bottom: parent.bottom
        width: parent.width
        height: parent.height * (theme.scrimReach + 0.08 * root.lift)
        radius: theme.radiusTile
        antialiasing: true
        gradient: Gradient {
            GradientStop { position: 0; color: "transparent" }
            GradientStop { position: 0.45; color: Qt.rgba(theme.posterScrim.r, theme.posterScrim.g, theme.posterScrim.b, theme.posterScrim.a * (0.7 + 0.12 * root.lift)) }
            GradientStop { position: 1; color: theme.posterScrim }
        }
    }

    // The click that opens the show; the button above it takes its own
    MouseArea { id: press; anchors.fill: parent; onClicked: root.opened() }

    // ---- The tag, top left: Complete in red, the rest quiet glass
    Rectangle {
        visible: root.tag !== ""
        x: root.pad * 0.8; y: root.pad * 0.8
        radius: theme.radiusSm
        antialiasing: true
        width: tagText.implicitWidth + theme.space(2) * 2
        height: tagText.implicitHeight + theme.space(0.75) * 2
        color: root.complete && !root.removing ? theme.red : Qt.rgba(8 / 255, 9 / 255, 11 / 255, 0.62)
        border.color: root.complete && !root.removing ? "transparent" : Qt.rgba(1, 1, 1, 0.14)
        border.width: 1
        Label {
            id: tagText
            anchors.centerIn: parent
            text: root.tag
            role: "ledger"
            font.pointSize: theme.typeSmall
            font.weight: root.complete ? 600 : theme.roleWeight("ledger")
            color: root.complete ? "#ffffff" : theme.onPoster
        }
    }

    // ---- The foot: the title, one ledger line, the episodes on disk as a line
    Column {
        id: foot
        x: root.pad
        width: root.width - root.pad * 2
        y: root.height - root.pad - height
        spacing: theme.space(1)
        Label {
            width: parent.width
            text: root.show ? root.show.title : (root.r.name || "")
            role: "title"
            font.pointSize: theme.typeNormal
            color: root.inkMain
            wrapMode: Text.Wrap
            maximumLineCount: 2
            elide: Text.ElideRight
            lineHeight: 0.95
        }
        Label {
            width: parent.width
            text: root.ledger
            role: "ledger"
            color: root.complete ? theme.onPoster : root.airing ? (root.hasPoster ? theme.accentOnPoster : theme.accent) : root.inkDim
            elide: Text.ElideRight
        }
        Item {
            visible: root.total > 0 && root.show.kind !== "Movie" && !removeButton.visible
            width: parent.width
            height: theme.space(1)
            Rectangle { anchors.fill: parent; radius: height / 2; color: Qt.rgba(1, 1, 1, root.hasPoster ? 0.16 : 0.08) }
            Rectangle {
                width: parent.width * Math.min(1, root.have / Math.max(1, root.total))
                height: parent.height
                radius: height / 2
                color: root.complete ? theme.caughtUp : root.hasPoster ? theme.accentOnPoster : theme.accent
            }
        }
        // Remove RSS: on hover, and held while armed
        ConfirmButton {
            id: removeButton
            objectName: "subscription-remove"
            visible: !root.removing && (hov.hovered || armed)
            width: parent.width
            label: "Remove RSS"
            icon: "rss"
            small: true
            onArt: root.hasPoster
            onConfirmed: root.removeRss()
            Tooltip { text: removeButton.armed ? "Click again: the feed, the rule and its torrents go. The files stay." : "Removes the feed, the rule and its torrents from the client. The files stay." }
        }
    }
}
