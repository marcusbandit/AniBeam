// One Lucide glyph, tinted. The SVGs under assets/icons carry a black stroke (QtSvg does not
// read currentColor), and ColorImage recolours every opaque pixel, so `color` is the whole
// tint. ColorImage, not IconImage: IconImage's load runs QIconLoader's icon-theme search
// over the filesystem and runs again on every geometry change, which was most of the
// prototype's start-up and re-layout time (55 percent of a perf profile of the first three
// seconds). `glyph` is the Lucide name; `size` is the box, a text-height glyph by default.
import QtQuick
import QtQuick.Controls.impl

ColorImage {
    id: root
    property string glyph: ""
    property real size: theme.space(5)

    source: glyph !== "" ? "qrc:/qt/qml/com/marcusrosado/AniBeam/assets/icons/" + glyph + ".svg" : ""
    color: theme.text
    sourceSize: Qt.size(size, size)
    width: size
    height: size
    fillMode: Image.PreserveAspectFit
    smooth: true
}
