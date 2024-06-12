#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    ConfigurationError(#[from] twelf::Error),
}