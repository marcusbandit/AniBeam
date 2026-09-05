// The derived tokens. Instantiated once at the root of Main.qml as `theme`, which every
// component reaches through the context chain; LookPane instantiates its own with `mode`
// forced, so a preview renders the other mode with the same components. Colours come from
// the Rust Theme singleton; the sizes are arithmetic on its settings.
import QtQuick
import com.marcusrosado.AniBeam

Item {
    id: root
    property string mode: Theme.resolvedMode
    readonly property bool dark: mode !== "light"
    readonly property var set: dark ? Theme.dark : Theme.light
    function token(name) { var v = set[name.replace(".", "_")]; return v === undefined ? "#ff00ff" : v }

    // Colours
    readonly property color bg: token("bg")
    readonly property color surface: token("surface")
    readonly property color surfaceRaised: token("surface_raised")
    readonly property color surfaceSunken: token("surface_sunken")
    readonly property color surfacePressed: token("surface_pressed")
    readonly property color line: token("line")
    readonly property color lineStrong: token("line_strong")
    readonly property color text: token("text")
    readonly property color textDim: token("text_dim")
    readonly property color textFaint: token("text_faint")
    readonly property color accent: token("accent")
    readonly property color accentText: token("accent_text")
    readonly property color accentSoft: token("accent_soft")
    readonly property color redSoft: token("red_soft")
    readonly property color focusRing: token("focus")
    readonly property color red: token("red")
    readonly property color orange: token("orange")
    readonly property color yellow: token("yellow")
    readonly property color green: token("green")
    readonly property color cyan: token("cyan")
    readonly property color blue: token("blue")
    readonly property color purple: token("purple")
    readonly property color brown: token("brown")
    readonly property color scrim: Qt.rgba(bg.r, bg.g, bg.b, 0.8)
    readonly property string sourceLabel: Theme.sourceLabel

    // Status and fraction colours: fixed mappings onto the hues
    readonly property color statusWatching: accent
    readonly property color statusCompleted: blue
    readonly property color statusPaused: yellow
    readonly property color statusDropped: red
    readonly property color statusPlanning: textFaint
    readonly property color statusRewatching: purple
    readonly property color behind: yellow
    readonly property color caughtUp: accent
    // A hue by the name Theme.formatHue or Theme.statusHue returns
    function hue(name) { return token(name) }

    // Spacing, radii, type, motion
    readonly property real densityFactor: Theme.densityFactor
    function space(n) { return Math.round(4 * densityFactor * n) }
    readonly property real cornerBase: 14
    readonly property real cornerSmoothing: Theme.smoothing
    readonly property real radiusSm: cornerBase * densityFactor
    readonly property real radiusMd: cornerBase * 1.4 * densityFactor
    readonly property real radiusLg: cornerBase * 1.4 * 1.4 * densityFactor
    readonly property real radiusXl: cornerBase * 1.4 * 1.4 * 1.4 * densityFactor
    readonly property int posterWidth: Theme.posterWidth
    readonly property real systemPointSize: Qt.application.font.pointSize > 0 ? Qt.application.font.pointSize : 10
    readonly property real typeNormal: systemPointSize
    readonly property real typeSmall: systemPointSize * 0.85
    readonly property real typeLarge: systemPointSize * 1.4
    readonly property string fontSans: Qt.application.font.family
    readonly property string fontMono: "monospace"
    readonly property int motionFast: 120
    readonly property int motionNormal: 200
    readonly property int motionSlow: 320
    readonly property real controlHeight: space(8)
    readonly property real disabledOpacity: 0.45

    // A colour t of the way from a to b, alpha included; takes Qt colours
    function tone(a, b, t) {
        t = Math.max(0, Math.min(1, t))
        return Qt.rgba(a.r + (b.r - a.r) * t, a.g + (b.g - a.g) * t, a.b + (b.b - a.b) * t, a.a + (b.a - a.a) * t)
    }
}
