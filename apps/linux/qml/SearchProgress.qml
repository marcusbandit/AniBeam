// What a search is doing while it runs, so a slow endpoint never looks like nothing: the
// phase in a line (Starting 1337x, Searching nyaa, Reading the results), the seconds since
// it began, a sentence on why it takes as long as it does, a bar, and the ways out: Cancel,
// or the same query on another endpoint. An on-demand endpoint's start has a known ceiling
// (anirss's timeout for it), so its bar fills toward that; everything else runs a sweep.
import QtQuick
import com.marcusrosado.AniBeam

Tile {
    id: root
    property string label: ""
    property real elapsed: 0
    // Seconds anirss gives an on-demand endpoint to come up; 0 for one that is always there
    property real total: 0
    property string endpoint: ""
    property var others: []
    signal cancel()
    signal switchTo(string name)

    readonly property bool starting: label.indexOf("Starting") === 0 && label !== "Starting anirss"
    readonly property bool ondemand: total > 0
    readonly property string why: {
        if (label === "Starting anirss") return "AniBeam is starting anirss; this takes a second or two."
        if (starting) return root.endpoint + " runs behind Jackett in a pod that anirss starts when a search needs it. The first search waits for it to come up, up to " + Math.round(root.total) + " seconds; the ones after skip this."
        if (label.indexOf("Reading") === 0) return "Splitting the releases into groups and checking them against the library."
        if (root.ondemand) return "Jackett scrapes " + root.endpoint + " for every new query, which usually takes ten to twenty seconds."
        return root.elapsed > 8 ? root.endpoint + " is slow to answer right now." : "This usually takes a second or two."
    }
    readonly property bool slow: elapsed >= (ondemand ? (starting ? total : 35) : 15)

    implicitHeight: body.implicitHeight + padding * 2
    lit: false

    Column {
        id: body
        width: parent.width
        spacing: theme.space(3)
        Item {
            width: parent.width
            height: Math.max(words.implicitHeight, buttons.height)
            Column {
                id: words
                width: parent.width - buttons.width - theme.space(6)
                spacing: theme.space(1.25)
                Row {
                    spacing: theme.space(3)
                    Label { id: phase; text: root.label === "" ? "Searching" : root.label; role: "title"; font.pointSize: theme.typeNormal * 1.15 }
                    Label { anchors.baseline: phase.baseline; text: Math.floor(root.elapsed) + " s"; role: "ledger"; color: root.slow ? theme.orange : theme.textDim }
                }
                Label {
                    width: parent.width
                    text: root.why + (root.slow ? " This is taking longer than usual; cancel to try another endpoint." : "")
                    role: "ui"
                    color: theme.textDim
                    wrapMode: Text.Wrap
                }
            }
            Row {
                id: buttons
                anchors.right: parent.right
                anchors.verticalCenter: parent.verticalCenter
                spacing: theme.space(2)
                Repeater {
                    model: root.others.slice(0, 2)
                    Button { required property string modelData; text: "Try " + modelData; icon: "search"; flat: true; small: true; onClicked: root.switchTo(modelData) }
                }
                Button { objectName: "search-cancel"; text: "Cancel"; icon: "x"; small: true; onClicked: root.cancel() }
            }
        }
        // The bar: toward the start's ceiling while an on-demand endpoint comes up, else a sweep
        Item {
            id: track
            width: parent.width
            height: theme.space(1)
            clip: true
            Rectangle { anchors.fill: parent; radius: height / 2; color: theme.line }
            Rectangle {
                visible: root.starting && root.ondemand
                width: parent.width * Math.min(1, root.elapsed / Math.max(1, root.total))
                height: parent.height
                radius: height / 2
                color: theme.orange
                Behavior on width { enabled: !theme.reducedMotion; NumberAnimation { duration: 900 } }
            }
            Rectangle {
                id: sweep
                visible: !(root.starting && root.ondemand)
                width: parent.width * 0.22
                height: parent.height
                radius: height / 2
                color: theme.accent
                opacity: theme.reducedMotion ? 0.6 : 1
                x: theme.reducedMotion ? (parent.width - width) / 2 : 0
                SequentialAnimation on x {
                    running: sweep.visible && !theme.reducedMotion && root.visible
                    loops: Animation.Infinite
                    NumberAnimation { from: -sweep.width; to: track.width; duration: 1400; easing.type: Easing.InOutQuad }
                }
            }
        }
    }
}
