use thiserror::Error;

pub type Result<T> = std::result::Result<T, S1bCr4ftError>;

#[derive(Error, Debug)]
pub enum S1bCr4ftError {
    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Module error: {0}")]
    Module(String),

    #[error("Package manager error: {0}")]
    Package(String),

    #[error("Backup error: {0}")]
    Backup(String),

    #[error("Audit error: {0}")]
    Audit(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Dependency resolution error: {0}")]
    Dependency(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("YAML parsing error: {0}")]
    Yaml(#[from] serde_yaml::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Lua script error: {0}")]
    #[cfg(feature = "lua-hooks")]
    Lua(#[from] mlua::Error),

    #[error("GPG error: {0}")]
    #[cfg(feature = "gpg-signing")]
    Gpg(String),

    #[error("Network error: {0}")]
    #[cfg(feature = "remote-modules")]
    Network(#[from] reqwest::Error),

    #[error("Unknown error: {0}")]
    Unknown(String),
}

impl S1bCr4ftError {
    pub fn config<S: Into<String>>(msg: S) -> Self {
        Self::Config(msg.into())
    }

    pub fn module<S: Into<String>>(msg: S) -> Self {
        Self::Module(msg.into())
    }

    pub fn package<S: Into<String>>(msg: S) -> Self {
        Self::Package(msg.into())
    }

    pub fn backup<S: Into<String>>(msg: S) -> Self {
        Self::Backup(msg.into())
    }

    pub fn validation<S: Into<String>>(msg: S) -> Self {
        Self::Validation(msg.into())
    }

    #[cfg(feature = "gpg-signing")]
    pub fn gpg<S: Into<String>>(msg: S) -> Self {
        Self::Gpg(msg.into())
    }
}

// Missing audit error variant - need to add it to the enum
impl S1bCr4ftError {
    pub fn audit<S: Into<String>>(msg: S) -> Self {
        Self::Audit(msg.into())
    }
}
