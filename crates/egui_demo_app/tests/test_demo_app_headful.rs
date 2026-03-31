//! Headful variant of the demo app test.
//!
//! Uses `harness = false` so it runs on the main thread (required for headful
//! mode on macOS). Set `KITTEST_HEADFUL=1` to see the window:
//!
//! ```sh
//! KITTEST_HEADFUL=1 cargo test -p egui_demo_app --test test_demo_app_headful
//! ```

use std::time::Duration;

use egui::accesskit::Role;
use egui::{CentralPanel, Color32, Key, Modifiers, Popup, Vec2};
use egui_demo_lib::{ColorTest, DemoWindows, View as _, WidgetGallery};
use egui_demo_app::{Anchor, WrapApp};
use egui_kittest::Harness;
use egui_kittest::kittest::{Queryable as _, by};

fn is_headful() -> bool {
    std::env::var("KITTEST_HEADFUL").is_ok()
}

fn pause<S>(harness: &mut Harness<'_, S>, seconds: f32) {
    if !is_headful() {
        return;
    }
    let steps = (seconds * 30.0) as usize;
    for _ in 0..steps {
        harness.step();
        std::thread::sleep(Duration::from_secs_f32(1.0 / 30.0));
    }
}

/// Open a demo by partial name inside a DemoWindows harness.
/// Targets the sidebar Button (checkbox), not the Window itself.
fn open_demo(harness: &mut Harness<'_>, name: &str) {
    harness
        .get_all(by().role(Role::Button).label_contains(name))
        .next()
        .unwrap()
        .click_accesskit();
    harness.run_ok();
    pause(harness, 0.3);
}

/// Close a demo by clicking its sidebar checkbox again.
fn close_demo(harness: &mut Harness<'_>, name: &str) {
    harness
        .get_all(by().role(Role::Button).label_contains(name))
        .next()
        .unwrap()
        .click_accesskit();
    harness.run_ok();
}

// ─── All demos in one shared DemoWindows harness ─────────────────────

fn test_all_demos() {
    let mut demo_windows = DemoWindows::default();
    let mut harness = Harness::builder()
        .with_size(Vec2::new(900.0, 700.0))
        .build_ui(move |ui| {
            egui_extras::install_image_loaders(ui.ctx());
            demo_windows.ui(ui);
        });
    harness.run_ok();
    pause(&mut harness, 0.5);

    // ── Widget Gallery (starts open by default — close others first) ──
    // Widget Gallery is already open, so just interact with it directly.
    // First close all other default-open windows.
    harness.run_ok();

    // Checkbox
    harness
        .get_by_role_and_label(Role::CheckBox, "Checkbox")
        .click();
    harness.run_ok();
    pause(&mut harness, 0.3);
    harness
        .get_by_role_and_label(Role::CheckBox, "Checkbox")
        .click();
    harness.run_ok();
    pause(&mut harness, 0.2);

    // Radio buttons
    harness
        .get_by_role_and_label(Role::RadioButton, "Second")
        .click();
    harness.run_ok();
    pause(&mut harness, 0.2);
    harness
        .get_by_role_and_label(Role::RadioButton, "Third")
        .click();
    harness.run_ok();
    pause(&mut harness, 0.2);

    // Slider
    harness.get_by_role(Role::Slider).focus();
    harness.run_ok();
    for _ in 0..5 {
        harness.key_press(Key::ArrowRight);
        harness.run_ok();
    }
    pause(&mut harness, 0.3);

    // Collapsing header
    harness
        .get_by_label("Click to see what is hidden!")
        .click();
    harness.run_steps(4);
    pause(&mut harness, 0.5);
    harness
        .get_by_label("Click to see what is hidden!")
        .click();
    harness.run_steps(4);
    pause(&mut harness, 0.2);

    // Interactive toggle
    harness
        .get_by_role_and_label(Role::CheckBox, "Interactive")
        .click();
    harness.run_ok();
    pause(&mut harness, 0.5);
    harness
        .get_by_role_and_label(Role::CheckBox, "Interactive")
        .click();
    harness.run_ok();
    pause(&mut harness, 0.3);

    close_demo(&mut harness, "Widget Gallery");

    // ── Sliders ──────────────────────────────────────────────────
    open_demo(&mut harness, "Sliders");

    harness.get_by_label("Assign PI").click();
    harness.run_ok();
    pause(&mut harness, 0.3);

    harness.get_by_label("Toggle trailing color").click();
    harness.run_ok();
    pause(&mut harness, 0.2);

    harness.get_by_label("Logarithmic").click();
    harness.run_ok();
    pause(&mut harness, 0.2);

    harness.get_by_label("Smart Aim").click();
    harness.run_ok();
    pause(&mut harness, 0.2);

    harness.get_by_label("i32").click();
    harness.run_ok();
    pause(&mut harness, 0.3);

    harness.get_by_label("Vertical").click();
    harness.run_ok();
    pause(&mut harness, 0.3);

    harness.get_by_label("Horizontal").click();
    harness.run_ok();
    pause(&mut harness, 0.2);

    harness.get_by_label("Edits").click();
    harness.run_ok();
    pause(&mut harness, 0.2);

    harness.get_by_label("Always").click();
    harness.run_ok();
    pause(&mut harness, 0.2);

    close_demo(&mut harness, "Sliders");

    // ── Modals ───────────────────────────────────────────────────
    open_demo(&mut harness, "Modals");

    harness.get_by_label("Open User Modal").click();
    harness.run_ok();
    pause(&mut harness, 0.5);

    // Open combobox, then Escape
    harness
        .get_by_role_and_label(Role::ComboBox, "Role")
        .click();
    harness.run_ok();
    assert!(Popup::is_any_open(&harness.ctx));
    pause(&mut harness, 0.3);

    harness.key_press(Key::Escape);
    harness.run_ok();
    assert!(!Popup::is_any_open(&harness.ctx));
    pause(&mut harness, 0.2);

    // Type into name field
    let has_input = harness.query_all(by().role(Role::TextInput)).next().is_some();
    if has_input {
        harness
            .get_all(by().role(Role::TextInput))
            .next()
            .unwrap()
            .focus();
        harness.key_press_modifiers(Modifiers::COMMAND, Key::A);
        harness
            .get_all(by().role(Role::TextInput))
            .next()
            .unwrap()
            .type_text("Alice");
        harness.run_ok();
        pause(&mut harness, 0.3);
    }

    // Save → Yes Please
    harness.get_by_label("Save").click();
    harness.run_ok();
    pause(&mut harness, 0.3);

    harness.get_by_label("Yes Please").click();
    harness.run_ok();
    pause(&mut harness, 0.3);

    close_demo(&mut harness, "Modals");

    // ── Scrolling ────────────────────────────────────────────────
    open_demo(&mut harness, "Scrolling");

    for tab in ["Scroll to", "Stick to end", "Bidirectional", "Appearance"] {
        harness.get_by_label(tab).click();
        harness.run_ok();
        pause(&mut harness, 0.3);
    }

    close_demo(&mut harness, "Scrolling");

    // ── Interactive Container ────────────────────────────────────
    open_demo(&mut harness, "Interactive Container");

    harness.get_by_label("Reset").click();
    harness.run_ok();
    pause(&mut harness, 0.2);

    harness.get_by_label("+ 100").click();
    harness.run_ok();
    pause(&mut harness, 0.2);

    harness.get_by_label("+ 100").click();
    harness.run_ok();
    pause(&mut harness, 0.2);

    close_demo(&mut harness, "Interactive Container");

    // ── Table ────────────────────────────────────────────────────
    open_demo(&mut harness, "Table");

    harness.get_by_label("Striped").click();
    harness.run_ok();
    pause(&mut harness, 0.2);

    harness.get_by_label("Resizable columns").click();
    harness.run_ok();
    pause(&mut harness, 0.2);

    harness.get_by_label("Clickable rows").click();
    harness.run_ok();
    pause(&mut harness, 0.2);

    harness
        .get_by_label("Thousands of rows of same height")
        .click();
    harness.run_ok();
    pause(&mut harness, 0.3);

    harness.get_by_label("Few, manual rows").click();
    harness.run_ok();
    pause(&mut harness, 0.2);

    close_demo(&mut harness, "Table");

    // ── Code Editor ──────────────────────────────────────────────
    open_demo(&mut harness, "Code Editor");

    let has_editor = harness.query_all(by().role(Role::TextInput)).next().is_some();
    if has_editor {
        harness
            .get_all(by().role(Role::TextInput))
            .next()
            .unwrap()
            .focus();
        harness.run_ok();
        pause(&mut harness, 0.3);
    }

    close_demo(&mut harness, "Code Editor");

    // ── Font Book ────────────────────────────────────────────────
    open_demo(&mut harness, "Font Book");

    let has_filter = harness.query_all(by().role(Role::TextInput)).next().is_some();
    if has_filter {
        harness
            .get_all(by().role(Role::TextInput))
            .next()
            .unwrap()
            .click();
        harness.run_ok();
        harness
            .get_all(by().role(Role::TextInput))
            .next()
            .unwrap()
            .type_text("arrow");
        harness.run_ok();
        pause(&mut harness, 0.5);
    }

    close_demo(&mut harness, "Font Book");

    // ── Window Options ───────────────────────────────────────────
    open_demo(&mut harness, "Window Options");

    for label in ["title_bar", "closable", "collapsible", "resizable"] {
        if let Some(cb) = harness.query_by_label(label) {
            cb.click();
            harness.run_ok();
            pause(&mut harness, 0.2);
        }
    }
    for label in ["title_bar", "closable", "collapsible", "resizable"] {
        if let Some(cb) = harness.query_by_label(label) {
            cb.click();
            harness.run_ok();
        }
    }
    pause(&mut harness, 0.3);

    close_demo(&mut harness, "Window Options");

    // ── Tooltips ─────────────────────────────────────────────────
    open_demo(&mut harness, "Tooltips");

    if let Some(cb) = harness.query_by_role_and_label(Role::CheckBox, "Enabled") {
        cb.click();
        harness.run_ok();
        pause(&mut harness, 0.3);

        harness
            .get_by_role_and_label(Role::CheckBox, "Enabled")
            .click();
        harness.run_ok();
        pause(&mut harness, 0.2);
    }

    close_demo(&mut harness, "Tooltips");

    // ── Text Layout ──────────────────────────────────────────────
    open_demo(&mut harness, "Text Layout");

    harness.get_by_label("word boundaries").click();
    harness.run_ok();
    pause(&mut harness, 0.2);

    harness.get_by_label("anywhere").click();
    harness.run_ok();
    pause(&mut harness, 0.2);

    for label in ["…", "—"] {
        if let Some(node) = harness.query_by_label(label) {
            node.click();
            harness.run_ok();
            pause(&mut harness, 0.2);
        }
    }

    close_demo(&mut harness, "Text Layout");

    // ── Undo Redo ────────────────────────────────────────────────
    open_demo(&mut harness, "Undo Redo");

    if let Some(cb) = harness.query_by_label("Checkbox with undo/redo") {
        cb.click();
        harness.run_ok();
        pause(&mut harness, 0.2);
    }

    let has_te = harness.query_all(by().role(Role::TextInput)).next().is_some();
    if has_te {
        harness
            .get_all(by().role(Role::TextInput))
            .next()
            .unwrap()
            .focus();
        harness.key_press_modifiers(Modifiers::COMMAND, Key::A);
        harness
            .get_all(by().role(Role::TextInput))
            .next()
            .unwrap()
            .type_text("undo me");
        harness.run_ok();
        pause(&mut harness, 0.3);
    }

    if let Some(undo) = harness.query_by_role_and_label(Role::Button, "⟲ Undo") {
        undo.click();
        harness.run_ok();
        pause(&mut harness, 0.3);
    }

    if let Some(redo) = harness.query_by_role_and_label(Role::Button, "⟳ Redo") {
        redo.click();
        harness.run_ok();
        pause(&mut harness, 0.3);
    }

    close_demo(&mut harness, "Undo Redo");

    // ── Highlighting ─────────────────────────────────────────────
    open_demo(&mut harness, "Highlighting");

    if let Some(label) = harness.query_by_label_contains("Hover me") {
        label.hover();
        harness.run_ok();
        pause(&mut harness, 0.3);
    }
    if harness.query_by_label_contains("Hover the button").is_some() {
        harness
            .get_by_label_contains("Hover the button")
            .hover();
        harness.run_ok();
        pause(&mut harness, 0.3);

        harness
            .get_by_label_contains("Hover the button")
            .click();
        harness.run_ok();
        pause(&mut harness, 0.2);
    }

    close_demo(&mut harness, "Highlighting");

    // ── Dancing Strings ──────────────────────────────────────────
    open_demo(&mut harness, "Dancing Strings");

    harness.get_by_label("Colored").click();
    harness.run_ok();
    pause(&mut harness, 0.3);

    harness.get_by_label("Colored").click();
    harness.run_ok();
    pause(&mut harness, 0.2);

    close_demo(&mut harness, "Dancing Strings");

    // ── Painting ─────────────────────────────────────────────────
    open_demo(&mut harness, "Painting");

    harness.get_by_label("Clear Painting").click();
    harness.run_ok();
    pause(&mut harness, 0.3);

    close_demo(&mut harness, "Painting");

    // ── Bézier Curve ─────────────────────────────────────────────
    open_demo(&mut harness, "zier Curve");

    if let Some(cubic) = harness.query_by_label("Cubic") {
        cubic.click();
        harness.run_ok();
        pause(&mut harness, 0.3);
    }
    if let Some(quad) = harness.query_by_label("Quadratic") {
        quad.click();
        harness.run_ok();
        pause(&mut harness, 0.3);
    }

    close_demo(&mut harness, "zier Curve");

    // ── Drag and Drop ────────────────────────────────────────────
    open_demo(&mut harness, "Drag and Drop");

    if let Some(item) = harness.query_by_label("Item A") {
        let from = item.rect().center();
        harness.drag_at(from);
        harness.step();
        harness.drop_at(from + Vec2::new(200.0, 0.0));
        harness.run_ok();
        pause(&mut harness, 0.3);
    }

    close_demo(&mut harness, "Drag and Drop");

    // ── Scene ────────────────────────────────────────────────────
    open_demo(&mut harness, "Scene");

    if let Some(btn) = harness.query_by_label("Reset view") {
        btn.click();
        harness.run_ok();
        pause(&mut harness, 0.3);
    }

    close_demo(&mut harness, "Scene");

    // ── Misc Demos ───────────────────────────────────────────────
    open_demo(&mut harness, "Misc Demos");

    for section in ["Checkboxes", "Colors", "Text layout", "Misc"] {
        if let Some(header) = harness.query_by_label(section) {
            header.click_accesskit();
            harness.run_ok();
            pause(&mut harness, 0.3);
        }
    }

    close_demo(&mut harness, "Misc Demos");

    // ── Remaining demos (just open and view) ─────────────────────
    for name in [
        "Popups",
        "Screenshot",
        "Panels",
        "Strip",
        "Frame",
        "TextEdit",
    ] {
        open_demo(&mut harness, name);
        pause(&mut harness, 0.3);
        close_demo(&mut harness, name);
    }
}

// ─── Standalone tests (text edit, selection, rendering) ──────────────

fn test_text_edit() {
    let text = "Hello, world!".to_owned();
    let mut harness = Harness::new_ui_state(
        move |ui, text| {
            CentralPanel::default().show_inside(ui, |ui| {
                ui.text_edit_singleline(text);
            });
        },
        text,
    );
    harness.run();

    let text_edit = harness.get_by_role(Role::TextInput);
    assert_eq!(text_edit.value().as_deref(), Some("Hello, world!"));
    text_edit.focus();
    harness.key_press_modifiers(Modifiers::COMMAND, Key::A);
    text_edit.type_text("Hi ");
    harness.run();
    pause(&mut harness, 0.3);

    harness.get_by_role(Role::TextInput).type_text("there!");
    harness.run();
    let text_edit = harness.get_by_role(Role::TextInput);
    assert_eq!(text_edit.value().as_deref(), Some("Hi there!"));
    assert_eq!(harness.state(), "Hi there!");
    pause(&mut harness, 0.3);
}

fn test_text_selection() {
    for drag_start_x in [0.2_f32, 0.9] {
        let mut harness = Harness::builder().build_ui(|ui| {
            let visuals = ui.visuals_mut();
            visuals.selection.bg_fill = Color32::LIGHT_GREEN;
            visuals.selection.stroke.color = Color32::RED;
            ui.label("Some varied ☺ text :)\nAnd it has a second line!");
        });
        harness.run();
        harness.fit_contents();

        let label = harness.get_by_role(Role::Label);
        harness.drag_at(label.rect().lerp_inside([drag_start_x, 0.25]));
        harness.drop_at(label.rect().lerp_inside([0.6, 0.75]));
        harness.run();
        pause(&mut harness, 0.3);
    }
}

fn test_rendering_color_test() {
    let mut color_test = ColorTest::default();
    let mut harness = Harness::builder().build_ui(|ui| {
        color_test.ui(ui);
    });

    harness.get_by_label("Color test").click_accesskit();
    harness.run();
    harness.fit_contents();
    pause(&mut harness, 0.5);
}

fn test_widget_gallery_standalone() {
    let mut demo = WidgetGallery::default();
    let mut harness = Harness::builder()
        .with_size(Vec2::new(380.0, 550.0))
        .build_ui(|ui| {
            egui_extras::install_image_loaders(ui.ctx());
            demo.ui(ui);
        });
    harness.fit_contents();
    harness.run_ok();
    pause(&mut harness, 0.5);
}

fn test_image_blending() {
    let mut harness = Harness::builder().build_ui(|ui| {
        egui_extras::install_image_loaders(ui.ctx());
        egui::Frame::new()
            .fill(egui::Color32::from_rgb(0x59, 0x81, 0xFF))
            .show(ui, |ui| {
                ui.add(
                    egui::Image::new(egui::include_image!(
                        "../../egui_demo_lib/data/ring.png"
                    ))
                    .max_height(18.0)
                    .tint(Color32::GRAY),
                );
            });
    });
    harness.run();
    harness.fit_contents();
    pause(&mut harness, 0.3);
}

fn test_kerning() {
    let mut harness = Harness::builder().build_ui(|ui| {
        ui.label("Hello world!");
        ui.label("Repeated: iiiiiiiiiiiii lllllllll mmmmmmmmmmmmmmmm");
        ui.label("Thin spaces: −123 456 789");
        ui.label("Ligature: fi :)");
        ui.label("\ttabbed");
    });
    harness.run();
    harness.fit_contents();
    pause(&mut harness, 0.3);
}

// ─── Demo App: navigate through every eframe tab ─────────────────────

fn test_demo_app_tabs() {
    let mut harness = Harness::builder()
        .with_size(Vec2::new(900.0, 600.0))
        .wgpu()
        .build_eframe(|cc| WrapApp::new(cc));

    let app = harness.state_mut();
    app.state.clock.mock_time = Some(36383.0);

    let apps = app
        .apps_iter_mut()
        .map(|(name, anchor, _)| (name, anchor))
        .collect::<Vec<_>>();

    for (name, anchor) in &apps {
        harness.get_by_role_and_label(Role::Button, name).click();
        match anchor {
            Anchor::Demo => {}
            Anchor::Rendering => {}
            #[cfg(feature = "http")]
            Anchor::Http => continue,
            #[cfg(feature = "image_viewer")]
            Anchor::ImageViewer => {
                harness.step();
                harness
                    .get_by_role_and_label(Role::TextInput, "URI:")
                    .focus();
                harness.key_press_modifiers(Modifiers::COMMAND, Key::A);
                harness
                    .get_by_role_and_label(Role::TextInput, "URI:")
                    .type_text("file://../eframe/data/icon.png");
                harness.get_by_role_and_label(Role::Button, "✔").click();
                harness.try_run_realtime().ok();
            }
            _ => {}
        }
        harness.run_steps(4);
        pause(&mut harness, 0.5);
    }
}

// ─── Main ────────────────────────────────────────────────────────────

fn main() {
    let tests: Vec<(&str, fn())> = vec![
        // This is the big one — single shared harness with a window,
        // opens each demo and interacts with its elements.
        ("all_demos", test_all_demos),
        // Standalone tests
        ("text_edit", test_text_edit),
        ("text_selection", test_text_selection),
        ("rendering_color_test", test_rendering_color_test),
        ("widget_gallery_standalone", test_widget_gallery_standalone),
        ("image_blending", test_image_blending),
        ("kerning", test_kerning),
        // eframe demo app tabs
        ("demo_app_tabs", test_demo_app_tabs),
    ];

    let mut passed = 0;
    let mut failed = Vec::new();

    for (name, test_fn) in &tests {
        eprint!("  {name} ... ");
        match std::panic::catch_unwind(std::panic::AssertUnwindSafe(test_fn)) {
            Ok(()) => {
                eprintln!("ok");
                passed += 1;
            }
            Err(_) => {
                eprintln!("FAILED");
                failed.push(*name);
            }
        }
    }

    eprintln!("\n{passed} passed, {} failed.", failed.len());
    if !failed.is_empty() {
        eprintln!("Failures:");
        for name in &failed {
            eprintln!("  - {name}");
        }
        std::process::exit(1);
    }
}
