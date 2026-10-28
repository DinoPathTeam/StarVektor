//! GPU memory counters from the `amdgpu` kernel driver.
//!
//! Files under `/sys/class/drm/cardN/device/`: `mem_info_vram_used` and
//! `mem_info_vram_total`, in bytes. Reading them needs no privileges.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use sv_types::MemoryBudget;

const MIB: u64 = 1024 * 1024;

/// Reader of the VRAM counters of one card.
#[derive(Debug, Clone)]
pub struct AmdgpuSysfs {
    device_dir: PathBuf,
}

impl AmdgpuSysfs {
    /// `device_dir` is the card's `device` directory
    /// (for example `/sys/class/drm/card1/device`).
    #[must_use]
    pub fn new(device_dir: impl Into<PathBuf>) -> Self {
        Self {
            device_dir: device_dir.into(),
        }
    }

    /// Finds the first card under `drm_root` (normally `/sys/class/drm`) that
    /// exposes `mem_info_vram_total`.
    #[must_use]
    pub fn detect(drm_root: &Path) -> Option<Self> {
        let mut cards: Vec<PathBuf> = fs::read_dir(drm_root)
            .ok()?
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .filter(|path| {
                path.file_name().and_then(|n| n.to_str()).is_some_and(|n| {
                    n.starts_with("card") && n[4..].chars().all(|c| c.is_ascii_digit())
                })
            })
            .map(|path| path.join("device"))
            .filter(|device| device.join("mem_info_vram_total").is_file())
            .collect();
        cards.sort();
        cards.into_iter().next().map(Self::new)
    }

    /// Current VRAM budget.
    ///
    /// # Errors
    /// If a counter cannot be read or does not hold a number.
    pub fn memory_budget(&self) -> io::Result<MemoryBudget> {
        let used = self.read_bytes("mem_info_vram_used")?;
        let total = self.read_bytes("mem_info_vram_total")?;
        Ok(budget(used, total))
    }

    /// Total VRAM in MiB.
    ///
    /// # Errors
    /// If the counter cannot be read.
    pub fn total_mb(&self) -> io::Result<u32> {
        Ok(to_mb(self.read_bytes("mem_info_vram_total")?))
    }

    fn read_bytes(&self, name: &str) -> io::Result<u64> {
        let text = fs::read_to_string(self.device_dir.join(name))?;
        text.trim().parse().map_err(|_| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("{name} does not hold a number: {}", text.trim()),
            )
        })
    }
}

fn to_mb(bytes: u64) -> u32 {
    u32::try_from(bytes / MIB).unwrap_or(u32::MAX)
}

/// Budget from raw byte counters; free memory never goes below zero.
#[must_use]
pub fn budget(used_bytes: u64, total_bytes: u64) -> MemoryBudget {
    MemoryBudget {
        used_mb: to_mb(used_bytes),
        free_mb: to_mb(total_bytes.saturating_sub(used_bytes)),
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicU32, Ordering};

    use super::*;

    static COUNTER: AtomicU32 = AtomicU32::new(0);

    fn temp_dir() -> PathBuf {
        let id = COUNTER.fetch_add(1, Ordering::Relaxed);
        let dir = std::env::temp_dir().join(format!("sv-sysfs-{}-{id}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn fake_card(root: &Path, name: &str, used: &str, total: &str) {
        let device = root.join(name).join("device");
        fs::create_dir_all(&device).unwrap();
        fs::write(device.join("mem_info_vram_used"), used).unwrap();
        fs::write(device.join("mem_info_vram_total"), total).unwrap();
    }

    #[test]
    fn reads_used_and_free() {
        let root = temp_dir();
        fake_card(&root, "card1", "2147483648\n", "8589934592\n");
        let reader = AmdgpuSysfs::new(root.join("card1/device"));
        assert_eq!(
            reader.memory_budget().unwrap(),
            MemoryBudget {
                used_mb: 2048,
                free_mb: 6144
            }
        );
        assert_eq!(reader.total_mb().unwrap(), 8192);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn detects_the_card_that_has_the_counters() {
        let root = temp_dir();
        fs::create_dir_all(root.join("card0/device")).unwrap(); // e.g. an iGPU without counters
        fs::create_dir_all(root.join("card1-HDMI-A-1")).unwrap(); // connector, not a card
        fake_card(&root, "card1", "0", "1048576");
        let found = AmdgpuSysfs::detect(&root).unwrap();
        assert_eq!(found.total_mb().unwrap(), 1);
        assert!(AmdgpuSysfs::detect(&root.join("missing")).is_none());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn reports_unreadable_counters() {
        let root = temp_dir();
        fake_card(&root, "card1", "not a number", "1");
        let reader = AmdgpuSysfs::new(root.join("card1/device"));
        assert_eq!(
            reader.memory_budget().unwrap_err().kind(),
            io::ErrorKind::InvalidData
        );
        assert!(AmdgpuSysfs::new(root.join("nope")).memory_budget().is_err());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn free_memory_never_goes_negative() {
        assert_eq!(budget(2 * MIB, MIB).free_mb, 0);
    }
}
