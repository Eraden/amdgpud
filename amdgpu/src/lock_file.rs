//! Create lock file and prevent running 2 identical services.
//! NOTE: For 2 amdfand services you may just give 2 different names

use crate::helper_cmd::Pid;
use crate::IoFailure;
use nix::libc;
use std::path::Path;

#[derive(Debug, thiserror::Error)]
pub enum LockFileError {
    #[error("Failed to read {path}. {err:?}")]
    Unreadable { err: std::io::Error, path: String },
    #[error("Pid {pid:?} file system error. {err:?}")]
    Io { err: std::io::Error, pid: Pid },
    #[error("Pid {0:?} does not exists")]
    NotExists(Pid),
    #[error("Pid {pid:?} with name {name:?} already exists")]
    Conflict { name: String, pid: Pid },
    #[error("Can't parse Pid value. {0:?}")]
    MalformedPidFile(#[from] std::num::ParseIntError),
}

pub enum State {
    NotExists,
    Pending(Pid),
    Dead,
    New(Pid),
}

pub struct PidLock {
    name: String,
    pid_path: String,
}

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
        log::debug!("Creating pid lock for path {:?}", pid_path);
        Ok(Self { pid_path, name })
    }

    /// Create new lock file. File will be created if:
    /// * pid file does not exists
    /// * pid file exists but process is dead
    /// * old pid and current pid have different names (lock file exists after reboot and PID was taken by other process)
    pub fn acquire(&mut self) -> Result<(), crate::error::AmdGpuError> {
        log::debug!("PID LOCK acquiring {}", self.pid_path);
        let pid = self.process_pid();
        if let Some(old) = self.old_pid() {
            let old = old?;
            if !self.is_alive(old) {
                log::debug!("Old pid {:?} is dead, overriding...", old.0);

                self.enforce_pid_file(pid)?;
                return Ok(());
            }
            match self.process_name(old) {
                Err(LockFileError::NotExists(..)) => {
                    log::debug!(
                        "Old pid {:?} doesn't have process stat, overriding....",
                        old.0
                    );
                    self.enforce_pid_file(old)?;
                }
                Err(e) => {
                    log::debug!("Lock error {:?}", e);
                    return Err(e.into());
                }
                Ok(name) if name.ends_with(&format!("/{}", self.name)) => {
                    log::warn!("Conflicting {} and {} for process {}", old.0, pid.0, name);
                    return Err(LockFileError::Conflict { pid: old, name }.into());
                }
                Ok(name /*name isn't the same*/) => {
                    log::debug!(
                        "Old process {:?} and current process {:?} have different names, overriding....",
                        name, self.name
                    );
                    self.enforce_pid_file(old)?;
                }
            }
        } else {
            log::debug!("No collision detected");
            self.enforce_pid_file(pid)?;
        }
        Ok(())
    }

    /// Remove lock file
    pub fn release(&mut self) -> Result<(), crate::error::AmdGpuError> {
        if let Err(e) = std::fs::remove_file(&self.pid_path) {
            log::error!("Failed to release pid file {}. {:?}", self.pid_path, e);
        }
        Ok(())
    }

    /// Read old pid value from file
    fn old_pid(&self) -> Option<Result<Pid, LockFileError>> {
        match std::fs::read_to_string(&self.pid_path) {
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => None,
            Err(e) => Some(Err(LockFileError::Unreadable {
                path: self.pid_path.clone(),
                err: e,
            })),
            Ok(s) => match s.parse::<i32>() {
                Err(e) => Some(Err(LockFileError::MalformedPidFile(e))),
                Ok(pid) => Some(Ok(Pid(pid))),
            },
        }
    }

    /// Check if PID is alive
    fn is_alive(&self, pid: Pid) -> bool {
        unsafe {
            let result = libc::kill(pid.0, 0);
            result == 0
        }
    }

    /// Get current process PID
    fn process_pid(&self) -> Pid {
        Pid(std::process::id() as i32)
    }

    /// Read target process name
    fn process_name(&self, pid: Pid) -> Result<String, LockFileError> {
        match std::fs::read_to_string(format!("/proc/{}/cmdline", *pid)) {
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                Err(LockFileError::NotExists(pid))
            }
            Err(err) => Err(LockFileError::Io { err, pid }),
            Ok(s) => Ok(String::from(s.split('\0').next().unwrap_or_default())),
        }
    }

    /// Override pid lock file
    fn enforce_pid_file(&self, pid: Pid) -> Result<(), LockFileError> {
        std::fs::write(&self.pid_path, format!("{}", pid.0))
            .map_err(|e| LockFileError::Io { pid, err: e })
    }
}
