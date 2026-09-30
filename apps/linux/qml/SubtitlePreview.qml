// The subtitle preview: the player's own item, paused on a frame, re-applying the
// defaults on every change. A QML approximation is rejected by spec 4.5.
import QtQuick
import com.marcusrosado.AniBeam

Corner {
    id: root
    property var defaults: ({})
    property string path: ""
    property string subtitle: ""
    property real startAt: 0
    radius: theme.radiusMd; smoothing: theme.cornerSmoothing; color: theme.surfaceSunken; borderColor: theme.line; borderWidth: 1
    fillItem: video.isReady ? video : null
    onDefaultsChanged: apply()
    // `path` can arrive either before or after mpv's core finishes initializing (the
    // episode lookup that fills it is synchronous, but so is nothing about mpv's own
    // startup), so both directions call this rather than only the one `onReady` sees.
    onPathChanged: load()
    function apply() {
        if (!video.isReady) return
        var opts = Player.subtitleOptions(defaults)
        for (var i = 0; i < opts.length; i++) video.setProperty(opts[i][0], opts[i][1])
    }
    function load() {
        if (!video.isReady || root.path === "") return
        video.setProperty("start", root.startAt > 0 ? String(root.startAt) : "none")
        video.command(["loadfile", root.path])
    }
    VideoItem {
        id: video
        width: parent.width; height: parent.height
        property bool isReady: false
        onReady: {
            var layers = Player.configLayers
            for (var i = 0; i < layers.length; i++) include(layers[i])
            var o = Player.previewOptions
            for (var j = 0; j < o.length; j++) if (o[j][0] !== "sid" && o[j][0] !== "sub-auto") setProperty(o[j][0], o[j][1])
            setProperty("sid", "auto")
            isReady = true
            root.apply()
            root.load()
        }
        onLoaded: { if (root.subtitle !== "") command(["sub-add", root.subtitle, "select"]); root.apply() }
    }
}
