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
    readonly property color line: glassEdge
    readonly property color lineStrong: glassEdgeStrong
    readonly property color text: token("text")
    readonly property color textDim: token("text_dim")
    readonly property color textFaint: token("text_faint")
    readonly property color accent: token("accent")
    readonly property color accentText: token("accent_text")
    // The soft fills are glass tinted by the hue, translucent like every other pane
    readonly property color accentSoft: Qt.rgba(accent.r, accent.g, accent.b, dark ? 0.16 : 0.14)
    // The edge of a pane filled with the soft accent (the rail's active page, a switch's
    // thumb, a list's current row)
    readonly property color accentEdge: Qt.rgba(accent.r, accent.g, accent.b, 0.5)
    readonly property color redSoft: Qt.rgba(red.r, red.g, red.b, dark ? 0.16 : 0.14)
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

    // ---- The glass mosaic (DESIGN.md, Tokens). Every surface is a pane of glass over the
    // ground: the ink (white on dark, the text colour on light) at a low alpha, a 1 px edge
    // and, on the panes that stand up from the page, a 1 px top highlight, the glassy cue.
    // No live blur (the frosted glass was removed for its cost); the glass is translucency
    // and hairlines. The theme's own opaque slots (surface, raised) stay for the code that
    // needs a solid colour.
    readonly property color ink: dark ? "#ffffff" : text
    function glassOf(a) { return Qt.rgba(ink.r, ink.g, ink.b, a) }
    readonly property color glass: glassOf(dark ? 0.035 : 0.045)
    readonly property color glassRaised: glassOf(dark ? 0.06 : 0.075)
    readonly property color glassPressed: glassOf(dark ? 0.09 : 0.11)
    readonly property color glassEdge: glassOf(dark ? 0.08 : 0.12)
    readonly property color glassEdgeStrong: glassOf(dark ? 0.16 : 0.22)
    readonly property color glassHighlight: dark ? Qt.rgba(1, 1, 1, 0.22) : Qt.rgba(1, 1, 1, 0.85)
    // The sheen under the highlight (Corner.qml): light falling on the glass from above,
    // strongest at the top edge and gone by sheenReach of the pane's height, never deeper
    // than sheenMax px.
    readonly property color sheen: dark ? Qt.rgba(1, 1, 1, 0.055) : Qt.rgba(1, 1, 1, 0.45)
    readonly property real sheenReach: 0.6
    readonly property real sheenMax: 120
    // A floating pane (menus, tips, sheets, popups, the drawer) sits over arbitrary content,
    // so it is opaque: the ground lifted one glass step (any alpha let text ghost through).
    function lifted(t, a) { var c = tone(bg, ink, t); return Qt.rgba(c.r, c.g, c.b, a) }
    readonly property color glassFloat: lifted(dark ? 0.07 : 0.06, 1)
    readonly property color glassFloatHover: lifted(dark ? 0.12 : 0.11, 1)
    // The player's islands: glass over video, dark in both modes so the picture never
    // washes the controls out
    readonly property color glassVideo: Qt.rgba(0.03, 0.035, 0.045, 0.55)
    // The poster scrim, the one gradient in the shell: near-black at the poster's foot,
    // clear by `scrimReach` of its height
    readonly property color posterScrim: Qt.rgba(8 / 255, 9 / 255, 11 / 255, 0.92)
    readonly property real scrimReach: 0.55
    // Text set on a poster (always over the dark scrim, in both modes)
    readonly property color onPoster: "#eef2f5"
    readonly property color onPosterDim: "#aab2bb"
    // The accent where it is text on the scrim: itself on dark, lifted toward white on light
    // (the light theme's accent is a deep blue that would sink into the near-black scrim)
    readonly property color accentOnPoster: dark ? accent : tone(accent, Qt.rgba(1, 1, 1, 1), 0.5)

    // The mosaic's names for the ramp: the ground is the grout every tile is set in, a tile
    // is a pane of glass on it, a raised tile is hovered or selected, a sunken tile takes
    // input (fields), and the line is the glass edge.
    readonly property color ground: bg
    readonly property color tile: glass
    readonly property color raised: glassRaised
    readonly property color tileSunken: dark ? Qt.rgba(0, 0, 0, 0.28) : glassOf(0.07)
    // One hover step up from a resting pane, and from a raised one (a menu row on a menu)
    readonly property color tileHover: glassRaised
    readonly property color raisedHover: glassPressed
    readonly property color raisedFloat: glassFloatHover   // a floating layer over a raised surface
    readonly property string sourceLabel: Theme.sourceLabel

    // Status and fraction colours: fixed mappings onto the hues
    readonly property color statusWatching: accent
    // Quiet: the accent marks the live states (watching, rewatching), the rest are greys;
    // the gilt and the alarm hues are kept for warnings and errors.
    readonly property color statusCompleted: text
    readonly property color statusPaused: textDim
    readonly property color statusDropped: textFaint
    readonly property color statusPlanning: textFaint
    readonly property color statusRewatching: accent
    readonly property color behind: Qt.rgba(ink.r, ink.g, ink.b, dark ? 0.32 : 0.3)
    readonly property color caughtUp: accent
    // A hue by the name Theme.formatHue or Theme.statusHue returns
    function hue(name) { return token(name) }

    // Spacing, radii, type, motion
    readonly property real densityFactor: Theme.densityFactor
    function space(n) { return Math.round(4 * densityFactor * n) }
    // Rounded panes, plain circular corners (no squircle smoothing): the tile radius is the
    // one every pane takes, the smaller steps are for what nests inside one, and a control
    // asks for height / 2, a pill. All of it scales with density.
    readonly property real cornerSmoothing: 0
    readonly property real radiusTile: space(3.5)
    readonly property real cornerBase: radiusTile
    readonly property real radiusSm: space(1.5)
    readonly property real radiusMd: space(2.5)
    readonly property real radiusLg: space(4)
    readonly property real radiusXl: space(5)
    // The mosaic's geometry: one joint between every pair of panes, wide enough for the
    // panes to read as set apart, and the page's own margin is the same joint.
    readonly property real grout: space(3)
    // The radius of a shape nested `padding` inside a tile, so the two curves stay
    // concentric; never below the small step so a nested row still reads as rounded.
    function radiusInner(padding) { return Math.max(radiusSm, radiusTile - padding) }
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
    // Four sizes only; widths and weights do the rest
    readonly property real typeDisplay: systemPointSize * 2.8
    readonly property real typeNormal: systemPointSize
    readonly property real typeSmall: systemPointSize * 0.85
    readonly property real typeLarge: systemPointSize * 1.5
    // The four cuts of the face, by role (DESIGN.md, Type). Label.qml applies one with
    // `role: "title"`; a plain Text can take the same through these functions:
    //   font.variableAxes: theme.roleAxes(r)   font.weight: theme.roleWeight(r)
    //   font.pointSize: theme.roleSize(r)      font.features: theme.roleFeatures(r)
    //   display  page titles, the hero tile's title   wdth 68, wght 330, typeDisplay (tall, condensed, light)
    //   title    titles on posters, section names     wdth 86, wght 600, typeLarge or typeNormal
    //   ui       controls, body                      wdth 100, wght 450, typeNormal
    //   ledger   tables, episode lists, counts, times wdth 88, wght 500, tabular, typeSmall
    readonly property var roles: ({
        display: { wdth: 68, wght: 330 },
        title: { wdth: 86, wght: 600 },
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

    // Poster plates (retired by the glass mosaic: no surface takes a poster's tone any more;
    // kept for the token page and any caller still asking). A poster's tone moved toward the
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
