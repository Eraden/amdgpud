use std::ops::RangeInclusive;

use egui::Ui;
use epaint::{Color32, Shape, Stroke};

use crate::items::plot_item::PlotItem;
use crate::items::value::Value;
use crate::items::values::Values;
use crate::items::LineStyle;
use crate::transform::{Bounds, ScreenTransform};

/// A horizontal line in a plot, filling the full width
#[derive(Clone, Debug, PartialEq)]
pub struct HLine {
    pub(crate) y: f64,
    pub(crate) stroke: Stroke,
    pub(crate) name: String,
    pub(crate) highlight: bool,
    pub(crate) style: LineStyle,
}

impl HLine {
    pub fn new(y: impl Into<f64>) -> Self {
        Self {
            y: y.into(),
            stroke: Stroke::new(1.0, Color32::TRANSPARENT),
            name: String::default(),
            highlight: false,
            style: LineStyle::Solid,
        }
    }

    /// Stroke color. Default is `Color32::TRANSPARENT` which means a color will
    /// be auto-assigned.
    #[must_use]
    pub fn color(mut self, color: impl Into<Color32>) -> Self {
        self.stroke.color = color.into();
        self
    }
}

impl PlotItem for HLine {
    fn get_shapes(&self, _ui: &mut Ui, transform: &ScreenTransform, shapes: &mut Vec<Shape>) {
        let HLine {
            y,
            stroke,
            highlight,
            style,
            ..
        } = self;
        let points = vec![
            transform.position_from_value(&Value::new(transform.bounds().min[0], *y)),
            transform.position_from_value(&Value::new(transform.bounds().max[0], *y)),
        ];
        style.style_line(points, *stroke, *highlight, shapes);
    }

    fn initialize(&mut self, _x_range: RangeInclusive<f64>) {}

    fn name(&self) -> &str {
        &self.name
    }

    fn color(&self) -> Color32 {
        self.stroke.color
    }

    fn highlight(&mut self) {
        self.highlight = true;
    }

    fn highlighted(&self) -> bool {
        self.highlight
    }

    fn values(&self) -> Option<&Values> {
        None
    }

    fn get_bounds(&self) -> Bounds {
        let mut bounds = Bounds::NOTHING;
        bounds.min[1] = self.y;
        bounds.max[1] = self.y;
        bounds
    }
}
