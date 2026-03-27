use crate::error::{OrchestratorError, Result};
use std::path::Path;

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
        let kind = infer::get_from_path(path.as_ref()).map_err(|e| {
            OrchestratorError::Classification(format!("Failed to read file: {}", e))
        })?;

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
                || mime.contains("text")
            {
                return Ok(FileType::Document);
            } else if mime.contains("zip")
                || mime.contains("rar")
                || mime.contains("archive")
                || mime.contains("compressed")
            {
                return Ok(FileType::Archive);
            }
        }

        // Fallback to extension-based classification
        Self::classify_by_extension(path)
    }

    /// Classify file by extension (fallback method)
    pub fn classify_by_extension<P: AsRef<Path>>(path: P) -> Result<FileType> {
        let extension = path
            .as_ref()
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_lowercase())
            .ok_or_else(|| OrchestratorError::Classification("No file extension".to_string()))?;

        let file_type =
            match extension.as_str() {
                // Images
                "jpg" | "jpeg" | "png" | "gif" | "bmp" | "webp" | "svg" | "ico" | "tiff"
                | "tif" => FileType::Image,

                // Videos
                "mp4" | "avi" | "mov" | "mkv" | "flv" | "wmv" | "webm" | "m4v" | "mpg" | "mpeg" => {
                    FileType::Video
                }

                // Audio
                "mp3" | "wav" | "flac" | "aac" | "ogg" | "m4a" | "wma" | "opus" | "alac" => {
                    FileType::Audio
                }

                // Documents
                "pdf" | "doc" | "docx" | "txt" | "rtf" | "odt" | "xlsx" | "xls" | "pptx"
                | "ppt" => FileType::Document,

                // Archives
                "zip" | "rar" | "7z" | "tar" | "gz" | "bz2" | "xz" | "iso" => FileType::Archive,

                _ => FileType::Unknown,
            };

        Ok(file_type)
    }

    /// Get comprehensive file info
    pub fn get_file_info<P: AsRef<Path>>(path: P) -> Result<FileInfo> {
        let path = path.as_ref();
        let metadata = std::fs::metadata(path).map_err(|e| {
            OrchestratorError::Classification(format!("Failed to read metadata: {}", e))
        })?;

        let file_type = Self::classify_by_content(path)
            .unwrap_or_else(|_| Self::classify_by_extension(path).unwrap_or(FileType::Unknown));

        Ok(FileInfo {
            path: path.to_path_buf(),
            size: metadata.len(),
            file_type,
            extension: path
                .extension()
                .and_then(|e| e.to_str())
                .map(|s| s.to_lowercase()),
        })
    }
}

#[derive(Debug, Clone)]
pub struct FileInfo {
    #[allow(dead_code)]
    pub path: std::path::PathBuf,
    pub size: u64,
    pub file_type: FileType,
    #[allow(dead_code)]
    pub extension: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use tempfile::TempDir;

    fn create_temp_file(extension: &str, content: &[u8]) -> (TempDir, PathBuf) {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let file_path = temp_dir.path().join(format!("test.{}", extension));
        std::fs::write(&file_path, content).expect("Failed to write file");
        (temp_dir, file_path)
    }

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

    #[test]
    fn test_classify_images_all_extensions() {
        let extensions = vec![
            "jpg", "jpeg", "png", "gif", "bmp", "webp", "svg", "ico", "tiff", "tif",
        ];
        for ext in extensions {
            let path = PathBuf::from(format!("test.{}", ext));
            let result = FileClassifier::classify_by_extension(&path).unwrap();
            assert_eq!(result, FileType::Image, "Failed for .{}", ext);
        }
    }

    #[test]
    fn test_classify_videos_all_extensions() {
        let extensions = vec![
            "mp4", "avi", "mov", "mkv", "flv", "wmv", "webm", "m4v", "mpg", "mpeg",
        ];
        for ext in extensions {
            let path = PathBuf::from(format!("test.{}", ext));
            let result = FileClassifier::classify_by_extension(&path).unwrap();
            assert_eq!(result, FileType::Video, "Failed for .{}", ext);
        }
    }

    #[test]
    fn test_classify_audio_all_extensions() {
        let extensions = vec![
            "mp3", "wav", "flac", "aac", "ogg", "m4a", "wma", "opus", "alac",
        ];
        for ext in extensions {
            let path = PathBuf::from(format!("test.{}", ext));
            let result = FileClassifier::classify_by_extension(&path).unwrap();
            assert_eq!(result, FileType::Audio, "Failed for .{}", ext);
        }
    }

    #[test]
    fn test_classify_documents_all_extensions() {
        let extensions = vec![
            "pdf", "doc", "docx", "txt", "rtf", "odt", "xlsx", "xls", "pptx", "ppt",
        ];
        for ext in extensions {
            let path = PathBuf::from(format!("test.{}", ext));
            let result = FileClassifier::classify_by_extension(&path).unwrap();
            assert_eq!(result, FileType::Document, "Failed for .{}", ext);
        }
    }

    #[test]
    fn test_classify_archives_all_extensions() {
        let extensions = vec!["zip", "rar", "7z", "tar", "gz", "bz2", "xz", "iso"];
        for ext in extensions {
            let path = PathBuf::from(format!("test.{}", ext));
            let result = FileClassifier::classify_by_extension(&path).unwrap();
            assert_eq!(result, FileType::Archive, "Failed for .{}", ext);
        }
    }

    #[test]
    fn test_classify_case_insensitive() {
        let cases = vec![
            ("test.JPG", FileType::Image),
            ("test.PNG", FileType::Image),
            ("test.MP4", FileType::Video),
            ("test.PDF", FileType::Document),
            ("test.ZIP", FileType::Archive),
        ];
        for (filename, expected) in cases {
            let path = PathBuf::from(filename);
            let result = FileClassifier::classify_by_extension(&path).unwrap();
            assert_eq!(result, expected);
        }
    }

    #[test]
    fn test_classify_no_extension() {
        let path = PathBuf::from("testfile");
        let result = FileClassifier::classify_by_extension(&path);
        assert!(result.is_err());
    }

    #[test]
    fn test_file_type_as_str() {
        assert_eq!(FileType::Image.as_str(), "images");
        assert_eq!(FileType::Video.as_str(), "videos");
        assert_eq!(FileType::Audio.as_str(), "music");
        assert_eq!(FileType::Document.as_str(), "documents");
        assert_eq!(FileType::Archive.as_str(), "archives");
        assert_eq!(FileType::Unknown.as_str(), "unknown");
    }

    #[test]
    fn test_file_type_clone() {
        let original = FileType::Image;
        let cloned = original.clone();
        assert_eq!(original, cloned);
    }

    #[test]
    fn test_file_type_partial_eq() {
        assert_eq!(FileType::Image, FileType::Image);
        assert_ne!(FileType::Image, FileType::Video);
    }

    #[test]
    fn test_classify_by_content_pdf() {
        let (_dir, path) = create_temp_file("pdf", &[0x25, 0x50, 0x44, 0x46]);
        let result = FileClassifier::classify_by_content(&path);
        assert!(result.is_ok());
    }

    #[test]
    fn test_classify_by_content_png() {
        let (_dir, path) = create_temp_file("png", &[0x89, 0x50, 0x4E, 0x47]);
        let result = FileClassifier::classify_by_content(&path);
        assert!(result.is_ok());
    }

    #[test]
    fn test_classify_by_content_nonexistent() {
        let path = PathBuf::from("/nonexistent/file.txt");
        let result = FileClassifier::classify_by_content(&path);
        assert!(result.is_err());
    }

    #[test]
    fn test_get_file_info() {
        let (_dir, path) = create_temp_file("txt", b"test content");
        let result = FileClassifier::get_file_info(&path);
        assert!(result.is_ok());
        let info = result.unwrap();
        assert_eq!(info.size, 12);
        assert_eq!(info.extension, Some("txt".to_string()));
    }

    #[test]
    fn test_get_file_info_nonexistent() {
        let path = PathBuf::from("/nonexistent/file.txt");
        let result = FileClassifier::get_file_info(&path);
        assert!(result.is_err());
    }
}
