use egui::{emath, pos2, remap_clamp, vec2, Align2, Layout, NumExt, Pos2, Response, Ui};
use epaint::{Color32, Rgba, Shape, Stroke, TextStyle};

use crate::items::Value;
use crate::items::{Line, PlotItem};
use crate::transform::ScreenTransform;

pub struct DragPlotPrepared {
    pub items: Vec<Box<dyn PlotItem>>,
    pub lines: Vec<Line>,
    pub show_x: bool,
    pub show_y: bool,
    pub show_axes: [bool; 2],
    pub transform: ScreenTransform,
    pub axis_names: [String; 2],
}

impl DragPlotPrepared {
    pub fn ui(self, ui: &mut Ui, response: &Response) {
        let mut shapes = Vec::new();

        for d in 0..2 {
            if self.show_axes[d] {
                self.paint_axis(ui, d, &mut shapes);
            }
        }

        let transform = &self.transform;

        let mut plot_ui = ui.child_ui(*transform.frame(), Layout::default());
        plot_ui.set_clip_rect(*transform.frame());
        for item in &self.items {
            item.get_shapes(&mut plot_ui, transform, &mut shapes);
        }
        for item in &self.lines {
            item.get_shapes(&mut plot_ui, transform, &mut shapes);
        }

        if let Some(pointer) = response.hover_pos() {
            self.hover(ui, pointer, &mut shapes);
        }

        ui.painter().sub_region(*transform.frame()).extend(shapes);
    }

    fn paint_axis(&self, ui: &Ui, axis: usize, shapes: &mut Vec<Shape>) {
        let Self { transform, .. } = self;

        let bounds = transform.bounds();
        let text_style = TextStyle::Body;

        let base: i64 = 10;
        let base_f = base as f64;

        let min_line_spacing_in_points = 6.0;
        let step_size = transform.dvalue_dpos()[axis] * min_line_spacing_in_points;
        let step_size = base_f.powi(step_size.abs().log(base_f).ceil() as i32);

        let step_size_in_points = (transform.dpos_dvalue()[axis] * step_size).abs() as f32;

        // Where on the cross-dimension to show the label values
        let value_cross = 0.0_f64.clamp(bounds.min[1 - axis], bounds.max[1 - axis]);

        for i in 0.. {
            let value_main = step_size * (bounds.min[axis] / step_size + i as f64).floor();
            if value_main > bounds.max[axis] {
                break;
            }

            let value = if axis == 0 {
                Value::new(value_main, value_cross)
            } else {
                Value::new(value_cross, value_main)
            };
            let pos_in_gui = transform.position_from_value(&value);

            let n = (value_main / step_size).round() as i64;
            let spacing_in_points = if n % (base * base) == 0 {
                step_size_in_points * (base_f * base_f) as f32 // think line (multiple of 100)
            } else if n % base == 0 {
                step_size_in_points * base_f as f32 // medium line (multiple of 10)
            } else {
                step_size_in_points // thin line
            };

            let line_alpha = remap_clamp(
                spacing_in_points,
                (min_line_spacing_in_points as f32)..=300.0,
                0.0..=0.15,
            );

            if line_alpha > 0.0 {
                let line_color = color_from_alpha(ui, line_alpha);

                let mut p0 = pos_in_gui;
                let mut p1 = pos_in_gui;
                p0[1 - axis] = transform.frame().min[1 - axis];
                p1[1 - axis] = transform.frame().max[1 - axis];
                shapes.push(Shape::line_segment([p0, p1], Stroke::new(1.0, line_color)));
            }

            let text_alpha = remap_clamp(spacing_in_points, 40.0..=150.0, 0.0..=0.4);

            if text_alpha > 0.0 {
                let color = color_from_alpha(ui, text_alpha);
                let text = emath::round_to_decimals(value_main, 5).to_string(); // hack

                let galley = ui.painter().layout_no_wrap(text, text_style, color);

                let mut text_pos = pos_in_gui + vec2(1.0, -galley.size().y);

                // Make sure we see the labels, even if the axis is off-screen:
                text_pos[1 - axis] = text_pos[1 - axis]
                    .at_most(transform.frame().max[1 - axis] - galley.size()[1 - axis] - 2.0)
                    .at_least(transform.frame().min[1 - axis] + 1.0);

                shapes.push(Shape::galley(text_pos, galley));
            }
        }

        fn color_from_alpha(ui: &Ui, alpha: f32) -> Color32 {
            if ui.visuals().dark_mode {
                Rgba::from_white_alpha(alpha).into()
            } else {
                Rgba::from_black_alpha((4.0 * alpha).at_most(1.0)).into()
            }
        }
    }

    pub fn find_clicked(&self, pointer: Pos2) -> Option<usize> {
        let Self {
            transform, lines, ..
        } = self;
        let interact_radius: f32 = 16.0;
        let mut closest_value = None;
        let mut closest_dist_sq = interact_radius.powi(2);
        for item in lines {
            if let Some(values) = item.values() {
                for (idx, value) in values.values.iter().enumerate() {
                    let pos = transform.position_from_value(value);
                    let dist_sq = pointer.distance_sq(pos);
                    if dist_sq < closest_dist_sq {
                        closest_dist_sq = dist_sq;
                        closest_value = Some(idx);
                    }
                }
            }
        }

        closest_value
    }

    fn hover(&self, ui: &Ui, pointer: Pos2, shapes: &mut Vec<Shape>) {
        let Self {
            transform,
            show_x,
            show_y,
            items,
            lines,
            ..
        } = self;

        if !show_x && !show_y {
            return;
        }

        let interact_radius: f32 = 16.0;
        let mut closest_value = None;
        let mut closest_item = None;
        let mut closest_dist_sq = interact_radius.powi(2);
        for item in items {
            if let Some(values) = item.values() {
                for value in &values.values {
                    let pos = transform.position_from_value(value);
                    let dist_sq = pointer.distance_sq(pos);
                    if dist_sq < closest_dist_sq {
                        closest_dist_sq = dist_sq;
                        closest_value = Some(value);
                        closest_item = Some(item.name());
                    }
                }
            }
        }
        for item in lines {
            if let Some(values) = item.values() {
                for value in &values.values {
                    let pos = transform.position_from_value(value);
                    let dist_sq = pointer.distance_sq(pos);
                    if dist_sq < closest_dist_sq {
                        closest_dist_sq = dist_sq;
                        closest_value = Some(value);
                        closest_item = Some(&item.name);
                    }
                }
            }
        }

        let mut prefix = String::new();
        if let Some(name) = closest_item {
            if !name.is_empty() {
                prefix = format!("{}\n", name);
            }
        }

        let line_color = if ui.visuals().dark_mode {
            Color32::from_gray(100).additive()
        } else {
            Color32::from_black_alpha(180)
        };

        let value = if let Some(value) = closest_value {
            let position = transform.position_from_value(value);
            shapes.push(Shape::circle_filled(position, 3.0, line_color));
            *value
        } else {
            transform.value_from_position(pointer)
        };
        let pointer = transform.position_from_value(&value);

        let rect = transform.frame();

        if *show_x {
            // vertical line
            shapes.push(Shape::line_segment(
                [pos2(pointer.x, rect.top()), pos2(pointer.x, rect.bottom())],
                (1.0, line_color),
            ));
        }
        if *show_y {
            // horizontal line
            shapes.push(Shape::line_segment(
                [pos2(rect.left(), pointer.y), pos2(rect.right(), pointer.y)],
                (1.0, line_color),
            ));
        }

        let text = {
            let scale = transform.dvalue_dpos();
            let x_decimals = ((-scale[0].abs().log10()).ceil().at_least(0.0) as usize).at_most(6);
            let y_decimals = ((-scale[1].abs().log10()).ceil().at_least(0.0) as usize).at_most(6);
            let [x_name, y_name] = &self.axis_names;
            if *show_x && *show_y {
                format!(
                    "{}{} = {:.*}\n{} = {:.*}",
                    prefix, x_name, x_decimals, value.x, y_name, y_decimals, value.y
                )
            } else if *show_x {
                format!("{}{} = {:.*}", prefix, x_name, x_decimals, value.x)
            } else if *show_y {
                format!("{}{} = {:.*}", prefix, y_name, y_decimals, value.y)
            } else {
                unreachable!()
            }
        };

        shapes.push(Shape::text(
            ui.fonts(),
            pointer + vec2(3.0, -2.0),
            Align2::LEFT_BOTTOM,
            text,
            TextStyle::Body,
            ui.visuals().text_color(),
        ));
    }
}
