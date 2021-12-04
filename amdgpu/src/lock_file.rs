//! Create lock file and prevent running 2 identical services.
//! NOTE: For 2 amdfand services you may just give 2 different names

use crate::{IoFailure, PidLockError};
use std::path::Path;

pub struct PidLock(pidlock::Pidlock);

impl PidLock {
    pub fn new<P: AsRef<Path>>(
        sub_dir: P,
        name: String,
    ) -> std::result::Result<Self, crate::error::AmdGpuError> {
        let pid_dir_path = std::path::Path::new("/var").join("lib").join(sub_dir);
        let pid_path = {
            std::fs::create_dir_all(&pid_dir_path).map_err(|io| IoFailure {
                io,
                path: pid_dir_path.clone(),
            })?;
            pid_dir_path
                .join(format!("{}.pid", name))
                .to_str()
                .map(String::from)
                .unwrap()
        };
        let pid_file = pidlock::Pidlock::new(&pid_path);
        Ok(Self(pid_file))
    }

    /// Create new lock file. File will be created if:
    /// * pid file does not exists
    /// * pid file exists but process is dead
    pub fn acquire(&mut self) -> Result<(), crate::error::AmdGpuError> {
        self.0.acquire().map_err(PidLockError::from)?;
        Ok(())
    }

    /// Remove lock file
    /// Remove lock file
    pub fn release(&mut self) -> Result<(), crate::error::AmdGpuError> {
        self.0.release().map_err(PidLockError::from)?;
        Ok(())
    }
}
