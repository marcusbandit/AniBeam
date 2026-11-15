// Library's tabs (#1012): one segmented row on four pages. The library's own
// tabs come first on the library page (All, Series, Movies, and Watched or Hidden when it has
// them), and one Library cell on the others; then the three pages that live under Library
// instead of in the rail: On my list (Watching), Stats and Fix matches (Show info). A library
// tab filters the library in place (`filterPicked`); the Library cell goes back to the library.
// Every move between them is sideways, so Back leaves Library rather than stepping
// through its tabs.
import QtQuick
import com.marcusrosado.AniBeam

Seg {
    id: root
    objectName: "library-tabs"
    // The page this row sits on: "library", "watching", "stats" or "metadata"
    property string here: "library"
    // The library's own tabs, as LibraryPage builds them ({ text, name, count }). The other
    // pages show one cell, Library, in their place: they have filters of their own, and two
    // rows both starting "All, Series" read as one
    property var filters: [{ text: words.t("rail-library"), name: "library" }]
    // Which of `filters` is lit on the library page
    property int filterIndex: 0
    signal filterPicked(int i)

    readonly property var pages: [
        { page: "watching", text: words.t("library-tab-on-my-list"), name: "on-my-list" },
        { page: "stats", text: words.t("library-tab-stats"), name: "stats" },
        { page: "metadata", text: words.t("library-tab-fix-matches"), name: "fix-matches" }
    ]
    options: filters.concat(pages)
    index: here === "library" ? filterIndex : filters.length + pages.findIndex(function(p) { return p.page === root.here })
    accessibleName: words.t("rail-library")
    onPicked: function(i) {
        if (i < filters.length) {
            if (here === "library") root.filterPicked(i)
            else frame.nav.replace("library", {}, "")
            return
        }
        var to = pages[i - filters.length].page
        if (to !== here) frame.nav.replace(to, {}, "")
    }
}
