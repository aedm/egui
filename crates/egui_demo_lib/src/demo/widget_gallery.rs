#[derive(Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
enum Enum {
    First,
    Second,
    Third,
}

/// Shows off one example of each major type of widget.
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct WidgetGallery {
    enabled: bool,
    visible: bool,
    boolean: bool,
    opacity: f32,
    radio: Enum,
    scalar: f32,
    string: String,
    color: egui::Color32,
    animate_progress_bar: bool,

    #[cfg(feature = "chrono")]
    #[cfg_attr(feature = "serde", serde(skip))]
    date: Option<chrono::NaiveDate>,

    #[cfg(feature = "chrono")]
    with_date_button: bool,
}

impl Default for WidgetGallery {
    fn default() -> Self {
        Self {
            enabled: true,
            visible: true,
            opacity: 1.0,
            boolean: false,
            radio: Enum::First,
            scalar: 42.0,
            string: Default::default(),
            color: egui::Color32::LIGHT_BLUE.linear_multiply(0.5),
            animate_progress_bar: false,
            #[cfg(feature = "chrono")]
            date: None,
            #[cfg(feature = "chrono")]
            with_date_button: true,
        }
    }
}

impl WidgetGallery {
    #[allow(clippy::allow_attributes, unused_mut)] // if not chrono
    #[inline]
    pub fn with_date_button(mut self, _with_date_button: bool) -> Self {
        #[cfg(feature = "chrono")]
        {
            self.with_date_button = _with_date_button;
        }
        self
    }
}

impl crate::Demo for WidgetGallery {
    fn name(&self) -> &'static str {
        "🗄 Widget Gallery"
    }

    fn show(&mut self, ui: &mut egui::Ui, open: &mut bool) {
        egui::Window::new(self.name())
            .open(open)
            .resizable([true, false]) // resizable so we can shrink if the text edit grows
            .default_width(280.0)
            .constrain_to(ui.available_rect_before_wrap())
            .show(ui, |ui| {
                use crate::View as _;
                self.ui(ui);
            });
    }
}

impl crate::View for WidgetGallery {
    fn ui(&mut self, ui: &mut egui::Ui) {
        let mut ui_builder = egui::UiBuilder::new();
        if !self.enabled {
            ui_builder = ui_builder.disabled();
        }
        if !self.visible {
            ui_builder = ui_builder.invisible();
        }

        ui.scope_builder(ui_builder, |ui| {
            ui.multiply_opacity(self.opacity);

            egui::Grid::new("my_grid")
                .num_columns(2)
                .spacing([40.0, 4.0])
                .striped(true)
                .show(ui, |ui| {
                    self.gallery_grid_contents(ui);
                });
        });

        ui.separator();

        ui.horizontal(|ui| {
            ui.checkbox(&mut self.visible, "Visible")
                .on_hover_text("Uncheck to hide all the widgets.");
            if self.visible {
                ui.checkbox(&mut self.enabled, "Interactive")
                    .on_hover_text("Uncheck to inspect how the widgets look when disabled.");
                (ui.add(
                    egui::DragValue::new(&mut self.opacity)
                        .speed(0.01)
                        .range(0.0..=1.0),
                ) | ui.label("Opacity"))
                .on_hover_text("Reduce this value to make widgets semi-transparent");
            }
        });

        ui.separator();

        ui.vertical_centered(|ui| {
            let tooltip_text = "The full egui documentation.\nYou can also click the different widgets names in the left column.";
            ui.hyperlink("https://docs.rs/egui/").on_hover_text(tooltip_text);
            ui.add(crate::egui_github_link_file!(
                "Source code of the widget gallery"
            ));
        });
    }
}

impl WidgetGallery {
    fn gallery_grid_contents(&mut self, ui: &mut egui::Ui) {
        let Self {
            enabled: _,
            visible: _,
            opacity: _,
            boolean,
            radio,
            scalar,
            string,
            color,
            animate_progress_bar,
            #[cfg(feature = "chrono")]
            date,
            #[cfg(feature = "chrono")]
            with_date_button,
        } = self;

        ui.add(doc_link_label("Label", "label"));
        ui.label("Welcome to the widget gallery!");
        ui.end_row();

        ui.add(doc_link_label("Hyperlink", "Hyperlink"));
        use egui::special_emojis::GITHUB;
        ui.hyperlink_to(
            format!("{GITHUB} egui on GitHub"),
            "https://github.com/emilk/egui",
        );
        ui.end_row();

        ui.add(doc_link_label("TextEdit", "TextEdit"));
        ui.add(egui::TextEdit::singleline(string).hint_text("Write something here"));
        ui.end_row();

        ui.add(doc_link_label("Button", "button"));
        if ui.button("Click me!").clicked() {
            *boolean = !*boolean;
        }
        ui.end_row();

        ui.add(doc_link_label("Link", "link"));
        if ui.link("Click me!").clicked() {
            *boolean = !*boolean;
        }
        ui.end_row();

        ui.add(doc_link_label("Checkbox", "checkbox"));
        ui.checkbox(boolean, "Checkbox");
        ui.end_row();

        ui.add(doc_link_label("RadioButton", "radio"));
        ui.horizontal(|ui| {
            ui.radio_value(radio, Enum::First, "First");
            ui.radio_value(radio, Enum::Second, "Second");
            ui.radio_value(radio, Enum::Third, "Third");
        });
        ui.end_row();

        ui.add(doc_link_label("SelectableLabel", "SelectableLabel"));
        ui.horizontal(|ui| {
            ui.selectable_value(radio, Enum::First, "First");
            ui.selectable_value(radio, Enum::Second, "Second");
            ui.selectable_value(radio, Enum::Third, "Third");
        });
        ui.end_row();

        ui.add(doc_link_label("ComboBox", "ComboBox"));

        egui::ComboBox::from_label("Take your pick")
            .selected_text(format!("{radio:?}"))
            .show_ui(ui, |ui| {
                ui.selectable_value(radio, Enum::First, "First");
                ui.selectable_value(radio, Enum::Second, "Second");
                ui.selectable_value(radio, Enum::Third, "Third");
            });
        ui.end_row();

        ui.add(doc_link_label("Slider", "Slider"));
        ui.add(egui::Slider::new(scalar, 0.0..=360.0).suffix("°"));
        ui.end_row();

        ui.add(doc_link_label("DragValue", "DragValue"));
        ui.add(egui::DragValue::new(scalar).speed(1.0));
        ui.end_row();

        ui.add(doc_link_label("ProgressBar", "ProgressBar"));
        let progress = *scalar / 360.0;
        let progress_bar = egui::ProgressBar::new(progress)
            .show_percentage()
            .animate(*animate_progress_bar);
        *animate_progress_bar = ui
            .add(progress_bar)
            .on_hover_text("The progress bar can be animated!")
            .hovered();
        ui.end_row();

        ui.add(doc_link_label("Color picker", "color_edit"));
        ui.color_edit_button_srgba(color);
        ui.end_row();

        ui.add(doc_link_label("Image", "Image"));
        let egui_icon = egui::include_image!("../../data/icon.png");
        ui.add(egui::Image::new(egui_icon.clone()));
        ui.end_row();

        ui.add(doc_link_label(
            "Button with image",
            "Button::image_and_text",
        ));
        if ui
            .add(egui::Button::image_and_text(egui_icon, "With image!"))
            .clicked()
        {
            *boolean = !*boolean;
        }
        ui.end_row();

        #[cfg(feature = "chrono")]
        if *with_date_button {
            let date = date.get_or_insert_with(|| chrono::offset::Utc::now().date_naive());
            ui.add(doc_link_label_with_crate(
                "egui_extras",
                "DatePickerButton",
                "DatePickerButton",
            ));
            ui.add(egui_extras::DatePickerButton::new(date));
            ui.end_row();
        }

        ui.add(doc_link_label("Separator", "separator"));
        ui.separator();
        ui.end_row();

        ui.add(doc_link_label("CollapsingHeader", "collapsing"));
        ui.collapsing("Click to see what is hidden!", |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing.x = 0.0;
                ui.label("It's a ");
                ui.add(doc_link_label("Spinner", "spinner"));
                ui.add_space(4.0);
                ui.add(egui::Spinner::new());
            });
        });
        ui.end_row();

        ui.hyperlink_to(
            "Custom widget",
            super::toggle_switch::url_to_file_source_code(),
        );
        ui.add(super::toggle_switch::toggle(boolean)).on_hover_text(
            "It's easy to create your own widgets!\n\
            This toggle switch is just 15 lines of code.",
        );
        ui.end_row();
    }
}

fn doc_link_label<'a>(title: &'a str, search_term: &'a str) -> impl egui::Widget + 'a {
    doc_link_label_with_crate("egui", title, search_term)
}

fn doc_link_label_with_crate<'a>(
    crate_name: &'a str,
    title: &'a str,
    search_term: &'a str,
) -> impl egui::Widget + 'a {
    let url = format!("https://docs.rs/{crate_name}?search={search_term}");
    move |ui: &mut egui::Ui| {
        ui.hyperlink_to(title, url).on_hover_ui(|ui| {
            ui.horizontal_wrapped(|ui| {
                ui.label("Search egui docs for");
                ui.code(search_term);
            });
        })
    }
}

#[cfg(feature = "chrono")]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::View as _;
    use egui::Vec2;
    use egui::accesskit;
    use egui_kittest::kittest::{NodeT as _, Queryable as _};
    use egui_kittest::{Harness, SnapshotResults};

    fn make_gallery() -> WidgetGallery {
        WidgetGallery {
            date: Some(chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()),
            ..Default::default()
        }
    }

    fn make_harness() -> Harness<'static, WidgetGallery> {
        Harness::builder()
            .with_size(Vec2::new(380.0, 550.0))
            .build_ui_state(
                |ui, demo: &mut WidgetGallery| {
                    egui_extras::install_image_loaders(ui.ctx());
                    demo.ui(ui);
                },
                make_gallery(),
            )
    }

    #[test]
    pub fn should_match_screenshot() {
        let mut results = SnapshotResults::new();

        for pixels_per_point in [1, 2] {
            for theme in [egui::Theme::Light, egui::Theme::Dark] {
                let mut harness = Harness::builder()
                    .with_pixels_per_point(pixels_per_point as f32)
                    .with_theme(theme)
                    .with_size(Vec2::new(380.0, 550.0))
                    .build_ui_state(
                        |ui, demo: &mut WidgetGallery| {
                            egui_extras::install_image_loaders(ui.ctx());
                            demo.ui(ui);
                        },
                        make_gallery(),
                    );

                harness.fit_contents();

                let theme_name = match theme {
                    egui::Theme::Light => "light",
                    egui::Theme::Dark => "dark",
                };
                let image_name = format!("widget_gallery_{theme_name}_x{pixels_per_point}");
                harness.snapshot(&image_name);
                results.extend_harness(&mut harness);
            }
        }
    }

    #[test]
    fn text_edit() {
        let mut harness = make_harness();
        assert_eq!(harness.state().string, "");

        // Focus the text input first, then type
        harness.get_by_role(accesskit::Role::TextInput).click();
        harness.run();

        harness.get_by_role(accesskit::Role::TextInput).type_text("Hello");
        harness.run();

        assert_eq!(harness.state().string, "Hello");
    }

    #[test]
    fn button_click() {
        let mut harness = make_harness();
        assert!(!harness.state().boolean);

        // "Click me!" with Role::Button — the first match is the regular Button
        harness
            .get_all_by_role_and_label(accesskit::Role::Button, "Click me!")
            .next()
            .unwrap()
            .click();
        harness.run();

        assert!(harness.state().boolean);
    }

    #[test]
    fn link_click() {
        let mut harness = make_harness();
        assert!(!harness.state().boolean);

        // The "Click me!" Link has Role::Link, so it's unambiguous
        harness
            .get_by_role_and_label(accesskit::Role::Link, "Click me!")
            .click();
        harness.run();

        assert!(harness.state().boolean);
    }

    #[test]
    fn checkbox_toggle() {
        let mut harness = make_harness();
        assert!(!harness.state().boolean);

        harness
            .get_by_role_and_label(accesskit::Role::CheckBox, "Checkbox")
            .click();
        harness.run();

        assert!(harness.state().boolean);

        // Toggle back
        harness
            .get_by_role_and_label(accesskit::Role::CheckBox, "Checkbox")
            .click();
        harness.run();

        assert!(!harness.state().boolean);
    }

    #[test]
    fn radio_button() {
        let mut harness = make_harness();
        assert_eq!(harness.state().radio, Enum::First);

        harness
            .get_by_role_and_label(accesskit::Role::RadioButton, "Second")
            .click();
        harness.run();

        assert_eq!(harness.state().radio, Enum::Second);

        harness
            .get_by_role_and_label(accesskit::Role::RadioButton, "Third")
            .click();
        harness.run();

        assert_eq!(harness.state().radio, Enum::Third);
    }

    #[test]
    fn selectable_label() {
        let mut harness = make_harness();
        assert_eq!(harness.state().radio, Enum::First);

        // SelectableLabel is rendered as Role::Button; "Second" button
        // that is NOT a RadioButton. There are two Role::Button "Second":
        // the selectable label and the ComboBox option (hidden).
        // The selectable label is visible so get_by_role_and_label works.
        harness
            .get_by_role_and_label(accesskit::Role::Button, "Second")
            .click();
        harness.run();

        assert_eq!(harness.state().radio, Enum::Second);
    }

    #[test]
    fn combo_box() {
        let mut harness = make_harness();
        assert_eq!(harness.state().radio, Enum::First);

        // Open the ComboBox by clicking it
        harness
            .get_by_role(accesskit::Role::ComboBox)
            .click();
        harness.run();

        // Now the popup is open, click "Third" option inside it.
        // The popup creates new Role::Button items. But "Third" also exists
        // as a selectable label. Use get_all and pick the last one (popup item).
        let thirds: Vec<_> = harness
            .get_all_by_role_and_label(accesskit::Role::Button, "Third")
            .collect();
        thirds.last().unwrap().click();
        harness.run();

        assert_eq!(harness.state().radio, Enum::Third);
    }

    #[test]
    fn slider() {
        let mut harness = make_harness();
        let initial = harness.state().scalar;
        assert!((initial - 42.0).abs() < f32::EPSILON);

        // Click on the right side of the slider to change the value
        let slider = harness.get_by_role(accesskit::Role::Slider);
        let rect = slider.rect();
        // Click at 75% of the slider width to set a value around 270°
        let click_pos = egui::pos2(rect.left() + rect.width() * 0.75, rect.center().y);
        harness.event(egui::Event::PointerMoved(click_pos));
        harness.event(egui::Event::PointerButton {
            pos: click_pos,
            button: egui::PointerButton::Primary,
            pressed: true,
            modifiers: egui::Modifiers::NONE,
        });
        harness.event(egui::Event::PointerButton {
            pos: click_pos,
            button: egui::PointerButton::Primary,
            pressed: false,
            modifiers: egui::Modifiers::NONE,
        });
        harness.run();

        // The value should have changed significantly from 42
        assert!(
            (harness.state().scalar - initial).abs() > 10.0,
            "Slider value should have changed, got {}",
            harness.state().scalar
        );
    }

    #[test]
    fn drag_value() {
        let mut harness = make_harness();
        assert!((harness.state().scalar - 42.0).abs() < f32::EPSILON);

        // There are multiple SpinButtons (scalar and opacity). Get the first
        // one (the scalar DragValue in the grid).
        harness
            .get_all_by_role(accesskit::Role::SpinButton)
            .next()
            .unwrap()
            .click();
        harness.run();

        // Select all existing text and replace it
        harness.key_press_modifiers(egui::Modifiers::COMMAND, egui::Key::A);
        harness.event(egui::Event::Text("100".to_owned()));
        harness.key_press(egui::Key::Enter);
        harness.run();

        assert!(
            (harness.state().scalar - 100.0).abs() < f32::EPSILON,
            "DragValue should be 100, got {}",
            harness.state().scalar
        );
    }

    #[test]
    fn collapsing_header() {
        let mut harness = make_harness();

        // Count ProgressIndicator nodes before expanding (ProgressBar is one)
        let before_count = harness
            .query_all_by_role(accesskit::Role::ProgressIndicator)
            .count();

        // Click the collapsing header to expand it, revealing the Spinner
        harness
            .get_by_label("Click to see what is hidden!")
            .click();
        // Use run_steps instead of run because the Spinner causes continuous repaints
        harness.run_steps(2);

        // After expanding, there should be one more ProgressIndicator (the Spinner)
        let after_count = harness
            .query_all_by_role(accesskit::Role::ProgressIndicator)
            .count();
        assert!(
            after_count > before_count,
            "Expanding the collapsing header should reveal the Spinner \
             (before: {before_count}, after: {after_count})"
        );
    }

    #[test]
    fn toggle_switch() {
        let mut harness = make_harness();
        assert!(!harness.state().boolean);

        // The toggle switch is a custom widget reporting as Checkbox with empty label.
        // Find it by looking for a Checkbox that is NOT "Checkbox" (which is the
        // standard checkbox) and not "Visible"/"Interactive" (bottom checkboxes).
        let toggles: Vec<_> = harness
            .get_all_by_role(accesskit::Role::CheckBox)
            .filter(|n| {
                let label = n.accesskit_node().label().unwrap_or_default();
                label.is_empty()
            })
            .collect();
        assert!(!toggles.is_empty(), "Should find the custom toggle switch");
        toggles[0].click();
        harness.run();

        assert!(harness.state().boolean);
    }

    #[test]
    fn image_button_click() {
        let mut harness = make_harness();
        assert!(!harness.state().boolean);

        harness.get_by_label("With image!").click();
        harness.run();

        assert!(harness.state().boolean);
    }

    #[test]
    fn visible_checkbox() {
        let mut harness = make_harness();
        assert!(harness.state().visible);

        harness.get_by_label("Visible").click();
        harness.run();

        assert!(!harness.state().visible);
    }

    #[test]
    fn interactive_checkbox() {
        let mut harness = make_harness();
        assert!(harness.state().enabled);

        harness.get_by_label("Interactive").click();
        harness.run();

        assert!(!harness.state().enabled);
    }
}
