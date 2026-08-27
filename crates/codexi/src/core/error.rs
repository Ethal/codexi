// src/core/error.rs

use thiserror::Error;

/// Error type for Core
#[derive(Debug, Error)]
pub enum CoreError {
    #[error("SYS_IO: {0}")]
    Io(#[from] std::io::Error),
    #[error("SYS_ID: {0}")]
    Id(#[from] nulid::Error),
    #[error("VAL_DECIMAL: Decimal error in field, {field}: {source}")]
    Decimal { source: rust_decimal::Error, field: String },
    #[error("VAL_INT: Number error in field, {field}: {source}")]
    Number {
        source: std::num::ParseIntError,
        field: String,
    },
    #[error("VAL_DATE:{0}")]
    InvalidStartDate(String),
    #[error("VAL_DATE: {0}")]
    InvalidEndDate(String),
    #[error("VAL_DATE: {0}")]
    InvalidDate(String),
    #[error("VAL_MONTH: {0}")]
    InvalidMonth(String),
    #[error("VAL_MONTH: {0}")]
    ErrorComputingEndOfMonth(String),
    #[error("VAL_DATE: {0}")]
    InvalidIntermediateDate(String),
    #[error("SYS_NO_DATA_DIR: {0}")]
    NoDataDirectory(String),
    #[error("SYS_NO_CONFIG_DIR: {0}")]
    NoConfigDirectory(String),
    #[error("SYS_NO_DOCUMENTS_DIR: {0}")]
    NoDocumentsDirectory(String),
    #[error("VAL_DATA: {0}")]
    InvalidData(String),
}
