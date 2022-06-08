use gdk_common::session::JsonError;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Greenlight method {0} not found.")]
    MethodNotFound(String),

    #[error("Greenlight internal method {0} not found.")]
    InternalMethodNotFound(String),

    #[error("GreenlightSession client not initalized, call `login()`")]
    ClientNotInitialized,

    #[error("Local encryption keys aren't present, call `set_local_encryption_keys()`")]
    EncryptionKeysRequired,

    #[error("`state_dir` must be a valid directory ({0})")]
    StateDirNotDirectory(std::path::PathBuf),

    #[error(transparent)]
    GdkCommon(#[from] gdk_common::Error),

    #[error(transparent)]
    Hex(#[from] bitcoin::hashes::hex::Error),

    #[error(transparent)]
    Anyhow(#[from] anyhow::Error),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Tonic(#[from] tonic::Status),

    #[error(transparent)]
    Serde(#[from] serde_json::Error),

    #[error(transparent)]
    Bip39(#[from] bip39::Error),

    #[error(transparent)]
    Bip32(#[from] bitcoin::util::bip32::Error),

    /// A generic error.
    #[error("{0}")]
    Generic(String),
}

// `aead::Error` doesn't implement `std::error::Error`.
impl From<aes_gcm_siv::aead::Error> for Error {
    fn from(err: aes_gcm_siv::aead::Error) -> Self {
        Error::Generic(err.to_string())
    }
}

impl From<Error> for JsonError {
    fn from(e: Error) -> Self {
        JsonError {
            message: e.to_string(),
            error: e.to_gdk_code(),
        }
    }
}

impl Error {
    pub fn to_gdk_code(&self) -> String {
        // TODO implement correct mapping
        "id_unknown".to_string()
    }
}
