//! A tap outside a text field ends editing, so the phone's keyboard goes away (#1172). Slint
//! keeps a TextInput focused until something else takes focus, and a plain tap on a label or
//! a row takes none, so the keyboard stayed up over the controls under it.

use i_slint_backend_testing::ElementHandle;
use slint::platform::{PointerEventButton, WindowEvent};
use slint::{ComponentHandle, LogicalPosition};

use crate::{AppData, AppWindow, RepoData, Tr};

/// A finger down and up at one point. The changed handlers that move focus run once the
/// event is handled, as they would before the next frame.
fn tap(page: &AppWindow, x: f32, y: f32) {
    let position = LogicalPosition::new(x, y);
    let button = PointerEventButton::Left;
    page.window()
        .dispatch_event(WindowEvent::PointerPressed { position, button });
    slint::platform::update_timers_and_animations();
    page.window()
        .dispatch_event(WindowEvent::PointerReleased { position, button });
    page.window().dispatch_event(WindowEvent::PointerExited);
    slint::platform::update_timers_and_animations();
}

fn type_text(page: &AppWindow, text: &str) {
    for c in text.chars() {
        let text = c.to_string();
        page.window().dispatch_event(WindowEvent::KeyPressed {
            text: text.clone().into(),
        });
        page.window()
            .dispatch_event(WindowEvent::KeyReleased { text: text.into() });
    }
}

fn centre(e: &ElementHandle) -> (f32, f32) {
    let p = e.absolute_position();
    let s = e.size();
    (p.x + s.width / 2.0, p.y + s.height / 2.0)
}

/// A step of the first run, tall enough that nothing on it is scrolled away.
fn step(n: i32) -> AppWindow {
    let page = AppWindow::new().unwrap();
    page.window()
        .set_size(slint::LogicalSize::new(400.0, 2400.0));
    // Each label reads as its catalog id, so a button can be found by it.
    page.global::<Tr>().on_lookup(|id, _rev, _args| id);
    let app = page.global::<AppData>();
    app.set_onboarding(true);
    app.set_step(n);
    slint::platform::update_timers_and_animations();
    page
}

fn fields(page: &AppWindow) -> Vec<ElementHandle> {
    ElementHandle::find_by_element_type_name(page, "LineInput")
        .filter(|e| e.size().width > 0.0 && e.size().height > 0.0)
        .collect()
}

/// The computer step with its address field focused and holding "a".
fn typing_an_address() -> (AppWindow, ElementHandle) {
    let page = step(7);
    let field = fields(&page)
        .into_iter()
        .next()
        .expect("no field on the computer step");
    let (fx, fy) = centre(&field);
    tap(&page, fx, fy);
    type_text(&page, "a");
    assert_eq!(
        page.global::<AppData>().get_host_address(),
        "a",
        "the tap did not focus the field"
    );
    (page, field)
}

#[test]
fn a_tap_on_the_page_ends_editing() {
    i_slint_backend_testing::init_no_event_loop();
    let (page, field) = typing_an_address();
    // A tap on the step's title, plain text over the page's empty space.
    let title = ElementHandle::find_by_accessible_label(&page, "onb-computer-title")
        .find(|e| e.size().width > 0.0 && e.size().height > 0.0)
        .expect("no title on the computer step");
    assert!(
        title.absolute_position().y + title.size().height < field.absolute_position().y,
        "the title is not above the field"
    );
    let (tx, ty) = centre(&title);
    tap(&page, tx, ty);
    type_text(&page, "b");
    assert_eq!(
        page.global::<AppData>().get_host_address(),
        "a",
        "the field kept the keys after a tap outside it"
    );
}

/// The tap that ends the editing still does what it was aimed at: Back goes back a step at
/// once, rather than only putting the keyboard away.
#[test]
fn a_tap_on_a_button_ends_editing_and_presses_it() {
    i_slint_backend_testing::init_no_event_loop();
    let (page, _) = typing_an_address();
    let back = ElementHandle::find_by_accessible_label(&page, "onb-back")
        .find(|e| e.size().width > 0.0 && e.size().height > 0.0)
        .expect("no Back on the computer step");
    let (bx, by) = centre(&back);
    tap(&page, bx, by);
    assert_eq!(
        page.global::<AppData>().get_step(),
        6,
        "Back did not take the tap"
    );
}

/// A tap on the field's own glass, beside the text, keeps typing into it.
#[test]
fn a_tap_on_the_fields_edge_keeps_editing() {
    i_slint_backend_testing::init_no_event_loop();
    let (page, field) = typing_an_address();
    let glass = ElementHandle::find_by_element_type_name(&page, "Field")
        .find(|e| e.size().width > 0.0)
        .expect("no field glass on the computer step");
    let p = glass.absolute_position();
    let (_, fy) = centre(&glass);
    // Just inside the glass's left edge, left of the text.
    assert!(
        p.x + 2.0 < field.absolute_position().x,
        "the text starts at the glass's edge"
    );
    tap(&page, p.x + 2.0, fy);
    type_text(&page, "b");
    assert_eq!(
        page.global::<AppData>().get_host_address(),
        "ab",
        "a tap on the field's edge took the keys away"
    );
}

/// Moving from one field to the next keeps the keys with the field tapped: the sites step's
/// by-hand name, then its address.
#[test]
fn a_tap_from_one_field_to_another_types_into_the_second() {
    i_slint_backend_testing::init_no_event_loop();
    let page = step(6);
    let all = fields(&page);
    assert!(all.len() >= 2, "the sites step shows {} fields", all.len());
    let (name, url) = (&all[all.len() - 2], &all[all.len() - 1]);
    let repos = page.global::<RepoData>();

    let (x, y) = centre(name);
    tap(&page, x, y);
    type_text(&page, "n");
    assert_eq!(
        repos.get_custom_name(),
        "n",
        "the name field did not take the keys"
    );

    let (x, y) = centre(url);
    tap(&page, x, y);
    type_text(&page, "u");
    assert_eq!(repos.get_custom_name(), "n", "the name field kept the keys");
    assert_eq!(
        repos.get_custom_url(),
        "u",
        "the address field did not take the keys"
    );
}
