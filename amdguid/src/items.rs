//! Contains items that can be added to a plot.

use std::ops::RangeInclusive;

use egui::Pos2;
use epaint::{Color32, Shape, Stroke};

pub use arrows::*;
pub use h_line::*;
pub use line::*;
pub use marker_shape::*;
pub use plot_image::*;
pub use plot_item::*;
pub use points::*;
pub use polygons::*;
pub use text::*;
pub use v_line::*;
pub use value::Value;
pub use values::Values;

mod arrows;
mod h_line;
mod line;
mod marker_shape;
mod plot_image;
mod plot_item;
mod points;
mod polygons;
mod text;
mod v_line;
mod value;
mod values;

const DEFAULT_FILL_ALPHA: f32 = 0.05;

// ----------------------------------------------------------------------------

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum LineStyle {
    Solid,
    Dotted { spacing: f32 },
    Dashed { length: f32 },
}

impl LineStyle {
    pub fn dashed_loose() -> Self {
        Self::Dashed { length: 10.0 }
    }

    pub fn dashed_dense() -> Self {
        Self::Dashed { length: 5.0 }
    }

    pub fn dotted_loose() -> Self {
        Self::Dotted { spacing: 10.0 }
    }

    pub fn dotted_dense() -> Self {
        Self::Dotted { spacing: 5.0 }
    }

    fn style_line(
        &self,
        line: Vec<Pos2>,
        mut stroke: Stroke,
        highlight: bool,
        shapes: &mut Vec<Shape>,
    ) {
        match line.len() {
            0 => {}
            1 => {
                let mut radius = stroke.width / 2.0;
                if highlight {
                    radius *= 2f32.sqrt();
                }
                shapes.push(Shape::circle_filled(line[0], radius, stroke.color));
            }
            _ => {
                match self {
                    LineStyle::Solid => {
                        if highlight {
                            stroke.width *= 2.0;
                        }
                        for point in line.iter() {
                            shapes.push(Shape::circle_filled(
                                *point,
                                stroke.width * 3.0,
                                Color32::DARK_BLUE,
                            ));
                        }
                        shapes.push(Shape::line(line, stroke));
                    }
                    LineStyle::Dotted { spacing } => {
                        let mut radius = stroke.width;
                        if highlight {
                            radius *= 2f32.sqrt();
                        }
                        shapes.extend(Shape::dotted_line(&line, stroke.color, *spacing, radius));
                    }
                    LineStyle::Dashed { length } => {
                        if highlight {
                            stroke.width *= 2.0;
                        }
                        let golden_ratio = (5.0_f32.sqrt() - 1.0) / 2.0; // 0.61803398875
                        shapes.extend(Shape::dashed_line(
                            &line,
                            stroke,
                            *length,
                            length * golden_ratio,
                        ));
                    }
                }
            }
        }
    }
}

impl ToString for LineStyle {
    fn to_string(&self) -> String {
        match self {
            LineStyle::Solid => "Solid".into(),
            LineStyle::Dotted { spacing } => format!("Dotted{}Px", spacing),
            LineStyle::Dashed { length } => format!("Dashed{}Px", length),
        }
    }
}

// ----------------------------------------------------------------------------

// ----------------------------------------------------------------------------

/// Describes a function y = f(x) with an optional range for x and a number of points.
pub struct ExplicitGenerator {
    function: Box<dyn Fn(f64) -> f64>,
    x_range: RangeInclusive<f64>,
    points: usize,
}

// ----------------------------------------------------------------------------

/// Returns the x-coordinate of a possible intersection between a line segment from `p1` to `p2` and
/// a horizontal line at the given y-coordinate.
#[inline(always)]
pub fn y_intersection(p1: &Pos2, p2: &Pos2, y: f32) -> Option<f32> {
    ((p1.y > y && p2.y < y) || (p1.y < y && p2.y > y))
        .then(|| ((y * (p1.x - p2.x)) - (p1.x * p2.y - p1.y * p2.x)) / (p1.y - p2.y))
}
