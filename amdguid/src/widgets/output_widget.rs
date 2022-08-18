use amdgpu::pidfile::ports::Output;
use egui::{Response, Sense, Ui, Vec2};
use epaint::Stroke;

use crate::app::StatefulConfig;

pub struct OutputWidget<'output, 'stateful> {
    output: &'output Output,
    state: &'stateful mut StatefulConfig,
}

impl<'output, 'stateful> OutputWidget<'output, 'stateful> {
    pub fn new(output: &'output Output, state: &'stateful mut StatefulConfig) -> Self {
        Self { output, state }
    }
}

impl<'output, 'stateful> egui::Widget for OutputWidget<'output, 'stateful> {
    fn ui(self, ui: &mut Ui) -> Response {
        let (rect, res) = ui.allocate_exact_size(Vec2::new(80.0, 80.0), Sense::click());
        if let Some(handle) = self.output.ty.and_then(|ty| self.state.textures.get(&ty)) {
            ui.image(handle.id(), handle.size_vec2());
        } else {
            let painter = ui.painter();
            painter.rect_filled(rect, 0.0, epaint::color::Color32::DARK_RED);
            painter.rect(rect, 2.0, epaint::color::Color32::DARK_RED, {
                Stroke {
                    width: 1.0,
                    color: epaint::color::Color32::GREEN,
                }
            });

            let rect_middle_point = (rect.max - rect.min) / 2.0;

            painter.circle_filled(
                rect.min + Vec2::new(rect_middle_point.x / 2.0, rect_middle_point.y),
                3.0,
                epaint::color::Color32::GREEN,
            );
            painter.circle_filled(
                rect.min + rect_middle_point,
                3.0,
                epaint::color::Color32::GREEN,
            );

            painter.circle_filled(
                rect.min
                    + Vec2::new(
                        rect_middle_point.x + (rect_middle_point.x / 2.0),
                        rect_middle_point.y,
                    ),
                3.0,
                epaint::color::Color32::GREEN,
            );
        }
        res
    }
}
