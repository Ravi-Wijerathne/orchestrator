use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher as NotifyWatcher, EventKind};
use std::path::Path;
use std::sync::mpsc::{channel, Receiver};
use std::time::Duration;
use tokio::sync::mpsc as tokio_mpsc;
use crate::error::{OrchestratorError, Result};
use tracing::{info, warn, error};

#[derive(Debug, Clone)]
pub enum FileEvent {
    Created(std::path::PathBuf),
    Modified(std::path::PathBuf),
    Removed(std::path::PathBuf),
}

pub struct FileWatcher {
    watcher: RecommendedWatcher,
    event_rx: Receiver<notify::Result<Event>>,
}

impl FileWatcher {
    /// Create a new file watcher
    pub fn new() -> Result<Self> {
        let (tx, rx) = channel();

        let watcher = RecommendedWatcher::new(
            move |res| {
                if let Err(e) = tx.send(res) {
                    error!("Failed to send file event: {}", e);
                }
            },
            Config::default()
                .with_poll_interval(Duration::from_secs(2))
        ).map_err(|e| OrchestratorError::Watch(format!("Failed to create watcher: {}", e)))?;

        Ok(Self {
            watcher,
            event_rx: rx,
        })

    }

    /// Start watching a directory
    pub fn watch<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        let path = path.as_ref();
        
        info!("Starting to watch directory: {}", path.display());
        
        self.watcher
            .watch(path, RecursiveMode::Recursive)
            .map_err(|e| OrchestratorError::Watch(format!("Failed to watch directory: {}", e)))?;

        Ok(())
    }

    /// Stop watching a directory
    #[allow(dead_code)]
    pub fn unwatch<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        let path = path.as_ref();
        
        self.watcher
            .unwatch(path)
            .map_err(|e| OrchestratorError::Watch(format!("Failed to unwatch directory: {}", e)))?;

        Ok(())
    }

    /// Process events and send simplified file events to a channel
    pub async fn process_events(
        &mut self,
        event_sender: tokio_mpsc::UnboundedSender<FileEvent>,
    ) -> Result<()> {
        loop {
            match self.event_rx.recv() {
                Ok(Ok(event)) => {
                    if let Some(file_event) = Self::convert_event(event) {
                        if let Err(e) = event_sender.send(file_event.clone()) {
                            error!("Failed to send file event to channel: {}", e);
                            break;
                        }
                        
                        match file_event {
                            FileEvent::Created(path) => info!("File created: {}", path.display()),
                            FileEvent::Modified(path) => info!("File modified: {}", path.display()),
                            FileEvent::Removed(path) => info!("File removed: {}", path.display()),
                        }
                    }
                }
                Ok(Err(e)) => {
                    warn!("Watch error: {}", e);
                }
                Err(e) => {
                    error!("Failed to receive event: {}", e);
                    break;
                }
            }
        }

        Ok(())
    }

    /// Convert notify events to our simplified FileEvent
    fn convert_event(event: Event) -> Option<FileEvent> {
        if event.paths.is_empty() {
            return None;
        }

        let path = event.paths[0].clone();

        // Filter out directories and only process files
        if path.is_dir() {
            return None;
        }

        match event.kind {
            EventKind::Create(_) => Some(FileEvent::Created(path)),
            EventKind::Modify(_) => Some(FileEvent::Modified(path)),
            EventKind::Remove(_) => Some(FileEvent::Removed(path)),
            _ => None,
        }
    }
}

/// A simplified async file watcher that can be used in a tokio runtime
pub struct AsyncFileWatcher {
    event_rx: tokio_mpsc::UnboundedReceiver<FileEvent>,
}

impl AsyncFileWatcher {
    /// Create a new async file watcher and start watching a path
    pub fn watch<P: AsRef<Path>>(path: P) -> Result<Self> {
        let (tx, rx) = tokio_mpsc::unbounded_channel();
        let path = path.as_ref().to_path_buf();

        // Spawn a blocking thread to handle the sync watcher
        std::thread::spawn(move || {
            let mut watcher = match FileWatcher::new() {
                Ok(w) => w,
                Err(e) => {
                    error!("Failed to create file watcher: {}", e);
                    return;
                }
            };

            if let Err(e) = watcher.watch(&path) {
                error!("Failed to watch path: {}", e);
                return;
            }

            // Create a tokio runtime for the blocking thread
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                if let Err(e) = watcher.process_events(tx).await {
                    error!("Error processing events: {}", e);
                }
            });
        });

        Ok(Self { event_rx: rx })
    }

    /// Receive the next file event
    pub async fn next_event(&mut self) -> Option<FileEvent> {
        self.event_rx.recv().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_file_watcher_creation() {
        let watcher = FileWatcher::new();
        assert!(watcher.is_ok());
    }

    #[tokio::test]
    async fn test_watch_directory() {
        let temp_dir = TempDir::new().unwrap();
        let mut watcher = FileWatcher::new().unwrap();
        
        let result = watcher.watch(temp_dir.path());
        assert!(result.is_ok());
    }
}
