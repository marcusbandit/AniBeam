// The shows hidden from Airing now (#1012), a page of Settings of its own that Library opens:
// Back to Library, the title with the count, a filter on the titles, then one row per mark,
// newest first, each with its poster and Show again. The rows are a ListView, so a list of a
// hundred and more builds only what is on screen. Opening it checks the marks against the
// seasons Airing now has already read (PruneNotInterested): a show none of them carries has
// stopped airing and its mark goes, with nothing asked of AniList.
import QtQuick
import QtQuick.Controls.Basic as QC
import com.marcusrosado.Anibeam

Item {
    id: tab
    anchors.fill: parent
    // What SettingsPage keeps and restores per section
    property alias contentY: list.contentY
    function reveal(text) { return false }

    property var shows: []
    property string filter: ""
    readonly property var shown: {
        var q = filter.trim().toLowerCase()
        return q === "" ? shows : shows.filter(function(s) { return s.title.toLowerCase().indexOf(q) >= 0 })
    }
    readonly property real blockWidth: Math.min(width - theme.space(6), page.sectionWidth || theme.space(210))

    function load() {
        var before = Door.listNotInterested()
        var r = Door.pruneNotInterested()
        if (r.error) { frame.fail(r.error, tab.load); return }
        shows = r.reply.shows
        var ended = before.error ? 0 : before.reply.shows.length - shows.length
        if (ended > 0) frame.toast(words.t("settings-hidden-airing-ended", { count: ended }))
    }
    function showAgain(id) {
        var r = Door.setNotInterested(id, false)
        if (r.error) { frame.fail(r.error, function() { tab.showAgain(id) }); return }
        shows = shows.filter(function(s) { return s.anilist_id !== id })
    }
    Component.onCompleted: load()

    Column {
        id: head
        width: tab.blockWidth
        spacing: theme.space(4)
        Button {
            objectName: "settings-hidden-back"
            msg: "settings-library-title"
            icon: "arrow-left"
            flat: true; small: true
            onClicked: page.open("library")
        }
        Row {
            spacing: theme.space(3)
            Label {
                id: titleLabel
                msg: "settings-library-hidden-from-airing"
                role: "display"
                font.pointSize: theme.typeHeading
            }
            Label {
                anchors.baseline: titleLabel.baseline
                text: words.t("count-shows", { count: tab.shows.length })
                role: "ui"
                color: theme.textDim
            }
        }
        SearchField {
            id: search
            objectName: "settings-hidden-filter"
            // Shown for a long list, and kept while it is filtering so it can be cleared
            visible: tab.shows.length > 8 || tab.filter !== ""
            width: Math.min(parent.width, theme.space(100))
            placeholder: words.t("settings-hidden-airing-filter")
            onTextChanged: tab.filter = text
            onCleared: tab.filter = ""
        }
    }

    Rectangle {
        id: pane
        y: head.height + theme.space(6)
        width: tab.blockWidth
        height: Math.min(tab.height - y - theme.space(6), list.contentHeight + theme.space(3))
        visible: tab.shown.length > 0
        radius: theme.radiusTile
        color: theme.tile
        ListView {
            id: list
            objectName: "settings-hidden-list"
            x: theme.space(5); y: theme.space(1.5)
            width: parent.width - theme.space(10)
            height: parent.height - theme.space(3)
            clip: true
            boundsBehavior: Flickable.StopAtBounds
            model: tab.shown
            reuseItems: true
            WheelStep { flick: list }
            QC.ScrollBar.vertical: ThinScrollBar {}
            delegate: Item {
                id: row
                required property var modelData
                required property int index
                objectName: "not-interested-" + modelData.anilist_id
                width: list.width
                height: theme.space(16)
                Rectangle {
                    id: thumb
                    anchors.verticalCenter: parent.verticalCenter
                    height: parent.height - theme.space(4)
                    width: Math.round(height * 2 / 3)
                    radius: theme.radiusSm
                    color: theme.raised
                    clip: true
                    Image {
                        anchors.fill: parent
                        source: row.modelData.poster ? Fmt.fileUrl(row.modelData.poster) : ""
                        sourceSize: Qt.size(Math.ceil(width * theme.devicePixelRatio), Math.ceil(height * theme.devicePixelRatio))
                        fillMode: Image.PreserveAspectCrop
                        asynchronous: true
                    }
                }
                Label {
                    anchors.left: thumb.right; anchors.leftMargin: theme.space(4)
                    anchors.right: again.left; anchors.rightMargin: theme.space(4)
                    anchors.verticalCenter: parent.verticalCenter
                    text: row.modelData.title
                    role: "ui"
                    elide: Text.ElideRight
                }
                Button {
                    id: again
                    objectName: "not-interested-again-" + row.modelData.anilist_id
                    anchors.right: parent.right
                    anchors.verticalCenter: parent.verticalCenter
                    msg: "settings-library-show-again"; icon: "eye"; flat: true; small: true
                    onClicked: tab.showAgain(row.modelData.anilist_id)
                }
                Rectangle {
                    visible: row.index < list.count - 1
                    anchors.bottom: parent.bottom
                    width: parent.width; height: 1
                    color: theme.line
                }
            }
        }
    }

    Label {
        visible: tab.shown.length === 0
        y: head.height + theme.space(6)
        text: tab.shows.length === 0 ? words.t("settings-library-hidden-from-airing-empty") : words.t("settings-results-none")
        role: "ui"
        color: theme.textDim
    }
}
