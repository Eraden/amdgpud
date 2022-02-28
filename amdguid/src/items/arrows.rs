use crate::items::plot_item::PlotItem;
use crate::items::values::Values;
use crate::transform::{Bounds, ScreenTransform};
use egui::Ui;
use epaint::{Color32, Shape};
use std::ops::RangeInclusive;

/// A set of arrows.
pub struct Arrows {
    pub(crate) origins: Values,
    pub(crate) tips: Values,
    pub(crate) color: Color32,
    pub(crate) name: String,
    pub(crate) highlight: bool,
}

impl Arrows {
    pub fn new(origins: Values, tips: Values) -> Self {
        Self {
            origins,
            tips,
            color: Color32::TRANSPARENT,
            name: Default::default(),
            highlight: false,
        }
    }
}

impl PlotItem for Arrows {
    fn get_shapes(&self, _ui: &mut Ui, transform: &ScreenTransform, shapes: &mut Vec<Shape>) {
        use egui::emath::*;
        use epaint::Stroke;
        let Self {
            origins,
            tips,
            color,
            highlight,
            ..
        } = self;
        let stroke = Stroke::new(if *highlight { 2.0 } else { 1.0 }, *color);
        origins
            .values
            .iter()
            .zip(tips.values.iter())
            .map(|(origin, tip)| {
                (
                    transform.position_from_value(origin),
                    transform.position_from_value(tip),
                )
            })
            .for_each(|(origin, tip)| {
                let vector = tip - origin;
                let rot = Rot2::from_angle(std::f32::consts::TAU / 10.0);
                let tip_length = vector.length() / 4.0;
                let tip = origin + vector;
                let dir = vector.normalized();
                shapes.push(Shape::line_segment([origin, tip], stroke));
                shapes.push(Shape::line(
                    vec![
                        tip - tip_length * (rot.inverse() * dir),
                        tip,
                        tip - tip_length * (rot * dir),
                    ],
                    stroke,
                ));
            });
    }

    fn initialize(&mut self, _x_range: RangeInclusive<f64>) {
        self.origins
            .generate_points(f64::NEG_INFINITY..=f64::INFINITY);
        self.tips.generate_points(f64::NEG_INFINITY..=f64::INFINITY);
    }

    fn name(&self) -> &str {
        self.name.as_str()
    }

    fn color(&self) -> Color32 {
        self.color
    }

    fn highlight(&mut self) {
        self.highlight = true;
    }

    fn highlighted(&self) -> bool {
        self.highlight
    }

    fn values(&self) -> Option<&Values> {
        Some(&self.origins)
    }

    fn get_bounds(&self) -> Bounds {
        self.origins.get_bounds()
    }
}
