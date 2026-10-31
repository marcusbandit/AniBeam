// Keyboard: every key AniBeam answers to, read off KeyHelp (never shown here) so the list
// is the one the player itself shows on ?, a group per KeyHelp group, the keys as chips at
// the right of what they do.
import QtQuick
import com.marcusrosado.AniBeam

SettingsTab {
    id: tab
    title: "Keyboard"
    lead: "Every key AniBeam answers to. Press ? in the player for the same list over the video."
    property Item keyMap: KeyHelp { visible: false; open: false }
    readonly property var names: ({ Playback: "In the player", Navigation: "Getting around the player", Library: "Anywhere" })

    Repeater {
        model: tab.keyMap.groups
        Panel {
            id: group
            required property var modelData
            title: tab.names[modelData.name] || modelData.name
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
