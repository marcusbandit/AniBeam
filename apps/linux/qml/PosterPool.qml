// The poster pool: the library's posters, decoded once and kept alive, so the
// first visit to any page finds its pictures already in the engine's pixmap
// cache. The pool holds up to `capacity` Image elements — referenced pixmaps
// are never evicted — ordered the way the pages show them: airing soonest
// first, then recently viewed, then the shelf. Page Images share the cache by
// URL and size, so a warmed poster is a pointer, not a decode.
import QtQuick
import com.marcusrosado.AniBeam

Item {
    id: root
    property int capacity: 100
    property var paths: []
    width: 0; height: 0
    visible: false

    Repeater {
        model: root.paths.slice(0, root.capacity).map(function(p) { return "file://" + p })
        Image {
            required property var modelData
            source: modelData
            sourceSize.width: 480
            fillMode: Image.PreserveAspectCrop
            asynchronous: true
            cache: true
            smooth: false
        }
    }
}
