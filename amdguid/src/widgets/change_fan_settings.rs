use amdgpu::helper_cmd::Command;
use amdgpu_config::fan::MatrixPoint;
use egui::{emath, pos2, Layout, PointerButton, Ui};
use epaint::Color32;

use crate::app::{ChangeState, FanConfig, FanServices, StatefulConfig};
use crate::widgets::drag_plot::PlotMsg;
use crate::widgets::reload_section::ReloadSection;
use crate::{widgets, widgets::ConfigFile};

pub struct ChangeFanSettings {
    config: FanConfig,
    selected: Option<usize>,
    matrix: Vec<MatrixPoint>,
}

impl ChangeFanSettings {
    pub fn new(config: FanConfig) -> Self {
        let matrix = config.lock().speed_matrix().to_vec();

        Self {
            matrix,
            config,
            selected: None,
        }
    }

    pub fn select(&mut self, idx: usize) {
        self.selected = Some(idx);
    }

    pub fn deselect(&mut self) {
        self.selected = None;
    }

    pub fn draw(&mut self, ui: &mut Ui, pid_files: &mut FanServices, state: &mut StatefulConfig) {
        let available = ui.available_rect_before_wrap();
        ui.horizontal_top(|ui| {
            ui.child_ui(
                emath::Rect {
                    min: available.min,
                    max: pos2(available.width() / 2.0, available.height()),
                },
                Layout::left_to_right(),
            )
            .vertical(|ui| {
                egui::ScrollArea::vertical()
                    .enable_scrolling(true)
                    .id_source("plot-and-reload")
                    .show(ui, |ui| {
                        ui.add({
                            let curve = {
                                let config = self.config.lock();
                                let iter = config
                                    .speed_matrix()
                                    .iter()
                                    .map(|v| crate::items::Value::new(v.temp, v.speed));
                                crate::items::Line::new(crate::items::Values::from_values_iter(
                                    iter,
                                ))
                                .color(Color32::BLUE)
                            };
                            widgets::drag_plot::DragPlot::new("change fan settings")
                                .height(600.0)
                                .width(available.width() / 2.0)
                                .selected(self.selected)
                                .allow_drag(true)
                                .allow_zoom(false)
                                .line(curve)
                                .y_axis_name("Speed")
                                .x_axis_name("Temperature")
                                .hline(crate::items::HLine::new(0.0).color(Color32::BLACK))
                                .hline(crate::items::HLine::new(100.0).color(Color32::BLACK))
                                .vline(crate::items::VLine::new(0.0).color(Color32::BLACK))
                                .vline(crate::items::VLine::new(100.0).color(Color32::BLACK))
                                .on_event(|msg| match msg {
                                    PlotMsg::Clicked(idx) => {
                                        self.selected = Some(idx);
                                    }
                                    PlotMsg::Drag(delta) => {
                                        if let Some(idx) = self.selected {
                                            let mut config = self.config.lock();
                                            let min = idx
                                                .checked_sub(1)
                                                .and_then(|i| config.speed_matrix().get(i).copied())
                                                .unwrap_or(MatrixPoint::MIN);
                                            let max = idx
                                                .checked_add(1)
                                                .and_then(|i| config.speed_matrix().get(i).copied())
                                                .unwrap_or(MatrixPoint::MAX);
                                            let current = config.speed_matrix_mut().get_mut(idx);
                                            if let Some((cache, current)) =
                                                self.matrix.get_mut(idx).zip(current.as_deref())
                                            {
                                                cache.speed = current.speed;
                                                cache.temp = current.temp;
                                            }

                                            if let Some(point) = current {
                                                point.speed = (point.speed + delta.y as f64)
                                                    .max(min.speed)
                                                    .min(max.speed);
                                                point.temp = (point.temp + delta.x as f64)
                                                    .max(min.temp)
                                                    .min(max.temp);
                                            }
                                        }
                                    }
                                })
                                .legend(widgets::legend::Legend::default())
                        });
                        ui.separator();
                        Self::save_button(self.config.clone(), state, ui);
                        ui.add(ReloadSection::new(pid_files));
                    });
            });

            ui.child_ui(
                emath::Rect {
                    min: pos2(available.width() / 2.0 + 20.0, available.min.y),
                    max: available.max,
                },
                Layout::left_to_right(),
            )
            .vertical(|ui| {
                ui.add(ConfigFile::new(self.config.clone(), &mut self.matrix));
            });
        });
    }

    fn save_button(config: FanConfig, state: &mut StatefulConfig, ui: &mut Ui) {
        ui.horizontal(|ui| {
            if ui.button("Save").clicked_by(PointerButton::Primary) {
                Self::save_config(config, state);
            }
            match &state.state {
                ChangeState::New => {}
                ChangeState::Reloading => {
                    ui.label("Saving...");
                }
                ChangeState::Success => {
                    ui.add(egui::Label::new("Saved").text_color(Color32::GREEN));
                }
                ChangeState::Failure(msg) => {
                    ui.add(egui::Label::new(format!("Failure. {}", msg)).text_color(Color32::RED));
                }
            }
        });
    }

    fn save_config(config: FanConfig, state: &mut StatefulConfig) {
        state.state = ChangeState::Reloading;

        let config = config.lock();
        let c: &amdgpu_config::fan::Config = &*config;
        let content = match toml::to_string(c) {
            Err(e) => {
                log::error!("Config file serialization failed. {:?}", e);
                return;
            }
            Ok(content) => content,
        };
        let command = Command::SaveFanConfig {
            path: String::from(config.path()),
            content,
        };
        match amdgpu::helper_cmd::send_command(command) {
            Ok(amdgpu::helper_cmd::Response::ConfigFileSaveFailed(msg)) => {
                state.state = ChangeState::Failure(msg);
            }
            Ok(amdgpu::helper_cmd::Response::ConfigFileSaved) => {
                state.state = ChangeState::Success;
            }
            _ => {}
        }
    }
}
