use std::sync::Arc;

use amdguid::parking_lot::Mutex;
use amdguid::AmdGui;
use tokio::sync::mpsc::UnboundedReceiver;

pub fn run_app(amd_gui: Arc<Mutex<AmdGui>>, _receiver: UnboundedReceiver<bool>) {
    eframe::run_native(
        "Amd GPU Client",
        eframe::NativeOptions {
            ..Default::default()
        },
        Box::new(|_cc| Ok(Box::new(MyApp { amd_gui }))),
    )
    .expect("AMD GUID failed");
}

struct MyApp {
    amd_gui: Arc<Mutex<AmdGui>>,
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        amdguid::backend::create_ui(self.amd_gui.clone(), ctx);
        ctx.request_repaint_after(std::time::Duration::from_millis(1000 / 60));
    }
}
