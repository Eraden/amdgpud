use crate::{log_file, watch};

#[derive(gumdrop::Options)]
pub enum Command {
    Watch(watch::Watch),
    LogFile(log_file::LogFile),
}

impl Default for Command {
    fn default() -> Self {
        Self::Watch(watch::Watch::default())
    }
}
