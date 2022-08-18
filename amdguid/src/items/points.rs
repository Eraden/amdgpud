use std::ops::RangeInclusive;

use egui::{pos2, vec2, Pos2, Ui};
use epaint::{Color32, Shape, Stroke};

use crate::items::marker_shape::MarkerShape;
use crate::items::plot_item::PlotItem;
use crate::items::value::Value;
use crate::items::values::Values;
use crate::transform::{Bounds, ScreenTransform};

/// A set of points.
pub struct Points {
    pub(crate) series: Values,
    pub(crate) shape: MarkerShape,
    /// Color of the marker. `Color32::TRANSPARENT` means that it will be picked
    /// automatically.
    pub(crate) color: Color32,
    /// Whether to fill the marker. Does not apply to all types.
    pub(crate) filled: bool,
    /// The maximum extent of the marker from its center.
    pub(crate) radius: f32,
    pub(crate) name: String,
    pub(crate) highlight: bool,
    pub(crate) stems: Option<f32>,
}

impl Points {
    pub fn new(series: Values) -> Self {
        Self {
            series,
            shape: MarkerShape::Circle,
            color: Color32::TRANSPARENT,
            filled: true,
            radius: 1.0,
            name: Default::default(),
            highlight: false,
            stems: None,
        }
    }

    /// Set the shape of the markers.
    #[must_use]
    pub fn shape(mut self, shape: MarkerShape) -> Self {
        self.shape = shape;
        self
    }

    /// Highlight these points in the plot by scaling up their markers.
    #[must_use]
    pub fn highlight(mut self) -> Self {
        self.highlight = true;
        self
    }

    /// Set the marker's color.
    #[must_use]
    pub fn color(mut self, color: impl Into<Color32>) -> Self {
        self.color = color.into();
        self
    }

    /// Whether to fill the marker.
    #[must_use]
    pub fn filled(mut self, filled: bool) -> Self {
        self.filled = filled;
        self
    }

    /// Whether to add stems between the markers and a horizontal reference
    /// line.
    #[must_use]
    pub fn stems(mut self, y_reference: impl Into<f32>) -> Self {
        self.stems = Some(y_reference.into());
        self
    }

    /// Set the maximum extent of the marker around its position.
    #[must_use]
    pub fn radius(mut self, radius: impl Into<f32>) -> Self {
        self.radius = radius.into();
        self
    }

    /// Name of this set of points.
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

impl PlotItem for Points {
    fn get_shapes(&self, _ui: &mut Ui, transform: &ScreenTransform, shapes: &mut Vec<Shape>) {
        let sqrt_3 = 3f32.sqrt();
        let frac_sqrt_3_2 = 3f32.sqrt() / 2.0;
        let frac_1_sqrt_2 = 1.0 / 2f32.sqrt();

        let Self {
            series,
            shape,
            color,
            filled,
            mut radius,
            highlight,
            stems,
            ..
        } = self;

        let stroke_size = radius / 5.0;

        let default_stroke = Stroke::new(stroke_size, *color);
        let mut stem_stroke = default_stroke;
        let stroke = (!filled)
            .then_some(default_stroke)
            .unwrap_or_else(Stroke::none);
        let fill = filled.then(|| *color).unwrap_or_default();

        if *highlight {
            radius *= 2f32.sqrt();
            stem_stroke.width *= 2.0;
        }

        let y_reference =
            stems.map(|y| transform.position_from_value(&Value::new(0.0, y)).y as f32);

        series
            .values
            .iter()
            .map(|value| transform.position_from_value(value))
            .for_each(|center| {
                let tf = |dx: f32, dy: f32| -> Pos2 { center + radius * vec2(dx, dy) };

                if let Some(y) = y_reference {
                    let stem = Shape::line_segment([center, pos2(center.x, y)], stem_stroke);
                    shapes.push(stem);
                }

                match shape {
                    MarkerShape::Circle => {
                        shapes.push(Shape::Circle(epaint::CircleShape {
                            center,
                            radius,
                            fill,
                            stroke,
                        }));
                    }
                    MarkerShape::Diamond => {
                        let points = vec![tf(1.0, 0.0), tf(0.0, -1.0), tf(-1.0, 0.0), tf(0.0, 1.0)];
                        shapes.push(Shape::convex_polygon(points, fill, stroke));
                    }
                    MarkerShape::Square => {
                        let points = vec![
                            tf(frac_1_sqrt_2, frac_1_sqrt_2),
                            tf(frac_1_sqrt_2, -frac_1_sqrt_2),
                            tf(-frac_1_sqrt_2, -frac_1_sqrt_2),
                            tf(-frac_1_sqrt_2, frac_1_sqrt_2),
                        ];
                        shapes.push(Shape::convex_polygon(points, fill, stroke));
                    }
                    MarkerShape::Cross => {
                        let diagonal1 = [
                            tf(-frac_1_sqrt_2, -frac_1_sqrt_2),
                            tf(frac_1_sqrt_2, frac_1_sqrt_2),
                        ];
                        let diagonal2 = [
                            tf(frac_1_sqrt_2, -frac_1_sqrt_2),
                            tf(-frac_1_sqrt_2, frac_1_sqrt_2),
                        ];
                        shapes.push(Shape::line_segment(diagonal1, default_stroke));
                        shapes.push(Shape::line_segment(diagonal2, default_stroke));
                    }
                    MarkerShape::Plus => {
                        let horizontal = [tf(-1.0, 0.0), tf(1.0, 0.0)];
                        let vertical = [tf(0.0, -1.0), tf(0.0, 1.0)];
                        shapes.push(Shape::line_segment(horizontal, default_stroke));
                        shapes.push(Shape::line_segment(vertical, default_stroke));
                    }
                    MarkerShape::Up => {
                        let points =
                            vec![tf(0.0, -1.0), tf(-0.5 * sqrt_3, 0.5), tf(0.5 * sqrt_3, 0.5)];
                        shapes.push(Shape::convex_polygon(points, fill, stroke));
                    }
                    MarkerShape::Down => {
                        let points = vec![
                            tf(0.0, 1.0),
                            tf(-0.5 * sqrt_3, -0.5),
                            tf(0.5 * sqrt_3, -0.5),
                        ];
                        shapes.push(Shape::convex_polygon(points, fill, stroke));
                    }
                    MarkerShape::Left => {
                        let points =
                            vec![tf(-1.0, 0.0), tf(0.5, -0.5 * sqrt_3), tf(0.5, 0.5 * sqrt_3)];
                        shapes.push(Shape::convex_polygon(points, fill, stroke));
                    }
                    MarkerShape::Right => {
                        let points = vec![
                            tf(1.0, 0.0),
                            tf(-0.5, -0.5 * sqrt_3),
                            tf(-0.5, 0.5 * sqrt_3),
                        ];
                        shapes.push(Shape::convex_polygon(points, fill, stroke));
                    }
                    MarkerShape::Asterisk => {
                        let vertical = [tf(0.0, -1.0), tf(0.0, 1.0)];
                        let diagonal1 = [tf(-frac_sqrt_3_2, 0.5), tf(frac_sqrt_3_2, -0.5)];
                        let diagonal2 = [tf(-frac_sqrt_3_2, -0.5), tf(frac_sqrt_3_2, 0.5)];
                        shapes.push(Shape::line_segment(vertical, default_stroke));
                        shapes.push(Shape::line_segment(diagonal1, default_stroke));
                        shapes.push(Shape::line_segment(diagonal2, default_stroke));
                    }
                }
            });
    }

    fn initialize(&mut self, x_range: RangeInclusive<f64>) {
        self.series.generate_points(x_range);
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
        Some(&self.series)
    }

    fn get_bounds(&self) -> Bounds {
        self.series.get_bounds()
    }
}
