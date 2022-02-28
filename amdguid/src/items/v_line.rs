use crate::items::plot_item::PlotItem;
use crate::items::value::Value;
use crate::items::values::Values;
use crate::items::LineStyle;
use crate::transform::{Bounds, ScreenTransform};
use egui::Ui;
use epaint::{Color32, Shape, Stroke};
use std::ops::RangeInclusive;

impl PlotItem for VLine {
    fn get_shapes(&self, _ui: &mut Ui, transform: &ScreenTransform, shapes: &mut Vec<Shape>) {
        let VLine {
            x,
            stroke,
            highlight,
            style,
            ..
        } = self;
        let points = vec![
            transform.position_from_value(&Value::new(*x, transform.bounds().min[1])),
            transform.position_from_value(&Value::new(*x, transform.bounds().max[1])),
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
        bounds.min[0] = self.x;
        bounds.max[0] = self.x;
        bounds
    }
}

/// A vertical line in a plot, filling the full width
#[derive(Clone, Debug, PartialEq)]
pub struct VLine {
    pub(crate) x: f64,
    pub(crate) stroke: Stroke,
    pub(crate) name: String,
    pub(crate) highlight: bool,
    pub(crate) style: LineStyle,
}

impl VLine {
    pub fn new(x: impl Into<f64>) -> Self {
        Self {
            x: x.into(),
            stroke: Stroke::new(1.0, Color32::TRANSPARENT),
            name: String::default(),
            highlight: false,
            style: LineStyle::Solid,
        }
    }

    /// Highlight this line in the plot by scaling up the line.
    #[must_use]
    pub fn highlight(mut self) -> Self {
        self.highlight = true;
        self
    }

    /// Add a stroke.
    #[must_use]
    pub fn stroke(mut self, stroke: impl Into<Stroke>) -> Self {
        self.stroke = stroke.into();
        self
    }

    /// Stroke width. A high value means the plot thickens.
    #[must_use]
    pub fn width(mut self, width: impl Into<f32>) -> Self {
        self.stroke.width = width.into();
        self
    }

    /// Stroke color. Default is `Color32::TRANSPARENT` which means a color will be auto-assigned.
    #[must_use]
    pub fn color(mut self, color: impl Into<Color32>) -> Self {
        self.stroke.color = color.into();
        self
    }

    /// Set the line's style. Default is `LineStyle::Solid`.
    #[must_use]
    pub fn style(mut self, style: LineStyle) -> Self {
        self.style = style;
        self
    }

    /// Name of this vertical line.
    ///
    /// This name will show up in the plot legend, if legends are turned on.
    ///
    /// Multiple plot items may share the same name, in which case they will also share an entry in
    /// the legend.
    #[allow(clippy::needless_pass_by_value)]
    #[must_use]
    pub fn name(mut self, name: impl ToString) -> Self {
        self.name = name.to_string();
        self
    }
}
