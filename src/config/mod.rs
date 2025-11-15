use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use crate::error::{OrchestratorError, Result};

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

    /// Save configuration to a TOML file
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let content = toml::to_string_pretty(self)?;
        fs::write(path, content)
            .map_err(|e| OrchestratorError::Config(format!("Failed to write config file: {}", e)))?;
        Ok(())
    }

    /// Validate configuration
    fn validate(&self) -> Result<()> {
        if !self.source.path.exists() {
            return Err(OrchestratorError::Config(
                format!("Source path does not exist: {:?}", self.source.path)
            ));
        }

        if self.drives.is_empty() {
            return Err(OrchestratorError::Config(
                "No drives configured".to_string()
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
        self.drives.iter().find(|(_, drive)| drive.target == category)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_file_category() {
        let config = Config::default_config();
        
        assert_eq!(config.get_file_category("jpg"), Some("images".to_string()));
        assert_eq!(config.get_file_category("mp4"), Some("videos".to_string()));
        assert_eq!(config.get_file_category("mp3"), Some("music".to_string()));
        assert_eq!(config.get_file_category("unknown"), None);
    }
}
