use amdgpu_config::fan::UsagePoint;
use egui::{PointerButton, Response, Sense, Ui, Widget};

use crate::app::FanConfig;

pub struct UsageConfigFile {
    config: FanConfig,
}

impl UsageConfigFile {
    pub fn new(config: FanConfig) -> Self {
        Self { config }
    }
}

impl Widget for UsageConfigFile {
    fn ui(self, ui: &mut Ui) -> Response {
        let c = self.config;

        ui.vertical(|ui| {
            let mut l = c.lock();
            let matrix = l.usage_matrix_vec_mut();
            let mut iter = matrix.clone().into_iter().enumerate().peekable();

            let mut prev: Option<UsagePoint> = None;

            while let Some((idx, mut current)) = iter.next() {
                let min: UsagePoint = match prev {
                    _ if current == UsagePoint::MIN => UsagePoint::MIN,
                    Some(p) => p,
                    _ => UsagePoint::MIN,
                };

                let next = iter.peek();
                let max = match next {
                    _ if current == UsagePoint::MAX => UsagePoint::MAX,
                    Some((_, next)) => UsagePoint::new(next.usage, next.speed),
                    _ => UsagePoint::MAX,
                };

                {
                    ui.label("Usage");
                    if ui
                        .add(egui::Slider::new(&mut current.speed, min.speed..=max.speed))
                        .changed()
                    {
                        if let Some(entry) = matrix.get_mut(idx) {
                            entry.speed = current.speed;
                        }
                    }
                }
                {
                    ui.label("Temperature");
                    if ui
                        .add(egui::Slider::new(&mut current.usage, min.usage..=max.usage))
                        .changed()
                    {
                        if let Some(entry) = matrix.get_mut(idx) {
                            entry.usage = current.usage;
                        }
                    }
                }

                ui.horizontal(|ui| {
                    if next.is_some() {
                        if ui
                            .add(egui::Button::new("Add in the middle"))
                            .clicked_by(PointerButton::Primary)
                        {
                            matrix.insert(
                                idx + 1,
                                UsagePoint::new(
                                    min.speed + ((max.speed - min.speed) / 2.0),
                                    min.usage + ((max.usage - min.usage) / 2.0),
                                ),
                            )
                        }
                    } else if next.is_none()
                        && current != UsagePoint::MAX
                        && ui
                            .add(egui::Button::new("Add"))
                            .clicked_by(PointerButton::Primary)
                    {
                        matrix.push(UsagePoint::new(100.0, 100.0))
                    }
                    if ui
                        .add(egui::Button::new("Remove"))
                        .clicked_by(PointerButton::Primary)
                    {
                        matrix.remove(idx);
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
