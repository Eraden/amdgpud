use amdgpu_config::fan::MatrixPoint;
use egui::{PointerButton, Response, Sense, Ui, Widget};

use crate::app::FanConfig;

pub struct ConfigFile<'l> {
    config: FanConfig,
    matrix: &'l mut [MatrixPoint],
}

impl<'l> ConfigFile<'l> {
    pub fn new(config: FanConfig, matrix: &'l mut [MatrixPoint]) -> Self {
        Self { config, matrix }
    }
}

impl<'l> Widget for ConfigFile<'l> {
    fn ui(self, ui: &mut Ui) -> Response {
        let config = self.config.clone();

        ui.vertical(|ui| {
            let mut matrix = self.matrix.iter_mut().enumerate().peekable();

            let mut prev: Option<MatrixPoint> = None;

            while let Some((idx, current)) = matrix.next() {
                let min: MatrixPoint = if current == &MatrixPoint::MIN {
                    MatrixPoint::MIN
                } else if let Some(prev) = &prev {
                    *prev
                } else {
                    MatrixPoint::MIN
                };
                let next = matrix.peek();
                let max: MatrixPoint = if current == &MatrixPoint::MAX {
                    MatrixPoint::MAX
                } else if let Some(next) = next.map(|(_, n)| n) {
                    MatrixPoint::new(next.temp, next.speed)
                } else {
                    MatrixPoint::MAX
                };

                {
                    ui.label("Speed");
                    if ui
                        .add(egui::Slider::new(&mut current.speed, min.speed..=max.speed))
                        .changed()
                    {
                        if let Some(entry) = config.lock().speed_matrix_mut().get_mut(idx) {
                            entry.speed = current.speed;
                        }
                    }
                }
                {
                    ui.label("Temperature");
                    if ui
                        .add(egui::Slider::new(&mut current.temp, min.temp..=max.temp))
                        .changed()
                    {
                        if let Some(entry) = config.lock().speed_matrix_mut().get_mut(idx) {
                            entry.temp = current.temp;
                        }
                    }
                }

                ui.horizontal(|ui| {
                    if next.is_some() {
                        if ui
                            .add(egui::Button::new("Add in the middle"))
                            .clicked_by(PointerButton::Primary)
                        {
                            config.lock().speed_matrix_vec_mut().insert(
                                idx + 1,
                                MatrixPoint::new(
                                    min.speed + ((max.speed - min.speed) / 2.0),
                                    min.temp + ((max.temp - min.temp) / 2.0),
                                ),
                            )
                        }
                    } else if next.is_none()
                        && *current != MatrixPoint::MAX
                        && ui
                            .add(egui::Button::new("Add"))
                            .clicked_by(PointerButton::Primary)
                    {
                        config
                            .lock()
                            .speed_matrix_vec_mut()
                            .push(MatrixPoint::new(100.0, 100.0))
                    }
                    if ui
                        .add(egui::Button::new("Remove"))
                        .clicked_by(PointerButton::Primary)
                    {
                        config.lock().speed_matrix_vec_mut().remove(idx);
                    }
                });

                ui.separator();
                prev = Some(*current);
            }

            ui.allocate_response(ui.available_size(), Sense::click())
        })
        .inner
    }
}
