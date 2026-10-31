# Page loads: where the time and the memory go, and the plan to make them cheap

Measured on 2026-10-31 for #287 against a copy of the owner's real library (61 series, 1,550
cached images, 27 MB database), release build, offscreen at 2560x1440, banditbox (12 cores,
RTX 3090, 32 GB). Every number below comes from the Developer tab this PR adds; `Time every
page` reproduces them in the app. Another build ran on the box during some runs, so treat
single numbers as plus or minus 20%; the ratios held across every run.

## What one page load does today

A click on the rail runs this, in order, all of it on the Qt (GUI) thread:

1. `nav.changed`, then `Qt.callLater(loadPageNow)`: the old page is destroyed.
2. `Loader.setSource` compiles nothing (the QML is precompiled into the binary) but creates
   every object the page file and its components declare, synchronously
   (`asynchronous: false`). For each object: a heap allocation for the C++ item, its private
   data, its property bindings; each binding is evaluated once to give the first value.
3. Inside that, `Component.onCompleted` calls the core through the Door: the Rust core
   answers from SQLite (or its kept snapshot), the answer is walked to a `serde_json::Value`,
   then to a `QJsonObject`, then QML reads it as JavaScript objects.
4. Every `Repeater` builds one delegate per row of that answer, whether the row is on screen
   or not.
5. Only now can the render thread sync the scene graph and paint the first frame.
6. Each `Image` queues its file on Qt's pixmap reader, which is **one thread**. Posters decode
   one after another; the page is "settled" when the last one lands.

The window shows nothing new between the click and step 5. That gap is "Built" below.

## Measured

Median of three visits per page, images on (images off in brackets where it differs).
Data is the JSON the core handed QML for that load.

| Page | Built | First frame | Settled | Items | Images (out of view) | Decodes, decode time | Pixels decoded | Data |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| Home | 466 ms (363) | 569 ms | 577 ms (484) | 6,990 | 65 (36) | 34, 182 ms | 13.9 MB | 167 KB |
| Library | 582 ms | 693 ms | 699 ms | 8,887 | 77 (46) | 26, 158 ms | 8.0 MB | 101 KB |
| Airing | 512 ms | 512 ms | 669 ms | 12,762 | 203 (162) | 197, 321 ms | 53.5 MB | 101 KB |
| Feed | 293 ms | 424 ms | 471 ms | 8,321 | 61 (17) | 54, 318 ms | 16.6 MB | 94 KB |
| Watching | 37 ms | 94 ms | 138 ms | 592 | 8 (0) | 2, 3 ms | 0.4 MB | 4 KB |
| Metadata | 157 ms | 207 ms | 447 ms (254) | 3,569 | 86 (16) | 61, 315 ms | 3.6 MB | 107 KB |
| Downloads | 424 ms | 825 ms | 841 ms | 11,381 | 96 (80) | 61, 374 ms | 18.0 MB | 255 KB |
| Settings | 136 ms | 290 ms | 298 ms | 1,067 | 8 (0) | 0, 0 ms | 0.0 MB | 504 KB |
| Series | 56 ms | 74 ms | 157 ms | 1,047 | 10 (0) | 2, 20 ms | 9.1 MB | 7 KB |

Images counts posters, banners and portraits, not icons. "Out of view" is how many of them sit
outside the window, built and decoded for nothing until a scroll.

Peak memory over the tour: 782 MB. The process sits at 650 to 800 MB whatever page is up.

### Is it the images?

No, not for the wait after a click. Turning images off moves Built by 0 to 15%. With images
off the tour still takes 400 to 700 ms on Home, Library, Airing and Downloads. Images matter
in three other places:

- **Settle time.** A first visit decodes 26 to 197 posters on the single reader thread:
  150 to 620 ms of decoding after the page is already up. That is the pop-in.
- **Memory.** Decoded pixels are 4 bytes each: a 225x338 poster is 0.3 MB, a full banner
  3 MB. Airing alone holds 56 MB of decoded posters, and 162 of its 203 posters are outside
  the window.
- **Two full-size decodes per visit.** The banner on Home and on a series page has no
  `sourceSize` (`HomePage.qml:620`, `SeriesPage.qml:469`), and `PosterTone` decodes the whole
  poster into a canvas to average its colour (`PosterTone.qml`), though a C++ 8x12 path for
  exactly this already exists and is unused (`Fmt.posterTone`).

### What the wait after a click is made of

The page's item count predicts Built almost exactly: about 50 to 60 µs per item.

| Page | Items | Built |
|---|---:|---:|
| Watching | 592 | ~35 to 90 ms |
| Series | 1,047 | ~60 to 110 ms |
| Metadata | 3,569 | ~175 to 390 ms |
| Home | 6,990 | ~380 to 470 ms |
| Feed | 8,321 | ~250 to 490 ms |
| Library | 8,887 | ~540 to 620 ms |
| Downloads | 11,381 | ~360 to 790 ms |
| Airing | 12,762 | ~450 to 700 ms |

Measured cost of one object of each kind (created 500 to 4,000 times, release build):

| Object | µs each |
|---|---:|
| `Card` (one library card with everything in it) | 6,800 |
| `Corner` (plain) | 151 |
| `Label` (the themed Text) | 66 |
| `Icon` | 64 |
| `Tooltip` | 45 |
| `Image` with a cached poster | 42 |
| `Text` (plain) | 28 |
| `Loader`, inactive | 12 |
| `Item` with a `HoverHandler` | 11 |
| `MouseArea`, `Rectangle`, `Image` without source, `Item` | 8 to 10 |

A card costs 6.8 ms, 700 bare Rectangles' worth. The x-ray says why. On Library
(77 cards) the page holds 2,157 Rectangles, 1,636 Items, 1,324 Labels, 894 MouseAreas,
**804 Tooltips** and **434 PosterTags**: each card builds seven tags and hides six
(`Card.qml:286-302`), and a tooltip for every hoverable thing whether anyone hovers it or not.

Three more costs hide inside every one of those objects:

- **Bindings run as JavaScript.** `qmllint --compiler` says 232 bindings in `Card.qml`, 161 in
  `LibraryPage.qml` and 358 in `HomePage.qml` cannot be compiled to C++ ahead of time, so the
  engine interprets them. The reasons, in Card: 91 reach a name through the context chain
  (`theme`, `frame`, `root` from a nested component), 80 read a property of an untyped
  object, 57 read fields out of `property var item`, which is a QVariant. There are 2,216
  `theme.` references in the shell.
- **Theme is a QML object reached by id from Main**, not a singleton, so no binding that
  touches a colour or a size can be typed.
- **Data crosses three times.** Rust struct to `serde_json::Value`, to `QJsonObject`, to
  JavaScript. For ListLibrary that is 7 ms core, 2 ms of walks, and 11 ms for QML to receive
  the `QJsonObject` from the invokable (measured: 20 calls of a cached ListLibrary, 11 ms
  each). JSON.parse of the same 101 KB as text takes 4.7 ms.

### Data per page

| Page | Calls | Data | Where it goes |
|---|---:|---:|---|
| Settings (Library tab) | 5 | 496 KB | `RecentEvents(2000)` is 383 KB, read to find the last scan; `ListMetadata` 107 KB |
| Downloads | 9 | 256 KB | `ListLibrary` three times (153 KB), `RecentEvents(400)` 100 KB for the Today column |
| Home | 9 | 167 KB | `ListLibrary` 101 KB plus eight `GetSeries` (66 KB) one after another |
| Metadata | 1 | 107 KB | every series as a full row |
| Library, Airing | 1 to 2 | 100 KB | about 1.6 KB per card, 60 fields; a card draws about 10 |
| Feed | 1 | 94 KB | every series with a file, no window |
| Series | 2 | 7 KB | already lean |
| Watching | 3 | 4 KB | already lean |

The read cache that should make the second page free never hits. In 39 seconds of the tour
the core sent 58 events; 13 were `TorrentsRead` at 85 KB each (1.1 MB), one every 3 seconds
while a library torrent downloads (`downloads/mod.rs:2053-2072`). Each is a Debug event, yet
each moves the state generation (`events.rs`), which empties the Door's read cache and the
core's kept snapshot. Not one page load in the tour got a cache hit.

### Memory

The kernel's map of the process at the end of the tour (795 MB resident):

| Held by | MB |
|---|---:|
| Anonymous mappings: malloc arenas of other threads, the JS heap, decoded images | 312 |
| The main malloc heap | 171 |
| NVIDIA driver mappings | 84 |
| Other libraries and files | 47 |
| Shared buffers with the GPU and the compositor (memfd) | 43 |
| The anibeam binary | 38 |
| Qt libraries | 34 |
| Fonts | 5 |
| mpv and ffmpeg | 5 |

The three largest anonymous regions are 64, 63 and 57 MB, the size of glibc's per-thread
malloc arenas: memory a thread freed that the allocator kept.

**A third of it is garbage.** After visiting six pages the app held 742 MB. Running the JS
collector gave back 31 MB; then releasing the scene graph's rebuildable caches and
`malloc_trim` took it to 472 MB, heap from 536 to 266 MB. 270 MB, 36% of the process, was
memory nothing was using. The Developer tab's "Free now" reproduces this.

## The plan

Ordered by what it buys per hour of work. Each step has a number to hit and the Developer tab
tells whether it did.

### 1. Build only what is on screen (Built: 500 ms to under 50 ms)

The single biggest lever. Library, Feed, Airing, Watching, Metadata and Downloads put a
`Repeater` inside a `Flickable`, so every card exists from the first frame.

- Move each poster grid to a `GridView` or `ListView` (sections as a `ListView` of rows) with
  `reuseItems: true` and a `cacheBuffer` of about one screen. Library goes from 8,887 items to
  roughly the 20 to 30 cards a 1440 px window shows, plus one screen of buffer: about
  1,500 items, about 80 ms.
- The page shell (title, filters, the first row) first, the rest one frame later: a
  `Loader { asynchronous: true }` for everything under the fold, so the first frame after a
  click lands within 16 ms.
- Keep the last three pages alive and hidden instead of destroying them (`StackLayout`, or a
  Loader per page that is kept). A revisit costs zero build and zero data. With virtualised
  pages the memory for a kept page is small.

### 2. Make each card cheap (6.8 ms to under 0.5 ms)

- Tags: one `Repeater` over the tags that apply, not seven `PosterTag`s with six hidden.
- Tooltips: one tip owned by the frame, fed by the card's existing `HoverHandler`, instead of
  a `Tooltip` item per hoverable thing. Removes 600 to 1,100 objects a page.
- Countdown, progress cubes, the delete affordance: `Loader`s that activate on need.
- Plain `Rectangle` instead of `Corner` wherever no Shape is needed (151 µs against 10 µs).
- Label at 66 µs against Text at 28 µs: the role functions (`roleFamily`, `roleAxes`,
  `roleFeatures`) run per Label per binding; precompute the five roles as constant font
  objects on the theme and bind `font: theme.ledgerFont`.
- Target: a card under 40 objects. Measure it with the x-ray.

### 3. Let the compiler compile (every binding: JS to C++)

- Theme becomes a QML singleton (`pragma Singleton` Tokens, registered in the module), read
  as `Theme.space(...)`, so the compiler knows its type. 2,216 references change mechanically.
- Cards take typed data: `required property string title`, `required property int
  progress`, and so on, filled from model roles, not `property var item`.
- No reaching through ids from inside components (`frame`, `root` from an inline
  component): pass what is needed as typed properties.
- Gate in CI: `qmllint --compiler warning` must not grow; target zero in Card and the pages.

### 4. Send only what the page draws (100 to 500 KB to a few KB)

- **A view record per surface.** `CardView` carries the ten things a card draws: id, title,
  poster key, a flags word (new, downloading, on hold, dropped, hidden), progress, episode
  count, next airing time, score. About 120 bytes against 1.6 KB.
- **Windows, not lists.** The page asks for counts and the first screen (`offset`, `limit`),
  the view asks for more as it scrolls. First paint of Library: 30 cards x 120 B, about
  4 KB.
- **Patches, not rereads.** `SeriesChanged` carries ids and the fields that moved; the page
  model applies them in place (`RecordModel.patch` already exists and no page uses it). No
  page rereads the library on an event.
- **Rows straight into a model.** Pages sit on `RecordModel` (or a C++ model per surface)
  whose roles are read lazily by the delegates on screen, so no reply is ever converted to
  JavaScript as a whole. The `serde_json::Value` then `QJsonObject` double walk goes too.
- **Specific calls for specific needs.** Settings asks `LastScan` (100 bytes) instead of 2,000
  events. Home asks one `HomeView` instead of `ListLibrary` plus eight `GetSeries`. Downloads
  reads the library once, and Today asks the core for today's download lines only.
- Budget: no page load over 16 KB from the core; a revisit 0 KB.

### 5. Images that cost nothing to show

- **Thumbnails at display size, written once.** When the core fetches a poster it also
  writes a 225x338 copy (and a 450x675 one for HiDPI). A 15 KB file decodes in about a
  tenth of the time of a 60 to 220 KB original.
- **Compressed textures.** Qt Quick shows `.ktx` files directly: the GPU reads the
  compressed blocks, nothing is decoded on the CPU and nothing is held as RGBA in RAM. A BC7
  or ETC2 poster at 225x338 is 76 KB of GPU memory against 304 KB; BC1 is 38 KB. This is the
  step that takes image decode time to zero.
- **More than one decoder.** Qt's pixmap reader is one thread. An async image provider with a
  `QThreadPool` decodes on all cores: 26 posters at 6 ms each settle in about 20 ms on 8
  threads instead of 150 ms.
- **Placeholders that arrive with the data.** The poster's tone colour (already computed
  at 8x12) in the card record, painted instantly, so a decode never shows as a hole.
- Banners get a `sourceSize` of the window width; `PosterTone` uses the C++ path.
- The poster pool (60 decoded posters held for the session) goes once thumbnails and
  virtualised grids make a cold poster cheap.

### 6. A quiet event pipe

- The generation moves on a change of state the store saw, never on a read. Debug events
  such as `TorrentsRead` stop emptying the caches.
- `TorrentsRead` is sent when something changed, as the ids that changed, not 85 KB of every
  torrent every 3 seconds; the Downloads page pulls detail.
- `SeriesChanged` stops carrying whole cards and stops being written whole into the events
  table.
- The shell subscribes by kind: an event no page listens for never crosses to the Qt thread.
- `Main.qml` and `Frame.qml` stop rereading the library and the download client on every
  `SeriesChanged` while no page that needs them is open.

### 7. Memory

- **Today, in an afternoon:** after a navigation settles and the app is idle, run the JS
  collector, `QQuickWindow::releaseResources()` and `malloc_trim(0)`. That alone measured
  742 MB down to 472 MB. Capping glibc's arenas (`M_ARENA_MAX` of 2 to 4 through `mallopt` at
  start) stops the 64 MB per-thread pools from forming at all.
- Find out what the 450 to 550 MB of heap is before cutting it: `heaptrack` on a tour, and the
  JS heap with `QV4_MM_STATS=1`. The Developer tab's "Free now" says how much is garbage the
  allocator kept.
- Virtualised pages and lean records remove most of the per-page JS objects and items.
- Compressed textures remove the CPU copy of decoded images.
- After a navigation settles, run the JS collector and `malloc_trim` once the app is idle.

### 8. Keep it fast

- `Time every page` runs in CI offscreen against a fixture library, and each page has a
  budget: items, KB from the core, Built. A PR that breaks a budget fails.
- The Developer tab stays in every build behind its switch, so a user's slow page can be
  measured on their machine and pasted into an issue with `Copy report`.

## Targets

| Page | Built now | Built target | Data now | Data target | Items now | Items target |
|---|---:|---:|---:|---:|---:|---:|
| Library | ~600 ms | < 50 ms | 101 KB | < 6 KB | 8,887 | < 1,500 |
| Airing | ~550 ms | < 50 ms | 101 KB | < 6 KB | 12,762 | < 1,500 |
| Home | ~430 ms | < 50 ms | 167 KB | < 8 KB | 6,990 | < 1,500 |
| Downloads | ~550 ms | < 60 ms | 256 KB | < 10 KB | 11,381 | < 2,000 |
| Feed | ~370 ms | < 50 ms | 94 KB | < 6 KB | 8,321 | < 1,500 |
| Settings | ~200 ms | < 40 ms | 496 KB | < 4 KB | 1,067 | same |
| Any revisit | as above | ~0 ms | as above | 0 KB | | |
| Memory | 650 to 800 MB | < 300 MB | | | | |
