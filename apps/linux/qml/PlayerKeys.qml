// The player's keyboard map, spec 4.4. Split out of PlayerPage.qml alongside the chrome: the
// page keeps the Keys attached property, since it is the item that holds focus, and hands the
// event straight here. KeyHelp.qml lists exactly what this file acts on, so a key added here
// gets a line there.
import QtQuick
import QtQuick.Window
import com.marcusrosado.AniBeam

QtObject {
    id: root
    // The keys a held press repeats: the seeks, the volume ramp and the two frame steps, as
    // every player does. Everything else acts once however long it is held.
    readonly property var repeatKeys: [Qt.Key_Left, Qt.Key_Right, Qt.Key_Up, Qt.Key_Down]

    function handle(e) {
        // Frame step sits above every guard: it repeats, it carries no modifier of its own,
        // and it must not bring the chrome back, so it never reaches the branches below.
        if ((e.key === Qt.Key_Period || e.key === Qt.Key_Comma) && !(e.modifiers & (Qt.ControlModifier | Qt.AltModifier | Qt.MetaModifier))) {
            e.accepted = true
            page.step(e.key === Qt.Key_Period ? 1 : -1)
            return
        }
        // Escape swallows its own repeat rather than falling through: unaccepted, the frame
        // would take the second press and leave the player on a key that was held, not hit.
        if (e.isAutoRepeat && e.key === Qt.Key_Escape) { e.accepted = true; return }
        if (e.isAutoRepeat && root.repeatKeys.indexOf(e.key) < 0) { e.accepted = false; return }
        // Ctrl+Right is the one combination the player claims, and it repeats like Right.
        if (e.key === Qt.Key_Right && (e.modifiers & Qt.ControlModifier) && !(e.modifiers & (Qt.AltModifier | Qt.MetaModifier))) {
            e.accepted = true
            page.skipForward()
            return
        }
        // Every other Ctrl, Alt and Meta press belongs to the frame's shortcuts. Shift
        // passes through: z and Z differ by it, and ? is Shift and the slash key.
        if (e.modifiers & (Qt.ControlModifier | Qt.AltModifier | Qt.MetaModifier)) { e.accepted = false; return }
        e.accepted = true
        if (e.key === Qt.Key_Space || e.key === Qt.Key_K) page.togglePause()
        else if (e.key === Qt.Key_Left) page.seekTo(page.timePos - 5)
        else if (e.key === Qt.Key_Right) page.seekTo(page.timePos + 5)
        else if (e.key === Qt.Key_M) page.setMute(!Player.mute)
        else if (e.key === Qt.Key_F) page.toggleFullscreen()          // F does not bring the chrome back
        else if (e.key === Qt.Key_Up) page.setVolume(Player.volume + 5)
        else if (e.key === Qt.Key_Down) page.setVolume(Player.volume - 5)
        else if (e.key === Qt.Key_C) page.toggleSubtitles()
        else if (e.key === Qt.Key_Z && !(e.modifiers & Qt.ShiftModifier)) page.nudgeDelay(-0.1)
        else if (e.key === Qt.Key_Z) page.nudgeDelay(0.1)
        else if (e.key === Qt.Key_Question) page.toggleHelp()          // R33: a second press closes it
        // A popover is open: the press belongs to the frame's escape stack, which closes the
        // topmost one, the key list before a track picker. Accepting it here would leave the
        // player and take the popover with it.
        else if (e.key === Qt.Key_Escape) {
            if (page.openMenus > 0) e.accepted = false
            else if (frame.hostWindow.visibility === Window.FullScreen) frame.hostWindow.visibility = Window.Windowed
            else page.leave()
        }
        else e.accepted = false
    }
}
