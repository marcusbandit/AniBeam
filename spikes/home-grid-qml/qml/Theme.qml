// Every token the prototype styles from, derived live from the knobs and the palettes the
// Rust half read. Instantiated once at the root of Main.qml as `theme`; the ratios are the
// theme ticket's prototype-tunable defaults.
import QtQuick

Item {
    id: root

    // From Rust: { terminal, portal, themes }
    property var palettes: ({})

    // Knobs
    property string mode: "system"           // dark | light | system
    property string colourSource: "system"   // system | theme
    property string themeDark: "anibeam-dark"
    property string themeLight: "anibeam-light"
    property string density: "normal"        // compact | normal | comfortable
    property int posterWidth: 180
    property real cornerSmoothing: 0.6
    property real cornerBase: 14
    property int accentSlot: 4

    // Ratios: mixes of bg toward text (negative: away from text)
    property real stepSunken: 0.03
    property real stepSurface: 0.05
    property real stepRaised: 0.10
    property real stepLine: 0.16
    property real stepLineStrong: 0.26
    property real stepFaint: 0.45
    property real stepDim: 0.70

    // Spacing, radii, type, motion
    readonly property real densityFactor: density === "compact" ? 0.75 : density === "comfortable" ? 1.25 : 1
    function space(n) { return Math.round(4 * densityFactor * n) }
    readonly property real radiusSm: cornerBase * densityFactor
    readonly property real radiusMd: cornerBase * 1.4 * densityFactor
    readonly property real radiusLg: cornerBase * 1.4 * 1.4 * densityFactor
    readonly property real radiusXl: cornerBase * 1.4 * 1.4 * 1.4 * densityFactor
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

    // Mode
    readonly property string terminalMode: palettes.terminal ? (lightness(hexToRgb(palettes.terminal.background)) < 0.5 ? "dark" : "light") : ""
    readonly property string portalMode: palettes.portal && palettes.portal.scheme === "light" ? "light" : "dark"
    readonly property string systemMode: terminalMode !== "" ? terminalMode : portalMode
    readonly property string resolvedMode: mode === "system" ? systemMode : mode
    readonly property bool dark: resolvedMode === "dark"

    // Tokens
    readonly property var tokens: computeTokens(palettes, resolvedMode, colourSource, themeDark, themeLight, accentSlot,
                                                stepSunken, stepSurface, stepRaised, stepLine, stepLineStrong, stepFaint, stepDim)
    readonly property color bg: tokens.bg
    readonly property color surface: tokens.surface
    readonly property color surfaceRaised: tokens.surfaceRaised
    readonly property color surfaceSunken: tokens.surfaceSunken
    readonly property color line: tokens.line
    readonly property color lineStrong: tokens.lineStrong
    readonly property color text: tokens.text
    readonly property color textDim: tokens.textDim
    readonly property color textFaint: tokens.textFaint
    readonly property color accent: tokens.accent
    readonly property color accentText: tokens.accentText
    readonly property color accentSoft: tokens.accentSoft
    readonly property color focusRing: tokens.focus
    readonly property color red: tokens.red
    readonly property color orange: tokens.orange
    readonly property color yellow: tokens.yellow
    readonly property color green: tokens.green
    readonly property color cyan: tokens.cyan
    readonly property color blue: tokens.blue
    readonly property color purple: tokens.purple
    readonly property color brown: tokens.brown
    readonly property color scrim: tokens.scrim
    readonly property string sourceLabel: tokens.sourceLabel

    // Status and fraction colours are fixed mappings onto the hues
    readonly property color statusWatching: accent
    readonly property color statusCompleted: blue
    readonly property color statusPaused: yellow
    readonly property color statusDropped: red
    readonly property color statusPlanning: textFaint
    readonly property color statusRewatching: purple
    readonly property color behind: yellow
    readonly property color caughtUp: accent

    // ---- colour maths (sRGB, good enough for the prototype; the shell does HCT)
    function hexToRgb(h) {
        h = String(h).replace("#", "")
        return { r: parseInt(h.substr(0, 2), 16) / 255, g: parseInt(h.substr(2, 2), 16) / 255, b: parseInt(h.substr(4, 2), 16) / 255 }
    }
    function q(c, a) { return Qt.rgba(c.r, c.g, c.b, a === undefined ? 1 : a) }
    function mix(a, b, t) { return { r: a.r + (b.r - a.r) * t, g: a.g + (b.g - a.g) * t, b: a.b + (b.b - a.b) * t } }
    function lightness(c) { return 0.2126 * c.r + 0.7152 * c.g + 0.0722 * c.b }
    function lin(v) { return v <= 0.03928 ? v / 12.92 : Math.pow((v + 0.055) / 1.055, 2.4) }
    function luminance(c) { return 0.2126 * lin(c.r) + 0.7152 * lin(c.g) + 0.0722 * lin(c.b) }
    function contrast(a, b) {
        var la = luminance(a), lb = luminance(b)
        return (Math.max(la, lb) + 0.05) / (Math.min(la, lb) + 0.05)
    }
    function rgbToHsl(c) {
        var max = Math.max(c.r, c.g, c.b), min = Math.min(c.r, c.g, c.b)
        var h = 0, s = 0, l = (max + min) / 2
        if (max !== min) {
            var d = max - min
            s = l > 0.5 ? d / (2 - max - min) : d / (max + min)
            if (max === c.r) h = (c.g - c.b) / d + (c.g < c.b ? 6 : 0)
            else if (max === c.g) h = (c.b - c.r) / d + 2
            else h = (c.r - c.g) / d + 4
            h /= 6
        }
        return { h: h, s: s, l: l }
    }
    function hslToRgb(h, s, l) {
        function hue(p, q, t) {
            if (t < 0) t += 1
            if (t > 1) t -= 1
            if (t < 1 / 6) return p + (q - p) * 6 * t
            if (t < 1 / 2) return q
            if (t < 2 / 3) return p + (q - p) * (2 / 3 - t) * 6
            return p
        }
        if (s === 0) return { r: l, g: l, b: l }
        var qq = l < 0.5 ? l * (1 + s) : l + s - l * s
        var p = 2 * l - qq
        return { r: hue(p, qq, h + 1 / 3), g: hue(p, qq, h), b: hue(p, qq, h - 1 / 3) }
    }
    // Hue halfway from a to b along the short way round, at their mean saturation and lightness
    function hueBetween(a, b) {
        var A = rgbToHsl(a), B = rgbToHsl(b)
        var dh = B.h - A.h
        if (dh > 0.5) dh -= 1
        if (dh < -0.5) dh += 1
        var h = A.h + dh / 2
        if (h < 0) h += 1
        if (h > 1) h -= 1
        return hslToRgb(h, (A.s + B.s) / 2, (A.l + B.l) / 2)
    }
    function browned(c) { var H = rgbToHsl(c); return hslToRgb(H.h, H.s * 0.55, H.l * 0.72) }
    // Same hue and saturation, lightness capped at 0.42 on a light ground or floored at 0.62 on a dark one
    function retone(c, m) {
        var H = rgbToHsl(c)
        var l = m === "light" ? Math.min(H.l, 0.42) : Math.max(H.l, 0.62)
        return hslToRgb(H.h, H.s, l)
    }
    function sameHex(a, b) { return a.toLowerCase() === b.toLowerCase() }

    function findTheme(slug) {
        var list = palettes.themes || []
        for (var i = 0; i < list.length; i++) if (list[i].slug === slug) return list[i]
        return list.length ? list[0] : null
    }

    function computeTokens(P, m, src, slugDark, slugLight, slot, sSunken, sSurface, sRaised, sLine, sLineStrong, sFaint, sDim) {
        P = P || {}
        var t = {}
        var away = m === "dark" ? { r: 0, g: 0, b: 0 } : { r: 1, g: 1, b: 1 }
        var contrastMul = P.portal && P.portal.contrast ? 1.5 : 1
        var bg, fg
        if (src === "system" && P.terminal) {
            var term = P.terminal
            bg = hexToRgb(term.background)
            fg = hexToRgb(term.foreground)
            var native = lightness(bg) < 0.5 ? "dark" : "light"
            var C = term.colors.map(hexToRgb)
            if (native !== m) {
                // Forced mode against the terminal: keep its hues and accent, derive a neutral
                // ground and text for the other mode, tinted a little toward the accent, and
                // re-tone the hues so they hold contrast on the new ground (a dark terminal's
                // pastels vanish on white).
                var tint = C[Math.max(1, Math.min(6, slot))]
                bg = mix(m === "dark" ? hexToRgb("#101216") : hexToRgb("#f6f7fa"), tint, 0.03)
                fg = m === "dark" ? hexToRgb("#e4e7ee") : hexToRgb("#1b1e26")
                C = C.map(function(c, i) { return i === 0 || i === 7 || i === 8 || i === 15 ? c : retone(c, m) })
            }
            t.surface = mix(bg, fg, sSurface); t.surfaceRaised = mix(bg, fg, sRaised)
            t.line = mix(bg, fg, sLine * contrastMul); t.lineStrong = mix(bg, fg, sLineStrong * contrastMul)
            t.textFaint = mix(bg, fg, sFaint * contrastMul); t.textDim = mix(bg, fg, sDim)
            t.red = C[1]; t.green = C[2]; t.yellow = C[3]; t.blue = C[4]; t.purple = C[5]; t.cyan = C[6]
            t.orange = hueBetween(t.red, t.yellow); t.brown = browned(t.orange)
            var s = Math.max(1, Math.min(6, slot))
            t.accent = C[s]
            t.focus = sameHex(term.colors[s], term.colors[s + 8]) ? t.accent : C[s + 8]
            t.sourceLabel = "terminal " + (term.source || "") + (native !== m ? " (forced " + m + ")" : "")
        } else if (src === "system") {
            var seed = P.portal && P.portal.accent ? hexToRgb(P.portal.accent) : hexToRgb("#46e0c4")
            bg = mix(m === "dark" ? hexToRgb("#101216") : hexToRgb("#f6f7fa"), seed, 0.03)
            fg = m === "dark" ? hexToRgb("#e4e7ee") : hexToRgb("#1b1e26")
            t.surface = mix(bg, fg, sSurface); t.surfaceRaised = mix(bg, fg, sRaised)
            t.line = mix(bg, fg, sLine * contrastMul); t.lineStrong = mix(bg, fg, sLineStrong * contrastMul)
            t.textFaint = mix(bg, fg, sFaint * contrastMul); t.textDim = mix(bg, fg, sDim)
            var fb = findTheme(m === "dark" ? "anibeam-dark" : "anibeam-light")
            var fp = fb ? fb.palette : {}
            t.red = hexToRgb(fp.base08 || "#f0718a"); t.orange = hexToRgb(fp.base09 || "#f0a772"); t.yellow = hexToRgb(fp.base0A || "#e8bf78")
            t.green = hexToRgb(fp.base0B || "#8adfb5"); t.cyan = hexToRgb(fp.base0C || "#46e0c4"); t.blue = hexToRgb(fp.base0D || "#7cbcf5")
            t.purple = hexToRgb(fp.base0E || "#c0abf0"); t.brown = hexToRgb(fp.base0F || "#b08968")
            t.accent = seed; t.focus = seed
            t.sourceLabel = "portal, derived (" + ((P.portal && P.portal.scheme) || "unset") + ")"
        } else {
            var th = findTheme(m === "dark" ? slugDark : slugLight)
            var p = th ? th.palette : {}
            function pc(k, d) { return hexToRgb(p[k] || d) }
            bg = pc("base00", "#101216"); fg = pc("base05", "#e4e7ee")
            t.surface = pc("base01", "#15181e"); t.surfaceRaised = pc("base02", "#1d2129")
            t.line = pc("base02", "#1d2129"); t.lineStrong = pc("base03", "#2c3140")
            t.textFaint = pc("base03", "#2c3140"); t.textDim = pc("base04", "#6b7590")
            t.red = pc("base08", "#f0718a"); t.orange = pc("base09", "#f0a772"); t.yellow = pc("base0A", "#e8bf78"); t.green = pc("base0B", "#8adfb5")
            t.cyan = pc("base0C", "#46e0c4"); t.blue = pc("base0D", "#7cbcf5"); t.purple = pc("base0E", "#c0abf0"); t.brown = pc("base0F", "#b08968")
            t.accent = pc((th && th.accent) || "base0D", "#7cbcf5"); t.focus = t.accent
            t.sourceLabel = "theme " + (th ? th.name : "none")
        }
        t.bg = bg; t.text = fg
        t.surfaceSunken = mix(bg, away, sSunken)
        t.accentSoft = mix(bg, t.accent, 0.2)
        t.accentText = contrast(t.accent, bg) > contrast(t.accent, fg) ? bg : fg
        var out = {}
        for (var k in t) out[k] = (k === "sourceLabel") ? t[k] : q(t[k])
        out.scrim = q(bg, 0.8)
        return out
    }
}
