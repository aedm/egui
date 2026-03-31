//! Demonstrates **headful mode** in `egui_kittest`.
//!
//! Headful mode opens a real desktop window so you can watch the test harness
//! drive a UI, just as a human user would: clicking buttons, typing text,
//! toggling checkboxes, and navigating combo-boxes.
//!
//! This example creates the [`egui_demo_lib::WidgetGallery`] and interacts with
//! it step by step.
//!
//! Run with:
//! ```sh
//! cargo run -p headful_kittest
//! ```

use std::time::Duration;

use egui::accesskit::Role;
use egui_demo_lib::{View as _, WidgetGallery};
use egui_kittest::Harness;
use egui_kittest::kittest::{Queryable as _, by};

/// Advance the harness for a wall-clock duration so the window stays visible.
fn pause(harness: &mut Harness<'_, WidgetGallery>, seconds: f32) {
    let steps = (seconds * 30.0) as usize;
    for _ in 0..steps {
        harness.step();
        std::thread::sleep(Duration::from_secs_f32(1.0 / 30.0));
    }
}

fn main() {
    let mut harness = Harness::builder()
        .with_size(egui::Vec2::new(420.0, 520.0))
        .headful("Widget Gallery – headful kittest demo")
        .build_ui_state(
            |ui, gallery: &mut WidgetGallery| {
                egui_extras::install_image_loaders(ui.ctx());
                gallery.ui(ui);
            },
            WidgetGallery::default(),
        );

    // Let the user see the initial state.
    pause(&mut harness, 1.5);

    // --- Type into the TextEdit ---
    harness.get_by_role(Role::TextInput).click();
    pause(&mut harness, 0.3);
    harness.get_by_role(Role::TextInput).type_text("Hello from kittest!");
    pause(&mut harness, 1.5);

    // --- Click "Click me!" button (there are two: plain + image, pick the first) ---
    harness
        .get_all(by().role(Role::Button).label("Click me!"))
        .next()
        .unwrap()
        .click();
    pause(&mut harness, 1.0);

    // --- Toggle the checkbox ---
    harness
        .get_by_role_and_label(Role::CheckBox, "Checkbox")
        .click();
    pause(&mut harness, 0.8);
    harness
        .get_by_role_and_label(Role::CheckBox, "Checkbox")
        .click();
    pause(&mut harness, 0.8);

    // --- Select radio buttons one by one ---
    harness
        .get_by_role_and_label(Role::RadioButton, "Second")
        .click();
    pause(&mut harness, 0.8);
    harness
        .get_by_role_and_label(Role::RadioButton, "Third")
        .click();
    pause(&mut harness, 0.8);

    // --- Open the collapsing header ---
    harness
        .get_by_label("Click to see what is hidden!")
        .click();
    pause(&mut harness, 1.5);

    // --- Disable the "Interactive" checkbox to show the disabled look ---
    harness
        .get_by_role_and_label(Role::CheckBox, "Interactive")
        .click();
    pause(&mut harness, 1.5);

    // --- Re-enable ---
    harness
        .get_by_role_and_label(Role::CheckBox, "Interactive")
        .click();
    pause(&mut harness, 1.0);

    println!("Demo finished.");
}
