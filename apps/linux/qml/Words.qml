// The words on screen (#792): every text asks here by message ID, and the catalog under
// i18n/ holds the words in every language. Main.qml makes the one instance, `words`, which
// every component reaches the way it reaches `theme`.
//
// t() reads Tr.lang before it looks the words up, so a binding that calls it is re-run when
// the language changes, and the screen switches without a restart.
import QtQuick
import QtCore
import com.marcusrosado.AniBeam

QtObject {
    id: root
    readonly property string lang: Tr.lang

    // The language picked in Settings or onboarding, by code; "" follows the system's.
    // ANIBEAM_LANG pins a run to one language (captures), and then the pick waits.
    readonly property string picked: kept.language
    readonly property string systemCode: Tr.systemCode()
    property Settings kept: Settings {
        location: Shell.settingsUrl
        category: "shell"
        property string language: ""
    }
    function pick(code) {
        kept.language = code
        if (!Tr.pinned) Tr.setLanguage(code || systemCode)
    }
    Component.onCompleted: if (!Tr.pinned) Tr.setLanguage(kept.language || systemCode)
    // Every language as a picker lists them: { code, name } in the language itself
    readonly property var languages: Tr.languages()

    // The words for message `id`, with `args` ({ count: 3, title: "..." }) filled in. A
    // number in args also picks the plural.
    function t(id, args) {
        return lang, Tr.lookup(id, args || ({}))
    }

    // Fmt's readouts that hold words, through the same language read, so they follow a
    // switch too. QML calls these, never Fmt's own.
    function relative(ts, now) { return lang, Fmt.relative(ts, now) }
    function countdown(secs) { return lang, Fmt.countdown(secs) }
    function countdownSeconds(secs) { return lang, Fmt.countdownSeconds(secs) }
    function runtime(minutes) { return lang, Fmt.runtime(minutes) }
    function episodeName(title, number) { return lang, Fmt.episodeName(title, number) }
    function episodeSpan(first, through) { return lang, Fmt.episodeSpan(first, through) }
    function bytes(n) { return lang, Fmt.bytes(n) }
    function score(x) { return lang, Fmt.score(x) }
    // AniList's words for a show, by its code, "" for a code with no words yet. The status
    // takes AniList's own codes (RELEASING) and the core's (Releasing) alike.
    function formatLabel(code) {
        var id = ({ TV: "anilist-format-tv", TV_SHORT: "anilist-format-tv-short", MOVIE: "anilist-format-film", SPECIAL: "anilist-format-special",
                    OVA: "anilist-format-ova", ONA: "anilist-format-ona", MUSIC: "anilist-format-music" })[code]
        return id ? t(id) : ""
    }
    function releaseStatus(code) {
        var id = ({ FINISHED: "anilist-status-finished", RELEASING: "anilist-status-airing", NOT_YET_RELEASED: "anilist-status-not-yet-aired",
                    CANCELLED: "anilist-status-cancelled", HIATUS: "anilist-status-hiatus" })[String(code || "").replace(/([a-z])([A-Z])/g, "$1_$2").toUpperCase()]
        return id ? t(id) : ""
    }
    function relationLabel(code) {
        var id = ({ ADAPTATION: "anilist-relation-adaptation", PREQUEL: "anilist-relation-prequel", SEQUEL: "anilist-relation-sequel", PARENT: "anilist-relation-parent",
                    SIDE_STORY: "anilist-relation-side-story", CHARACTER: "anilist-relation-character", SUMMARY: "anilist-relation-summary",
                    ALTERNATIVE: "anilist-relation-alternative", SPIN_OFF: "anilist-relation-spin-off", OTHER: "anilist-relation-other",
                    SOURCE: "anilist-relation-source", COMPILATION: "anilist-relation-compilation", CONTAINS: "anilist-relation-contains" })[code]
        return id ? t(id) : ""
    }
    // An episode number as a message argument the way episodeName rounds it (12.5, never
    // 12.333), or null when there is none, so the caller can pick a message without one.
    function episodeNumber(n) {
        return (typeof n === "number" && isFinite(n)) ? Math.round(n * 100) / 100 : null
    }
}
