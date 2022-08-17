use std::ops::RangeInclusive;

use egui::{NumExt, Ui};
use epaint::{Color32, Rgba, Shape, Stroke};

use crate::items::plot_item::PlotItem;
use crate::items::values::Values;
use crate::items::{LineStyle, DEFAULT_FILL_ALPHA};
use crate::transform::{Bounds, ScreenTransform};

/// A convex polygon.
pub struct Polygon {
    pub series: Values,
    pub stroke: Stroke,
    pub name: String,
    pub highlight: bool,
    pub fill_alpha: f32,
    pub style: LineStyle,
}

impl Polygon {
    pub fn new(series: Values) -> Self {
        Self {
            series,
            stroke: Stroke::new(1.0, Color32::TRANSPARENT),
            name: Default::default(),
            highlight: false,
            fill_alpha: DEFAULT_FILL_ALPHA,
            style: LineStyle::Solid,
        }
    }

    /// Highlight this polygon in the plot by scaling up the stroke and reducing
    /// the fill transparency.
    #[must_use]
    pub fn highlight(mut self) -> Self {
        self.highlight = true;
        self
    }

    /// Add a custom stroke.
    #[must_use]
    pub fn stroke(mut self, stroke: impl Into<Stroke>) -> Self {
        self.stroke = stroke.into();
        self
    }

    /// Set the stroke width.
    #[must_use]
    pub fn width(mut self, width: impl Into<f32>) -> Self {
        self.stroke.width = width.into();
        self
    }

    /// Stroke color. Default is `Color32::TRANSPARENT` which means a color will
    /// be auto-assigned.
    #[must_use]
    pub fn color(mut self, color: impl Into<Color32>) -> Self {
        self.stroke.color = color.into();
        self
    }

    /// Alpha of the filled area.
    #[must_use]
    pub fn fill_alpha(mut self, alpha: impl Into<f32>) -> Self {
        self.fill_alpha = alpha.into();
        self
    }

    /// Set the outline's style. Default is `LineStyle::Solid`.
    #[must_use]
    pub fn style(mut self, style: LineStyle) -> Self {
        self.style = style;
        self
    }

    /// Name of this polygon.
    ///
    /// This name will show up in the plot legend, if legends are turned on.
    ///
    /// Multiple plot items may share the same name, in which case they will
    /// also share an entry in the legend.
    #[allow(clippy::needless_pass_by_value)]
    #[must_use]
    pub fn name(mut self, name: impl ToString) -> Self {
        self.name = name.to_string();
        self
    }
}

impl PlotItem for Polygon {
    fn get_shapes(&self, _ui: &mut Ui, transform: &ScreenTransform, shapes: &mut Vec<Shape>) {
        let Self {
            series,
            stroke,
            highlight,
            mut fill_alpha,
            style,
            ..
        } = self;

        if *highlight {
            fill_alpha = (2.0 * fill_alpha).at_most(1.0);
        }

        let mut values_tf: Vec<_> = series
            .values
            .iter()
            .map(|v| transform.position_from_value(v))
            .collect();

        let fill = Rgba::from(stroke.color).to_opaque().multiply(fill_alpha);

        let shape = Shape::convex_polygon(values_tf.clone(), fill, Stroke::none());
        shapes.push(shape);
        values_tf.push(*values_tf.first().unwrap());
        style.style_line(values_tf, *stroke, *highlight, shapes);
    }

    fn initialize(&mut self, x_range: RangeInclusive<f64>) {
        self.series.generate_points(x_range);
    }

    fn name(&self) -> &str {
        self.name.as_str()
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
        Some(&self.series)
    }

    fn get_bounds(&self) -> Bounds {
        self.series.get_bounds()
    }
}
