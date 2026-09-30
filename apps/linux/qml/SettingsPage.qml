// Spec 4.5: Settings as four tabs in a segmented switch, each two panel columns that fill
// the viewport. The tab and each tab's scroll are session state kept on the frame.
import QtQuick
import com.marcusrosado.AniBeam

FocusScope {
    id: page
    property var props: ({})
    property string title: "Settings"
    property real scrollY: 0
    readonly property var tabNames: ["Library", "Appearance", "Playback", "Data"]
    readonly property var tabIcons: ["folder", "palette", "play", "hard-drive"]
    property int tab: frame.settingsTab
    onTabChanged: frame.settingsTab = tab
    Component.onCompleted: {
        var want = tabNames.map(function(n) { return n.toLowerCase() }).indexOf(String(props.tab || "").toLowerCase())
        if (want >= 0) tab = want
        forceActiveFocus()
    }
    // Named escapePressed, not escape: QML's compiler reserves "escape" and refuses it as a
    // property, method or signal name on any object.
    function escapePressed() { return false }

    Column {
        id: head
        anchors.top: parent.top; anchors.topMargin: theme.space(7)
        x: content.item ? content.item.blockX + theme.space(8) : theme.space(8)
        spacing: theme.space(4)
        Text { text: "Settings"; color: theme.text; font.family: theme.fontSans; font.pointSize: theme.typeLarge; font.weight: Font.Bold }
        Seg {
            options: page.tabNames.map(function(n, i) { return { text: n, icon: page.tabIcons[i] } })
            index: page.tab
            onPicked: function(i) { page.tab = i }
        }
    }
    Loader {
        id: content
        anchors.top: head.bottom; anchors.topMargin: theme.space(6)
        anchors.left: parent.left; anchors.right: parent.right; anchors.bottom: parent.bottom
        sourceComponent: [libraryTab, appearanceTab, playbackTab, dataTab][page.tab]
        onLoaded: { item.contentY = frame.settingsScroll[page.tab] || 0 }
        Connections { target: content.item; function onContentYChanged() { var s = frame.settingsScroll.slice(); s[page.tab] = content.item.contentY; frame.settingsScroll = s } }
    }
    Component { id: libraryTab; SettingsLibraryTab {} }
    Component { id: appearanceTab; SettingsAppearanceTab {} }
    Component { id: playbackTab; SettingsPlaybackTab {} }
    Component { id: dataTab; SettingsDataTab {} }
}
