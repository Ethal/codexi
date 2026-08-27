// src/logic/codexi/settings.rs

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

use crate::core::get_config_dir;
use crate::logic::codexi::CodexiError;

const SETTINGS_FILE: &str = "codexi.cfg";
const DEFAULT_LANGUAGE: &str = "en";
const DEFAULT_CURRENCY: &str = "EUR";

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CodexiSettings {
    pub language: String,
    pub default_currency: String,
}

impl Default for CodexiSettings {
    fn default() -> Self {
        Self {
            language: DEFAULT_LANGUAGE.to_string(),
            default_currency: DEFAULT_CURRENCY.to_string(),
        }
    }
}

impl CodexiSettings {
    /// Load settings from config file, or create with defaults if absent.
    /// On creation, resolves data_dir via ProjectDirs and persists the file.
    pub fn load_or_create() -> Result<Self, CodexiError> {
        let config_dir = get_config_dir()?;
        let config_file = config_dir.join(SETTINGS_FILE);

        if config_file.exists() {
            Self::load(&config_file)
        } else {
            let settings = Self::create_defaults()?;
            settings.save(&config_file)?;
            Ok(settings)
        }
    }

    /// Persist current settings to disk.
    pub fn save(&self, path: &Path) -> Result<(), CodexiError> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self)?;
        fs::write(path, json)?;
        Ok(())
    }

    /// Save to the standard config location.
    pub fn save_default_path(&self) -> Result<(), CodexiError> {
        let config_file = get_config_dir()?.join(SETTINGS_FILE);
        self.save(&config_file)
    }

    // ----------------------------------------------------------------
    // Private
    // ----------------------------------------------------------------

    fn load(path: &Path) -> Result<Self, CodexiError> {
        let content = fs::read_to_string(path)?;
        let settings: Self = serde_json::from_str(&content)?;
        Ok(settings)
    }

    fn create_defaults() -> Result<Self, CodexiError> {
        Ok(Self::default())
    }
}
