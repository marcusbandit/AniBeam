//! Poster bytes from the store, decoded and cut to a thumbnail off the UI thread, with the
//! poster's own hue for the bands that sit behind it.

use std::cell::{Cell, RefCell};
use std::collections::{BTreeMap, HashMap, VecDeque};
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::rc::Rc;
use std::sync::{Condvar, Mutex, OnceLock};
use std::time::{Duration, Instant};

use image::RgbaImage;
use image::imageops::FilterType;
use slint::{Rgba8Pixel, SharedPixelBuffer};

/// Three times the largest poster on screen (a series page band at 3x density), cut to 2:3.
const WIDTH: u32 = 360;
const HEIGHT: u32 = 540;

pub struct Decoded {
    pub buffer: SharedPixelBuffer<Rgba8Pixel>,
    /// The poster's dominant hue, or `None` for a grey one.
    pub tone: Option<[u8; 3]>,
}

pub fn decode(bytes: &[u8]) -> Option<Decoded> {
    let image = image::load_from_memory(bytes).ok()?;
    let image = image
        .resize_to_fill(WIDTH, HEIGHT, FilterType::Triangle)
        .to_rgba8();
    let tone = tone(&image);
    Some(Decoded {
        buffer: SharedPixelBuffer::clone_from_slice(image.as_raw(), image.width(), image.height()),
        tone,
    })
}

/// The desktop's PosterTone: on a 24x36 copy, every pixel with some colour votes for one of
/// 24 hue bins with the square of its chroma, and the heaviest bin's average colour wins,
/// held between 30% and 70% lightness so a band made from it never goes black or white.
fn tone(image: &RgbaImage) -> Option<[u8; 3]> {
    const BINS: usize = 24;
    let small = image::imageops::resize(image, 24, 36, FilterType::Triangle);
    let mut weight = [0f32; BINS];
    let mut sum = [[0f32; 3]; BINS];
    for p in small.pixels() {
        let [r, g, b] = [p[0], p[1], p[2]].map(|c| f32::from(c) / 255.0);
        let max = r.max(g).max(b);
        let min = r.min(g).min(b);
        let chroma = max - min;
        if chroma < 0.12 || max < 0.18 {
            continue;
        }
        let sector = if max == r {
            ((g - b) / chroma).rem_euclid(6.0)
        } else if max == g {
            (b - r) / chroma + 2.0
        } else {
            (r - g) / chroma + 4.0
        };
        let bin = ((sector * 60.0 / (360.0 / BINS as f32)) as usize).min(BINS - 1);
        let w = chroma * chroma;
        weight[bin] += w;
        for (acc, c) in sum[bin].iter_mut().zip([r, g, b]) {
            *acc += c * w;
        }
    }
    let (best, &w) = weight
        .iter()
        .enumerate()
        .max_by(|a, b| a.1.total_cmp(b.1))?;
    if w <= 0.0 {
        return None;
    }
    let [r, g, b] = sum[best].map(|c| c / w);
    Some(clamp_lightness([r, g, b], 0.3, 0.7).map(|c| (c * 255.0).round() as u8))
}

fn clamp_lightness([r, g, b]: [f32; 3], low: f32, high: f32) -> [f32; 3] {
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let l = (max + min) / 2.0;
    let target = l.clamp(low, high);
    if (target - l).abs() < f32::EPSILON {
        return [r, g, b];
    }
    // Move toward black or white by the share that brings the lightness to the bound.
    if target > l {
        let t = (target - l) / (1.0 - l);
        [r, g, b].map(|c| c + (1.0 - c) * t)
    } else {
        let t = target / l;
        [r, g, b].map(|c| c * t)
    }
}

/// A poster file decoded for the screens: the picture and its hue.
#[derive(Clone)]
pub struct Poster {
    pub image: slint::Image,
    pub tone: Option<[u8; 3]>,
}

/// How many decoded posters are kept for a page that asks again: about two screens of tiles,
/// some 40 MB. The pages hold what they show themselves.
const KEPT: usize = 48;
/// How long a file that did not decode waits before it is asked for again, each time it
/// fails. The pages are told when each of these comes due. After the last, the file keeps that
/// wait with nobody told: a page that draws asks again, so a cover the image cache fetches
/// late still shows, and opening Downloads asks at once.
const RETRIES: [Duration; 3] = [
    Duration::from_secs(5),
    Duration::from_secs(30),
    Duration::from_secs(120),
];
/// How many threads decode posters.
const THREADS: usize = 2;
/// Slint counts its timers in whole milliseconds, so one may fire a moment before the instant
/// it was set for; the retry timer is set this much later.
const MARGIN: Duration = Duration::from_millis(50);

type Land = Box<dyn FnOnce(Option<Poster>)>;
type Listener = Rc<dyn Fn()>;

thread_local! {
    /// The posters decoded last, by path, so one shown on two pages is decoded once.
    static KEPT_POSTERS: RefCell<Lru<Poster>> = RefCell::new(Lru::new(KEPT));
    /// The pages waiting on each file being decoded now.
    static WAITING: RefCell<Joins<Land>> = RefCell::new(Joins::default());
    /// The files that did not decode, and when each may be tried again.
    static FAILED: RefCell<Tries> = RefCell::new(Tries::default());
    /// The batch the asks made in this turn of the event loop go in.
    static BATCH: Cell<Option<u64>> = const { Cell::new(None) };
    static NEXT_BATCH: Cell<u64> = const { Cell::new(0) };
    /// What each page does when a retry comes due: ask for what it shows again.
    static RETRY_LISTENERS: RefCell<Vec<Listener>> = const { RefCell::new(Vec::new()) };
    /// One timer for every retry, set for the soonest due, and when that is.
    static RETRY_TIMER: slint::Timer = slint::Timer::default();
    static RETRY_AT: Cell<Option<Instant>> = const { Cell::new(None) };
}

/// What the posters threads have still to decode, shared with them.
static QUEUE: Mutex<Queue> = Mutex::new(Queue::new());
static READY: Condvar = Condvar::new();
static STARTED: OnceLock<bool> = OnceLock::new();

/// The app is starting a new event loop: a file asked of the last one, whose answer could not
/// be handed back, is asked again rather than waited on for good.
pub fn reset() {
    KEPT_POSTERS.with(|k| *k.borrow_mut() = Lru::new(KEPT));
    WAITING.with(|w| *w.borrow_mut() = Joins::default());
    FAILED.with(|f| *f.borrow_mut() = Tries::default());
    BATCH.with(|b| b.set(None));
    RETRY_LISTENERS.with(|l| l.borrow_mut().clear());
    RETRY_TIMER.with(slint::Timer::stop);
    RETRY_AT.with(|a| a.set(None));
    lock(&QUEUE).clear();
}

/// Calls `listener` on the UI thread each time a poster that did not decode may be asked for
/// again, while its retries last: the page asks for what it shows again. One timer serves
/// every file and every page.
pub fn on_retry(listener: impl Fn() + 'static) {
    RETRY_LISTENERS.with(|l| l.borrow_mut().push(Rc::new(listener)));
}

/// Sets the retry timer for the soonest retry nobody was told of yet, unless it is set for
/// one sooner already.
fn arm_retry() {
    let Some(due) = FAILED.with(|f| f.borrow().next_due()) else {
        return;
    };
    if RETRY_AT.with(Cell::get).is_some_and(|at| at <= due) {
        return;
    }
    RETRY_AT.with(|a| a.set(Some(due)));
    let wait = due.saturating_duration_since(Instant::now()) + MARGIN;
    RETRY_TIMER.with(|t| t.start(slint::TimerMode::SingleShot, wait, retry_due));
}

fn retry_due() {
    RETRY_AT.with(|a| a.set(None));
    FAILED.with(|f| f.borrow_mut().told(Instant::now()));
    let listeners = RETRY_LISTENERS.with(|l| l.borrow().clone());
    for listener in listeners {
        listener();
    }
    arm_retry();
}

/// The poster at `path` if it was decoded lately, for a page to show at once.
pub fn cached(path: &str) -> Option<Poster> {
    KEPT_POSTERS.with(|k| k.borrow_mut().get(path))
}

/// A page the person opened asks for its failed files again at once.
pub fn forget_failures() {
    FAILED.with(|f| *f.borrow_mut() = Tries::default());
}

/// Whether the file at `path` may be asked for now: it did not just fail.
pub fn may_ask(path: &str) -> bool {
    FAILED.with(|f| f.borrow().may_ask(path, Instant::now()))
}

/// Decodes the poster file at `path` off the UI thread, once however many pages ask, and
/// hands it to `land` on the UI thread: `None` when it could not be read or decoded. Called
/// on the UI thread; `land` always runs later, never inside this call.
///
/// One thread decodes the newest asks first, the ones made in the last turn of the event
/// loop in the order they were made, so the page on screen never waits behind a page asked
/// for earlier; the other decodes the oldest first, so no page waits for good behind newer
/// ones (#1206). Downloads used to start a thread for each draw's posters, and each poster
/// that came in drew the page again, so a long list made threads, and decoded copies waiting
/// for the UI thread, faster than they finished.
pub fn load(path: String, land: impl FnOnce(Option<Poster>) + 'static) {
    if let Some(poster) = KEPT_POSTERS.with(|k| k.borrow_mut().get(&path)) {
        slint::Timer::single_shot(Duration::ZERO, move || land(Some(poster)));
        return;
    }
    let batch = batch();
    let first = WAITING.with(|w| w.borrow_mut().ask(path.clone(), Box::new(land)));
    if !first {
        // Asked again while it waits: it moves up with the newest asks.
        lock(&QUEUE).bump(batch, &path);
        return;
    }
    if !start() {
        // No thread to decode it: the pages hear it did not come, rather than wait for good.
        slint::Timer::single_shot(Duration::ZERO, move || landed(path, None));
        return;
    }
    lock(&QUEUE).push(batch, path);
    READY.notify_one();
}

/// The batch for this turn of the event loop, opened by its first ask.
fn batch() -> u64 {
    if let Some(batch) = BATCH.with(std::cell::Cell::get) {
        return batch;
    }
    let batch = NEXT_BATCH.with(|n| {
        let batch = n.get();
        n.set(batch + 1);
        batch
    });
    BATCH.with(|b| b.set(Some(batch)));
    slint::Timer::single_shot(Duration::ZERO, || BATCH.with(|b| b.set(None)));
    batch
}

/// Starts the posters threads once; false when none would start.
fn start() -> bool {
    *STARTED.get_or_init(|| {
        let mut any = false;
        for i in 0..THREADS {
            any |= std::thread::Builder::new()
                .name("posters".into())
                .spawn(move || decode_queued(i == 0))
                .is_ok();
        }
        any
    })
}

/// A posters thread: the newest asks first, or the oldest, each read, decoded and handed
/// back. A decode that panics is caught and handed back as not decoded, so the thread and the
/// files queued behind it carry on.
fn decode_queued(newest: bool) {
    loop {
        let path = {
            let mut queue = lock(&QUEUE);
            loop {
                if let Some(path) = queue.pop(newest) {
                    break path;
                }
                queue = READY.wait(queue).unwrap_or_else(|e| e.into_inner());
            }
        };
        let decoded = catch_unwind(AssertUnwindSafe(|| {
            std::fs::read(&path).ok().and_then(|b| decode(&b))
        }))
        .ok()
        .flatten();
        // A loop that has gone cannot be told; `reset` asks again on the next one.
        let _ = slint::invoke_from_event_loop(move || landed(path, decoded));
    }
}

fn lock(queue: &Mutex<Queue>) -> std::sync::MutexGuard<'_, Queue> {
    queue.lock().unwrap_or_else(|e| e.into_inner())
}

fn landed(path: String, decoded: Option<Decoded>) {
    let poster = decoded.map(|d| Poster {
        image: slint::Image::from_rgba8(d.buffer),
        tone: d.tone,
    });
    match &poster {
        Some(poster) => {
            KEPT_POSTERS.with(|k| k.borrow_mut().put(path.clone(), poster.clone()));
            FAILED.with(|f| f.borrow_mut().landed(&path));
        }
        None => {
            FAILED.with(|f| f.borrow_mut().failed(&path, Instant::now()));
            arm_retry();
        }
    }
    let waiting = WAITING.with(|w| w.borrow_mut().take(&path));
    for land in waiting {
        land(poster.clone());
    }
}

/// The pages waiting on each file: the first ask sends it to be decoded, a later one joins it.
struct Joins<W> {
    waiting: HashMap<String, Vec<W>>,
}

impl<W> Default for Joins<W> {
    fn default() -> Self {
        Joins {
            waiting: HashMap::new(),
        }
    }
}

impl<W> Joins<W> {
    /// Adds a waiter for `path`; true when it is the first, so the file is to be decoded.
    fn ask(&mut self, path: String, waiter: W) -> bool {
        let waiting = self.waiting.entry(path).or_default();
        waiting.push(waiter);
        waiting.len() == 1
    }

    /// Everyone waiting on `path`, who are no longer waiting.
    fn take(&mut self, path: &str) -> Vec<W> {
        self.waiting.remove(path).unwrap_or_default()
    }
}

/// The files still to decode, in batches, each in the order it was asked, taken from the
/// newest batch or the oldest.
struct Queue {
    batches: BTreeMap<u64, VecDeque<String>>,
}

impl Queue {
    const fn new() -> Queue {
        Queue {
            batches: BTreeMap::new(),
        }
    }

    fn push(&mut self, batch: u64, path: String) {
        self.batches.entry(batch).or_default().push_back(path);
    }

    /// Moves `path` into `batch` if it is still waiting in an older one.
    fn bump(&mut self, batch: u64, path: &str) {
        let older = self
            .batches
            .range(..batch)
            .find_map(|(b, paths)| paths.iter().position(|p| p == path).map(|i| (*b, i)));
        let Some((b, i)) = older else {
            return;
        };
        if let Some(paths) = self.batches.get_mut(&b) {
            paths.remove(i);
            if paths.is_empty() {
                self.batches.remove(&b);
            }
        }
        self.push(batch, path.to_string());
    }

    fn pop(&mut self, newest: bool) -> Option<String> {
        let mut batch = if newest {
            self.batches.last_entry()?
        } else {
            self.batches.first_entry()?
        };
        let path = batch.get_mut().pop_front();
        if batch.get().is_empty() {
            batch.remove();
        }
        path
    }

    fn clear(&mut self) {
        self.batches.clear();
    }
}

/// The last `cap` entries put or got, the oldest let go first.
struct Lru<V> {
    cap: usize,
    order: VecDeque<String>,
    entries: HashMap<String, V>,
}

impl<V: Clone> Lru<V> {
    fn new(cap: usize) -> Lru<V> {
        Lru {
            cap,
            order: VecDeque::new(),
            entries: HashMap::new(),
        }
    }

    fn get(&mut self, key: &str) -> Option<V> {
        let value = self.entries.get(key)?.clone();
        self.touch(key);
        Some(value)
    }

    fn put(&mut self, key: String, value: V) {
        if self.entries.insert(key.clone(), value).is_some() {
            self.touch(&key);
            return;
        }
        self.order.push_back(key);
        while self.order.len() > self.cap {
            if let Some(oldest) = self.order.pop_front() {
                self.entries.remove(&oldest);
            }
        }
    }

    fn touch(&mut self, key: &str) {
        if let Some(i) = self.order.iter().position(|k| k == key)
            && let Some(key) = self.order.remove(i)
        {
            self.order.push_back(key);
        }
    }
}

/// One file that did not decode.
struct Try {
    /// How often it failed.
    tries: usize,
    /// When it may be asked for again.
    at: Instant,
    /// The pages were told it may be asked for again, or will not be: past the last retry.
    told: bool,
}

/// The files that did not decode, and when each may be tried again.
#[derive(Default)]
struct Tries {
    failed: HashMap<String, Try>,
}

impl Tries {
    fn failed(&mut self, path: &str, now: Instant) {
        let tries = self.failed.get(path).map_or(0, |t| t.tries);
        let wait = RETRIES.get(tries).unwrap_or(&RETRIES[RETRIES.len() - 1]);
        let next = Try {
            tries: tries + 1,
            at: now + *wait,
            told: tries >= RETRIES.len(),
        };
        self.failed.insert(path.to_string(), next);
    }

    fn landed(&mut self, path: &str) {
        self.failed.remove(path);
    }

    fn may_ask(&self, path: &str, now: Instant) -> bool {
        self.failed.get(path).is_none_or(|t| now >= t.at)
    }

    /// The soonest retry the pages are still to be told of.
    fn next_due(&self) -> Option<Instant> {
        self.failed.values().filter(|t| !t.told).map(|t| t.at).min()
    }

    /// The pages were told of every retry due by `now`.
    fn told(&mut self, now: Instant) {
        for t in self.failed.values_mut().filter(|t| t.at <= now) {
            t.told = true;
        }
    }
}

/// The band behind a poster: its hue darkened most of the way into the ground.
pub fn band(tone: Option<[u8; 3]>) -> slint::Color {
    const GROUND: [u8; 3] = [0x0c, 0x0d, 0x10];
    match tone {
        Some(t) => mix(t, GROUND, 0.4),
        None => slint::Color::from_rgb_u8(0x1c, 0x1e, 0x21),
    }
}

/// Text and bars on that band: the hue lifted toward white.
pub fn ink(tone: Option<[u8; 3]>) -> slint::Color {
    match tone {
        Some(t) => mix(t, [255, 255, 255], 0.55),
        None => slint::Color::from_rgb_u8(0xf2, 0xf4, 0xf6),
    }
}

/// `a` with `share` of it kept and the rest `b`.
fn mix(a: [u8; 3], b: [u8; 3], share: f32) -> slint::Color {
    let c = |i: usize| (f32::from(a[i]) * share + f32::from(b[i]) * (1.0 - share)).round() as u8;
    slint::Color::from_rgb_u8(c(0), c(1), c(2))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The page on screen asks last; its posters come first, in the order it asked (#1206).
    #[test]
    fn the_newest_asks_are_decoded_first_in_their_own_order() {
        let mut q = Queue::new();
        for p in ["a", "b", "c"] {
            q.push(0, p.into());
        }
        for p in ["d", "e"] {
            q.push(1, p.into());
        }
        // Asked again by the newest page, `b` moves up with it.
        q.bump(2, "b");
        let order: Vec<String> = std::iter::from_fn(|| q.pop(true)).collect();
        assert_eq!(order, ["b", "d", "e", "a", "c"]);
        // A file not waiting (being decoded) is not queued twice by a second ask.
        q.bump(3, "a");
        assert_eq!(q.pop(true), None);
    }

    /// The other thread takes the oldest asks, so Home's posters never wait for good behind
    /// every page opened after it.
    #[test]
    fn the_oldest_asks_still_move_while_newer_ones_come() {
        let mut q = Queue::new();
        q.push(0, "home".into());
        q.push(1, "downloads".into());
        q.push(2, "search".into());
        assert_eq!(q.pop(true).as_deref(), Some("search"));
        assert_eq!(q.pop(false).as_deref(), Some("home"));
        assert_eq!(q.pop(false).as_deref(), Some("downloads"));
    }

    #[test]
    fn a_second_ask_for_a_file_being_decoded_joins_the_first() {
        let mut j: Joins<u8> = Joins::default();
        assert!(j.ask("a".into(), 1));
        assert!(!j.ask("a".into(), 2));
        assert!(j.ask("b".into(), 3));
        assert_eq!(j.take("a"), [1, 2]);
        // Once it landed, the next ask decodes it again.
        assert!(j.ask("a".into(), 4));
    }

    #[test]
    fn the_kept_posters_let_the_least_recently_used_go() {
        let mut k = Lru::new(2);
        k.put("a".into(), 1);
        k.put("b".into(), 2);
        assert_eq!(k.get("a"), Some(1));
        k.put("c".into(), 3);
        assert_eq!(k.get("b"), None);
        assert_eq!(k.get("a"), Some(1));
        assert_eq!(k.get("c"), Some(3));
    }

    /// The pages are told of each retry once, and only while the retries last: a broken
    /// file never wakes the app every two minutes for good, and a cover the image cache
    /// fetches late is still asked for by the next draw.
    #[test]
    fn a_file_that_does_not_decode_is_retried_a_few_times_then_waits_for_a_draw() {
        let t0 = Instant::now();
        let mut t = Tries::default();
        assert!(t.may_ask("a", t0));
        t.failed("a", t0);
        assert!(!t.may_ask("a", t0 + Duration::from_secs(1)));
        assert_eq!(t.next_due(), Some(t0 + RETRIES[0]));
        // A timer that fires a moment early tells nobody yet.
        t.told(t0 + RETRIES[0] - Duration::from_millis(2));
        assert_eq!(t.next_due(), Some(t0 + RETRIES[0]));
        t.told(t0 + RETRIES[0]);
        assert_eq!(t.next_due(), None);
        assert!(t.may_ask("a", t0 + RETRIES[0]));
        let mut now = t0;
        for wait in RETRIES {
            now += wait;
            t.failed("a", now);
            t.told(now + RETRIES[RETRIES.len() - 1]);
        }
        // Past the last retry there is nothing to tell, but a draw may still ask.
        assert_eq!(t.next_due(), None);
        let last = RETRIES[RETRIES.len() - 1];
        assert!(!t.may_ask("a", now + last - Duration::from_secs(1)));
        assert!(t.may_ask("a", now + last));
        // One that decodes is clear again.
        t.landed("a");
        assert!(t.may_ask("a", now));
    }
}
