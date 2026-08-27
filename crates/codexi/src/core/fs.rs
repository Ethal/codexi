// src/core/fs.rs

use dirs;
use std::fs;
use std::path::PathBuf;

use crate::core::CoreError;

const BUNDLE_ID: &str = "fr.ethal.codexi";

/// Returns the OS data directory for codexi.
/// Linux   : ~/.local/share/fr.ethal.codexi/
/// macOS   : ~/Library/Application Support/fr.ethal.codexi/
pub fn get_data_dir() -> Result<PathBuf, CoreError> {
    if let Some(data_dir) = dirs::data_local_dir() {
        let data_dir = data_dir.join(BUNDLE_ID);
        fs::create_dir_all(&data_dir)?;
        return Ok(data_dir);
    }
    Err(CoreError::NoDataDirectory(
        "Could not determine data user directory".to_string(),
    ))
}

/// Returns the OS config directory for codexi.
/// Linux   : ~/.config/fr.ethal.codexi/
/// macOS   : ~/Library/Application Support/fr.ethal.codexi/
pub fn get_config_dir() -> Result<PathBuf, CoreError> {
    if let Some(config_dir) = dirs::config_local_dir() {
        let config_dir = config_dir.join(BUNDLE_ID);
        fs::create_dir_all(&config_dir)?;
        return Ok(config_dir);
    }
    Err(CoreError::NoConfigDirectory(
        "Could not determine config user directory".to_string(),
    ))
}

/// Returns the OS config directory for codexi.
/// Linux   : ~/Documents/
/// macOS   : ~/Documents/
pub fn get_documents_dir() -> Result<PathBuf, CoreError> {
    if let Some(documents_dir) = dirs::document_dir() {
        return Ok(documents_dir);
    }
    Err(CoreError::NoConfigDirectory(
        "Could not determine Documents directory".to_string(),
    ))
}
