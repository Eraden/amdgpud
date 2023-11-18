use amdgpu_config::fan::TempPoint;
use egui::{PointerButton, Response, Sense, Ui, Widget};

use crate::app::FanConfig;

pub struct TempConfigFile<'l> {
    config: FanConfig,
    matrix: &'l mut [TempPoint],
}

impl<'l> TempConfigFile<'l> {
    pub fn new(config: FanConfig, matrix: &'l mut [TempPoint]) -> Self {
        Self { config, matrix }
    }
}

impl<'l> Widget for TempConfigFile<'l> {
    fn ui(self, ui: &mut Ui) -> Response {
        let config = self.config.clone();

        ui.vertical(|ui| {
            let mut matrix = self.matrix.iter_mut().enumerate().peekable();

            let mut prev: Option<TempPoint> = None;

            while let Some((idx, current)) = matrix.next() {
                let min: TempPoint = if current == &TempPoint::MIN {
                    TempPoint::MIN
                } else if let Some(prev) = &prev {
                    *prev
                } else {
                    TempPoint::MIN
                };
                let next = matrix.peek();
                let max: TempPoint = if current == &TempPoint::MAX {
                    TempPoint::MAX
                } else if let Some(next) = next.map(|(_, n)| n) {
                    TempPoint::new(next.temp, next.speed)
                } else {
                    TempPoint::MAX
                };

                {
                    ui.label("Speed");
                    if ui
                        .add(egui::Slider::new(&mut current.speed, min.speed..=max.speed))
                        .changed()
                    {
                        if let Some(entry) = config.lock().temp_matrix_mut().get_mut(idx) {
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
                        if let Some(entry) = config.lock().temp_matrix_mut().get_mut(idx) {
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
                            config.lock().temp_matrix_vec_mut().insert(
                                idx + 1,
                                TempPoint::new(
                                    min.speed + ((max.speed - min.speed) / 2.0),
                                    min.temp + ((max.temp - min.temp) / 2.0),
                                ),
                            )
                        }
                    } else if next.is_none()
                        && *current != TempPoint::MAX
                        && ui
                            .add(egui::Button::new("Add"))
                            .clicked_by(PointerButton::Primary)
                    {
                        config
                            .lock()
                            .temp_matrix_vec_mut()
                            .push(TempPoint::new(100.0, 100.0))
                    }
                    if ui
                        .add(egui::Button::new("Remove"))
                        .clicked_by(PointerButton::Primary)
                    {
                        config.lock().temp_matrix_vec_mut().remove(idx);
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
