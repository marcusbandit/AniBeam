// The Developer tab (#287): why a page is slow, measured. The switches change what the app
// does for this session's next page (images, the read cache, the poster pool); the page
// loads list is every navigation timed by the frame's recorder, newest first, and picking
// one opens its calls, decodes and scene. The tour times every page with images on and
// again with them off. Memory is the kernel's count for this process; Events is the whole
// technical stream, every level, with the size each event crossed to QML at.
import QtQuick
import QtQuick.Layouts
import QtQuick.Controls.Basic as QC
import com.marcusrosado.AniBeam

SettingsTab {
    id: tab
    readonly property var rec: frame.devRecorder
    readonly property var kept: frame.devKept
    property var picked: rec.last
    property var mem: ({})
    property var events: []
    property string eventFilter: ""
    property var released: null
    function memRead() { mem = JSON.parse(Dev.memory()) }
    function eventsRead() { events = JSON.parse(Dev.events(3000)).reverse() }
    Timer { interval: 2000; repeat: true; running: true; triggeredOnStart: true; onTriggered: { tab.memRead(); tab.eventsRead() } }
    Connections { target: tab.rec; function onFinished(l) { tab.picked = l } }

    function mb(kbOrBytes, isKb) { return tab.rec.mb(isKb ? kbOrBytes * 1024 : kbOrBytes) }
    function ms(v) { return v < 0 ? "-" : Math.round(v) + " ms" }
    // Whole numbers grouped by a thin space: a comma or a dot reads as a decimal somewhere
    function n(v) { return String(Math.round(v)).replace(/\B(?=(\d{3})+(?!\d))/g, "\u2009") }
    function mbWhole(kb) { return n(kb / 1024) + " MB" }
    function tone(v) { return v < 100 ? theme.green : v < 300 ? theme.yellow : theme.red }
    // A table column: `parts` of the 100 the table's width is cut into
    function col(parts) { return Math.floor((tab.blockWidth - theme.space(2)) * parts / 100) }
    function copy(text) { clip.text = text; clip.selectAll(); clip.copy(); frame.toast("Copied") }
    TextEdit { id: clip; visible: false }

    // Events per kind over the last minute: where the traffic to QML comes from
    readonly property var eventRates: {
        var cut = Dev.now() - 60000, by = {}
        events.forEach(function(e) {
            if (e.at < cut) return
            var k = String(e.kind), r = by[k] || (by[k] = { kind: k, n: 0, bytes: 0 })
            r.n += 1; r.bytes += e.bytes
        })
        return Object.keys(by).map(function(k) { return by[k] }).sort(function(a, b) { return b.bytes - a.bytes }).slice(0, 10)
    }
    readonly property var shownEvents: {
        var f = eventFilter.toLowerCase()
        return f === "" ? events.slice(0, 400) : events.filter(function(e) {
            return (e.message + " " + e.kind + " " + e.stage + " " + (e.plain || "")).toLowerCase().indexOf(f) >= 0
        }).slice(0, 400)
    }

    SettingsPair {
        split: 1 / 2
        evenColumns: true
        leftPanels: [ switchesPanel ]
        rightPanels: [ memoryPanel ]
    }
    Loader { sourceComponent: loadsPanel; Layout.fillWidth: true; Layout.minimumHeight: item ? item.implicitHeight : 0 }
    Loader { active: tab.picked !== null; sourceComponent: detailPanel; Layout.fillWidth: true; Layout.minimumHeight: item ? item.implicitHeight : 0 }
    Loader { active: tab.rec.tourResult !== null; sourceComponent: tourPanel; Layout.fillWidth: true; Layout.minimumHeight: item ? item.implicitHeight : 0 }
    Loader { sourceComponent: eventsPanel; Layout.fillWidth: true; Layout.minimumHeight: item ? item.implicitHeight : 0 }

    Component {
        id: switchesPanel
        Panel {
            title: "Switches"; icon: "toggle-right"
            helper: "Each one applies to the next page you open, so an A and B test is one switch and one click."
            SettingRow { label: "Load images"; helper: "Off, no poster, banner or portrait is read or decoded."
                Switch { objectName: "dev-images"; checked: tab.kept.images; onToggled: function(on) { frame.devSetImages(on); frame.loadPage() } } }
            SettingRow { label: "Page load overlay"; helper: "The last page load and the memory, in the window's bottom right corner."
                Switch { objectName: "dev-hud"; checked: tab.kept.hud; onToggled: function(on) { tab.kept.hud = on } } }
            SettingRow { label: "Keep library reads"; helper: "The answer to a whole library read is kept until the library changes. Off, every page asks the core again."
                Switch { checked: tab.kept.readCache; onToggled: function(on) { tab.kept.readCache = on; frame.devApply() } } }
            SettingRow { label: "Warm posters in the background"; helper: "Decodes the 60 posters you are most likely to see and keeps them in memory."
                Switch { checked: tab.kept.posterPool; onToggled: function(on) { tab.kept.posterPool = on } } }
            SettingRow { visible: !Dev.forced; label: "Developer options"; helper: "Turns this tab, the overlay and the measuring off."
                Switch { checked: true; onToggled: function(on) { if (!on) { tab.kept.enabled = false; frame.devApply(); frame.go("settings", { tab: "data" }) } } } }
        }
    }
    Component {
        id: memoryPanel
        Panel {
            title: "Memory"; icon: "memory-stick"
            helper: tab.mem.VmRSS ? "What the kernel counts for this process. Images hold decoded pixels in memory and on the graphics card." : "This system does not say how much memory the app uses."
            Tiles { tiles: tab.mem.VmRSS ? [
                { value: tab.mbWhole(tab.mem.VmRSS), caption: "In use" },
                { value: tab.mbWhole(tab.mem.VmHWM), caption: "Peak" },
                { value: tab.mbWhole(tab.mem.RssAnon), caption: "Heap" },
                { value: tab.mbWhole(tab.mem.RssFile), caption: "Files mapped" },
                { value: String(tab.mem.Threads || 0), caption: "Threads" }] : [] }
            SettingRow { label: "Give memory back"; helper: "Drops what the scene graph can rebuild, collects JavaScript garbage and hands free heap to the system. What it frees is what the app was holding without needing it."
                line: tab.released ? tab.mb(tab.released.before, true) + " to " + tab.mb(tab.released.after, true) : ""
                Button { text: "Free now"; icon: "recycle"; small: true
                    onClicked: { var b = JSON.parse(Dev.memory()).VmRSS || 0; Dev.release(frame.hostWindow); var a = JSON.parse(Dev.memory()).VmRSS || 0; tab.released = { before: b, after: a }; tab.memRead() } } }
            SettingRow { label: "Images since the start"
                line: { var t = JSON.parse(Dev.images(Number.MAX_VALUE)); tab.mem; return t.decodes + " decoded, " + tab.rec.mb(t.decodedBytes) + " of pixels from " + tab.rec.mb(t.fileBytes) + " on disk, " + Math.round(t.decodeMs) + " ms" } }
        }
    }
    Component {
        id: loadsPanel
        Panel {
            title: "Page loads"; icon: "timer"
            helper: "Built is the wait after a click: the Qt thread does nothing else until the page exists. Settled is when its last image arrived."
            Row {
                spacing: theme.space(2)
                Button { objectName: "dev-tour"; text: tab.rec.tourState ? "Timing, " + tab.rec.tourState.at + " of " + tab.rec.tourState.steps.length : "Time every page"; icon: "gauge"; primary: true; enabled: !tab.rec.tourState; onClicked: tab.rec.tour(3) }
                Button { text: "Copy report"; icon: "copy"; enabled: tab.rec.loads.length > 0; onClicked: tab.copy(tab.rec.report()) }
                Button { text: "Clear"; icon: "trash-2"; flat: true; enabled: tab.rec.loads.length > 0; onClicked: { tab.rec.loads = []; tab.picked = null } }
            }
            Column {
                width: parent.width
                Item {
                    width: parent.width; height: theme.space(8)
                    Row {
                        anchors.verticalCenter: parent.verticalCenter
                        Repeater {
                            model: [["Page", 16], ["Built", 8], ["Painted", 8], ["Settled", 8], ["Calls, data", 12], ["Decodes, time, pixels", 18], ["Items", 8], ["Images out of view", 12], ["Memory", 10]]
                            Label { required property var modelData; width: tab.col(modelData[1]); text: modelData[0]; role: "ledger"; color: theme.textFaint }
                        }
                    }
                }
                Repeater {
                    model: tab.rec.loads.slice(0, 20)
                    Rectangle {
                        required property var modelData
                        width: parent.width; height: theme.space(7)
                        color: tab.picked === modelData ? theme.accentSoft : rowMouse.containsMouse ? theme.tile : "transparent"
                        radius: theme.space(1)
                        Row {
                            anchors.verticalCenter: parent.verticalCenter
                            Label { width: tab.col(16); role: "ledger"; text: modelData.label + (modelData.images ? "" : ", no images"); elide: Text.ElideRight }
                            Label { width: tab.col(8); role: "ledger"; text: tab.ms(modelData.built); color: tab.tone(modelData.built) }
                            Label { width: tab.col(8); role: "ledger"; text: tab.ms(modelData.firstFrame); color: theme.textDim }
                            Label { width: tab.col(8); role: "ledger"; text: tab.ms(modelData.settled); color: tab.tone(modelData.settled) }
                            Label { width: tab.col(12); role: "ledger"; text: modelData.calls.n + ", " + tab.rec.kb(modelData.calls.bytes); color: theme.textDim }
                            Label { width: tab.col(18); role: "ledger"; text: modelData.decodes.n + ", " + tab.ms(modelData.decodes.ms) + ", " + tab.rec.mb(modelData.decodes.bytes); color: theme.textDim }
                            Label { width: tab.col(8); role: "ledger"; text: tab.n(modelData.scene.items); color: modelData.scene.items > 3000 ? theme.orange : theme.textDim }
                            Label { width: tab.col(12); role: "ledger"; text: modelData.scene.images.offscreen + " of " + modelData.scene.images.total; color: theme.textDim }
                            Label { width: tab.col(10); role: "ledger"; text: tab.mb(modelData.memAfter.VmRSS || 0, true); color: theme.textDim }
                        }
                        MouseArea { id: rowMouse; anchors.fill: parent; hoverEnabled: true; cursorShape: Qt.PointingHandCursor; onClicked: tab.picked = modelData }
                    }
                }
                Label { visible: tab.rec.loads.length === 0; topPadding: theme.space(2); text: "Open any page and it shows up here."; color: theme.textDim }
            }
        }
    }
    Component {
        id: detailPanel
        Panel {
            readonly property var l: tab.picked
            title: l.label + (l.images ? "" : ", images off"); icon: "scan-search"
            helper: l.frames + " frames painted while it loaded, the longest gap between two " + tab.ms(l.maxGap) + (l.timedOut ? ". Gave up waiting for its images after 15 s." : ".")
            Tiles { tiles: [
                { value: tab.ms(l.built), caption: "Built" }, { value: tab.ms(l.settled), caption: "Settled" },
                { value: tab.rec.kb(l.calls.bytes), caption: "Data from the core" }, { value: tab.rec.mb(l.decodes.bytes), caption: "Pixels decoded" },
                { value: tab.n(l.scene.items), caption: "Items on the page" }, { value: String(l.decodes.cached), caption: "Images already in memory" }] }
            Row {
                id: detailCols
                width: parent.width
                topPadding: theme.space(4)
                spacing: theme.space(10)
                readonly property real third: (width - spacing * 2) / 3
                Column {
                    width: detailCols.third
                    spacing: theme.space(1)
                    Label { text: "Core calls, slowest first"; role: "title"; font.pointSize: theme.typeNormal; bottomPadding: theme.space(1) }
                    Repeater {
                        model: l.calls.list.slice(0, 14)
                        Label { required property var modelData; width: parent.width; role: "ledger"; elide: Text.ElideRight
                            text: modelData.name + "  " + modelData.total.toFixed(1) + " ms  " + tab.rec.kb(modelData.bytes) + (modelData.hit ? "  cached" : "  core " + modelData.core.toFixed(1) + ", to QML " + (modelData.json + modelData.qjson).toFixed(1))
                            color: modelData.hit ? theme.textFaint : theme.textDim }
                    }
                }
                Column {
                    width: detailCols.third
                    spacing: theme.space(1)
                    Label { text: "Slowest decodes"; role: "title"; font.pointSize: theme.typeNormal; bottomPadding: theme.space(1) }
                    Label { visible: l.decodes.n > 0; width: parent.width; wrapMode: Text.Wrap; role: "ledger"; color: theme.textDim
                        text: l.decodes.n + " decodes one after another on the image thread, " + tab.ms(l.decodes.span) + " from the first to the last; " + l.decodes.requests + " asked for, " + l.decodes.cached + " already decoded, " + l.decodes.blocked + " blocked" }
                    Repeater {
                        model: l.decodes.slowest
                        Label { required property var modelData; width: parent.width; role: "ledger"; elide: Text.ElideMiddle; color: theme.textDim
                            text: modelData.ms.toFixed(1) + " ms  " + modelData.srcW + "x" + modelData.srcH + " to " + modelData.w + "x" + modelData.h + "  " + tab.rec.kb(modelData.bytes) }
                    }
                    Label { visible: l.decodes.n === 0; text: l.images ? "Nothing decoded: every image was in memory already." : "Images were off."; role: "ledger"; color: theme.textFaint }
                }
                Column {
                    width: detailCols.third
                    spacing: theme.space(1)
                    Label { text: "What the page is made of"; role: "title"; font.pointSize: theme.typeNormal; bottomPadding: theme.space(1) }
                    Label { width: parent.width; wrapMode: Text.Wrap; role: "ledger"; color: theme.textDim
                        text: l.scene.items + " items, " + l.scene.offscreen + " outside the window; " + l.scene.texts + " texts, " + l.scene.shapes + " shapes, " + l.scene.icons + " icons; "
                              + l.scene.images.total + " images, " + l.scene.images.offscreen + " of them outside the window holding " + tab.rec.mb(l.scene.images.offscreenPixels * 4) }
                    Repeater {
                        model: l.scene.types.slice(0, 12)
                        Label { required property var modelData; role: "ledger"; color: theme.textDim; text: modelData[1] + "  " + modelData[0] }
                    }
                }
            }
            Button { text: "Copy this load"; icon: "copy"; small: true; onClicked: tab.copy(tab.rec.describe(l)) }
        }
    }
    Component {
        id: tourPanel
        Panel {
            title: "Every page, images on and off"; icon: "gauge"
            helper: "The middle of three visits to each page. Where the two columns match, images are not what makes the page slow."
            Column {
                width: parent.width
                Row {
                    height: theme.space(8)
                    Repeater {
                        model: [["Page", 14], ["Built, images on", 11], ["Built, images off", 11], ["Settled, on", 11], ["Settled, off", 11], ["Items", 9], ["Decodes, time", 13], ["Data", 10], ["Memory", 10]]
                        Label { required property var modelData; anchors.verticalCenter: parent.verticalCenter; width: tab.col(modelData[1]); text: modelData[0]; role: "ledger"; color: theme.textFaint }
                    }
                }
                Repeater {
                    model: tab.rec.tourResult.rows
                    Row {
                        required property var modelData
                        height: theme.space(7)
                        Label { anchors.verticalCenter: parent.verticalCenter; width: tab.col(14); role: "ledger"; text: modelData.page }
                        Label { anchors.verticalCenter: parent.verticalCenter; width: tab.col(11); role: "ledger"; text: tab.ms(modelData.on.built); color: tab.tone(modelData.on.built) }
                        Label { anchors.verticalCenter: parent.verticalCenter; width: tab.col(11); role: "ledger"; text: tab.ms(modelData.off.built); color: tab.tone(modelData.off.built) }
                        Label { anchors.verticalCenter: parent.verticalCenter; width: tab.col(11); role: "ledger"; text: tab.ms(modelData.on.settled); color: tab.tone(modelData.on.settled) }
                        Label { anchors.verticalCenter: parent.verticalCenter; width: tab.col(11); role: "ledger"; text: tab.ms(modelData.off.settled); color: tab.tone(modelData.off.settled) }
                        Label { anchors.verticalCenter: parent.verticalCenter; width: tab.col(9); role: "ledger"; text: tab.n(modelData.on.items); color: theme.textDim }
                        Label { anchors.verticalCenter: parent.verticalCenter; width: tab.col(13); role: "ledger"; text: modelData.on.decodes + ", " + tab.ms(modelData.on.decodeMs); color: theme.textDim }
                        Label { anchors.verticalCenter: parent.verticalCenter; width: tab.col(10); role: "ledger"; text: tab.rec.kb(modelData.on.bytes); color: theme.textDim }
                        Label { anchors.verticalCenter: parent.verticalCenter; width: tab.col(10); role: "ledger"; text: tab.mb(modelData.on.rss, true); color: theme.textDim }
                    }
                }
                Label { topPadding: theme.space(2); role: "ledger"; color: theme.textDim; text: "Peak memory during the tour " + tab.mb(tab.rec.tourResult.peak, true) }
            }
        }
    }
    Component {
        id: eventsPanel
        Panel {
            title: "Events"; icon: "activity"
            helper: "Everything the core says, every level, newest first. Each event is copied to the Qt thread as JSON; the size is what it cost to send."
            Flow {
                width: parent.width
                spacing: theme.space(2)
                Repeater {
                    model: tab.eventRates
                    Chip { required property var modelData; small: true; color: theme.tileSunken; textColor: theme.textDim
                        text: modelData.kind + " " + modelData.n + "/min, " + tab.rec.kb(modelData.bytes) }
                }
            }
            Field { width: Math.min(parent.width, theme.space(120)); placeholder: "Filter by words, kind or stage"; onEdited: function(t) { tab.eventFilter = t } }
            ListView {
                id: list
                width: parent.width
                height: theme.space(120)
                clip: true
                boundsBehavior: Flickable.StopAtBounds
                model: tab.shownEvents
                WheelStep { flick: list }
                delegate: Item {
                    required property var modelData
                    width: list.width
                    height: theme.space(6)
                    Row {
                        anchors.verticalCenter: parent.verticalCenter
                        spacing: theme.space(2)
                        Label { width: theme.space(14); role: "ledger"; color: theme.textFaint; text: (modelData.at / 1000).toFixed(1) + " s" }
                        Label { width: theme.space(10); role: "ledger"; text: String(modelData.level); color: modelData.level === "Error" ? theme.red : modelData.level === "Warn" ? theme.orange : modelData.level === "Debug" ? theme.textFaint : theme.textDim }
                        Label { width: theme.space(22); role: "ledger"; color: theme.textDim; text: String(modelData.kind); elide: Text.ElideRight }
                        Label { width: theme.space(12); role: "ledger"; color: theme.textFaint; text: tab.rec.kb(modelData.bytes) }
                        Label { width: list.width - theme.space(66); role: "ledger"; elide: Text.ElideRight; color: theme.text
                            text: modelData.message + (modelData.plain ? "   (plain: " + modelData.plain + ")" : "") }
                    }
                }
                QC.ScrollBar.vertical: ThinScrollBar {}
            }
            Button { text: "Copy events"; icon: "copy"; small: true
                onClicked: tab.copy(tab.shownEvents.map(function(e) { return (e.at / 1000).toFixed(3) + " " + e.level + " " + e.stage + " " + e.kind + " " + e.bytes + "B " + e.message }).join("\n")) }
        }
    }
}
