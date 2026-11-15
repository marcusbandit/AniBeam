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
    title: words.t("settings-developer-title")
    lead: words.t("settings-developer-lead")
    wide: true
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
    function ms(v) { return v < 0 ? "-" : words.t("settings-developer-ms", { ms: Math.round(v) }) }
    // Whole numbers grouped by a thin space: a comma or a dot reads as a decimal somewhere
    function n(v) { return String(Math.round(v)).replace(/\B(?=(\d{3})+(?!\d))/g, "\u2009") }
    function mbWhole(kb) { return words.t("settings-developer-mb", { mb: n(kb / 1024) }) }
    function tone(v) { return v < 100 ? theme.green : v < 300 ? theme.yellow : theme.red }
    // A table column: `parts` of the 100 the table's width is cut into
    function col(parts) { return Math.floor((tab.blockWidth - theme.space(2)) * parts / 100) }

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
            title: words.t("settings-developer-switches"); icon: "toggle-right"
            SettingRow { label: words.t("settings-developer-load-images"); helper: words.t("settings-developer-load-images-helper")
                Switch { objectName: "dev-images"; checked: tab.kept.images; onToggled: function(on) { frame.devSetImages(on); frame.loadPage() } } }
            SettingRow { label: words.t("settings-developer-overlay"); helper: words.t("settings-developer-overlay-helper")
                Switch { objectName: "dev-hud"; checked: tab.kept.hud; onToggled: function(on) { tab.kept.hud = on } } }
            SettingRow { label: words.t("settings-developer-read-cache"); helper: words.t("settings-developer-read-cache-helper")
                Switch { checked: tab.kept.readCache; onToggled: function(on) { tab.kept.readCache = on; frame.devApply() } } }
            SettingRow { label: words.t("settings-developer-poster-pool"); helper: words.t("settings-developer-poster-pool-helper")
                Switch { checked: tab.kept.posterPool; onToggled: function(on) { tab.kept.posterPool = on } } }
            SettingRow { visible: !Dev.forced; label: words.t("settings-developer-options"); helper: words.t("settings-developer-options-helper")
                Switch { checked: true; onToggled: function(on) { if (!on) { tab.kept.enabled = false; frame.devApply(); frame.go("settings", { tab: "about" }) } } } }
        }
    }
    Component {
        id: memoryPanel
        Panel {
            title: words.t("settings-developer-memory"); icon: "memory-stick"
            helper: tab.mem.VmRSS ? words.t("settings-developer-memory-helper") : words.t("settings-developer-memory-unknown")
            Tiles { tiles: tab.mem.VmRSS ? [
                { value: tab.mbWhole(tab.mem.VmRSS), caption: words.t("settings-developer-in-use") },
                { value: tab.mbWhole(tab.mem.VmHWM), caption: words.t("settings-developer-peak") },
                { value: tab.mbWhole(tab.mem.RssAnon), caption: words.t("settings-developer-heap") },
                { value: tab.mbWhole(tab.mem.RssFile), caption: words.t("settings-developer-files-mapped") },
                { value: String(tab.mem.Threads || 0), caption: words.t("settings-developer-threads") }] : [] }
            SettingRow { label: words.t("settings-developer-give-back"); helper: words.t("settings-developer-give-back-helper")
                line: tab.released ? words.t("settings-developer-released", { before: tab.mb(tab.released.before, true), after: tab.mb(tab.released.after, true) }) : ""
                Button { msg: "settings-developer-free-now"; icon: "recycle"; small: true
                    onClicked: { var b = JSON.parse(Dev.memory()).VmRSS || 0; Dev.release(frame.hostWindow); var a = JSON.parse(Dev.memory()).VmRSS || 0; tab.released = { before: b, after: a }; tab.memRead() } } }
            SettingRow { label: words.t("settings-developer-images-total")
                line: { var t = JSON.parse(Dev.images(Number.MAX_VALUE)); tab.mem; return words.t("settings-developer-images-total-line", { decodes: t.decodes, pixels: tab.rec.mb(t.decodedBytes), disk: tab.rec.mb(t.fileBytes), ms: Math.round(t.decodeMs) }) } }
        }
    }
    Component {
        id: loadsPanel
        Panel {
            title: words.t("settings-developer-loads"); icon: "timer"
            helper: words.t("settings-developer-loads-helper")
            Row {
                spacing: theme.space(2)
                Button { objectName: "dev-tour"; text: tab.rec.tourState ? words.t("settings-developer-touring", { at: tab.rec.tourState.at, steps: tab.rec.tourState.steps.length }) : words.t("settings-developer-tour"); icon: "gauge"; primary: true; enabled: !tab.rec.tourState; onClicked: tab.rec.tour(3) }
                Button { msg: "settings-developer-copy-report"; icon: "copy"; enabled: tab.rec.loads.length > 0; onClicked: frame.copyText(tab.rec.report()) }
                Button { msg: "action-clear"; icon: "trash-2"; flat: true; enabled: tab.rec.loads.length > 0; onClicked: { tab.rec.loads = []; tab.picked = null } }
            }
            Column {
                width: parent.width
                Item {
                    width: parent.width; height: theme.space(8)
                    Row {
                        anchors.verticalCenter: parent.verticalCenter
                        Repeater {
                            model: [[words.t("settings-developer-col-page"), 16], [words.t("settings-developer-col-built"), 8], [words.t("settings-developer-col-painted"), 8], [words.t("settings-developer-col-settled"), 8],
                                    [words.t("settings-developer-col-calls"), 12], [words.t("settings-developer-col-decodes"), 18], [words.t("settings-developer-col-items"), 8],
                                    [words.t("settings-developer-col-offscreen"), 12], [words.t("settings-developer-col-memory"), 10]]
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
                            Label { width: tab.col(16); role: "ledger"; text: modelData.images ? modelData.label : words.t("settings-developer-load-no-images", { page: modelData.label }); elide: Text.ElideRight }
                            Label { width: tab.col(8); role: "ledger"; text: tab.ms(modelData.built); color: tab.tone(modelData.built) }
                            Label { width: tab.col(8); role: "ledger"; text: tab.ms(modelData.firstFrame); color: theme.textDim }
                            Label { width: tab.col(8); role: "ledger"; text: tab.ms(modelData.settled); color: tab.tone(modelData.settled) }
                            Label { width: tab.col(12); role: "ledger"; text: modelData.calls.n + ", " + tab.rec.kb(modelData.calls.bytes); color: theme.textDim }
                            Label { width: tab.col(18); role: "ledger"; text: modelData.decodes.n + ", " + tab.ms(modelData.decodes.ms) + ", " + tab.rec.mb(modelData.decodes.bytes); color: theme.textDim }
                            Label { width: tab.col(8); role: "ledger"; text: tab.n(modelData.scene.items); color: modelData.scene.items > 3000 ? theme.orange : theme.textDim }
                            Label { width: tab.col(12); role: "ledger"; text: words.t("settings-developer-offscreen-of", { offscreen: modelData.scene.images.offscreen, total: modelData.scene.images.total }); color: theme.textDim }
                            Label { width: tab.col(10); role: "ledger"; text: tab.mb(modelData.memAfter.VmRSS || 0, true); color: theme.textDim }
                        }
                        MouseArea { id: rowMouse; anchors.fill: parent; hoverEnabled: true; cursorShape: Qt.PointingHandCursor; onClicked: tab.picked = modelData; Accessible.role: Accessible.Button; Accessible.name: theme.nameNear(parent)
                            Accessible.onPressAction: tab.picked = modelData }
                    }
                }
                Label { visible: tab.rec.loads.length === 0; topPadding: theme.space(2); msg: "settings-developer-loads-empty"; color: theme.textDim }
            }
        }
    }
    Component {
        id: detailPanel
        Panel {
            readonly property var l: tab.picked
            title: l.images ? l.label : words.t("settings-developer-load-images-off", { page: l.label }); icon: "scan-search"
            helper: words.t(l.timedOut ? "settings-developer-load-frames-timed-out" : "settings-developer-load-frames", { frames: l.frames, gap: tab.ms(l.maxGap) })
            Tiles { tiles: [
                { value: tab.ms(l.built), caption: words.t("settings-developer-built") }, { value: tab.ms(l.settled), caption: words.t("settings-developer-settled") },
                { value: tab.rec.kb(l.calls.bytes), caption: words.t("settings-developer-core-data") }, { value: tab.rec.mb(l.decodes.bytes), caption: words.t("settings-developer-pixels") },
                { value: tab.n(l.scene.items), caption: words.t("settings-developer-page-items") }, { value: String(l.decodes.cached), caption: words.t("settings-developer-cached-images") }] }
            Row {
                id: detailCols
                width: parent.width
                topPadding: theme.space(4)
                spacing: theme.space(10)
                readonly property real third: (width - spacing * 2) / 3
                Column {
                    width: detailCols.third
                    spacing: theme.space(1)
                    Label { msg: "settings-developer-calls"; role: "title"; font.pointSize: theme.typeNormal; bottomPadding: theme.space(1) }
                    Repeater {
                        model: l.calls.list.slice(0, 14)
                        Label { required property var modelData; width: parent.width; role: "ledger"; elide: Text.ElideRight
                            text: modelData.hit ? words.t("settings-developer-call-cached", { name: modelData.name, ms: modelData.total.toFixed(1), size: tab.rec.kb(modelData.bytes) })
                                  : words.t("settings-developer-call", { name: modelData.name, ms: modelData.total.toFixed(1), size: tab.rec.kb(modelData.bytes), core: modelData.core.toFixed(1), qml: (modelData.json + modelData.qjson).toFixed(1) })
                            color: modelData.hit ? theme.textFaint : theme.textDim }
                    }
                }
                Column {
                    width: detailCols.third
                    spacing: theme.space(1)
                    Label { msg: "settings-developer-decodes"; role: "title"; font.pointSize: theme.typeNormal; bottomPadding: theme.space(1) }
                    Label { visible: l.decodes.n > 0; width: parent.width; wrapMode: Text.Wrap; role: "ledger"; color: theme.textDim
                        text: words.t("settings-developer-decodes-line", { count: l.decodes.n, span: tab.ms(l.decodes.span), requests: l.decodes.requests, cached: l.decodes.cached, blocked: l.decodes.blocked }) }
                    Repeater {
                        model: l.decodes.slowest
                        Label { required property var modelData; width: parent.width; role: "ledger"; elide: Text.ElideMiddle; color: theme.textDim
                            text: words.t("settings-developer-decode", { ms: modelData.ms.toFixed(1), srcW: modelData.srcW, srcH: modelData.srcH, w: modelData.w, h: modelData.h, size: tab.rec.kb(modelData.bytes) }) }
                    }
                    Label { visible: l.decodes.n === 0; text: l.images ? words.t("settings-developer-decodes-none") : words.t("settings-developer-images-were-off"); role: "ledger"; color: theme.textFaint }
                }
                Column {
                    width: detailCols.third
                    spacing: theme.space(1)
                    Label { msg: "settings-developer-scene"; role: "title"; font.pointSize: theme.typeNormal; bottomPadding: theme.space(1) }
                    Label { width: parent.width; wrapMode: Text.Wrap; role: "ledger"; color: theme.textDim
                        text: words.t("settings-developer-scene-line", { items: l.scene.items, offscreen: l.scene.offscreen, texts: l.scene.texts, shapes: l.scene.shapes, icons: l.scene.icons,
                              images: l.scene.images.total, imagesOffscreen: l.scene.images.offscreen, size: tab.rec.mb(l.scene.images.offscreenPixels * 4) }) }
                    Repeater {
                        model: l.scene.types.slice(0, 12)
                        Label { required property var modelData; role: "ledger"; color: theme.textDim; text: modelData[1] + "  " + modelData[0] }
                    }
                }
            }
            Button { msg: "settings-developer-copy-load"; icon: "copy"; small: true; onClicked: frame.copyText(tab.rec.describe(l)) }
        }
    }
    Component {
        id: tourPanel
        Panel {
            title: words.t("settings-developer-tour-title"); icon: "gauge"
            helper: words.t("settings-developer-tour-helper")
            Column {
                width: parent.width
                Row {
                    height: theme.space(8)
                    Repeater {
                        model: [[words.t("settings-developer-col-page"), 14], [words.t("settings-developer-col-built-on"), 11], [words.t("settings-developer-col-built-off"), 11],
                                [words.t("settings-developer-col-settled-on"), 11], [words.t("settings-developer-col-settled-off"), 11], [words.t("settings-developer-col-items"), 9],
                                [words.t("settings-developer-col-decodes-time"), 13], [words.t("settings-developer-col-data"), 10], [words.t("settings-developer-col-memory"), 10]]
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
                Label { topPadding: theme.space(2); role: "ledger"; color: theme.textDim; text: words.t("settings-developer-tour-peak", { size: tab.mb(tab.rec.tourResult.peak, true) }) }
            }
        }
    }
    Component {
        id: eventsPanel
        Panel {
            title: words.t("settings-developer-events"); icon: "activity"
            helper: words.t("settings-developer-events-helper")
            Flow {
                width: parent.width
                spacing: theme.space(2)
                Repeater {
                    model: tab.eventRates
                    Chip { required property var modelData; small: true; color: theme.tileSunken; textColor: theme.textDim
                        text: words.t("settings-developer-event-rate", { kind: modelData.kind, count: modelData.n, size: tab.rec.kb(modelData.bytes) }) }
                }
            }
            Field { width: Math.min(parent.width, theme.space(120)); placeholder: words.t("settings-developer-events-filter"); onEdited: function(t) { tab.eventFilter = t } }
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
                        Label { width: theme.space(14); role: "ledger"; color: theme.textFaint; text: words.t("settings-developer-seconds", { seconds: (modelData.at / 1000).toFixed(1) }) }
                        Label { width: theme.space(10); role: "ledger"; text: String(modelData.level); color: modelData.level === "Error" ? theme.red : modelData.level === "Warn" ? theme.orange : modelData.level === "Debug" ? theme.textFaint : theme.textDim }
                        Label { width: theme.space(22); role: "ledger"; color: theme.textDim; text: String(modelData.kind); elide: Text.ElideRight }
                        Label { width: theme.space(12); role: "ledger"; color: theme.textFaint; text: tab.rec.kb(modelData.bytes) }
                        Label { width: list.width - theme.space(66); role: "ledger"; elide: Text.ElideRight; color: theme.text
                            text: modelData.plain ? words.t("settings-developer-event-plain", { message: modelData.message, plain: modelData.plain }) : modelData.message }
                    }
                }
                QC.ScrollBar.vertical: ThinScrollBar {}
            }
            Button { msg: "settings-developer-copy-events"; icon: "copy"; small: true
                onClicked: frame.copyText(tab.shownEvents.map(function(e) { return (e.at / 1000).toFixed(3) + " " + e.level + " " + e.stage + " " + e.kind + " " + e.bytes + "B " + e.message }).join("\n")) } // i18n: none
        }
    }
}
