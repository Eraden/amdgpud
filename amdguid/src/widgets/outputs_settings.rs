use std::collections::{BTreeMap, HashMap};

use amdgpu::pidfile::ports::{Output, Status};
use egui::Ui;
use epaint::ColorImage;
use image::ImageFormat;

use crate::app::StatefulConfig;
use crate::widgets::output_widget::OutputWidget;

#[derive(Default)]
pub struct OutputsSettings {
    textures: HashMap<String, egui::TextureHandle>,
}

impl OutputsSettings {
    pub fn draw(
        &mut self,
        ui: &mut Ui,
        _state: &mut StatefulConfig,
        outputs: &BTreeMap<String, Vec<Output>>,
    ) {
        outputs.values().flatten().for_each(|output| {
            // 160x160
            let image = {
                let bytes = include_bytes!("../../assets/icons/ports.jpg");
                image::load_from_memory(bytes).unwrap()
            };

            for (_idx, _pixel, _p) in image.to_rgba8().enumerate_pixels() {
                // let bytes = pixel.;
                // eprintln!("{:?}", bytes);
            }

            if !self.textures.contains_key(&output.port_type) {
                let img = image::load_from_memory_with_format(
                    include_bytes!("../../assets/icons/ports.jpg"),
                    ImageFormat::Jpeg,
                )
                .unwrap();
                let image_buffer = img.to_rgba8();
                let size = [image.width() as _, image.height() as _];
                let pixels = image_buffer.as_flat_samples();
                let _ = ui.ctx().load_texture(
                    output.port_type.clone(),
                    ColorImage::from_rgba_unmultiplied(size, pixels.as_slice()),
                );
            }
        });
        let _available = ui.available_rect_before_wrap();

        ui.vertical(|ui| {
            ui.horizontal_top(|ui| {
                outputs.iter().for_each(|(name, outputs)| {
                    ui.vertical(|ui| {
                        ui.label(format!("name {name}"));
                        ui.horizontal_top(|ui| {
                            outputs.iter().for_each(|output| {
                                ui.vertical(|ui| {
                                    ui.add(OutputWidget::new(output, &self.textures));

                                    ui.label(format!("port_number {}", output.port_number));
                                    ui.label(format!("port_type {:?}", output.port_type));
                                    ui.label(format!("card {}", output.card));
                                    ui.label(format!(
                                        "port_name {}",
                                        output.port_name.as_deref().unwrap_or_default()
                                    ));
                                    ui.label(match output.status {
                                        Status::Connected => "Connected",
                                        Status::Disconnected => "Disconnected",
                                    });
                                });
                            });
                        });
                    });
                });
            });
            // eprintln!("==============================================================");
        });
    }
}
