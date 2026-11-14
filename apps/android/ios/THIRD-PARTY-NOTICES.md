# Third-party notices (iPhone app)

AniBeam is free software under the GNU General Public License, version 3 or
later (see `LICENSE` at the root of the repository). The iPhone app plays video
with libmpv, which is statically linked together with FFmpeg and the libraries
below. They come from the LGPL build of MPVKit 1.1.0-n9.0.2
(https://github.com/mpvkit/MPVKit/releases/tag/1.1.0-n9.0.2), pinned and
hash-checked by `apps/android/scripts/fetch-mpv-ios.sh`.

That build is not GPL: mpv is configured with `-Dgpl=false` and FFmpeg with
`--enable-version3` but without `--enable-gpl`, so libmpv and FFmpeg are
distributed under the LGPL (FFmpeg as LGPL-3.0-or-later because of
`--enable-version3`). The source code of every library is public at the URLs
below. AniBeam's own source is public too (https://github.com/marcusbandit/AniBeam),
and the build scripts in this repository relink it against these libraries, which
meets the relinking terms of the LGPL. The full texts of the licenses that the
libraries require are in the `licenses` folder next to this file (inside the
app, in the `licenses` folder of the app bundle).

Versions are the upstream tags that MPVKit's build scripts check out for this
release. Where a library is dual or multi licensed, the license AniBeam relies
on is listed first.

| Library | Version | License (SPDX) | Source |
| --- | --- | --- | --- |
| mpv (libmpv) | v0.41.0 | LGPL-2.1-or-later (built with `-Dgpl=false`; the default is GPL-2.0-or-later) | https://github.com/mpv-player/mpv/tree/v0.41.0 |
| FFmpeg (libavcodec, libavdevice, libavfilter, libavformat, libavutil, libswresample, libswscale) | n9.0.2 | LGPL-3.0-or-later (`--enable-version3`, no `--enable-gpl`) | https://github.com/FFmpeg/FFmpeg/tree/n9.0.2 |
| OpenSSL (libcrypto, libssl) | 3.3.5 | Apache-2.0 | https://github.com/openssl/openssl/tree/openssl-3.3.5 |
| GnuTLS | 3.8.11 | LGPL-2.1-or-later | https://github.com/gnutls/gnutls/tree/3.8.11 |
| Nettle and Hogweed | 3.10 (nettle_3.10_release_20240616) | LGPL-3.0-or-later OR GPL-2.0-or-later | https://git.lysator.liu.se/nettle/nettle/-/tree/nettle_3.10_release_20240616 |
| GMP | 6.2.1 | LGPL-3.0-or-later OR GPL-2.0-or-later | https://github.com/alisw/GMP/tree/v6.2.1 |
| libass | 0.17.5 | ISC | https://github.com/libass/libass/tree/0.17.5 |
| libunibreak | 6.1 (libunibreak_6_1) | Zlib | https://github.com/adah1972/libunibreak/tree/libunibreak_6_1 |
| FreeType | 2.14.3 (VER-2-14-3) | FTL (dual licensed with GPL-2.0-only; AniBeam uses it under the FTL) | https://github.com/freetype/freetype/tree/VER-2-14-3 |
| FriBidi | 1.0.16 | LGPL-2.1-or-later | https://github.com/fribidi/fribidi/tree/v1.0.16 |
| HarfBuzz | 14.2.0 | MIT ("Old MIT") | https://github.com/harfbuzz/harfbuzz/tree/14.2.0 |
| libbluray | 1.4.0 | LGPL-2.1-or-later | https://code.videolan.org/videolan/libbluray/-/tree/1.4.0 |
| libuavs3d (uavs3d) | 1.2.1-fix build, from the master branch of uavs3d at build time (no tag) | BSD-3-Clause style (custom text, see below) | https://github.com/uavs3/uavs3d |
| libdovi (dovi_tool) | 3.3.2 (libdovi-3.3.2) | MIT | https://github.com/quietvoid/dovi_tool/tree/libdovi-3.3.2 |
| MoltenVK | 1.4.2 (v1.4.2) | Apache-2.0 | https://github.com/KhronosGroup/MoltenVK/tree/v1.4.2 |
| shaderc (Libshaderc_combined) | 2025.5 (v2025.5) | Apache-2.0 | https://github.com/google/shaderc/tree/v2025.5 |
| Little CMS (lcms2) | 2.17 (lcms2.17) | MIT | https://github.com/mm2/Little-CMS/tree/lcms2.17 |
| libplacebo | 7.360.1 (v7.360.1) | LGPL-2.1-or-later | https://github.com/haasn/libplacebo/tree/v7.360.1 |
| SPIRV-Cross (built into libplacebo) | vulkan-sdk-1.4.309.0 | Apache-2.0 | https://github.com/KhronosGroup/SPIRV-Cross/tree/vulkan-sdk-1.4.309.0 |
| dav1d | 1.5.3 | BSD-2-Clause | https://github.com/videolan/dav1d/tree/1.5.3 |
| uchardet | 0.0.8 (v0.0.8) | MPL-1.1 OR GPL-2.0-or-later OR LGPL-2.1-or-later (AniBeam uses it under the LGPL) | https://gitlab.freedesktop.org/uchardet/uchardet/-/tree/v0.0.8 |

The MPVKit packaging itself (https://github.com/mpvkit/MPVKit, tag
1.1.0-n9.0.2) is LGPL-3.0. Its build scripts (Sources/BuildScripts) and the
per-library repositories under https://github.com/mpvkit show the exact
configure flags and the patches applied to mpv, libbluray, libuavs3d and
MoltenVK.

Notes:

- No library here is GPL-only. mpv's GPL-only files are excluded by
  `-Dgpl=false`, FFmpeg's GPL parts are excluded by leaving out `--enable-gpl`,
  and the libraries that offer a GPL option (Nettle, GMP, FreeType, uchardet)
  also offer an LGPL or permissive one, which is the one used.
- shaderc bundles glslang, SPIRV-Tools and SPIRV-Headers, which carry their own
  permissive licenses (BSD-3-Clause, MIT and Apache-2.0 files) in shaderc's
  source tree at the tag above. Their individual notices are not reproduced here.

## License texts

In the `licenses` folder:

| File | Applies to |
| --- | --- |
| `LGPL-2.1.txt` | mpv, GnuTLS, FriBidi, libbluray, libplacebo, uchardet (LGPL option), FFmpeg (LGPL-2.1 base) |
| `LGPL-3.0.txt` | FFmpeg (with `--enable-version3`), Nettle, GMP |
| `Apache-2.0.txt` | OpenSSL, MoltenVK, shaderc, SPIRV-Cross |
| `FFmpeg-LICENSE.md` | FFmpeg's own statement of which parts are under which license |
| `libass-ISC.txt` | libass |
| `libunibreak-zlib.txt` | libunibreak |
| `freetype-FTL.txt` | FreeType |
| `harfbuzz-MIT.txt` | HarfBuzz |
| `libuavs3d-BSD-3-Clause.txt` | libuavs3d |
| `libdovi-MIT.txt` | libdovi |
| `lcms2-MIT.txt` | Little CMS |
| `dav1d-BSD-2-Clause.txt` | dav1d |
| `uchardet-MPL-GPL-LGPL.txt` | uchardet (upstream COPYING, holding the MPL-1.1, GPL-2.0 and LGPL-2.1 texts) |

The LGPL and Apache texts are the canonical copies from gnu.org and apache.org.
The others are the upstream license files at the tags listed above.
