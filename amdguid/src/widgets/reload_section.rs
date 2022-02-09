use crate::app::{ChangeState, FanServices};
use amdgpu::helper_cmd::Command;
use egui::{PointerButton, Response, Sense, Ui};
use epaint::Color32;

pub struct ReloadSection<'l> {
    pub services: &'l mut FanServices,
}

impl<'l> egui::Widget for ReloadSection<'l> {
    fn ui(self, ui: &mut Ui) -> Response {
        ui.vertical(|ui| {
            ui.label("Reload config for service");

            self.services.0.iter_mut().for_each(|service| {
                ui.horizontal(|ui| {
                    ui.label(format!("PID {}", service.pid.0));
                    if ui.button("Reload").clicked_by(PointerButton::Primary) {
                        service.reload = ChangeState::Reloading;

                        match amdgpu::helper_cmd::send_command(Command::ReloadConfig {
                            pid: service.pid,
                        }) {
                            Ok(response) => {
                                service.reload = ChangeState::Success;
                                log::info!("{:?}", response)
                            }
                            Err(e) => {
                                service.reload = ChangeState::Failure(format!("{:?}", e));
                                log::error!("Failed to reload config. {:?}", e)
                            }
                        }
                    }
                    match &service.reload {
                        ChangeState::New => {}
                        ChangeState::Reloading => {
                            ui.label("Reloading...");
                        }
                        ChangeState::Success => {
                            ui.add(egui::Label::new("Reloaded").text_color(Color32::DARK_GREEN));
                        }
                        ChangeState::Failure(msg) => {
                            ui.add(
                                egui::Label::new(format!("Failure. {}", msg))
                                    .text_color(Color32::RED),
                            );
                        }
                    }
                });
            });
        });
        ui.allocate_response(ui.available_size(), Sense::click())
    }
}

impl<'l> ReloadSection<'l> {
    pub fn new(services: &'l mut FanServices) -> Self {
        Self { services }
    }
}
