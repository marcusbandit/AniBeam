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
use i_slint_core::items::{AccessibleRole, Opacity, TextInput};
use i_slint_core::window::WindowInner;
use slint::platform::{Key as SlintKey, PointerEventButton, WindowEvent};
use slint::{ComponentHandle, LogicalPosition, Model, SharedString, Timer, TimerMode};

use super::command::{Command, Dir, Field, HELP, Key, Onboarding, Series, Tab, Target};
use crate::{
    AiringPick, AppData, AppWindow, Ask, DiskData, FindData, Fold, MoveData, Nav, PlayerData,
    Settings, ShowData,
};

/// Onboarding's pages, `steps` in onboarding.slint.
const ONBOARDING_STEPS: i32 = 8;
/// How long a finger stays down for a long press: past Slint's and the app's hold.
const HOLD: Duration = Duration::from_millis(800);
/// How many frames the stats keep: a few seconds of animation.
const FRAMES_KEPT: usize = 600;

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

/// One element a screen reader would see.
struct Element {
    item: ItemRc,
    role: AccessibleRole,
    label: String,
    value: Option<String>,
    enabled: bool,
    checked: Option<bool>,
    focused: bool,
    /// Top left and size in the window's logical pixels.
    rect: (f32, f32, f32, f32),
    /// Where a tap on it lands.
    centre: (f32, f32),
    /// On screen: not hidden, not clipped or scrolled away, not faded out.
    shown: bool,
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
        if self.focused {
            s.push_str(" focused");
        }
        if !self.enabled {
            s.push_str(" disabled");
        }
        match self.checked {
            Some(true) => s.push_str(" on"),
            Some(false) => s.push_str(" off"),
            None => {}
        }
        if !self.shown {
            s.push_str(" offscreen");
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
}

/// A long label, cut to what a line of the dump can hold.
fn clip(s: &str) -> String {
    let flat: String = s.split_whitespace().collect::<Vec<_>>().join(" ");
    if flat.chars().count() > 90 {
        format!("{}...", flat.chars().take(87).collect::<String>())
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

fn window_size(w: &AppWindow) -> (f32, f32) {
    let size = w.window().size().to_logical(w.window().scale_factor());
    (size.width, size.height)
}

/// The opacity the item draws at, its parents' included.
fn opacity(item: &ItemRc) -> f32 {
    let mut opacity = 1.0;
    let mut current = Some(item.clone());
    while let Some(i) = current {
        if let Some(o) = i.downcast::<Opacity>() {
            opacity *= o.as_pin_ref().opacity();
        }
        current = i.parent_item(ParentItemTraversalMode::StopAtPopups);
    }
    opacity
}

/// Every accessible element in the window, in the tree's order.
fn elements(w: &AppWindow) -> Vec<Element> {
    let (width, height) = window_size(w);
    let root = ItemRc::new_root(WindowInner::from_pub(w.window()).component());
    let mut out = Vec::new();
    root.visit_descendants(|item| {
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
        let rect = (origin.x, origin.y, g.size.width, g.size.height);
        let on_window =
            centre.x >= 0.0 && centre.y >= 0.0 && centre.x <= width && centre.y <= height;
        let shown = on_window && item.is_visible() && opacity(item) > 0.05;
        let focused = item
            .downcast::<TextInput>()
            .is_some_and(|t| t.as_pin_ref().has_focus());
        out.push(Element {
            item: item.clone(),
            role,
            label,
            value: prop(AccessibleStringProperty::Value)
                .filter(|v| !v.is_empty() || role == AccessibleRole::TextInput),
            enabled: prop(AccessibleStringProperty::Enabled).as_deref() != Some("false"),
            checked: prop(AccessibleStringProperty::Checkable)
                .filter(|c| c == "true")
                .map(|_| prop(AccessibleStringProperty::Checked).as_deref() == Some("true")),
            focused,
            rect,
            centre: (centre.x, centre.y),
            shown,
        });
        ControlFlow::Continue(())
    });
    out
}

/// The elements worth listing: on screen, and without the text inside a button that names
/// it already.
fn listed(all: Vec<Element>) -> Vec<Element> {
    let shown: Vec<Element> = all.into_iter().filter(|e| e.shown).collect();
    let keep: Vec<bool> = shown
        .iter()
        .map(|e| {
            e.role != AccessibleRole::Text
                || !shown.iter().any(|o| {
                    o.role != AccessibleRole::Text
                        && norm(&o.label) == norm(&e.label)
                        && o.holds(e.centre.0, e.centre.1)
                })
        })
        .collect();
    shown
        .into_iter()
        .zip(keep)
        .filter_map(|(e, k)| k.then_some(e))
        .collect()
}

/// The shown elements a label names: whole-label matches if any, else the ones holding it,
/// the controls before plain text.
fn matching(w: &AppWindow, label: &str) -> Vec<Element> {
    let want = norm(label);
    let list = listed(elements(w));
    let names = |e: &Element| norm(&e.label) == want;
    let mut hits: Vec<Element> = if list.iter().any(names) {
        list.into_iter().filter(names).collect()
    } else {
        list.into_iter()
            .filter(|e| norm(&e.label).contains(&want))
            .collect()
    };
    hits.sort_by_key(|e| e.role == AccessibleRole::Text);
    hits
}

/// Where a touch lands: the element, if a label named one, the point, and how to say it.
type Landing = (Option<Element>, (f32, f32), String);

/// What a target lands on.
fn resolve(w: &AppWindow, target: &Target) -> Result<Landing, String> {
    match target {
        Target::Point(x, y) => Ok((None, (*x, *y), format!("{x:.0},{y:.0}"))),
        Target::Label { label, nth } => {
            let mut hits = matching(w, label);
            if hits.is_empty() {
                return Err(format!(
                    "nothing on screen is labelled {label:?}; dump lists what is"
                ));
            }
            let count = hits.len();
            if *nth > count {
                return Err(format!("{label:?} matches {count}, not {nth}"));
            }
            let e = hits.remove(nth - 1);
            let mut said = e.line();
            if count > 1 {
                said.push_str(&format!(" (match {nth} of {count}; #n picks another)"));
            }
            let centre = e.centre;
            Ok((Some(e), centre, said))
        }
    }
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

fn key(w: &AppWindow, k: Key) {
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
    w.window()
        .dispatch_event(WindowEvent::KeyPressed { text: text.clone() });
    w.window().dispatch_event(WindowEvent::KeyReleased { text });
}

/// The page and everything open over it, in one line.
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
    parts.extend(overlays(w));
    parts.join(", ")
}

/// What is open over the page, from the top down.
fn overlays(w: &AppWindow) -> Vec<String> {
    let mut out = Vec::new();
    let data = w.global::<AppData>();
    let ask = w.global::<Ask>();
    if w.global::<PlayerData>().get_open() {
        out.push("player".into());
    }
    if data.get_notes_open() {
        out.push("what's new".into());
    }
    if data.get_onboarding() {
        out.push(format!(
            "onboarding step {} of {ONBOARDING_STEPS}",
            data.get_step()
        ));
    }
    for (open, name) in [
        (ask.get_mobile(), "mobile data question"),
        (data.get_notify_ask(), "notification question"),
        (ask.get_menu_id() >= 0, "show menu"),
        (ask.get_mark_open(), "mark menu"),
        (ask.get_versions_open(), "versions menu"),
        (data.get_airing_menu().show, "airing menu"),
        (w.global::<AiringPick>().get_open(), "season picker"),
        (w.global::<MoveData>().get_open(), "list move sheet"),
    ] {
        if open {
            out.push(name.into());
        }
    }
    let sheet = w.global::<ShowData>().get_sheet();
    if sheet != 0 {
        let name = match sheet {
            1 => "list status",
            2 => "score",
            3 => "related show",
            4 => "notes",
            5 => "fixes",
            6 => "match",
            7 => "episodes and dates",
            _ => "sheet",
        };
        out.push(format!("show sheet {name}"));
    }
    if w.global::<DiskData>().get_sheet() != 0 {
        out.push(format!("disk sheet {}", w.global::<DiskData>().get_sheet()));
    }
    out
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
    let list: Vec<Element> = listed(elements(w))
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
    out.push(format!("shown: {}", list.len()));
    out.extend(list.iter().map(|e| format!("  {}", e.line())));
    out.join("\n")
}

/// For `wait`: the first shown element the label names, if one shows yet.
pub fn probe(label: &str) -> Option<String> {
    let w = window()?;
    matching(&w, label).first().map(Element::line)
}

/// The series on the phone a title names: a whole-title match first, then one holding it.
fn series_by_title(w: &AppWindow, title: &str) -> Option<(i32, String)> {
    let data = w.global::<AppData>();
    let mut tiles: Vec<(i32, String)> = Vec::new();
    let library = data.get_library();
    tiles.extend(library.iter().map(|t| (t.id, t.title.to_string())));
    for shelf in data.get_home_shelves().iter() {
        tiles.extend(shelf.tiles.iter().map(|t| (t.id, t.title.to_string())));
    }
    tiles.retain(|(id, _)| *id >= 0);
    let want = norm(title);
    tiles
        .iter()
        .find(|(_, t)| norm(t) == want)
        .or_else(|| tiles.iter().find(|(_, t)| norm(t).contains(&want)))
        .cloned()
}

fn fields(w: &AppWindow) -> Vec<Element> {
    elements(w)
        .into_iter()
        .filter(|e| e.shown && e.role == AccessibleRole::TextInput)
        .collect()
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
                Timer::single_shot(Duration::ZERO, || {
                    SWIPE.with(|s| drop(s.borrow_mut().take()))
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
        Command::Back => {
            key(w, Key::Back);
            where_line(w)
        }
        Command::Key(keys) => {
            for k in keys {
                key(w, k);
            }
            where_line(w)
        }
        Command::Open(series) => {
            let id = match series {
                Series::Id(id) => id,
                Series::Title(title) => {
                    series_by_title(w, &title)
                        .ok_or_else(|| format!("no series on the phone is titled {title:?}"))?
                        .0
                }
            };
            data.invoke_open_series(id);
            if !data.get_series_open() || data.get_series().id != id {
                return Err(format!(
                    "series {id} did not open; open takes the phone's ids"
                ));
            }
            where_line(w)
        }
        Command::Press(target) => {
            let (e, point, said) = resolve(w, &target)?;
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
            w.window().dispatch_event(WindowEvent::KeyPressed {
                text: text.as_str().into(),
            });
            w.window().dispatch_event(WindowEvent::KeyReleased {
                text: text.as_str().into(),
            });
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
                    .ok_or_else(|| format!("there is no field {n} on screen; dump lists them"))?,
                Field::Named(name) => {
                    let want = norm(name);
                    // A field shows its placeholder as a text over it while it is empty.
                    let marks: Vec<(f32, f32)> = listed(elements(w))
                        .into_iter()
                        .filter(|e| {
                            e.role == AccessibleRole::Text && norm(&e.label).contains(&want)
                        })
                        .map(|e| e.centre)
                        .collect();
                    all.into_iter()
                        .find(|f| {
                            norm(&f.label).contains(&want)
                                || f.value.as_deref().is_some_and(|v| norm(v) == want)
                                || marks.iter().any(|(x, y)| {
                                    let (fx, fy, fw, fh) = f.rect;
                                    // The placeholder sits on the field's line, left of it
                                    // by its glyph at most.
                                    *y >= fy && *y <= fy + fh && *x >= fx - 80.0 && *x <= fx + fw
                                })
                        })
                        .ok_or_else(|| format!("no field on screen shows {name:?}"))?
                }
            };
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
            let (point, said) = match place {
                Some(t) => {
                    let (_, p, said) = resolve(w, &t)?;
                    (p, format!(" at {said}"))
                }
                None => {
                    let (width, height) = window_size(w);
                    ((width / 2.0, height / 2.0), String::new())
                }
            };
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
            let dir = format!("{dir:?}").to_lowercase();
            format!("scrolled {dir} {amount:.0} px{said}")
        }
        Command::Onboarding(step) => {
            match step {
                Onboarding::Status => {}
                Onboarding::Start => data.invoke_restart_onboarding(),
                Onboarding::Finish => data.invoke_finish_onboarding(),
                _ if !data.get_onboarding() => {
                    return Err("onboarding is not open; onboarding start opens it".into());
                }
                Onboarding::Next if data.get_step() >= ONBOARDING_STEPS - 1 => {
                    data.invoke_finish_onboarding()
                }
                Onboarding::Next => data.set_step(data.get_step() + 1),
                Onboarding::Back => data.set_step((data.get_step() - 1).max(0)),
                Onboarding::Step(n) => {
                    if n >= ONBOARDING_STEPS {
                        return Err(format!(
                            "onboarding has steps 0 to {}",
                            ONBOARDING_STEPS - 1
                        ));
                    }
                    data.set_step(n)
                }
            }
            if data.get_onboarding() {
                format!("onboarding step {} of {ONBOARDING_STEPS}", data.get_step())
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
                    norm(&e.label).contains(&want)
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

        // The dump names the page and what shows, the bar's tabs among it.
        let dump = say("tree");
        assert!(dump.starts_with("page home"), "{dump}");
        assert_eq!(say("where").lines().count(), 1);
        assert!(dump.contains("window 400x900, folded"), "{dump}");
        let downloads = tr_in(Lang::En, "phone-nav-downloads", &[]);
        assert!(dump.contains(&format!("text {downloads:?}")), "{dump}");
        let only = say(&format!("tree {downloads:?}"));
        assert!(only.contains(&downloads), "{only}");
        assert!(only.lines().count() < dump.lines().count(), "{only}");

        // A click on the bar's label taps it, and the tab opens.
        let pressed = say(&format!("click {downloads:?}"));
        assert!(pressed.starts_with("tapped text"), "{pressed}");
        assert_eq!(w.global::<Nav>().get_page(), 2, "{pressed}");

        // A search lands in Downloads with its words in the field.
        say("go home");
        assert!(say("go downloads frieren").contains("page downloads"));
        assert_eq!(w.global::<FindData>().get_text(), "frieren");

        // Onboarding steps forward and back, and its Continue presses like a button.
        data.set_onboarding(true);
        data.set_step(0);
        assert_eq!(
            say("onboarding next"),
            format!("onboarding step 1 of {ONBOARDING_STEPS}")
        );
        assert_eq!(
            say("onboarding step 4"),
            format!("onboarding step 4 of {ONBOARDING_STEPS}")
        );
        let next = tr_in(Lang::En, "onb-continue", &[]);
        let pressed = say(&format!("click {next:?}"));
        // The app's Button declares no accessible action, so the click taps its label, and the
        // touch reaches the button under it as a finger would.
        assert!(pressed.starts_with("tapped text"), "{pressed}");
        assert_eq!(data.get_step(), 5, "{pressed}");
        assert!(say("onboarding back").ends_with("step 4 of 8"));
        assert!(say("tree").contains("onboarding step 4 of 8"));

        // What is not there says so.
        assert!(say("click \"No such control\"").starts_with("error: nothing on screen"));
        assert!(say("go series 99999").starts_with("error"));
        assert!(say("type hello").starts_with("error: no field has the focus"));
        assert!(say("find zzzz-nothing").starts_with("error"));
    }
}
