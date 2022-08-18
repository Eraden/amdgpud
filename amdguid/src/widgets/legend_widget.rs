use std::collections::BTreeMap;

use egui::style::Margin;
use egui::{Align, Direction, Frame, Layout, Rect, Response, Ui, Widget};
use epaint::ahash::AHashSet;
use epaint::Color32;

use crate::items::PlotItem;
use crate::widgets::legend::{Corner, Legend, LegendEntry};

#[derive(Clone)]
pub struct LegendWidget {
    rect: Rect,
    entries: BTreeMap<String, LegendEntry>,
    config: Legend,
}

impl LegendWidget {
    /// Create a new legend from items, the names of items that are hidden and
    /// the style of the text. Returns `None` if the legend has no entries.
    pub fn try_new(
        rect: Rect,
        config: Legend,
        items: &[Box<dyn PlotItem>],
        hidden_items: &AHashSet<String>,
    ) -> Option<Self> {
        let mut entries: BTreeMap<String, LegendEntry> = BTreeMap::new();
        items
            .iter()
            .filter(|item| !item.name().is_empty())
            .for_each(|item| {
                entries
                    .entry(item.name().to_string())
                    .and_modify(|entry| {
                        if entry.color != item.color() {
                            // Multiple items with different colors
                            entry.color = Color32::TRANSPARENT;
                        }
                    })
                    .or_insert_with(|| {
                        let color = item.color();
                        let checked = !hidden_items.contains(item.name());
                        LegendEntry::new(color, checked)
                    });
            });
        (!entries.is_empty()).then_some(Self {
            rect,
            entries,
            config,
        })
    }

    // Get the names of the hidden items.
    pub fn get_hidden_items(&self) -> AHashSet<String> {
        self.entries
            .iter()
            .filter(|(_, entry)| !entry.checked)
            .map(|(name, _)| name.clone())
            .collect()
    }

    // Get the name of the hovered items.
    pub fn get_hovered_entry_name(&self) -> Option<String> {
        self.entries
            .iter()
            .find(|(_, entry)| entry.hovered)
            .map(|(name, _)| name.to_string())
    }
}

impl Widget for &mut LegendWidget {
    fn ui(self, ui: &mut Ui) -> Response {
        let LegendWidget {
            rect,
            entries,
            config,
        } = self;

        let main_dir = match config.position {
            Corner::LeftTop | Corner::RightTop => Direction::TopDown,
            Corner::LeftBottom | Corner::RightBottom => Direction::BottomUp,
        };
        let cross_align = match config.position {
            Corner::LeftTop | Corner::LeftBottom => Align::LEFT,
            Corner::RightTop | Corner::RightBottom => Align::RIGHT,
        };
        let layout = Layout::from_main_dir_and_cross_align(main_dir, cross_align);
        let legend_pad = 4.0;
        let legend_rect = rect.shrink(legend_pad);
        let mut legend_ui = ui.child_ui(legend_rect, layout);
        legend_ui
            .scope(|ui| {
                // ui.style_mut().body_text_style = config.text_style;
                let background_frame = Frame {
                    inner_margin: Margin::symmetric(8.0, 4.0),
                    outer_margin: Default::default(),
                    rounding: ui.style().visuals.window_rounding,
                    shadow: epaint::Shadow::default(),
                    fill: ui.style().visuals.extreme_bg_color,
                    stroke: ui.style().visuals.window_stroke(),
                }
                .multiply_with_opacity(config.background_alpha);
                background_frame
                    .show(ui, |ui| {
                        entries
                            .iter_mut()
                            .map(|(name, entry)| entry.ui(ui, name.clone()))
                            .reduce(|r1, r2| r1.union(r2))
                            .unwrap()
                    })
                    .inner
            })
            .inner
    }
}
