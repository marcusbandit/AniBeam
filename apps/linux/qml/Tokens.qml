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

    // The one face, Archivo, variable in width (wdth 62 to 125) and weight (wght 100 to 900),
    // from the qrc so the packaged binary is self-contained. It ships as three cuts of the one
    // variable font, registered as one family: Qt registers only a variable font's default
    // instance and never maps font.weight onto the wght axis, so a single file would set
    // every Text without variableAxes at its default (SemiBold). Each cut keeps all of wdth
    // and a slice of wght (100-500, 500-700, 700-900) around its default, so font.weight
    // picks a cut and a role's axes land inside it (scripts/cut-fonts.py). Multiple Tokens
    // instances share the engine's font cache, so a second loader on a URL costs nothing.
    // Until the loader resolves, text falls back to the system face for a frame at most.
    FontLoader { id: face; source: "qrc:/qt/qml/com/marcusrosado/AniBeam/assets/fonts/Archivo-Variable-Regular.ttf" }
    FontLoader { source: "qrc:/qt/qml/com/marcusrosado/AniBeam/assets/fonts/Archivo-Variable-SemiBold.ttf" }
    FontLoader { source: "qrc:/qt/qml/com/marcusrosado/AniBeam/assets/fonts/Archivo-Variable-ExtraBold.ttf" }
    readonly property string fontFamily: face.name !== "" ? face.name : Qt.application.font.family
    readonly property string fontDisplay: fontFamily

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
    // The mosaic's names for the ramp: the ground is the grout every tile is set in, a tile
    // rests on it, a raised tile is hovered, selected or floating (menus, tips, sheets), a
    // sunken tile takes input (fields), and the line is the rare hairline.
    readonly property color ground: bg
    readonly property color tile: surface
    readonly property color raised: surfaceRaised
    readonly property color tileSunken: surfaceSunken
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
    readonly property real cornerBase: 4
    readonly property real cornerSmoothing: Theme.smoothing
    readonly property real radiusSm: cornerBase * densityFactor
    readonly property real radiusMd: cornerBase * 1.4 * densityFactor
    readonly property real radiusLg: cornerBase * 1.4 * 1.4 * densityFactor
    readonly property real radiusXl: cornerBase * 1.4 * 1.4 * 1.4 * densityFactor
    // The mosaic's geometry: one joint between every pair of tiles, one tile radius, and a
    // radius for anything set inside a tile at `padding` from its edge so the corners nest.
    // Pills (chips, segmented switches, buttons) are radius: height / 2.
    readonly property real grout: Math.round(6 * densityFactor)
    readonly property real radiusTile: 12 * densityFactor
    function radiusInner(padding) { return Math.max(2, radiusTile - padding) }
    readonly property int posterWidth: Theme.posterWidth
    // The one size posters decode at, in device pixels: a grid card is laid out between the
    // poster width and about a quarter wider (the columns stretch to fill the row), so the
    // decode covers that at the screen's pixel ratio. Cards and the poster pool share it,
    // and the pixmap cache keys on it, so a warmed poster is a hit, never a second decode.
    readonly property real devicePixelRatio: Screen.devicePixelRatio > 0 ? Screen.devicePixelRatio : 1
    readonly property size posterDecodeSize: Qt.size(Math.ceil(posterWidth * 1.25 * devicePixelRatio), Math.ceil(posterWidth * 1.25 * 1.5 * devicePixelRatio))
    readonly property real systemPointSize: Qt.application.font.pointSize > 0 ? Qt.application.font.pointSize : 10
    readonly property string fontSans: fontFamily
    // Shims kept while the surfaces are restyled: the old names all resolve to Archivo.
    // There is no monospace anywhere; tabular figures come from the ledger role.
    readonly property string fontUi: fontFamily
    readonly property string fontSerif: fontFamily
    readonly property string fontMono: fontFamily
    // Four sizes only; widths and weights do the rest
    readonly property real typeDisplay: systemPointSize * 2.2
    readonly property real typeNormal: systemPointSize
    readonly property real typeSmall: systemPointSize * 0.85
    readonly property real typeLarge: systemPointSize * 1.5
    // The four cuts of the face, by role (DESIGN.md, Type). Label.qml applies one with
    // `role: "title"`; a plain Text can take the same through these functions:
    //   font.variableAxes: theme.roleAxes(r)   font.weight: theme.roleWeight(r)
    //   font.pointSize: theme.roleSize(r)      font.features: theme.roleFeatures(r)
    //   display  page titles, the hero tile's title   wdth 125, wght 800, typeDisplay
    //   title    tile plates, section names          wdth 112, wght 650, typeLarge or typeNormal
    //   ui       controls, body                      wdth 100, wght 450, typeNormal
    //   ledger   tables, episode lists, counts, times wdth 88, wght 500, tabular, typeSmall
    readonly property var roles: ({
        display: { wdth: 125, wght: 800 },
        title: { wdth: 112, wght: 650 },
        ui: { wdth: 100, wght: 450 },
        ledger: { wdth: 88, wght: 500 }
    })
    function roleOf(r) { return roles[r] || roles.ui }
    function roleAxes(r) { var c = roleOf(r); return { "wdth": c.wdth, "wght": c.wght } }
    function roleWeight(r) { return roleOf(r).wght }
    function roleSize(r) { return r === "display" ? typeDisplay : r === "title" ? typeLarge : r === "ledger" ? typeSmall : typeNormal }
    function roleFeatures(r) { return r === "ledger" ? { "tnum": 1 } : {} }
    // Exponential smoothing toward a target, bandit shell's rate: x += (target - x) * k
    // with k = 1 - exp(-followRate * dt). follow() is one step of it.
    readonly property real followRate: 14
    function follow(x, target, dt) { return x + (target - x) * (1 - Math.exp(-followRate * dt)) }
    // Reduced motion: the orchestrated moments (the library's opening sweep) are skipped
    readonly property bool reducedMotion: Theme.reducedMotion
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

    // WCAG relative luminance and contrast ratio of two opaque Qt colours
    function luminance(c) {
        function lin(v) { return v <= 0.04045 ? v / 12.92 : Math.pow((v + 0.055) / 1.055, 2.4) }
        return 0.2126 * lin(c.r) + 0.7152 * lin(c.g) + 0.0722 * lin(c.b)
    }
    function contrast(a, b) {
        var la = luminance(a), lb = luminance(b)
        return (Math.max(la, lb) + 0.05) / (Math.min(la, lb) + 0.05)
    }

    // Poster plates, the one bold spend: a poster's tone (Fmt.posterTone) moved toward the
    // ground in steps of `plateStep`. The hover plate is the first step at which the theme's
    // text holds 4.5:1; the resting plate sits one step further toward the ground, so the
    // hover lifts it one step back toward the tone and both stay legible. On a dark theme
    // that darkens the tone toward the grout; on a light one it pales it toward the light
    // ground under dark text.
    readonly property real plateStep: 0.08
    function plateMix(c) {
        c = Qt.color(c)
        for (var i = 0; i * plateStep < 1; i++)
            if (contrast(text, tone(c, ground, i * plateStep)) >= 4.5) return i * plateStep
        return 1
    }
    function plate(c) { c = Qt.color(c); return tone(c, ground, Math.min(1, plateMix(c) + plateStep)) }
    function plateHover(c) { c = Qt.color(c); return tone(c, ground, plateMix(c)) }
}
