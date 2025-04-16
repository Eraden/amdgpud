use std::sync::Arc;

use egui::panel::TopBottomSide;
use egui::{Align, Layout, PointerButton};
use parking_lot::Mutex;

use crate::app::Page;
use crate::AmdGui;

pub fn create_ui(amd_gui: Arc<Mutex<AmdGui>>, ctx: &egui::Context) {
    egui::containers::TopBottomPanel::new(TopBottomSide::Top, "menu").show(ctx, |ui| {
        let mut child = ui.child_ui(
            ui.available_rect_before_wrap(),
            Layout::left_to_right(Align::default()),
            None,
        );

        if child
            .add(
                egui::Button::new("Temp Config"), /* .text_style(TextStyle::Heading) */
            )
            .clicked_by(PointerButton::Primary)
        {
            amd_gui.lock().page = Page::TempConfig;
        }
        if child
            .add(
                egui::Button::new("Usage Config"), /* .text_style(TextStyle::Heading) */
            )
            .clicked_by(PointerButton::Primary)
        {
            amd_gui.lock().page = Page::UsageConfig;
        }
        if child
            .add(
                egui::Button::new("Monitoring"), /* .text_style(TextStyle::Heading) */
            )
            .clicked_by(PointerButton::Primary)
        {
            amd_gui.lock().page = Page::Monitoring;
        }
        if child
            .add(
                egui::Button::new("Outputs"), /* .text_style(TextStyle::Heading) */
            )
            .clicked_by(PointerButton::Primary)
        {
            amd_gui.lock().page = Page::Outputs;
        }
        if child
            .add(
                egui::Button::new("Settings"), /* .text_style(TextStyle::Heading) */
            )
            .clicked_by(PointerButton::Primary)
        {
            amd_gui.lock().page = Page::Settings;
        }
    });

    egui::containers::CentralPanel::default().show(ctx, |ui| {
        let mut gui = amd_gui.lock();
        match gui.page {
            Page::TempConfig => {
                gui.ui(ui);
            }
            Page::UsageConfig => {
                gui.ui(ui);
            }
            Page::Monitoring => {
                gui.ui(ui);
            }
            Page::Outputs => {
                gui.ui(ui);
            }
            Page::Settings => {
                ctx.settings_ui(ui);
            }
        }
    });
}
