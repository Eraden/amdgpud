use app::AmdGui;
use tokio::sync::mpsc::UnboundedReceiver;

mod app;
mod backend;
pub mod items;
pub mod transform;
pub mod widgets;

#[tokio::main]
async fn main() {
    use std::sync::Arc;

    use parking_lot::Mutex;

    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "DEBUG");
    }
    pretty_env_logger::init();
    let config = Arc::new(Mutex::new(
        amdgpu_config::fan::load_config(amdgpu_config::fan::DEFAULT_FAN_CONFIG_PATH)
            .expect("No FAN config"),
    ));
    let amd_gui = Arc::new(Mutex::new(AmdGui::new_with_config(config)));

    let receiver = schedule_tick(amd_gui.clone());

    backend::run_app(amd_gui, receiver);
}

fn schedule_tick(amd_gui: std::sync::Arc<parking_lot::Mutex<AmdGui>>) -> UnboundedReceiver<bool> {
    let (sender, receiver) = tokio::sync::mpsc::unbounded_channel();
    tokio::spawn(async move {
        let sender = sender;
        loop {
            amd_gui.lock().tick();
            if let Err(e) = sender.send(true) {
                log::error!("Failed to propagate tick update. {:?}", e);
            }
            tokio::time::sleep(tokio::time::Duration::from_millis(166)).await;
        }
    });
    receiver
}
