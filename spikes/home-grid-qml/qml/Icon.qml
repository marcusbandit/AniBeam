// One Lucide glyph, tinted. The SVGs under assets/icons carry a black stroke (QtSvg does not
// read currentColor), and IconImage recolours every opaque pixel, so `color` is the whole
// tint. `glyph` is the Lucide name (IconImage keeps `name` for theme icons); `size` is the
// box, a text-height glyph by default.
import QtQuick
import QtQuick.Controls.impl

IconImage {
    id: root
    property string glyph: ""
    property real size: theme.space(5)

    source: glyph !== "" ? "qrc:/qt/qml/dev/anibeam/proto/assets/icons/" + glyph + ".svg" : ""
    color: theme.text
    sourceSize: Qt.size(size, size)
    width: size
    height: size
    fillMode: Image.PreserveAspectFit
    smooth: true
}
