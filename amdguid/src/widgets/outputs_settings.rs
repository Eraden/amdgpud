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
                        ui.label(format!("name {name}"));
                        ui.horizontal_top(|ui| {
                            outputs.iter().for_each(|output| {
                                ui.vertical(|ui| {
                                    ui.add(OutputWidget::new(output, state));

                                    ui.label(format!("port_number {}", output.port_number));
                                    ui.label(format!("port_type {:?}", output.port_type));
                                    ui.label(format!("card {}", output.card));
                                    ui.label(format!(
                                        "port_name {}",
                                        output.port_name.as_deref().unwrap_or_default()
                                    ));

                                    let state = WidgetText::RichText(
                                        RichText::new(match output.status {
                                            Status::Connected => "Connected",
                                            Status::Disconnected => "Disconnected",
                                        })
                                        .color(
                                            match output.status {
                                                Status::Connected => Color32::DARK_GREEN,
                                                Status::Disconnected => Color32::GRAY,
                                            },
                                        ),
                                    );

                                    ui.label(state);
                                });
                            });
                        });
                    });
                });
            });
        });
    }
}
