use crate::widgets::{ChangeFanSettings, CoolingPerformance};
use amdgpu::helper_cmd::Pid;
use egui::{CtxRef, Ui};
use epi::Frame;
use parking_lot::Mutex;
use std::collections::HashMap;
use std::sync::Arc;

pub enum ChangeState {
    New,
    Reloading,
    Success,
    Failure(String),
}

impl Default for ChangeState {
    fn default() -> Self {
        ChangeState::New
    }
}

pub struct FanService {
    pub pid: Pid,
    pub reload: ChangeState,
}

impl FanService {
    pub fn new(pid: Pid) -> FanService {
        Self {
            pid,
            reload: Default::default(),
        }
    }
}

pub struct FanServices(pub Vec<FanService>);

impl FanServices {
    pub fn list_changed(&self, other: &[Pid]) -> bool {
        if self.0.len() != other.len() {
            return true;
        }
        let c = self
            .0
            .iter()
            .fold(HashMap::with_capacity(other.len()), |mut h, service| {
                h.insert(service.pid.0, true);
                h
            });
        !other.iter().all(|s| c.contains_key(&s.0))
    }
}

impl From<Vec<Pid>> for FanServices {
    fn from(v: Vec<Pid>) -> Self {
        Self(v.into_iter().map(FanService::new).collect())
    }
}

#[derive(Debug, Copy, Clone)]
pub enum Page {
    Config,
    Monitoring,
    Settings,
}

impl Default for Page {
    fn default() -> Self {
        Self::Config
    }
}

pub type FanConfig = Arc<Mutex<amdgpu_config::fan::Config>>;

static RELOAD_PID_LIST_DELAY: u8 = 18;

pub struct StatefulConfig {
    pub config: FanConfig,
    pub state: ChangeState,
}

pub struct AmdGui {
    pub page: Page,
    pid_files: FanServices,
    cooling_performance: CoolingPerformance,
    change_fan_settings: ChangeFanSettings,
    config: StatefulConfig,
    reload_pid_list_delay: u8,
}

impl epi::App for AmdGui {
    fn update(&mut self, _ctx: &CtxRef, _frame: &mut Frame<'_>) {}

    fn name(&self) -> &str {
        "AMD GUI"
    }
}

impl AmdGui {
    pub fn new_with_config(config: FanConfig) -> Self {
        Self {
            page: Default::default(),
            pid_files: FanServices::from(vec![]),
            cooling_performance: CoolingPerformance::new(100, config.clone()),
            change_fan_settings: ChangeFanSettings::new(config.clone()),
            config: StatefulConfig {
                config,
                state: ChangeState::New,
            },
            reload_pid_list_delay: RELOAD_PID_LIST_DELAY,
        }
    }

    pub fn ui(&mut self, ui: &mut Ui) {
        match self.page {
            Page::Config => {
                self.change_fan_settings
                    .draw(ui, &mut self.pid_files, &mut self.config);
            }
            Page::Monitoring => {
                self.cooling_performance.draw(ui, &self.pid_files);
            }
            Page::Settings => {}
        }
    }

    pub fn tick(&mut self) {
        self.cooling_performance.tick();
        if self.pid_files.0.is_empty() || self.reload_pid_list_delay.checked_sub(1).is_none() {
            self.reload_pid_list_delay = RELOAD_PID_LIST_DELAY;
            match amdgpu::helper_cmd::send_command(amdgpu::helper_cmd::Command::FanServices) {
                Ok(amdgpu::helper_cmd::Response::Services(services))
                    if self.pid_files.list_changed(&services) =>
                {
                    self.pid_files = FanServices::from(services);
                }
                Ok(amdgpu::helper_cmd::Response::Services(_services)) => {
                    // SKIP
                }
                Ok(res) => {
                    log::warn!("Unexpected response {:?} while loading fan services", res);
                }
                Err(e) => {
                    log::warn!("Failed to load amd fan services pid list. {:?}", e);
                }
            }
        } else {
            self.reload_pid_list_delay -= 1;
        }
    }
}
