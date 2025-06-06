// filepath: /home/renhoshizora/Project/git_switch/src/error.rs
use thiserror::Error;

/// Custom error types for git-switch
#[derive(Error, Debug)]
pub enum GitSwitchError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON serialization/deserialization error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Account '{name}' not found")]
    AccountNotFound { name: String },

    #[error("Account '{name}' already exists")]
    AccountExists { name: String },

    #[error("SSH key generation failed: {message}")]
    SshKeyGeneration { message: String },

    // Commenting out unused variants for now, can be re-enabled if needed
    // #[error("Git command failed: {command} - {message}")]
    // GitCommand { command: String, message: String },

    #[error("SSH command failed: {command} - {message}")]
    SshCommand { command: String, message: String },

    // #[error("Invalid configuration: {message}")]
    // InvalidConfig { message: String },

    #[error("Home directory not found. Please ensure the HOME environment variable is set.")]
    HomeDirectoryNotFound,

    #[error("Path expansion failed for: {path}")]
    PathExpansion { path: String },

    #[error("Command execution failed: {command} - {message}")]
    CommandExecution { command: String, message: String },

    #[error("Failed to get argument '{arg_name}' from CLI matches.")]
    CliArgumentError { arg_name: String },
}

/// Result type alias for git-switch
pub type Result<T, E = GitSwitchError> = std::result::Result<T, E>;

impl GitSwitchError {
    pub fn exit_code(&self) -> i32 {
        match self {
            Self::Io(_) => 1,
            Self::Json(_) => 1,
            Self::AccountNotFound { .. } => 2,
            Self::AccountExists { .. } => 3,
            Self::SshKeyGeneration { .. } => 4,
            // Self::GitCommand { .. } => 5, // Commented out
            Self::SshCommand { .. } => 6,
            // Self::InvalidConfig { .. } => 7, // Commented out
            Self::HomeDirectoryNotFound => 8,
            Self::PathExpansion { .. } => 9,
            Self::CommandExecution { .. } => 10,
            Self::CliArgumentError { .. } => 11,
        }
    }
}
