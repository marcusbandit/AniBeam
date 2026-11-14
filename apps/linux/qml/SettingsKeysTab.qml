// Keyboard: every key Anibeam answers to, read off KeyHelp (never shown here) so the list
// is the one ? shows: the player's two groups, then the frame's two, the keys as chips at
// the right of what they do.
import QtQuick
import com.marcusrosado.AniBeam

SettingsTab {
    id: tab
    title: words.t("settings-keys-title")
    lead: words.t("settings-keys-lead")
    property Item keyMap: KeyHelp { visible: false; open: false }
    // The player's own keys first, then the ones that work on every page
    readonly property var groups: [
        { title: words.t("settings-keys-player"), keys: keyMap.playerGroups[0].keys },
        { title: words.t("settings-keys-player-around"), keys: keyMap.playerGroups[1].keys },
        { title: words.t("settings-keys-around"), keys: keyMap.appGroups[0].keys },
        { title: words.t("settings-keys-anywhere"), keys: keyMap.appGroups[1].keys }
    ]

    Repeater {
        model: tab.groups
        Panel {
            id: group
            required property var modelData
            title: modelData.title
            Repeater {
                model: group.modelData.keys
                SettingRow {
                    id: keyRow
                    required property var modelData
                    label: modelData[1]
                    Row {
                        spacing: theme.space(1.5)
                        Repeater {
                            // "Left / Right" is two keys, so two chips
                            model: String(keyRow.modelData[0]).split(" / ")
                            Chip { required property var modelData; text: modelData; small: true; color: theme.raised }
                        }
                    }
                }
            }
        }
    }
}
