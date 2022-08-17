use std::ops::RangeInclusive;

use egui::Ui;
use epaint::{Color32, Shape};

use crate::items::Values;
use crate::transform::{Bounds, ScreenTransform};

/// Trait shared by things that can be drawn in the plot.
pub trait PlotItem {
    fn get_shapes(&self, ui: &mut Ui, transform: &ScreenTransform, shapes: &mut Vec<Shape>);
    fn initialize(&mut self, x_range: RangeInclusive<f64>);
    fn name(&self) -> &str;
    fn color(&self) -> Color32;
    fn highlight(&mut self);
    fn highlighted(&self) -> bool;
    fn values(&self) -> Option<&Values>;
    fn get_bounds(&self) -> Bounds;
}
