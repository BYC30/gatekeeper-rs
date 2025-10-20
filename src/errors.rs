use thiserror::Error;

pub type Result<T> = std::result::Result<T, GatekeeperError>;

#[derive(Error, Debug)]
pub enum GatekeeperError {
    #[error("invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("no available keys")]
    NoAvailableKeys,

    #[error("unsupported provider: {0}")]
    UnsupportedProvider(String),

    #[error("upstream error: {0}")]
    Upstream(String),
}
