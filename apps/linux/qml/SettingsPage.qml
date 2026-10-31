// Settings, flat (DESIGN.md, the flat home pass): a head row at the page margin, the
// title in the display cut with the section switch at the right, and under it the chosen
// tab's flat sections filling the rest of the viewport. The tab and each tab's scroll are
// session state kept on the frame.
import QtQuick
import com.marcusrosado.AniBeam

FocusScope {
    id: page
    objectName: "settings-page"
    property var props: ({})
    property string title: "Settings"
    property real scrollY: 0
    // Developer joins the tabs while developer options are on (#287)
    readonly property var tabNames: ["Library", "Appearance", "Playback", "Downloads", "Data"].concat(frame.devOn ? ["Developer"] : [])
    readonly property var tabIcons: ["folder", "palette", "play", "download", "hard-drive"].concat(frame.devOn ? ["bug"] : [])
    property int tab: Math.min(frame.settingsTab, tabNames.length - 1)
    onTabChanged: frame.settingsTab = tab
    Component.onCompleted: {
        var want = tabNames.map(function(n) { return n.toLowerCase() }).indexOf(String(props.tab || "").toLowerCase())
        if (want >= 0) tab = want
        forceActiveFocus()
    }
    // Named escapePressed, not escape: QML's compiler reserves "escape" and refuses it as a
    // property, method or signal name on any object.
    function escapePressed() { return false }

    // The page's margins: the same centred column PageScroll gives every other page
    readonly property real sideInset: Math.round((page.width - Math.min(page.width, theme.contentMax)) / 2) + theme.space(8)
    readonly property real topInset: theme.space(7)

    // ---- The head: the title, the section switch at the right
    Item {
        id: head
        x: page.sideInset
        y: page.topInset
        width: page.width - page.sideInset * 2
        height: theme.space(20)
        Label {
            id: titleLabel
            anchors.left: parent.left
            anchors.verticalCenter: parent.verticalCenter
            text: "Settings"
            role: "display"
            font.pointSize: theme.typePage
        }
        Seg {
            id: tabsSeg
            objectName: "settings-tabs"
            anchors.right: parent.right
            anchors.verticalCenter: parent.verticalCenter
            options: page.tabNames.map(function(n, i) { return { text: n, icon: page.tabIcons[i] } })
            index: page.tab
            onPicked: function(i) { page.tab = i }
        }
    }
    Loader {
        id: content
        x: 0
        y: head.y + head.height + theme.space(4)
        width: page.width
        height: page.height - y
        sourceComponent: [libraryTab, appearanceTab, playbackTab, downloadsTab, dataTab, developerTab][page.tab]
        onLoaded: {
            item.contentY = frame.settingsScroll[page.tab] || 0
            // `focus` in the props names a section to bring into view (Airing now's link to
            // its not interested list), once the tab has laid out
            if (page.props.focus && item.focusSection) focusLater.restart()
        }
        Timer { id: focusLater; interval: 150; onTriggered: if (content.item && content.item.focusSection) content.item.focusSection(page.props.focus) }
        Connections { target: content.item; function onContentYChanged() { var s = frame.settingsScroll.slice(); s[page.tab] = content.item.contentY; frame.settingsScroll = s } }
    }
    Component { id: libraryTab; SettingsLibraryTab {} }
    Component { id: appearanceTab; SettingsAppearanceTab {} }
    Component { id: playbackTab; SettingsPlaybackTab {} }
    Component { id: downloadsTab; SettingsDownloadsTab {} }
    Component { id: dataTab; SettingsDataTab {} }
    Component { id: developerTab; SettingsDeveloperTab {} }
}
