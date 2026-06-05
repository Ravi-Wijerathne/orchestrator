use crate::error::{OrchestratorError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub source: SourceConfig,
    pub rules: FileRules,
    pub drives: HashMap<String, DriveConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceConfig {
    pub path: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileRules {
    pub images: Vec<String>,
    pub videos: Vec<String>,
    pub music: Vec<String>,
    pub documents: Option<Vec<String>>,
    pub archives: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriveConfig {
    pub label: String,
    pub target: String,
    pub path: Option<PathBuf>,
    pub last_seen: Option<String>,
}

impl Config {
    /// Load configuration from a TOML file
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = fs::read_to_string(path)
            .map_err(|e| OrchestratorError::Config(format!("Failed to read config file: {}", e)))?;

        let config: Config = toml::from_str(&content)?;
        config.validate()?;
        Ok(config)
    }

    /// Load configuration without validating paths (for GUI mode)
    pub fn load_lenient<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = fs::read_to_string(path)
            .map_err(|e| OrchestratorError::Config(format!("Failed to read config file: {}", e)))?;

        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }

    /// Save configuration to a TOML file
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let content = toml::to_string_pretty(self)?;
        fs::write(path, content).map_err(|e| {
            OrchestratorError::Config(format!("Failed to write config file: {}", e))
        })?;
        Ok(())
    }

    /// Validate configuration
    fn validate(&self) -> Result<()> {
        if !self.source.path.exists() {
            return Err(OrchestratorError::Config(format!(
                "Source path does not exist: {:?}",
                self.source.path
            )));
        }

        if self.drives.is_empty() {
            return Err(OrchestratorError::Config(
                "No drives configured".to_string(),
            ));
        }

        Ok(())
    }

    /// Create a default configuration
    pub fn default_config() -> Self {
        let mut drives = HashMap::new();

        drives.insert(
            "example-uuid-1".to_string(),
            DriveConfig {
                label: "ImageUSB".to_string(),
                target: "images".to_string(),
                path: None,
                last_seen: None,
            },
        );

        drives.insert(
            "example-uuid-2".to_string(),
            DriveConfig {
                label: "VideoUSB".to_string(),
                target: "videos".to_string(),
                path: None,
                last_seen: None,
            },
        );

        drives.insert(
            "example-uuid-3".to_string(),
            DriveConfig {
                label: "MusicUSB".to_string(),
                target: "music".to_string(),
                path: None,
                last_seen: None,
            },
        );

        Config {
            source: SourceConfig {
                path: PathBuf::from("D:/MainStorage"),
            },
            rules: FileRules {
                images: vec!["jpg", "jpeg", "png", "gif", "bmp", "webp", "svg"]
                    .iter()
                    .map(|s| s.to_string())
                    .collect(),
                videos: vec!["mp4", "avi", "mov", "mkv", "flv", "wmv", "webm"]
                    .iter()
                    .map(|s| s.to_string())
                    .collect(),
                music: vec!["mp3", "wav", "flac", "aac", "ogg", "m4a", "wma"]
                    .iter()
                    .map(|s| s.to_string())
                    .collect(),
                documents: Some(
                    vec!["pdf", "doc", "docx", "txt", "xlsx", "pptx"]
                        .iter()
                        .map(|s| s.to_string())
                        .collect(),
                ),
                archives: Some(
                    vec!["zip", "rar", "7z", "tar", "gz"]
                        .iter()
                        .map(|s| s.to_string())
                        .collect(),
                ),
            },
            drives,
        }
    }

    /// Get file category based on extension
    #[allow(dead_code)]
    pub fn get_file_category(&self, extension: &str) -> Option<String> {
        let ext = extension.to_lowercase();

        if self.rules.images.contains(&ext) {
            return Some("images".to_string());
        }
        if self.rules.videos.contains(&ext) {
            return Some("videos".to_string());
        }
        if self.rules.music.contains(&ext) {
            return Some("music".to_string());
        }
        if let Some(docs) = &self.rules.documents {
            if docs.contains(&ext) {
                return Some("documents".to_string());
            }
        }
        if let Some(archives) = &self.rules.archives {
            if archives.contains(&ext) {
                return Some("archives".to_string());
            }
        }

        None
    }

    /// Find drive UUID for a given category
    pub fn find_drive_for_category(&self, category: &str) -> Option<(&String, &DriveConfig)> {
        self.drives
            .iter()
            .find(|(_, drive)| drive.target == category)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use tempfile::TempDir;

    fn create_test_config() -> Config {
        let mut drives = HashMap::new();
        drives.insert(
            "test-uuid".to_string(),
            DriveConfig {
                label: "TestDrive".to_string(),
                target: "images".to_string(),
                path: Some(std::path::PathBuf::from("E:/")),
                last_seen: None,
            },
        );
        Config {
            source: SourceConfig {
                path: std::path::PathBuf::from("D:/TestSource"),
            },
            rules: FileRules {
                images: vec!["jpg".to_string(), "png".to_string()],
                videos: vec!["mp4".to_string()],
                music: vec!["mp3".to_string()],
                documents: Some(vec!["pdf".to_string()]),
                archives: Some(vec!["zip".to_string()]),
            },
            drives,
        }
    }

    #[test]
    fn test_get_file_category() {
        let config = Config::default_config();

        assert_eq!(config.get_file_category("jpg"), Some("images".to_string()));
        assert_eq!(config.get_file_category("mp4"), Some("videos".to_string()));
        assert_eq!(config.get_file_category("mp3"), Some("music".to_string()));
        assert_eq!(config.get_file_category("unknown"), None);
    }

    #[test]
    fn test_get_file_category_images() {
        let config = Config::default_config();
        assert_eq!(config.get_file_category("jpg"), Some("images".to_string()));
        assert_eq!(config.get_file_category("png"), Some("images".to_string()));
        assert_eq!(config.get_file_category("gif"), Some("images".to_string()));
    }

    #[test]
    fn test_get_file_category_videos() {
        let config = Config::default_config();
        assert_eq!(config.get_file_category("mp4"), Some("videos".to_string()));
        assert_eq!(config.get_file_category("avi"), Some("videos".to_string()));
        assert_eq!(config.get_file_category("mkv"), Some("videos".to_string()));
    }

    #[test]
    fn test_get_file_category_music() {
        let config = Config::default_config();
        assert_eq!(config.get_file_category("mp3"), Some("music".to_string()));
        assert_eq!(config.get_file_category("wav"), Some("music".to_string()));
        assert_eq!(config.get_file_category("flac"), Some("music".to_string()));
    }

    #[test]
    fn test_get_file_category_documents() {
        let config = Config::default_config();
        assert_eq!(
            config.get_file_category("pdf"),
            Some("documents".to_string())
        );
        assert_eq!(
            config.get_file_category("doc"),
            Some("documents".to_string())
        );
        assert_eq!(
            config.get_file_category("docx"),
            Some("documents".to_string())
        );
    }

    #[test]
    fn test_get_file_category_archives() {
        let config = Config::default_config();
        assert_eq!(
            config.get_file_category("zip"),
            Some("archives".to_string())
        );
        assert_eq!(
            config.get_file_category("rar"),
            Some("archives".to_string())
        );
        assert_eq!(config.get_file_category("7z"), Some("archives".to_string()));
    }

    #[test]
    fn test_get_file_category_case_insensitive() {
        let config = Config::default_config();
        assert_eq!(config.get_file_category("JPG"), Some("images".to_string()));
        assert_eq!(config.get_file_category("PNG"), Some("images".to_string()));
        assert_eq!(config.get_file_category("MP4"), Some("videos".to_string()));
    }

    #[test]
    fn test_find_drive_for_category() {
        let config = create_test_config();
        let result = config.find_drive_for_category("images");
        assert!(result.is_some());
        let (uuid, drive) = result.unwrap();
        assert_eq!(*uuid, "test-uuid");
        assert_eq!(drive.target, "images");
    }

    #[test]
    fn test_find_drive_for_nonexistent_category() {
        let config = create_test_config();
        let result = config.find_drive_for_category("nonexistent");
        assert!(result.is_none());
    }

    #[test]
    fn test_default_config_creation() {
        let config = Config::default_config();
        assert!(!config.rules.images.is_empty());
        assert!(!config.rules.videos.is_empty());
        assert!(!config.rules.music.is_empty());
        assert_eq!(config.drives.len(), 3);
    }

    #[test]
    fn test_default_config_drives_have_labels() {
        let config = Config::default_config();
        let labels: Vec<&String> = config.drives.values().map(|d| &d.label).collect();
        assert!(labels.contains(&&"ImageUSB".to_string()));
        assert!(labels.contains(&&"VideoUSB".to_string()));
        assert!(labels.contains(&&"MusicUSB".to_string()));
    }

    #[test]
    fn test_config_to_toml() {
        let config = create_test_config();
        let toml_str = toml::to_string_pretty(&config);
        assert!(toml_str.is_ok());
    }

    #[test]
    fn test_config_roundtrip() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let config_path = temp_dir.path().join("config.toml");

        let mut config = Config::default_config();
        config.source.path = temp_dir.path().to_path_buf();
        config.save(&config_path).expect("Failed to save config");

        let loaded = Config::load(&config_path);
        assert!(loaded.is_ok());
        let loaded = loaded.unwrap();
        assert_eq!(loaded.drives.len(), config.drives.len());
    }

    #[test]
    fn test_config_load_invalid_path() {
        let result = Config::load("/nonexistent/path/config.toml");
        assert!(result.is_err());
    }

    #[test]
    fn test_source_config_clone() {
        let original = SourceConfig {
            path: std::path::PathBuf::from("/test/path"),
        };
        let cloned = original.clone();
        assert_eq!(original.path, cloned.path);
    }

    #[test]
    fn test_file_rules_clone() {
        let original = FileRules {
            images: vec!["jpg".to_string()],
            videos: vec!["mp4".to_string()],
            music: vec!["mp3".to_string()],
            documents: Some(vec!["pdf".to_string()]),
            archives: Some(vec!["zip".to_string()]),
        };
        let cloned = original.clone();
        assert_eq!(original.images, cloned.images);
    }

    #[test]
    fn test_drive_config_clone() {
        let original = DriveConfig {
            label: "Test".to_string(),
            target: "images".to_string(),
            path: Some(std::path::PathBuf::from("E:/")),
            last_seen: Some("2024-01-01".to_string()),
        };
        let cloned = original.clone();
        assert_eq!(original.label, cloned.label);
        assert_eq!(original.target, cloned.target);
    }
}
