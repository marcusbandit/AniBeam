// The words on screen (#792): every text asks here by message ID, and the catalog under
// i18n/ holds the words in every language. Main.qml makes the one instance, `words`, which
// every component reaches the way it reaches `theme`.
//
// t() reads Tr.lang before it looks the words up, so a binding that calls it is re-run when
// the language changes, and the screen switches without a restart.
import QtQuick
import com.marcusrosado.AniBeam

QtObject {
    readonly property string lang: Tr.lang

    // The words for message `id`, with `args` ({ count: 3, title: "..." }) filled in. A
    // number in args also picks the plural.
    function t(id, args) {
        return lang, Tr.lookup(id, args || ({}))
    }
}
