use std::ops::RangeInclusive;

use egui::{pos2, Color32, Image, Rect, Shape, Stroke, StrokeKind, TextureId, Ui, Vec2};

use crate::items::plot_item::PlotItem;
use crate::items::value::Value;
use crate::items::values::Values;
use crate::transform::{Bounds, ScreenTransform};

/// An image in the plot.
pub struct PlotImage {
    pub position: Value,
    pub texture_id: TextureId,
    pub uv: Rect,
    pub size: Vec2,
    pub bg_fill: Color32,
    pub tint: Color32,
    pub highlight: bool,
    pub name: String,
}

impl PlotImage {
    /// Create a new image with position and size in plot coordinates.
    pub fn new(texture_id: TextureId, position: Value, size: impl Into<Vec2>) -> Self {
        Self {
            position,
            name: Default::default(),
            highlight: false,
            texture_id,
            uv: Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
            size: size.into(),
            bg_fill: Default::default(),
            tint: Color32::WHITE,
        }
    }

    /// Highlight this image in the plot.
    #[must_use]
    pub fn highlight(mut self) -> Self {
        self.highlight = true;
        self
    }

    /// Select UV range. Default is (0,0) in top-left, (1,1) bottom right.
    #[must_use]
    pub fn uv(mut self, uv: impl Into<Rect>) -> Self {
        self.uv = uv.into();
        self
    }

    /// A solid color to put behind the image. Useful for transparent images.
    #[must_use]
    pub fn bg_fill(mut self, bg_fill: impl Into<Color32>) -> Self {
        self.bg_fill = bg_fill.into();
        self
    }

    /// Multiply image color with this. Default is WHITE (no tint).
    #[must_use]
    pub fn tint(mut self, tint: impl Into<Color32>) -> Self {
        self.tint = tint.into();
        self
    }

    /// Name of this image.
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

impl PlotItem for PlotImage {
    fn shapes(&self, ui: &mut Ui, transform: &ScreenTransform, shapes: &mut Vec<Shape>) {
        let Self {
            position,
            texture_id,
            uv,
            size,
            bg_fill,
            tint,
            highlight,
            ..
        } = self;
        let rect = {
            let left_top = Value::new(
                position.x as f32 - size.x / 2.0,
                position.y as f32 - size.y / 2.0,
            );
            let right_bottom = Value::new(
                position.x as f32 + size.x / 2.0,
                position.y as f32 + size.y / 2.0,
            );
            let left_top_tf = transform.position_from_value(&left_top);
            let right_bottom_tf = transform.position_from_value(&right_bottom);
            Rect::from_two_pos(left_top_tf, right_bottom_tf)
        };
        Image::new((*texture_id, *size))
            .bg_fill(*bg_fill)
            .tint(*tint)
            .uv(*uv)
            .paint_at(ui, rect);
        if *highlight {
            shapes.push(Shape::rect_stroke(
                rect,
                0.0,
                Stroke::new(1.0, ui.visuals().strong_text_color()),
                StrokeKind::Inside,
            ));
        }
    }

    fn initialize(&mut self, _x_range: RangeInclusive<f64>) {}

    fn name(&self) -> &str {
        self.name.as_str()
    }

    fn color(&self) -> Color32 {
        Color32::TRANSPARENT
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

    fn bounds(&self) -> Bounds {
        let mut bounds = Bounds::NOTHING;
        let left_top = Value::new(
            self.position.x as f32 - self.size.x / 2.0,
            self.position.y as f32 - self.size.y / 2.0,
        );
        let right_bottom = Value::new(
            self.position.x as f32 + self.size.x / 2.0,
            self.position.y as f32 + self.size.y / 2.0,
        );
        bounds.extend_with(&left_top);
        bounds.extend_with(&right_bottom);
        bounds
    }
}
