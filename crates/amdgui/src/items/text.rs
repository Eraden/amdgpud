use std::ops::RangeInclusive;

use egui::{Align2, Color32, Rect, Shape, Stroke, TextStyle, Ui};

use crate::items::plot_item::PlotItem;
use crate::items::value::Value;
use crate::items::values::Values;
use crate::transform::{Bounds, ScreenTransform};

/// Text inside the plot.
pub struct Text {
    pub(crate) text: String,
    pub(crate) style: TextStyle,
    pub(crate) position: Value,
    pub(crate) name: String,
    pub(crate) highlight: bool,
    pub(crate) color: Color32,
    pub(crate) anchor: Align2,
}

impl Text {
    #[allow(clippy::needless_pass_by_value)]
    pub fn new(position: Value, text: impl ToString) -> Self {
        Self {
            text: text.to_string(),
            style: TextStyle::Small,
            position,
            name: Default::default(),
            highlight: false,
            color: Color32::TRANSPARENT,
            anchor: Align2::CENTER_CENTER,
        }
    }

    /// Highlight this text in the plot by drawing a rectangle around it.
    #[must_use]
    pub fn highlight(mut self) -> Self {
        self.highlight = true;
        self
    }

    /// Text style. Default is `TextStyle::Small`.
    #[must_use]
    pub fn style(mut self, style: TextStyle) -> Self {
        self.style = style;
        self
    }

    /// Text color. Default is `Color32::TRANSPARENT` which means a color will
    /// be auto-assigned.
    #[must_use]
    pub fn color(mut self, color: impl Into<Color32>) -> Self {
        self.color = color.into();
        self
    }

    /// Anchor position of the text. Default is `Align2::CENTER_CENTER`.
    #[must_use]
    pub fn anchor(mut self, anchor: Align2) -> Self {
        self.anchor = anchor;
        self
    }

    /// Name of this text.
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

impl PlotItem for Text {
    fn shapes(&self, ui: &mut Ui, transform: &ScreenTransform, shapes: &mut Vec<Shape>) {
        let color = if self.color == Color32::TRANSPARENT {
            ui.style().visuals.text_color()
        } else {
            self.color
        };
        let fond_id = ui.style().text_styles.get(&self.style).unwrap();
        let pos = transform.position_from_value(&self.position);
        let galley = ui
            .fonts()
            .layout_no_wrap(self.text.clone(), fond_id.clone(), color);
        let rect = self
            .anchor
            .anchor_rect(Rect::from_min_size(pos, galley.size()));
        shapes.push(Shape::galley(rect.min, galley));
        if self.highlight {
            shapes.push(Shape::rect_stroke(
                rect.expand(2.0),
                1.0,
                Stroke::new(0.5, color),
            ));
        }
    }

    fn initialize(&mut self, _x_range: RangeInclusive<f64>) {}

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
        None
    }

    fn bounds(&self) -> Bounds {
        let mut bounds = Bounds::NOTHING;
        bounds.extend_with(&self.position);
        bounds
    }
}
