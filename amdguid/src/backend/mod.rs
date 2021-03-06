#[cfg(feature = "xorg-glium")]
pub mod glium_backend;
#[cfg(feature = "xorg-glow")]
pub mod glow_backend;
#[cfg(feature = "wayland")]
pub mod wayland_backend;

use egui::panel::TopBottomSide;
use egui::{Layout, PointerButton};
use epaint::TextStyle;
use parking_lot::Mutex;
use std::sync::Arc;
#[cfg(feature = "wayland")]
pub use wayland_backend::run_app;

#[cfg(feature = "xorg-glow")]
pub use glow_backend::run_app;

use crate::app::Page;
use crate::AmdGui;
#[cfg(feature = "xorg-glium")]
pub use glium_backend::run_app;

pub fn create_ui(amd_gui: Arc<Mutex<AmdGui>>, ctx: &egui::CtxRef) {
    egui::containers::TopBottomPanel::new(TopBottomSide::Top, "menu").show(ctx, |ui| {
        let mut child = ui.child_ui(ui.available_rect_before_wrap(), Layout::left_to_right());
        if child
            .add(egui::Button::new("Config").text_style(TextStyle::Heading))
            .clicked_by(PointerButton::Primary)
        {
            amd_gui.lock().page = Page::Config;
        }
        if child
            .add(egui::Button::new("Monitoring").text_style(TextStyle::Heading))
            .clicked_by(PointerButton::Primary)
        {
            amd_gui.lock().page = Page::Monitoring;
        }
        if child
            .add(egui::Button::new("Settings").text_style(TextStyle::Heading))
            .clicked_by(PointerButton::Primary)
        {
            amd_gui.lock().page = Page::Settings;
        }
    });

    egui::containers::CentralPanel::default().show(ctx, |ui| {
        let mut gui = amd_gui.lock();
        let page = gui.page;
        match page {
            Page::Config => {
                gui.ui(ui);
            }
            Page::Monitoring => {
                gui.ui(ui);
            }
            Page::Settings => {
                ctx.settings_ui(ui);
            }
        }
    });
}
