# Where a Linux app reads the user's system colours

Research for ticket #7, part of the native line map (#2). Written 2026-09-03 against xdg-desktop-portal 1.22.1, xdg-desktop-portal-gtk 1.15.3, xdg-desktop-portal-hyprland 1.4.1, Hyprland 0.56.1, Qt 6.11.1, qt6ct 0.11, caelestia-shell 2.2.0 and caelestia-cli 1.1.2 on the owner's Arch desktop. Vocabulary follows CONTEXT.md: the core is the Rust crate, the shell is the Qt app, and a colour change reaches the shell as an event.

> **Erratum, 2026-09-04.** The owner does not run caelestia. Their desktop shell is bandit shell (`~/banditshell`, their own Quickshell shell), on its `slate` theme, and it writes no colour file. The caelestia packages, `scheme.json`, gtk.css and qt6ct palette this document reads are leftovers that nothing launches. Section 4 and every conclusion built on scheme.json or matugen are void. The theme model ticket (#14) settled the source instead: the terminal palette, read from the terminal's own config. Sections 1 to 3 and 5 stand.

## Summary

1. The portal is the only cross-desktop standard, and it carries three keys under `org.freedesktop.appearance`: `color-scheme` (0 no preference, 1 dark, 2 light), `accent-color` (three sRGB doubles in 0..1) and `contrast` (0 or 1). No standard anywhere exposes a background, surface or text colour. The core reads these itself over D-Bus with `ReadOne` and listens to `SettingChanged`; it does not go through Qt for them.
2. On Hyprland the Settings interface is not served by xdg-desktop-portal-hyprland, which implements only ScreenCast, Screenshot, GlobalShortcuts and InputCapture. It falls through to xdg-desktop-portal-gtk, which reads GNOME gsettings. That backend maps `color-scheme` and `contrast` and will never map `accent-color`; its maintainer closed the request saying accent colours are desktop specific.
3. The owner's desktop today returns `color-scheme` 1 (dark) and `contrast` 0, and `ReadOne` on `accent-color` fails with "Requested setting not found". gsettings does hold `accent-color 'blue'`, but nothing on this desktop turns it into a portal value. Treat a ReadOne error as "unset", because that is the normal case here.
4. Everything past scheme and accent is derived. The inputs are the scheme, one accent (portal if present, else the user's pick, else the brand colour) and the contrast flag. Material's HCT dynamic colour turns one accent into every role and is what both caelestia and matugen already use, so a Rust port of it, or a plain tone ladder, is the derivation to build.
5. Qt 6.11 on Hyprland is not a reliable source of the scheme. With no `QT_QPA_PLATFORMTHEME`, `QStyleHints::colorScheme` is Unknown and the palette is light Fusion grey on a dark desktop. With `qt6ct`, which is what the owner runs, the scheme is still Unknown but the palette is the qt6ct file. Only `xdgdesktopportal`, `gtk3` and `kde` report Dark.
6. `QGuiApplication::palette()` is still worth reading as a fourth source: under qt6ct, gtk3 or kde it holds the user's real window, text and highlight colours, and Qt's own themes infer the scheme from text versus window lightness when nothing better exists. `Qt::ContrastPreference` exists since 6.10 on `QPlatformTheme` only; `QStyleHints` has no contrast property.
7. The owner's shell is caelestia. Its CLI generates Material schemes with python-materialyoucolor and writes `~/.local/state/caelestia/scheme.json`: name, flavour, mode, variant and a flat map of Material roles, Catppuccin-style aliases and sixteen terminal colours as bare hex. The shell watches that file itself, so the app can watch it too and push the change as an event.
8. matugen 4.1.0 is installed but idle: no config, no template, nothing on the box calls it. Its `--json` output uses the same Material role names as scheme.json in snake_case, so one reader covers both if a user ever points the app at matugen output.
9. Catppuccin, Gruvbox and Tokyo Night each keep their canonical palette in a different shape (JSON, a Vim script, Lua tables). The formats all three actually ship in paste-able form are base16 YAML (tinted-theming carries every flavour of all three) and kitty `.conf`. Import base16 YAML first, since its sixteen slots have defined roles; accept kitty conf as the second format.
10. Source order the findings support: a pasted user theme, then the shell's scheme.json, then Qt's palette when a theme plugin is active, then the portal-derived palette, then the built-in one.

## 1. The portal Settings interface

### What the spec defines

The interface is `org.freedesktop.portal.Settings` on the bus name `org.freedesktop.portal.Desktop` at `/org/freedesktop/portal/desktop`, described as "read-only access to a small number of standardized host settings required for toolkits similar to XSettings" (https://flatpak.github.io/xdg-desktop-portal/docs/doc-org.freedesktop.portal.Settings.html). Methods: `ReadAll (IN namespaces as, OUT value a{sa{sv}})`, where an empty array matches all and globbing works for trailing sections only; `Read (IN namespace s, IN key s, OUT value v)`, deprecated because it "is actually returned inside two layers of variant"; and `ReadOne (IN namespace s, IN key s, OUT value v)`, which "Returns an error on any unknown namespace or key" and was added in interface version 2. The signal is `SettingChanged (namespace s, key s, value v)`. The `version` property is 2 (same page).

The three keys under `org.freedesktop.appearance`, quoted from the same page:

- `color-scheme` (`u`): "Indicates the system's preferred color scheme. Supported values are: 0: No preference, 1: Prefer dark appearance, 2: Prefer light appearance." Unknown values are treated as 0.
- `accent-color` (`(ddd)`): "Indicates the system's preferred accent color as a tuple of RGB values in the sRGB color space, in the range [0,1]. Out-of-range RGB values should be treated as an unset accent color."
- `contrast` (`u`): "Indicates the system's preferred contrast level. Supported values are: 0: No preference (normal contrast), 1: Higher contrast."

Qt's own reader converts these as 0 to `Qt::ColorScheme::Unknown`, 1 to Dark, 2 to Light (https://raw.githubusercontent.com/qt/qtbase/6.11/src/gui/platform/unix/qdbussettings.cpp).

### When each key arrived

From the history of `data/org.freedesktop.portal.Settings.xml` (`gh api 'repos/flatpak/xdg-desktop-portal/commits?path=data/org.freedesktop.portal.Settings.xml'`): `color-scheme` on 2021-09-19 (d7a304a), `accent-color` on 2023-03-21 (96b883c), the ReadOne minimum version on 2023-08-29 (33a0854), `contrast` on 2024-01-25 (7605cb6). NEWS.md places `color-scheme` in 1.12.0 (2021-12-21) and both ReadOne and the accent-color documentation in 1.17.1 (2023-08-27) (https://raw.githubusercontent.com/flatpak/xdg-desktop-portal/main/NEWS.md). NEWS.md never mentions `contrast`; the tag check shows it absent from the 1.18.4 XML and present in 1.19.0 (2024-10-09) (`gh api 'repos/flatpak/xdg-desktop-portal/contents/data/org.freedesktop.portal.Settings.xml?ref=1.19.0' -H 'Accept: application/vnd.github.raw' | rg -c contrast` prints 4; the same for 1.18.4 prints nothing). The installed frontend is 1.22.1-2 (`pacman -Q xdg-desktop-portal`), so all three keys are in the spec the owner runs.

### Which backend answers on Hyprland

xdg-desktop-portal picks a backend per interface from `portals.conf`: each key holds "a semi-colon separated list of portal backend implementation, to be searched for an implementation of the requested interface, in the same order as specified", with `none` and `*` as special values, and it looks for `{desktop}-portals.conf` from `XDG_CURRENT_DESKTOP` before the generic file (https://flatpak.github.io/xdg-desktop-portal/docs/portals.conf.html).

The Hyprland backend declares what it serves in its portal file, upstream and installed alike:

```
$ cat /usr/share/xdg-desktop-portal/portals/hyprland.portal
[portal]
DBusName=org.freedesktop.impl.portal.desktop.hyprland
Interfaces=org.freedesktop.impl.portal.Screenshot;org.freedesktop.impl.portal.ScreenCast;org.freedesktop.impl.portal.GlobalShortcuts;org.freedesktop.impl.portal.InputCapture;
UseIn=wlroots;Hyprland;sway;Wayfire;river;
```

(same content at https://raw.githubusercontent.com/hyprwm/xdg-desktop-portal-hyprland/master/hyprland.portal). No Settings. The wiki says XDPH "allows for screen sharing, global shortcuts, etc." and "XDPH doesn't implement a file picker. For that, it is recommended to install `xdg-desktop-portal-gtk` alongside XDPH" (https://raw.githubusercontent.com/hyprwm/hyprland-wiki/main/content/hypr-ecosystem/user/xdg-desktop-portal-hyprland.md; the wiki site itself returns 403 to fetch tools). The GTK backend's file lists `org.freedesktop.impl.portal.Settings` among its interfaces (`cat /usr/share/xdg-desktop-portal/portals/gtk.portal`).

The preference files on the box put gtk second:

```
$ cat /usr/share/xdg-desktop-portal/hyprland-portals.conf
[preferred]
default=hyprland;gtk
$ cat ~/.config/xdg-desktop-portal/portals.conf
[preferred]
default=hyprland;gtk
org.freedesktop.impl.portal.RemoteDesktop=hyprland
org.freedesktop.impl.portal.ScreenCast=hyprland
```

Live introspection confirms which process holds the interface:

```
$ busctl --user list | rg portal
org.freedesktop.impl.portal.desktop.gtk        4525 xdg-desktop-por bandit :1.47 user@1000.service
org.freedesktop.impl.portal.desktop.hyprland   3583 xdg-desktop-por bandit :1.18 user@1000.service
org.freedesktop.impl.portal.desktop.kde           - (activatable)
org.freedesktop.impl.portal.desktop.wlr           - (activatable)
org.freedesktop.portal.Desktop                 3520 xdg-desktop-por bandit :1.12 user@1000.service
$ busctl --user introspect org.freedesktop.impl.portal.desktop.hyprland /org/freedesktop/portal/desktop | rg '^org.freedesktop.impl'
org.freedesktop.impl.portal.GlobalShortcuts
org.freedesktop.impl.portal.InputCapture
org.freedesktop.impl.portal.ScreenCast
org.freedesktop.impl.portal.Screenshot
$ busctl --user introspect org.freedesktop.impl.portal.desktop.gtk /org/freedesktop/portal/desktop | rg Settings
org.freedesktop.impl.portal.Settings
```

The kde and wlr backends are installed (`pacman -Qs portal`) but only activatable, and the config never names them, so they do not take part.

### What the GTK backend computes

xdg-desktop-portal-gtk's `settings.c` reads `color-scheme` from the `org.gnome.desktop.interface` schema's `color-scheme` enum, returned as uint32, and `contrast` from `org.gnome.desktop.a11y.interface` `high-contrast` as 1 or 0. There is no `accent-color` anywhere in the file, and an unknown key returns the error "Requested setting not found" (https://raw.githubusercontent.com/flatpak/xdg-desktop-portal-gtk/main/src/settings.c). Its NEWS.md dates the colour scheme key to 1.12.0 and "Implement the contrast setting" to 1.15.2 (2025-01-09); the latest release is 1.15.3 (2025-03-21), the version installed here (https://raw.githubusercontent.com/flatpak/xdg-desktop-portal-gtk/main/NEWS.md, `pacman -Q xdg-desktop-portal-gtk`).

Accent colour was asked for in issue #498 and closed on 2025-03-13 with this from the maintainer: "The GTK portal backend is desktop-neutral, because it's used as a fallback implementation. Accent colors are very much desktop-specific, so we cannot really provide an implementation for them. This is something that every desktop environment will have to deal with in their own portal backend." (https://github.com/flatpak/xdg-desktop-portal-gtk/issues/498). So on a Hyprland desktop with the gtk fallback, `accent-color` stays absent until xdg-desktop-portal-hyprland grows a Settings implementation, and nothing in its repo suggests one.

### What the owner's desktop returns today

```
$ busctl --user get-property org.freedesktop.portal.Desktop /org/freedesktop/portal/desktop org.freedesktop.portal.Settings version
u 2
$ busctl --user call org.freedesktop.portal.Desktop /org/freedesktop/portal/desktop org.freedesktop.portal.Settings ReadOne ss org.freedesktop.appearance color-scheme
v u 1
$ busctl --user call org.freedesktop.portal.Desktop /org/freedesktop/portal/desktop org.freedesktop.portal.Settings ReadOne ss org.freedesktop.appearance accent-color
Call failed: Requested setting not found
$ busctl --user call org.freedesktop.portal.Desktop /org/freedesktop/portal/desktop org.freedesktop.portal.Settings ReadOne ss org.freedesktop.appearance contrast
v u 0
$ busctl --user call org.freedesktop.portal.Desktop /org/freedesktop/portal/desktop org.freedesktop.portal.Settings ReadAll as 1 org.freedesktop.appearance
a{sa{sv}} 1 "org.freedesktop.appearance" 2 "contrast" u 0 "color-scheme" u 1
```

The gsettings behind those values: `gsettings get org.gnome.desktop.interface color-scheme` prints `'prefer-dark'`, `accent-color` prints `'blue'`, `gtk-theme` prints `'adw-gtk3-dark'`, and `org.gnome.desktop.a11y.interface high-contrast` prints `false`. The dark value is written by caelestia: its GTK step runs `dconf write /org/gnome/desktop/interface/color-scheme 'prefer-{mode}'` every time a scheme is applied (`/usr/lib/python3.14/site-packages/caelestia/utils/theme.py` line 317). So the portal's scheme on this desktop follows the shell's light or dark mode, which is the right coupling, and the accent the shell computes never reaches the portal.

### How desktops that do fill accent-color compute it

Two data points on what an accent means elsewhere, useful for the derivation. xdg-desktop-portal-gnome maps the gsettings enum through libadwaita's `adw_accent_color_to_rgba` and returns `(ddd)` from the resulting RGBA (https://gitlab.gnome.org/GNOME/xdg-desktop-portal-gnome/-/raw/main/src/settings.c); libadwaita's table is blue #3584e4, teal #2190a4, green #3a944a, yellow #c88800, orange #ed5b00, red #e62d42, pink #d56199, purple #9141ac, slate #6f8396 (https://gitlab.gnome.org/GNOME/libadwaita/-/raw/main/src/adw-accent-color.c). xdg-desktop-portal-kde returns `qGuiApp->palette().highlight().color()` as the accent and decides the scheme from `qGray(palette.window().color().rgb())` below 192 meaning dark; the fetched source shows no `contrast` key under the appearance namespace (https://invent.kde.org/plasma/xdg-desktop-portal-kde/-/raw/master/src/settings.cpp). So "accent" in practice means the highlight or selection colour, a saturated mid tone meant to sit on both light and dark surfaces.

## 2. Full palette or only scheme and accent

No standard exposes a palette. The spec's appearance namespace stops at the three keys above. What the backends add is desktop private: the GTK backend also serves `org.gnome.desktop.interface` with 45 keys including `gtk-theme` "adw-gtk3-dark", `icon-theme`, fonts and `accent-color` "blue" as a string, but no colour values (`busctl --user call org.freedesktop.impl.portal.desktop.gtk /org/freedesktop/portal/desktop org.freedesktop.impl.portal.Settings ReadAll as 1 org.gnome.desktop.interface`); the KDE backend exposes every kdeglobals group as an `org.kde.kdeglobals.<Group>` namespace, which does carry `Colors:Window`, `Colors:View` and the rest, but only on Plasma (settings.cpp above). Neither is a contract the app can build on.

So the shell derives the palette, and the derivation inputs are:

- the scheme: dark, light or no preference from the portal, with the owner's box saying dark;
- one accent: the portal's `(ddd)` when present, otherwise the user's pick in the app, otherwise the brand colour, and on this desktop the portal never has one;
- the contrast flag: 0 or 1 from the portal, to widen the tone gap between text and surfaces;
- optionally a source image, which is how caelestia and matugen get their accent when the user asks for a wallpaper scheme.

The algorithm both of the owner's tools use is Material's dynamic colour: HCT, "a new color space (hue, chrome, tone) based on CAM16 x L*, that accounts for viewing conditions", a scheme variant that fixes hue and chroma rules (tonal spot, vibrant, expressive, content, fidelity, monochrome, neutral, rainbow, fruit salad), a contrast level, and for images a quantizer ("Celebi, which runs Wu, then WSMeans") followed by a scorer that ranks "colors for suitability for theming" (https://github.com/material-foundation/material-color-utilities). Official ports are Dart, Java, Kotlin, Swift, TypeScript and C++; caelestia uses the third-party Python port python-materialyoucolor (`pacman -Qi caelestia-cli` lists it as a dependency). I did not verify a Rust port. The output of that algorithm is exactly the role list in section 4, which is what makes scheme.json and matugen JSON interchangeable readers for the app.

## 3. Qt 6.11

### What QStyleHints and QPalette expose

`QStyleHints::colorScheme` is `Qt::ColorScheme`, added in 6.5, with `setColorScheme` and `unsetColorScheme` since 6.8 and a `colorSchemeChanged` signal. "By default, this follows the system's default color scheme (also known as appearance), and changes when the system color scheme changes." And: "When this property changes, Qt will read the system palette and update the default palette, but won't overwrite palette entries that have been explicitly set by the application." (https://doc.qt.io/qt-6/qstylehints.html). The enum is Unknown, Light, Dark (https://doc.qt.io/qt-6/qt.html#ColorScheme-enum). `setColorScheme` just calls `QPlatformTheme::requestColorScheme` (https://raw.githubusercontent.com/qt/qtbase/6.11/src/gui/kernel/qstylehints.cpp).

`Qt::ContrastPreference` exists since 6.10 with `NoPreference` and `HighContrast`, described as a setting styles "can honor by increasing the contrast of foreground and background colors, as well as giving widgets and controls thicker borders" (https://doc.qt.io/qt-6/qt.html#ContrastPreference-enum). It is exposed on `QPlatformTheme::contrastPreference()` (https://raw.githubusercontent.com/qt/qtbase/6.11/src/gui/kernel/qplatformtheme.h) and not on QStyleHints: the 6.11 header declares only the colour scheme property (https://raw.githubusercontent.com/qt/qtbase/6.11/src/gui/kernel/qstylehints.h), the 6.11.2 doc page lists no such property, and PySide6 6.11.1 raises `'QStyleHints' object has no attribute 'contrastPreference'`. A shell that wants the contrast flag reads the portal.

The defaults on `QPlatformTheme` are `colorScheme()` returning Unknown and `contrastPreference()` returning NoPreference, and `qt_fusionPalette()` picks its base from the theme's scheme: `darkAppearance ? QColor(50, 50, 50) : QColor(239, 239, 239)` for the window and `QColor(240, 240, 240)` or black for text (https://raw.githubusercontent.com/qt/qtbase/6.11/src/gui/kernel/qplatformtheme.cpp). `QGuiApplication::palette()` documents that "Roles that have not been explicitly set will reflect the system's platform theme" (https://doc.qt.io/qt-6/qguiapplication.html). `QPalette::Accent` is role 21, since 6.6, "a color that typically contrasts or complements Base, Window and Button colors" (https://doc.qt.io/qt-6/qpalette.html).

### How Qt picks the platform theme on Wayland

`-platformtheme` on the command line overrides `QT_QPA_PLATFORMTHEME` (https://doc.qt.io/qt-6/qguiapplication.html). Without either, `QGenericUnixTheme::themeNames()` reads the desktop environment: KDE gives `kde`; GNOME, X-CINNAMON, PANTHEON, UNITY, MATE, XFCE and LXDE give `gtk3` then `gnome`; anything else is lowercased with an `x-` prefix stripped and used as a name; `generic` is always appended last (https://raw.githubusercontent.com/qt/qtbase/6.11/src/gui/platform/unix/qgenericunixtheme.cpp). With `XDG_CURRENT_DESKTOP=Hyprland` that list is `hyprland`, `generic`, and no plugin is called hyprland: the plugins on the box are `KDEPlasmaPlatformTheme6.so`, `libqgtk3.so`, `libqt6ct.so` and `libqxdgdesktopportal.so` (`ls /usr/lib/qt6/plugins/platformthemes/`). So a bare Qt app on Hyprland gets QGenericUnixTheme, which reads no portal and defines no palette (same file), which is what the measurement below shows. I could not find the qtwayland integration source at its old path to quote the Wayland side of this; the behaviour under `-platform wayland` matched the generic path.

What the owner's session sets, from `~/.config/hypr/hyprland/env.conf` line 10: `env = QT_QPA_PLATFORMTHEME,qt6ct`. The shell environment agrees (`echo $QT_QPA_PLATFORMTHEME` prints `qt6ct`). The Lua config notes that env.conf once set qt5ct then qt6ct and "last write won, so qt6ct is what the session has actually been running" (`~/.config/hypr/lua/env.lua` lines 34 to 37).

### What each theme does with the portal

- `xdgdesktopportal` wraps whichever base theme `themeNames()` would have picked, reads `org.freedesktop.appearance` `color-scheme` and `contrast` at startup, listens to `SettingChanged`, serves `colorScheme()` and `contrastPreference()` from them, and does not read `accent-color` (https://raw.githubusercontent.com/qt/qtbase/6.11/src/plugins/platformthemes/xdgdesktopportal/qxdgdesktopportaltheme.cpp).
- `gnome` (QGnomeTheme) uses `QGnomePortalInterface`, which calls `ReadAll` for `org.freedesktop.appearance` and `org.gnome.desktop.a11y.interface` and subscribes through `QDBusListener`; the colour scheme comes from the portal or, failing that, from "light" or "dark" in the GTK theme name; it reads no accent and defines no palette (https://raw.githubusercontent.com/qt/qtbase/6.11/src/gui/platform/unix/qgnometheme.cpp, https://raw.githubusercontent.com/qt/qtbase/6.11/src/gui/platform/unix/qgnomeportalinterface.cpp; the string literals `"org.freedesktop.appearance"`, `"color-scheme"`, `"contrast"`, `"org.gnome.desktop.a11y.interface"`, `"high-contrast"` are in https://raw.githubusercontent.com/qt/qtbase/6.11/src/gui/platform/unix/qdbussettings_p.h).
- `gtk3` reads real colours from GTK with `gtk_style_context_lookup_color` for names like `theme_bg_color` and `theme_selected_bg_color`, and decides the scheme by comparing foreground and background lightness (https://raw.githubusercontent.com/qt/qtbase/6.11/src/plugins/platformthemes/gtk3/qgtk3interface.cpp, https://raw.githubusercontent.com/qt/qtbase/6.11/src/plugins/platformthemes/gtk3/qgtk3theme.cpp).
- `kde` (Qt's QKdeTheme) reads `Colors:*` groups from kdeglobals into the palette and infers the scheme from the theme name or from `text().lightness() < base().lightness()` meaning light; no accent, no portal for colours (https://raw.githubusercontent.com/qt/qtbase/6.11/src/gui/platform/unix/qkdetheme.cpp).
- `qt6ct` 0.11 overrides neither `colorScheme()` nor `contrastPreference()`. It builds its palette from `custom_palette` and `color_scheme_path`, loads the file with `Qt6CT::loadColorScheme`, which assigns `active_colors`, `inactive_colors` and `disabled_colors` index by index to `QPalette::ColorRole(i)` for `i < QPalette::NColorRoles`, watches the config with `QFileSystemWatcher`, and sends `QEvent::ThemeChange` to every widget on change (https://www.opencode.net/trialuser/qt6ct/-/raw/master/src/qt6ct-qtplugin/qt6ctplatformtheme.cpp, https://www.opencode.net/trialuser/qt6ct/-/raw/master/src/qt6ct-common/qt6ct.cpp). The GitHub repository is archived; the ChangeLog for 0.10 says "Moved homepage to opencode.net" and 0.11 adds Qt 6.10 support, and the Arch package points there (`pacman -Qi qt6ct` URL https://www.opencode.net/trialuser/qt6ct). Nothing in the changelog mentions Qt 6.5 or colour schemes (https://www.opencode.net/trialuser/qt6ct/-/raw/master/ChangeLog).

### Measured on the owner's box

A PySide6 6.11.1 probe (`pacman -Q pyside6`) constructed `QGuiApplication(["probe", "-platform", "wayland"])`, printed `styleHints().colorScheme()` and the active palette at once and again after 1.5 s of event loop, and connected to `colorSchemeChanged`. Run as `QT_QPA_PLATFORMTHEME=<name> python3 probe.py` from the scratchpad; the unset row used `env -u QT_QPA_PLATFORMTHEME`.

| theme | scheme at t0 | scheme at 1.5 s | Window | Text | Highlight | Accent |
| --- | --- | --- | --- | --- | --- | --- |
| qt6ct (owner's session) | Unknown | Unknown | #201f23 | #e5e1e7 | #c9bfff | #c9bfff |
| unset (generic) | Unknown | Unknown | #efefef | #000000 | #308cc6 | #308cc6 |
| xdgdesktopportal | Dark | Dark | #323232 | #f0f0f0 | #308cc6 | #308cc6 |
| gtk3 | Dark | Dark | #323232 | #ffffff | #c9bfff | #000000 |
| gnome | Unknown | Dark, signal fired | #efefef | #000000 | #308cc6 | #308cc6 |
| kde | Dark | Dark | #202326 | #fcfcfc | #3daee9 | #3daee9 |

Reading the rows. The qt6ct palette is `~/.config/qt6ct/colors/caelestia.conf` (`~/.config/qt6ct/qt6ct.conf` has `custom_palette=true`, `color_scheme_path=/home/bandit/.config/qt6ct/colors/caelestia.conf`, `style=Fusion`), whose window entry is `#ff201f23` and highlight `#ffc9bfff`; that file has 21 entries per row, one short of QPalette's 22 roles, and the Accent role came out equal to Highlight. The unset row is `qt_fusionPalette` light because the generic theme answers Unknown. The xdgdesktopportal row is `qt_fusionPalette` dark with the Fusion default highlight, so it gets the scheme right and the accent wrong. The gtk3 highlight is caelestia's `@define-color accent_color #c9bfff` from `~/.config/gtk-3.0/gtk.css` line 1, and the Accent role came out black, so that theme does not set it. The gnome row is the one to remember: the portal answer arrives asynchronously, the scheme flips to Dark after startup, and in this probe the palette had not followed 1.5 s later. The kde row is Breeze Dark; `~/.config/kdeglobals` names `ColorScheme=BreezeDark` but has no `Colors:*` groups, which Qt's own QKdeTheme needs, so plasma-integration's `KDEPlasmaPlatformTheme6.so` (installed, 6.7.3) most likely resolved the name; I could not get Qt's plugin logging to print under PySide6 to confirm which file loaded.

The consequence for the shell: `QStyleHints::colorScheme` says Unknown in the owner's own session and in the no-config case, so it cannot be the primary signal. The core reads the portal for the scheme, and when a palette-carrying theme is active (qt6ct, gtk3, kde) the shell can offer `QGuiApplication::palette()` as a "match my Qt apps" source, inferring dark from `text().lightness() > window().lightness()` the way Qt's kde and gtk3 themes do.

### The QML side

`SystemPalette` gives QML "the Qt application palettes" with `window`, `windowText`, `base`, `text`, `button`, `highlight`, `highlightedText`, `placeholderText` (since 6.2) and `accent` (since 6.7), per colour group (https://doc.qt.io/qt-6/qml-qtquick-systempalette.html). The default Qt Quick Controls style on Linux is Fusion, chosen by `QT_QUICK_CONTROLS_STYLE`, `qtquickcontrols2.conf` or `QQuickStyle::setStyle` (https://doc.qt.io/qt-6/qtquickcontrols-styles.html). Fusion "uses the standard system Palette to provide colors that match the desktop environment" and "automatically switches dark and light themes according to the system settings", with explicit palette attributes propagating from parent to children (https://doc.qt.io/qt-6/qtquickcontrols-fusion.html). `Material.theme` has `Material.System`, which "chooses either the light or dark theme based on the system theme colors", but `Material.accent` defaults to `Material.Pink` and the page never mentions a system accent (https://doc.qt.io/qt-6/qtquickcontrols-material.html). Given the measurements, "system settings" means whatever the platform theme answered, so a QML shell on Hyprland sees the same Unknown unless the app sets the palette itself.

## 4. How the owner's shell makes colours

### The generator

caelestia-shell 2.2.0 delegates colour work to caelestia-cli 1.1.2, which depends on `python-materialyoucolor` (`pacman -Qi caelestia-cli`). `utils/material/generator.py` imports `SchemeContent`, `SchemeExpressive`, `SchemeFidelity`, `SchemeFruitSalad`, `SchemeMonochrome`, `SchemeNeutral`, `SchemeRainbow`, `SchemeTonalSpot` and `SchemeVibrant` from materialyoucolor, maps the variant string to one of them in `get_scheme` (default vibrant), and `gen_scheme` builds `get_scheme(scheme.variant)(source_color_hct=primary, is_dark=not is_light, contrast_level=0.0)` (`/usr/lib/python3.14/site-packages/caelestia/utils/material/generator.py` lines 4 to 12 and 158 to 184). A `dynamic` scheme takes its source colour from the wallpaper through `get_colours_for_image`; every other scheme reads a shipped text file of `name hex` lines from `data/schemes/<name>/<flavour>/<mode>.txt` (`utils/scheme.py` lines 156 to 176 and 203 to 205). Shipped names: caelestia, catppuccin, darkgreen, dracula, everblush, everforest, gruvbox, nord, oldworld, onedark, rosepine, shadotheme, solarized, tokyonight (`ls /usr/lib/python3.14/site-packages/caelestia/data/schemes/`). Note that caelestia's catppuccin and gruvbox files are Material re-renderings seeded from those palettes (the catppuccin mocha file's `background` is `131317` and its `mauve` is `bfb8ff`, neither of which is a Catppuccin value), so they are not a way to import the real projects' colours.

### The files it writes

`Scheme.save()` writes `~/.local/state/caelestia/scheme.json` with `name`, `flavour`, `mode`, `variant` and `colours` through an atomic dump (`utils/scheme.py` lines 123 to 134, path from `utils/paths.py` line 32). `apply_colours` then, gated by `enable*` keys in cli.json that default to true: writes OSC sequences to `~/.local/state/caelestia/sequences.txt` and every `/dev/pts/*`; writes `~/.config/hypr/scheme/current.conf` (or `.lua`) as `$name = hex` lines; writes `gtk.css` for gtk-3.0 and gtk-4.0 and the dconf `color-scheme`; writes `~/.config/qtengine/caelestia.colors` (a KDE `[ColorEffects:*]`/`[Colors:*]` scheme from the `qt{mode}.colors` template) plus `~/.config/qtengine/config.json`; renders each file in `~/.config/caelestia/templates/` into `~/.local/state/caelestia/theme/`; and runs `theme.postHook` with `SCHEME_NAME`, `SCHEME_FLAVOUR`, `SCHEME_MODE`, `SCHEME_VARIANT` and `SCHEME_COLOURS` (JSON) in the environment (`utils/theme.py` lines 124 to 149, 307 to 335 and 398 to 470). The shell reads scheme.json with a `FileView` that has `watchChanges: true`, maps each colour to an `m3<name>` property (terminal colours keep their `term` name), and switches mode by running `caelestia scheme set --notify -m <mode>` (`/etc/xdg/quickshell/caelestia/services/Colours.qml` lines 62 to 83 and 116 to 120).

On this box (`fd -t f . ~/.config/caelestia ~/.config/matugen ~/.local/state/caelestia`): scheme.json, sequences.txt, `theme/kitty-dynamic.conf` (118 bytes: background, cursor and selection colours), `wallpaper/path.txt` and unrelated state. `~/.config/caelestia/templates/` is empty and `cli.json` only sets `wallpaper.postHook` to `~/.local/bin/kitty-dynamic-apply`, a script that restores two terminal colours and pushes border colours to Hyprland from `hypr/scheme/current.conf`.

The current scheme.json (mode 0600, 2026-07-06): `name` rosepine, `flavour` main, `mode` dark, `variant` tonalspot, and 110 colours as bare six-digit hex: the Material roles (`background` 141317, `surface`, `surfaceDim`, `surfaceBright`, `surfaceContainerLowest` through `surfaceContainerHighest`, `onSurface` e5e1e7, `onSurfaceVariant`, `outline`, `outlineVariant`, `primary` c9bfff, `onPrimary`, `primaryContainer`, `secondary`, `tertiary`, `error`, the `*Fixed` set, `inverseSurface`, `shadow`, `scrim`, `surfaceTint`), `term0` to `term15`, Catppuccin-style aliases (`rosewater` to `lavender`, `text`, `subtext1`, `subtext0`, `overlay2` to `overlay0`, `surface2` to `surface0`, `base`, `mantle`, `crust`), KDE-style `klink`, `kpositive` and friends, and `success` roles (`cat ~/.local/state/caelestia/scheme.json`; `caelestia scheme get` prints the same). That is a complete palette in the role vocabulary the app would derive anyway, so reading it is cheaper than deriving from an accent, and it carries `mode`, which the portal also carries.

Two things do not line up today. `~/.config/hypr/scheme/current.conf` (2026-08-23) holds a different scheme from scheme.json (2026-07-06): `$background = 191113` against `141317`, `$primary_paletteKeyColor = aa6174` against `786fab`. The shell reads scheme.json, so the app should too, and treat the hypr file as derived output. And `~/.config/qt6ct/colors/caelestia.conf` (2026-08-26, next to a `qt6ct.conf.bak.20260826`) is not written by anything installed: caelestia-cli writes `~/.config/qtengine/` instead, and `rg -l qt6ct ~/.local/bin ~/.config/caelestia ~/.config/hypr` finds only the env files. Its colours match scheme.json's rosepine (base `#ff141317`, highlight `#ffc9bfff`), so it is in step by hand and will drift on the next scheme change. What consumes `~/.config/qtengine/config.json` I could not verify; the wiki describes hyprqt6engine as "a replacement for qt6ct, compatible with KDE Apps/KColorScheme", enabled with `QT_QPA_PLATFORMTHEME=hyprqt6engine` and configured in `~/.config/hypr/hyprqt6engine.conf` (https://raw.githubusercontent.com/hyprwm/hyprland-wiki/main/content/hypr-ecosystem/user/hyprqt6engine.md), which is a different path, and it is not installed (`pacman -Q hyprqt6engine` fails; `hyprland-qt-support 0.1.0-13` is).

### matugen

`matugen-bin 4.1.0-1` is installed explicitly with "Required By: None" (`pacman -Qi matugen-bin`), `~/.config/matugen` does not exist, and no file under `~/.config/hypr`, `~/.config/caelestia`, `~/.local/bin` or `~/.config/quickshell` mentions it. It describes itself as "A cross-platform material you and base16 color generation tool", takes `matugen image <path>` or `matugen color hex "#..."` with `--mode`, `--contrast` and `--type`, renders templates listed in config.toml with `input_path` and `output_path` using keywords such as `{{colors.primary.default.hex}}`, and `--json` prints hex, rgb, hsl or strip (https://raw.githubusercontent.com/InioX/matugen/main/README.md). A dry run here, `matugen color hex '#c9bfff' --json hex --dry-run --mode dark`, produced top-level keys `base16`, `colors`, `image`, `is_dark_mode`, `mode`, `palettes`; `colors` holds 50 snake_case Material roles (`primary`, `on_primary`, `surface`, `surface_container_high`, `outline`, `error`, `source_color` and so on), each with `dark`, `default` and `light` entries of the form `{"color": "#c9bfff"}`; `surface.dark` came out `#141318`, one unit from caelestia's `141317` for the same seed, which is the same algorithm in a different port. A reader that lowercases and strips underscores from role names handles both files.

## 5. Theme file formats of Catppuccin, Gruvbox and Tokyo Night

### Catppuccin

The canonical file is `palette.json` in catppuccin/palette, published as the npm package `@catppuccin/palette`, with four flavours (latte, frappe, macchiato, mocha), 26 named colours each carrying `hex`, `rgb`, `hsl`, an `accent` boolean and an `order`, plus an `ansiColors` block of the 16 terminal colours, and exports to CSS, Sass, ASE, GIMP, Procreate and language ports (https://github.com/catppuccin/palette). Mocha's `base` is `#1e1e2e`, `text` `#cdd6f4`, `mauve` `#cba6f7` with `accent: true` (https://raw.githubusercontent.com/catppuccin/palette/main/palette.json). Ports that matter here: catppuccin/qt5ct ships 56 `catppuccin-<flavor>-<accent>.conf` files in the qt6ct `[ColorScheme]` format with 22 entries per row, installed by copying into `~/.config/qt6ct/colors/` and selecting "custom" in qt6ct (https://github.com/catppuccin/qt5ct, https://raw.githubusercontent.com/catppuccin/qt5ct/main/themes/catppuccin-mocha-mauve.conf); catppuccin/kitty ships `latte.conf`, `frappe.conf`, `macchiato.conf`, `mocha.conf` (https://raw.githubusercontent.com/catppuccin/kitty/main/themes/mocha.conf); catppuccin/xresources ships one `.Xresources` per flavour (`gh api repos/catppuccin/xresources/contents/themes`).

### Gruvbox

The canonical definition is the Vim script in morhetz/gruvbox, `colors/gruvbox.vim`: `let s:gb.dark0_hard = ['#1d2021', 234]`, `dark0 = '#282828'`, `dark0_soft = '#32302f'`, `dark1` to `dark4`, `gray_245 = '#928374'`, `light0_hard = '#f9f5d7'`, `light0 = '#fbf1c7'`, `light0_soft`, `light1` to `light4`, then `bright_red = '#fb4934'`, `bright_green = '#b8bb26'`, `bright_yellow = '#fabd2f'`, `bright_blue = '#83a598'`, `bright_purple = '#d3869b'`, `bright_aqua = '#8ec07c'`, `bright_orange = '#fe8019'`, and `neutral_*` and `faded_*` rows; the background is chosen from `g:gruvbox_contrast_dark` as soft, medium or hard (https://raw.githubusercontent.com/morhetz/gruvbox/master/colors/gruvbox.vim). The repo ships no JSON or YAML, only `gruvbox_256palette.sh` scripts (https://github.com/morhetz/gruvbox). morhetz/gruvbox-contrib carries ports for xresources (`gruvbox-dark.xresources`, `gruvbox-light.xresources`), iterm2, konsole, xfce4-terminal and others, but no kitty (`gh api repos/morhetz/gruvbox-contrib/contents`); kitty's own theme collection has `gruvbox-dark.conf`, `-hard`, `-soft` and the light three (`gh api repos/kovidgoyal/kitty-themes/contents/themes`).

### Tokyo Night

folke/tokyonight.nvim is the source. Colours live in Lua tables, `lua/tokyonight/colors/storm.lua` (`bg = "#24283b"`, `bg_dark = "#1f2335"`, `bg_highlight = "#292e42"`, `fg = "#c0caf5"`, `fg_dark = "#a9b1d6"`, `comment = "#565f89"`, `blue = "#7aa2f7"`, `cyan = "#7dcfff"`, `magenta = "#bb9af7"`, `purple = "#9d7cd8"`, `orange = "#ff9e64"`, `yellow = "#e0af68"`, `green = "#9ece6a"`, `red = "#f7768e"`, `teal = "#1abc9c"`, `terminal_black = "#414868"` and more) with `night.lua` overriding `bg = "#1a1b26"`, `bg_dark = "#16161e"`, `bg_dark1 = "#0C0E14"` on top of storm (https://raw.githubusercontent.com/folke/tokyonight.nvim/main/lua/tokyonight/colors/storm.lua, https://raw.githubusercontent.com/folke/tokyonight.nvim/main/lua/tokyonight/colors/night.lua). Styles are storm, moon, night and day. The `extras/` directory is generated from those tables and includes alacritty, foot, ghostty, kitty, wezterm, xresources, windows_terminal and around forty others (https://github.com/folke/tokyonight.nvim, `gh api repos/folke/tokyonight.nvim/contents/extras`); the kitty file sets `background #1a1b26`, `foreground #c0caf5`, `active_tab_background #7aa2f7` and color0 to color17 (https://raw.githubusercontent.com/folke/tokyonight.nvim/main/extras/kitty/tokyonight_night.conf).

### Formats all three share

base16 YAML. tinted-theming/schemes carries `catppuccin-latte`, `-frappe`, `-macchiato`, `-mocha`; `gruvbox-dark-hard`, `-medium`, `-soft` and the light three (plus gruvbox-material); `tokyo-night-dark`, `-light`, `-moon`, `-storm` (`gh api repos/tinted-theming/schemes/contents/base16`). A file is `system`, `name`, `author`, `variant` and a `palette` of `base00` to `base0F`; catppuccin-mocha.yaml reads `base00: "#1e1e2e" # base`, `base05: "#cdd6f4" # text`, `base0E: "#cba6f7" # mauve` (https://raw.githubusercontent.com/tinted-theming/schemes/spec-0.11/base16/catppuccin-mocha.yaml). The styling guide fixes the roles: base00 default background, base01 lighter background, base02 selection background, base03 comments, base04 dark foreground, base05 default foreground, base06 and base07 light foregrounds, base08 red, base09 orange, base0A yellow, base0B green, base0C cyan, base0D blue, base0E purple, base0F brown (https://raw.githubusercontent.com/tinted-theming/home/main/styling.md). matugen emits the same sixteen slots in its `base16` block, so one mapping serves both.

kitty `.conf`. Key value lines: `background`, `foreground`, `selection_background`, `selection_foreground`, `cursor`, `url_color`, `active_tab_background`, `active_border_color` and `color0` to `color15`. Catppuccin and Tokyo Night ship it themselves; Gruvbox comes through kitty-themes; the owner's shell writes a small one too.

Xresources is the third common format (catppuccin/xresources, tokyonight extras, gruvbox-contrib), and it is the same sixteen colours plus foreground and background in a different syntax.

### The pick

Import base16 YAML as the paste format: every flavour of all three projects exists in it, the slot roles are documented, and it maps cleanly to what the shell needs: base00 to background, base01 to surface, base02 to the raised surface or selection, base03 to outline, base05 to text, base07 to the brightest text, base0D (or a slot the user picks) to the accent, base08 to error, base0B to success. Accept kitty conf as the second format because it is what a terminal user most likely has in a file already: background and foreground map directly, color8 becomes the outline, and the accent is the user's choice among color1 to color6 with color4 (blue) as the default. Skip the qt6ct `[ColorScheme]` rows (only Catppuccin ships them) and the three canonical files (three shapes for three projects).

## 6. What I could not verify

- The Hyprland wiki site returns 403 to fetch tools; the quotes come from the wiki's source repository on GitHub.
- The qtwayland integration source moved from `src/client/` and I did not find its new path, so the Wayland theme selection is stated from `QGenericUnixTheme::themeNames()` in qtbase and from the observed generic behaviour under `-platform wayland`, not from qtwayland's own code.
- Qt's plugin loading log printed nothing under PySide6, so the table names the theme by its `QT_QPA_PLATFORMTHEME` value; which `.so` served the kde row is an inference from the Breeze Dark values and the missing `Colors:*` groups in kdeglobals.
- Why `hypr/scheme/current.conf` and `scheme.json` carry different schemes on the box.
- What reads `~/.config/qtengine/config.json`; caelestia-cli writes it, hyprqt6engine documents a different path, and hyprqt6engine is not installed.
- A Rust port of material-color-utilities; the README lists Dart, Java, Kotlin, Swift, TypeScript and C++.
- The KDE portal backend's `contrast` handling: the fetched settings.cpp shows none under the appearance namespace, and no KDE backend runs here.
- The `doc:since` annotations for the appearance keys are not in the rendered spec; the dates above come from the file's git history and from tag contents.
