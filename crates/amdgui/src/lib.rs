use std::path::PathBuf;
use std::sync::Arc;

pub use app::AmdGui;
use parking_lot::Mutex;
use tokio::sync::mpsc::UnboundedReceiver;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::EnvFilter;

pub mod app;
pub mod backend;
pub mod items;
pub mod transform;
pub mod widgets;
pub use {egui, image, parking_lot};

#[derive(Debug, gumdrop::Options)]
struct Opts {
    config: Option<PathBuf>,
}

pub async fn start_app<F>(f: F)
where
    F: Fn(Arc<Mutex<AmdGui>>, UnboundedReceiver<bool>),
{
    let config_path = gumdrop::parse_args_default_or_exit::<Opts>()
        .config
        .unwrap_or_else(|| PathBuf::new().join(amdgpu_config::fan::DEFAULT_FAN_CONFIG_PATH));
    let config_path = config_path.to_str().expect("Config path must exists");

    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "DEBUG");
    }
    let config = Arc::new(Mutex::new(
        amdgpu_config::fan::load_config(config_path).expect("No FAN config"),
    ));
    tracing_subscriber::fmt::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_max_level({
            let level = config
                .lock()
                .log_level()
                .as_str()
                .parse::<LevelFilter>()
                .unwrap();
            if cfg!(debug_assertions) {
                eprintln!("log level is: {level:?}");
            }
            level
        })
        .init();
    let amd_gui = Arc::new(Mutex::new(AmdGui::new_with_config(config)));

    let receiver = schedule_tick(amd_gui.clone());

    f(amd_gui, receiver);
}

fn schedule_tick(amd_gui: Arc<Mutex<AmdGui>>) -> UnboundedReceiver<bool> {
    let (sender, receiver) = tokio::sync::mpsc::unbounded_channel();
    tokio::spawn(async move {
        let sender = sender;
        loop {
            amd_gui.lock().tick();
            if let Err(e) = sender.send(true) {
                tracing::error!("Failed to propagate tick update. {:?}", e);
            }
            tokio::time::sleep(tokio::time::Duration::from_millis(166)).await;
        }
    });
    receiver
}
