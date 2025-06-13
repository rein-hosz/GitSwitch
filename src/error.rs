use thiserror::Error;

/// Custom error types for git-switch
#[derive(Error, Debug)]
pub enum GitSwitchError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON serialization/deserialization error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("TOML serialization/deserialization error: {0}")]
    Toml(#[from] toml::de::Error),

    #[error("TOML serialization error: {0}")]
    TomlSer(#[from] toml::ser::Error),

    #[error("Clap parser error: {0}")]
    Clap(#[from] clap::Error),

    #[error("Dialog error: {0}")]
    Dialog(#[from] dialoguer::Error),

    #[error("Account '{name}' not found")]
    AccountNotFound { name: String },

    #[error("Account '{name}' already exists")]
    AccountExists { name: String },

    #[error("Profile '{name}' not found")]
    ProfileNotFound { name: String },

    #[error("Profile '{name}' already exists")]
    ProfileAlreadyExists { name: String },

    #[error("Account '{account}' not found in profile '{profile}'")]
    AccountNotInProfile { profile: String, account: String },

    #[error("Invalid default account '{account}' for profile '{profile}'")]
    InvalidDefaultAccount { profile: String, account: String },

    #[error("No repositories discovered. Run discovery first.")]
    NoRepositoriesDiscovered,

    #[error("SSH key generation failed: {message}")]
    SshKeyGeneration { message: String },

    #[error("SSH command failed: {command} - {message}")]
    SshCommand { command: String, message: String },

    #[error("Home directory not found. Please ensure the HOME environment variable is set.")]
    HomeDirectoryNotFound,

    #[error("Path expansion failed for: {path}")]
    PathExpansion { path: String },

    #[error("Invalid path: {0}")]
    InvalidPath(std::path::PathBuf),

    #[error("Command execution failed: {command} - {message}")]
    CommandExecution { command: String, message: String },

    #[error(
        "Not in a Git repository. This command requires being run from within a Git repository."
    )]
    NotInGitRepository,

    #[error(
        "Git command '{command}' failed with status: {status}\\nstdout: {stdout}\\nstderr: {stderr}"
    )]
    GitCommandFailed {
        command: String,
        status: std::process::ExitStatus,
        stdout: String,
        stderr: String,
    },

    #[error("Failed to find remote URL for '{remote_name}' in git configuration")]
    GitRemoteUrlNotFound { remote_name: String },

    #[error("Configuration file is corrupted: {message}")]
    CorruptedConfig { message: String },

    #[error("SSH agent is not running or accessible")]
    SshAgentNotRunning,

    #[error("Invalid email format: {email}")]
    InvalidEmail { email: String },

    #[error("Invalid SSH key format: {message}")]
    InvalidSshKey { message: String },

    #[error("Git is not installed or accessible")]
    GitNotInstalled,

    #[error("Keyring error: {message}")]
    Keyring { message: String },

    #[error("Backup operation failed: {message}")]
    BackupFailed { message: String },

    #[error("Restore operation failed: {message}")]
    RestoreFailed { message: String },

    #[error("Migration failed: {message}")]
    MigrationFailed { message: String },

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("An otherwise unhandled error occurred: {0}")]
    Other(String),
}

/// Result type alias for git-switch
pub type Result<T, E = GitSwitchError> = std::result::Result<T, E>;

impl GitSwitchError {
    pub fn exit_code(&self) -> i32 {
        match self {
            Self::Io(_) => 1,
            Self::Json(_) => 1,
            Self::Toml(_) | Self::TomlSer(_) => 1,
            Self::Clap(_) => 1, // Clap errors are usually usage errors
            Self::Dialog(_) => 1,
            Self::AccountNotFound { .. } => 2,
            Self::AccountExists { .. } => 3,
            Self::ProfileNotFound { .. } => 2,
            Self::ProfileAlreadyExists { .. } => 3,
            Self::AccountNotInProfile { .. } => 4,
            Self::InvalidDefaultAccount { .. } => 5,
            Self::NoRepositoriesDiscovered => 22,
            Self::SshKeyGeneration { .. } => 4,
            Self::SshCommand { .. } => 6,
            Self::HomeDirectoryNotFound => 8,
            Self::PathExpansion { .. } => 9,
            Self::InvalidPath(_) => 10,
            Self::CommandExecution { .. } => 11,
            Self::GitCommandFailed { .. } => 11,
            Self::GitRemoteUrlNotFound { .. } => 12,
            Self::CorruptedConfig { .. } => 13,
            Self::SshAgentNotRunning => 14,
            Self::InvalidEmail { .. } => 15,
            Self::InvalidSshKey { .. } => 16,
            Self::GitNotInstalled => 17,
            Self::Keyring { .. } => 18,
            Self::BackupFailed { .. } => 19,
            Self::RestoreFailed { .. } => 20,
            Self::MigrationFailed { .. } => 21,
            Self::SerializationError(_) => 23,
            Self::NotInGitRepository => 13,
            Self::Other(_) => 100, // General error
        }
    }
}
