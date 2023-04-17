use std::path::PathBuf;

use tracing::error;

use crate::{utils, AmdGpuError, Card, IoFailure, ROOT_DIR};

#[derive(Debug)]
pub struct HwMonName(pub String);

impl std::ops::Deref for HwMonName {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub trait RootPath {
    fn root_dir(&self) -> PathBuf;
    fn device_dir(&self, card: &Card) -> PathBuf;
    fn mon_dir(&self, card: &Card, name: &HwMonName) -> PathBuf;
}

#[derive(Debug)]
pub struct SysFs;

impl RootPath for SysFs {
    fn root_dir(&self) -> PathBuf {
        PathBuf::new().join(ROOT_DIR)
    }

    fn device_dir(&self, card: &Card) -> PathBuf {
        self.root_dir().join(card.to_string()).join("device")
    }

    fn mon_dir(&self, card: &Card, name: &HwMonName) -> PathBuf {
        self.device_dir(card).join("hwmon").join(name.as_str())
    }
}

#[derive(Debug)]
pub struct HwMon<Root: RootPath = SysFs> {
    /// HW MOD cord (ex. card0)
    pub card: Card,
    /// MW MOD name (ex. hwmod0)
    pub name: HwMonName,
    root: Root,
}

impl<Root: RootPath> HwMon<Root> {
    pub fn new(card: &Card, name: HwMonName, root: Root) -> Self {
        Self {
            card: *card,
            name,
            root,
        }
    }

    #[inline]
    pub fn card(&self) -> &Card {
        &self.card
    }

    #[inline]
    pub fn name(&self) -> utils::Result<String> {
        self.hw_mon_read("name")
    }

    /// GPU usage percent
    pub fn gpu_usage(&self) -> utils::Result<f64> {
        Ok(self.read_gpu_usage()? as f64)
    }

    pub fn read_gpu_usage(&self) -> utils::Result<u16> {
        let path = self.root.device_dir(self.card()).join("gpu_busy_percent");
        let value = utils::read_to_string(path)?
            .trim()
            .parse()
            .unwrap_or_else(|e| {
                error!("{e}");
                0
            });
        Ok(value)
    }

    #[inline]
    pub fn is_amd(&self) -> bool {
        self.device_read("vendor")
            .map_err(|_| AmdGpuError::FailedReadVendor)
            .map(|vendor| vendor.trim() == "0x1002")
            .unwrap_or_default()
    }

    #[inline]
    pub fn name_is_amd(&self) -> bool {
        self.name().ok().filter(|s| s.trim() == "amdgpu").is_some()
    }

    fn mon_file_path(&self, name: &str) -> PathBuf {
        self.mon_dir().join(name)
    }

    pub fn device_dir(&self) -> PathBuf {
        self.root.device_dir(self.card())
    }

    pub fn mon_dir(&self) -> PathBuf {
        self.root.mon_dir(self.card(), &self.name)
    }

    #[inline]
    pub fn value_or<R: std::str::FromStr>(&self, name: &str, fallback: R) -> R {
        self.hw_mon_read(name)
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(fallback)
    }

    pub fn hw_mon_read(&self, name: &str) -> utils::Result<String> {
        utils::read_to_string(self.mon_file_path(name)).map(|s| String::from(s.trim()))
    }

    pub fn device_read(&self, name: &str) -> utils::Result<String> {
        utils::read_to_string(self.device_dir().join(name)).map(|s| String::from(s.trim()))
    }

    pub fn hw_mon_write(&self, name: &str, value: u64) -> utils::Result<()> {
        utils::write(self.mon_file_path(name), format!("{}", value))?;
        Ok(())
    }

    pub fn device_write<C: AsRef<[u8]>>(&self, name: &str, value: C) -> utils::Result<()> {
        utils::write(self.device_dir().join(name), value)?;
        Ok(())
    }
}

#[inline]
fn hw_mon_dirs_path(card: &Card) -> PathBuf {
    PathBuf::new()
        .join(ROOT_DIR)
        .join(card.to_string())
        .join("device")
        .join("hwmon")
}

pub fn open_hw_mon(card: Card) -> crate::Result<HwMon> {
    let read_path = hw_mon_dirs_path(&card);
    let entries = std::fs::read_dir(&read_path).map_err(|io| IoFailure {
        io,
        path: read_path,
    })?;
    let name = entries
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| {
            entry
                .file_name()
                .as_os_str()
                .to_str()
                .filter(|name| name.starts_with("hwmon"))
                .map(String::from)
                .map(HwMonName)
        })
        .take(1)
        .last()
        .ok_or(AmdGpuError::NoAmdHwMon)?;
    Ok(HwMon::new(&card, name, SysFs))
}
