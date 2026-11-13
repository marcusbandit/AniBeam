// What a search is doing while it runs, so a slow endpoint never looks like nothing: the
// phase in a line (Starting a source, Searching it, Reading the results), the seconds since
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

    // The phase label comes worded from the core's job progress, so these compare its words
    readonly property bool starting: label.indexOf("Starting") === 0 && label !== "Starting the search" // i18n: none
    readonly property bool ondemand: total > 0
    readonly property string why: {
        if (label === "Starting the search") return words.t("search-progress-why-helper") // i18n: none
        if (starting) return words.t("search-progress-why-starting", { endpoint: root.endpoint, seconds: Math.round(root.total) })
        if (label.indexOf("Reading") === 0) return words.t("search-progress-why-reading") // i18n: none
        if (root.ondemand) return words.t("search-progress-why-ondemand", { endpoint: root.endpoint })
        return root.elapsed > 8 ? words.t("search-progress-why-slow", { endpoint: root.endpoint }) : words.t("search-progress-why-usual")
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
            height: Math.max(wordsColumn.implicitHeight, buttons.height)
            Column {
                // Not `words`, which would hide the global words in this file
                id: wordsColumn
                width: parent.width - buttons.width - theme.space(6)
                spacing: theme.space(1.25)
                Row {
                    spacing: theme.space(3)
                    Label { id: phase; text: root.label === "" ? words.t("search-progress-searching") : root.label; role: "title"; font.pointSize: theme.typeNormal * 1.15 }
                    Label { anchors.baseline: phase.baseline; text: words.t("search-progress-seconds", { seconds: Math.floor(root.elapsed) }); role: "ledger"; color: root.slow ? theme.orange : theme.textDim }
                }
                Label {
                    width: parent.width
                    text: root.slow ? words.t("search-progress-why-and-slow", { why: root.why }) : root.why
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
                    Button { required property string modelData; text: words.t("search-progress-try", { endpoint: modelData }); icon: "search"; flat: true; small: true; onClicked: root.switchTo(modelData) }
                }
                Button { objectName: "search-cancel"; msg: "action-cancel"; icon: "x"; small: true; onClicked: root.cancel() }
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
