use egui::{emath, vec2, CursorIcon, Id, NumExt, PointerButton, Response, Sense, Ui, Vec2};
use epaint::ahash::AHashSet;
use epaint::color::Hsva;
use epaint::Color32;

use crate::items::HLine;
use crate::items::*;
use crate::transform::{Bounds, ScreenTransform};
use crate::widgets::drag_plot_prepared::DragPlotPrepared;
use crate::widgets::legend::Legend;
use crate::widgets::legend_widget::LegendWidget;

#[derive(Debug)]
pub enum PlotMsg {
    Clicked(usize),
    Drag(emath::Vec2),
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
struct PlotMemory {
    bounds: Bounds,
    auto_bounds: bool,
    hovered_entry: Option<String>,
    hidden_items: AHashSet<String>,
    min_auto_bounds: Bounds,
}

pub struct DragPlot<OnEvent>
where
    OnEvent: FnMut(PlotMsg),
{
    id: egui::Id,

    items: Vec<Box<dyn PlotItem>>,

    min_auto_bounds: Bounds,
    min_size: Vec2,
    width: Option<f32>,
    height: Option<f32>,
    data_aspect: Option<f32>,
    view_aspect: Option<f32>,
    legend_config: Option<Legend>,

    next_auto_color_idx: usize,
    allow_zoom: bool,
    allow_drag: bool,
    margin_fraction: Vec2,
    selected: Option<usize>,
    on_event: Option<OnEvent>,

    lines: Vec<Line>,
    axis_names: [String; 2],
}

impl<OnEvent> DragPlot<OnEvent>
where
    OnEvent: FnMut(PlotMsg),
{
    pub fn new(id_source: impl std::hash::Hash) -> Self {
        Self {
            id: Id::new(id_source),

            items: Default::default(),
            min_size: Vec2::splat(64.0),
            width: None,
            height: None,
            data_aspect: None,
            view_aspect: None,
            min_auto_bounds: Bounds::NOTHING,
            legend_config: None,
            next_auto_color_idx: 0,
            allow_zoom: true,
            allow_drag: true,
            margin_fraction: Vec2::splat(0.05),
            selected: None,
            on_event: None,
            lines: vec![],
            axis_names: [String::from("x"), String::from("y")],
        }
    }

    #[must_use]
    pub fn x_axis_name(mut self, name: String) -> Self {
        self.axis_names[0] = name;
        self
    }

    #[must_use]
    pub fn y_axis_name(mut self, name: String) -> Self {
        self.axis_names[1] = name;
        self
    }

    /// Show a legend including all named items.
    #[must_use]
    pub fn legend(mut self, legend: Legend) -> Self {
        self.legend_config = Some(legend);
        self
    }

    /// Add a data lines.
    #[must_use]
    pub fn line(mut self, mut line: Line) -> Self {
        if line.series.is_empty() {
            return self;
        };

        // Give the stroke an automatic color if no color has been assigned.
        if line.stroke.color == Color32::TRANSPARENT {
            line.stroke.color = self.auto_color();
        }
        self.lines.push(line);
        self
    }

    #[must_use]
    pub fn selected(mut self, selected: Option<usize>) -> Self {
        self.selected = selected;
        self
    }

    #[must_use]
    fn auto_color(&mut self) -> Color32 {
        let i = self.next_auto_color_idx;
        self.next_auto_color_idx += 1;
        let golden_ratio = (5.0_f32.sqrt() - 1.0) / 2.0; // 0.61803398875
        let h = i as f32 * golden_ratio;
        Hsva::new(h, 0.85, 0.5, 1.0).into() // TODO: OkLab or some other perspective color space
    }

    /// width / height ratio of the data.
    /// For instance, it can be useful to set this to `1.0` for when the two axes show the same
    /// unit.
    /// By default the plot window's aspect ratio is used.
    #[must_use]
    pub fn data_aspect(mut self, data_aspect: f32) -> Self {
        self.data_aspect = Some(data_aspect);
        self
    }

    /// width / height ratio of the plot region.
    /// By default no fixed aspect ratio is set (and width/height will fill the ui it is in).
    #[must_use]
    pub fn view_aspect(mut self, view_aspect: f32) -> Self {
        self.view_aspect = Some(view_aspect);
        self
    }

    /// Width of plot. By default a plot will fill the ui it is in.
    /// If you set [`Self::view_aspect`], the width can be calculated from the height.
    #[must_use]
    pub fn width(mut self, width: f32) -> Self {
        self.min_size.x = width;
        self.width = Some(width);
        self
    }

    /// Height of plot. By default a plot will fill the ui it is in.
    /// If you set [`Self::view_aspect`], the height can be calculated from the width.
    #[must_use]
    pub fn height(mut self, height: f32) -> Self {
        self.min_size.y = height;
        self.height = Some(height);
        self
    }

    /// Minimum size of the plot view.
    #[must_use]
    pub fn min_size(mut self, min_size: Vec2) -> Self {
        self.min_size = min_size;
        self
    }

    #[must_use]
    pub fn allow_drag(mut self, allow_drag: bool) -> Self {
        self.allow_drag = allow_drag;
        self
    }

    #[must_use]
    pub fn allow_zoom(mut self, allow_zoom: bool) -> Self {
        self.allow_zoom = allow_zoom;
        self
    }

    /// Add a horizontal line.
    /// Can be useful e.g. to show min/max bounds or similar.
    /// Always fills the full width of the plot.
    #[must_use]
    pub fn hline(mut self, mut hline: HLine) -> Self {
        if hline.stroke.color == Color32::TRANSPARENT {
            hline.stroke.color = self.auto_color();
        }
        self.items.push(Box::new(hline));
        self
    }

    /// Add a vertical line.
    /// Can be useful e.g. to show min/max bounds or similar.
    /// Always fills the full height of the plot.
    #[must_use]
    pub fn vline(mut self, mut vline: VLine) -> Self {
        if vline.stroke.color == Color32::TRANSPARENT {
            vline.stroke.color = self.auto_color();
        }
        self.items.push(Box::new(vline));
        self
    }

    #[must_use]
    pub fn on_event(mut self, f: OnEvent) -> Self {
        self.on_event = Some(f);
        self
    }
}

impl<OnEvent> egui::Widget for DragPlot<OnEvent>
where
    OnEvent: FnMut(PlotMsg),
{
    fn ui(self, ui: &mut Ui) -> Response {
        let Self {
            id,
            mut items,
            min_auto_bounds,
            min_size,
            width,
            height,
            data_aspect,
            view_aspect,
            legend_config,
            next_auto_color_idx: _,
            allow_zoom,
            allow_drag,
            margin_fraction,
            selected: _,
            on_event,
            mut lines,
            axis_names,
        } = self;
        let plot_id = ui.make_persistent_id(id);
        let memory = ui
            .memory()
            .id_data
            .get_mut_or_insert_with(plot_id, || PlotMemory {
                bounds: min_auto_bounds,
                auto_bounds: false,
                hovered_entry: None,
                hidden_items: Default::default(),
                min_auto_bounds,
            })
            .clone();

        let PlotMemory {
            mut bounds,
            mut auto_bounds,
            mut hovered_entry,
            mut hidden_items,
            min_auto_bounds,
        } = memory;

        // Determine the size of the plot in the UI
        let size = {
            let width = width
                .unwrap_or_else(|| {
                    if let (Some(height), Some(aspect)) = (height, view_aspect) {
                        height * aspect
                    } else {
                        ui.available_size_before_wrap().x
                    }
                })
                .at_least(min_size.x);

            let height = height
                .unwrap_or_else(|| {
                    if let Some(aspect) = view_aspect {
                        width / aspect
                    } else {
                        ui.available_size_before_wrap().y
                    }
                })
                .at_least(min_size.y);
            vec2(width, height)
        };

        let (rect, response) = ui.allocate_exact_size(size, Sense::click_and_drag());
        let plot_painter = ui.painter().sub_region(rect);

        plot_painter.add(epaint::RectShape {
            rect,
            corner_radius: 2.0,
            fill: ui.visuals().extreme_bg_color,
            stroke: ui.visuals().widgets.noninteractive.bg_stroke,
        });

        // Legend
        let legend = legend_config
            .and_then(|config| LegendWidget::try_new(rect, config, &items, &hidden_items));

        // Remove the deselected items.
        items.retain(|item| !hidden_items.contains(item.name()));
        lines.retain(|item| !hidden_items.contains(&item.name));

        // Highlight the hovered items.
        if let Some(hovered_name) = &hovered_entry {
            items
                .iter_mut()
                .filter(|entry| entry.name() == hovered_name)
                .for_each(|entry| entry.highlight());
            lines
                .iter_mut()
                .filter(|entry| &entry.name == hovered_name)
                .for_each(|entry| {
                    entry.highlight();
                });
        }
        // Move highlighted items to front.
        items.sort_by_key(|item| item.highlighted());
        lines.sort_by_key(|item| item.highlighted());

        // Set bounds automatically based on content.
        if auto_bounds || !bounds.is_valid() {
            bounds = min_auto_bounds;
            items
                .iter()
                .for_each(|item| bounds.merge(&item.get_bounds()));
            lines
                .iter()
                .for_each(|item| bounds.merge(&item.get_bounds()));
            bounds.add_relative_margin(margin_fraction);
        }
        // Make sure they are not empty.
        if !bounds.is_valid() {
            bounds = Bounds::new_symmetrical(1.0);
        }

        let mut transform = ScreenTransform::new(rect, bounds, false, false);
        // Enforce equal aspect ratio.
        if let Some(data_aspect) = data_aspect {
            transform.set_aspect(data_aspect as f64);
        }

        // Zooming
        if allow_zoom {
            if let Some(hover_pos) = response.hover_pos() {
                let zoom_factor = if data_aspect.is_some() {
                    Vec2::splat(ui.input().zoom_delta())
                } else {
                    ui.input().zoom_delta_2d()
                };
                if zoom_factor != Vec2::splat(1.0) {
                    transform.zoom(zoom_factor, hover_pos);
                    auto_bounds = false;
                }

                let scroll_delta = ui.input().scroll_delta;
                if scroll_delta != Vec2::ZERO {
                    transform.translate_bounds(-scroll_delta);
                    auto_bounds = false;
                }
            }
        }

        // Initialize values from functions.
        items
            .iter_mut()
            .for_each(|item| item.initialize(transform.bounds().range_x()));
        lines
            .iter_mut()
            .for_each(|line| line.initialize(transform.bounds().range_x()));

        let t_bounds = *transform.bounds();

        let prepared = DragPlotPrepared {
            items,
            lines,
            show_x: true,
            show_y: true,
            show_axes: [true, true],
            transform: transform.clone(),
            axis_names,
        };

        if let Some(mut f) = on_event {
            if let Some(pointer) = response.hover_pos() {
                if response.mouse_down(PointerButton::Primary) {
                    if let Some(idx) = prepared.find_clicked(pointer) {
                        f(PlotMsg::Clicked(idx));
                    }
                }
            }
            if allow_drag
                && response.dragged_by(PointerButton::Primary)
                && response.hover_pos().is_some()
            {
                let mut delta = response.drag_delta();
                delta.x *= transform.dvalue_dpos()[0] as f32;
                delta.y *= transform.dvalue_dpos()[1] as f32;
                f(PlotMsg::Drag(delta));
            }
        }

        prepared.ui(ui, &response);

        if let Some(mut legend) = legend {
            ui.add(&mut legend);
            hidden_items = legend.get_hidden_items();
            hovered_entry = legend.get_hovered_entry_name();
        }

        ui.memory().id_data.insert(
            plot_id,
            PlotMemory {
                bounds: t_bounds,
                auto_bounds,
                hovered_entry,
                hidden_items,
                min_auto_bounds,
            },
        );
        response.on_hover_cursor(CursorIcon::Crosshair)
    }
}

pub trait PointerExt {
    fn mouse_down(&self, pointer: PointerButton) -> bool;
}

impl PointerExt for Response {
    fn mouse_down(&self, p: PointerButton) -> bool {
        let pointer = &self.ctx.input().pointer;
        match p {
            PointerButton::Primary => pointer.primary_down(),
            PointerButton::Secondary => pointer.secondary_down(),
            PointerButton::Middle => pointer.middle_down(),
        }
    }
}
