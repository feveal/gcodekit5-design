//! Speeds and Feeds Calculator Tool

use gtk4::prelude::*;
use gtk4::{Align, Box, Button, ComboBoxText, Label, Orientation, Paned, ScrolledWindow, Stack};
use libadwaita::prelude::*;
use libadwaita::{ActionRow, PreferencesGroup};
use std::rc::Rc;

use super::common::set_paned_initial_fraction;
use crate::ui::gtk::help_browser;
use gcodekit5_settings::SettingsController;

pub struct SpeedsFeedsTool {
    content: Box,
}

impl SpeedsFeedsTool {
    pub fn new(stack: &Stack, _settings: Rc<SettingsController>) -> Self {
        let content_box = Box::new(Orientation::Vertical, 0);

        // Header
        let header = Box::new(Orientation::Horizontal, 12);
        header.set_margin_top(12);
        header.set_margin_bottom(12);
        header.set_margin_start(12);
        header.set_margin_end(12);

        let back_btn = Button::builder().icon_name("go-previous-symbolic").build();
        let stack_clone = stack.clone();
        back_btn.connect_clicked(move |_| {
            stack_clone.set_visible_child_name("dashboard");
        });
        header.append(&back_btn);

        let title = Label::builder()
            .label("Speeds and Feeds Calculator")
            .css_classes(vec!["title-2"])
            .build();
        title.set_hexpand(true);
        title.set_halign(Align::Start);
        header.append(&title);
        header.append(&help_browser::make_help_button("speeds_feeds_calculator"));
        content_box.append(&header);

        // Paned Layout
        let paned = Paned::new(Orientation::Horizontal);
        paned.set_hexpand(true);
        paned.set_vexpand(true);

        // Sidebar (40%)
        let sidebar = Box::new(Orientation::Vertical, 12);
        sidebar.add_css_class("sidebar");
        sidebar.set_margin_top(24);
        sidebar.set_margin_bottom(24);
        sidebar.set_margin_start(24);
        sidebar.set_margin_end(24);

        let title_label = Label::builder()
            .label("Speeds and Feeds")
            .css_classes(vec!["title-3"])
            .halign(Align::Start)
            .build();
        sidebar.append(&title_label);

        let desc = Label::builder()
            .label("Calculate optimal cutting speeds and feed rates based on material properties and tool specifications. Uses standard machining formulas.")
            .css_classes(vec!["body"])
            .wrap(true)
            .halign(Align::Start)
            .build();
        sidebar.append(&desc);

        // Results display area
        let results_box = Box::new(Orientation::Vertical, 6);
        results_box.set_vexpand(true);

        let results_frame = gtk4::Frame::new(Some("Calculated Results"));
        results_frame.set_margin_top(12);

        let results_content = Box::new(Orientation::Vertical, 6);
        results_content.set_margin_top(12);
        results_content.set_margin_bottom(12);
        results_content.set_margin_start(12);
        results_content.set_margin_end(12);

        let rpm_label = Label::builder()
            .label("RPM: --")
            .halign(Align::Start)
            .build();
        let feed_label = Label::builder()
            .label("Feed Rate: --")
            .halign(Align::Start)
            .build();
        let source_label = Label::builder()
            .label("")
            .css_classes(vec!["caption", "dim-label"])
            .halign(Align::Start)
            .wrap(true)
            .build();
        let warnings_label = Label::builder()
            .label("")
            .css_classes(vec!["caption", "warning"])
            .halign(Align::Start)
            .wrap(true)
            .build();

        results_content.append(&rpm_label);
        results_content.append(&feed_label);
        results_content.append(&source_label);
        results_content.append(&warnings_label);
        results_frame.set_child(Some(&results_content));
        results_box.append(&results_frame);
        sidebar.append(&results_box);

        // Content Area
        let right_panel = Box::new(Orientation::Vertical, 0);
        let scroll_content = Box::new(Orientation::Vertical, 0);
        let scrolled = ScrolledWindow::builder()
            .hscrollbar_policy(gtk4::PolicyType::Never)
            .vexpand(true)
            .child(&scroll_content)
            .build();

        // Material Selection
        let material_group = PreferencesGroup::builder().title("Material").build();
        let material_combo = ComboBoxText::new();
        material_combo.append(Some("aluminum"), "Aluminum");
        material_combo.append(Some("wood"), "Wood (Softwood)");
        material_combo.append(Some("acrylic"), "Acrylic");
        material_combo.append(Some("steel"), "Steel (Mild)");
        material_combo.set_active_id(Some("aluminum"));
        let material_row = ActionRow::builder().title("Material Type:").build();
        material_row.add_suffix(&material_combo);
        material_group.add(&material_row);
        scroll_content.append(&material_group);

        // Tool Selection
        let tool_group = PreferencesGroup::builder().title("Tool").build();
        let tool_combo = ComboBoxText::new();
        tool_combo.append(Some("endmill_6mm"), "End Mill - 6mm");
        tool_combo.append(Some("endmill_3mm"), "End Mill - 3mm");
        tool_combo.append(Some("vbit_30deg"), "V-Bit - 30°");
        tool_combo.set_active_id(Some("endmill_6mm"));
        let tool_row = ActionRow::builder().title("Tool Type:").build();
        tool_row.add_suffix(&tool_combo);
        tool_group.add(&tool_row);
        scroll_content.append(&tool_group);

        right_panel.append(&scrolled);

        // Action Buttons
        let action_box = Box::new(Orientation::Horizontal, 12);
        action_box.set_margin_top(12);
        action_box.set_margin_bottom(12);
        action_box.set_margin_end(12);
        action_box.set_halign(Align::End);

        let calculate_btn = Button::with_label("Calculate");
        calculate_btn.add_css_class("suggested-action");
        action_box.append(&calculate_btn);
        right_panel.append(&action_box);

        paned.set_start_child(Some(&sidebar));
        paned.set_end_child(Some(&right_panel));
        // Initial ratio only; do not fight user resizing.
        set_paned_initial_fraction(&paned, 0.40);

        content_box.append(&paned);

        // Calculate button handler - simplified placeholder
        let rpm_label_calc = rpm_label.clone();
        let feed_label_calc = feed_label.clone();
        let source_label_calc = source_label.clone();
        let warnings_label_calc = warnings_label.clone();

        calculate_btn.connect_clicked(move |_| {
            // Placeholder calculation
            rpm_label_calc.set_text("RPM: 12,000");
            feed_label_calc.set_text("Feed Rate: 1,500 mm/min");
            source_label_calc.set_text("Source: Material defaults + Tool specifications");
            warnings_label_calc.set_text("");
        });

        Self {
            content: content_box,
        }
    }

    pub fn widget(&self) -> &Box {
        &self.content
    }
}

// Spoilboard Surfacing Tool
