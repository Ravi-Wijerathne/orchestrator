use thiserror::Error;

#[derive(Error, Debug)]
pub enum OrchestratorError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Drive not found: {0}")]
    DriveNotFound(String),

    #[error("File classification error: {0}")]
    Classification(String),

    #[error("Sync error: {0}")]
    Sync(String),

    #[error("State management error: {0}")]
    State(String),

    #[error("Watch error: {0}")]
    Watch(String),

    #[error("Database error: {0}")]
    Database(#[from] sled::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("TOML parsing error: {0}")]
    TomlParse(#[from] toml::de::Error),

    #[error("TOML serialization error: {0}")]
    TomlSerialize(#[from] toml::ser::Error),
}

pub type Result<T> = std::result::Result<T, OrchestratorError>;
