#[cfg(feature = "xorg-glium")]
pub mod glium_backend;
#[cfg(feature = "xorg-glow")]
pub mod glow_backend;
#[cfg(feature = "wayland")]
pub mod wayland_backend;

use std::sync::Arc;

use egui::panel::TopBottomSide;
use egui::{Layout, PointerButton};
#[cfg(feature = "xorg-glium")]
pub use glium_backend::*;
#[cfg(feature = "xorg-glow")]
pub use glow_backend::*;
use parking_lot::Mutex;
#[cfg(feature = "wayland")]
pub use wayland_backend::*;

use crate::app::Page;
use crate::AmdGui;

pub fn create_ui(amd_gui: Arc<Mutex<AmdGui>>, ctx: &egui::Context) {
    egui::containers::TopBottomPanel::new(TopBottomSide::Top, "menu").show(ctx, |ui| {
        let mut child = ui.child_ui(ui.available_rect_before_wrap(), Layout::left_to_right());

        if child
            .add(
                egui::Button::new("Config"), /* .text_style(TextStyle::Heading) */
            )
            .clicked_by(PointerButton::Primary)
        {
            amd_gui.lock().page = Page::Config;
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
        let page = gui.page;
        match page {
            Page::Config => {
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
