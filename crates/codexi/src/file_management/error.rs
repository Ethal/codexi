// src/file_management/error.rs

use thiserror::Error;

use crate::core::CoreError;
use crate::exchange::ExchangeError;
use crate::file_management::StorageFormat;
use crate::logic::account::AccountError;
use crate::logic::codexi::CodexiError;

/// Error type for FileManagement
#[derive(Debug, Error)]
pub enum FileManagementError {
    #[error("SYS_IO: {0}")]
    Io(#[from] std::io::Error),
}

/// Error type for archive file (.cld)
#[derive(Debug, Error)]
pub enum FileArchiveError {
    #[error("SYS_IO: {0}")]
    Io(#[from] std::io::Error),
    #[error("SYS_STORAGE: {0}")]
    Storage(#[from] StorageError),
}

/// Error type for backup file (.zip)
#[derive(Debug, Error)]
pub enum FileBackupError {
    #[error("SYS_IO: {0}")]
    Io(#[from] std::io::Error),
    #[error("DATA_TOML: {0}")]
    InvalidTomlDe(#[from] toml::de::Error),
    #[error("DATA_TOML: {0}")]
    InvalidTomlSer(#[from] toml::ser::Error),
    #[error("SYS_TAR: {0}")]
    Maintenance(#[from] FileMaintenanceError),
    #[error("SYS_CORE: {0}")]
    Core(#[from] CoreError),
    #[error("DATA_NO_DIR_OR_FILE: {0}")]
    NoDirOrFile(String),
    #[error("DATA_NO_RELATIVE_PATH: {0}")]
    RelativePath(String),
    #[error("DATA_INVALID_PATH: {0}")]
    InvalidPath(String),
    #[error("DATA_NO_BACKUP_PATH: {0}")]
    InvalidBackupPath(String),
    #[error("DATA_NO_DOCUMENTS_DIR: {0}")]
    NoDocumentsDirectory(String),
    #[error("DATA_INVALID: {0}")]
    InvalidData(String),
}

/// Error type for ledger file (codexi.dat)
#[derive(Debug, Error)]
pub enum FileCodexiError {
    #[error("SYS_IO: {0}")]
    Io(#[from] std::io::Error),
    #[error("SYS_STORAGE: {0}")]
    Storage(#[from] StorageError),
    #[error("SYS_CODEXI: {0}")]
    Codexi(#[from] CodexiError),
    #[error("OP_ACCOUNT: {0}")]
    Account(#[from] AccountError),
}

/// Error type for Storage
#[derive(Debug, Error)]
pub enum StorageError {
    #[error("SYS_IO: {0}")]
    Io(#[from] std::io::Error),
    #[error("SYS_CIBORIUM_SER: {0}, Try the command migrate <VERSION>")]
    CiboriumSer(#[from] ciborium::ser::Error<std::io::Error>),
    #[error("SYS_CIBORIUM_DE: {0}, Try the command migrate <VERSION>")]
    CiboriumDe(#[from] ciborium::de::Error<std::io::Error>),
    #[error("VAL_INVALID_FILE: Not a codexi data file, Try the command migrate <VERSION>")]
    InvalidMagic,
    #[error("VAL_INVALID_VERSION: Found: {found}, Expected: {expected}, Try the migrate command")]
    InvalidVersion { found: u16, expected: u16 },
    #[error("VAL_INVALID_CHECKSUM: file corrupted, Try the command  migrate <VERSION>")]
    InvalidChecksum,
    #[error("VAL_INVALID_STORAGE_FORMAT: Unsupported storage format {format}, Try the command  migrate <VERSION>")]
    InvalidStorageFormat { format: StorageFormat },
    #[error("VAL_INVALID_STORAGE_ENTITY: Invalid storage entity expected: {expected}")]
    InvalidStoreEntity { expected: String },
}

/// Error type for archive file (.cld)
#[derive(Debug, Error)]
pub enum FileMaintenanceError {
    #[error("SYS_IO: {0}")]
    Io(#[from] std::io::Error),
    #[error("SYS_IO: {0}")]
    Walkdir(#[from] walkdir::Error),
    #[error("DATA_NO_: {0}")]
    NoTrashSnapshot(String),
    #[error("SYS_IO: {0}")]
    InvalidTrashSnapshot(String),
    #[error("DATA_INVALID: {0}")]
    InvalidData(String),
}

/// Error type for snapshot (.snp)
#[derive(Debug, Error)]
pub enum FileSnapshotError {
    #[error("SYS_IO: {0}")]
    Io(#[from] std::io::Error),
    #[error("SYS_STORAGE: {0}")]
    Storage(#[from] StorageError),
    #[error("DATA_NO_ACCOUNT: No Account, snapshot not create")]
    NoAccount,
}

/// Error type for toml,json,csv file
#[derive(Debug, Error)]
pub enum FileExchangeError {
    #[error("SYS_IO: {0}")]
    Io(#[from] std::io::Error),
    #[error("DATA_TOML: {0}")]
    InvalidTomlDe(#[from] toml::de::Error),
    #[error("DATA_TOML: {0}")]
    InvalidTomlSer(#[from] toml::ser::Error),
    #[error("DATA_JSON: {0}")]
    InvalidJson(#[from] serde_json::Error),
    #[error("DATA_CSV: {0}")]
    InvalidCSV(#[from] csv::Error),
    #[error("DATA_EXCHANGE: {0}")]
    Exchange(#[from] ExchangeError),
    #[error("Import Error: {0}")]
    Generic(String),
    #[error("Unsupported format")]
    UnsupportedFormat,
}
