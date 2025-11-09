use std::path::Path;
use crate::error::{OrchestratorError, Result};

#[derive(Debug, Clone, PartialEq)]
pub enum FileType {
    Image,
    Video,
    Audio,
    Document,
    Archive,
    Unknown,
}

impl FileType {
    pub fn as_str(&self) -> &str {
        match self {
            FileType::Image => "images",
            FileType::Video => "videos",
            FileType::Audio => "music",
            FileType::Document => "documents",
            FileType::Archive => "archives",
            FileType::Unknown => "unknown",
        }
    }
}

pub struct FileClassifier;

impl FileClassifier {
    /// Classify file by reading its magic bytes (more reliable than extension)
    pub fn classify_by_content<P: AsRef<Path>>(path: P) -> Result<FileType> {
        let kind = infer::get_from_path(path.as_ref())
            .map_err(|e| OrchestratorError::Classification(format!("Failed to read file: {}", e)))?;

        if let Some(file_type) = kind {
            let mime = file_type.mime_type();
            
            if mime.starts_with("image/") {
                return Ok(FileType::Image);
            } else if mime.starts_with("video/") {
                return Ok(FileType::Video);
            } else if mime.starts_with("audio/") {
                return Ok(FileType::Audio);
            } else if mime == "application/pdf" 
                || mime.contains("word") 
                || mime.contains("document") 
                || mime.contains("text") {
                return Ok(FileType::Document);
            } else if mime.contains("zip") 
                || mime.contains("rar") 
                || mime.contains("archive") 
                || mime.contains("compressed") {
                return Ok(FileType::Archive);
            }
        }

        // Fallback to extension-based classification
        Self::classify_by_extension(path)
    }

    /// Classify file by extension (fallback method)
    pub fn classify_by_extension<P: AsRef<Path>>(path: P) -> Result<FileType> {
        let extension = path.as_ref()
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_lowercase())
            .ok_or_else(|| OrchestratorError::Classification("No file extension".to_string()))?;

        let file_type = match extension.as_str() {
            // Images
            "jpg" | "jpeg" | "png" | "gif" | "bmp" | "webp" | "svg" | "ico" | "tiff" | "tif" 
                => FileType::Image,
            
            // Videos
            "mp4" | "avi" | "mov" | "mkv" | "flv" | "wmv" | "webm" | "m4v" | "mpg" | "mpeg" 
                => FileType::Video,
            
            // Audio
            "mp3" | "wav" | "flac" | "aac" | "ogg" | "m4a" | "wma" | "opus" | "alac" 
                => FileType::Audio,
            
            // Documents
            "pdf" | "doc" | "docx" | "txt" | "rtf" | "odt" | "xlsx" | "xls" | "pptx" | "ppt" 
                => FileType::Document,
            
            // Archives
            "zip" | "rar" | "7z" | "tar" | "gz" | "bz2" | "xz" | "iso" 
                => FileType::Archive,
            
            _ => FileType::Unknown,
        };

        Ok(file_type)
    }

    /// Get comprehensive file info
    pub fn get_file_info<P: AsRef<Path>>(path: P) -> Result<FileInfo> {
        let path = path.as_ref();
        let metadata = std::fs::metadata(path)
            .map_err(|e| OrchestratorError::Classification(format!("Failed to read metadata: {}", e)))?;

        let file_type = Self::classify_by_content(path)
            .unwrap_or_else(|_| Self::classify_by_extension(path).unwrap_or(FileType::Unknown));

        Ok(FileInfo {
            path: path.to_path_buf(),
            size: metadata.len(),
            file_type,
            extension: path.extension()
                .and_then(|e| e.to_str())
                .map(|s| s.to_lowercase()),
        })
    }
}

#[derive(Debug, Clone)]
pub struct FileInfo {
    pub path: std::path::PathBuf,
    pub size: u64,
    pub file_type: FileType,
    pub extension: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_classify_by_extension() {
        let test_cases = vec![
            ("test.jpg", FileType::Image),
            ("test.mp4", FileType::Video),
            ("test.mp3", FileType::Audio),
            ("test.pdf", FileType::Document),
            ("test.zip", FileType::Archive),
            ("test.unknown", FileType::Unknown),
        ];

        for (filename, expected) in test_cases {
            let path = PathBuf::from(filename);
            let result = FileClassifier::classify_by_extension(&path).unwrap();
            assert_eq!(result, expected, "Failed for {}", filename);
        }
    }
}
