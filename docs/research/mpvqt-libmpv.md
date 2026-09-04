# MpvQt and the libmpv render API on Wayland

Research for ticket #4. Target: Qt 6.11, mpv 0.41, MpvQt 1.2.0 on Arch, Wayland under Hyprland, one NVIDIA desktop and one AMD laptop. Every claim below is followed by the source it came from. Local checks ran on the NVIDIA desktop (mpv 0.41.0, qt6-base 6.11.1, nvidia-utils 610.43.03); the AMD laptop was not touched.

## Summary

1. Build the Linux shell's video surface as a subclass of `MpvAbstractItem` marked `QML_ELEMENT`; MpvQt 1.2.0 asks for Qt 6.5 and libmpv client API 2.x and nothing in it pins or conflicts with Qt 6.11 or mpv 0.41, and Arch builds it against exactly that pair.
2. Run Qt Quick on the OpenGL RHI backend, set with `QQuickWindow::setGraphicsApi(QSGRendererInterface::OpenGL)` before the first window: the render API has only an OpenGL and a software backend, `QQuickFramebufferObject` is OpenGL only, and the Vulkan request on mpv has sat open since 2019.
3. Set `hwdec=auto`: on NVIDIA that lands on `nvdec` through the CUDA to OpenGL interop, on AMD on `vaapi` through EGL dmabuf import, and both run inside Qt's EGL context on Wayland because MpvQt passes the `wl_display` to the render context. Skip nvidia-vaapi-driver; its own README says mpv should use nvdec.
4. Vulkan video decoding, which mpv 0.41 now prefers, is not reachable through `vo=libmpv`: it needs `gpu-next`, and the render API still renders through the `vo_gpu` code path.
5. A thumbnail or seek preview needs its own mpv core: a core allows one render context, and the OpenGL backend needs a current GL context but no window. For seek hover, copy Haruna's `MpvPreview` (a second `MpvAbstractItem` with audio, subs and OSD off). For library thumbnails run a headless mpv process the way thumbfast does (encoding mode into a raw BGRA file) or `--vo=image --frames=1 --start=T`; that fits a core job because it is a process, not a GL context.
6. libmpv starts with `config=no`, `osc=no`, `input-default-bindings=no`, `input-vo-keyboard=no`, `terminal=no`, `idle=yes`. `config`, `config-dir`, `load-scripts` and `scripts` are read only before `mpv_initialize()`, and MpvQt calls `mpv_initialize()` inside the base constructor, so a subclass cannot use them.
7. To honour the user's mpv.conf under MpvQt as shipped, set `include=~/.config/mpv/mpv.conf` after construction (MpvQt itself loads `~/.config/mpvqt/mpvqt.conf` that way) or call `mpv_load_config_file()`, then re-assert `osc=no` and friends; load scripts with the `load-script` command per file. Loading them before init means patching MpvQt or owning the ~800 lines ourselves.
8. Subtitle settings map one to one onto runtime properties: `sub-font-size` (scaled pixels at a 720 line window, default 38), `sub-pos` (0 to 150, default 100, above 100 can clip), `sub-scale` (factor, default 1), `sub-ass-override` (default `scale`). With the default only `sub-scale` touches ASS; font size and position reach ASS only under `force`, which the manual says can break rendering.
9. `aid` and `sid` are per file ids and the property reports the effective track during playback, so persist language (`alang`, `slang`, IETF tags, two and three letter codes treated the same) and map a remembered track back through `track-list` on load, the way Haruna does.
10. mpv's own watch_later saves `aid`, `sid`, `sub-pos`, `sub-scale` and `sub-ass-override` per file hash; the core owns the resume point, so keep `resume-playback=no` and `save-position-on-quit=no` whenever user config is loaded.

## Versions checked

Arch ships mpvqt 1.2.0-1, built 2026-05-18, depending on mpv, qt6-base and qt6-declarative; the installed mpv is 0.41.0 with libplacebo 7.360.1, qt6-base and qt6-declarative are 6.11.1.
Source: local check, `pacman -Si mpvqt`, `mpv --version`, `pacman -Q qt6-base qt6-declarative mpv libplacebo`.

The mpv 0.41 client API version is 2.5.
Source: /usr/include/mpv/client.h on this machine, and https://github.com/mpv-player/mpv/blob/v0.41.0/include/mpv/client.h

MpvQt tags: v1.2.0 (edb1e321, 2026-05-18), v1.1.1 (2025-04-15), v1.1.0 (2025-04-07), v1.0.1 (2024-07-29), v1.0.0 (2023-12-10).
Source: https://invent.kde.org/libraries/mpvqt/-/tags and `gh api repos/KDE/mpvqt/tags`

Haruna was read at master 40d4c5c65a (2026-09-02); its latest tag is v1.8.1.
Source: `gh api repos/KDE/haruna/commits/master` and `gh api repos/KDE/haruna/tags`

## MpvQt 1.2.0

### What it is and how a shell uses it

MpvQt describes itself as "a libmpv wrapper for Qt Quick 2/Qml". The README's contract is: create a class extending `MpvAbstractItem` with `Q_OBJECT` and `QML_ELEMENT`, add it to a QML module with `qt_add_qml_module(... URI com.example.mpvqt ...)`, then instantiate `MpvItem {}` from QML; `QML_NAMED_ELEMENT(VideoPlayer)` renames it.
Source: https://github.com/KDE/mpvqt/blob/v1.2.0/README.md

`class MPVQT_EXPORT MpvAbstractItem : public QQuickFramebufferObject`, including `<mpv/client.h>` and `<mpv/render_gl.h>`. Every control method is `Q_INVOKABLE`: `observeProperty(name, mpv_format, id)`, `unobserveProperty(id)`, `setProperty(name, QVariant)`, `setPropertyAsync(name, QVariant, id)`, `setPropertyBlocking`, `getProperty(name)`, `getPropertyAsync(name, id)`, `command(QStringList)`, `commandBlocking(QStringList)`, `commandAsync(QStringList, id)`, `expandText(text)`, `requestUpdateFromRenderer()`. The item's only signal is `ready()`; `propertyChanged` and `asyncReply` live on `MpvController`, reached through the protected `mpvController()`.
Source: https://github.com/KDE/mpvqt/blob/v1.2.0/src/mpvabstractitem.h

The bundled example registers with `QML_ELEMENT` and no `qmlRegisterType`, its `main.cpp` calls `QQuickWindow::setGraphicsApi(QSGRendererInterface::OpenGL)` before constructing `QGuiApplication`, and its QML does `MpvItem { id: mpv; anchors.fill: parent }`, `mpv.setPropertyAsync(MpvProperties.Mute, !mpv.getProperty(MpvProperties.Mute))` and `commandAsync(["expand-text", ...], MpvItem.ExpandText)`. A comment in `Main.qml` warns not to load a file in `Component.onCompleted`; use `onReady`.
Source: https://github.com/KDE/mpvqt/blob/v1.2.0/examples/video-player/mpvitem.h, https://github.com/KDE/mpvqt/blob/v1.2.0/examples/video-player/main.cpp, https://github.com/KDE/mpvqt/blob/v1.2.0/examples/video-player/Main.qml

Haruna's `MpvItem : public MpvAbstractItem` also uses `Q_OBJECT` plus `QML_ELEMENT`, exposes `Q_PROPERTY` for position, duration, pause, mute, volume, audioId, subtitleId, playbackState and more, with `Q_INVOKABLE loadFile`, `userCommand`, `setTrack` and friends. Haruna's `main.cpp` sets the OpenGL graphics API with the comment "required by mpv", and its QML calls `root.m_mpv.command(["frame-step"])`, `root.m_mpv.command(["seek", 0, "absolute"])` and `root.m_mpv.setProperty(MpvProperties.Speed, 1.0)` straight from `Actions.qml`.
Source: https://github.com/KDE/haruna/blob/master/src/mpv/mpvitem.h, https://github.com/KDE/haruna/blob/master/src/main.cpp, https://github.com/KDE/haruna/blob/master/src/qml/Actions.qml

### Threading model

Three threads are involved. The `MpvAbstractItem` constructor logs a `qCCritical` if `QQuickWindow::graphicsApi()` is not OpenGL, creates a `QThread`, creates an `MpvController`, moves it to that thread, starts it, and calls `MpvController::init` with `Qt::BlockingQueuedConnection` ("must wait for init to finish or the mpv object could be accessed while not initialized").
Source: https://github.com/KDE/mpvqt/blob/v1.2.0/src/mpvabstractitem.cpp

`MpvController::init()` runs on that worker thread: it sets `LC_NUMERIC` back to "C" (Qt changes the locale in `QGuiApplication`), calls `mpv_create()` and `mpv_initialize()` (both `qFatal` on failure), installs `mpv_set_wakeup_callback(mpv, MpvController::mpvEvents, this)`, then sets `include=<ConfigLocation>/mpvqt/mpvqt.conf` and `vo=libmpv` ("otherwise mpv opens a separate window").
Source: https://github.com/KDE/mpvqt/blob/v1.2.0/src/mpvcontroller.cpp

The wakeup callback does nothing but `QMetaObject::invokeMethod(ctrl, &MpvController::eventHandler, Qt::QueuedConnection)`; `eventHandler()` drains `mpv_wait_event(mpv, 0)` until `MPV_EVENT_NONE` and emits `fileStarted`, `fileLoaded`, `endFile(reason)`, `videoReconfig`, `asyncReply` and `propertyChanged`. That matches the client.h pattern for GUI toolkits: a wakeup callback plus polling with a zero timeout.
Source: https://github.com/KDE/mpvqt/blob/v1.2.0/src/mpvcontroller.cpp, https://github.com/mpv-player/mpv/blob/v0.41.0/include/mpv/client.h

From the item, `observeProperty`, `setProperty`, `setPropertyAsync`, `getPropertyAsync`, `command` and `commandAsync` are forwarded with `Qt::QueuedConnection`; `unobserveProperty`, `setPropertyBlocking`, `getProperty`, `commandBlocking` and `expandText` use `Qt::BlockingQueuedConnection` with a return value. So a "get" from the GUI thread blocks the GUI thread until the controller thread finishes a synchronous `mpv_get_property`, and client.h says synchronous calls "have to wait until the playback core is ready, which currently can take an unbounded time". Ticks should therefore come from an observed `time-pos`, never from a getter.
Source: https://github.com/KDE/mpvqt/blob/v1.2.0/src/mpvabstractitem.cpp, https://github.com/mpv-player/mpv/blob/v0.41.0/include/mpv/client.h

Rendering happens on Qt's scene graph render thread. `MpvRenderer::createFramebufferObject()` lazily creates the render context there with `MPV_RENDER_PARAM_API_TYPE = MPV_RENDER_API_TYPE_OPENGL`, `MPV_RENDER_PARAM_OPENGL_INIT_PARAMS` (a `get_proc_address` that calls `QOpenGLContext::currentContext()->getProcAddress`), and on Linux either `MPV_RENDER_PARAM_X11_DISPLAY` or `MPV_RENDER_PARAM_WL_DISPLAY` from `QNativeInterface::QWaylandApplication`. It then calls `mpv_render_context_set_update_callback(ctx, on_mpv_redraw, this)`; the redraw callback queues `requestUpdateFromRenderer` on the item, which calls `update()`. `render()` fills an `mpv_opengl_fbo` from `framebufferObject()->handle()`, `width()`, `height()`, `internal_format = 0`, passes `MPV_RENDER_PARAM_FLIP_Y = 0`, and calls `mpv_render_context_render`. `MPV_RENDER_PARAM_ADVANCED_CONTROL` is not set.
Source: https://github.com/KDE/mpvqt/blob/v1.2.0/src/mpvrenderer.cpp

That split satisfies render.h, which recommends rendering on a separate thread, requires that the GL context be current on the calling thread for every `mpv_render_*` call, and warns that a render thread waiting on a non safe libmpv call degrades to "mpv_render_context_render() not being called or stuck" unless `ADVANCED_CONTROL` is set, in which case it deadlocks for real.
Source: https://github.com/mpv-player/mpv/blob/v0.41.0/include/mpv/render.h

Qt's own docs for the base class say rendering happens on a dedicated thread and that the class is only functional when Qt Quick renders through OpenGL.
Source: https://doc.qt.io/qt-6/qquickframebufferobject.html

The `mpv_handle` and the render context are held by `MpvHandleManager` and `MpvResourceManager` shared pointers; `freeContext()` unsets the update callback then frees the render context and is documented "MUST be called from the Qt Render Thread", and the handle manager's destructor calls `mpv_terminate_destroy`, so the core outlives the render context as render.h requires.
Source: https://github.com/KDE/mpvqt/blob/v1.2.0/src/mpvabstractitem.h, https://github.com/KDE/mpvqt/blob/v1.2.0/src/mpvcontroller.h

### Property observation

`observeProperty` is `mpv_observe_property(mpv(), id, name, format)` with the caller's format; `MPV_EVENT_PROPERTY_CHANGE` becomes `propertyChanged(name, QVariant)` and decodes DOUBLE, STRING, INT64, FLAG and NODE (anything else arrives as an empty QVariant). `command()` builds an `mpv_node` array and calls `mpv_command_node`; `commandAsync` uses `mpv_command_node_async`; the set and get paths use `MPV_FORMAT_NODE`.
Source: https://github.com/KDE/mpvqt/blob/v1.2.0/src/mpvcontroller.cpp

Haruna observes `media-title` STRING, `time-pos`, `time-remaining`, `duration` DOUBLE, `pause` FLAG, `volume`, `aid`, `sid`, `secondary-sid`, `chapter` INT64, `track-list`, `chapter-list`, `demuxer-cache-state` NODE, `eof-reached`, `vo-configured` FLAG, `speed` DOUBLE, and connects `MpvController::propertyChanged` to `onPropertyChanged` with `Qt::QueuedConnection`, caching each value in a member and emitting a Qt signal. That is the shape a tick takes on the Qt side: `time-pos` observed as DOUBLE, delivered on the GUI thread.
Source: https://github.com/KDE/haruna/blob/master/src/mpv/mpvitem.cpp

### Qt 6.11 and mpv 0.41 compatibility

MpvQt's CMake: `project(MpvQt VERSION 1.2.0)`, `set(REQUIRED_QT_VERSION 6.5.0)`, `find_package(Libmpv)` with no version (the finder is a bare `pkg_search_module(... mpv)`), ECM 6.15.0, no KDE Frameworks. Between v1.1.1 and v1.2.0 the guard for the pre 2.0 three field `mpv_opengl_init_params` was dropped, so libmpv client API 2.0 (mpv 0.35) is the effective floor.
Source: https://github.com/KDE/mpvqt/blob/v1.2.0/CMakeLists.txt, https://github.com/KDE/mpvqt/blob/v1.2.0/cmake/FindLibmpv.cmake, https://github.com/KDE/mpvqt/blob/v1.1.1/src/mpvrenderer.cpp

Everything MpvQt uses from the render API (`MPV_RENDER_PARAM_API_TYPE`, `OPENGL_INIT_PARAMS`, `OPENGL_FBO`, `FLIP_Y`, `X11_DISPLAY`, `WL_DISPLAY`) is present in the 0.41 headers. The only client API changes since 2.0 are additions (`mpv_del_property` 2.1, `mpv_time_ns` 2.2, an ICC fix 2.4) and the 2.5 deprecation of `MPV_RENDER_PARAM_AMBIENT_LIGHT`, which MpvQt does not use.
Source: https://github.com/mpv-player/mpv/blob/v0.41.0/include/mpv/render.h, https://github.com/mpv-player/mpv/blob/v0.41.0/DOCS/client-api-changes.rst

Qt 6.11.2's documentation still ships `QQuickFramebufferObject` and calls it "a legacy class that is only present in order to enable Qt 5 applications to function without source compatibility breaks as long as they tie themselves to OpenGL". It is not marked deprecated.
Source: https://doc.qt.io/qt-6/qquickframebufferobject.html

API changes 1.1.1 to 1.2.0 that affect a subclass: `observeProperty`, `setProperty` and `command` moved from signals to `Q_INVOKABLE` functions, `commandBlocking` now takes a `QStringList` instead of a `QVariant`, `init()` became blocking, `endFile` reports the `stop` reason, and the SOVERSION went 2 to 3.
Source: https://github.com/KDE/mpvqt/compare/v1.1.1...v1.2.0, https://github.com/KDE/mpvqt/blob/v1.1.1/src/mpvabstractitem.h

Haruna 1.8.1 asks for Qt 6.8.0 and `find_package(MpvQt)` with no version, and links `MpvQt::MpvQt`; nothing in either project mentions Qt 6.10, 6.11 or mpv 0.40, 0.41.
Source: https://github.com/KDE/haruna/blob/master/CMakeLists.txt, https://github.com/KDE/haruna/blob/master/src/CMakeLists.txt

Two things a shell must copy from every MpvQt consumer: set the graphics API to OpenGL before the first window, and wait for `ready()` before the first `loadfile`.
Source: https://github.com/KDE/mpvqt/blob/v1.2.0/src/mpvabstractitem.cpp, https://github.com/KDE/haruna/blob/master/src/mpv/mpvitem.cpp

## The render API on Wayland

### vo=libmpv with the OpenGL render context

The render API supports two backends: OpenGL (`MPV_RENDER_API_TYPE_OPENGL`) and software (`MPV_RENDER_API_TYPE_SW`). The context must exist before playback starts, "Video initialization will fail if the render context was not initialized yet, or it will revert to a VO that creates its own window", and "there can be only 1 mpv_render_context at a time per mpv core".
Source: https://github.com/mpv-player/mpv/blob/v0.41.0/include/mpv/render.h

The manual lists `libmpv` as "render API for libmpv", for "libmpv direct embedding", and notes it "supports many of the options the gpu VO has". Both `libmpv` and `image` are compiled into the Arch build.
Source: https://github.com/mpv-player/mpv/blob/v0.41.0/DOCS/man/vo.rst, local check `mpv --vo=help`

The OpenGL backend of `vo=libmpv` loads GL through the caller's `get_proc_address`, refuses to continue if neither `gl->version` nor `gl->es` is set, builds a `ra_gl` context with `allow_sw = true`, locks `SwapInterval` to NULL, and wraps the caller's FBO with `ra_gl_ctx_resize(sw, fbo->w, fbo->h, fbo->fbo)` per frame. Rendering to a non zero FBO needs the FBO extension (`MPGL_CAP_FB`).
Source: https://github.com/mpv-player/mpv/blob/v0.41.0/video/out/opengl/libmpv_gl.c

Above that, `libmpv_gpu.c` creates a `gl_video` renderer (`gl_video_init`), maps `MPV_RENDER_PARAM_X11_DISPLAY` to the native resource "x11", `MPV_RENDER_PARAM_WL_DISPLAY` to "wl" and the DRM params to "drm_params_v2", and calls `gl_video_init_hwdecs(renderer, ra_ctx, hwdec_devs, true)` at context creation. This is the `vo_gpu` renderer, not libplacebo's `gpu-next`, which matters for the Vulkan decode point below.
Source: https://github.com/mpv-player/mpv/blob/v0.41.0/video/out/gpu/libmpv_gpu.c

The manual confirms the load all behaviour: for `libmpv`, which "has no on-demand loading", `--gpu-hwdec-interop=auto` is equivalent to `all`, and the value is read once when the renderer is created.
Source: https://github.com/mpv-player/mpv/blob/v0.41.0/DOCS/man/options.rst (`--gpu-hwdec-interop`)

render_gl.h's rules for the caller's GL state: mpv expects standard defaults and leaves standard defaults except viewport, scissor, blend func, clear color, the debug callback, and `GL_DITHER` (always disabled at init). On GL 2.1 objects must be created with `glGen*` so names do not clash. Qt's `QQuickFramebufferObject` runs `render()` inside the scene graph's GL context, so those rules apply to whatever custom GL the shell adds in the same context.
Source: https://github.com/mpv-player/mpv/blob/v0.41.0/include/mpv/render_gl.h

The header also states the hardware decoding requirements per platform: "Intel/Linux: EGL is required, and also the native display resource needs to be provided (e.g. MPV_RENDER_PARAM_X11_DISPLAY for X11 and MPV_RENDER_PARAM_WL_DISPLAY for Wayland)"; "nVidia/Linux: Both GLX and EGL should work"; and "Once these things are setup, hardware decoding can be enabled/disabled at any time by setting the hwdec property".
Source: https://github.com/mpv-player/mpv/blob/v0.41.0/include/mpv/render_gl.h

Qt on Wayland gives us an EGL context. The Qt Wayland page says driver support "is provided through an extension to EGL which is called EXT_platform_wayland", and qt6-base 6.11.1 installs the client integration as `/usr/lib/qt6/plugins/wayland-graphics-integration-client/libqt-plugin-wayland-egl.so`. MpvQt passes the `wl_display` (see above), so the native resource the vaapi interop needs is present.
Source: https://doc.qt.io/qt-6/wayland-and-qt.html, local check `pacman -Ql qt6-base`, https://github.com/KDE/mpvqt/blob/v1.2.0/src/mpvrenderer.cpp

### hwdec on NVIDIA and AMD

The manual: hardware decoding is off by default (`hwdec=no`); `auto` enables any whitelisted decoder, `auto-safe` and `yes` are the same as `auto`, `auto-unsafe` forces everything, and names can be mixed, for example `vaapi,auto`. Actively supported entries: `vaapi` "requires --vo=gpu, --vo=gpu-next, --vo=vaapi or --vo=dmabuf-wayland (Linux only)", `vaapi-copy`, `nvdec` "requires --vo=gpu or --vo=gpu-next (Any platform CUDA is available)", `nvdec-copy`, `drm`, `drm-copy`, `vulkan` "requires --vo=gpu-next", `vulkan-copy`. The table does not name `libmpv`, but render_gl.h says hardware decoding through the render API is "fully supported" and `libmpv_gpu.c` runs the same `gl_video` interop loader as `vo=gpu`.
Source: https://github.com/mpv-player/mpv/blob/v0.41.0/DOCS/man/options.rst (`--hwdec`), https://github.com/mpv-player/mpv/blob/v0.41.0/include/mpv/render_gl.h, https://github.com/mpv-player/mpv/blob/v0.41.0/video/out/gpu/libmpv_gpu.c

This mpv build carries the interops `vaapi`, `cuda`, `vdpau-gl`, `drmprime`, `drmprime-overlay` and `vulkan`, and the decoders `nvdec`, `vaapi` and `vulkan`.
Source: local check `mpv --gpu-hwdec-interop=help`, `mpv --hwdec=help`

NVIDIA, `nvdec`: the cuda hwdec loads libcuda at runtime (`cuda_load_functions`), tries its interops in order, and fails with "CUDA hwdec only works with OpenGL or Vulkan backends" if none initialises. The GL interop creates a `ra_tex`, gets the raw GL texture, and registers it with `cuGraphicsGLRegisterImage(..., CU_GRAPHICS_REGISTER_FLAGS_WRITE_DISCARD)`, then maps decoded CUDA arrays into it. That is a pure CUDA to GL path with no dependency on EGL or X11 resources, which is why render_gl.h says GLX and EGL both work on NVIDIA.
Source: https://github.com/mpv-player/mpv/blob/v0.41.0/video/out/hwdec/hwdec_cuda.c, https://github.com/mpv-player/mpv/blob/v0.41.0/video/out/hwdec/hwdec_cuda_gl.c

AMD, `vaapi`: the vaapi hwdec picks an interop from `dmabuf_interop_gl_init`, `dmabuf_interop_pl_init`, `dmabuf_interop_wl_init` in that order, else "VAAPI hwdec only works with OpenGL or Vulkan backends". It then opens a VA display from the native resource "x11", then "wl" (`vaGetDisplayWl`), then "drm_params_v2" (`vaGetDisplayDRM(render_fd)`), in that order, so on Wayland the `wl_display` MpvQt passes is what makes the VA display exist.
Source: https://github.com/mpv-player/mpv/blob/v0.41.0/video/out/hwdec/hwdec_vaapi.c

The GL dmabuf interop requires an OpenGL `ra`, a current EGL context (`eglGetCurrentContext()`), and the extensions `EGL_EXT_image_dma_buf_import`, `EGL_KHR_image_base`, `GL_OES_EGL_image` (GLES) or `GL_EXT_EGL_image_storage` (desktop GL), plus RG texture support; `EGL_EXT_image_dma_buf_import_modifiers` enables modifiers. A GLX context cannot satisfy this, which is the concrete reason the render API on Linux wants EGL for vaapi.
Source: https://github.com/mpv-player/mpv/blob/v0.41.0/video/out/hwdec/dmabuf_interop_gl.c

NVIDIA through nvidia-vaapi-driver: its README says it "is specifically designed to be used by Firefox for accelerated decode of web content, and may not operate correctly in other applications", that under MPV "There's no real reason to run it with mpv except for testing, as mpv already supports using nvdec directly", that the EGL backend "is broken on driver versions 525 or later" so the `direct` backend is the default, that it needs `nvidia-drm.modeset=1`, and that libva 2.20+ needs `LIBVA_DRIVER_NAME=nvidia` to load it. It decodes AV1, H.264, HEVC, VP8, VP9, MPEG-2 and VC-1 and not MPEG-4 or JPEG. The Arch package libva-nvidia-driver 0.0.17 is installed on the desktop; nothing in the shell should depend on it.
Source: https://github.com/elFarto/nvidia-vaapi-driver/blob/master/README.md, local check `pacman -Q libva-nvidia-driver`

Vulkan video decoding: mpv 0.41 "prefers Vulkan hwdec when available" and prefers non copy hwdecs before the copy variant. The manual pins `vulkan` hwdec to `--vo=gpu-next`, and `vo=libmpv` renders through `gl_video`, so with an OpenGL render context `hwdec=auto` cannot land on `vulkan` and falls through to `nvdec` or `vaapi`. The copy variants (`nvdec-copy`, `vaapi-copy`, `vulkan-copy`) copy frames back to system RAM and do not need an interop.
Source: https://github.com/mpv-player/mpv/blob/v0.41.0/RELEASE_NOTES, https://github.com/mpv-player/mpv/blob/v0.41.0/DOCS/man/options.rst (`--hwdec`), https://github.com/mpv-player/mpv/blob/v0.41.0/video/out/gpu/libmpv_gpu.c

Haruna sets `hwdec` from a user setting on both its player item and its preview item and nothing else GPU specific.
Source: https://github.com/KDE/haruna/blob/master/src/mpv/mpvitem.cpp, https://github.com/KDE/haruna/blob/master/src/mpv/mpvpreview.cpp

### Is there a Vulkan path for Qt Quick's RHI

No. render.h lists OpenGL and software as the only backends. The request "Add vulkan output to embedded rendering API (libmpv)" is issue #6575, open since 2019-03-19, labelled `core:libmpv`, `meta:developer-needed`, `meta:feature-request`, with no maintainer commitment. A draft PR, #16818 "vo_libmpv: introduce 'gpu-next' render backend", would move the render API onto libplacebo but still exposes only `MPV_RENDER_API_TYPE_OPENGL` and names Vulkan and D3D11 as "future"; reviewers asked for a redesign and it is not merged.
Source: https://github.com/mpv-player/mpv/blob/v0.41.0/include/mpv/render.h, https://github.com/mpv-player/mpv/issues/6575, https://github.com/mpv-player/mpv/pull/16818

On the Qt side, the scene graph defaults are "Direct3D 11 for Windows, Metal for macOS, OpenGL elsewhere", overridable with `QSG_RHI_BACKEND` or `QQuickWindow::setGraphicsApi()`, which "must happen before constructing the first QQuickWindow". `QQuickFramebufferObject` "is not compatible with other graphics APIs, such as Vulkan or Metal". `QQuickRhiItem` (since Qt 6.7) is the portable replacement that runs on Vulkan, Metal, D3D and OpenGL through `QRhi`, but libmpv has nothing to feed into a `QRhi` texture other than a GL FBO.
Source: https://doc.qt.io/qt-6/qtquick-visualcanvas-scenegraph-renderer.html, https://doc.qt.io/qt-6/qquickwindow.html#setGraphicsApi, https://doc.qt.io/qt-6/qquickframebufferobject.html, https://doc.qt.io/qt-6/qquickrhiitem.html

The only way to host libmpv under a Vulkan RHI today is `MPV_RENDER_API_TYPE_SW`, which renders into a caller supplied memory surface (`SW_SIZE`, `SW_FORMAT`, `SW_STRIDE`, `SW_POINTER`) and is described by the header as "extremely simple (but slow)", with colour conversion, scaling and OSD "done on the CPU, single-threaded", and "You probably don't want to use this". Decision: the Linux shell runs Qt Quick on OpenGL.
Source: https://github.com/mpv-player/mpv/blob/v0.41.0/include/mpv/render.h

## Rendering one frame offscreen

### The render API into an FBO without a window

The OpenGL backend needs a GL context current on the calling thread and a `get_proc_address`; the header says nothing about a window. `mpv_opengl_fbo.fbo` "must be either a valid FBO generated by glGenFramebuffers() that is complete and color-renderable, or 0" for the default framebuffer, with `w`, `h` always set. So an offscreen frame is a `QOpenGLContext` on a `QOffscreenSurface` plus an FBO, no window.
Source: https://github.com/mpv-player/mpv/blob/v0.41.0/include/mpv/render_gl.h

The constraint that shapes the design is "Currently, there can be only 1 mpv_render_context at a time per mpv core", and that video "will revert to a VO that creates its own window" if a core starts playback without a render context. A preview or thumbnail therefore needs its own `mpv_handle`, not the playback core.
Source: https://github.com/mpv-player/mpv/blob/v0.41.0/include/mpv/render.h

`MPV_RENDER_PARAM_BLOCK_FOR_TARGET_TIME` (default 1) makes `mpv_render_context_render` wait until the frame's target time, `MPV_RENDER_PARAM_SKIP_RENDERING` runs timing without drawing, and `mpv_render_context_update()` returns `MPV_RENDER_UPDATE_FRAME` when a frame is ready; these are the knobs for a "render exactly one frame, now" loop.
Source: https://github.com/mpv-player/mpv/blob/v0.41.0/include/mpv/render.h

Haruna's seek preview is this pattern, on screen rather than offscreen: `MpvPreview : public MpvAbstractItem` (its own core and render context), constructor sets `vo=libmpv`, observes `time-pos` and `video-params/aspect`, and sets `mute=yes`, `pause=yes`, `really-quiet=yes`, `hwdec=<setting>`, `hr-seek=<accurate preview setting>`, `aid=no`, `audio-file-auto=no`, `sid=no`, `sub-auto=no`, OSD level 0, `audio-pitch-correction=no`, `use-text-osd=no`, `audio-display=no`. It loads the file on `ready()` and moves with `setPropertyAsync("time-pos", value)`.
Source: https://github.com/KDE/haruna/blob/master/src/mpv/mpvpreview.cpp, https://github.com/KDE/haruna/blob/master/src/mpv/mpvpreview.h

For the frame currently on screen, the `screenshot-raw [<flags> [<format>]]` command returns the image in memory through the client API as a node with `w`, `h`, `stride`, `format` and a `MPV_FORMAT_BYTE_ARRAY` `data`, formats `bgr0` (default), `bgra`, `rgba`, `rgba64`; `libmpv_gpu.c` wires screenshots to `gl_video_screenshot`. That is a cheap way to grab a still from the playback core without a second core, but only at the current position.
Source: https://github.com/mpv-player/mpv/blob/v0.41.0/DOCS/man/input.rst (`screenshot-raw`), https://github.com/mpv-player/mpv/blob/v0.41.0/video/out/gpu/libmpv_gpu.c

### A headless mpv process with --vo=image

`--vo=image` writes "each frame into an image file in the current directory", named by zero padded frame number, with `--vo-image-format` (jpg default, jpeg, png, webp; this build also lists jxl and avif), `--vo-image-outdir` (default `./`), and per format quality knobs. `--frames=<number>` plays only that many frames then quits, `--start=<time>` seeks first, `--hr-seek=yes` makes that seek exact, `--vid=auto|no` and `--aid=no` pick or drop tracks. So `mpv --no-config --vo=image --vo-image-outdir=DIR --start=T --hr-seek=yes --frames=1 --no-audio --no-sub FILE` yields one image per call.
Source: https://github.com/mpv-player/mpv/blob/v0.41.0/DOCS/man/vo.rst, https://github.com/mpv-player/mpv/blob/v0.41.0/DOCS/man/options.rst (`--frames`, `--start`, `--hr-seek`, `--aid`, `--vid`), local check `mpv --list-options`

Encoding mode is the other headless route: `--o=<filename>` enables it, `--of` picks the libavformat muxer, `--ovc` the codec, `--ofopts` muxer options. Encoding options are among those that must be set before `mpv_initialize()`, so this is a process level choice, not something to flip on a live core.
Source: https://github.com/mpv-player/mpv/blob/v0.41.0/DOCS/man/encode.rst, https://github.com/mpv-player/mpv/blob/v0.41.0/include/mpv/client.h

### How thumbfast does it

thumbfast spawns a second mpv as a subprocess with `--no-config --msg-level=all=no --idle --pause --keep-open=always --really-quiet --no-terminal --load-scripts=no --osc=no --ytdl=no --load-stats-overlay=no --load-osd-console=no --load-auto-profiles=no --edition=<e> --vid=<v> --no-sub --no-audio --start=<t> --hr-seek=no|yes --ytdl-format=worst --demuxer-readahead-secs=0 --demuxer-max-bytes=128KiB --vd-lavc-skiploopfilter=all --vd-lavc-software-fallback=1 --vd-lavc-fast --vd-lavc-threads=2 --hwdec=auto|no --vf=<crop,scale=w:h,pad,format=bgra> --sws-scaler=fast-bilinear --video-rotate=<r> --ovc=rawvideo --of=image2 --ofopts=update=1 --o=<thumbnail path> --input-ipc-server=<socket> -- <path>`. It is the encoding path, not `--vo=image`: `image2` with `update=1` rewrites the same file, `rawvideo` plus `format=bgra` makes that file a bare BGRA buffer.
Source: https://github.com/po5/thumbfast/blob/master/thumbfast.lua

It then drives the child over the IPC socket with `async seek <t> absolute+keyframes` on a 3/60 s timer while the cursor moves and `absolute+exact` once it settles, and reads the frame by moving the output file aside, checking its size against `w * h * 4`, renaming it to `.bgra`, and showing it with `overlay-add <id> <x> <y> <file> 0 bgra <w> <h> <4*w>`. Defaults: 200 by 200 max, `hwdec` off, network and audio off, `spawn_first` off, `quit_after_inactivity` 0.
Source: https://github.com/po5/thumbfast/blob/master/thumbfast.lua, https://github.com/po5/thumbfast/blob/master/README.md

Decision this enables: seek hover previews belong in the shell as a second `MpvAbstractItem` (Haruna's `MpvPreview`, no process, no file). Library thumbnails belong in the core as a job that runs a child mpv, either thumbfast's rawvideo recipe when a stream of frames is wanted or `--vo=image --frames=1` for one still, because a process crosses the bridge as plain data and needs no GL context in Rust.

## Loading the user's mpv.conf and scripts from libmpv

### What libmpv changes by default

client.h: a libmpv core differs from the command line player in four ways. The terminal is never touched (`--no-terminal`). "No config files will be loaded. This is roughly equivalent to using --config=no. Since libmpv 1.15, you can actually re-enable this option, which will make libmpv load config files during mpv_initialize(). If you do this, you are strongly encouraged to set the "config-dir" option too. (Otherwise it will load the mpv command line player's config.)" with the example `config-dir=/my/path` then `config=yes` then `mpv_initialize()`. Idle mode is on. "Disable parts of input handling." The full list is `mpv --show-profile=libmpv`.
Source: https://github.com/mpv-player/mpv/blob/v0.41.0/include/mpv/client.h

On this mpv the `libmpv` profile is `config=no`, `idle=yes`, `terminal=no`, `input-terminal=no`, `osc=no`, `input-default-bindings=no`, `input-vo-keyboard=no`, `input-media-keys=no`, `media-controls=no`.
Source: local check `mpv --show-profile=libmpv`

client.h names the options that only work before `mpv_initialize()`: `config`, `config-dir`, `input-conf`, `load-scripts`, `script`, `player-operation-mode`, `input-app-events`, and all encoding options.
Source: https://github.com/mpv-player/mpv/blob/v0.41.0/include/mpv/client.h

### The options

`--config-dir=<path>`: "Force a different configuration directory. If this is set, the given directory is used to load configuration files, and all other configuration directories are ignored", including the global directory, per user directories and `MPV_HOME`; cache and state paths keep their auto detection; `--no-config` takes precedence. It is marked "not in config files".
Source: https://github.com/mpv-player/mpv/blob/v0.41.0/DOCS/man/options.rst (`--config-dir`), local check `mpv --list-options`

`--no-config`: "Do not load default configuration or any user files", covering user and system `mpv.conf` and `input.conf`, resume playback files and cache files; files named through `--include` are still loaded. `--include=<file>`: "Specify configuration file to be parsed after the default ones."
Source: https://github.com/mpv-player/mpv/blob/v0.41.0/DOCS/man/options.rst (`--no-config`, `--include`)

`--load-scripts=<yes|no>`: "If set to no, don't auto-load scripts from the scripts configuration subdirectory (usually ~/.config/mpv/scripts/)", default yes. `--script=<file>` and `--scripts=a.lua:b.lua` load specific scripts; `--script-opts` passes options to them.
Source: https://github.com/mpv-player/mpv/blob/v0.41.0/DOCS/man/options.rst (`--load-scripts`, `--scripts`)

The default paths: `~/.config/mpv` (overridden by `$XDG_CONFIG_HOME/mpv`, then `$MPV_HOME`), `~/.config/mpv/mpv.conf`, `input.conf`, `fonts.conf`, `fonts/`, `scripts/` ("loaded as if they were passed to the --script option ... in alphabetical order"), `script-opts/`, and watch later files in `~/.local/state/mpv/watch_later/` (`$XDG_STATE_HOME`, `$MPV_HOME`). `/etc/mpv/mpv.conf` is the system file on most distributions.
Source: https://github.com/mpv-player/mpv/blob/v0.41.0/DOCS/man/mpv.rst (FILES)

### What MpvQt and Haruna do

MpvQt does not enable config loading. Its controller calls `mpv_initialize()` first and only then sets `include=<ConfigLocation>/mpvqt/mpvqt.conf`, and its README says that file "has to be manually created and is a regular mpv config file" whose settings "apply to all applications using MpvQt" and can be overridden by the application. A merge request to "Set mpv profile libmpv and enable config by default" was closed without merging.
Source: https://github.com/KDE/mpvqt/blob/v1.2.0/src/mpvcontroller.cpp, https://github.com/KDE/mpvqt/blob/v1.2.0/README.md, https://invent.kde.org/libraries/mpvqt/-/merge_requests/10

Consequence for the shell: because the base constructor has already initialised the core, a subclass cannot set `config`, `config-dir`, `load-scripts`, `scripts` or `input-conf`. What still works after init: `include=<path>` (this is how MpvQt loads its own file), `mpv_load_config_file(ctx, absolute_path)`, which "sets every entry in the config file's default section as if mpv_set_option_string() is called" and ignores unknown options, and the `load-script <filename>` command, "similar to the --script option", which returns the new script's `client_id`. Loading the user's mpv.conf before init means patching MpvQt or replacing it.
Source: https://github.com/KDE/mpvqt/blob/v1.2.0/src/mpvabstractitem.cpp, https://github.com/mpv-player/mpv/blob/v0.41.0/include/mpv/client.h, https://github.com/mpv-player/mpv/blob/v0.41.0/DOCS/man/input.rst (`load-script`)

Haruna never loads the user's mpv.conf. Its `initProperties()` sets `reset-on-next-file=ab-loop-a,ab-loop-b`, `vo=libmpv`, `pause`, `hwdec`, `volume`, `volume-max=100`, `ytdl-format`, `sub-auto=fuzzy`, `sub-use-margins`, `sub-ass-force-margins`, `sub-font`, `sub-font-size`, colours, border, bold, italic, replaygain options, `sub-file-paths`, `screenshot-template`, `screenshot-format`, `audio-client-name=haruna`, `alang`, and `sid=no` then `slang` when auto select is on; a commented out line shows `terminal` was once tied to a logging setting. Users get an escape hatch instead: "custom commands" from Haruna's own config file run at startup through `userCommand()`, which splits the string with `KShell::splitArgs` and calls `commandBlocking`.
Source: https://github.com/KDE/haruna/blob/master/src/mpv/mpvitem.cpp

### What a hosting app disables

With the libmpv profile, `osc`, `input-default-bindings`, `input-vo-keyboard` and the terminal are already off. If the shell then includes the user's mpv.conf, a user line like `osc=yes` comes back, so re-set `osc=no` after the include. The render API "does not include keyboard or mouse input directly", so mpv's key bindings only fire if the shell sends `keypress` commands; `--input-default-bindings=no` also disables bindings that scripts register with `mp.add_key_binding` (not `mp.add_forced_key_binding`).
Source: https://github.com/mpv-player/mpv/blob/v0.41.0/include/mpv/client.h, https://github.com/mpv-player/mpv/blob/v0.41.0/DOCS/man/options.rst (`--osc`, `--input-default-bindings`, `--input-vo-keyboard`), https://github.com/mpv-player/mpv/blob/v0.41.0/DOCS/man/input.rst (`keypress`)

Built in scripts have their own switches, all default yes unless noted: `--load-stats-overlay` (the `i` key), `--load-console`, `--load-commands` (the backtick key), `--load-select` (`g` prefix), `--load-context-menu` (yes where no native menu exists; new in 0.41), `--load-positioning`, `--load-auto-profiles` (default auto). `--load-osd-console` is a deprecated alias for `--load-console`. thumbfast's child turns off `load-scripts`, `osc`, `ytdl`, `load-stats-overlay`, `load-osd-console` and `load-auto-profiles`; `ytdl=no` also stops libmpv from spawning the youtube-dl wrapper, which client.h lists among the sub processes mpv may start.
Source: https://github.com/mpv-player/mpv/blob/v0.41.0/DOCS/man/options.rst (`--load-*`), local check `mpv --list-options`, https://github.com/mpv-player/mpv/blob/v0.41.0/RELEASE_NOTES, https://github.com/po5/thumbfast/blob/master/thumbfast.lua, https://github.com/mpv-player/mpv/blob/v0.41.0/include/mpv/client.h

Resume handling: `--resume-playback` defaults to yes and restores from `watch_later`; `--no-config` blocks those files, so today's libmpv never reads them, but a shell that turns config back on would. `--save-position-on-quit` is off by default. The core owns the resume point, so set both to no explicitly.
Source: https://github.com/mpv-player/mpv/blob/v0.41.0/DOCS/man/options.rst (`--resume-playback`, `--save-position-on-quit`, `--no-config`)

## Subtitle options and track selection persistence

### The four subtitle options

`--sub-font-size=<size>`: "The unit is the size in scaled pixels at a window height of 720. The actual pixel size is scaled with the window height". Default 38.
Source: https://github.com/mpv-player/mpv/blob/v0.41.0/DOCS/man/options.rst (`--sub-font-size`)

`--sub-pos=<0-150>`: vertical position "in % of the screen height. 100 is the original position, which is often not the absolute bottom of the screen". Warning: text subtitles "may be cut off if the value of the option is above 100. This is a libass restriction", it "affects ASS subtitles as well", and `--sub-margin-y` "can achieve this in a better way".
Source: https://github.com/mpv-player/mpv/blob/v0.41.0/DOCS/man/options.rst (`--sub-pos`)

`--sub-scale=<0-100>`: "Factor for the text subtitle font size (default: 1)", with the note "This affects ASS subtitles as well, and may lead to incorrect subtitle rendering. Use with care, or use --sub-font-size instead." `--sub-scale-signs` (default no) restricts it to dialogue on a best effort basis; `--sub-scale-by-window` and `--sub-scale-with-window` (both default yes) decide whether size follows the window or the video, "plain text subtitles only (or ASS if --sub-ass-override is set high enough)".
Source: https://github.com/mpv-player/mpv/blob/v0.41.0/DOCS/man/options.rst (`--sub-scale`, `--sub-scale-signs`, `--sub-scale-by-window`, `--sub-scale-with-window`)

`--sub-ass-override=<no|yes|scale|force|strip>`: `no` renders as scripted; `yes` applies the `--sub-ass-*` overrides; `scale` is `yes` plus `--sub-scale` and is the default; `force` "also force all --sub-* options. Can break rendering easily"; `strip` removes all ASS tags and styles. `--secondary-sub-ass-override` defaults to `strip`. `--sub-ass-style-overrides` can target individual style fields (`FontName=Arial`, `PlayResY=768`) at the same risk.
Source: https://github.com/mpv-player/mpv/blob/v0.41.0/DOCS/man/options.rst (`--sub-ass-override`, `--sub-ass-style-overrides`), local check `mpv --list-options` (defaults `scale`, 38, 100, 1)

What that means for an anime library, where most subtitle tracks are ASS: with the default override level a user "subtitle size" setting is `sub-scale`; `sub-font-size` and `sub-pos` only reach ASS tracks under `force`, so expose `force` as an opt in labelled as risky, and keep `sub-margin-y` in mind for "move subtitles up". All four are plain properties that Haruna sets from its settings at init and that can be changed and observed at runtime.
Source: https://github.com/KDE/haruna/blob/master/src/mpv/mpvitem.cpp

### Track selection

`--aid=<ID|auto|no>` and `--sid=<ID|auto|no>` select by id, `auto` picks the default, `no` disables. The manual warns: "The track selection properties will return the option value outside of playback (as expected), but during playback, the affective track selection is returned. For example, with --aid=auto, the aid property will suddenly return 2 after playback initialization", and that this behaviour "tends to change around with each mpv release".
Source: https://github.com/mpv-player/mpv/blob/v0.41.0/DOCS/man/options.rst (`--aid`, `--sid`)

`--alang` and `--slang` are prioritised string lists "as IETF language tags. Equivalent ISO 639-1 two-letter and ISO 639-2 three-letter codes are treated the same. The first tag in the list that matches track's language in the file will be used. A track that matches more subtags will be preferred", for example `--slang=pt-BR`. `--track-auto-selection=no` turns off automatic selection entirely. Related knobs: `--subs-with-matching-audio` (default yes), `--subs-match-os-language` (default yes, "completely ignored" when `--slang` is set), `--subs-fallback` (default `default`), `--subs-fallback-forced`.
Source: https://github.com/mpv-player/mpv/blob/v0.41.0/DOCS/man/options.rst (`--alang`, `--slang`, `--track-auto-selection`, `--subs-*`)

Haruna's persistence: it sets `alang` from settings, sets `sid=no` then `slang` when auto selection is on, and after `track-list` arrives it applies a "preferred track" id from settings only if `m_audioTracksModel->hasTrackId(value)`, else falls back to `auto`; `-1` from the GUI means `no`. It observes `aid`, `sid` and `secondary-sid` as INT64 to mirror the effective selection.
Source: https://github.com/KDE/haruna/blob/master/src/mpv/mpvitem.cpp

mpv's own watch_later persistence saves, by default, `start`, `aid`, `vid`, `sid`, `secondary-sid`, `sub-delay`, `sub-pos`, `sub-scale`, `sub-use-margins`, `sub-ass-force-margins`, `sub-ass-override`, `volume`, `mute`, `speed` and more per file hash, and the manual's own example is `--watch-later-options-remove=sid` to stop restoring the subtitle choice. That is a per file, per machine store outside the core, which is one more reason to keep `resume-playback=no` and hold the user's choice in the core.
Source: local check `mpv --help=watch-later-options`, https://github.com/mpv-player/mpv/blob/v0.41.0/DOCS/man/options.rst (`--watch-later-options`)

Decision this enables: the core stores a language preference per user and per series as `alang` and `slang` lists, and a chosen track as language plus title plus type rather than as an id; the shell sets the lists before `loadfile`, reads `track-list` on `fileLoaded`, and sets `aid` or `sid` to the id whose language and title match. New in 0.41, `sub-add` and `audio-add` accept `forced` and `default` flags for external tracks.
Source: https://github.com/mpv-player/mpv/blob/v0.41.0/DOCS/interface-changes.rst

## Not verified

- Nothing was run against a real Qt Quick window: neither MpvQt nor Haruna was built or launched here, and nvdec and vaapi inside a Qt EGL context under Hyprland were not exercised. The AMD laptop was not checked at all (its mesa VA driver and EGL extensions are assumed from the mpv source requirements).
- Whether `include=<path>` set after `mpv_initialize()` applies every option class the user might have in mpv.conf; init only options such as `load-scripts` inside that file will be ignored, and the manual does not spell out what else is.
- Whether `load-script` waits for script initialisation; input.rst says that behaviour "changed multiple times, and the future behavior is left undefined".
- The Qt 6.11 render loop in use on Wayland (threaded or basic) and MpvQt's behaviour under the basic loop were not checked.
- PR #16818 is a moving target; its status was read on 2026-09-03.
- The nvidia-vaapi-driver README was read at master; the installed Arch package is 0.0.17 and may differ.
- mpv.io/manual/stable renders the same rst files but its per option anchors were not verified, so this note cites the rst sources at the v0.41.0 tag.
