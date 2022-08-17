use std::collections::HashMap;

use amdgpu::pidfile::ports::Output;
use egui::{Response, Sense, Ui, Vec2};
use epaint::Stroke;

pub struct OutputWidget<'output, 'textures> {
    output: &'output Output,
    textures: &'textures HashMap<String, egui::TextureHandle>,
}

impl<'output, 'textures> OutputWidget<'output, 'textures> {
    pub fn new(
        output: &'output Output,
        textures: &'textures HashMap<String, egui::TextureHandle>,
    ) -> Self {
        Self { output, textures }
    }
}

impl<'output, 'textures> egui::Widget for OutputWidget<'output, 'textures> {
    fn ui(self, ui: &mut Ui) -> Response {
        let (rect, res) = ui.allocate_exact_size(Vec2::new(80.0, 70.0), Sense::click());
        // let _transform = ScreenTransform::new(rect, Bounds::new_symmetrical(1.0),
        // false, false); let _frame = *transform.frame();
        // eprintln!("min {:?} max {:?}", frame.min, frame.max);
        let painter = ui.painter(); //.sub_region(*transform.frame());

        painter.rect_filled(rect, 0.0, epaint::color::Color32::DARK_RED);
        painter.rect(rect, 2.0, epaint::color::Color32::DARK_RED, {
            let mut s = Stroke::default();
            s.color = epaint::color::Color32::GREEN;
            s.width = 1.0;
            s
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
        res
    }
}

fn icon(name: &str, textures: &HashMap<String, egui::TextureHandle>) {
    if let Some(_texture) = textures.get(name) {
        //
    } else {
        //
    }
}
