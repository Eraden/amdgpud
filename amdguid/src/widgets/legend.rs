use std::string::String;

use egui::{pos2, vec2, Align, PointerButton, Rect, Response, Sense, WidgetInfo, WidgetType};
use epaint::{Color32, TextStyle};

/// Where to place the plot legend.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Corner {
    LeftTop,
    RightTop,
    LeftBottom,
    RightBottom,
}

impl Corner {
    pub fn all() -> impl Iterator<Item = Corner> {
        [
            Corner::LeftTop,
            Corner::RightTop,
            Corner::LeftBottom,
            Corner::RightBottom,
        ]
        .iter()
        .copied()
    }
}

/// The configuration for a plot legend.
#[derive(Clone, Copy, PartialEq)]
pub struct Legend {
    pub text_style: TextStyle,
    pub background_alpha: f32,
    pub position: Corner,
}

impl Default for Legend {
    fn default() -> Self {
        Self {
            text_style: TextStyle::Body,
            background_alpha: 0.75,
            position: Corner::RightTop,
        }
    }
}

impl Legend {
    /// Which text style to use for the legend. Default: `TextStyle::Body`.
    pub fn text_style(mut self, style: TextStyle) -> Self {
        self.text_style = style;
        self
    }

    /// The alpha of the legend background. Default: `0.75`.
    pub fn background_alpha(mut self, alpha: f32) -> Self {
        self.background_alpha = alpha;
        self
    }

    /// In which corner to place the legend. Default: `Corner::RightTop`.
    pub fn position(mut self, corner: Corner) -> Self {
        self.position = corner;
        self
    }
}

#[derive(Clone)]
pub struct LegendEntry {
    pub color: Color32,
    pub checked: bool,
    pub hovered: bool,
}

impl LegendEntry {
    pub fn new(color: Color32, checked: bool) -> Self {
        Self {
            color,
            checked,
            hovered: false,
        }
    }

    pub fn ui(&mut self, ui: &mut egui::Ui, text: String) -> Response {
        let Self {
            color,
            checked,
            hovered,
        } = self;

        let galley =
            ui.fonts()
                .layout_delayed_color(text, ui.style().body_text_style, f32::INFINITY);

        let icon_size = galley.size().y;
        let icon_spacing = icon_size / 5.0;
        let total_extra = vec2(icon_size + icon_spacing, 0.0);

        let desired_size = total_extra + galley.size();
        let (rect, response) = ui.allocate_exact_size(desired_size, Sense::click());

        response
            .widget_info(|| WidgetInfo::selected(WidgetType::Checkbox, *checked, galley.text()));

        let visuals = ui.style().interact(&response);
        let label_on_the_left = ui.layout().horizontal_placement() == Align::RIGHT;

        let icon_position_x = if label_on_the_left {
            rect.right() - icon_size / 2.0
        } else {
            rect.left() + icon_size / 2.0
        };
        let icon_position = pos2(icon_position_x, rect.center().y);
        let icon_rect = Rect::from_center_size(icon_position, vec2(icon_size, icon_size));

        let painter = ui.painter();

        painter.add(epaint::CircleShape {
            center: icon_rect.center(),
            radius: icon_size * 0.5,
            fill: visuals.bg_fill,
            stroke: visuals.bg_stroke,
        });

        if *checked {
            let fill = if *color == Color32::TRANSPARENT {
                ui.visuals().noninteractive().fg_stroke.color
            } else {
                *color
            };
            painter.add(epaint::Shape::circle_filled(
                icon_rect.center(),
                icon_size * 0.4,
                fill,
            ));
        }

        let text_position_x = if label_on_the_left {
            rect.right() - icon_size - icon_spacing - galley.size().x
        } else {
            rect.left() + icon_size + icon_spacing
        };

        let text_position = pos2(text_position_x, rect.center().y - 0.5 * galley.size().y);
        painter.galley_with_color(text_position, galley, visuals.text_color());

        *checked ^= response.clicked_by(PointerButton::Primary);
        *hovered = response.hovered();

        response
    }
}
