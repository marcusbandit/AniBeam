//! The commands' UI half, run on the Slint event loop: reading what the window shows and
//! acting on it the way a finger would.
//!
//! The window is read through Slint's accessibility properties, the tree a screen reader
//! would get: every `Text` carries its text as a label, every field its value, and the
//! app's buttons a label and a default action. It needs no debug info compiled in, so the
//! release build's item tree is the one the person runs.

use std::cell::RefCell;
use std::collections::VecDeque;
use std::ops::ControlFlow;
use std::rc::Rc;
use std::sync::mpsc::Sender;
use std::time::{Duration, Instant};

use i_slint_core::accessibility::{
    AccessibilityAction, AccessibleStringProperty, SupportedAccessibilityAction,
};
use i_slint_core::item_tree::{ItemRc, ParentItemTraversalMode};
use i_slint_core::items::{
    AccessibleRole, Clip, Flickable, InputType, Opacity, TextInput, TouchArea,
};
use i_slint_core::platform::WindowEventDispatchResult;
use i_slint_core::window::WindowInner;
use slint::platform::{Key as SlintKey, PointerEventButton, WindowEvent};
use slint::{ComponentHandle, LogicalPosition, SharedString, Timer, TimerMode};

use super::command::{Command, Dir, Field, HELP, Key, Onboarding, Series, Tab, Target};
use crate::{AppData, AppWindow, FindData, Fold, Nav, PlayerData, Settings};

/// How long a finger stays down for a long press: past Slint's and the app's hold.
const HOLD: Duration = Duration::from_millis(800);
/// How many frames the stats keep: `perf`'s longest sample at 144 Hz, so it counts them all.
const FRAMES_KEPT: usize = 4800;

thread_local! {
    static WINDOW: RefCell<Option<slint::Weak<AppWindow>>> = const { RefCell::new(None) };
    static FRAMES: RefCell<Frames> = RefCell::new(Frames::default());
    /// The swipe under way, its timer held here so it can be let go of outside its own tick.
    static SWIPE: RefCell<Option<Rc<Timer>>> = const { RefCell::new(None) };
}

/// The window the commands act on.
pub fn install(window: &AppWindow) {
    WINDOW.with(|w| *w.borrow_mut() = Some(window.as_weak()));
}

fn window() -> Option<AppWindow> {
    WINDOW.with(|w| w.borrow().as_ref().and_then(|w| w.upgrade()))
}

// Frame times, from the window's rendering notifier (player.rs holds it).

#[derive(Default)]
struct Frames {
    drawing: Option<Instant>,
    /// When each recent frame started, and how long the UI thread took to draw it.
    list: VecDeque<(Instant, Duration)>,
    mark: Option<Instant>,
}

/// A frame starts drawing.
pub fn rendering() {
    FRAMES.with(|f| f.borrow_mut().drawing = Some(Instant::now()));
}

/// The frame that started is drawn.
pub fn rendered() {
    FRAMES.with(|f| {
        let mut f = f.borrow_mut();
        if let Some(start) = f.drawing.take() {
            if f.list.len() == FRAMES_KEPT {
                f.list.pop_front();
            }
            f.list.push_back((start, start.elapsed()));
        }
    });
}

/// Starts a `perf` window: `frames_since_mark` counts from here.
pub fn frames_mark() {
    FRAMES.with(|f| f.borrow_mut().mark = Some(Instant::now()));
}

fn ms(d: Duration) -> f64 {
    d.as_secs_f64() * 1000.0
}

/// The frames drawn since `since`, summed up in one line.
fn frames_line(since: Instant, f: &Frames) -> String {
    let frames: Vec<&(Instant, Duration)> = f.list.iter().filter(|(t, _)| *t >= since).collect();
    if frames.is_empty() {
        return "frames: none drawn (nothing moved on screen)".into();
    }
    let total: Duration = frames.iter().map(|(_, d)| *d).sum();
    let max = frames.iter().map(|(_, d)| *d).max().unwrap_or_default();
    let gap = frames
        .windows(2)
        .map(|w| w[1].0 - w[0].0)
        .max()
        .unwrap_or_default();
    let span = since.elapsed().as_secs_f64().max(0.001);
    format!(
        "frames: {} drawn, {:.1} per s, draw avg {:.1} ms, max {:.1} ms, longest gap {:.0} ms",
        frames.len(),
        frames.len() as f64 / span,
        ms(total) / frames.len() as f64,
        ms(max),
        ms(gap)
    )
}

/// The frames since `frames_mark`.
pub fn frames_since_mark() -> String {
    FRAMES.with(|f| {
        let f = f.borrow();
        frames_line(f.mark.unwrap_or_else(Instant::now), &f)
    })
}

/// What `state` reads on the UI thread: where the app is and how its last frames went.
pub fn state_lines() -> String {
    let Some(w) = window() else {
        return "error: the window is not up".into();
    };
    let (last, recent) = FRAMES.with(|f| {
        let f = f.borrow();
        let last = f
            .list
            .back()
            .map(|(t, d)| {
                format!(
                    "last frame {:.1} ms, {:.1} s ago",
                    ms(*d),
                    t.elapsed().as_secs_f64()
                )
            })
            .unwrap_or_else(|| "no frame drawn yet".into());
        let since = Instant::now()
            .checked_sub(Duration::from_secs(5))
            .unwrap_or_else(Instant::now);
        (last, frames_line(since, &f))
    });
    let data = w.global::<AppData>();
    let mut out = vec![
        where_line(&w),
        format!("{last}; last 5 s {}", recent.trim_start_matches("frames: ")),
        format!(
            "downloads: {} unfinished, {} moving, {} finished, down {} up {}",
            data.get_downloading_count(),
            data.get_moving_count(),
            data.get_finished_count(),
            nonempty(data.get_rate_down()),
            nonempty(data.get_rate_up()),
        ),
    ];
    if w.global::<PlayerData>().get_open() {
        out.push("player: open".into());
    }
    out.join("\n")
}

fn nonempty(s: SharedString) -> String {
    if s.is_empty() {
        "-".into()
    } else {
        s.to_string()
    }
}

// Reading the window.

/// A rect in the window's logical pixels: left, top, right, bottom.
#[derive(Clone, Copy)]
struct Area(f32, f32, f32, f32);

impl Area {
    fn of(item: &ItemRc) -> Area {
        let g = item.geometry();
        let o = item.map_to_window(g.origin);
        Area(o.x, o.y, o.x + g.size.width, o.y + g.size.height)
    }

    fn and(self, o: Area) -> Area {
        Area(
            self.0.max(o.0),
            self.1.max(o.1),
            self.2.min(o.2),
            self.3.min(o.3),
        )
    }

    fn holds(self, (x, y): (f32, f32)) -> bool {
        x >= self.0 && x <= self.2 && y >= self.1 && y <= self.3
    }
}

/// What a walk up from an item finds: its ancestors nearest first, the opacity it draws at,
/// and the part of the window its clipping parents (a `clip: true` or a Flickable) let show.
struct Up {
    ancestors: Vec<ItemRc>,
    opacity: f32,
    clip: Area,
}

fn walk_up(item: &ItemRc, window: Area) -> Up {
    let mut up = Up {
        ancestors: Vec::new(),
        opacity: 1.0,
        clip: window,
    };
    let mut current = item.parent_item(ParentItemTraversalMode::StopAtPopups);
    if let Some(o) = item.downcast::<Opacity>() {
        up.opacity *= o.as_pin_ref().opacity();
    }
    while let Some(a) = current {
        if let Some(o) = a.downcast::<Opacity>() {
            up.opacity *= o.as_pin_ref().opacity();
        }
        let clips = a.downcast::<Clip>().is_some_and(|c| c.as_pin_ref().clip())
            || a.downcast::<Flickable>().is_some();
        if clips {
            up.clip = up.clip.and(Area::of(&a));
        }
        current = a.parent_item(ParentItemTraversalMode::StopAtPopups);
        up.ancestors.push(a);
    }
    up
}

/// One element a screen reader would see.
struct Element {
    item: ItemRc,
    /// Its place in the tree's order, which is the order Slint draws in.
    order: usize,
    ancestors: Vec<ItemRc>,
    role: AccessibleRole,
    label: String,
    /// The label folded for matching: blanks collapsed, lower case.
    key: String,
    value: Option<String>,
    enabled: bool,
    read_only: bool,
    checked: Option<bool>,
    /// A tab of the bar that is the page shown.
    selected: bool,
    focused: bool,
    /// Top left and size in the window's logical pixels.
    rect: (f32, f32, f32, f32),
    /// Where a tap on it lands.
    centre: (f32, f32),
    /// Not hidden, faded out, or clipped or scrolled away at the point a tap lands.
    shown: bool,
    /// Shown, but under something drawn later that takes the touch there: a sheet, a menu,
    /// a question, the bar over a page scrolled under it.
    covered: bool,
}

/// A touch area that takes a tap, for working out what covers what.
struct Touch {
    order: usize,
    item: ItemRc,
    ancestors: Vec<ItemRc>,
    area: Area,
}

impl Element {
    fn role_name(&self) -> String {
        match self.role {
            AccessibleRole::TextInput => "field".into(),
            AccessibleRole::ListItem => "item".into(),
            r => format!("{r:?}").to_lowercase(),
        }
    }

    fn line(&self) -> String {
        let mut s = self.role_name();
        if !self.label.is_empty() {
            s.push_str(&format!(" {:?}", clip(&self.label)));
        }
        if let Some(v) = &self.value {
            s.push_str(&format!(" value {:?}", clip(v)));
        }
        for (on, word) in [
            (self.focused, " focused"),
            (!self.enabled, " disabled"),
            (self.read_only, " read-only"),
            (self.checked == Some(true), " on"),
            (self.checked == Some(false), " off"),
            (self.selected, " selected"),
            (!self.shown, " offscreen"),
            (self.covered, " covered"),
        ] {
            if on {
                s.push_str(word);
            }
        }
        let (_, _, w, h) = self.rect;
        s.push_str(&format!(
            " at {:.0},{:.0} ({:.0}x{:.0})",
            self.centre.0, self.centre.1, w, h
        ));
        s
    }

    fn holds(&self, x: f32, y: f32) -> bool {
        let (ex, ey, w, h) = self.rect;
        x >= ex && x <= ex + w && y >= ey && y <= ey + h
    }

    /// On screen and in reach of a finger.
    fn reachable(&self) -> bool {
        self.shown && !self.covered
    }
}

/// Labels longer than this are cut in a line, and a cut label still matches what it starts
/// with (`matches`), so a line copied from `tree` works as a target.
const CUT: usize = 90;

/// A long label, cut to what a line of the tree can hold.
fn clip(s: &str) -> String {
    let flat: String = s.split_whitespace().collect::<Vec<_>>().join(" ");
    if flat.chars().count() > CUT {
        format!("{}...", flat.chars().take(CUT - 3).collect::<String>())
    } else {
        flat
    }
}

fn norm(s: &str) -> String {
    s.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

/// Whether a folded label is the one asked for: the same, or, for a label `tree` cut short,
/// one that starts with what it kept.
fn names(key: &str, want: &str) -> bool {
    match want.strip_suffix("...") {
        Some(start) if want.chars().count() == CUT => key.starts_with(start),
        _ => key == want,
    }
}

fn window_size(w: &AppWindow) -> (f32, f32) {
    let size = w.window().size().to_logical(w.window().scale_factor());
    (size.width, size.height)
}

/// Every accessible element in the window, in the tree's order.
fn elements(w: &AppWindow) -> Vec<Element> {
    let (width, height) = window_size(w);
    let window = Area(0.0, 0.0, width, height);
    let root = ItemRc::new_root(WindowInner::from_pub(w.window()).component());
    let mut out = Vec::new();
    let mut touches = Vec::new();
    let mut order = 0;
    root.visit_descendants(|item| {
        order += 1;
        if let Some(t) = item.downcast::<TouchArea>()
            && t.as_pin_ref().enabled()
            && item.is_visible()
        {
            let up = walk_up(item, window);
            touches.push(Touch {
                order,
                item: item.clone(),
                area: Area::of(item).and(up.clip),
                ancestors: up.ancestors,
            });
        }
        if !item.is_accessible() {
            return ControlFlow::<()>::Continue(());
        }
        let role = item.accessible_role();
        if role == AccessibleRole::None {
            return ControlFlow::Continue(());
        }
        let prop =
            |p: AccessibleStringProperty| item.accessible_string_property(p).map(|s| s.to_string());
        let label = prop(AccessibleStringProperty::Label).unwrap_or_default();
        // Empty texts and unnamed pictures are layout, not something to read or press.
        if label.trim().is_empty() && matches!(role, AccessibleRole::Text | AccessibleRole::Image) {
            return ControlFlow::Continue(());
        }
        let g = item.geometry();
        let origin = item.map_to_window(g.origin);
        let centre = item.map_to_native_window(g.center());
        let centre = (centre.x, centre.y);
        let up = walk_up(item, window);
        let shown = up.clip.holds(centre) && item.is_visible() && up.opacity > 0.05;
        let input = item.downcast::<TextInput>();
        let focused = input.as_ref().is_some_and(|t| t.as_pin_ref().has_focus());
        // A password field's text stays out of the answer: answers end up in logs.
        let secret = input
            .as_ref()
            .is_some_and(|t| matches!(t.as_pin_ref().input_type(), InputType::Password));
        let value = prop(AccessibleStringProperty::Value)
            .filter(|v| !v.is_empty() || role == AccessibleRole::TextInput)
            .map(|v| {
                if secret && !v.is_empty() {
                    "(hidden)".into()
                } else {
                    v
                }
            });
        out.push(Element {
            item: item.clone(),
            order,
            ancestors: up.ancestors,
            role,
            key: norm(&label),
            label,
            value,
            enabled: prop(AccessibleStringProperty::Enabled).as_deref() != Some("false"),
            read_only: prop(AccessibleStringProperty::ReadOnly).as_deref() == Some("true"),
            checked: prop(AccessibleStringProperty::Checkable)
                .filter(|c| c == "true")
                .map(|_| prop(AccessibleStringProperty::Checked).as_deref() == Some("true")),
            selected: prop(AccessibleStringProperty::ItemSelected).as_deref() == Some("true"),
            focused,
            rect: (origin.x, origin.y, g.size.width, g.size.height),
            centre,
            shown,
            covered: false,
        });
        ControlFlow::Continue(())
    });
    // Slint hands a touch to the last-drawn touch area under it. One drawn after the
    // element covers it, unless it belongs to the element: inside it, or a child of one of
    // its parents, the way a button's own touch area sits over its label.
    for e in out.iter_mut().filter(|e| e.shown) {
        e.covered = touches.iter().any(|t| {
            t.order > e.order
                && t.area.holds(e.centre)
                && !t.ancestors.contains(&e.item)
                && !t.ancestors.first().is_some_and(|p| e.ancestors.contains(p))
                && t.item != e.item
        });
    }
    out
}

/// The elements worth listing: in reach, and without the text inside a control that names
/// it already.
fn listed(all: Vec<Element>) -> Vec<Element> {
    let shown: Vec<Element> = all.into_iter().filter(Element::reachable).collect();
    let keep: Vec<bool> = shown
        .iter()
        .map(|e| {
            e.role != AccessibleRole::Text
                || !shown.iter().any(|o| {
                    o.role != AccessibleRole::Text
                        && o.holds(e.centre.0, e.centre.1)
                        && o.key == e.key
                })
        })
        .collect();
    shown
        .into_iter()
        .zip(keep)
        .filter_map(|(e, k)| k.then_some(e))
        .collect()
}

/// The elements in reach a label names, in the order `tree` lists them: whole-label matches
/// if any, else, unless `whole` says not to, the ones holding it.
fn matching(w: &AppWindow, label: &str, whole: bool) -> Vec<Element> {
    let want = norm(label);
    let list = listed(elements(w));
    if list.iter().any(|e| names(&e.key, &want)) || whole {
        list.into_iter().filter(|e| names(&e.key, &want)).collect()
    } else {
        list.into_iter().filter(|e| e.key.contains(&want)).collect()
    }
}

/// Where a touch lands: the element, if a label named one, the point, and how to say it.
type Landing = (Option<Element>, (f32, f32), String);

/// What a target lands on.
fn resolve(w: &AppWindow, target: &Target) -> Result<Landing, String> {
    let (label, nth) = match target {
        Target::Point(x, y) => return Ok((None, (*x, *y), format!("{x:.0},{y:.0}"))),
        // `4,5` names a shown label first, Stats' hours in a language that writes a decimal
        // comma, and a point only when nothing shows by that name.
        Target::Spot { word, x, y } => {
            if matching(w, word, true).is_empty() {
                return Ok((None, (*x, *y), format!("{x:.0},{y:.0}")));
            }
            (word, 1)
        }
        Target::Label { label, nth } => (label, *nth),
    };
    let mut hits = matching(w, label, false);
    if hits.is_empty() {
        return Err(format!(
            "nothing in reach is labelled {label:?}; tree lists what is, find what is not"
        ));
    }
    let count = hits.len();
    if nth > count {
        return Err(format!("{label:?} matches {count}, not {nth}"));
    }
    let e = hits.remove(nth - 1);
    let mut said = e.line();
    if count > 1 {
        said.push_str(&format!(" (match {nth} of {count}; --nth picks another)"));
    }
    let centre = e.centre;
    Ok((Some(e), centre, said))
}

// Acting on it.

fn at(x: f32, y: f32) -> LogicalPosition {
    LogicalPosition::new(x, y)
}

fn press_down(w: &AppWindow, (x, y): (f32, f32)) {
    let position = at(x, y);
    w.window()
        .dispatch_event(WindowEvent::PointerMoved { position });
    w.window().dispatch_event(WindowEvent::PointerPressed {
        position,
        button: PointerEventButton::Left,
    });
}

/// A finger lifting, then leaving: Slint sees a touch the way Android hands it one.
fn lift(w: &AppWindow, (x, y): (f32, f32)) {
    let position = at(x, y);
    w.window().dispatch_event(WindowEvent::PointerReleased {
        position,
        button: PointerEventButton::Left,
    });
    w.window().dispatch_event(WindowEvent::PointerExited);
}

fn tap(w: &AppWindow, point: (f32, f32)) {
    press_down(w, point);
    lift(w, point);
}

/// Sends a key and says whether anything took it: the press, or failing that the release.
fn key(w: &AppWindow, k: Key) -> bool {
    let text: SharedString = match k {
        Key::Back => SlintKey::Back.into(),
        Key::Enter => SlintKey::Return.into(),
        Key::Tab => SlintKey::Tab.into(),
        Key::Escape => SlintKey::Escape.into(),
        Key::Backspace => SlintKey::Backspace.into(),
        Key::Up => SlintKey::UpArrow.into(),
        Key::Down => SlintKey::DownArrow.into(),
        Key::Left => SlintKey::LeftArrow.into(),
        Key::Right => SlintKey::RightArrow.into(),
    };
    send_text(w, text)
}

/// Text or a key as key events, the way Android's backend sends them, and whether anything
/// took them.
fn send_text(w: &AppWindow, text: SharedString) -> bool {
    let taken = |r| matches!(r, Ok(WindowEventDispatchResult::Accepted));
    let pressed = w
        .window()
        .dispatch_event_with_result(WindowEvent::KeyPressed { text: text.clone() });
    let released = w
        .window()
        .dispatch_event_with_result(WindowEvent::KeyReleased { text });
    taken(pressed) || taken(released)
}

/// The page and what is open over it, in one line. app.slint names what is open, next to
/// the Back handler whose order it follows.
fn where_line(w: &AppWindow) -> String {
    let page = w.global::<Nav>().get_page();
    let mut parts = vec![format!(
        "page {}",
        Tab::of_page(page).map(Tab::name).unwrap_or("unknown")
    )];
    let find = w.global::<FindData>();
    if page == 2 && find.get_shown() && find.get_view() > 0 {
        parts.push(format!(
            "downloads {}",
            if find.get_view() == 1 {
                "show view"
            } else {
                "release list"
            }
        ));
    }
    let settings = w.global::<Settings>();
    if page == 3 && settings.get_section() >= 0 {
        parts.push(format!("more section {}", settings.get_section()));
    }
    let data = w.global::<AppData>();
    if data.get_series_open() {
        let s = data.get_series();
        parts.push(format!(
            "series {:?} id {}{}{}",
            s.title.as_str(),
            s.id,
            if data.get_series_remote() {
                " remote"
            } else {
                ""
            },
            if data.get_deleting() {
                " picking to delete"
            } else {
                ""
            }
        ));
    }
    let over = w.invoke_over();
    if !over.is_empty() {
        parts.push(format!("over it {over}"));
    }
    if data.get_onboarding() {
        parts.push(onboarding_line(w));
    }
    parts.join(", ")
}

fn onboarding_line(w: &AppWindow) -> String {
    let data = w.global::<AppData>();
    format!(
        "onboarding step {} of {}",
        data.get_step(),
        data.get_onboarding_steps()
    )
}

/// What shows; with a filter, only the controls whose line holds it.
fn dump(w: &AppWindow, filter: &str) -> String {
    let (width, height) = window_size(w);
    let posture = match w.global::<Fold>().get_posture() {
        0 => "folded",
        1 => "open",
        _ => "half folded",
    };
    let want = norm(filter);
    let all = elements(w);
    let covered = all.iter().filter(|e| e.shown && e.covered).count();
    let list: Vec<Element> = listed(all)
        .into_iter()
        .filter(|e| want.is_empty() || norm(&e.line()).contains(&want))
        .collect();
    let mut out = vec![
        where_line(w),
        format!(
            "window {width:.0}x{height:.0}, {posture}, scale {:.2}",
            w.window().scale_factor()
        ),
    ];
    if let Some(f) = list.iter().find(|e| e.focused) {
        out.push(format!("focus: {}", f.line()));
    }
    let under = if covered > 0 {
        format!(", {covered} more under what is open (find lists them)")
    } else {
        String::new()
    };
    out.push(format!("shown: {}{under}", list.len()));
    out.extend(list.iter().map(|e| format!("  {}", e.line())));
    out.join("\n")
}

/// For `wait`: the first control in reach the whole label names, if one shows yet. A wait
/// is for something to appear, so a label that only holds the words does not count.
pub fn probe(label: &str) -> Option<String> {
    let w = window()?;
    matching(&w, label, true).first().map(Element::line)
}

/// The phone series a `go series` names, from the phone's own library rather than a page's
/// list, which can hold a computer's shows or be narrowed by a search. A number is an id
/// when the phone has one, else a title.
fn series_id(series: &Series) -> Result<u64, String> {
    let all = crate::app::phone_series();
    let title = match series {
        Series::Id(id) => {
            if let Some((id, _)) = all.iter().find(|(i, _)| *i == u64::from(id.unsigned_abs())) {
                return Ok(*id);
            }
            id.to_string()
        }
        Series::Title(title) => title.clone(),
    };
    let want = norm(&title);
    all.iter()
        .find(|(_, t)| norm(t) == want)
        .or_else(|| all.iter().find(|(_, t)| norm(t).contains(&want)))
        .map(|(id, _)| *id)
        .ok_or_else(|| format!("the phone has no series {title:?}"))
}

fn fields(w: &AppWindow) -> Vec<Element> {
    elements(w)
        .into_iter()
        .filter(|e| e.reachable() && e.role == AccessibleRole::TextInput)
        .collect()
}

/// The visible scroll areas, largest first.
fn scrollers(w: &AppWindow) -> Vec<(ItemRc, Area)> {
    let (width, height) = window_size(w);
    let window = Area(0.0, 0.0, width, height);
    let root = ItemRc::new_root(WindowInner::from_pub(w.window()).component());
    let mut out = Vec::new();
    root.visit_descendants(|item| {
        if item.downcast::<Flickable>().is_some() && item.is_visible() {
            let area = Area::of(item).and(walk_up(item, window).clip);
            if area.2 > area.0 && area.3 > area.1 {
                out.push((item.clone(), area));
            }
        }
        ControlFlow::<()>::Continue(())
    });
    let size = |a: &Area| (a.2 - a.0) * (a.3 - a.1);
    out.sort_by(|a, b| size(&b.1).total_cmp(&size(&a.1)));
    out
}

fn offset(item: &ItemRc) -> (f32, f32) {
    item.downcast::<Flickable>()
        .map(|f| {
            let f = f.as_pin_ref();
            (f.content_x().get(), f.content_y().get())
        })
        .unwrap_or_default()
}

/// Presses the control a translated label names, through its own action: onboarding's
/// buttons, so `onboarding next` runs what the person's tap runs.
fn press_named(w: &AppWindow, ids: &[&str]) -> Result<String, String> {
    for id in ids {
        let label = crate::tr::tr(id, &[]);
        let hit = matching(w, &label, true).into_iter().find(|e| {
            e.item
                .supported_accessibility_actions()
                .contains(SupportedAccessibilityAction::Default)
        });
        if let Some(e) = hit {
            if !e.enabled {
                return Err(format!("{} is disabled", e.line()));
            }
            e.item.accessible_action(&AccessibilityAction::Default);
            return Ok(e.line());
        }
    }
    Err(format!("no button {:?} is in reach", ids.join(" or ")))
}

/// Runs one command and sends what it did, or `error: ...`, to `reply` once it is done:
/// at once for most, after the finger lifts for a hold or a swipe.
pub fn exec(command: Command, reply: Sender<String>) {
    let Some(w) = window() else {
        let _ = reply.send("error: the window is not up".into());
        return;
    };
    let done = |reply: Sender<String>, r: Result<String, String>| {
        let _ = reply.send(match r {
            Ok(s) => s,
            Err(e) => format!("error: {e}"),
        });
    };
    match command {
        Command::Hold(target) => match resolve(&w, &target) {
            Ok((_, point, said)) => {
                press_down(&w, point);
                let weak = w.as_weak();
                Timer::single_shot(HOLD, move || {
                    if let Some(w) = weak.upgrade() {
                        lift(&w, point);
                    }
                    let _ = reply.send(format!("held {said}"));
                });
            }
            Err(e) => done(reply, Err(e)),
        },
        Command::Swipe { from, to, ms } => swipe(&w, from, to, ms, reply),
        other => done(reply, run(&w, other)),
    }
}

fn swipe(w: &AppWindow, from: (f32, f32), to: (f32, f32), ms: u64, reply: Sender<String>) {
    const STEP: u64 = 16;
    let steps = (ms / STEP).max(2);
    press_down(w, from);
    let timer = Rc::new(Timer::default());
    let keep = Rc::downgrade(&timer);
    let weak = w.as_weak();
    let mut n = 0;
    let mut reply = Some(reply);
    timer.start(
        TimerMode::Repeated,
        Duration::from_millis(STEP),
        move || {
            n += 1;
            let t = n as f32 / steps as f32;
            let p = (from.0 + (to.0 - from.0) * t, from.1 + (to.1 - from.1) * t);
            if let Some(w) = weak.upgrade() {
                w.window().dispatch_event(WindowEvent::PointerMoved {
                    position: at(p.0, p.1),
                });
                if n >= steps {
                    lift(&w, to);
                }
            }
            if n >= steps {
                if let Some(t) = keep.upgrade() {
                    t.stop();
                }
                // Let go of this swipe's timer only: a swipe sent right after may hold the
                // slot by now.
                let mine = keep.clone();
                Timer::single_shot(Duration::ZERO, move || {
                    SWIPE.with(|s| {
                        let mut s = s.borrow_mut();
                        if s.as_ref().is_some_and(|t| Rc::as_ptr(t) == mine.as_ptr()) {
                            *s = None;
                        }
                    })
                });
                if let Some(r) = reply.take() {
                    let _ = r.send(format!(
                        "swiped {:.0},{:.0} to {:.0},{:.0} in {ms} ms",
                        from.0, from.1, to.0, to.1
                    ));
                }
            }
        },
    );
    // A swipe still going is replaced, and stops as its timer drops.
    SWIPE.with(|s| *s.borrow_mut() = Some(timer));
}

fn run(w: &AppWindow, command: Command) -> Result<String, String> {
    let data = w.global::<AppData>();
    Ok(match command {
        Command::Help => HELP.into(),
        Command::Go(tab) => {
            w.invoke_go(tab.page());
            where_line(w)
        }
        // Android's backend finishes the activity when nothing takes Back, so a Back that
        // goes untaken here is one that would close the app on the phone.
        Command::Back | Command::Key(_) => {
            let keys = match command {
                Command::Key(keys) => keys,
                _ => vec![Key::Back],
            };
            let mut untaken = Vec::new();
            for k in keys {
                if !key(w, k) {
                    if k == Key::Back {
                        return Err(format!(
                            "nothing took Back; Android would close the app here ({})",
                            where_line(w)
                        ));
                    }
                    untaken.push(format!("{k:?}").to_lowercase());
                }
            }
            if untaken.is_empty() {
                where_line(w)
            } else {
                format!("{}; nothing took {}", where_line(w), untaken.join(", "))
            }
        }
        Command::Open(series) => {
            let id = series_id(&series)?;
            let slint_id = i32::try_from(id).map_err(|_| format!("series {id} is out of range"))?;
            data.invoke_open_series(slint_id);
            if !data.get_series_open() || data.get_series().id != slint_id {
                return Err(format!("series {id} did not open"));
            }
            where_line(w)
        }
        Command::Press(target) => {
            let (e, point, said) = resolve(w, &target)?;
            if let Some(e) = &e
                && !e.enabled
            {
                return Err(format!("{said} is disabled; a tap would do nothing"));
            }
            let action = e.as_ref().filter(|e| {
                e.item
                    .supported_accessibility_actions()
                    .contains(SupportedAccessibilityAction::Default)
            });
            match action {
                Some(e) => {
                    e.item.accessible_action(&AccessibilityAction::Default);
                    format!("pressed {said}")
                }
                None => {
                    tap(w, point);
                    format!("tapped {said}")
                }
            }
        }
        Command::Tap(target) => {
            let (_, point, said) = resolve(w, &target)?;
            tap(w, point);
            format!("tapped {said}")
        }
        // exec handles both, for they answer later.
        Command::Hold(_) | Command::Swipe { .. } => unreachable!("exec runs these"),
        Command::Type(text) => {
            if !fields(w).iter().any(|f| f.focused) {
                return Err("no field has the focus; tap one first, or use fill".into());
            }
            if !send_text(w, text.as_str().into()) {
                return Err("the field did not take the text".into());
            }
            let now = fields(w).into_iter().find(|f| f.focused);
            format!(
                "typed into {}",
                now.map(|f| f.line()).unwrap_or_else(|| "the field".into())
            )
        }
        Command::Fill(field, text) => {
            let all = fields(w);
            let picked = match &field {
                Field::Index(n) => all
                    .into_iter()
                    .nth(n - 1)
                    .ok_or_else(|| format!("there is no field {n} in reach; tree lists them"))?,
                Field::Named(name) => {
                    let want = norm(name);
                    // A field shows its placeholder as a text over it while it is empty.
                    let marks: Vec<(f32, f32)> = listed(elements(w))
                        .into_iter()
                        .filter(|e| e.role == AccessibleRole::Text && e.key.contains(&want))
                        .map(|e| e.centre)
                        .collect();
                    all.into_iter()
                        .find(|f| {
                            f.key.contains(&want)
                                || f.value.as_deref().is_some_and(|v| norm(v) == want)
                                || marks.iter().any(|(x, y)| {
                                    let (fx, fy, fw, fh) = f.rect;
                                    // The placeholder sits on the field's line, left of it
                                    // by its glyph at most.
                                    *y >= fy && *y <= fy + fh && *x >= fx - 80.0 && *x <= fx + fw
                                })
                        })
                        .ok_or_else(|| format!("no field in reach shows {name:?}"))?
                }
            };
            // Slint's set-value checks neither, and no finger can type into such a field.
            if !picked.enabled || picked.read_only {
                return Err(format!("{} takes no typing", picked.line()));
            }
            picked
                .item
                .accessible_action(&AccessibilityAction::SetValue(text.as_str().into()));
            let after = fields(w)
                .into_iter()
                .find(|f| f.item == picked.item)
                .map(|f| f.line())
                .unwrap_or_default();
            format!("filled {after}")
        }
        Command::Search(text) => {
            w.invoke_go(Tab::Downloads.page());
            let find = w.global::<FindData>();
            for _ in 0..3 {
                if find.get_view() == 0 {
                    break;
                }
                find.invoke_back();
            }
            find.set_text(text.as_str().into());
            find.invoke_search(text.as_str().into());
            format!("searching Downloads for {text:?}; {}", where_line(w))
        }
        Command::Scroll {
            dir,
            amount,
            at: place,
        } => {
            let areas = scrollers(w);
            // Without a place, the largest scroll area on screen: the window's centre is
            // the crease when the phone is half folded, where nothing scrolls.
            let (point, said) = match place {
                Some(t) => {
                    let (_, p, said) = resolve(w, &t)?;
                    (p, format!(" at {said}"))
                }
                None => {
                    let a = areas
                        .first()
                        .map(|(_, a)| *a)
                        .ok_or("nothing on screen scrolls")?;
                    (((a.0 + a.2) / 2.0, (a.1 + a.3) / 2.0), String::new())
                }
            };
            // The innermost scroll area under the point, to see whether it moved.
            let under = areas
                .iter()
                .filter(|(_, a)| a.holds(point))
                .min_by(|a, b| {
                    let size = |a: &Area| (a.2 - a.0) * (a.3 - a.1);
                    size(&a.1).total_cmp(&size(&b.1))
                })
                .map(|(i, _)| i.clone());
            let before = under.as_ref().map(offset);
            let (dx, dy) = match dir {
                Dir::Down => (0.0, -amount),
                Dir::Up => (0.0, amount),
                Dir::Right => (-amount, 0.0),
                Dir::Left => (amount, 0.0),
            };
            w.window().dispatch_event(WindowEvent::PointerScrolled {
                position: at(point.0, point.1),
                delta_x: dx,
                delta_y: dy,
            });
            let moved = match (before, under.as_ref().map(offset)) {
                (Some(b), Some(a)) if (a.0 - b.0).abs() + (a.1 - b.1).abs() > 0.5 => {
                    format!("; it moved {:.0} px", (a.0 - b.0).abs() + (a.1 - b.1).abs())
                }
                (Some(_), Some(_)) => "; it did not move (at its end?)".into(),
                _ => "; nothing under the point scrolls".into(),
            };
            let dir = format!("{dir:?}").to_lowercase();
            format!(
                "scrolled {dir} {amount:.0} px at {:.0},{:.0}{said}{moved}",
                point.0, point.1
            )
        }
        Command::Onboarding(step) => {
            let steps = data.get_onboarding_steps();
            match step {
                Onboarding::Status => {}
                Onboarding::Start => data.invoke_restart_onboarding(),
                Onboarding::Finish => data.invoke_finish_onboarding(),
                _ if !data.get_onboarding() => {
                    return Err("onboarding is not open; onboarding start opens it".into());
                }
                // The page's own buttons, so a typed computer is saved and a host that does
                // not answer keeps the last step open, as a tap would.
                Onboarding::Next => {
                    press_named(w, &["onb-continue", "onb-start"])?;
                }
                Onboarding::Back => {
                    if data.get_step() == 0 {
                        return Err("onboarding is on its first step".into());
                    }
                    press_named(w, &["onb-back"])?;
                }
                Onboarding::Step(n) => {
                    if n >= steps {
                        return Err(format!("onboarding has steps 0 to {}", steps - 1));
                    }
                    data.set_step(n)
                }
            }
            if data.get_onboarding() {
                onboarding_line(w)
            } else {
                format!("onboarding off; {}", where_line(w))
            }
        }
        Command::Where => where_line(w),
        Command::Tree(filter) => dump(w, &filter),
        Command::Find(text) => {
            let want = norm(&text);
            let hits: Vec<String> = elements(w)
                .into_iter()
                .filter(|e| {
                    e.key.contains(&want)
                        || e.value.as_deref().is_some_and(|v| norm(v).contains(&want))
                })
                .map(|e| format!("  {}", e.line()))
                .collect();
            if hits.is_empty() {
                return Err(format!("nothing in the window holds {text:?}"));
            }
            format!("{} found\n{}", hits.len(), hits.join("\n"))
        }
        // The bridge thread runs these, round trips and all.
        Command::Wait { .. } | Command::State | Command::Perf(_) => {
            unreachable!("serve runs these")
        }
    })
}

#[cfg(test)]
mod tests {
    use std::sync::mpsc::channel;

    use slint::Model;

    use super::*;
    use crate::Tr;
    use crate::tr::{Arg, Lang, tr_in};

    fn window() -> AppWindow {
        i_slint_backend_testing::init_no_event_loop();
        let w = AppWindow::new().unwrap();
        w.window().set_size(slint::LogicalSize::new(400.0, 900.0));
        w.global::<Tr>().on_lookup(|id, _rev, args| {
            let flat: Vec<SharedString> = args.iter().collect();
            let pairs: Vec<(&str, Arg)> = flat
                .as_chunks::<2>()
                .0
                .iter()
                .map(|[k, v]| (k.as_str(), Arg::from(v.as_str())))
                .collect();
            tr_in(Lang::En, &id, &pairs).into()
        });
        install(&w);
        w
    }

    fn say(command: &str) -> String {
        let mut out = Vec::new();
        for c in super::super::command::parse_line(command).unwrap() {
            let (tx, rx) = channel();
            exec(c, tx);
            out.push(rx.try_recv().expect("answered at once"));
        }
        out.join("\n")
    }

    #[test]
    fn the_window_answers_commands_the_way_a_finger_would() {
        let w = window();
        let data = w.global::<AppData>();

        // Tabs, by name, and Back home again.
        assert!(
            say("go downloads").starts_with("page downloads"),
            "{}",
            say("tree")
        );
        assert_eq!(w.global::<Nav>().get_page(), 2);
        assert!(say("go more").starts_with("page more"));
        assert!(say("back").starts_with("page home"));
        assert_eq!(w.global::<Nav>().get_page(), 0);

        // Back on Home with nothing open is the one Back Android turns into leaving the app.
        assert!(
            say("back").starts_with("error: nothing took Back"),
            "{}",
            say("where")
        );

        // The tree names the page and what shows: the bar's tabs, the home one selected, and
        // not the text inside each tab that names it again.
        let dump = say("tree");
        assert!(dump.starts_with("page home"), "{dump}");
        assert_eq!(say("where").lines().count(), 1);
        assert!(dump.contains("window 400x900, folded"), "{dump}");
        let home = tr_in(Lang::En, "phone-nav-home", &[]);
        let downloads = tr_in(Lang::En, "phone-nav-downloads", &[]);
        assert!(dump.contains(&format!("tab {home:?} selected")), "{dump}");
        assert!(dump.contains(&format!("tab {downloads:?} at")), "{dump}");
        assert!(!dump.contains(&format!("text {downloads:?}")), "{dump}");
        let only = say(&format!("tree {downloads:?}"));
        assert!(only.contains(&downloads), "{only}");
        assert!(only.lines().count() < dump.lines().count(), "{only}");

        // A click on a tab runs its own action, and the tab opens.
        let pressed = say(&format!("click {downloads:?}"));
        assert!(pressed.starts_with("pressed tab"), "{pressed}");
        assert_eq!(w.global::<Nav>().get_page(), 2, "{pressed}");

        // A search lands in Downloads with its words in the field.
        say("go home");
        assert!(say("go downloads frieren").contains("page downloads"));
        assert_eq!(w.global::<FindData>().get_text(), "frieren");

        // Onboarding covers the page and the bar: what is under it is out of reach.
        data.set_onboarding(true);
        data.set_step(0);
        let steps = data.get_onboarding_steps();
        let dump = say("tree");
        assert!(dump.contains("more under what is open"), "{dump}");
        assert!(
            say(&format!("click {downloads:?}")).starts_with("error: nothing in reach"),
            "{dump}"
        );

        // Its own buttons step it on and back, as a finger would.
        assert_eq!(
            say("onboarding next"),
            format!("onboarding step 1 of {steps}")
        );
        assert_eq!(
            say("onboarding step 4"),
            format!("onboarding step 4 of {steps}")
        );
        let next = tr_in(Lang::En, "onb-continue", &[]);
        let pressed = say(&format!("click {next:?}"));
        assert!(pressed.starts_with("pressed button"), "{pressed}");
        assert_eq!(data.get_step(), 5, "{pressed}");
        assert_eq!(
            say("onboarding back"),
            format!("onboarding step 4 of {steps}")
        );
        assert!(say("tree").contains(&format!("onboarding step 4 of {steps}")));
        data.set_onboarding(false);

        // What is not there says so.
        assert!(say("click \"No such control\"").starts_with("error: nothing in reach"));
        assert!(say("go series 99999").starts_with("error"));
        assert!(say("type hello").starts_with("error: no field has the focus"));
        assert!(say("find zzzz-nothing").starts_with("error"));
    }
}
