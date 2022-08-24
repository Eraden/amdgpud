use std::collections::BTreeMap;

use amdgpu::pidfile::ports::{Output, Status};
use egui::{RichText, Ui, WidgetText};
use epaint::Color32;

use crate::app::StatefulConfig;
use crate::widgets::output_widget::OutputWidget;

#[derive(Default)]
pub struct OutputsSettings {}

impl OutputsSettings {
    pub fn draw(
        &mut self,
        ui: &mut Ui,
        state: &mut StatefulConfig,
        outputs: &BTreeMap<String, Vec<Output>>,
    ) {
        let _available = ui.available_rect_before_wrap();

        ui.vertical(|ui| {
            ui.horizontal_top(|ui| {
                outputs.iter().for_each(|(name, outputs)| {
                    ui.vertical(|ui| {
                        ui.label(format!("Card {name}"));
                        ui.horizontal_top(|ui| {
                            outputs.iter().for_each(|output| {
                                Self::render_single(ui, state, output);
                            });
                        });
                    });
                });
            });
        });
    }

    fn render_single(ui: &mut Ui, state: &mut StatefulConfig, output: &Output) {
        ui.vertical(|ui| {
            ui.add(OutputWidget::new(output, state));

            ui.label(format!("Port type {:?}", output.port_type));
            ui.label(format!("Port number {}", output.port_number));
            if let Some(name) = output.port_name.as_deref() {
                ui.label(format!("Port name {}", name));
            }

            ui.label(WidgetText::RichText(
                RichText::new(match output.status {
                    Status::Connected => "Connected",
                    Status::Disconnected => "Disconnected",
                })
                .color(match output.status {
                    Status::Connected => Color32::GREEN,
                    Status::Disconnected => Color32::GRAY,
                })
                .code()
                .strong()
                .monospace(),
            ));
            ui.label("Display Power Management");
            if ui
                .button(WidgetText::RichText(
                    RichText::new(match output.display_power_managment {
                        true => "On",
                        false => "Off",
                    })
                    .color(match output.display_power_managment {
                        true => Color32::GREEN,
                        false => Color32::GRAY,
                    })
                    .monospace()
                    .code()
                    .strong(),
                ))
                .clicked()
            {
                //
            }
        });
    }
}
