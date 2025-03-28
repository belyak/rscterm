use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Provider error: {0}")]
    Provider(String),
    
    #[error("Invalid model: {0}")]
    InvalidModel(String),
    
    #[error("No active team")]
    NoActiveTeam,
    
    #[error("Invalid command: {0}")]
    InvalidCommand(String),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, Error>; 