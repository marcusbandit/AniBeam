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
    // The figures: B612, the face Airbus drew for reading numbers on a cockpit screen at a
    // glance (OFL). Only numbers take it, through the `figure` role: speeds, sizes, counts,
    // seeders, times, percentages. Words stay in Archivo.
    FontLoader { id: figureFace; source: "qrc:/qt/qml/com/marcusrosado/AniBeam/assets/fonts/B612-Regular.ttf" }
    FontLoader { source: "qrc:/qt/qml/com/marcusrosado/AniBeam/assets/fonts/B612-Bold.ttf" }
    readonly property string fontFigure: figureFace.name !== "" ? figureFace.name : fontFamily

    // ---- Episode cubes (EpisodeCubes.qml). One cube per episode, its colour its state. The
    // codes come from the core (library::cards::cubes), plus the client states the Downloads
    // page lays over them (t, e). A new state is one entry here and one code there; `order`
    // is the legend's order. The hues are the theme's own, so every theme keeps a palette
    // that holds together: green and cyan sit side by side because both mean watched.
    readonly property var cubeStates: ({
        w: { label: "Watched, tracked online", fill: green, edge: "transparent", order: 0 },
        l: { label: "Watched here, not tracked yet", fill: cyan, edge: "transparent", order: 1 },
        d: { label: "On disk", fill: dark ? "#ffffff" : text, edge: "transparent", order: 2 },
        t: { label: "Downloading", fill: purple, edge: "transparent", order: 3 },
        m: { label: "Missing", fill: yellow, edge: "transparent", order: 4 },
        e: { label: "Error", fill: red, edge: "transparent", order: 5 },
        g: { label: "Not downloaded", fill: glassOf(dark ? 0.16 : 0.18), edge: "transparent", order: 6 },
        f: { label: "Not aired yet", fill: "transparent", edge: glassOf(dark ? 0.2 : 0.28), order: 7 },
        // Downloads-only marks on a provider's row: a release there, and one picked
        a: { label: "Available", fill: glassOf(dark ? 0.1 : 0.12), edge: glassOf(dark ? 0.32 : 0.4), order: 8 },
        p: { label: "Picked to get", fill: blue, edge: "transparent", order: 9 },
        o: { label: "Already on disk", fill: glassOf(dark ? 0.05 : 0.07), edge: "transparent", order: 10 },
        n: { label: "No release", fill: "transparent", edge: "transparent", order: 11 }
    })
    function cubeState(code) { return cubeStates[code] || cubeStates.g }
    // A fraction as a row of step cubes, for progress that is not episodes (bytes, a job):
    // `steps` cubes, the done share in `on`, the rest in `off`. Rounds down, so a cube
    // lights only once its whole step is done, and 100% is every cube.
    function cubeSteps(fraction, steps, on, off) {
        var n = Math.max(1, steps || 20)
        var k = fraction >= 1 ? n : Math.floor(Math.max(0, fraction) * n)
        return (on || "d").repeat(k) + (off || "g").repeat(n - k)
    }

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
    // Flat (2026-10-13): every pane is one solid step up from the ground, no top highlight,
    // no sheen. The names stay so every surface follows the one change.
    // Three roles: the ground (bg), the surface (the theme's own secondary colour: the
    // sidebar, cards' plates, the toolbar's controls) and the accent, used sparingly.
    readonly property color glass: surface
    readonly property color glassRaised: surfaceRaised
    readonly property color glassPressed: surfacePressed
    readonly property color glassEdge: glassOf(dark ? 0.06 : 0.1)
    readonly property color glassEdgeStrong: glassOf(dark ? 0.14 : 0.2)
    readonly property color glassHighlight: "transparent"
    // The sheen under the highlight (Corner.qml): light falling on the glass from above,
    // strongest at the top edge and gone by sheenReach of the pane's height, never deeper
    // than sheenMax px.
    readonly property color sheen: "transparent"
    readonly property real sheenReach: 0.6
    readonly property real sheenMax: 120
    // A floating pane (menus, tips, sheets, popups, the drawer) sits over arbitrary content,
    // so it is opaque: the ground lifted one glass step (any alpha let text ghost through).
    function lifted(t, a) { var c = tone(bg, ink, t); return Qt.rgba(c.r, c.g, c.b, a) }
    readonly property color glassFloat: lifted(dark ? 0.07 : 0.06, 1)
    readonly property color glassFloatHover: lifted(dark ? 0.12 : 0.11, 1)
    // The hovered row of a list on a floating pane (a dropdown's option, a menu row): a
    // clear step up from the float itself. The flat pass made the raised surface the float's
    // own colour, so a row set in it vanished and hovering looked broken.
    readonly property color floatHover: lifted(dark ? 0.16 : 0.14, 1)
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
    readonly property real radiusTile: space(4.5)
    readonly property real cornerBase: radiusTile
    readonly property real radiusSm: space(1.5)
    readonly property real radiusMd: space(2.5)
    readonly property real radiusLg: space(5)
    readonly property real radiusXl: space(7)
    // The mosaic's geometry: one joint between every pair of panes, wide enough for the
    // panes to read as set apart, and the page's own margin is the same joint.
    readonly property real grout: space(3)
    // The widest the page content grows: past it the pages sit in a centred column, so an
    // ultrawide window shows a readable page with ground either side rather than one
    // endless wall.
    readonly property real contentMax: space(500)
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
    // A page's own title (Library, Feed, Downloads) and a section's heading: bigger than the
    // display cut's default, the first thing read on a page
    readonly property real typePage: systemPointSize * 4
    readonly property real typeHeading: systemPointSize * 2.2
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
        ledger: { wdth: 88, wght: 500 },
        figure: { wdth: 100, wght: 400 }
    })
    function roleOf(r) { return roles[r] || roles.ui }
    function roleAxes(r) { var c = roleOf(r); return { "wdth": c.wdth, "wght": c.wght } }
    function roleWeight(r) { return roleOf(r).wght }
    function roleSize(r) { return r === "display" ? typeDisplay : r === "title" ? typeLarge : r === "ledger" ? typeSmall : r === "figure" ? typeSmall : typeNormal }
    function roleFeatures(r) { return r === "ledger" || r === "figure" ? { "tnum": 1 } : {} }
    function roleFamily(r) { return r === "figure" ? fontFigure : fontFamily }
    // Exponential smoothing toward a target, bandit shell's rate: x += (target - x) * k
    // with k = 1 - exp(-followRate * dt). follow() is one step of it.
    readonly property real followRate: 14
    // Reduced motion lands every follower at once: hovers, thumbs and lifts jump, nothing glides
    function follow(x, target, dt) { return reducedMotion ? target : x + (target - x) * (1 - Math.exp(-followRate * dt)) }
    // Reduced motion: the orchestrated moments (the library's opening sweep) are skipped, and
    // every transition timed by these two is instant. Scrolling keeps its ease (WheelStep).
    readonly property bool reducedMotion: Theme.reducedMotion
    readonly property int motionFast: reducedMotion ? 0 : 120
    readonly property int motionNormal: reducedMotion ? 0 : 200
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

    // A poster band (the series page's head, Home's hero): the poster's tone over the dark
    // base, at `mix` or less, so the band's white ink holds 7:1 whatever the poster; a bright
    // poster gives a deeper band rather than a washed one
    readonly property color bandDark: Qt.rgba(0.05, 0.055, 0.07, 1)
    function bandBase(t, mix) {
        var m = mix
        while (m > 0 && contrast(Qt.color("#f4f6f8"), tone(bandDark, t, m)) < 7) m -= 0.05
        return tone(bandDark, t, Math.max(0, m))
    }
    // The band's accent: the tone lifted toward white, at least 0.45 and further until it
    // stands 7:1 off the band, so a highlight never sinks into its own hue
    function bandAccent(t, base) {
        var white = Qt.rgba(1, 1, 1, 1)
        var m = 0.45
        while (m < 1 && contrast(tone(t, white, m), base) < 7) m += 0.05
        return tone(t, white, Math.min(1, m))
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
