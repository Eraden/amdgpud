use std::collections::vec_deque::VecDeque;

use amdmond_lib::AmdMon;
use egui::{Color32, Ui};

use crate::app::{FanConfig, FanServices};

pub struct CoolingPerformance {
    capacity: usize,
    data: VecDeque<f64>,
    amd_mon: Option<AmdMon>,
}

impl CoolingPerformance {
    #[allow(clippy::explicit_auto_deref)]
    pub fn new(capacity: usize, fan_config: FanConfig) -> Self {
        let amd_mon = amdgpu::utils::hw_mons(true)
            .map_err(|e| {
                tracing::error!("Failed to open hw mon: {e}");
                e
            })
            .ok()
            .and_then(|mut v| {
                if v.is_empty() {
                    None
                } else {
                    Some(v.remove(0))
                }
            })
            .map(|c| AmdMon::wrap(c, &*fan_config.lock()));

        Self {
            capacity,
            data: VecDeque::with_capacity(capacity),
            amd_mon,
        }
    }

    pub fn tick(&mut self) {
        if let Some(temp) = self
            .amd_mon
            .as_ref()
            .and_then(|mon| mon.gpu_temp_of(0))
            .and_then(|(_, value)| value.ok())
        {
            self.push(temp);
        }
    }

    pub fn draw(&self, ui: &mut Ui, pid_files: &FanServices) {
        use egui_plot::*;

        let current = self.data.iter().last().copied().unwrap_or_default();

        let iter = self
            .data
            .iter()
            .enumerate()
            .map(|(i, v)| [i as f64, *v])
            .collect();

        let curve = Line::new(PlotPoints::new(iter)).color(Color32::BLUE);
        let zero = HLine::new(0.0).color(Color32::from_white_alpha(0));
        let optimal = HLine::new(45.0).name("Optimal").color(Color32::LIGHT_BLUE);
        let target = HLine::new(80.0)
            .name("Overheating")
            .color(Color32::DARK_RED);

        ui.label("Temperature");
        Plot::new("cooling performance")
            .allow_drag(false)
            .allow_zoom(false)
            .height(600.0)
            .show(ui, |plot_ui| {
                //
                plot_ui.line(curve);
                plot_ui.hline(zero);
                plot_ui.hline(optimal);
                plot_ui.hline(target);
                // plot_ui.legend(Legend::default());
            });

        // ui.add(plot);
        ui.horizontal(|ui| {
            ui.label("Current temperature");
            ui.label(format!("{:<3.2}°C", current));
        });
        ui.label("Working services");
        if pid_files.0.is_empty() {
            ui.label("  There's no working services");
        } else {
            pid_files.0.iter().for_each(|service| {
                ui.label(format!("  {}", service.pid.0));
            });
        }
    }

    pub fn push(&mut self, v: f64) {
        if self.data.len() >= self.capacity {
            self.data.pop_front();
        }
        self.data.push_back(v);
    }
}
