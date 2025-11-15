use sysinfo::Disks;
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct DriveInfo {
    pub name: String,
    pub mount_point: PathBuf,
    pub total_space: u64,
    pub available_space: u64,
    pub file_system: String,
    pub is_removable: bool,
}

pub struct DriveDetector {
    disks: Disks,
}

impl DriveDetector {
    /// Create a new drive detector
    pub fn new() -> Self {
        let mut disks = Disks::new_with_refreshed_list();
        disks.refresh_list();
        
        Self { disks }
    }

    /// Refresh the list of available drives
    pub fn refresh(&mut self) {
        self.disks.refresh_list();
    }

    /// Get all currently connected drives
    pub fn get_all_drives(&self) -> Vec<DriveInfo> {
        self.disks
            .iter()
            .map(|disk| DriveInfo {
                name: disk.name().to_string_lossy().to_string(),
                mount_point: disk.mount_point().to_path_buf(),
                total_space: disk.total_space(),
                available_space: disk.available_space(),
                file_system: disk.file_system().to_string_lossy().to_string(),
                is_removable: disk.is_removable(),
            })
            .collect()
    }

    /// Get only removable drives (USB drives)
    #[allow(dead_code)]
    pub fn get_removable_drives(&self) -> Vec<DriveInfo> {
        self.get_all_drives()
            .into_iter()
            .filter(|drive| drive.is_removable)
            .collect()
    }

    /// Check if a specific drive is connected by mount point
    pub fn is_drive_connected(&self, mount_point: &PathBuf) -> bool {
        self.disks
            .iter()
            .any(|disk| disk.mount_point() == mount_point)
    }

    /// Find drive by label/name (case-insensitive partial match)
    pub fn find_drive_by_label(&self, label: &str) -> Option<DriveInfo> {
        let label_lower = label.to_lowercase();
        
        self.get_all_drives()
            .into_iter()
            .find(|drive| {
                drive.name.to_lowercase().contains(&label_lower) ||
                drive.mount_point.to_string_lossy().to_lowercase().contains(&label_lower)
            })
    }

    /// Get drive info for a specific path
    #[allow(dead_code)]
    pub fn get_drive_for_path(&self, path: &PathBuf) -> Option<DriveInfo> {
        // Find the disk that contains this path
        self.get_all_drives()
            .into_iter()
            .find(|drive| path.starts_with(&drive.mount_point))
    }

    /// Create a simple UUID-like identifier from drive info
    /// Note: This is a simple implementation. For production, you might want to use
    /// platform-specific APIs to get real UUIDs
    pub fn generate_drive_id(drive: &DriveInfo) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        drive.name.hash(&mut hasher);
        drive.mount_point.hash(&mut hasher);
        drive.total_space.hash(&mut hasher);
        
        format!("drive-{:x}", hasher.finish())
    }

    /// Monitor for newly connected drives (blocking)
    /// Returns a list of newly detected drives
    #[allow(dead_code)]
    pub fn wait_for_new_drives(&mut self, timeout_secs: u64) -> Vec<DriveInfo> {
        let initial_drives = self.get_drive_map();
        let start = std::time::Instant::now();

        loop {
            std::thread::sleep(std::time::Duration::from_secs(2));
            self.refresh();

            let current_drives = self.get_drive_map();
            let new_drives: Vec<DriveInfo> = current_drives
                .into_iter()
                .filter(|(mount, _)| !initial_drives.contains_key(mount))
                .map(|(_, drive)| drive)
                .collect();

            if !new_drives.is_empty() {
                return new_drives;
            }

            if start.elapsed().as_secs() >= timeout_secs {
                return Vec::new();
            }
        }
    }

    /// Get a map of mount points to drive info
    #[allow(dead_code)]
    fn get_drive_map(&self) -> HashMap<PathBuf, DriveInfo> {
        self.get_all_drives()
            .into_iter()
            .map(|drive| (drive.mount_point.clone(), drive))
            .collect()
    }

    /// Print information about all connected drives
    pub fn print_drives(&self) {
        println!("\n=== Connected Drives ===");
        for drive in self.get_all_drives() {
            println!("\nDrive: {}", drive.name);
            println!("  Mount Point: {}", drive.mount_point.display());
            println!("  Total Space: {} GB", drive.total_space / 1_000_000_000);
            println!("  Available: {} GB", drive.available_space / 1_000_000_000);
            println!("  File System: {}", drive.file_system);
            println!("  Removable: {}", drive.is_removable);
            println!("  Drive ID: {}", Self::generate_drive_id(&drive));
        }
        println!("\n========================\n");
    }
}

impl Default for DriveDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_drive_detector_creation() {
        let detector = DriveDetector::new();
        let drives = detector.get_all_drives();
        
        // Should detect at least the system drive
        assert!(!drives.is_empty(), "Should detect at least one drive");
    }

    #[test]
    fn test_drive_id_generation() {
        let drive = DriveInfo {
            name: "TestDrive".to_string(),
            mount_point: PathBuf::from("/mnt/test"),
            total_space: 1000000000,
            available_space: 500000000,
            file_system: "NTFS".to_string(),
            is_removable: true,
        };

        let id = DriveDetector::generate_drive_id(&drive);
        assert!(id.starts_with("drive-"));
        assert!(id.len() > 6);
    }
}
