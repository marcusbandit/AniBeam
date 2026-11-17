//! A tap outside a text field ends editing, so the phone's keyboard goes away (#1172). Slint
//! keeps a TextInput focused until something else takes focus, and a plain tap on a label or
//! a row takes none, so the keyboard stayed up over the controls under it.

use i_slint_backend_testing::ElementHandle;
use slint::platform::{PointerEventButton, WindowEvent};
use slint::{ComponentHandle, LogicalPosition};

use crate::{AppData, AppWindow, Tr};

fn tap(page: &AppWindow, x: f32, y: f32) {
    let position = LogicalPosition::new(x, y);
    let button = PointerEventButton::Left;
    page.window()
        .dispatch_event(WindowEvent::PointerPressed { position, button });
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

/// The computer step of the first run: one address field under the step's title.
fn computer_step() -> AppWindow {
    let page = AppWindow::new().unwrap();
    page.window()
        .set_size(slint::LogicalSize::new(400.0, 900.0));
    // Each label reads as its catalog id, so every button has a size to tap.
    page.global::<Tr>().on_lookup(|id, _rev, _args| id);
    let app = page.global::<AppData>();
    app.set_onboarding(true);
    app.set_step(7);
    slint::platform::update_timers_and_animations();
    page
}

#[test]
fn a_tap_on_the_page_ends_editing() {
    i_slint_backend_testing::init_no_event_loop();
    let page = computer_step();
    let field = ElementHandle::find_by_element_type_name(&page, "LineInput")
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

    // A tap well above the field, on the step's title.
    tap(&page, 200.0, field.absolute_position().y - 120.0);
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
    let page = computer_step();
    let field = ElementHandle::find_by_element_type_name(&page, "LineInput")
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

    let back = ElementHandle::find_by_element_type_name(&page, "TextButton")
        .find(|e| e.size().width > 0.0 && e.size().height > 0.0)
        .expect("no Back on the computer step");
    let (bx, by) = centre(&back);
    tap(&page, bx, by);
    assert_eq!(
        page.global::<AppData>().get_step(),
        6,
        "Back did not take the tap"
    );
    type_text(&page, "b");
    assert_eq!(
        page.global::<AppData>().get_host_address(),
        "a",
        "the field kept the keys after Back"
    );
}
