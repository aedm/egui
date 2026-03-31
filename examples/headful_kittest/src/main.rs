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

use egui::accesskit::{Role, Toggled};
use egui_demo_lib::{View as _, WidgetGallery};
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT as _, Queryable as _, by};

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
    pause(&mut harness, 1.0);

    // ── TextEdit ──────────────────────────────────────────────────
    harness.get_by_role(Role::TextInput).click();
    harness.run();
    harness
        .get_by_role(Role::TextInput)
        .type_text("Hello from kittest!");
    harness.run();
    assert_eq!(
        harness.get_by_role(Role::TextInput).value(),
        Some("Hello from kittest!".to_owned()),
        "TextEdit should contain the typed text"
    );
    pause(&mut harness, 1.0);

    // ── Button "Click me!" (toggles `boolean`, checked via Checkbox) ─
    let checkbox = harness.get_by_role_and_label(Role::CheckBox, "Checkbox");
    assert_eq!(checkbox.accesskit_node().toggled(), Some(Toggled::False));

    harness
        .get_all(by().role(Role::Button).label("Click me!"))
        .next()
        .unwrap()
        .click();
    harness.run();

    let checkbox = harness.get_by_role_and_label(Role::CheckBox, "Checkbox");
    assert_eq!(
        checkbox.accesskit_node().toggled(),
        Some(Toggled::True),
        "Button 'Click me!' should have toggled `boolean` → true"
    );
    pause(&mut harness, 0.8);

    // ── Link "Click me!" (also toggles `boolean`) ─────────────────
    let has_link = harness
        .query_all(by().role(Role::Link).label("Click me!"))
        .next()
        .is_some();
    if has_link {
        harness
            .get_all(by().role(Role::Link).label("Click me!"))
            .next()
            .unwrap()
            .click();
        harness.run();

        let checkbox = harness.get_by_role_and_label(Role::CheckBox, "Checkbox");
        assert_eq!(
            checkbox.accesskit_node().toggled(),
            Some(Toggled::False),
            "Link 'Click me!' should have toggled `boolean` → false"
        );
        pause(&mut harness, 0.8);
    } else {
        // Link may be outside the visible area; toggle boolean back via Checkbox.
        harness
            .get_by_role_and_label(Role::CheckBox, "Checkbox")
            .click();
        harness.run();
        pause(&mut harness, 0.3);
    }

    // ── Checkbox ──────────────────────────────────────────────────
    harness
        .get_by_role_and_label(Role::CheckBox, "Checkbox")
        .click();
    harness.run();
    assert_eq!(
        harness
            .get_by_role_and_label(Role::CheckBox, "Checkbox")
            .accesskit_node()
            .toggled(),
        Some(Toggled::True),
        "Checkbox should be checked after click"
    );
    pause(&mut harness, 0.8);

    harness
        .get_by_role_and_label(Role::CheckBox, "Checkbox")
        .click();
    harness.run();
    assert_eq!(
        harness
            .get_by_role_and_label(Role::CheckBox, "Checkbox")
            .accesskit_node()
            .toggled(),
        Some(Toggled::False),
        "Checkbox should be unchecked after second click"
    );
    pause(&mut harness, 0.8);

    // ── RadioButtons ─────────────────────────────────────────────
    harness
        .get_by_role_and_label(Role::RadioButton, "Second")
        .click();
    harness.run();
    assert_eq!(
        harness
            .get_by_role_and_label(Role::RadioButton, "Second")
            .accesskit_node()
            .toggled(),
        Some(Toggled::True),
        "Radio 'Second' should be selected"
    );
    assert_eq!(
        harness
            .get_by_role_and_label(Role::RadioButton, "First")
            .accesskit_node()
            .toggled(),
        Some(Toggled::False),
        "Radio 'First' should be deselected"
    );
    pause(&mut harness, 0.8);

    harness
        .get_by_role_and_label(Role::RadioButton, "Third")
        .click();
    harness.run();
    assert_eq!(
        harness
            .get_by_role_and_label(Role::RadioButton, "Third")
            .accesskit_node()
            .toggled(),
        Some(Toggled::True),
        "Radio 'Third' should be selected"
    );
    pause(&mut harness, 0.8);

    // ── SelectableLabels (share `radio` state with RadioButtons) ─
    // SelectableLabels have Role::Button; use the fact that radio is currently "Third"
    // and click the "First" selectable to switch back.
    // There are two "First" buttons: SelectableLabel and ComboBox internal.
    // SelectableLabels come after RadioButtons in the tree, so pick accordingly.
    harness
        .get_all(by().role(Role::Button).label("First"))
        .next()
        .unwrap()
        .click();
    harness.run();
    assert_eq!(
        harness
            .get_by_role_and_label(Role::RadioButton, "First")
            .accesskit_node()
            .toggled(),
        Some(Toggled::True),
        "SelectableLabel 'First' should have switched radio back to First"
    );
    pause(&mut harness, 0.8);

    // ── ComboBox ─────────────────────────────────────────────────
    harness.get_by_role(Role::ComboBox).click();
    harness.run();
    pause(&mut harness, 0.5);

    // The dropdown shows three buttons; pick "Second".
    // There are now duplicate "Second" buttons (SelectableLabel + dropdown).
    // The dropdown items come last in the tree, so pick the last one.
    harness
        .get_all(by().role(Role::Button).label("Second"))
        .last()
        .unwrap()
        .click();
    harness.run();
    assert_eq!(
        harness
            .get_by_role_and_label(Role::RadioButton, "Second")
            .accesskit_node()
            .toggled(),
        Some(Toggled::True),
        "ComboBox selection should have switched radio to Second"
    );
    pause(&mut harness, 0.8);

    // ── Slider (focus via accesskit, then arrow keys) ──────────────
    let before = harness
        .get_by_role(Role::Slider)
        .accesskit_node()
        .numeric_value();

    harness.get_by_role(Role::Slider).focus();
    harness.run();

    for _ in 0..5 {
        harness.key_press(egui::Key::ArrowRight);
        harness.run();
    }
    let after = harness
        .get_by_role(Role::Slider)
        .accesskit_node()
        .numeric_value();
    assert!(
        after > before,
        "Slider value should have increased: {before:?} -> {after:?}"
    );
    pause(&mut harness, 0.8);

    // ── CollapsingHeader ─────────────────────────────────────────
    harness
        .get_by_label("Click to see what is hidden!")
        .click();
    // Use run_steps because the spinner inside requests continuous repaints.
    harness.run_steps(4);
    assert!(
        harness.query_by_label("Spinner").is_some(),
        "Collapsing header content should be visible"
    );
    pause(&mut harness, 0.8);

    // Close it again so the spinner stops requesting repaints.
    harness
        .get_by_label("Click to see what is hidden!")
        .click();
    harness.run_steps(4);
    assert!(
        harness.query_by_label("Spinner").is_none(),
        "Collapsing header should be closed"
    );
    pause(&mut harness, 0.5);

    // ── Button with image "Click me!" (also toggles boolean) ─────
    // It's the second Button with label "Click me!".
    let checkbox_before = harness
        .get_by_role_and_label(Role::CheckBox, "Checkbox")
        .accesskit_node()
        .toggled();
    harness
        .get_all(by().role(Role::Button).label("Click me!"))
        .nth(1)
        .unwrap()
        .click();
    harness.run();
    let checkbox_after = harness
        .get_by_role_and_label(Role::CheckBox, "Checkbox")
        .accesskit_node()
        .toggled();
    assert_ne!(
        checkbox_before, checkbox_after,
        "Image button 'Click me!' should have toggled `boolean`"
    );
    pause(&mut harness, 0.8);

    // ── Custom toggle switch (also toggles boolean) ──────────────
    let checkbox_before = harness
        .get_by_role_and_label(Role::CheckBox, "Checkbox")
        .accesskit_node()
        .toggled();
    // The toggle switch is a CheckBox with label "It's easy to create…" (the tooltip text),
    // but it might not have a stable label. Look for any CheckBox that is NOT "Checkbox",
    // "Interactive", or "Visible".
    let toggle = harness
        .get_all(by().role(Role::CheckBox))
        .find(|n| {
            let label = n.accesskit_node().label().unwrap_or_default();
            label != "Checkbox" && label != "Interactive" && label != "Visible"
        })
        .expect("Should find the custom toggle switch");
    toggle.click();
    harness.run();
    let checkbox_after = harness
        .get_by_role_and_label(Role::CheckBox, "Checkbox")
        .accesskit_node()
        .toggled();
    assert_ne!(
        checkbox_before, checkbox_after,
        "Custom toggle switch should have toggled `boolean`"
    );
    pause(&mut harness, 0.8);

    // ── "Interactive" checkbox (disables all widgets) ────────────
    harness
        .get_by_role_and_label(Role::CheckBox, "Interactive")
        .click();
    harness.run();
    // The main Checkbox should now be disabled.
    assert!(
        harness
            .get_by_role_and_label(Role::CheckBox, "Checkbox")
            .accesskit_node()
            .is_disabled(),
        "Widgets should be disabled after unchecking 'Interactive'"
    );
    pause(&mut harness, 1.0);

    // Re-enable.
    harness
        .get_by_role_and_label(Role::CheckBox, "Interactive")
        .click();
    harness.run();
    assert!(
        !harness
            .get_by_role_and_label(Role::CheckBox, "Checkbox")
            .accesskit_node()
            .is_disabled(),
        "Widgets should be enabled again"
    );
    pause(&mut harness, 0.8);

    // ── "Visible" checkbox (hides all widgets visually) ────────────
    harness
        .get_by_role_and_label(Role::CheckBox, "Visible")
        .click();
    harness.run();
    assert_eq!(
        harness
            .get_by_role_and_label(Role::CheckBox, "Visible")
            .accesskit_node()
            .toggled(),
        Some(Toggled::False),
        "'Visible' should be unchecked"
    );
    // The "Interactive" checkbox disappears when Visible is off.
    assert!(
        harness
            .query_by_role_and_label(Role::CheckBox, "Interactive")
            .is_none(),
        "'Interactive' checkbox should be gone when not visible"
    );
    pause(&mut harness, 1.0);

    // Re-show.
    harness
        .get_by_role_and_label(Role::CheckBox, "Visible")
        .click();
    harness.run();
    assert_eq!(
        harness
            .get_by_role_and_label(Role::CheckBox, "Visible")
            .accesskit_node()
            .toggled(),
        Some(Toggled::True),
        "'Visible' should be checked again"
    );
    assert!(
        harness
            .query_by_role_and_label(Role::CheckBox, "Interactive")
            .is_some(),
        "'Interactive' checkbox should be back"
    );
    pause(&mut harness, 0.8);

    println!("All assertions passed. Demo finished.");
}
