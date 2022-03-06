use egui::{PointerButton, Response, Sense, Ui, Widget};

use amdgpu_config::fan::MatrixPoint;

use crate::app::FanConfig;

pub struct ConfigFile {
    config: FanConfig,
}

impl ConfigFile {
    pub fn new(config: FanConfig) -> Self {
        Self { config }
    }
}

impl Widget for ConfigFile {
    fn ui(self, ui: &mut Ui) -> Response {
        ui.vertical(|ui| {
            let mut matrix = { self.config.lock().speed_matrix().to_vec() }
                .into_iter()
                .enumerate()
                .peekable();

            let mut prev = None;

            while let Some((idx, mut current)) = matrix.next() {
                let min = if current == MatrixPoint::MIN {
                    MatrixPoint::MIN
                } else if let Some(prev) = prev {
                    prev
                } else {
                    MatrixPoint::MIN
                };
                let next = matrix.peek();
                let max = if current == MatrixPoint::MAX {
                    MatrixPoint::MAX
                } else if let Some(next) = next.map(|(_, n)| n) {
                    *next
                } else {
                    MatrixPoint::MAX
                };

                {
                    ui.label("Temperature");
                    if ui
                        .add(egui::Slider::new(&mut current.temp, min.temp..=max.temp))
                        .changed()
                    {
                        if let Some(entry) = self.config.lock().speed_matrix_mut().get_mut(idx) {
                            entry.temp = current.temp;
                        }
                    }
                }
                {
                    ui.label("Speed");
                    if ui
                        .add(egui::Slider::new(&mut current.speed, min.speed..=max.speed))
                        .changed()
                    {
                        if let Some(entry) = self.config.lock().speed_matrix_mut().get_mut(idx) {
                            entry.speed = current.speed;
                        }
                    }
                }

                ui.horizontal(|ui| {
                    if next.is_some() {
                        if ui
                            .add(egui::Button::new("Add in middle"))
                            .clicked_by(PointerButton::Primary)
                        {
                            self.config.lock().speed_matrix_vec_mut().insert(
                                idx + 1,
                                MatrixPoint::new(
                                    min.temp + ((max.temp - min.temp) / 2.0),
                                    min.speed + ((max.speed - min.speed) / 2.0),
                                ),
                            )
                        }
                    } else if next.is_none()
                        && current != MatrixPoint::MAX
                        && ui
                            .add(egui::Button::new("Add"))
                            .clicked_by(PointerButton::Primary)
                    {
                        self.config
                            .lock()
                            .speed_matrix_vec_mut()
                            .push(MatrixPoint::new(100.0, 100.0))
                    }
                    if ui
                        .add(egui::Button::new("Remove"))
                        .clicked_by(PointerButton::Primary)
                    {
                        self.config.lock().speed_matrix_vec_mut().remove(idx);
                    }
                });

                ui.separator();
                prev = Some(current);
            }

            ui.allocate_response(ui.available_size(), Sense::click())
        })
        .inner
    }
}
